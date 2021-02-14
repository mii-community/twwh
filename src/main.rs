use dotenv::dotenv;
use reqwest::header::*;
use futures_util::StreamExt;
use chrono::Utc;
use json_flex;

fn from_env(name: &str) -> String {
    match std::env::var(name) {
        Ok(val) => val,
        Err(err) => {
            println!("{}: {}", err, name);
            std::process::exit(1);
        }
    }
}

async fn post_rules(map: HeaderMap) {
    let client = reqwest::Client::new();
    let endpoint = "https://api.twitter.com/2/tweets/search/stream/rules";
    let response = client.post(endpoint)
        .bearer_auth(from_env("BEARER_TOKEN"))
        .headers(map)
        .body(from_env("TRACK"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{}", response);
}

async fn get_rules(map:HeaderMap) {
    let endpoint = "https://api.twitter.com/2/tweets/search/stream/rules";
    let response = reqwest::Client::new()
        .request(reqwest::Method::GET, endpoint)
        .bearer_auth(from_env("BEARER_TOKEN"))
        .headers(map)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{}", response);
}

async fn del_rules(map:HeaderMap) {
    let client = reqwest::Client::new();
    let endpoint = "https://api.twitter.com/2/tweets/search/stream/rules";
    let response = client.post(endpoint)
        .bearer_auth(from_env("BEARER_TOKEN"))
        .headers(map)
        .body(from_env("DELETE"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{}", response);
}

async fn filtered_stream(map:HeaderMap) {
    let endpoint = "https://api.twitter.com/2/tweets/search/stream";
    let mut stream = reqwest::Client::new()
        .request(reqwest::Method::GET, endpoint)
        .bearer_auth(from_env("BEARER_TOKEN"))
        .headers(map)
        .query(&[("expansions", "author_id"), ("user.fields", "username")])
        .send()
        .await
        .unwrap()
        .bytes_stream();
    while let Some(item) = stream.next().await {
        if let Ok(i) = item {
            let converted: String = String::from_utf8(i.to_vec()).unwrap();
            if converted != "\r\n" {
                let jf = json_flex::decode(converted);
                println!("Chunk {}: {:?}", Utc::now().format("%H:%M:%S%.9f").to_string(), jf["data"]["id"]);
                webhook(format!("https://twitter.com/{}/status/{}"
                , jf["includes"]["users"][0]["username"].unwrap_string().to_string()
                , jf["data"]["id"].unwrap_string().to_string()))
                .await;
            }
        }
    }
}

async fn webhook(content: String) {
    println!("sending... {}", from_env("WEBHOOK"));
    let client = reqwest::Client::new();
    let endpoint = from_env("WEBHOOK");
    let mut header = HeaderMap::new();
    header.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let params = [("content", content)];
    let response = client.post(&endpoint)
        .headers(header)
        .query(&[("wait", "true")])
        .form(&params)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{}", response);
}

fn main() {
    dotenv().ok();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut map = HeaderMap::new();
    map.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let _task = async {
        let _mode = from_env("MODE");
        if let _mode = "post".to_string() {
            post_rules(map).await; 
        } else if let _mode = "get".to_string() {
            get_rules(map).await; 
        } else if let _mode = "delete".to_string() {
            del_rules(map).await;
        } else if let _mode = "stream".to_string() {
            filtered_stream(map).await;
        }
    };
    rt.block_on(_task);
}