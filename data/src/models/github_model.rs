use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct GithubProfile {
    #[validate(length(min = 1, max = 40))]
    pub id: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1, max = 80))]
    pub name: String,

    #[validate(url)]
    pub avatar_url: Option<String>,
}

impl GithubProfile {
    pub fn new(id: String, email: String, name: String, avatar_url: Option<String>) -> Self {
        Self {
            id,
            email,
            name,
            avatar_url,
        }
    }
}