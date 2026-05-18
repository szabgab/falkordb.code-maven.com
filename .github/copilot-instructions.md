# Copilot instructions for this repository

This repository is an mdBook-based site with runnable examples under `examples/`.

## General

- Keep changes focused and local to the requested page or example.
- Do not rewrite unrelated content or reformat large parts of the repository.
- Prefer small, readable examples over clever abstractions.
- When updating examples, keep the surrounding book content and file layout intact.

## Documentation and site content

- The site is built with `mdbook`.
- Preserve existing Markdown style and structure.
- Do not add planning or scratch Markdown files to the repository.
- As instructions are given to copilot update `.github/copilot-instructions.md` with the description of the features, behavior, coding-style.

## Rust examples

- Follow existing Rust style in each example crate.
- For CLI handling, prefer `clap`.
- For FalkorDB examples, use the `falkordb` crate already chosen by the example unless there is a clear reason to change it.
- Keep data-loading code batched and readable rather than building one huge query per file.
- When constructing Cypher strings from CSV or user data, escape string values explicitly.
- Prefer explicit error propagation over `unwrap()`.

## `examples/rust-movielens`

- The dataset lives in `examples/rust-movielens/ml-latest-small/`.
- The program should treat `--load` as the entry point for importing MovieLens CSV data into the `Movielens` graph.
- Keep the implementation async with `tokio`.
- Preserve the current graph model:
  - `(:Movie {movie_id, title, genres, imdb_id, tmdb_id})`
  - `(:User {user_id})`
  - `(:User)-[:RATED {rating, timestamp}]->(:Movie)`
  - `(:User)-[:TAGGED {tag, timestamp}]->(:Movie)`

## Validation

- For changes inside `examples/rust-movielens`, run:
  - `cargo fmt`
  - `cargo build`
  - `cargo test`
