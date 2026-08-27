

// `perm:route:{METHOD}:{path}` holds the JSON array of permission names that
// grant access to a route. `perm:user:{user_id}` holds the JSON array of the

pub fn route_permission_key(method: &str, path: &str) -> String {
    format!("perm:route:{}:{}", method.to_uppercase(), path)
}

pub fn user_permission_key(user_id: &str) -> String {
    format!("perm:user:{}", user_id)
}
