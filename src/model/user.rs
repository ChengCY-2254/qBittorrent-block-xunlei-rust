use crate::model::config::Config;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct User<'a> {
    username: &'a str,
    password: &'a str,
}

impl<'a> User<'a> {
    pub fn new(username: &'a str, password: &'a str) -> Self {
        Self { username, password }
    }
}

pub trait IntoUser {
    fn to_user(&self) -> User<'_>;
}

impl IntoUser for Config {
    fn to_user(&self) -> User<'_> {
        User::new(self.username(), self.password())
    }
}
