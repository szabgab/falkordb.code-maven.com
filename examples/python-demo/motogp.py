import sys
from falkordb import FalkorDB


def load(graph):

    graph.query("""CREATE
               (:Rider {name:'Valentino Rossi'})-[:rides]->(:Team {name:'Yamaha'}),
               (:Rider {name:'Dani Pedrosa'})-[:rides]->(:Team {name:'Honda'}),
               (:Rider {name:'Andrea Dovizioso'})-[:rides]->(:Team {name:'Ducati'})""")


def all_riders(graph):
    res = graph.query("""MATCH (r:Rider)
                     RETURN r""")

    # print(type(res.result_set))
    # `result_set` is a `list` of tuples.
    # In this case they are 1-element tuples. (Hence the comma)
    for (rider,) in res.result_set:
        print(rider.properties["name"])


def all_pairs(graph):
    res = graph.query("""MATCH (r:Rider)-[:rides]->(t:Team)
                     RETURN r, t""")

    for rider, team in res.result_set:
        print(f"{rider.properties['name']:18} - {team.properties['name']}")
        # alias', 'id', 'labels', 'properties', 'to_string'


def all_names(graph):
    res = graph.query("""MATCH (r:Rider)-[:rides]->(t:Team)
                     RETURN r.name, t.name""")

    for rider, team in res.result_set:
        print(f"{rider:18} - {team}")


def which_rider_represents_yamaha(graph):
    # Query which riders represent Yamaha?
    company_name = "Yamaha"
    # company_name = 'Honda'
    res = graph.query(
        """MATCH (r:Rider)-[:rides]->(t:Team)
                     WHERE t.name = $value
                     RETURN r.name""",
        {"value": company_name},
    )

    for row in res.result_set:
        print(row[0])  # Prints: "Valentino Rossi"


def how_many_riders_represent_ducati(graph):
    # Query how many riders represent team Ducati ?
    company_name = "Ducati"
    res = graph.query(
        # """MATCH (r:Rider)-[:rides]->(t:Team {name:'Ducati'}) RETURN count(r)"""
        # """MATCH (r:Rider)-[:rides]->(t:Team) WHERE t.name = 'Ducati' RETURN count(r)"""
        """MATCH (r:Rider)-[:rides]->(t:Team) WHERE t.name = $name RETURN count(r)""",
        {"name": company_name},
    )

    print(res.result_set[0][0])  # Prints: 1


def delete(graph):
    try:
        graph.delete()
    except Exception:
        # Graph doesn't exist yet, which is fine
        pass


def main() -> None:
    if len(sys.argv) != 2:
        usage()
    cmd = sys.argv[1]
    if cmd not in DISPATCH:
        usage()

    db = FalkorDB(host="localhost", port=6379)
    graph = db.select_graph("MotoGP")

    DISPATCH[cmd](graph)


def usage():
    cmds = ", ".join(sorted(DISPATCH.keys()))
    print(f"Usage: {sys.argv[0]} {cmds}")
    exit(1)


DISPATCH = {
    "load": load,
    "delete": delete,
    "yamaha": which_rider_represents_yamaha,
    "ducati": how_many_riders_represent_ducati,
    "all_names": all_names,
    "all_riders": all_riders,
    "all_pairs": all_pairs,
}


if __name__ == "__main__":
    main()
