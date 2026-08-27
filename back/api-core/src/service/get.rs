use std::future::Future;

pub async fn cached_get<T, CacheGet, CacheGetFut, DbGet, DbGetFut, CacheSet, CacheSetFut>(
    cache_get: CacheGet,
    db_get: DbGet,
    cache_set: CacheSet,
) -> Result<Option<T>, String>
where
    CacheGet: FnOnce() -> CacheGetFut,
    CacheGetFut: Future<Output = Result<Option<T>, String>>,
    DbGet: FnOnce() -> DbGetFut,
    DbGetFut: Future<Output = Result<Option<T>, String>>,
    CacheSet: FnOnce(T) -> CacheSetFut,
    CacheSetFut: Future<Output = Result<(), String>>,
    T: Clone,
{
    if let Ok(Some(item)) = cache_get().await {
        return Ok(Some(item));
    }
    let db_result = db_get().await?;
    if let Some(ref item) = db_result {
        let _ = cache_set(item.clone()).await;
    }
    Ok(db_result)
}

#[macro_export]
macro_rules! cached_get {
    ($cache_get:expr, $db_get:expr, $cache_set:expr) => {{
        $crate::service::get::cached_get(
            || $cache_get,
            || $db_get,
            |__item| async move { $cache_set(&__item).await },
        )
        .await
    }};
}
