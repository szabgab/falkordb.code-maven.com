import json
import re
import sys
from urllib.error import HTTPError, URLError
from urllib.request import urlopen

from falkordb import FalkorDB

GRAPH_NAME = "PyPI"
PYPI_URL = "https://pypi.org/pypi/{package}/json"
REQUIREMENT_NAME = re.compile(r"^\s*([A-Za-z0-9][A-Za-z0-9._-]*)")


def fetch_package_metadata(package_name: str) -> dict:
    url = PYPI_URL.format(package=package_name)
    with urlopen(url, timeout=30) as response:
        return json.load(response)


def extract_dependency_names(requires_dist: list[str] | None) -> list[str]:
    if not requires_dist:
        return []

    dependencies: list[str] = []
    seen: set[str] = set()

    for requirement in requires_dist:
        marker = requirement.partition(";")[2]
        if "extra ==" in marker:
            continue

        match = REQUIREMENT_NAME.match(requirement)
        if not match:
            continue

        name = match.group(1)
        normalized = name.lower().replace("_", "-")
        if normalized in seen:
            continue

        seen.add(normalized)
        dependencies.append(name)

    return dependencies


def normalize_package_name(name: str) -> str:
    return name.lower().replace("_", "-")


def save_package(graph, package_name: str, leaf: bool) -> None:
    graph.query(
        """
        MERGE (package:Package {normalized_name: $normalized_name})
        ON CREATE SET package.name = $package_name
        SET package.name = $package_name,
            package.leaf = $leaf
        """,
        {
            "normalized_name": normalize_package_name(package_name),
            "package_name": package_name,
            "leaf": leaf,
        },
    )


def save_dependencies(graph, package_name: str, dependencies: list[str]) -> None:
    if not dependencies:
        return

    graph.query(
        """
        MATCH (package:Package {normalized_name: $package_normalized_name})
        UNWIND $dependencies AS dependency
        MERGE (dep:Package {normalized_name: dependency.normalized_name})
        ON CREATE SET dep.name = dependency.name
        SET dep.name = dependency.name
        MERGE (package)-[:DEPENDS_ON]->(dep)
        """,
        {
            "package_normalized_name": normalize_package_name(package_name),
            "dependencies": [
                {
                    "name": dependency,
                    "normalized_name": normalize_package_name(dependency),
                }
                for dependency in dependencies
            ],
        },
    )


def usage() -> None:
    print(f"Usage: {sys.argv[0]} PACKAGE_NAME")
    raise SystemExit(1)


def main() -> None:
    if len(sys.argv) != 2:
        usage()

    package_name = sys.argv[1]

    try:
        metadata = fetch_package_metadata(package_name)
    except HTTPError as error:
        if error.code == 404:
            print(f"Package not found on PyPI: {package_name}", file=sys.stderr)
            raise SystemExit(2) from error
        raise
    except URLError as error:
        print(f"Failed to reach PyPI: {error.reason}", file=sys.stderr)
        raise SystemExit(3) from error

    package_name = metadata["info"]["name"]
    dependencies = extract_dependency_names(metadata["info"].get("requires_dist"))

    db = FalkorDB(host="localhost", port=6379)
    graph = db.select_graph(GRAPH_NAME)

    save_package(graph, package_name, leaf=not dependencies)
    save_dependencies(graph, package_name, dependencies)

    print(package_name)
    for dependency in dependencies:
        print(f"{package_name} -> {dependency}")


if __name__ == "__main__":
    main()
