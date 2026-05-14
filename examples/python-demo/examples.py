import sys
from falkordb import Edge, FalkorDB, Node, Operation, Path


def add_node(graph):
    graph.query("""CREATE (:Person {name:'Foo Bar'})""")


def list_nodes(graph):
    res = graph.query("""MATCH (p:Person) RETURN count(p)""")
    print(f"count: {res.result_set[0][0]}")

    res = graph.query("""MATCH (p:Person) RETURN p""")
    for row in res.result_set:
        print(f"row: {row} - {row[0]}")


def add_people(graph):
    people = ["Joe", "Jane", "Mary", "q'ote"]
    for name in people:
        graph.query("""CREATE (:Person {name: $name})""", {"name": name})
    # graph.query("""CREATE (Joe)-[:KNOWS]->(Jane)""")


#    (alice)-[:KNOWS]->(bob),
# Joe knows Jane
# Joe likes Jane
# Joe knows Mary


def objects(graph):
    john = Node(
        alias="p",
        labels="person",
        properties={
            "name": "John Doe",
            "age": 33,
            "gender": "male",
            "status": "single",
        },
    )
    japan = Node(alias="c", labels="country", properties={"name": "Japan"})

    query = f"CREATE {john}, {japan} RETURN c, p"
    result = graph.query(query)

    country = result.result_set[0][0]
    person = result.result_set[0][1]
    print(person)
    print(country)


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
    graph = db.select_graph("Examples")

    DISPATCH[cmd](graph)


def usage():
    cmds = ", ".join(sorted(DISPATCH.keys()))
    print(f"Usage: {sys.argv[0]} {cmds}")
    exit(1)


DISPATCH = {
    "delete": delete,
    "add": add_node,
    "list": list_nodes,
    "people": add_people,
    "objects": objects,
}


if __name__ == "__main__":
    main()
