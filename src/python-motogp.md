# MotoGP in Python

This example is based on the one on the front page of the [FalkorDB documentation](https://docs.falkordb.com/)

Run the following to show the "usage":

```shell
$ cd examples/python-demo
$ uv run motogp.py
```

The `main` function sets up the connection to the database and selects the graph called **MotoGP**.

* `$ uv run motogp.py delete` - Delete the existing graph (in case we had it already in the database):
* `$ uv run motogp.py load`   - `CREATE` a few nodes and relationships.
* `$ uv run motogp.py all_riders` - List the names of the riders.
* `$ uv run motogp.py all_pairs` - Get back all the Rider-rides-Team relations and print out the names.
* `$ uv run motogp.py all_names` - The same, but now we only return the names.
* `$ uv run motogp.py yamaha` - Which riders represent Yamaha?
* `$ uv run motogp.py ducati` - How many riders represent team Ducati? - An aggregator.


{% embed include file="examples/python-demo/motogp.py" %}


