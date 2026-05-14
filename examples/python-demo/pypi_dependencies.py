import json
import re
import sys
from urllib.error import HTTPError, URLError
from urllib.request import urlopen

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

    dependencies = extract_dependency_names(metadata["info"].get("requires_dist"))
    for dependency in dependencies:
        print(dependency)


if __name__ == "__main__":
    main()
