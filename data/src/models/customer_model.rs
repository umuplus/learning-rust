use std::collections::HashMap;

use crate::common::helpers::{now, since};

use aws_sdk_dynamodb::model::AttributeValue;
use serde::Deserialize;
use validator::Validate;

pub static PROJECTION_FIELDS_FOR_CUSTOMER: [&str; 5] = ["pk", "sk", "n", "dis", "ua"];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum CustomerRole {
    ADMIN,
    CUSTOMER,
}

pub fn extract_role(role: CustomerRole) -> char {
    match role {
        CustomerRole::ADMIN => 'A',
        CustomerRole::CUSTOMER => 'C',
    }
}

pub fn convert_to_role(role: char) -> CustomerRole {
    match role {
        'A' => CustomerRole::ADMIN,
        _ => CustomerRole::CUSTOMER,
    }
}

fn is_admin(role: CustomerRole) -> bool {
    match role {
        CustomerRole::ADMIN => true,
        _ => false,
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct Customer {
    #[validate(length(min = 1, max = 40))]
    pub pk: String, // * <customer-id>

    pub sk: char, // * C

    #[validate(length(min = 1, max = 80))]
    pub n: String, // * name

    #[validate(email)]
    pub e: String, // * email

    pub r: CustomerRole, // * role

    #[validate(url)]
    pub img: Option<String>,

    pub dis: bool, // * is disabled

    pub ua: u64, // * updated at in milliseconds
}

impl Customer {
    pub fn new(id: String, name: String, email: String, image: Option<String>) -> Self {
        Self {
            pk: id,
            sk: 'C',
            n: name,
            e: email,
            r: CustomerRole::CUSTOMER,
            img: image,
            dis: false,
            ua: now(),
        }
    }

    pub fn is_admin(&self) -> bool {
        is_admin(self.r)
    }

    pub fn since(&self) -> u64 {
        since(self.ua)
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomerListItem {
    pub pk: String, // * <customer-id>
    pub sk: char,   // * C
    pub n: String,  // * name
    pub dis: bool,  // * is disabled
    pub ua: u64,    // * updated at in milliseconds
}

impl CustomerListItem {
    pub fn since(&self) -> u64 {
        since(self.ua)
    }
}

#[derive(Debug)]
pub struct CustomerListResult {
    pub customers: Vec<CustomerListItem>,
    pub last_key: Option<HashMap<String, AttributeValue>>,
}

impl CustomerListResult {
    pub fn new(
        customers: Vec<CustomerListItem>,
        last_key: Option<HashMap<String, AttributeValue>>,
    ) -> Self {
        Self {
            customers,
            last_key,
        }
    }
}
