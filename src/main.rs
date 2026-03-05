use chrono::Local;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, header};
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Clone)]
struct Tophub {
    base_url: String,
    api_key: String,
    client: Client,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum DumpFormat {
    Csv,
    Json,
    Jsonl,
}

fn timestamp_filename(ext: &str) -> String {
    format!("{}.{}", Local::now().format("%Y-%m-%d-%H-%M-%S"), ext)
}

fn value_to_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn dump_node_items(format: DumpFormat, items: &[Value]) -> Result<String, Box<dyn std::error::Error>> {
    let output = match format {
        DumpFormat::Csv => {
            let path = timestamp_filename("csv");
            let mut writer = csv::Writer::from_path(&path)?;
            writer.write_record([
                "hashid",
                "name",
                "display",
                "domain",
                "logo",
                "latest_update_timestamp",
                "rank",
                "title",
                "description",
                "url",
                "extra",
                "thumbnail",
                "time",
            ])?;

            for item in items {
                let input_hashid = item
                    .get("hashid")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let response = item.get("data").unwrap_or(&Value::Null);
                let node_data = response.get("data").unwrap_or(response);

                let hashid = node_data
                    .get("hashid")
                    .and_then(Value::as_str)
                    .unwrap_or(input_hashid)
                    .to_string();
                let name = node_data
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let display = node_data
                    .get("display")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let domain = node_data
                    .get("domain")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let logo = node_data
                    .get("logo")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let latest_update_timestamp = node_data
                    .get("latest_update_timestamp")
                    .map(value_to_text)
                    .unwrap_or_default();

                if let Some(hots) = node_data.get("items").and_then(Value::as_array) {
                    for hot in hots {
                        writer.write_record([
                            hashid.as_str(),
                            name.as_str(),
                            display.as_str(),
                            domain.as_str(),
                            logo.as_str(),
                            latest_update_timestamp.as_str(),
                            &hot.get("rank").map(value_to_text).unwrap_or_default(),
                            &hot.get("title").map(value_to_text).unwrap_or_default(),
                            &hot.get("description").map(value_to_text).unwrap_or_default(),
                            &hot.get("url").map(value_to_text).unwrap_or_default(),
                            &hot.get("extra").map(value_to_text).unwrap_or_default(),
                            &hot.get("thumbnail").map(value_to_text).unwrap_or_default(),
                            &hot.get("time").map(value_to_text).unwrap_or_default(),
                        ])?;
                    }
                }
            }

            writer.flush()?;
            path
        }
        DumpFormat::Json => {
            let path = timestamp_filename("json");
            let file = File::create(&path)?;
            let mut writer = BufWriter::new(file);
            let payload = serde_json::json!({ "items": items });
            writer.write_all(serde_json::to_string_pretty(&payload)?.as_bytes())?;
            writer.write_all(b"\n")?;
            writer.flush()?;
            path
        }
        DumpFormat::Jsonl => {
            let path = timestamp_filename("jsonl");
            let file = File::create(&path)?;
            let mut writer = BufWriter::new(file);
            for item in items {
                writeln!(writer, "{}", serde_json::to_string(item)?)?;
            }
            writer.flush()?;
            path
        }
    };

    Ok(output)
}

impl Tophub {
    fn new(api_key: impl Into<String>) -> Result<Self, reqwest::Error> {
        let client = Client::builder().build()?;
        Ok(Self {
            base_url: "https://api.tophubdata.com".to_string(),
            api_key: api_key.into(),
            client,
        })
    }

    async fn nodes(&self, p: u32) -> Result<Value, reqwest::Error> {
        self.call("/nodes", &[("p", p.to_string())]).await
    }

    async fn node(&self, hashid: &str) -> Result<Value, reqwest::Error> {
        self.call(&format!("/nodes/{hashid}"), &[]).await
    }

    async fn node_historys(&self, hashid: &str, date: &str) -> Result<Value, reqwest::Error> {
        self.call(
            &format!("/nodes/{hashid}/historys"),
            &[("date", date.to_string())],
        )
        .await
    }

    async fn search(&self, q: &str, p: u32, hashid: &str) -> Result<Value, reqwest::Error> {
        self.call(
            "/search",
            &[
                ("q", q.to_string()),
                ("p", p.to_string()),
                ("hashid", hashid.to_string()),
            ],
        )
        .await
    }

    async fn hot(&self, date: &str) -> Result<Value, reqwest::Error> {
        self.call("/hot", &[("date", date.to_string())]).await
    }

    async fn snapshots(
        &self,
        hashid: &str,
        date: Option<&str>,
        details: Option<u8>,
    ) -> Result<Value, reqwest::Error> {
        let mut params = Vec::new();
        if let Some(date) = date {
            params.push(("date", date.to_string()));
        }
        if let Some(details) = details {
            params.push(("details", details.to_string()));
        }
        self.call(&format!("/nodes/{hashid}/snapshots"), &params).await
    }

    async fn snapshot(&self, hashid: &str, ssid: u64) -> Result<Value, reqwest::Error> {
        self.call(&format!("/nodes/{hashid}/snapshots/{ssid}"), &[])
            .await
    }

    async fn calendar_events(
        &self,
        mode: &str,
        date: Option<&str>,
        categories: Option<&str>,
    ) -> Result<Value, reqwest::Error> {
        let mut params = vec![("mode", mode.to_string())];
        if let Some(date) = date {
            params.push(("date", date.to_string()));
        }
        if let Some(categories) = categories {
            params.push(("categories", categories.to_string()));
        }
        self.call("/calendar/events", &params).await
    }

    async fn call(
        &self,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> Result<Value, reqwest::Error> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request = self
            .client
            .get(url)
            .header(header::AUTHORIZATION, self.api_key.clone());

        if !params.is_empty() {
            request = request.query(params);
        }

        let response = request.send().await?.error_for_status()?;
        response.json::<Value>().await
    }
}

#[derive(Parser, Debug)]
#[command(name = "tophub-cli")]
#[command(about = "Tophub API command line client")]
#[command(arg_required_else_help = true)]
#[command(after_help = "Examples:\n  tophub-cli --apikey <KEY> nodes -p 1\n  tophub-cli nodes --dumpall\n  tophub-cli node mproPpoq6O\n  tophub-cli node mproPpoq6O,KqndgxeLl9\n  tophub-cli node mproPpoq6O,KqndgxeLl9 --dump jsonl\n  tophub-cli node-historys mproPpoq6O 2023-01-01\n  tophub-cli search 苹果 -p 1 --hashid mproPpoq6O\n  tophub-cli hot --date 2023-11-04\n  tophub-cli snapshots mproPpoq6O --date 2025-11-11 --details 1\n  tophub-cli snapshot mproPpoq6O 12345\n  tophub-cli calendar-events --mode week --date 2023-11-04 --categories 1,2,3\n  tophub-cli batch --p 1 --hashid mproPpoq6O --date 2023-01-01 --q 苹果")]
struct Cli {
    #[arg(long, global = true, value_name = "KEY", help = "Tophub API key, higher priority than TOPHUB_APIKEY in .env")]
    apikey: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "获取全部榜单列表")]
    Nodes {
        #[arg(short, long, default_value_t = 1, help = "页码，默认 1，每页 100 条")]
        p: u32,
        #[arg(long, help = "从 p=1 拉取到 p=100，并将所有榜单逐行写入 nodes.jsonl")]
        dumpall: bool,
    },
    #[command(about = "获取单个或多个榜单最新详细内容")]
    Node {
        #[arg(help = "榜单 hashid，支持逗号分隔多个值，例如 mproPpoq6O,KqndgxeLl9")]
        hashid: String,
        #[arg(long, value_enum, help = "导出结果格式：csv/json/jsonl")]
        dump: Option<DumpFormat>,
    },
    #[command(about = "获取单个榜单历史数据集")]
    NodeHistorys {
        #[arg(help = "榜单 hashid，例如 mproPpoq6O")]
        hashid: String,
        #[arg(help = "日期，格式 YYYY-MM-DD")]
        date: String,
    },
    #[command(about = "全网热点内容搜索")]
    Search {
        #[arg(help = "搜索关键词")]
        q: String,
        #[arg(short, long, default_value_t = 1, help = "页码，默认 1")]
        p: u32,
        #[arg(long, default_value = "", help = "可选：限定某个榜单 hashid")]
        hashid: String,
    },
    #[command(about = "获取今日热榜榜中榜")]
    Hot {
        #[arg(long, value_name = "YYYY-MM-DD", help = "日期，格式 YYYY-MM-DD")]
        date: String,
    },
    #[command(about = "获取某榜单的快照列表或快照详情集合")]
    Snapshots {
        #[arg(help = "榜单 hashid，例如 mproPpoq6O")]
        hashid: String,
        #[arg(long, value_name = "YYYY-MM-DD", help = "可选：日期，默认当天")]
        date: Option<String>,
        #[arg(long, help = "可选：0=仅快照列表(免费)，1=包含详细内容(收费)")]
        details: Option<u8>,
    },
    #[command(about = "获取单个榜单的指定快照详情")]
    Snapshot {
        #[arg(help = "榜单 hashid，例如 mproPpoq6O")]
        hashid: String,
        #[arg(help = "快照 ID (ssid)")]
        ssid: u64,
    },
    #[command(about = "获取热点日历事件")]
    CalendarEvents {
        #[arg(long, default_value = "day", help = "模式：day/week/month")]
        mode: String,
        #[arg(long, value_name = "YYYY-MM-DD", help = "可选：日期，默认当天")]
        date: Option<String>,
        #[arg(long, help = "可选：分类 ID，多个用逗号分隔，或 all")]
        categories: Option<String>,
    },
    #[command(about = "并发请求多个接口 (nodes/node/node-historys/search)")]
    Batch {
        #[arg(long, default_value_t = 1, help = "页码，默认 1")]
        p: u32,
        #[arg(long, help = "榜单 hashid，例如 mproPpoq6O")]
        hashid: String,
        #[arg(long, value_name = "YYYY-MM-DD", help = "日期，格式 YYYY-MM-DD")]
        date: String,
        #[arg(long, help = "搜索关键词")]
        q: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let api_key = cli.apikey.or_else(|| env::var("TOPHUB_APIKEY").ok()).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Missing API key: use --apikey <KEY> or set TOPHUB_APIKEY in .env",
        )
    })?;

    let tophub = Tophub::new(api_key)?;

    match cli.command {
        Commands::Nodes { p, dumpall } => {
            if dumpall {
                let output_path = timestamp_filename("jsonl");
                let file = File::create(&output_path)?;
                let mut writer = BufWriter::new(file);
                let mut total_items = 0usize;

                for page in 1..=100 {
                    let result = tophub.nodes(page).await?;
                    if let Some(items) = result.get("data").and_then(Value::as_array) {
                        for item in items {
                            writeln!(writer, "{}", serde_json::to_string(item)?)?;
                            total_items += 1;
                        }
                    }
                }

                writer.flush()?;
                println!(
                    "dump completed: {total_items} records written to {output_path} (p=1..100)"
                );
            } else {
                let result = tophub.nodes(p).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }
        Commands::Node { hashid, dump } => {
            let hashids: Vec<String> = hashid
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect();

            if hashids.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "hashid is empty, please provide at least one hashid",
                )
                .into());
            }

            if hashids.len() == 1 {
                let hid = hashids[0].clone();
                let result = tophub.node(&hid).await?;
                if let Some(format) = dump {
                    let items = vec![serde_json::json!({
                        "hashid": hid,
                        "data": result
                    })];
                    let path = dump_node_items(format, &items)?;
                    println!("dump completed: {} records written to {}", items.len(), path);
                } else {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
            } else {
                let mut set = tokio::task::JoinSet::new();
                let total = hashids.len();
                let pb = ProgressBar::new(total as u64);
                pb.set_style(
                    ProgressStyle::with_template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                    )?
                    .progress_chars("=>-"),
                );
                pb.set_message("fetching nodes");

                for (index, hid) in hashids.into_iter().enumerate() {
                    let client = tophub.clone();
                    set.spawn(async move { (index, hid.clone(), client.node(&hid).await) });
                }

                let mut ordered: Vec<Option<Value>> = vec![None; total];
                while let Some(joined) = set.join_next().await {
                    let (index, hid, resp) = joined?;
                    ordered[index] = Some(serde_json::json!({
                        "hashid": hid,
                        "data": resp?
                    }));
                    pb.inc(1);
                }
                pb.finish_and_clear();

                let items: Vec<Value> = ordered
                    .into_iter()
                    .map(|item| {
                        item.ok_or_else(|| {
                            std::io::Error::other("failed to build ordered node results")
                        })
                    })
                    .collect::<Result<_, _>>()?;

                if items.len() != total {
                    return Err(std::io::Error::other("node result count mismatch").into());
                }

                if let Some(format) = dump {
                    let path = dump_node_items(format, &items)?;
                    println!("dump completed: {} records written to {}", items.len(), path);
                } else {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({ "items": items }))?
                    );
                }
            }
        }
        Commands::NodeHistorys { hashid, date } => {
            let result = tophub.node_historys(&hashid, &date).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Search { q, p, hashid } => {
            let result = tophub.search(&q, p, &hashid).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Hot { date } => {
            let result = tophub.hot(&date).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Snapshots {
            hashid,
            date,
            details,
        } => {
            if let Some(d) = details {
                if d != 0 && d != 1 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--details only supports 0 or 1",
                    )
                    .into());
                }
            }
            let result = tophub
                .snapshots(&hashid, date.as_deref(), details)
                .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Snapshot { hashid, ssid } => {
            let result = tophub.snapshot(&hashid, ssid).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::CalendarEvents {
            mode,
            date,
            categories,
        } => {
            let mode = mode.to_lowercase();
            if mode != "day" && mode != "week" && mode != "month" {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "--mode only supports day/week/month",
                )
                .into());
            }
            let result = tophub
                .calendar_events(&mode, date.as_deref(), categories.as_deref())
                .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Batch {
            p,
            hashid,
            date,
            q,
        } => {
            // 并发请求四个接口，减少总等待时间。
            let (nodes, node, node_historys, search) = tokio::join!(
                tophub.nodes(p),
                tophub.node(&hashid),
                tophub.node_historys(&hashid, &date),
                tophub.search(&q, p, &hashid)
            );

            let output = serde_json::json!({
                "nodes": nodes?,
                "node": node?,
                "node_historys": node_historys?,
                "search": search?
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}
