# Start FalkorDB locally

* Start FalkroDB locally in a Docker container (with autoremove)


```shell
docker run -p 6379:6379 -p 3000:3000 -it --rm falkordb/falkordb:latest
```

## Persistand storage (in the container)

* Start FalkorDB locally and call the container `falkor` to make it easy to restart.

```shell
docker run -p 6379:6379 -p 3000:3000 -it --name falkor falkordb/falkordb:latest
```

`Ctrl-C` will stop it


Restart it as a daemon:


```shell
docker restart falkor
```

To stop:


```shell
docker stop falkor
```

## Persistand storage (outside the container)

TBD

