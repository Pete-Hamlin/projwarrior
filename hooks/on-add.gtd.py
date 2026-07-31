#!/usr/bin/env python

import json
import subprocess
import sys
from pathlib import Path

task = json.loads(sys.stdin.readline())

project = task.get("project")
msg = ""

if "project" in task:
    msg = subprocess.run(
        ["proj", "add", project],
        check=True,
        stdout=subprocess.PIPE,
        universal_newlines=True,
    )
    # Clear cache
    cache_dir = Path().home() / ".cache" / "projwarrior" / "tasks.json"
    cache_dir.unlink(missing_ok=True)

print(json.dumps(task))
if msg:
    print(msg.stdout)
sys.exit(0)
