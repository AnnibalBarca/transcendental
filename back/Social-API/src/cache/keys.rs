use uuid::Uuid;

pub fn user_profile(id: &Uuid) -> String {
    format!("user:profile:{}", id)
}

pub fn room_profile(id: &str) -> String {
    format!("room:profile:{}", id)
}

pub fn friend_list(id: &Uuid) -> String {
    format!("friend:list:{}", id)
}

pub fn friend_requests(id: &Uuid) -> String {
    format!("friend:requests:{}", id)
}

pub fn friend_sent(id: &Uuid) -> String {
    format!("friend:sent:{}", id)
}

pub fn friend_status(user_id: &Uuid, friend_id: &Uuid) -> String {
    format!("friend:status:{}:{}", user_id, friend_id)
}

pub fn friend_blocked(id: &Uuid) -> String {
    format!("friend:blocked:{}", id)
}

pub fn message_conv(user_id: &Uuid, friend_id: &Uuid) -> String {
    format!("msg:conv:{}:{}", user_id, friend_id)
}
