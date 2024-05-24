#[macro_use] extern crate rocket;
use rocket::tokio::sync::RwLock;
use rocket::http::Status;
use rocket::response::content;
use rocket::State;
use rocket::serde::json::{Value, json};
use reqwest::Client;
use std::sync::Arc;
use std::path::PathBuf;
// use rocket::http::uri::Path;
use serde::{Deserialize, Serialize};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    company: String,
    exp: u64,
}

struct ProxyConfig {
    client: Client,
    target: String,
}

// request guard
#[get("/<path..>", data = "<token>")]
async fn proxy(path: PathBuf, state: &State<Arc<RwLock<ProxyConfig>>>, token: String) -> Result<content::RawText<String>, Status> {
    
    // 1. token yg diencode akan memunculkan data link sebenarnya
    // 2. url pengecoh (walaupun pada akhirya url tsb dibuang)

    // get data from client app
    println!("====================");
    println!("{:?}", token);
    println!("====================");

    // decoding token to get the real url
    // DECODING
    let key = b"secret";
    let token_data: jsonwebtoken::TokenData<Claims> = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(key),
        &Validation::new(Algorithm::HS512),
    ) {
        Ok(c) => c,
        Err(err) => match *err.kind() {
            ErrorKind::InvalidToken => panic!(), // Example on how to handle a specific error
            _ => panic!(),
        },
    };

    println!("\n\n\n");
    println!("{:?}", token_data.claims);
    println!("{:?}", token_data.header);

    println!("the real path: {:?}", token_data.claims.sub);

    println!("\n\n\n");
    // ================================================
    
    
    
    let config = state.read().await;
    // let target_path = path.to_str().unwrap(); // Get the path as a string
    let target_url = format!("{}/{}", config.target, token_data.claims.sub); // assembly path

    println!("target url: {:?}", target_url);

    // request to correct REST API endpoint
    match config.client.get(&target_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await.unwrap_or_else(|_| "Failed to read response text".into());
                Ok(content::RawText(text))
            } else {
                Err(Status::new(response.status().as_u16()))
            }
        },
        Err(_) => Err(Status::InternalServerError),
    }
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

#[launch]
fn rocket() -> _ {
    let client = Client::new();
    let config = ProxyConfig {
        client,
        target: "http://103.127.133.115:8000".to_string(),
    };

    rocket::build()
        .manage(Arc::new(RwLock::new(config)))
        .mount("/", routes![proxy])
        .register("/", catchers![not_found])
}