from falkordb import FalkorDB

def main():
    db = FalkorDB(host='localhost', port=6379)

    g = db.select_graph('Demo')
    try:
        g.delete()
    except Exception:
        # Graph doesn't exist yet, which is fine
        pass

    res = g.query("""CREATE
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

    (alice)-[:LIVES_IN]->(london),
    (bob)-[:LIVES_IN]->(london),
    (carol)-[:LIVES_IN]->(paris),
    (dave)-[:LIVES_IN]->(paris),
    (eve)-[:LIVES_IN]->(london);
    """)
    
    print(f"Nodes created: {res.nodes_created}")
    print(f"Relationships created: {res.relationships_created}")
    print()

    # Get all the people
    res = g.query("""MATCH (p:Person)
                     RETURN p""")

    for row in res.result_set:
        print(row[0])
        # labels and properties
        print(row[0].properties['name'])

    print()

    # Get the names of the people
    res = g.query("""MATCH (p:Person)
                     RETURN p.name""")

    for row in res.result_set:
        print(row[0])
        # labels and properties
        # print(row[0].properties['name'])

    print()

    # Find person by name
    res = g.query("""MATCH (p:Person)
                     WHERE p.name = 'Alice'
                     RETURN p""")

    for row in res.result_set:
        print(row[0])
        print(row[0].properties['name'])

    print()


    # Find younger people
    res = g.query("""MATCH (p:Person)
                     WHERE p.age < 30
                     RETURN p""")

    for row in res.result_set:
        print(row[0])


    print()

    print("Find who knows whom")
    res = g.query("""MATCH (who:Person)-[:KNOWS]->(whom)
                     RETURN who, whom""")

    for who, whom in res.result_set:
        print(f"{who.properties['name']:5} knows {whom.properties['name']}")

    print()

    print("Find who Alices knows")
    res = g.query("""MATCH (p:Person)-[:KNOWS]->(other)
                     WHERE p.name = 'Alice'
                     RETURN other""")

    for row in res.result_set:
        print(row[0])

if __name__ == "__main__":
    main()
