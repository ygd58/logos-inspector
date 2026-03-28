use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose};

#[derive(Parser)]
#[command(name = "logos-inspector")]
#[command(about = "Inspect Logos blockchain state", version = "0.1.0")]
struct Cli {
    #[arg(long, default_value = "http://localhost:3040")]
    rpc: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get account balance, nonce, and program owner
    Account {
        /// Account address (base58)
        address: String,
    },
    /// Get decoded block info (id, timestamp, hash)
    Block {
        /// Block height number
        height: u64,
    },
    /// Look up a transaction by its hash
    Tx {
        /// Transaction hash (hex)
        hash: String,
    },
    /// Get latest block info
    Latest,
    /// List deployed programs
    Programs,
    /// Get a range of blocks
    Blocks {
        /// Start block height
        from: u64,
        /// End block height
        to: u64,
    },
    /// Watch new blocks in real-time
    Watch {
        /// Polling interval in seconds
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },
}

fn rpc_call(url: &str, method: &str, params: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let body = json!({"jsonrpc": "2.0", "method": method, "params": params, "id": 1});
    let resp: Value = client.post(url).json(&body).send()?.json()?;
    if let Some(err) = resp.get("error") {
        let msg = err["message"].as_str().unwrap_or("unknown error");
        let cause = err["cause"]["name"].as_str().unwrap_or("");
        anyhow::bail!("{} ({})", msg, cause);
    }
    Ok(resp["result"].clone())
}

fn format_program_id(arr: &Value) -> String {
    if let Some(arr) = arr.as_array() {
        arr.iter()
            .filter_map(|v| v.as_u64())
            .map(|n| format!("{:08x}", n))
            .collect::<Vec<_>>()
            .join("")
    } else {
        arr.to_string()
    }
}

fn decode_block_data(b64: &str) -> String {
    match general_purpose::STANDARD.decode(b64) {
        Ok(bytes) => {
            // Format: [block_id: u64 le][hash: 32 bytes][timestamp_ms: u64 le]
            if bytes.len() >= 48 {
                let block_id = u64::from_le_bytes(bytes[0..8].try_into().unwrap_or([0;8]));
                let hash_hex: String = bytes[8..40].iter().map(|b| format!("{:02x}", b)).collect();
                let timestamp_ms = u64::from_le_bytes(bytes[40..48].try_into().unwrap_or([0;8]));
                let secs = timestamp_ms / 1000;
                let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| secs.to_string());
                format!("id={} time={} hash={}...", block_id, dt, &hash_hex[..16])
            } else {
                format!("{} bytes (raw)", bytes.len())
            }
        }
        Err(_) => b64.to_string(),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rpc = &cli.rpc;

    match cli.command {
        Commands::Account { address } => {
            println!("{}", "Account Info".bold().cyan());
            println!("{}: {}", "Address".bold(), address.yellow());
            match rpc_call(rpc, "get_account", json!({"account_id": address})) {
                Ok(result) => {
                    let acc = &result["account"];
                    let balance = acc["balance"].as_u64().unwrap_or(0);
                    let nonce = acc["nonce"].as_u64().unwrap_or(0);
                    let owner = format_program_id(&acc["program_owner"]);
                    let data_len = acc["data"].as_array().map(|a| a.len()).unwrap_or(0);
                    println!("{}: {}", "Balance".bold(), balance.to_string().green());
                    println!("{}: {}", "Nonce".bold(), nonce);
                    println!("{}: {}...", "Program Owner".bold(), &owner[..16.min(owner.len())]);
                    println!("{}: {} bytes", "Data".bold(), data_len);
                }
                Err(e) => println!("{}: {}", "Error".red().bold(), e),
            }
        }

        Commands::Block { height } => {
            println!("{} {}", "Block".bold().cyan(), height.to_string().yellow());
            match rpc_call(rpc, "get_block", json!({"block_id": height})) {
                Ok(result) => {
                    let raw = result["block"].as_str().unwrap_or("");
                    let decoded = decode_block_data(raw);
                    println!("{}: {}", "Decoded".bold(), decoded);
                    println!("{}: {}...", "Raw (base64)".bold().dimmed(), &raw[..20.min(raw.len())]);
                }
                Err(e) => println!("{}: {}", "Error".red().bold(), e),
            }
        }

        Commands::Tx { hash } => {
            println!("{} {}", "Transaction".bold().cyan(), hash.yellow());
            match rpc_call(rpc, "get_transaction_by_hash", json!({"hash": hash})) {
                Ok(result) => {
                    if result.is_null() || result["transaction"].is_null() {
                        println!("{}", "Transaction not found".yellow());
                        println!("{}", "Tip: make sure the hash is correct and the tx is finalized".dimmed());
                    } else {
                        println!("{}", serde_json::to_string_pretty(&result["transaction"])?);
                    }
                }
                Err(e) => println!("{}: {}", "Error".red().bold(), e),
            }
        }

        Commands::Latest => {
            println!("{}", "Latest Block".bold().cyan());
            match rpc_call(rpc, "get_last_block", json!({"_": 0})) {
                Ok(result) => {
                    let height = result["last_block"].as_u64().unwrap_or(0);
                    println!("{}: {}", "Height".bold(), height.to_string().green().bold());
                    match rpc_call(rpc, "get_block", json!({"block_id": height})) {
                        Ok(block) => {
                            let raw = block["block"].as_str().unwrap_or("");
                            println!("{}: {}", "Block".bold(), decode_block_data(raw));
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => println!("{}: {}", "Error".red().bold(), e),
            }
        }

        Commands::Programs => {
            println!("{}", "Deployed Programs".bold().cyan());
            match rpc_call(rpc, "get_program_ids", json!({"_": 0})) {
                Ok(result) => {
                    if let Some(programs) = result["program_ids"].as_object() {
                        if programs.is_empty() {
                            println!("{}", "No programs deployed".yellow());
                        } else {
                            println!("{} programs found", programs.len().to_string().green());
                            println!();
                            for (name, id) in programs {
                                let hex = format_program_id(id);
                                println!("  {} {}", "▸".cyan(), name.bold().green());
                                println!("    {}", hex.dimmed());
                            }
                        }
                    }
                }
                Err(e) => println!("{}: {}", "Error".red().bold(), e),
            }
        }

        Commands::Blocks { from, to } => {
            println!("{} {} {} {}", "Blocks".bold().cyan(), from, "→".dimmed(), to);
            match rpc_call(rpc, "get_block_range", json!({"start_block_id": from, "end_block_id": to})) {
                Ok(result) => {
                    if let Some(blocks) = result["blocks"].as_array() {
                        println!("{} blocks", blocks.len().to_string().green());
                        for b64 in blocks {
                            let raw = b64.as_str().unwrap_or("");
                            println!("  {}", decode_block_data(raw));
                        }
                    }
                }
                Err(e) => println!("{}: {}", "Error".red().bold(), e),
            }
        }

        Commands::Watch { interval } => {
            println!("{}", "Watching Logos chain...".bold().cyan());
            println!("{}: {}s  |  Press Ctrl+C to stop", "Interval".bold(), interval);
            println!("{}", "─".repeat(50).dimmed());
            let mut last_height = 0u64;
            let mut same_count = 0u64;
            loop {
                match rpc_call(rpc, "get_last_block", json!({"_": 0})) {
                    Ok(result) => {
                        let height = result["last_block"].as_u64().unwrap_or(0);
                        if height != last_height {
                            let now = chrono::Utc::now().format("%H:%M:%S");
                            println!(
                                "{} {} {} {}",
                                format!("[{}]", now).dimmed(),
                                "▸".cyan(),
                                "Block".bold(),
                                height.to_string().green().bold()
                            );
                            last_height = height;
                            same_count = 0;
                        } else {
                            same_count += 1;
                            if same_count % 5 == 0 {
                                print!(".");
                                use std::io::Write;
                                std::io::stdout().flush().ok();
                            }
                        }
                    }
                    Err(e) => println!("{}: {}", "Error".red().bold(), e),
                }
                std::thread::sleep(std::time::Duration::from_secs(interval));
            }
        }
    }

    Ok(())
}
