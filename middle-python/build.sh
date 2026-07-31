#!/bin/bash

set -ex

cd middle-python
python3 -m venv venv
source venv/bin/activate
pip install componentize-py==0.25.0 mypy==1.13.0
rm -rf bindings
componentize-py -d ../wit -w demo:demo/middle bindings bindings
#MYPYPATH=$(pwd)/bindings mypy --strict -m component
componentize-py -d ../wit -w demo:demo/middle componentize component -o component.wasm
