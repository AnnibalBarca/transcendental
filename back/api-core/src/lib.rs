pub mod auth;
pub mod cache;
pub mod db;
pub mod http;
pub mod metrics;
pub mod permission;
pub mod ratelimit;
pub mod redis;
pub mod service;
pub mod sse;
pub mod types;
pub mod username;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
