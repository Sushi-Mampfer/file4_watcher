use std::time::Duration;

use reqwest::{Client, Method};

use crate::{file4::File4, watcher::Watcher};

mod file4;
pub mod watcher;

#[tokio::main]
async fn main() {
    let mut watcher = Watcher::new(
        "https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&CIK=&type=4&company=&dateb=&owner=include&start=0&count=100&output=atom".to_string(),
        Duration::from_secs(30),
    );
    loop {
        if let Ok(Some(res)) = watcher.wait().await {
            for i in res {
                let client = Client::new();
                let req = client
                    .request(Method::GET, &i)
                    .header("User-Agent", "some@email.com");
                let Ok(req) = req.build() else {
                    continue;
                };
                let Ok(res) = client.execute(req).await else {
                    continue;
                };
                let Ok(content) = res.text().await else {
                    continue;
                };
                File4::new(content);
            }
        }
    }
}
