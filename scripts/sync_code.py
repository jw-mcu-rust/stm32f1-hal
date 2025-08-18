import argparse

from base import blue, green, red

TABLE = {"src/uart/uart.rs": "src/uart/usart.rs"}


def get_code(file: str, mark: str) -> tuple[str, str, str]:
    with open(file, "r", encoding="utf-8") as f:
        code = f.read()
        i = code.find(f"// {mark} begin")
        before = code[:i]
        code = code[i:]
        i = code.find(f"// {mark} end")
        i = code.find("end", i) + 3
        after = code[i:]
        code = code[:i]
        return (before, code, after)


def sync_code(table: dict[str, str], check: bool) -> bool:
    ok = True
    for key, value in table.items():
        (_, code1, _) = get_code(value, "sync")
        (before, code2, after) = get_code(key, "sync")
        if code1 == code2:
            print(f"{blue('Synced')}: {key}", flush=True)
        elif check:
            print(f"{red('Unsynced')}: {key}", flush=True)
            ok = False
        else:
            print(f"{green('Syncing')}: {key}", flush=True)
            with open(key, "w", encoding="utf-8") as f:
                f.write(before)
                f.write(code1)
                f.write(after)
    return ok


def sync_all() -> None:
    sync_code(TABLE, False)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    opts = parser.parse_args()

    if not sync_code(TABLE, opts.check):
        exit(1)


if __name__ == "__main__":
    main()
