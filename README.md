# FalkorDB


## Notes



## Examples

* Family tree
* Dependency tree
* Social network
* recommendation system

* Placeholders - not mentioned in the documentation

* Represent a building
    Floors
    Flats
    Rooms
    Doors
    Walls (between two rooms, or between public area and private area)

* A map with or without traffic?


## Friends of my friends

```
(me {name:'swilly'})-[:FRIENDS_WITH]->()-[:FRIENDS_WITH]->(foaf)
```

Q: How can I filter out those who are also my friends?
Something like this?

```
(me {name:'swilly'})-[:FRIENDS_WITH]->()-[:FRIENDS_WITH]->(foaf)
AND NOT
(me {name:'swilly'})-[:FRIENDS_WITH]->(foaf)
```

Or maybe this:

```
(me {name:'swilly'})-[:FRIENDS_WITH*2..2]->()-[:FRIENDS_WITH]->(foaf)
```

