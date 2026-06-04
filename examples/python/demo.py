import sys
from falkordb import FalkorDB


def run(graph):
    commands = [
        # Create a `Node` with the `Person` label (type) and the `name` attribute (property)
        """CREATE (:Person {name: "Alice"})""",
        # Create another `Node` of type `Person`, set `p` as an alias to the created node and return it.
        # Later we'll be able to reuse this alias either inside the command or in the client language.
        """CREATE (p:Person {name: "Bob", email: "bob@example.org"}) RETURN p""",

        # Create a `Node` of type `Article`.
        """CREATE (a:Article {title: "Introduction for FalkorDB"}) RETURN a""",

        # If we (mistakenly) leave out the colon `:` we create a `Node` without a label
        # In this case `Person` is the alias that was not used to return the Node.
        """CREATE (Person {name: "Clark"})""",

        # Create a link (edge, relationship) from Alice to Bob called `KNOWS`. (Alice knows Bob)
        """MATCH (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'})  MERGE (a)-[:KNOWS]->(b)""",
        # CREate another type of link between `Alice` and that `Article`.
        """MATCH (p:Person {name: 'Alice'}), (a: Article) WHERE ID(a) = 0  MERGE (p)-[:published]->(a)"""


        # Return all the Nodes.
        """MATCH (p) RETURN p""",

        # Return all the Nodes with `Person` label
        """MATCH (p:Person) RETURN p""",

        # Instead of changing the existing Node,
        # This will create another `Node` of type `Person` with name `Alice`.
        # It is probably not a good idea.
        """CREATE (p:Person {name: "Alice", email: "alice@example.org"}) RETURN p""",

        # List all the Alice-es
        """MATCH (p:Person, {name: "Alice"}) RETURN p""",

        # Update all the Person Nodes where the name is Alice adding a new attribute
        # As we have 2 nodes mathching, this will update both.
        """MATCH (p:Person {name: 'Alice'}) SET p.age = 42 RETURN p"""


        # Delete all the nodes (and thus all the relationships)
        #"""MATCH (n) DETACH DELETE n"""

        # TODO
        """CREATE (:Person {name: 'Jane'})-[:knows]->(:Person {name: 'Joe'})""",
        """CREATE (:Person {name: 'Alice'})-[:knows]->(:Person {name: 'Cecile'})""",
        """MATCH (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'})  MERGE (a)-[:KNOWS]->(b)""",
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
