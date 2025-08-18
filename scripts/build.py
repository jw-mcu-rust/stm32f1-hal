import argparse
import os
import subprocess

from base import green
from sync_code import sync_all


def run_cmd(cmd: list[str]) -> None:
    print(f"{green('Running')}: {' '.join(cmd)}", flush=True)
    subprocess.run(cmd, text=True, check=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--features", type=str, nargs="*")
    parser.add_argument("-e", "--examples", type=str, nargs="*")
    opts = parser.parse_args()

    sync_all()

    cmd = ["cargo"]

    if opts.examples:
        for e in opts.examples:
            os.chdir("examples/" + e)
            cmd.extend(["build", "--release"])
            run_cmd(cmd)
            os.chdir("../../")
    else:
        if opts.release:
            cmd.extend(["build", "--release"])
        else:
            cmd.append("check")

        if opts.features is None:
            cmd.append(f"--features=stm32f103,xG")
        else:
            print(opts.features)
            for feature in opts.features:
                cmd.append(f"--features={feature}")
        run_cmd(cmd)

    return 0


if __name__ == "__main__":
    ret = 0

    try:
        ret = main()
    except KeyboardInterrupt as e:
        print(e)
        ret = -1
    except subprocess.CalledProcessError as e:
        print(e)
        ret = e.returncode

    exit(ret)
