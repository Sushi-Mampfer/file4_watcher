use std::{env::var, time::Duration};

use reqwest::{Client, Method};
use serde_json::{json, to_string};
use sqlx::{SqlitePool, query};
use tokio::{fs::OpenOptions, time::sleep};

use crate::{file4::File4, watcher::Watcher};

mod file4;
mod watcher;

const PERCENTAGE: f32 = 20.0;

#[tokio::main]
async fn main() {
    let webhook: String = var("WEBHOOK").expect("No WEBHOOK env set.");
    {
        OpenOptions::new()
            .create(true)
            .write(true)
            .open("db.sqlite")
            .await
            .unwrap();
    }

    let pool = SqlitePool::connect("sqlite://db.sqlite").await.unwrap();

    query(
        r#"CREATE TABLE IF NOT EXISTS file4s ("id"	TEXT NOT NULL UNIQUE,
	"file4"	TEXT NOT NULL,
	PRIMARY KEY("id"))"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let mut watcher = Watcher::new(
        "https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&CIK=&type=4&company=&dateb=&owner=include&start=0&count=100&output=atom".to_string(),
        Duration::from_secs(30),
    );

    loop {
        if let Ok(Some(res)) = watcher.wait().await {
            println!("Received {} new file4s.", res.len());
            for i in res {
                sleep(Duration::from_millis(250)).await;
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
                let file4 = File4::new(content).unwrap();
                if query("INSERT INTO file4s (id, file4) VALUES (?, ?)")
                    .bind(file4.id.clone())
                    .bind(to_string(&file4).unwrap())
                    .execute(&pool)
                    .await
                    .is_err()
                {
                    continue;
                };
                for i in file4.non_derivative {
                    let Some(data) = i.tx_data else {
                        continue;
                    };
                    let percentage = if data.acqired {
                        data.amount / ((i.owned - data.amount) / 100.0)
                    } else {
                        data.amount / ((i.owned + data.amount) / 100.0)
                    };
                    if percentage >= PERCENTAGE {
                        let data = json!({
                            "embeds": [
                                {
                                    "author": {
                                        "name": "100% correct market advise xD",
                                        "icon_url": "https://www.descargarstickers.com/src_img/2020/05/856404.png"
                                    },
                                    "footer": {
                                        "text": "Don't trust this if you don't know what you're doing"
                                    },
                                    "color": if data.acqired { 65280 } else { 16711680 },
                                    "title": format!("{}% {}!", percentage, if data.acqired { "buy" } else { "sale" }),
                                    "url": format!("https://www.sec.gov/Archives/edgar/data/{}/{}/xslF345X05/{}", file4.reporters[0].cik, file4.id.replace("-", ""), file4.file_name.clone()),
                                    "description": format!(
                                        "{} {} {}({}%) of [{}({})](https://www.sec.gov/edgar/browse/?CIK={}){}",
                                        if file4.reporters.len() == 1 {
                                            format!(
                                                "[{}](https://www.sec.gov/edgar/browse/?CIK={})",
                                                file4.reporters[0].name,
                                                file4.reporters[0].cik
                                            )
                                        } else {
                                            format!("{} people", file4.reporters.len())
                                        },
                                        if data.acqired { "bought" } else { "sold" },
                                        data.amount,
                                        percentage,
                                        file4.issuer.name,
                                        file4.issuer.symbol,
                                        file4.issuer.cik,
                                        if let Some(date) = i.date {
                                            format!(" {}.", date)
                                        } else {
                                            ".".to_owned()
                                        }
                                    )
                                }
                            ]
                        });

                        let client = Client::new();

                        let _ = client
                            .execute(client.post(webhook.clone()).json(&data).build().unwrap())
                            .await;
                    }
                }
            }
        }
    }
}
