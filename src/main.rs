use std::time::Duration;

use crate::watcher::Watcher;

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
                println!("{}", i);
            }
            println!()
        }
    }
}
