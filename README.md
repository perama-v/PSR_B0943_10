# PSR_B0943_10
A wallet explorer prototype.

## What

Get wallet history:

- Without using APIs.
- Less than 1GB disk.

## How

Tools:

- Distributed databases (user shard and share)
    - Names/tags https://github.com/perama-v/TODD/blob/main/example_specs/nametag.md
    - 4 byte Signatures https://github.com/perama-v/TODD/blob/main/example_specs/signatures.md
    - Address appearance index https://github.com/perama-v/address-appearance-index-specs
    - TODO Distributed ABI database
- Min-know distributed database manager library https://github.com/perama-v/min-know
- Portal node (simulated here by using a full node for now). https://github.com/ethereum/trin

## Modes

- `Mode::AvoidApis` (default). P2P clients only.
- `Mode::UseApis` connects to [4byte.directory](4byte.directory) and [sourcify.dev](sourcify.dev) APIs.
## Why

- Human readable history of your own on chain activity.
- Doesn't rely on single counterparties.
- Doesn't use a large portion of your hard drive.

Some loose ramblings on this journey: https://perama-v.github.io/ethereum/protocol/poking

Try to get to the bottom without using an API or >1GB.

![svg](https://perama-v.github.io/ethereum/protocol/poking/diagrams/source.svg)

## Status

Does:

- Operate in `Mode::AvoidApis` mode
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
        Calldata: 228 bytes
        Tx Hash: 1a8d94dda1694bad33384215bb3dc0a56652b7069c71d2b1afed35b24c9b54df
        Ether sent: 140 mETH
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
        Tx Hash: 48bef06ec38f53a9f9f193717dad9b301842077d47d55e0e94fa27a05ec7193c
        Ether sent: 2 mETH
        Events emitted: 0
```
Next
- ?Match calldata components to emitted events
    - Note that one can reasonably guess the types from abi.encode()-ed data
    - Thus armed, address and values can be kept. (input address 1, input address 2, input value 1, etc.)
    - When looking at emitted topics and data, these can be matched to inputs. (event topic 1 is equal to input address 2).
    - Hence "user derived" values can be communicated. That's useful information in say a
    dex trade (user sends x and receives y) because x will appear in calldata!
- ?Use ABIs to get parameters.
    - Either using metadata IPFS/Swarm hash for ABI or a new TODD-compliant ABI database.
- Use data beyond TODD sample data (appearances, nametags and signatures databases)
    1. Use Min-know to call broadcast contract
    2. Get IPNS name
    3. Fetch manifest
    4. Survey transactions to determine relevant data
    5. Fetch data (TODD Chapters) using Min-know

## Name

PSR B0943+10 is:
- A terrible name
- The name of the smallest known pulsar
- Tiny (2.6km radius) a pulsar (does things periodically) with a radius of only 2.6km (small).
