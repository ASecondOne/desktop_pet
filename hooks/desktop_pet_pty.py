#!/usr/bin/env python3

from __future__ import annotations

import argparse
import os
import subprocess
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run one command under a PTY")
    parser.add_argument("--shell", required=True)
    parser.add_argument("--command", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    master_fd, slave_fd = os.openpty()

    try:
        process = subprocess.Popen(
            [args.shell, "-c", args.command],
            stdin=subprocess.DEVNULL,
            stdout=slave_fd,
            stderr=slave_fd,
            close_fds=True,
            env=os.environ.copy(),
        )
    finally:
        os.close(slave_fd)

    try:
        while True:
            try:
                chunk = os.read(master_fd, 4096)
            except InterruptedError:
                continue
            except OSError:
                break

            if not chunk:
                break

            sys.stdout.buffer.write(chunk)
            sys.stdout.buffer.flush()
    finally:
        os.close(master_fd)

    return process.wait()


if __name__ == "__main__":
    sys.exit(main())
