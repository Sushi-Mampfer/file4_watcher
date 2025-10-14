use chrono::{DateTime, NaiveDateTime};
use reqwest::{Client, Method};
use roxmltree::{Document, ParsingOptions};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, oneshot, watch},
    time::interval,
};
use url::Url;

pub struct Watcher {
    url: String,
    interval: Duration,
    stop_tx: Option<oneshot::Sender<()>>,
    change_rx: watch::Receiver<Option<Vec<String>>>,
    last_change: Arc<Mutex<NaiveDateTime>>,
}

impl Watcher {
    pub fn new(url: String, interval: Duration) -> Self {
        let (stop_tx, stop_rx) = oneshot::channel();
        let (change_tx, change_rx) = watch::channel::<Option<Vec<String>>>(None);

        let watcher = Self {
            url: url,
            interval: interval,
            stop_tx: Some(stop_tx),
            change_rx: change_rx,
            last_change: Arc::new(Mutex::new(DateTime::UNIX_EPOCH.naive_local())),
        };
        watcher.start(stop_rx, change_tx);
        watcher
    }

    fn start(&self, stop_rx: oneshot::Receiver<()>, change_tx: watch::Sender<Option<Vec<String>>>) {
        let url = self.url.clone();
        let mut interval = interval(self.interval);
        let last_time = self.last_change.clone();
        tokio::spawn(async move {
            let query = tokio::spawn(async move {
                loop {
                    interval.tick().await;
                    let client = Client::new();
                    let req = client
                        .request(Method::GET, &url)
                        .header("User-Agent", "some@email.com");
                    let Ok(req) = req.build() else {
                        continue;
                    };
                    let Ok(res) = client.execute(req).await else {
                        continue;
                    };
                    let Ok(data) = res.text().await else {
                        continue;
                    };
                    let opt = ParsingOptions {
                        ..Default::default()
                    };
                    let Ok(document) = Document::parse_with_options(&data, opt) else {
                        continue;
                    };
                    let entries: Vec<_> = document
                        .descendants()
                        .filter(|n| n.has_tag_name(("http://www.w3.org/2005/Atom", "entry")))
                        .collect();
                    let mut out = Vec::new();
                    let mut peekable = entries.iter().peekable();
                    let Some(first) = peekable.peek() else {
                        continue;
                    };
                    let Some(updated) = first
                        .children()
                        .find(|n| n.has_tag_name(("http://www.w3.org/2005/Atom", "updated")))
                    else {
                        continue;
                    };
                    let Some(updated) = updated.text() else {
                        continue;
                    };
                    let Ok(updated) = DateTime::parse_from_rfc3339(updated) else {
                        continue;
                    };
                    for i in entries {
                        match i
                            .children()
                            .find(|n| n.has_tag_name(("http://www.w3.org/2005/Atom", "title")))
                        {
                            Some(t) => {
                                let Some(t) = t.text() else {
                                    continue;
                                };
                                if !t.starts_with("4 ") {
                                    continue;
                                }
                            }
                            None => continue,
                        }
                        let Some(time) = i
                            .children()
                            .find(|n| n.has_tag_name(("http://www.w3.org/2005/Atom", "updated")))
                        else {
                            continue;
                        };
                        let Some(time) = time.text() else {
                            continue;
                        };
                        let Ok(time) = DateTime::parse_from_rfc3339(time) else {
                            continue;
                        };
                        if time.naive_local() <= *last_time.lock().await {
                            continue;
                        }
                        let Some(link) = i
                            .children()
                            .find(|n| n.has_tag_name(("http://www.w3.org/2005/Atom", "link")))
                        else {
                            continue;
                        };
                        let Some(link) = link.attribute("href") else {
                            continue;
                        };
                        let Ok(mut url) = Url::parse(link) else {
                            continue;
                        };
                        url.path_segments_mut().expect("no way lol").pop();
                        out.push(url.to_string());
                    }
                    *last_time.lock().await = updated.naive_local();
                    let _ = change_tx.send(Some(out));
                }
            });
            let stop = tokio::spawn(stop_rx);
            tokio::select! {
                _ = query => {},
                _ = stop => {}
            };
        });
    }

    pub async fn wait(
        &mut self,
    ) -> Result<Option<Vec<String>>, tokio::sync::watch::error::RecvError> {
        self.change_rx.changed().await?;
        Ok(self.change_rx.borrow().clone())
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        let _ = self.stop_tx.take().unwrap().send(());
    }
}
