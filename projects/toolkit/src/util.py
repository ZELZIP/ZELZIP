from plumbum.cmd import git
from pathlib import Path


def root_path() -> Path:
    return Path(git("rev-parse", "--show-toplevel").rstrip("\r\n"))
