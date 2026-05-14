import sys
from falkordb import FalkorDB


def run(graph):
    commands = [
        """CREATE (Person {name: "Alice"})""",
        """CREATE (:Person {name: "George"})""",
        """CREATE (p:Person {name: "Bob"})""",
        """CREATE (p:Person {name: "Cecile"}) RETURN p""",
        """CREATE (c:City {name: "Jerusalem"})""",
        """MATCH (p:Person) RETURN p""",
        """CREATE (:Person {name: 'Jane'})-[:knows]->(:Person {name: 'Joe'})""",
        """CREATE (:Person {name: 'Alice'})-[:knows]->(:Person {name: 'Cecile'})""",
        """MATCH (p:Person) RETURN p""",
        #"""MATCH (p:Person)-[:knows]->(q:Person) RETURN p.name, q.name""",
    ]
    # MATCH (n) OPTIONAL MATCH (n)-[e]-(m) RETURN * LIMIT 100
    for cmd in commands:
        input(cmd)
        res = graph.query(cmd)
        print(res.result_set)
        for row in res.result_set:
            for item in row:
                print(type(item).__name__)
                print(f"id: {item.id}")
                print(f"alias: {item.alias}")
                print(f"labels: {item.labels}")
                print(f"properties: {item.properties}")
        input()

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
    graph = db.select_graph("Demo")

    DISPATCH[cmd](graph)


def usage():
    cmds = ", ".join(sorted(DISPATCH.keys()))
    print(f"Usage: {sys.argv[0]} {cmds}")
    exit(1)


DISPATCH = {
    "run": run,
    "delete": delete,
}


if __name__ == "__main__":
    main()
