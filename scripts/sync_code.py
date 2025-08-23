import argparse
from typing import Any

from base import blue, green, red

TABLE = {"src/uart/uart.rs": {"src": "src/uart/usart.rs", "marks": ["sync1", "sync2"]}}


def get_marked_code(code: str, mark: str) -> tuple[str, str, str]:
    i = code.find(f"// {mark} begin")
    before = code[:i]
    code = code[i:]
    i = code.find(f"// {mark} end")
    i = code.find("end", i) + 3
    after = code[i:]
    code = code[:i]
    return (before, code, after)


def sync_code(dest: str, info: dict[str, Any], check: bool) -> bool:
    synced = True

    with open(info["src"], "r", encoding="utf-8") as f:
        src = f.read()
    with open(dest, "r", encoding="utf-8") as f:
        output = f.read()

    for mark in info["marks"]:
        (_, code1, _) = get_marked_code(src, mark)
        (before, code2, after) = get_marked_code(output, mark)
        if code1 != code2:
            synced = False
            if not check:
                output = before + code1 + after

    if synced:
        print(f"{blue('Synced')}: {dest}", flush=True)
    elif check:
        print(f"{red('Unsynced')}: {dest}", flush=True)
    else:
        print(f"{green('Syncing')}: {dest}", flush=True)
        with open(dest, "w", encoding="utf-8") as f:
            f.write(output)

    return synced


def sync_all(check: bool = False) -> bool:
    ok = True
    for dest, info in TABLE.items():
        if not sync_code(dest, info, check):
            ok = False
    return ok


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    opts = parser.parse_args()

    if not sync_all(opts.check):
        exit(1)


if __name__ == "__main__":
    main()
