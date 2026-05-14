# OpenCypher Basics

## Node

```
(p:Person {name: "Alice"})
```

Where
* `p` = variable
* `Person` = label
* `{}` = properties


## Relationship

(direction matters)

```
(p)-[:KNOWS]->(q)
```

* Both `p` and `q` represent nodes.

## READ: Match nodes

```
MATCH (p:Person)
RETURN p
```

## READ: Filter nodes

```
MATCH (p:Person)
WHERE p.name = "Alice"
RETURN p
```

## READ: Match relationship

```
MATCH (p:Person)-[:KNOWS]->(friend)
RETURN p, friend
```

## READ: Pattern matching

```
MATCH (p:Person)-[:KNOWS]->(f:Person)-[:KNOWS]->(fof)
RETURN fof
```

## WRITE: Create nodes

```
CREATE (p:Person {name: "Alice", age: 30})
```

## WRITE: Create relationships

```
MATCH (a:Person {name: "Alice"}), (b:Person {name: "Bob"})
CREATE (a)-[:KNOWS]->(b)
```

## MERGE

```
MERGE (p:Person {name: "Alice"})
```

* Creates if not exists
* Matches if exists

(a bit like UPSERT)

## Updating data:

```
MATCH (p:Person {name: "Alice"})
SET p.age = 31
```

## Remove property.

```
REMOVE p.age
```

## DELETE node

```
MATCH (p:Person {name: "Alice"})
DELETE p
```

## DELETE relationship?

```
MATCH (p:Person {name: "Alice"})
DETACH DELETE p
```

## Aggregation

```
MATCH (p:Person)
RETURN count(p)
```


```
MATCH (p:Person)
RETURN p.country, count(*)
```

## Variable-length paths

```
MATCH (p:Person)-[:KNOWS*1..3]->(other)
RETURN other
```


## SQL to Cyper mental switch

* JOIN -> Pattern matching
* Tables -> Graph
* Rows -> Paths
* Foreign Key -> Relationships



