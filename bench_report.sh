#!/usr/bin/env bash

set -e

export TREE_TOOL_SUPPRESS_NODES="true"
export VIRTUAL_ENV="$PWD/py/.venv"

cargo build --release -p pathfinder --example merkle_storage_tree
cargo build --release -p pathfinder --example merkle_global_tree

# already found around 800 node examples
# cargo run --quiet -p tree_tool -- storage >| output.bench.storage
# cargo run --quiet -p tree_tool -- global >| output.bench.global

# a is the code based off on b091cb889e624897dbb0cbec3c1df9a9e411eb1e
# b is the code based off on projective, ddc398c25aa90bf84ea1d9ac99234061bb151303
# c is cairo-lang 0.7.0

echo "contract commitment root"
head -2 output.bench.storage
a=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.storage.old target/release/examples/merkle_storage_tree.old < output.bench.storage 2>/dev/null)
b=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.storage target/release/examples/merkle_storage_tree < output.bench.storage 2>/dev/null)
c=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.cairolang.storage py/.venv/bin/python py/src/generate_tree.py storage < output.bench.storage 2>/dev/null)

if [[ "$a" == "$b" ]] && [[ "$b" == "$c" ]]; then
	echo "roots match: $a"
else
	echo "mismatch storage $a vs. $b vs. $c"
	exit 1
fi

echo -n "pathfinder.old: "
cat time.pathfinder.storage.old

echo -n "pathfinder:     "
cat time.pathfinder.storage

echo -n "cairolang:      "
cat time.cairolang.storage

echo
echo "global commitment root"

head -2 output.bench.global
a=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.global.old target/release/examples/merkle_global_tree.old < output.bench.global 2>/dev/null)
b=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.global target/release/examples/merkle_global_tree < output.bench.global 2>/dev/null)
c=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.cairolang.global py/.venv/bin/python py/src/generate_tree.py global < output.bench.global 2>/dev/null)

if [[ "$a" == "$b" ]] && [[ "$b" == "$c" ]]; then
	echo "roots match: $a"
else
	echo "mismatch $a vs. $b vs. $c"
	exit 1
fi

echo -n "pathfinder.old: "
cat time.pathfinder.global.old

echo -n "pathfinder:     "
cat time.pathfinder.global

echo -n "cairolang:      "
cat time.cairolang.global
