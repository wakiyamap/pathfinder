#!/usr/bin/env bash

set -e

export TREE_TOOL_SUPPRESS_NODES="true"
export VIRTUAL_ENV="$PWD/py/.venv"

# a is the code based off on before projective coordinates
OLD_EXE=target/release/generate_tree.old
# b is the code based off on projective coordinates
NEW_EXE=target/release/generate_tree
# c is cairo-lang 0.7.0
PY_SCRIPT=py/src/generate_tree.py

if ! [[ -x "$OLD_EXE" ]] {
	echo "missing: $OLD_EXE" >&2
	echo "HINT: you can compile it by rebasing this patch set to older commit:" >&2
	cat << EOF >&2
1. rebasing this patch set to ce9aabcb315baf7fd78ed492cb625d8aeb3d2114
2. building the generate_tree binary
3. renaming it mv target/release/generate_tree{,.old}
4. rebase this to the d2091002621b408cf2d16be95b56458cb7a649ea
5. run the script
EOF
	exit 1
fi

if ! [[ -x "$NEW_EXE" ]]; then
	echo "missing: $NEW_EXE" >&2
	echo "HINT: just rebase this patch set to newest commit and build --release -p tree_tool --bin generate_tree" >&2
	exit 1
fi

if ! [[ -x "py/.venv/bin/python" ]]; then
	echo "missing: py/.venv/bin/python" >&2
	echo "HINT: look at py/README.md for setup" >&2
	exit 1
fi

# already found around 800 node examples
cargo run --quiet -p tree_tool -- storage --seed 24a7b496d94d7f05a8ca503d10d223318ad42082c6d369ae3346ac44fee4893b >| output.bench.storage
cargo run --quiet -p tree_tool -- global --seed c4559117fd46873b11a54458093d5a7924873c84e458a28e1b3ed9e1623a24de >| output.bench.global

echo "contract commitment root"
head -2 output.bench.storage
a=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.storage.old "$OLD_EXE" storage < output.bench.storage 2>/dev/null)
b=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.storage "$NEW_EXE" storage < output.bench.storage 2>/dev/null)
c=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.cairolang.storage py/.venv/bin/python "$PY_SCRIPT" storage < output.bench.storage 2>/dev/null)

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
a=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.global.old "$OLD_EXE" global < output.bench.global 2>/dev/null)
b=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.pathfinder.global "$NEW_EXE" global < output.bench.global 2>/dev/null)
c=$(/usr/bin/time -f '%E wall, %S kernel, %U user' -o time.cairolang.global py/.venv/bin/python "$PY_SCRIPT" global < output.bench.global 2>/dev/null)

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
