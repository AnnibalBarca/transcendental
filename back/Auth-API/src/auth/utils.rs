use axum::http::HeaderMap;

pub fn get_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token.to_string())
}

pub fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').map(|c| c.trim()).find_map(|cookie| {
                let mut parts = cookie.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next()?;
                if key == name {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        })
}
