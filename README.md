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

Does:

- Operate in Mode::AvoidApis mode
- Use sample data from TODD-compliant databases
    - Appearances (find transactions for a wallet address)
    - Nametags (label a contract involved in a transaction)
    - Signatures (translate an event emitted during a transaction)
- Use a local node to get transaction receipts
- Substitues nametags/signatures and prints transactions to the terminal.

```sh
There are 2 txs for address: 0x846be97d3bf1e3865f3caf55d749864d39e54cb9

Transaction 0:
        Sender: Self
        Recipient: 0x7a250d5630b4cf539739df2c5dacb4c659f2488d
        Contract: None
        Tx Hash: 1a8d94dda1694bad33384215bb3dc0a56652b7069c71d2b1afed35b24c9b54df
        Events emitted: 5

                Deposit(address,uint256) event (e1fffcc4)
                |WETH|erc20|contract| c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 contract
                        Topic values: 2, topic 1 0xe1ff…109c, topic 2 0x0000…488d
                        Data: 32 bytes.. Event 0/5

                Transfer(address,address,uint256) event (ddf252ad)
                |WETH|erc20|contract| c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 contract
                        Topic values: 3, topic 1 0xddf2…b3ef, topic 2 0x0000…488d, topic 3 0x0000…a614
                        Data: 32 bytes.. Event 1/5

                Transfer(address,address,uint256) event (ddf252ad)
                |LabraCoin|erc20| 106d3c66d22d2dd0446df23d7f5960752994d600 contract
                        Topic values: 3, topic 1 0xddf2…b3ef, topic 2 0x0000…a614, topic 3 0x0000…4cb9
                        Data: 32 bytes.. Event 2/5

                Unknown event (1c411e9a)
                |unlabelled| 1636a5dfcf7a21945c06d1bea40b52ce975ea614 contract
                        Topic values: 1, topic 1 0x1c41…bad1
                        Data: 64 bytes.. Event 3/5

                Unknown event (d78ad95f)
                |unlabelled| 1636a5dfcf7a21945c06d1bea40b52ce975ea614 contract
                        Topic values: 3, topic 1 0xd78a…d822, topic 2 0x0000…488d, topic 3 0x0000…4cb9
                        Data: 128 bytes.. Event 4/5

Transaction 1:
        Sender: Self
        Recipient: 0x8028cfc2e08a6b569530d4809cfa75b1f3ffd6ad
        Contract: None
        Tx Hash: 48bef06ec38f53a9f9f193717dad9b301842077d47d55e0e94fa27a05ec7193c
        Events emitted: 0
```
Next
- Use ABIs to get paramters.
    - Either using metadata IPFS/Swarm hash for ABI or a new TODD-compliant ABI database.
- Use data beyond TODD sample data (appearances, nametags and signatures databases)
    1. Use Min-know to call broadcast contract
    2. Get IPNS name
    3. Fetch manifest
    4. Survey transactions to determine relevant data
    5. Fetch data (TODD Chapters) using Min-know


