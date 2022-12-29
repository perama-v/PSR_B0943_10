# PSR_B0943_10
A (WIP) local wallet explorer.

PSR B0943+10 is a pulsar (does things periodically) with a radius of only 2.6km (small).
It possibly has the smallest radius of all the pulsars that we have found.

Similarly a wallet is something that acts periodically and is quite small.

Both seem hard to examine closely.

## What

It's about trying to get local wallet history without using:

- APIs
- Lots of disk (say max 1GB)

So the strategy is:

- Min-know distributed index (a flavour of the unchained-index). https://github.com/perama-v/min-know
- Portal node (simulated here by using a full node for now). https://github.com/ethereum/trin
- TODO: Distributed 4byte registry. https://github.com/perama-v/min-know/blob/main/GETTING_STARTED.md
- TODO: Distributed Sourcify registry. https://github.com/perama-v/min-know/blob/main/GETTING_STARTED.md
- Heimdall for local decompilation where source code is unavailable. https://github.com/Jon-Becker/heimdall-rs

## Why

It is probably possible to get a human readable history of your
own on chain activity. In a way that doesn't rely on single counterparties,
or a large portion of your hard drive.

https://perama-v.github.io/ethereum/protocol/poking

Try to get to the bottom without using an API or >1GB.

![svg](https://perama-v.github.io/ethereum/protocol/poking/diagrams/source.svg)

## Status

Currently:

- Can deduce which transactions are important for an address
    - By fetching 1/256th of an index that is relevant to the address.
- Can talk to a node and get the transactions and receipts.
- For each transaction can produce the events emitted. (an okay start)
- Gets links to any source code attached to the contracts that emit those events
(IPFS/swarm)
```sh
Address xyz has n transactions...
(picks first transaction)
Tx 0x1a8d94dda1694bad33384215bb3dc0a56652b7069c71d2b1afed35b24c9b54df has 5 logs:

Contract: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
        Topics logged: [0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c, 0x0000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488d]
        Metadata CID: Swarm("deb4c2ccab3c2fdca32ab3f46728389c2fe2c165d5fafa07661e4e004f6c344a")
Contract: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
        Topics logged: [0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, 0x0000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488d, 0x0000000000000000000000001636a5dfcf7a21945c06d1bea40b52ce975ea614]
        Metadata CID: Swarm("deb4c2ccab3c2fdca32ab3f46728389c2fe2c165d5fafa07661e4e004f6c344a")
Contract: 0x106d3c66d22d2dd0446df23d7f5960752994d600
        Topics logged: [0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, 0x0000000000000000000000001636a5dfcf7a21945c06d1bea40b52ce975ea614, 0x000000000000000000000000846be97d3bf1e3865f3caf55d749864d39e54cb9]
        Metadata CID: Ipfs("QmZwxURkw5nD5ZCnrhqLdDFG1G52JYKXoXhvvQV2e6cmMH")
Contract: 0x1636a5dfcf7a21945c06d1bea40b52ce975ea614
        Topics logged: [0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1]
        Metadata CID: Swarm("7dca18479e58487606bf70c79e44d8dee62353c9ee6d01f9a9d70885b8765f22")
Contract: 0x1636a5dfcf7a21945c06d1bea40b52ce975ea614
        Topics logged: [0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822, 0x0000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488d, 0x000000000000000000000000846be97d3bf1e3865f3caf55d749864d39e54cb9]
        Metadata CID: Swarm("7dca18479e58487606bf70c79e44d8dee62353c9ee6d01f9a9d70885b8765f22")
```
Next
- Get event signatures (4byte)
- Get the source code (Sourcify), or heimdall if unavailable
- Present some readable lists of things that have happened in each transaction
- See if it can be intelligible to a human interested in remembering what they have previously
been up to on-chain.

