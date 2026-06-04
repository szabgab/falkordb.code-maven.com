use falkordb::{FalkorAsyncClient, FalkorClientBuilder, FalkorConnectionInfo};

async fn get_client() -> Result<FalkorAsyncClient, Box<dyn std::error::Error>> {
    let mut hostname = std::env::var("FALKORDB").unwrap_or_else(|_| "127.0.0.1".to_string());
    if !hostname.contains(':') {
        hostname.push_str(":6379");
    }

    let connection_info: FalkorConnectionInfo = format!("falkor://{}", hostname)
        .try_into()
        .expect("Invalid connection info");

    let client = FalkorClientBuilder::new_async()
        .with_connection_info(connection_info)
        .build()
        .await?;

    Ok(client)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client().await?;

    // Select the 'MotoGP' graph
    let mut graph = client.select_graph("MotoGP");

    // Clear out this graph in case you've run this script before.
    graph.delete().await?;

    graph
        .query(
            r#"CREATE
           (:Rider {name:'Valentino Rossi'})-[:rides]->(:Team {name:'Yamaha'}),
           (:Rider {name:'Dani Pedrosa'})-[:rides]->(:Team {name:'Honda'}),
           (:Rider {name:'Andrea Dovizioso'})-[:rides]->(:Team {name:'Ducati'})"#,
        )
        .execute()
        .await?;

    // Query which riders represent Yamaha?
    let mut nodes = graph
        .query(
            r#"MATCH (r:Rider)-[:rides]->(t:Team)
                 WHERE t.name = 'Yamaha'
                 RETURN r.name"#,
        )
        .execute()
        .await?;

    for node in nodes.data.by_ref() {
        println!("{:?}", node);
    }

    // Query how many riders represent team Ducati?
    let mut nodes = graph
        .query(r#"MATCH (r:Rider)-[:rides]->(t:Team {name:'Ducati'}) RETURN count(r)"#)
        .execute()
        .await?;

    for node in nodes.data.by_ref() {
        println!("{:?}", node);
    }

    Ok(())
}
