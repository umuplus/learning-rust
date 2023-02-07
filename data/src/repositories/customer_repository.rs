use crate::common::helpers::now;
use crate::models::customer_model::{
    convert_to_role, extract_role, Customer, CustomerListItem, CustomerListResult,
};
use crate::models::github_model::GithubProfile;

use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::env::var;

pub async fn create_customer(ddb: &Client, customer: Customer) -> Result<(), Error> {
    let mut item = HashMap::new();
    item.insert("pk".to_string(), AttributeValue::S(customer.pk));
    item.insert("sk".to_string(), AttributeValue::S(customer.sk.to_string()));
    item.insert("n".to_string(), AttributeValue::S(customer.n));
    item.insert("e".to_string(), AttributeValue::S(customer.e));
    item.insert(
        "r".to_string(),
        AttributeValue::S(extract_role(customer.r).to_string()),
    );
    if customer.img.is_some() {
        item.insert("img".to_string(), AttributeValue::S(customer.img.unwrap()));
    }
    item.insert("dis".to_string(), AttributeValue::Bool(customer.dis));
    item.insert("ua".to_string(), AttributeValue::N(customer.ua.to_string()));

    // TODO! put lookup key into common table via transaction write

    ddb.put_item()
        .table_name(var("CUSTOMER_TABLE").unwrap())
        .set_item(Some(item))
        .send()
        .await?;
    Ok(())
}

pub async fn update_customer(ddb: &Client, profile: GithubProfile) -> Result<(), Error> {
    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert("#n".to_string(), "n".to_string());
    expression_attribute_names.insert("#img".to_string(), "img".to_string());
    expression_attribute_names.insert("#ua".to_string(), "ua".to_string());
    expression_attribute_names.insert("#dis".to_string(), "dis".to_string());

    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(":n".to_string(), AttributeValue::S(profile.name));
    expression_attribute_values.insert(":ua".to_string(), AttributeValue::N(now().to_string()));
    expression_attribute_values.insert(":dis".to_string(), AttributeValue::Bool(false));
    if profile.avatar_url.is_some() {
        expression_attribute_values.insert(
            ":img".to_string(),
            AttributeValue::S(profile.avatar_url.unwrap()),
        );
    }

    ddb.update_item()
        .table_name(var("CUSTOMER_TABLE").unwrap())
        .set_key({
            let mut key: HashMap<String, AttributeValue, RandomState> = HashMap::new();
            key.insert("pk".to_string(), AttributeValue::S(profile.id));
            key.insert("sk".to_string(), AttributeValue::S("C".to_string()));
            Some(key)
        })
        .update_expression("SET #n = :n, #img = :img, #ua = :ua")
        .condition_expression("attribute_exists(pk) AND attribute_exists(sk) AND #dis <> :dis")
        .set_expression_attribute_names(Some(expression_attribute_names))
        .set_expression_attribute_values(Some(expression_attribute_values))
        .send()
        .await?;

    Ok(())
}

async fn toggle_customer(
    ddb: &Client,
    customer_id: String,
    disabled: bool,
) -> Result<(), Error> {
    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert("#ua".to_string(), "ua".to_string());
    expression_attribute_names.insert("#dis".to_string(), "dis".to_string());

    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(":ua".to_string(), AttributeValue::N(now().to_string()));
    expression_attribute_values.insert(":dis".to_string(), AttributeValue::Bool(disabled));

    ddb.update_item()
        .table_name(var("CUSTOMER_TABLE").unwrap())
        .set_key({
            let mut key: HashMap<String, AttributeValue, RandomState> = HashMap::new();
            key.insert("pk".to_string(), AttributeValue::S(customer_id));
            key.insert("sk".to_string(), AttributeValue::S("C".to_string()));
            Some(key)
        })
        .update_expression("SET #dis = :dis, #ua = :ua")
        .condition_expression("attribute_exists(pk) AND attribute_exists(sk)")
        .set_expression_attribute_names(Some(expression_attribute_names))
        .set_expression_attribute_values(Some(expression_attribute_values))
        .send()
        .await?;

    Ok(())
}

pub async fn disable_customer(ddb: &Client, customer_id: String) -> Result<(), Error> {
    toggle_customer(ddb, customer_id, true).await
}

pub async fn enable_customer(ddb: &Client, customer_id: String) -> Result<(), Error> {
    toggle_customer(ddb, customer_id, false).await
}

pub async fn lookup_customer_id(ddb: &Client, email: String) -> Result<Option<String>, Error> {
    let pk = format!("L#{}", email);
    let result_set = ddb
        .get_item()
        .table_name(var("COMMON_TABLE").unwrap())
        .set_key({
            let mut key: HashMap<String, AttributeValue, RandomState> = HashMap::new();
            key.insert("pk".to_string(), AttributeValue::S(pk));
            key.insert("sk".to_string(), AttributeValue::S("E".to_string()));
            Some(key)
        })
        .projection_expression("id")
        .send()
        .await?;

    if let Some(item) = result_set.item() {
        return Ok(Some(item.get("id").unwrap().as_s().unwrap().to_string()));
    }
    Ok(None)
}

pub async fn get_customer(ddb: &Client, customer_id: String) -> Result<Option<Customer>, Error> {
    let result_set = ddb
        .get_item()
        .table_name(var("CUSTOMER_TABLE").unwrap())
        .set_key({
            let mut key: HashMap<String, AttributeValue, RandomState> = HashMap::new();
            key.insert("pk".to_string(), AttributeValue::S(customer_id));
            key.insert("sk".to_string(), AttributeValue::S("C".to_string()));
            Some(key)
        })
        .send()
        .await?;

    if let Some(item) = result_set.item() {
        let image = item.get("img").unwrap().as_s();
        let customer = Customer {
            pk: item.get("pk").unwrap().as_s().unwrap().to_string(),
            sk: item
                .get("sk")
                .unwrap()
                .as_s()
                .unwrap()
                .chars()
                .next()
                .unwrap(),
            n: item.get("n").unwrap().as_s().unwrap().to_string(),
            e: item.get("e").unwrap().as_s().unwrap().to_string(),
            r: convert_to_role(
                item.get("r")
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap(),
            ),
            img: if image.is_ok() {
                Some(image.unwrap().to_string())
            } else {
                None
            },
            dis: item.get("dis").unwrap().as_bool().unwrap().clone(),
            ua: item
                .get("ua")
                .unwrap()
                .as_n()
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        };
        return Ok(Some(customer));
    }
    Ok(None)
}

pub async fn list_customers(
    ddb: &Client,
    start_from: Option<HashMap<String, AttributeValue, RandomState>>,
) -> Result<CustomerListResult, Error> {
    let result_set = ddb
        .scan()
        .table_name(var("CUSTOMER_TABLE").unwrap())
        .set_exclusive_start_key(start_from)
        .send()
        .await?;

    let mut results = CustomerListResult::new(Vec::new(), result_set.last_evaluated_key);
    if let Some(items) = result_set.items {
        for item in items {
            let item = CustomerListItem {
                pk: item.get("pk").unwrap().as_s().unwrap().to_string(),
                sk: item
                    .get("sk")
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap(),
                n: item.get("n").unwrap().as_s().unwrap().to_string(),
                dis: item.get("dis").unwrap().as_bool().unwrap().clone(),
                ua: item.get("ua").unwrap().as_n().unwrap().parse().unwrap(),
            };
            results.customers.push(item);
        }
    }
    Ok(results)
}
