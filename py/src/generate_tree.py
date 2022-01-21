# # generate_tree.py
#
# usage: echo "1 2" | python src/generate_test_storage_tree.py
#
# read stdin for lines of "key value", after closing stdin will report a root
# hash on stdout for this per-contract storage merkle tree. nodes will be
# dumped on stderr.
#
# keys and values are either:
#
# - hex for big endian integers (whatever accepted by bytes.fromhex) prefixed by `0x`
# - base 10 integers
#
# No input validation is done for keys or values. Values will be put in StorageLeaf, keys will be used as ints.
#
# does not accept any arguments.

import os
import asyncio
import sys

from starkware.starkware_utils.commitment_tree.patricia_tree.patricia_tree import PatriciaTree
from starkware.starknet.testing.state import StarknetState
from starkware.starknet.business_logic.state_objects import ContractState
from starkware.starknet.storage.starknet_storage import StorageLeaf
from copy import deepcopy


INCLUDE_NODES = os.environ.get("TREE_TOOL_SUPPRESS_NODES") == None

async def generate_root_and_nodes(input, impl, calculate_nodes = False):
    """
    Input is a generator of (key, value)
    Returns (root, nodes)
    """

    for one_line in input:
        await impl.push(*one_line)

    new_root = await impl.commit()

    # nodes = impl.get_nodes() if INCLUDE_NODES else {}
    nodes = impl.get_nodes()

    return (new_root, nodes)

class StorageTreeBuilder:
    def __init__(self):
        state = asyncio.run(StarknetState.empty())
        self.ffc = state.state.ffc

        # creation of starknetstate will create in 0.6.2 meaningless entries in the
        # default dictionary storage; deepcopy now to filter them out later

        self.initial_ignorable_state = deepcopy(state.state.ffc.storage.db)
        self.db = state.state.ffc.storage.db

        # this should have no meaning on the output
        contract_address = (
            3434122877859862112550733797372700318255828812231958594412050293946922622982
        )

        # the testing state has a nice defaultdict which will create an entry when you try to request it
        # as the contract states. this is as opposed to raising KeyError
        # StarknetState (state) -> CarriedState (state?) -> contract_states (dict int => ContractCarriedState)
        self.contract_carried_state = state.state.contract_states[contract_address]
        assert self.contract_carried_state is not None

        self.ccs_updates = self.contract_carried_state.storage_updates
        # we'd be fine with anything dict alike but if this passes lets keep it for now
        assert type(self.ccs_updates) == dict

    async def push(self, k, v):
        self.ccs_updates[k] = StorageLeaf(v)

    async def commit(self):
        self.new_root = (
            await self.contract_carried_state.update(ffc=self.ffc)
        ).state.storage_commitment_tree.root

        return self.new_root

    def get_nodes(self):
        nodes = {}
        for k, v in self.db.items():
            if k in self.initial_ignorable_state and self.initial_ignorable_state[k] == v:
                # just filter the initial zeros and related json
                continue

            nodes[k] = v
        return nodes

class GlobalTreeBuilder:
    def __init__(self):
        # still create this for the ffc it creates.
        state = asyncio.run(StarknetState.empty())
        self.general_config = state.general_config

        # StarknetState (state) -> CarriedState (state) -> ffc with DictStorage
        self.ffc = state.state.ffc
        self.updates = {}

    async def push(self, address, hash, root):
        assert type(address) == int
        assert type(hash) == bytes, f"{type(hash)}"
        assert type(root) == bytes

        self.updates[address] = await ContractState.create(
            hash,
            PatriciaTree(
                root=root,
                height=self.general_config.contract_storage_commitment_tree_height,
            ),
        )

    async def commit(self):
        # TODO: not sure why this needs to be given
        empty_contract_state = await ContractState.empty(
            self.general_config.contract_storage_commitment_tree_height, self.ffc
        )
        root = await PatriciaTree.empty_tree(
            self.ffc,
            self.general_config.global_state_commitment_tree_height,
            empty_contract_state,
        )
        # creation of starknetstate will create in 0.6.2 meaningless entries in the
        # default dictionary storage; deepcopy now to filter them out later
        self.initial_ignorable_state = deepcopy(self.ffc.storage.db)
        self.new_root = (await root.update(self.ffc, self.updates.items())).root
        return self.new_root

    def get_nodes(self):
        nodes = {}

        for k, v in self.ffc.storage.db.items():
            if k in self.initial_ignorable_state and self.initial_ignorable_state[k] == v:
                # just filter the initial zeros and related json
                continue

            nodes[k] = v

        return nodes


def parse_storage_line(s):
    s = s.strip()
    [key, value] = s.split(maxsplit=1)
    return (parse_value(key), parse_value(value))


def parse_global_line(s):
    s = s.strip()
    [key, hash, root] = s.split(maxsplit=2)
    return (parse_value(key), parse_bytes(hash), parse_bytes(root))


def parse_value(s):
    if s.startswith("0x"):
        hex = s[2:]
        if len(hex) == 0:
            return 0
        assert len(hex) % 2 == 0, f"unsupported: odd length ({len(hex)}) hex input"
        data = bytes.fromhex(hex)
        return int.from_bytes(data, "big")

    return int(s)


def parse_bytes(s):
    if s.startswith("0x"):
        hex = s[2:]
        if len(hex) == 0:
            return (0).to_bytes(32, "big")
        assert len(hex) % 2 == 0, f"unsupported: odd length ({len(hex)}) hex input"
        return bytes.fromhex(hex)

    return int(s).to_bytes(32, "big")


if __name__ == "__main__":

    assert len(sys.argv) == 2, f"specify storage or global as argument"
    assert sys.argv[1] in ["storage", "global"], f"invalid mode: {sys.argv[1]}"

    (impl, lineparser) = (StorageTreeBuilder(), parse_storage_line) if sys.argv[1] == "storage" else (GlobalTreeBuilder(), parse_global_line)

    gen = (lineparser(line) for line in sys.stdin if len(line.strip()) > 0 and not line.startswith('#'))
    (root, nodes) = asyncio.run(generate_root_and_nodes(gen, impl, INCLUDE_NODES))
    print(root.hex())

    for k, v in nodes.items():
        [prefix, suffix] = k.split(b":", maxsplit=1)

        if prefix != b"patricia_node":
            # filter others for now
            continue

        print(f"{str(prefix, 'utf-8')}:{suffix.hex()} => {v.hex()}", file=sys.stderr)
