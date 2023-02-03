use std::collections::{hash_map::RandomState, HashMap};

use data::models::github_model::GithubProfile;

fn main() {
    println!("Hello, world!");

    let profile = GithubProfile::new(
        "123".to_string(),
        "name".to_string(),
        "name".to_string(),
        // Some("https://avatars.githubusercontent.com/u/123?v=4".to_string()),
        None,
    );

    let mut item: HashMap<String, String, RandomState> = HashMap::new();
    item.insert("id".to_string(), profile.id);
    item.insert("n".to_string(), profile.name);
    item.insert("e".to_string(), profile.email);
    if profile.avatar_url.is_some() {
        item.insert("img".to_string(), profile.avatar_url.unwrap());
    }
    println!("{:?}", item);
}
