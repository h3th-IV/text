use std::io;

use dotenvy::dotenv;
use reqwest::Client;
pub struct PaystackClient {
    pub client: Client,
    pub base_url: String,
    pub bearer_token: String,
}

impl PaystackClient {
    //assoc new functiom
    pub fn new() -> Result<Self, io::Error> {
        dotenv().ok();

        let beare_token = String::from("sk_test_fa16b06664111cf77ebcd2df5d58a1110ca0dfa6");
        Ok(Self { 
            client: Client::new(), 
            base_url: "https://api.paystack.co".to_string(),
            bearer_token: beare_token
        })
    }
}