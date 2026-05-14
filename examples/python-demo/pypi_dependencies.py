import argparse
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


def clear_dependencies(graph, package_name: str) -> None:
    graph.query(
        """
        MATCH (package:Package {normalized_name: $normalized_name})
        OPTIONAL MATCH (package)-[rel:DEPENDS_ON]->()
        DELETE rel
        """,
        {"normalized_name": normalize_package_name(package_name)},
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


def delete_graph(graph) -> None:
    graph.delete()


def top_packages_by_cumulative_dependencies(graph) -> list[tuple[str, int]]:
    result = graph.query(
        """
        MATCH (package:Package)
        OPTIONAL MATCH (package)-[:DEPENDS_ON*1..]->(dependency:Package)
        RETURN package.name, count(DISTINCT dependency) AS dependency_count
        ORDER BY dependency_count DESC, package.name ASC
        LIMIT 10
        """
    )
    return [(name, count) for name, count in result.result_set]


def top_packages_by_usage(graph) -> list[tuple[str, int]]:
    result = graph.query(
        """
        MATCH (dependency:Package)
        OPTIONAL MATCH (package:Package)-[:DEPENDS_ON*1..]->(dependency)
        RETURN dependency.name, count(DISTINCT package) AS usage_count
        ORDER BY usage_count DESC, dependency.name ASC
        LIMIT 10
        """
    )
    return [(name, count) for name, count in result.result_set]


def print_report_section(title: str, rows: list[tuple[str, int]]) -> None:
    print(title)
    for index, (name, count) in enumerate(rows, start=1):
        print(f"{index:2}. {name}: {count}")
    if not rows:
        print("No packages found.")
    print()


def report_graph(graph) -> None:
    print_report_section(
        "Top 10 packages by cumulative dependency count",
        top_packages_by_cumulative_dependencies(graph),
    )
    print_report_section(
        "Top 10 most used packages",
        top_packages_by_usage(graph),
    )


def fetch_package_metadata_or_exit(package_name: str) -> dict:
    try:
        return fetch_package_metadata(package_name)
    except HTTPError as error:
        if error.code == 404:
            print(f"Package not found on PyPI: {package_name}", file=sys.stderr)
            raise SystemExit(2) from error
        raise
    except URLError as error:
        print(f"Failed to reach PyPI: {error.reason}", file=sys.stderr)
        raise SystemExit(3) from error


def import_packages(graph, package_names: list[str]) -> None:
    pending = list(reversed(package_names))
    processed: set[str] = set()

    while pending:
        requested_name = pending.pop()
        requested_normalized_name = normalize_package_name(requested_name)
        if requested_normalized_name in processed:
            continue

        metadata = fetch_package_metadata_or_exit(requested_name)
        package_name = metadata["info"]["name"]
        normalized_package_name = normalize_package_name(package_name)
        if normalized_package_name in processed:
            continue

        dependencies = extract_dependency_names(metadata["info"].get("requires_dist"))

        save_package(graph, package_name, leaf=not dependencies)
        clear_dependencies(graph, package_name)
        save_dependencies(graph, package_name, dependencies)

        processed.add(requested_normalized_name)
        processed.add(normalized_package_name)

        print(package_name)
        for dependency in dependencies:
            print(f"{package_name} -> {dependency}")
            if normalize_package_name(dependency) not in processed:
                pending.append(dependency)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("package_names", nargs="*")
    parser.add_argument("--delete", action="store_true", dest="delete_graph")
    parser.add_argument("--report", action="store_true")
    args = parser.parse_args()

    selected_modes = sum(
        [
            bool(args.package_names),
            args.delete_graph,
            args.report,
        ]
    )
    if selected_modes != 1:
        parser.error("provide PACKAGE_NAME [PACKAGE_NAME ...], --delete, or --report")

    return args


def main() -> None:
    args = parse_args()

    db = FalkorDB(host="localhost", port=6379)
    graph = db.select_graph(GRAPH_NAME)

    if args.delete_graph:
        delete_graph(graph)
        print(f"Deleted graph {GRAPH_NAME}")
        return

    if args.report:
        report_graph(graph)
        return

    import_packages(graph, args.package_names)


if __name__ == "__main__":
    main()
