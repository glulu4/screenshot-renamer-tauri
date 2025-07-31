use std::env;
// use reqwest::Client;
use reqwest::blocking::Client;  // sync
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_with::serde_as;
use dotenvy_macro::dotenv;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Tier {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "pro")]
    Pro,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Incomplete,
    Canceled,
    PastDue,
    Trialing,
}


#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDevice {
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub system: String,
    #[serde(rename = "appVersion")]
    pub app_version: String,
    pub tier: Tier,
    #[serde(rename = "maxQuota")]
    pub max_quota: u32,
    #[serde(rename = "quotaUsed")]
    pub quota_used: u32,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(rename = "subscribedAt")]
    pub subscribed_at: Option<DateTime<Utc>>,

    pub email: Option<String>,
    #[serde(rename = "stripeCustomerId")]
    pub stripe_customer_id: Option<String>,
    #[serde(rename = "cancelAtPeriodEnd")]
    pub cancel_at_period_end: Option<bool>,
    #[serde(rename = "currentPeriodEnd")]
    pub current_period_end: Option<DateTime<Utc>>,
    #[serde(rename = "subscriptionStatus")]
    pub subscription_status: Option<SubscriptionStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponseData {
    #[serde(rename = "userDevice")]
    pub user_device: UserDevice,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    pub data: ApiResponseData,
}



pub fn register() -> Option<UserDevice> {

    // get device id
    let id = get_device_id();
    let system = std::env::consts::OS; // returns "macos", "windows", "linux", etc.
    let version = env!("CARGO_PKG_VERSION");

    println!("Device ID: {}", id);
    println!("Platform: {}", system);
    println!("Version: {}", version);


    let payload = serde_json::json!({
        "deviceId": id,
        "system": system,
        "appVersion": version,
    });

    dotenvy::dotenv().ok();

    let client = Client::new();
    let register_url = dotenv!("REGISTER_URL");


// Send a POST request to the register_url with our JSON payload
let result = client.post(register_url).json(&payload).send();

let response = match result{ 
    Ok(res) => {
        if !res.status().is_success() {
            eprintln!("❌ Server returned an error status: {}", res.status());
            return None;
        }

        res
    }

    Err(err) => {
        eprintln!("❌ Failed to send request to register device: {}", err);
        return None;
    }
};


    let api_response: ApiResponse = match response.json() {
        Ok(json) => json,
        Err(e) => {
            eprintln!("❌ Error: Failed to parse JSON response from register: {}", e);
            return None;
        }
    };

    if !api_response.success {
        eprintln!("❌ Error: API response indicates failure: {}", api_response.message);
        return None;
    }

    println!("✅ Device registered successfully: {:?}", api_response.data);

    let user_device = api_response.data.user_device;
    return Some(user_device);

}



fn get_device_id() -> String {
    machine_uid::get().unwrap_or_else(|_| "unknown-device".into())
}
