use uuid::Uuid;

pub fn user_profile(id: &Uuid) -> String {
    format!("user:profile:{}", id)
}

pub fn room_profile(id: &str) -> String {
    format!("room:profile:{}", id)
}

pub fn room_public_list() -> String {
    "room:public:list".to_string()
}

pub fn room_join_code(code: &str) -> String {
    format!("room:join_code:{}", code)
}
