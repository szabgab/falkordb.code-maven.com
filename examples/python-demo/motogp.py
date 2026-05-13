from falkordb import FalkorDB


def main():
    # Connect to FalkorDB
    db = FalkorDB(host='localhost', port=6379)

    # Create the 'MotoGP' graph
    g = db.select_graph('MotoGP')

    # Clear out this graph in case you've run this script before.
    # Check if the graph exists before trying to delete it.
    try:
        g.delete()
    except Exception:
        # Graph doesn't exist yet, which is fine
        pass

    g.query("""CREATE
               (:Rider {name:'Valentino Rossi'})-[:rides]->(:Team {name:'Yamaha'}),
               (:Rider {name:'Dani Pedrosa'})-[:rides]->(:Team {name:'Honda'}),
               (:Rider {name:'Andrea Dovizioso'})-[:rides]->(:Team {name:'Ducati'})""")

    # Query which riders represent Yamaha?
    res = g.query("""MATCH (r:Rider)-[:rides]->(t:Team)
                     WHERE t.name = 'Yamaha'
                     RETURN r.name""")

    for row in res.result_set:
        print(row[0]) # Prints: "Valentino Rossi"

    # Query how many riders represent team Ducati ?
    res = g.query("""MATCH (r:Rider)-[:rides]->(t:Team {name:'Ducati'}) RETURN count(r)""")

    print(res.result_set[0][0]) # Prints: 1


if __name__ == "__main__":
    main()
