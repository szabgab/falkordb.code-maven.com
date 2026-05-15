import sys
import csv
from io import StringIO
from urllib.request import urlopen

from falkordb import FalkorDB

CSV_URL = "https://raw.githubusercontent.com/szabgab/exercises.code-maven.com/refs/heads/main/examples/data/family.csv"


def download_csv_to_memory(url: str) -> list[dict[str, str]]:
    with urlopen(url, timeout=30) as response:
        csv_text = response.read().decode("utf-8")

    reader = csv.DictReader(
        StringIO(csv_text),
        fieldnames=[
            name.strip() for name in csv.DictReader(StringIO(csv_text)).fieldnames
        ],
    )
    rows = list(reader)
    rows.pop(0)
    return rows


def delete(graph):
    try:
        graph.delete()
    except Exception:
        # Graph doesn't exist yet, which is fine
        pass


def load(graph) -> None:

    rows = download_csv_to_memory(CSV_URL)

    # print(f"Loaded {len(rows)} rows into memory")
    # if rows:
    #     print("First row:", rows[0])

    delete(graph)

    for row in rows:
        row = {key: value.strip() for key, value in row.items()}
        print(row)
        # print(row["Name"])
        res = graph.query("CREATE (person:Person {name: $name})", {"name": row["Name"]})
        if row["Father"]:
            res = graph.query(
                """
                   MATCH (c:Person {name: $child}), (f:Person {name: $father})
                       MERGE (c)-[:FATHER]->(f)
                """,
                {"child": row["Name"], "father": row["Father"]}
            )


def main() -> None:
    if len(sys.argv) != 2:
        usage()
    cmd = sys.argv[1]
    if cmd not in DISPATCH:
        usage()

    db = FalkorDB(host="localhost", port=6379)
    graph = db.select_graph("Family")

    DISPATCH[cmd](graph)


def usage():
    cmds = ", ".join(sorted(DISPATCH.keys()))
    print(f"Usage: {sys.argv[0]} {cmds}")
    exit(1)


DISPATCH = {
    "load": load,
    "delete": delete,
}

if __name__ == "__main__":
    main()
