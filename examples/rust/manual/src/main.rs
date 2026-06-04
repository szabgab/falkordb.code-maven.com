use std::collections::HashMap;

use clap::Parser;
use falkordb::{FalkorClientBuilder, FalkorConnectionInfo, FalkorValue};

#[derive(Parser, Debug)]
#[command(version, about = "Manage the Manual FalkorDB graph")]
struct Cli {
    /// Delete the graph before applying other actions.
    #[arg(long)]
    delete: bool,

    /// Add a Person node with the given name. Can be passed multiple times.
    #[arg(long = "node", value_name = "NAME", value_parser = validate_name)]
    node_names: Vec<String>,

    /// List all nodes in the graph.
    #[arg(long)]
    nodes: bool,
}

fn validate_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("name cannot be empty".to_string());
    }

    if name.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Ok(name.to_string());
    }

    Err("name must contain only a-z, A-Z, or 0-9 characters".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Connect to FalkorDB
    let connection_info: FalkorConnectionInfo = "falkor://127.0.0.1:6379"
        .try_into()
        .expect("Invalid connection info");

    let client = FalkorClientBuilder::new_async()
        .with_connection_info(connection_info)
        .build()
        .await?;

    let mut graph = client.select_graph("Manual");

    if cli.delete {
        graph.delete().await?;
        println!("Deleted graph {}", graph.graph_name());
    }

    for name in cli.node_names {
        let mut params = HashMap::new();
        params.insert("name".to_string(), format!("'{name}'"));

        graph
            .query("CREATE (:Person {name: $name})")
            .with_params(&params)
            .execute()
            .await?;

        println!("Added Person node with name {name}");
    }

    if cli.nodes {
        let mut result = graph.query("MATCH (n) RETURN n").execute().await?;

        if result.data.is_empty() {
            println!("No nodes found");
        } else {
            while let Some(row) = result.data.next() {
                for value in row {
                    match value {
                        FalkorValue::Node(node) => println!("{node:?}"),
                        other => println!("{other:?}"),
                    }
                }
            }
        }
    }


    Ok(())
}
