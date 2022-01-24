#!/usr/bin/env bash

set -e

export TREE_TOOL_SUPPRESS_NODES="true"
export VIRTUAL_ENV="$PWD/py/.venv"

cargo build --release -p tree_tool --bin generate_tree

# already found around 800 node examples
# cargo run --quiet -p tree_tool -- storage >| output.bench.storage
# cargo run --quiet -p tree_tool -- global >| output.bench.global

# a is the code based off on before projective coordinates
# b is the code based off on projective coordinates
# c is cairo-lang 0.7.0

# you can get generate_tree.old by:
#
# 1. rebasing this patch set to ce9aabcb315baf7fd78ed492cb625d8aeb3d2114
# 2. building the generate_tree binary
# 3. renaming it
# 4. rebase this to the d2091002621b408cf2d16be95b56458cb7a649ea
# 5. run the script

echo "contract commitment root"
head -2 output.bench.storage
a=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.storage.old target/release/generate_tree.old storage < output.bench.storage 2>/dev/null)
b=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.storage target/release/generate_tree storage < output.bench.storage 2>/dev/null)
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
a=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.global.old target/release/generate_tree.old global < output.bench.global 2>/dev/null)
b=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.global target/release/generate_tree global < output.bench.global 2>/dev/null)
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
