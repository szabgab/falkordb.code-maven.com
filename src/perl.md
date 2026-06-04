# Perl

[FalkorDB](https://metacpan.org/pod/FalkorDB) perl module.


{% embed include file="examples/perl/docker-compose.yaml" %}

```
cd examples/perl
docker compose up
```

We can now visit the FalkorDB web UI via http://localhost:3000/

Start a client session from which we can access the server called `falkrodb`.

```
docker compose exec client bash
```
