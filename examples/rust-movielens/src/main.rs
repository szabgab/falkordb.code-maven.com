use clap::Parser;
use csv::ReaderBuilder;
use falkordb::{
    AsyncGraph, FalkorClientBuilder, FalkorConnectionInfo, FalkorValue, LazyResultSet, QueryResult,
};
use serde::Deserialize;
use std::{error::Error, io, path::Path};

const DEFAULT_FALKORDB_URL: &str = "falkor://127.0.0.1:6379";
const GRAPH_NAME: &str = "Movielens";
const DATA_DIR: &str = "ml-latest-small";
const MOVIE_BATCH_SIZE: usize = 250;
const LINK_BATCH_SIZE: usize = 500;
const RATING_BATCH_SIZE: usize = 500;
const TAG_BATCH_SIZE: usize = 500;
const REPORT_MENU_ID: usize = 0;
const REPORT_COUNT: usize = 12;

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

    /// List available reports, or run a report by numeric id.
    #[arg(long, value_name = "ID", num_args = 0..=1, default_missing_value = "0")]
    report: Option<usize>,
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

    if cli.report == Some(REPORT_MENU_ID) {
        print_report_menu();
        return Ok(());
    }

    let report_to_run = cli.report.filter(|report_id| *report_id != REPORT_MENU_ID);
    let needs_graph = cli.delete || cli.load || report_to_run.is_some();

    if needs_graph {
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

        if let Some(report_id) = report_to_run {
            run_report(&mut graph, report_id).await?;
        }
    }

    Ok(())
}

fn print_report_menu() {
    println!("Available reports:");
    for report_id in 1..=REPORT_COUNT {
        if let Some(title) = report_title(report_id) {
            println!("{report_id}) {title}");
        }
    }
}

fn report_title(report_id: usize) -> Option<&'static str> {
    match report_id {
        1 => Some("Best-rated movies with enough votes"),
        2 => Some("Most active users"),
        3 => Some("Most tagged movies"),
        4 => Some("Most common tags"),
        5 => Some("Most popular genres"),
        6 => Some("Highest-rated genres"),
        7 => Some("Movies that fans of Toy Story also rated highly"),
        8 => Some("Similar users by overlapping movie ratings"),
        9 => Some("Simple personalized recommendations for user 1"),
        10 => Some("Hidden gems"),
        11 => Some("Tag-to-genre associations"),
        12 => Some("Users whose taste differs most from the crowd"),
        _ => None,
    }
}

async fn run_report(graph: &mut AsyncGraph, report_id: usize) -> AppResult<()> {
    match report_id {
        1 => report_best_rated_movies(graph).await,
        2 => report_most_active_users(graph).await,
        3 => report_most_tagged_movies(graph).await,
        4 => report_most_common_tags(graph).await,
        5 => report_most_popular_genres(graph).await,
        6 => report_highest_rated_genres(graph).await,
        7 => report_movies_liked_by_toy_story_fans(graph).await,
        8 => report_similar_users(graph).await,
        9 => report_recommendations_for_user_1(graph).await,
        10 => report_hidden_gems(graph).await,
        11 => report_tag_to_genre_associations(graph).await,
        12 => report_users_with_unusual_taste(graph).await,
        _ => Err(app_error(format!(
            "Unknown report id: {report_id}. Run --report to list the available reports."
        ))),
    }
}

async fn report_best_rated_movies(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Best-rated movies with enough votes",
        "MATCH (:User)-[r:RATED]->(m:Movie)
         WITH m, count(r) AS ratings, avg(r.rating) AS avg_rating
         WHERE ratings >= 20
         RETURN m.title AS title, ratings, round(avg_rating * 100) / 100.0 AS avg_rating
         ORDER BY avg_rating DESC, ratings DESC
         LIMIT 20",
    )
    .await
}

async fn report_most_active_users(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Most active users",
        "MATCH (u:User)-[r:RATED]->(:Movie)
         RETURN u.user_id AS user_id, count(r) AS rating_count
         ORDER BY rating_count DESC
         LIMIT 20",
    )
    .await
}

async fn report_most_tagged_movies(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Most tagged movies",
        "MATCH (:User)-[t:TAGGED]->(m:Movie)
         RETURN m.title AS title, count(t) AS tag_count
         ORDER BY tag_count DESC
         LIMIT 20",
    )
    .await
}

async fn report_most_common_tags(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Most common tags",
        "MATCH (:User)-[t:TAGGED]->(:Movie)
         RETURN t.tag AS tag, count(*) AS uses
         ORDER BY uses DESC
         LIMIT 30",
    )
    .await
}

async fn report_most_popular_genres(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Most popular genres",
        "MATCH (:User)-[:RATED]->(m:Movie)
         UNWIND m.genres AS genre
         RETURN genre, count(*) AS ratings
         ORDER BY ratings DESC
         LIMIT 20",
    )
    .await
}

async fn report_highest_rated_genres(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Highest-rated genres",
        "MATCH (:User)-[r:RATED]->(m:Movie)
         UNWIND m.genres AS genre
         WITH genre, count(*) AS ratings, avg(r.rating) AS avg_rating
         WHERE ratings >= 50
         RETURN genre, ratings, round(avg_rating * 100) / 100.0 AS avg_rating
         ORDER BY avg_rating DESC, ratings DESC
         LIMIT 20",
    )
    .await
}

async fn report_movies_liked_by_toy_story_fans(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Movies that fans of Toy Story also rated highly",
        "MATCH (u:User)-[r1:RATED]->(seed:Movie {title: 'Toy Story (1995)'})
         MATCH (u)-[r2:RATED]->(other:Movie)
         WHERE r1.rating >= 4.0 AND r2.rating >= 4.0 AND other <> seed
         RETURN other.title AS title, count(*) AS shared_fans, round(avg(r2.rating) * 100) / 100.0 AS avg_rating
         ORDER BY shared_fans DESC, avg_rating DESC
         LIMIT 20",
    )
    .await
}

async fn report_similar_users(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Similar users by overlapping movie ratings",
        "MATCH (u1:User)-[r1:RATED]->(m:Movie)<-[r2:RATED]-(u2:User)
         WHERE u1.user_id < u2.user_id
         WITH u1, u2, count(m) AS overlap, avg(abs(r1.rating - r2.rating)) AS avg_diff
         WHERE overlap >= 20
         RETURN u1.user_id AS user_1, u2.user_id AS user_2, overlap, round(avg_diff * 100) / 100.0 AS avg_diff
         ORDER BY overlap DESC, avg_diff ASC
         LIMIT 20",
    )
    .await
}

async fn report_recommendations_for_user_1(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Simple personalized recommendations for user 1",
        "MATCH (me:User {user_id: 1})-[my:RATED]->(m:Movie)<-[their:RATED]-(other:User)
         WHERE my.rating >= 4.0 AND their.rating >= 4.0
         MATCH (other)-[rec:RATED]->(candidate:Movie)
         WHERE rec.rating >= 4.0
         OPTIONAL MATCH (me)-[seen:RATED]->(candidate)
         WITH candidate, other, rec, seen
         WHERE seen IS NULL
         WITH candidate, count(DISTINCT other) AS supporters, avg(rec.rating) AS avg_rating
         WHERE supporters >= 3
         RETURN candidate.title AS title, supporters, round(avg_rating * 100) / 100.0 AS avg_rating
         ORDER BY supporters DESC, avg_rating DESC
         LIMIT 20",
    )
    .await
}

async fn report_hidden_gems(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Hidden gems",
        "MATCH (:User)-[r:RATED]->(m:Movie)
         WITH m, count(r) AS ratings, avg(r.rating) AS avg_rating
         WHERE ratings >= 10 AND ratings <= 50
         RETURN m.title AS title, ratings, round(avg_rating * 100) / 100.0 AS avg_rating
         ORDER BY avg_rating DESC
         LIMIT 20",
    )
    .await
}

async fn report_tag_to_genre_associations(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Tag-to-genre associations",
        "MATCH (:User)-[t:TAGGED]->(m:Movie)
         UNWIND m.genres AS genre
         RETURN t.tag AS tag, genre, count(*) AS freq
         ORDER BY freq DESC
         LIMIT 30",
    )
    .await
}

async fn report_users_with_unusual_taste(graph: &mut AsyncGraph) -> AppResult<()> {
    execute_report(
        graph,
        "Users whose taste differs most from the crowd",
        "MATCH (u:User)-[r:RATED]->(m:Movie)
         MATCH (:User)-[allr:RATED]->(m)
         WITH u, m, r.rating AS user_rating, avg(allr.rating) AS movie_avg
         WITH u, avg(abs(user_rating - movie_avg)) AS deviation, count(*) AS rated_movies
         WHERE rated_movies >= 20
         RETURN u.user_id AS user_id, rated_movies, round(deviation * 100) / 100.0 AS deviation
         ORDER BY deviation DESC
         LIMIT 20",
    )
    .await
}

async fn execute_report(graph: &mut AsyncGraph, title: &str, query: &str) -> AppResult<()> {
    let mut result = graph.query(query).execute().await?;
    println!("{title}");
    render_query_result(&mut result);
    Ok(())
}

fn render_query_result(result: &mut QueryResult<LazyResultSet<'_>>) {
    let headers = result.header.clone();
    let rows = result
        .data
        .by_ref()
        .map(|row| row.into_iter().map(render_value).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    if headers.is_empty() {
        println!("No columns returned.");
        return;
    }

    let mut widths = headers
        .iter()
        .map(|header| header.len())
        .collect::<Vec<_>>();
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            if index >= widths.len() {
                widths.push(cell.len());
            } else {
                widths[index] = widths[index].max(cell.len());
            }
        }
    }

    print_table_row(&headers, &widths);
    print_table_separator(&widths);

    if rows.is_empty() {
        println!("(no rows)");
        return;
    }

    for row in &rows {
        print_table_row(row, &widths);
    }
}

fn print_table_row(row: &[String], widths: &[usize]) {
    let formatted = row
        .iter()
        .zip(widths.iter())
        .map(|(value, width)| format!("{value:<width$}", width = width))
        .collect::<Vec<_>>()
        .join(" | ");
    println!("{formatted}");
}

fn print_table_separator(widths: &[usize]) {
    let separator = widths
        .iter()
        .map(|width| "-".repeat(*width))
        .collect::<Vec<_>>()
        .join("-+-");
    println!("{separator}");
}

fn render_value(value: FalkorValue) -> String {
    match value {
        FalkorValue::String(value) => value,
        FalkorValue::Bool(value) => value.to_string(),
        FalkorValue::I64(value) => value.to_string(),
        FalkorValue::F64(value) => format!("{value:.2}"),
        FalkorValue::None => "NULL".to_string(),
        FalkorValue::Array(values) => {
            let rendered = values.into_iter().map(render_value).collect::<Vec<_>>();
            format!("[{}]", rendered.join(", "))
        }
        other => format!("{other:?}"),
    }
}

fn app_error(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    io::Error::other(message.into()).into()
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
        assert_eq!(cli.report, None);
    }

    #[test]
    fn parses_delete_flag() {
        let cli = Cli::parse_from(["movielens", "--delete"]);
        assert!(cli.delete);
        assert!(!cli.load);
        assert_eq!(cli.report, None);
    }

    #[test]
    fn parses_report_menu_flag_without_value() {
        let cli = Cli::parse_from(["movielens", "--report"]);
        assert_eq!(cli.report, Some(REPORT_MENU_ID));
    }

    #[test]
    fn parses_report_flag_with_value() {
        let cli = Cli::parse_from(["movielens", "--report", "3"]);
        assert_eq!(cli.report, Some(3));
    }

    #[test]
    fn command_defines_load_delete_and_report_flags() {
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
        assert!(
            command
                .get_arguments()
                .any(|arg| arg.get_long() == Some("report"))
        );
    }

    #[test]
    fn knows_all_report_titles() {
        assert_eq!(report_title(1), Some("Best-rated movies with enough votes"));
        assert_eq!(
            report_title(12),
            Some("Users whose taste differs most from the crowd")
        );
        assert_eq!(report_title(13), None);
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

    #[test]
    fn renders_values_for_tables() {
        assert_eq!(render_value(FalkorValue::I64(7)), "7");
        assert_eq!(render_value(FalkorValue::F64(4.125)), "4.12");
        assert_eq!(
            render_value(FalkorValue::Array(vec![FalkorValue::String(
                "Drama".to_string()
            )])),
            "[Drama]"
        );
    }
}
