# `tree_tool`

Used to test with random inputs the two:

- `crates/pathfinder/examples/merkle_{global,storage}_tree`
- `py/src/generate_test_{global,storage}_tree.py`

## Usage

Simplest way is to generate input, run it with both implementations and compare output hashes.
The following assumes you are in the repository root, and have set up the `py/`.
The set up virtual environment does not need to be activated.

1. generate an input file by `cargo run -p tree_tool -- {global,storage} > output`.
2. feed it to pathfinder `cargo run -p pathfinder --examples merkle_{global,storage}_tree < output 2>/dev/null`
3. feed it to python `VIRTUAL_ENV=py/.venv py/.venv/python py/src/generate_test_{global,storage}_tree.py < output 2>/dev/null`
4. compare the output hashes
