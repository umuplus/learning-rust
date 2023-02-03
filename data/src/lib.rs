pub mod common;
pub mod models;
pub mod repositories;

use aws_config::{SdkConfig};
use aws_sdk_dynamodb::{Region};

pub async fn get_aws_config() -> SdkConfig {
    aws_config::from_env().region(Region::new("eu-west-1")).load().await
}
