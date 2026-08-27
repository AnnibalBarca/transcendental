// MinIO client for the shop's cosmetic images, talking raw S3 HTTP + AWS
// SigV4 signing (no aws-sdk / minio crate — everything below is hand-rolled).
//
// Vocabulary, and how each maps onto this project's setup:
// - MinIO: a self-hosted, S3-API-compatible object store. We run it as our
//   own docker container instead of paying for AWS S3; from the client's
//   point of view it IS S3, because it implements the same REST API.
// - "Bucket": one flat namespace of objects, here named by MINIO_SHOP_BUCKET
//   (default "cosmetics"). One bucket holds every cosmetic image.
// - "Object" / "key": an object is a file inside the bucket; its key is its
//   path, e.g. "hat/3.png" (see object_key() below) — MinIO/S3 has no real
//   subfolders, the "/" in a key is just a naming convention.
// - "Bucket policy": a JSON document (set_public_read, below) that makes
//   every object in the bucket readable by anyone with the URL, with no
//   auth — that's what lets the front's <img> tags load cosmetic images
//   directly from MinIO without going through our own API.
// - SigV4 ("Signature Version 4"): AWS's request-signing scheme. Every
//   write (PUT a bucket, PUT an object, ...) must carry a HMAC-SHA256
//   signature computed from the request itself (method, path, headers,
//   body hash) plus a key derived from the MinIO root credentials — this
//   proves to MinIO the caller holds MINIO_ROOT_USER/PASSWORD without
//   sending them in the clear on every request. `send()`/`signing_key()`
//   implement exactly this chain by hand.
// - Two distinct endpoints, on purpose: MINIO_ENDPOINT is the address this
//   Rust service uses to reach MinIO *inside* the docker network
//   (http://minio:9000) — that's what gets signed. MINIO_PUBLIC_ENDPOINT
//   (aka IMAGE_MINIO) is the address the *browser* uses, reached through
//   nginx — that's what ends up in every image_url sent to the front. Read
//   requests (plain GET on a public-read bucket) need no signature at all,
//   which is exactly why the front can hit MinIO directly.
use hmac::{Hmac, Mac};
use log::{info, warn};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

// The 5 equip slots a cosmetic can belong to — mirrored on the front in
// front/src/features/shop/utils.ts::SHOP_SLOT_TYPES, and used as the
// item_type discriminant throughout shop_catalog / player_inventory.
pub const SLOTS: [&str; 5] = ["base", "hat", "mask", "clothes", "accessory"];

pub fn is_valid_slot(slot: &str) -> bool {
    SLOTS.contains(&slot)
}

#[derive(Clone)]
pub struct Storage {
    client: reqwest::Client,
    endpoint: String,
    public_endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
    region: String,
}

// Treats an unset AND an empty-string env var the same way (falls through
// to the default) — matters here because docker-compose can hand a service
// an empty string for a var that's simply absent from its .env file.
fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

impl Storage {
    /// Reads MinIO credentials/config from the environment (see the module
    /// doc-comment above for what each var is for). Returns None — rather
    /// than an error — when credentials are missing, because that's a
    /// legitimate deployment state: main.rs treats a None Storage as "shop
    /// image upload is disabled", not a fatal startup error.
    pub fn from_env() -> Option<Self> {
        let access_key = env_non_empty("MINIO_ROOT_USER")?;
        let secret_key = env_non_empty("MINIO_ROOT_PASSWORD")?;

        let endpoint = env_non_empty("MINIO_ENDPOINT")
            .unwrap_or_else(|| "http://minio:9000".to_string())
            .trim_end_matches('/')
            .to_string();

        let public_endpoint = env_non_empty("MINIO_PUBLIC_ENDPOINT")
            .or_else(|| env_non_empty("IMAGE_MINIO"))
            .unwrap_or_else(|| "http://localhost:9000".to_string())
            .trim_end_matches('/')
            .to_string();

        let bucket = env_non_empty("MINIO_SHOP_BUCKET").unwrap_or_else(|| "cosmetics".to_string());
        let region = env_non_empty("MINIO_REGION").unwrap_or_else(|| "us-east-1".to_string());

        Some(Self {
            client: reqwest::Client::new(),
            endpoint,
            public_endpoint,
            bucket,
            access_key,
            secret_key,
            region,
        })
    }
    
    // Not mine — added later by a teammate for a healthcheck path.
    pub async fn ping(&self) -> bool {
        match self
            .send("HEAD", &format!("/{}", self.bucket), "", &[], None)
            .await
        {
            Ok(status) => (200..300).contains(&status) || status == 404,
            Err(_) => false,
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Builds the object key ("hat/3.png") stored as shop_catalog.asset_key
    /// and used both to PUT the image and, later, to reconstruct its URL —
    /// so the DB never stores a full URL, only this stable relative key
    /// (see resolve_image_url in http/handlers/shop.rs, which does the
    /// reverse: asset_key -> public_url()).
    pub fn object_key(item_type: &str, item_id: &str, ext: &str) -> String {
        format!("{}/{}.{}", item_type, item_id, ext)
    }

    /// Turns an asset_key into a browser-loadable URL against the *public*
    /// endpoint (nginx-facing), never the internal docker one used for
    /// signing. Needs no signature: the bucket is public-read (see
    /// set_public_read), so this is a plain unauthenticated GET URL.
    pub fn public_url(&self, key: &str) -> String {
        format!("{}/{}/{}", self.public_endpoint, self.bucket, key)
    }

    /// Called once at boot (see main.rs): creates the shop bucket if it
    /// doesn't exist yet (HEAD to check, PUT to create — S3's usual
    /// idempotent "create if 404" dance), then makes it public-read so the
    /// front can load images without hitting our API at all.
    pub async fn ensure_bucket(&self) -> Result<(), String> {
        let exists = self
            .send("HEAD", &format!("/{}", self.bucket), "", &[], None)
            .await?;

        if exists == 404 {
            let created = self
                .send("PUT", &format!("/{}", self.bucket), "", &[], None)
                .await?;
            if !(200..300).contains(&created) {
                return Err(format!("bucket create failed (HTTP {})", created));
            }
            info!("[Storage] Created bucket '{}'", self.bucket);
        } else if !(200..300).contains(&exists) {
            return Err(format!("bucket head failed (HTTP {})", exists));
        }

        self.set_public_read().await
    }

    /// PUTs an S3 bucket policy (the `?policy` sub-resource) granting
    /// anonymous `s3:GetObject` on every object in the bucket. This is the
    /// single line that decides "cosmetic images are public" — anyone with
    /// a key can GET it, no cookie/token involved, which is intentional
    /// (the shop UI, and every equipped-cosmetic render, needs to load
    /// images with a bare <img src>).
    async fn set_public_read(&self) -> Result<(), String> {
        let policy = format!(
            r#"{{"Version":"2012-10-17","Statement":[{{"Effect":"Allow","Principal":{{"AWS":["*"]}},"Action":["s3:GetObject"],"Resource":["arn:aws:s3:::{}/*"]}}]}}"#,
            self.bucket
        );

        let status = self
            .send(
                "PUT",
                &format!("/{}", self.bucket),
                "policy=",
                &[("content-type", "application/json")],
                Some(policy.into_bytes()),
            )
            .await?;

        if !(200..300).contains(&status) {
            return Err(format!("bucket policy failed (HTTP {})", status));
        }
        info!("[Storage] Bucket '{}' is now public-read", self.bucket);
        Ok(())
    }

    /// Uploads one object (a signed S3 PUT), returning its public URL on
    /// success. Called from handle_upload_item with the raw decoded image
    /// bytes. Note: PUT on the same key overwrites silently — there's no
    /// existence check here, upsert semantics are the caller's choice.
    pub async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<String, String> {
        let status = self
            .send(
                "PUT",
                &format!("/{}/{}", self.bucket, key),
                "",
                &[("content-type", content_type)],
                Some(body),
            )
            .await?;

        if !(200..300).contains(&status) {
            return Err(format!("upload failed (HTTP {})", status));
        }
        Ok(self.public_url(key))
    }

    /// Signed HEAD on one object — a boolean existence check, swallowing
    /// the error (an unreachable MinIO is treated the same as "doesn't
    /// exist" by callers, just logged).
    pub async fn object_exists(&self, key: &str) -> bool {
        match self
            .send("HEAD", &format!("/{}/{}", self.bucket, key), "", &[], None)
            .await
        {
            Ok(status) => (200..300).contains(&status),
            Err(e) => {
                warn!("[Storage] HEAD {} failed: {}", key, e);
                false
            }
        }
    }

    /// The one place every S3 call goes through. Builds and sends a single
    /// SigV4-signed HTTP request against `endpoint` (the internal MinIO
    /// address) and returns just the status code — callers decide what a
    /// given status means (404 = "doesn't exist" for a HEAD, "create it"
    /// for ensure_bucket; 2xx = success everywhere).
    ///
    /// SigV4 signing, step by step (AWS's canonical algorithm, same one
    /// `aws-sdk`/`rusoto` would run for you — done by hand here):
    /// 1. Hash the body -> `payload_hash` (SHA-256 of the raw bytes, or of
    ///    an empty body for GET/HEAD). This hash goes both in a header
    ///    (`x-amz-content-sha256`) and inside the signature itself, so a
    ///    tampered body invalidates the signature.
    /// 2. Build the "canonical request": method + path + query string +
    ///    canonical (sorted, lowercased) headers + the list of signed
    ///    header names + the payload hash, newline-joined. This is the
    ///    exact byte sequence that gets hashed and signed — any proxy that
    ///    reorders headers or alters the path would break verification.
    /// 3. Build the "string to sign": the algorithm name, the request
    ///    timestamp, a "scope" (date/region/service/"aws4_request"), and
    ///    the hash of the canonical request from step 2.
    /// 4. Derive a request-specific signing key by HMAC-chaining the
    ///    secret key through date -> region -> service -> "aws4_request"
    ///    (see `signing_key`) — this is what makes the key single-use per
    ///    day/region/service instead of reusing the raw secret directly.
    /// 5. HMAC-SHA256 the string-to-sign with that key -> the signature,
    ///    sent in the `authorization` header alongside the access key id
    ///    and the list of signed headers.
    async fn send(
        &self,
        method: &str,
        path: &str,
        query: &str,
        extra_headers: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Result<u16, String> {
        let body = body.unwrap_or_default();
        let payload_hash = hex_sha256(&body);

        let now = chrono::Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let host = self
            .endpoint
            .split("://")
            .nth(1)
            .ok_or_else(|| format!("invalid MINIO_ENDPOINT '{}'", self.endpoint))?
            .to_string();

        let mut headers: Vec<(String, String)> = vec![
            ("host".to_string(), host.clone()),
            ("x-amz-content-sha256".to_string(), payload_hash.clone()),
            ("x-amz-date".to_string(), amz_date.clone()),
        ];
        for (k, v) in extra_headers {
            headers.push((k.to_lowercase(), v.to_string()));
        }
        headers.sort_by(|a, b| a.0.cmp(&b.0));

        let canonical_headers: String = headers
            .iter()
            .map(|(k, v)| format!("{}:{}\n", k, v.trim()))
            .collect();
        let signed_headers: Vec<String> = headers.iter().map(|(k, _)| k.clone()).collect();
        let signed_headers = signed_headers.join(";");

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, path, query, canonical_headers, signed_headers, payload_hash
        );

        let scope = format!("{}/{}/s3/aws4_request", date_stamp, self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amz_date,
            scope,
            hex_sha256(canonical_request.as_bytes())
        );

        let signature = hex::encode(hmac(
            &self.signing_key(&date_stamp),
            string_to_sign.as_bytes(),
        ));

        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key, scope, signed_headers, signature
        );

        let url = if query.is_empty() {
            format!("{}{}", self.endpoint, path)
        } else {
            format!("{}{}?{}", self.endpoint, path, query)
        };

        let method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|e| format!("bad method: {}", e))?;
        let mut req = self
            .client
            .request(method, &url)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", &payload_hash)
            .header("authorization", &authorization);
        for (k, v) in extra_headers {
            req = req.header(*k, *v);
        }
        if !body.is_empty() {
            req = req.body(body);
        }

        let response = req.send().await.map_err(|e| e.to_string())?;
        Ok(response.status().as_u16())
    }

    /// The HMAC key-derivation chain from SigV4 step 4 above: each level
    /// scopes the key tighter (date, then region, then "s3" service, then
    /// the literal "aws4_request" terminator) so a signature is only ever
    /// valid for one day, one region, and one service.
    fn signing_key(&self, date_stamp: &str) -> Vec<u8> {
        let k_date = hmac(
            format!("AWS4{}", self.secret_key).as_bytes(),
            date_stamp.as_bytes(),
        );
        let k_region = hmac(&k_date, self.region.as_bytes());
        let k_service = hmac(&k_region, b"s3");
        hmac(&k_service, b"aws4_request")
    }
}

// SHA-256 of `data`, hex-encoded — the plain hash primitive SigV4 uses for
// both the payload hash and the canonical-request hash.
fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// HMAC-SHA256(key, data) — the keyed hash primitive; chained repeatedly in
// signing_key() and used once more to produce the final signature in send().
fn hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Whitelist of accepted upload content-types -> file extension, used by
/// handle_upload_item both to validate the request and to name the stored
/// object (object_key). Anything not in this list (e.g. "image/avif") is
/// rejected outright rather than guessed at.
pub fn extension_for(content_type: &str) -> Result<&'static str, String> {
    match content_type {
        "image/png" => Ok("png"),
        "image/jpeg" | "image/jpg" => Ok("jpg"),
        "image/webp" => Ok("webp"),
        "image/svg+xml" => Ok("svg"),
        "image/gif" => Ok("gif"),
        other => Err(format!("unsupported image type '{}'", other)),
    }
}
