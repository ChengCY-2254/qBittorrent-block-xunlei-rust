#[derive(Debug, Clone)]
pub struct Config {
    url: String,
    username: String,
    password: String,
    cookie: String,
}

impl Config {
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn new(url: String, username: String, password: String) -> Self {
        Self {
            url,
            username,
            password,
            cookie: String::new(),
        }
    }

    pub fn set_cookie(&mut self, cookie: String) {
        self.cookie = cookie;
    }

    pub fn cookie(&self) -> &str {
        &self.cookie
    }
}
