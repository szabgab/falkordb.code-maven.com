# Movies

From GroupLens Research the [MovieLens](https://grouplens.org/datasets/movielens/) project.
Download the "MovieLens for education and development" `ml-latest-small.zip` file and unzip it in the root of the rust project.
It will create a folder called `ml-latest-small/` and the csv files in it.

Load the data:

```
FALKORDB=falkordb cargo run -- --load
```


{% embed include file="examples/rust/movielens/Cargo.toml" %}

{% embed include file="examples/rust/movielens/src/main.rs" %}


DEFAULT_FALKORDB_URL=falkor://falkordb:6379
