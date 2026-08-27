use rand::Rng;

pub fn is_port_free(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub fn generate_random_port() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(10000..60000)
}

pub fn find_free_port() -> Option<u16> {
    for _ in 0..100 {
        let port = generate_random_port();
        if is_port_free(port) {
            return Some(port);
        }
    }
    None
}
