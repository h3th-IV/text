use std::io;

use actix_web::http;
use dotenvy::dotenv;
use reqwest::{Client, Method, Response};
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

    pub async fn make_request(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
    ) -> Result<reqwest::Response, io::Error> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        let mut request = self.client.request(method, url);
        request = request
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .header("Content-Type", "application/json");
        if let Some(body) = body {
            request = request.body(body);
        }
        let response = match request.send().await {
            Ok(response) => response,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Request failed: {}", e))),
        };
        Ok(response)
    }
}