def green(s: str) -> str:
    return f"\033[1;32m{s}\033[0m"


def red(s: str) -> str:
    return f"\033[1;31m{s}\033[0m"


def blue(s: str) -> str:
    return f"\033[1;34m{s}\033[0m"


class Write:
    def __init__(self, file_name: str, dry_run: bool = False) -> None:
        if dry_run:
            self.f = None
        else:
            self.f = open(file_name, "w", encoding="utf-8")

    def write(self, content: str) -> None:
        if self.f:
            self.f.write(content)
        else:
            print(content, end="")

    def close(self) -> None:
        if self.f:
            self.f.close()
            self.f = None
