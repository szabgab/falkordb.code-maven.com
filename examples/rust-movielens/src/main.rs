use clap::Parser;
use csv::ReaderBuilder;
use falkordb::{AsyncGraph, FalkorClientBuilder, FalkorConnectionInfo};
use serde::Deserialize;
use std::{error::Error, path::Path};

const DEFAULT_FALKORDB_URL: &str = "falkor://127.0.0.1:6379";
const GRAPH_NAME: &str = "Movielens";
const DATA_DIR: &str = "ml-latest-small";
const MOVIE_BATCH_SIZE: usize = 250;
const LINK_BATCH_SIZE: usize = 500;
const RATING_BATCH_SIZE: usize = 500;
const TAG_BATCH_SIZE: usize = 500;

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Delete the MovieLens graph from FalkorDB.
    #[arg(long)]
    delete: bool,

    /// Load the MovieLens CSV files into FalkorDB.
    #[arg(long)]
    load: bool,
}

#[derive(Debug, Deserialize)]
struct MovieRecord {
    #[serde(rename = "movieId")]
    movie_id: u64,
    title: String,
    genres: String,
}

#[derive(Debug, Deserialize)]
struct LinkRecord {
    #[serde(rename = "movieId")]
    movie_id: u64,
    #[serde(rename = "imdbId")]
    imdb_id: String,
    #[serde(rename = "tmdbId")]
    tmdb_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RatingRecord {
    #[serde(rename = "userId")]
    user_id: u64,
    #[serde(rename = "movieId")]
    movie_id: u64,
    rating: f64,
    timestamp: i64,
}

#[derive(Debug, Deserialize)]
struct TagRecord {
    #[serde(rename = "userId")]
    user_id: u64,
    #[serde(rename = "movieId")]
    movie_id: u64,
    tag: String,
    timestamp: i64,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();

    if cli.delete || cli.load {
        let connection_info: FalkorConnectionInfo = DEFAULT_FALKORDB_URL.try_into()?;
        let client = FalkorClientBuilder::new_async()
            .with_connection_info(connection_info)
            .build()
            .await?;

        let mut graph = client.select_graph(GRAPH_NAME);

        if cli.delete {
            graph.delete().await?;
            println!("Deleted the {GRAPH_NAME} graph.");
        }

        if cli.load {
            let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(DATA_DIR);
            load_dataset(&mut graph, &data_dir).await?;
            println!("Loaded MovieLens data into the {GRAPH_NAME} graph.");
        }
    }

    Ok(())
}

async fn load_dataset(graph: &mut AsyncGraph, data_dir: &Path) -> AppResult<()> {
    load_movies(graph, &data_dir.join("movies.csv")).await?;
    load_links(graph, &data_dir.join("links.csv")).await?;
    load_ratings(graph, &data_dir.join("ratings.csv")).await?;
    load_tags(graph, &data_dir.join("tags.csv")).await?;
    Ok(())
}

async fn load_movies(graph: &mut AsyncGraph, path: &Path) -> AppResult<()> {
    let mut reader = ReaderBuilder::new().from_path(path)?;
    let mut batch = Vec::with_capacity(MOVIE_BATCH_SIZE);
    let mut imported = 0usize;

    for record in reader.deserialize() {
        batch.push(record?);
        if batch.len() == MOVIE_BATCH_SIZE {
            imported += flush_movies(graph, &batch).await?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        imported += flush_movies(graph, &batch).await?;
    }

    println!("Imported {imported} movies.");
    Ok(())
}

async fn load_links(graph: &mut AsyncGraph, path: &Path) -> AppResult<()> {
    let mut reader = ReaderBuilder::new().from_path(path)?;
    let mut batch = Vec::with_capacity(LINK_BATCH_SIZE);
    let mut imported = 0usize;

    for record in reader.deserialize() {
        batch.push(record?);
        if batch.len() == LINK_BATCH_SIZE {
            imported += flush_links(graph, &batch).await?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        imported += flush_links(graph, &batch).await?;
    }

    println!("Imported {imported} movie links.");
    Ok(())
}

async fn load_ratings(graph: &mut AsyncGraph, path: &Path) -> AppResult<()> {
    let mut reader = ReaderBuilder::new().from_path(path)?;
    let mut batch = Vec::with_capacity(RATING_BATCH_SIZE);
    let mut imported = 0usize;

    for record in reader.deserialize() {
        batch.push(record?);
        if batch.len() == RATING_BATCH_SIZE {
            imported += flush_ratings(graph, &batch).await?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        imported += flush_ratings(graph, &batch).await?;
    }

    println!("Imported {imported} ratings.");
    Ok(())
}

async fn load_tags(graph: &mut AsyncGraph, path: &Path) -> AppResult<()> {
    let mut reader = ReaderBuilder::new().from_path(path)?;
    let mut batch = Vec::with_capacity(TAG_BATCH_SIZE);
    let mut imported = 0usize;

    for record in reader.deserialize() {
        batch.push(record?);
        if batch.len() == TAG_BATCH_SIZE {
            imported += flush_tags(graph, &batch).await?;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        imported += flush_tags(graph, &batch).await?;
    }

    println!("Imported {imported} tags.");
    Ok(())
}

async fn flush_movies(graph: &mut AsyncGraph, batch: &[MovieRecord]) -> AppResult<usize> {
    let rows = batch
        .iter()
        .map(|record| {
            let genres = genres_literal(&record.genres);
            format!(
                "{{movie_id: {}, title: {}, genres: {genres}}}",
                record.movie_id,
                cypher_string(&record.title),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "UNWIND [{rows}] AS row \
         MERGE (m:Movie {{movie_id: row.movie_id}}) \
         SET m.title = row.title, m.genres = row.genres"
    );

    graph.query(query).execute().await?;
    Ok(batch.len())
}

async fn flush_links(graph: &mut AsyncGraph, batch: &[LinkRecord]) -> AppResult<usize> {
    let rows = batch
        .iter()
        .map(|record| {
            format!(
                "{{movie_id: {}, imdb_id: {}, tmdb_id: {}}}",
                record.movie_id,
                cypher_string(&record.imdb_id),
                cypher_optional_u64(record.tmdb_id),
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "UNWIND [{rows}] AS row \
         MATCH (m:Movie {{movie_id: row.movie_id}}) \
         SET m.imdb_id = row.imdb_id, m.tmdb_id = row.tmdb_id"
    );

    graph.query(query).execute().await?;
    Ok(batch.len())
}

async fn flush_ratings(graph: &mut AsyncGraph, batch: &[RatingRecord]) -> AppResult<usize> {
    let rows = batch
        .iter()
        .map(|record| {
            format!(
                "{{user_id: {}, movie_id: {}, rating: {}, timestamp: {}}}",
                record.user_id, record.movie_id, record.rating, record.timestamp,
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "UNWIND [{rows}] AS row \
         MERGE (u:User {{user_id: row.user_id}}) \
         WITH u, row \
         MATCH (m:Movie {{movie_id: row.movie_id}}) \
         MERGE (u)-[r:RATED]->(m) \
         SET r.rating = row.rating, r.timestamp = row.timestamp"
    );

    graph.query(query).execute().await?;
    Ok(batch.len())
}

async fn flush_tags(graph: &mut AsyncGraph, batch: &[TagRecord]) -> AppResult<usize> {
    let rows = batch
        .iter()
        .map(|record| {
            format!(
                "{{user_id: {}, movie_id: {}, tag: {}, timestamp: {}}}",
                record.user_id,
                record.movie_id,
                cypher_string(&record.tag),
                record.timestamp,
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "UNWIND [{rows}] AS row \
         MERGE (u:User {{user_id: row.user_id}}) \
         WITH u, row \
         MATCH (m:Movie {{movie_id: row.movie_id}}) \
         MERGE (u)-[:TAGGED {{tag: row.tag, timestamp: row.timestamp}}]->(m)"
    );

    graph.query(query).execute().await?;
    Ok(batch.len())
}

fn cypher_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("'{escaped}'")
}

fn cypher_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "NULL".to_string())
}

fn genres_literal(genres: &str) -> String {
    let values = if genres == "(no genres listed)" {
        Vec::new()
    } else {
        genres.split('|').map(cypher_string).collect::<Vec<_>>()
    };

    format!("[{}]", values.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn parses_load_flag() {
        let cli = Cli::parse_from(["movielens", "--load"]);
        assert!(cli.load);
        assert!(!cli.delete);
    }

    #[test]
    fn parses_delete_flag() {
        let cli = Cli::parse_from(["movielens", "--delete"]);
        assert!(cli.delete);
        assert!(!cli.load);
    }

    #[test]
    fn command_defines_load_and_delete_flags() {
        let command = Cli::command();
        assert!(
            command
                .get_arguments()
                .any(|arg| arg.get_long() == Some("load"))
        );
        assert!(
            command
                .get_arguments()
                .any(|arg| arg.get_long() == Some("delete"))
        );
    }

    #[test]
    fn escapes_cypher_strings() {
        assert_eq!(cypher_string("Schindler's List"), "'Schindler\\'s List'");
        assert_eq!(cypher_string(r"c:\tmp"), r"'c:\\tmp'");
    }

    #[test]
    fn converts_no_genres_to_empty_list() {
        assert_eq!(genres_literal("(no genres listed)"), "[]");
    }
}
