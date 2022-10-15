pub struct Client {
    token: String,
    client: Discord
}

impl Client {
    pub fn new(token: &str) -> Client {
        Client {
            token,
        }
    }
}