# MotoGP in Rust


{% embed include file="examples/rust/motogp/Cargo.toml" %}

{% embed include file="examples/rust/motogp/src/main.rs" %}


When using `docker compose` we need to set the name of the FalkorDB server:

```
FALKORDB=falkordb cargo run
```

* Those extra single-quotes in the paramaters are needed or we get an error. `RedisError("to parse  query parameter 'team' value")`.

