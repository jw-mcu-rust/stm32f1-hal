import argparse
import subprocess


def run_cmd(cmd: list[str]) -> None:
    print(f"Running: {' '.join(cmd)}", flush=True)
    subprocess.run(cmd, text=True, check=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mcu", choices=["100", "101", "103", "105", "107"], default="103")
    parser.add_argument("--release", action="store_true")
    opt = parser.parse_args()

    cmd = ["cargo"]
    if opt.release:
        cmd.append("build")
        cmd.append("--release")
    else:
        cmd.append("check")

    cmd.append(f"--features=stm32f{opt.mcu},critical-section-single-core")
    cmd.append("--example=blinky")
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
