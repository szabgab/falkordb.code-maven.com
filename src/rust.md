# Rust

The [falkordb](https://crates.io/crates/falkordb) crate is the client library.


{% embed include file="examples/rust/docker-compose.yaml" %}

```
cd examples/rust
docker compose up
```

We can now visit the FalkorDB web UI via http://localhost:3000/

Start a client session from which we can access the server called `falkrodb`.

```
docker compose exec client bash
```
