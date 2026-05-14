import sys
from falkordb import FalkorDB


def load(graph):

    graph.query("""CREATE
               (:Rider {name:'Valentino Rossi'})-[:rides]->(:Team {name:'Yamaha'}),
               (:Rider {name:'Dani Pedrosa'})-[:rides]->(:Team {name:'Honda'}),
               (:Rider {name:'Andrea Dovizioso'})-[:rides]->(:Team {name:'Ducati'})""")


def which_rider_represents_yamaha(graph):
    # Query which riders represent Yamaha?
    res = graph.query("""MATCH (r:Rider)-[:rides]->(t:Team)
                     WHERE t.name = 'Yamaha'
                     RETURN r.name""")

    for row in res.result_set:
        print(row[0])  # Prints: "Valentino Rossi"


def how_many_riders_represent_ducati(graph):
    # Query how many riders represent team Ducati ?
    res = graph.query(
        """MATCH (r:Rider)-[:rides]->(t:Team {name:'Ducati'}) RETURN count(r)"""
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
}


if __name__ == "__main__":
    main()
