
#![warn(clone_double_ref)]

pub mod json;

use serde_json::Value;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use std::collections::HashMap;
use chrono::prelude::*;
use crate::error::*;
use actix_codec::Framed;
use awc::{ws::{Codec}, Client, BoxedSocket};

// Helper functions

pub fn url_encode_hashmap(hashmap: &HashMap<&str, &str>) -> String {
    if hashmap.is_empty() {
        return "".to_string();
    }
    let mut acc = "".to_string();
    for (name, param) in hashmap {
        acc += &(name.to_string() + "=" + param + "&");
    }
    acc.pop(); // remove the last "&"
    acc
}

pub fn get_unix_timestamp_ms() -> i64 {
    let now = Utc::now();
    let seconds: i64 = now.timestamp();
    let nanoseconds: i64 = now.nanosecond() as i64;
    (seconds * 1000) + (nanoseconds / 1000 / 1000)
}

pub fn get_unix_timestamp_us() -> i64 {
    let now = Utc::now();
    let seconds: i64 = now.timestamp();
    let nanoseconds: i64 = now.nanosecond() as i64;
    (seconds * 1000 * 1000) + (nanoseconds / 1000)
}

pub fn strip_empties(x: &mut HashMap<&str, &str>) {
    let empties: Vec<_> = x.iter()
        .filter(|&(_, &v)| v.is_empty())
        .map(|(k, _)| (*k).clone())
        .collect();
    for empty in empties {
        x.remove(&empty);
    }
}

pub fn get_json_string<'a>(json_obj: &'a Value, key: &str) -> Result<&'a str> {
    Ok(json_obj
           .get(key)
           .ok_or_else(|| ErrorKind::MissingField(key.to_string()))?
           .as_str()
           .ok_or_else(|| ErrorKind::InvalidFieldFormat(key.to_string()))?)
}

pub fn from_json_bigdecimal(json_obj: &Value, key: &str) -> Result<BigDecimal> {
    let num = json_obj
        .as_str()
        .ok_or_else(|| ErrorKind::MissingField(key.to_string()))?;

    Ok(BigDecimal::from_str(num).chain_err(|| ErrorKind::InvalidFieldFormat(key.to_string()))?)
}

pub async fn new_ws_client(url: &str) -> Framed<BoxedSocket, Codec> {
    let ssl = {
        let mut ssl = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls()).unwrap();
        let _ = ssl.set_alpn_protos(b"\x08http/1.1");
        ssl.build()
    };
    let connector = awc::Connector::new().ssl(ssl).finish();
    let client = Client::build()
        .connector(connector)
        .finish();

    let (response, framed) = client
        .ws(url)
        .connect()
        .await
        .map_err(|e| {
            println!("Error: {}", e);
        })
        .unwrap();

    println!("{:?}", response);
    framed
}
