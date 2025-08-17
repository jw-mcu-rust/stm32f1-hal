def green(s: str) -> str:
    return f"\033[1;32m{s}\033[0m"


def red(s: str) -> str:
    return f"\033[1;31m{s}\033[0m"


def blue(s: str) -> str:
    return f"\033[1;34m{s}\033[0m"
