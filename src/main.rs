use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::{json, Value};

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
    /// Get account info by address
    Account { address: String },
    /// Get block by height
    Block { height: u64 },
    /// Get transaction by hash
    Tx { hash: String },
    /// Get latest block info
    Latest,
    /// List deployed programs
    Programs,
}

fn rpc_call(url: &str, method: &str, params: Value) -> Result<Value> {
    let client = reqwest::blocking::Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let resp: Value = client
        .post(url)
        .json(&body)
        .send()?
        .json()?;

    if let Some(err) = resp.get("error") {
        anyhow::bail!("RPC error: {}", err);
    }

    Ok(resp["result"].clone())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rpc = &cli.rpc;

    match cli.command {
        Commands::Account { address } => {
            println!("{}", "Account Info".bold().cyan());
            println!("{}: {}", "Address".bold(), address);

            match rpc_call(rpc, "get_account", json!({"account_id": address})) {
                Ok(result) => {
                    let balance = result["balance"].as_u64().unwrap_or(0);
                    let nonce = &result["nonce"];
                    let owner = &result["program_owner"];
                    println!("{}: {}", "Balance".bold(), balance.to_string().green());
                    println!("{}: {}", "Nonce".bold(), nonce);
                    println!("{}: {}", "Program Owner".bold(), owner);
                }
                Err(e) => println!("{}: {}", "Error".red(), e),
            }
        }

        Commands::Block { height } => {
            println!("{} {}", "Block".bold().cyan(), height);

            match rpc_call(rpc, "get_block", json!({"block_id": height})) {
                Ok(result) => {
                    println!("{}: {}", "Raw".bold(), result["block"]);
                }
                Err(e) => println!("{}: {}", "Error".red(), e),
            }
        }

        Commands::Tx { hash } => {
            println!("{} {}", "Transaction".bold().cyan(), hash);

            match rpc_call(rpc, "get_transaction_by_hash", json!({"tx_hash": hash})) {
                Ok(result) => {
                    if result.is_null() {
                        println!("{}", "Transaction not found".yellow());
                    } else {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    }
                }
                Err(e) => println!("{}: {}", "Error".red(), e),
            }
        }

        Commands::Latest => {
            println!("{}", "Latest Block".bold().cyan());

            match rpc_call(rpc, "get_last_block", json!({})) {
                Ok(result) => {
                    let height = result["last_block"].as_u64().unwrap_or(0);
                    println!("{}: {}", "Height".bold(), height.to_string().green());

                    match rpc_call(rpc, "get_block", json!({"block_id": height})) {
                        Ok(block) => {
                            println!("{}: {}", "Block data".bold(), block["block"]);
                        }
                        Err(_) => {}
                    }
                }
                Err(e) => println!("{}: {}", "Error".red(), e),
            }
        }

        Commands::Programs => {
            println!("{}", "Deployed Programs".bold().cyan());

            match rpc_call(rpc, "get_program_ids", json!({})) {
                Ok(result) => {
                    if let Some(programs) = result["program_ids"].as_object() {
                        if programs.is_empty() {
                            println!("{}", "No programs deployed".yellow());
                        } else {
                            for (name, id) in programs {
                                // Convert u32 array to hex string
                                let hex = if let Some(arr) = id.as_array() {
                                    arr.iter()
                                        .filter_map(|v| v.as_u64())
                                        .map(|n| format!("{:08x}", n))
                                        .collect::<Vec<_>>()
                                        .join("")
                                } else {
                                    id.to_string()
                                };
                                println!("  {} {}", "▸".cyan(), name.bold().green());
                                println!("    {}", hex.dimmed());
                            }
                        }
                    }
                }
                Err(e) => println!("{}: {}", "Error".red(), e),
            }
        }
    }

    Ok(())
}
