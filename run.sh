#!/bin/bash
# Quick run script for debugging without installation

export PYTHONPATH="/home/jsilva/git/tanotly/src:$PYTHONPATH"
python -m tanotly "$@"
