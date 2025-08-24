import argparse
import re

from base import blue, green, red

TABLE = {
    "src/uart/usart2.rs": "src/uart/usart1.rs",
    "src/uart/usart3.rs": "src/uart/usart1.rs",
    "src/uart/uart4.rs": "src/uart/usart1.rs",
    "src/uart/uart5.rs": "src/uart/uart4.rs",
    "src/timer/timer2.rs": "src/timer/timer1.rs",
    "src/timer/timer3.rs": "src/timer/timer2.rs",
    "src/timer/timer4.rs": "src/timer/timer2.rs",
    "src/timer/timer5.rs": "src/timer/timer2.rs",
    "src/timer/timer6.rs": "src/timer/timer1.rs",
    "src/timer/timer7.rs": "src/timer/timer6.rs",
    "src/timer/timer8.rs": "src/timer/timer1.rs",
    "src/timer/timer15.rs": "src/timer/timer2.rs",
    "src/timer/timer16.rs": "src/timer/timer2.rs",
    "src/timer/timer17.rs": "src/timer/timer16.rs",
}
BEGIN_PATTERN = re.compile(r"\/\/ sync .+\s")


def get_marked_code(code: str, mark: str) -> tuple[str, str, str]:
    i = code.find(mark)
    if mark == "" or i < 0:
        return ("", "", "")

    before = code[:i]
    code = code[i:]
    i = code.find("// sync", 1)
    after = code[i:]
    code = code[:i]
    return (before, code, after)


def sync_code(dest: str, src: str, check: bool) -> bool:
    synced = True

    with open(src, "r", encoding="utf-8") as f:
        src = f.read()
    with open(dest, "r", encoding="utf-8") as f:
        output = f.read()

    last_mark = ""
    for mark in BEGIN_PATTERN.findall(src):
        (_, code1, _) = get_marked_code(src, last_mark)
        (before, code2, after) = get_marked_code(output, last_mark)
        if code2 and code1 != code2:
            synced = False
            if not check:
                output = before + code1 + after
        last_mark = mark

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
    for dest, src in TABLE.items():
        if not sync_code(dest, src, check):
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
