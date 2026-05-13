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
    return list(reader)


def load_data(rows: list[dict[str, str]]) -> None:
    db = FalkorDB(host="localhost", port=6379)

    g = db.select_graph("Family")
    try:
        g.delete()
    except Exception:
        # Graph doesn't exist yet, which is fine
        pass

    for row in rows:
        row = {key: value.strip() for key, value in row.items()}
        print(row["Name"])
        res = g.query(
            f"""CREATE (person:Person {{name: $name}})""", {"name": row["Name"]}
        )


def main() -> None:
    rows = download_csv_to_memory(CSV_URL)
    load_data(rows)
    # print(f"Loaded {len(rows)} rows into memory")
    # if rows:
    #     print("First row:", rows[0])


if __name__ == "__main__":
    main()
