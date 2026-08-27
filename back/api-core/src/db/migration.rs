pub struct Migration {
    pub name: &'static str,
    pub sql: &'static str,
}

impl Migration {
    pub const fn new(name: &'static str, sql: &'static str) -> Self {
        Self { name, sql }
    }
}
