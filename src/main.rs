// I have modifield your sample above about rust reeverse proxy, it runs and it can return kind of json like from my server
// my question is in those sample you use  rust rocket `request multiple segment --> <path..>` `
// why using that ?
// and how if I want to route couple of rest API path  ? in this case I use http://103.127.133.115:8000/token_xyz/table_list
#[macro_use] extern crate rocket;

use rocket::tokio::sync::RwLock;
use rocket::http::Status;
use rocket::response::content;
use rocket::State;
use reqwest::Client;
use std::sync::Arc;
use std::path::PathBuf;
// use rocket::http::uri::Path;

struct ProxyConfig {
    client: Client,
    target: String,
}

#[get("/<path..>")]
async fn proxy(path: PathBuf, state: &State<Arc<RwLock<ProxyConfig>>>) -> Result<content::RawText<String>, Status> {
    let config = state.read().await;
    let target_path = path.to_str().unwrap(); // Get the path as a string
    let target_url = format!("{}/{}", config.target, target_path);

    println!("target url: {:?}", target_url);

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

#[launch]
fn rocket() -> _ {
    let client = Client::new();
    let config = ProxyConfig {
        client,
        target: "http://103.127.133.115:8000/token_xyz/table_list".to_string(),
    };

    rocket::build()
        .manage(Arc::new(RwLock::new(config)))
        .mount("/", routes![proxy])
}