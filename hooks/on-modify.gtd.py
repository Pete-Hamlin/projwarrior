#!/usr/bin/env python

import json
import subprocess
import sys

_ = json.loads(sys.stdin.readline())
new_task = json.loads(sys.stdin.readline())
msg = ""

if "project" in new_task:
    msg = subprocess.run(
        ["proj", "add", new_task["project"]], check=True, stdout=subprocess.PIPE, universal_newlines=True
    )

print(json.dumps(new_task))
if msg:
    print(msg.stdout)
sys.exit(0)
