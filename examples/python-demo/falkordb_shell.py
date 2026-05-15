import argparse
import atexit
from pathlib import Path
import readline

from falkordb import FalkorDB

HISTORY_FILE = Path.home() / ".falkordb_shell_history"
PROMPT = "falkordb> "
EXIT_COMMANDS = {".exit", ".quit"}
HELP_COMMANDS = {".help"}

HELP = """
.help, - Show this help page.
.exit, .quit - Quit the REPL.
.intro - Introduction to the OpenCyper commands.
"""

INTRO = """
# Create a `Node` with the `Person` label (type) and the `name` attribute (property)
CREATE (:Person {name: "Alice"})
"""


def setup_history() -> None:
    readline.parse_and_bind("tab: complete")
    readline.set_history_length(1000)
    if HISTORY_FILE.exists():
        readline.read_history_file(HISTORY_FILE)
    atexit.register(readline.write_history_file, HISTORY_FILE)


def print_result(result) -> None:
    if result.result_set:
        for row in result.result_set:
            for item in row:
                print(type(item).__name__)
                print(f"id: {item.id}")
                print(f"alias: {item.alias}")
                print(f"labels: {item.labels}")
                print(f"properties: {item.properties}")
    else:
        print("OK")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="localhost")
    parser.add_argument("--port", type=int, default=6379)
    parser.add_argument("--graph", default="Shell")
    return parser.parse_args()


def run_shell(graph) -> None:
    while True:
        try:
            command = input(PROMPT).strip()
        except EOFError:
            print()
            return
        except KeyboardInterrupt:
            print()
            continue

        if not command:
            continue

        if command == ".intro":
            print(INTRO)
            continue

        if command in HELP_COMMANDS:
            print(HELP)
            continue

        if command in EXIT_COMMANDS:
            return

        try:
            result = graph.query(command)
        except Exception as error:
            print(f"ERROR: {error}")
            continue

        print_result(result)


def main() -> None:
    args = parse_args()
    setup_history()

    db = FalkorDB(host=args.host, port=args.port)
    graph = db.select_graph(args.graph)

    print("Welcome to the interactive FalkorDB shell.")
    print("Type .help to see the help.")
    run_shell(graph)


if __name__ == "__main__":
    main()
