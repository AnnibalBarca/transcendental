use std::future::Future;

pub async fn cached_update<DbUpdate, DbUpdateFut, CacheInvalidate, CacheInvalidateFut>(
    db_update: DbUpdate,
    cache_invalidate: CacheInvalidate,
) -> Result<(), String>
where
    DbUpdate: FnOnce() -> DbUpdateFut,
    DbUpdateFut: Future<Output = Result<(), String>>,
    CacheInvalidate: FnOnce() -> CacheInvalidateFut,
    CacheInvalidateFut: Future<Output = Result<(), String>>,
{
    db_update().await?;
    cache_invalidate().await?;
    Ok(())
}

#[macro_export]
macro_rules! cached_update {
    ($db_update:expr, $cache_invalidate:expr) => {{ $crate::service::set::cached_update(|| $db_update, || $cache_invalidate).await }};
}
