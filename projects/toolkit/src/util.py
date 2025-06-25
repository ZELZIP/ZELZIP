# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0

from plumbum.cmd import git
from pathlib import Path


def root_path() -> Path:
    return Path(git("rev-parse", "--show-toplevel").rstrip("\r\n"))
