#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import socket
import sys
from datetime import datetime


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Send one desktop_pet event")
    parser.add_argument("--socket", required=True, dest="socket_path")
    parser.add_argument("--kind", required=True, choices=("start", "output", "finish"))
    parser.add_argument("--command-id", required=True)
    parser.add_argument("--shell-pid", required=True, type=int)
    parser.add_argument("--tty", required=True)
    parser.add_argument("--cwd", required=True)
    parser.add_argument("--command", required=True)
    parser.add_argument("--stream")
    parser.add_argument("--text")
    parser.add_argument("--exit-code", dest="exit_code", type=int)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    payload = {
        "timestamp": datetime.now().astimezone().isoformat(timespec="seconds"),
        "command_id": args.command_id,
        "shell_pid": args.shell_pid,
        "tty": args.tty,
        "cwd": args.cwd,
        "command": args.command,
        "kind": args.kind,
        "stream": args.stream,
        "text": args.text,
        "exit_code": args.exit_code,
    }

    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.connect(args.socket_path)
            client.sendall((json.dumps(payload, ensure_ascii=False) + "\n").encode("utf-8"))
    except OSError:
        return 0

    return 0


if __name__ == "__main__":
    sys.exit(main())
