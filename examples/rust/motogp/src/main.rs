use falkordb::{AsyncGraph, FalkorAsyncClient, FalkorClientBuilder, FalkorConnectionInfo};

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

async fn delete_graph(graph: &mut AsyncGraph) -> Result<(), Box<dyn std::error::Error>> {
    graph.delete().await?;
    Ok(())
}

async fn create(graph: &mut AsyncGraph) -> Result<(), Box<dyn std::error::Error>> {
    // let _ = graph
    //     .query(
    //         r#"CREATE
    //        (:Rider {name:'Valentino Rossi'})-[:rides]->(:Team {name:'Yamaha'}),
    //        (:Rider {name:'Dani Pedrosa'})-[:rides]->(:Team {name:'Honda'}),
    //        (:Rider {name:'Andrea Dovizioso'})-[:rides]->(:Team {name:'Ducati'})"#,
    //     )
    //     .execute()
    //     .await?;

    let pairs = vec![
        ("Valentino Rossi", "Yamaha"),
        ("Dani Pedrosa", "Honda"),
        ("Andrea Dovizioso", "Ducati"),
        // ("d'Aartagnan", "Horse"),
        // Error: RedisError("Invalid input 'A': expected ';', ':', a statement option, a query hint, call clause, a clause or a schema command line: 1, column: 1, offset: 0 errCtx: Aartagnan' MERGE (r:Rider {name: $rider}) errCtxOffset: 0")
    ];
    for (rider_name, team_name) in pairs {
        let mut params = std::collections::HashMap::new();
        params.insert(String::from("rider"), format!("'{rider_name}'"));
        params.insert(String::from("team"), format!("'{team_name}'"));

        let _ = graph
            .query(
                r#"MERGE (r:Rider {name: $rider})
                   MERGE (t:Team {name: $team})
                   MERGE (r)-[:rides]->(t)"#,
            )
            .with_params(&params)
            .execute()
            .await?;
    }

    Ok(())
}

async fn get_yamaha(graph: &mut AsyncGraph) -> Result<(), Box<dyn std::error::Error>> {
    // Query which riders represent Yamaha?
    let team_name = String::from("Yamaha");
    let mut params = std::collections::HashMap::new();
    params.insert(String::from("team"), format!("'{team_name}'"));

    let mut nodes = graph
        .query(
            r#"MATCH (r:Rider)-[:rides]->(t:Team)
                 WHERE t.name = $team
                 RETURN r.name"#,
        )
        .with_params(&params)
        .execute()
        .await?;

    for node in nodes.data.by_ref() {
        println!("{:?}", node);
    }
    Ok(())
}

async fn riders(graph: &mut AsyncGraph) -> Result<(), Box<dyn std::error::Error>> {
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

async fn list_all(graph: &mut AsyncGraph) -> Result<(), Box<dyn std::error::Error>> {
    let mut res = graph
        .query(r#"MATCH (r:Rider)-[:rides]->(t:Team) RETURN r, t"#)
        .execute()
        .await?;

    for row in res.data.by_ref() {
        let rider = &row[0];
        let team = &row[1];
        let rider_name = rider
            .as_node()
            .unwrap()
            .properties
            .get("name")
            .unwrap()
            .as_string()
            .unwrap();
        let team_name = team
            .as_node()
            .unwrap()
            .properties
            .get("name")
            .unwrap()
            .as_string()
            .unwrap();
        println!("{rider_name:20}  rides {team_name}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client().await?;

    // Select the 'MotoGP' graph
    let mut graph = client.select_graph("MotoGP");

    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("Usage: {} <delete|create|yamaha|riders|list>", args[0]);
        std::process::exit(1);
    }
    let cmd = &args[1];

    match cmd as &str {
        "delete" => {
            delete_graph(&mut graph).await?;
        }
        "create" => {
            create(&mut graph).await?;
        }
        "yamaha" => {
            get_yamaha(&mut graph).await?;
        }
        "riders" => {
            riders(&mut graph).await?;
        }
        "list" => {
            list_all(&mut graph).await?;
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
    }

    Ok(())
}
