import sys
from falkordb import FalkorDB


def main():
    if len(sys.argv) != 2:
        usage()
    cmd = sys.argv[1]
    if cmd not in DISPATCH:
        usage()

    db = FalkorDB(host="localhost", port=6379)
    graph = db.select_graph("Demo")

    DISPATCH[cmd](graph)


def delete(graph):
    try:
        graph.delete()
    except Exception:
        # Graph doesn't exist yet, which is fine
        pass


def load(graph):
    delete(graph)

    #    (bob)-[:KNOWS]->(alice),

    res = graph.query("""CREATE
    (alice:Person {name: "Alice", age: 30}),
    (bob:Person {name: "Bob", age: 25}),
    (carol:Person {name: "Carol", age: 27}),
    (dave:Person {name: "Dave", age: 35}),
    (eve:Person {name: "Eve", age: 29}),

    (london:City {name: "London"}),
    (paris:City {name: "Paris"}),

    (alice)-[:KNOWS]->(bob),
    (bob)-[:KNOWS]->(carol),
    (carol)-[:KNOWS]->(dave),
    (alice)-[:KNOWS]->(eve),
    (alice)-[:KNOWS]->(carol),

    (alice)-[:LIVES_IN]->(london),
    (bob)-[:LIVES_IN]->(london),
    (carol)-[:LIVES_IN]->(paris),
    (dave)-[:LIVES_IN]->(paris),
    (eve)-[:LIVES_IN]->(london);
    """)

    print(f"Nodes created: {res.nodes_created}")
    print(f"Relationships created: {res.relationships_created}")
    print()


def list_all_the_people(graph):
    res = graph.query("""MATCH (p:Person)
                     RETURN p""")

    for row in res.result_set:
        print(row[0])
        # labels and properties
        print(row[0].properties["name"])


def get_the_names_of_the_people(graph):
    res = graph.query("""MATCH (p:Person)
                     RETURN p.name""")

    for row in res.result_set:
        print(row[0])
        # labels and properties
        # print(row[0].properties['name'])


def find_younger_people(graph):
    res = graph.query("""MATCH (p:Person)
                     WHERE p.age < 30
                     RETURN p""")

    for row in res.result_set:
        print(row[0])


def find_who_knows_whom(graph):
    res = graph.query("""MATCH (who:Person)-[:KNOWS]->(whom)
                     RETURN who, whom""")

    for who, whom in res.result_set:
        print(f"{who.properties['name']:5} knows {whom.properties['name']}")


def find_who_alice_knows(graph):
    res = graph.query("""MATCH (p:Person)-[:KNOWS]->(other)
                     WHERE p.name = 'Alice'
                     RETURN other""")

    for row in res.result_set:
        print(row[0])


def find_person_by_name(graph):
    res = graph.query("""MATCH (p:Person)
                     WHERE p.name = 'Alice'
                     RETURN p""")

    for row in res.result_set:
        print(row[0])
        print(row[0].properties["name"])


def who_lives_where(graph):
    res = graph.query("""
                    MATCH (p:Person)-[:LIVES_IN]->(city:City)
                    RETURN p, city
                    """)

    for person, city in res.result_set:
        print(f"{person.properties['name']:5} -> {city.properties['name']}")


def find_people_who_live_in_london(graph):
    res = graph.query("""MATCH (p:Person)-[:LIVES_IN]->(city:City)
                     WHERE city.name = 'London'
                     RETURN p""")

    for row in res.result_set:
        print(row[0])
        print(row[0].properties["name"])


# What if A -> B -> C and also A -> C ?
# What if A -> B -> A ?
# Can we filter those out
def friends_of_friends_of_alice(graph):
    res = graph.query("""MATCH (p:Person)-[:KNOWS]->(f:Person)-[:KNOWS]->(ff:Person)
                     WHERE p.name = 'Alice'
                      AND 
                      ff.name <> 'Alice'
                     RETURN ff""")

    for row in res.result_set:
        print(row[0].properties["name"])


def how_many_people_live_in_each_city(graph):
    res = graph.query("""MATCH (p:Person)-[:LIVES_IN]->(c:City)
                     RETURN c.name, count(c)""")

    for name, count in res.result_set:
        print(f"{name} - {count}")


def usage():
    cmds = ", ".join(sorted(DISPATCH.keys()))
    print(f"Usage: {sys.argv[0]} {cmds}")
    exit(1)


DISPATCH = {
    "load": load,
    "people": list_all_the_people,
    "names": get_the_names_of_the_people,
    "younger": find_younger_people,
    "who": find_who_knows_whom,
    "alice": find_who_alice_knows,
    "by_name": find_person_by_name,
    "london": find_people_who_live_in_london,
    "where": who_lives_where,
    "ff": friends_of_friends_of_alice,
    "population": how_many_people_live_in_each_city,
    "delete": delete,
}

if __name__ == "__main__":
    main()
