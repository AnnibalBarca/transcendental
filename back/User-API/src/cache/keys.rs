use uuid::Uuid;

pub fn inventory(user_id: &Uuid) -> String {
    format!("user:{}:inventory", user_id)
}

pub fn profile_picture(user_id: &Uuid) -> String {
    format!("user:{}:profile_picture", user_id)
}

pub fn friend_list(user_id: &Uuid) -> String {
    format!("friend:list:{}", user_id)
}

pub fn friend_requests(user_id: &Uuid) -> String {
    format!("friend:requests:{}", user_id)
}

pub fn friend_sent(user_id: &Uuid) -> String {
    format!("friend:sent:{}", user_id)
}

pub fn friend_blocked(user_id: &Uuid) -> String {
    format!("friend:blocked:{}", user_id)
}
