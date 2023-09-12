# Homestar Roadmap





- Networking
  - Receipt DHT(?)
  - pubsub/gossipsub/episub
- Blockstore
  - Host
    - Load Wasm & other data by CID
    - "Content handles"
  - Guest
    - Block streaming API (in & out)
- Standard Wasm library
- (Co)effect plugin system
  - First-party effects:
    - HTTPS
    - DNS
    - Randomness oracle
    - Encryption/decryption
- Dynamic limits
  - Fuel
  - Memory
  - Storage
- Safety
  - Dataflow checking
- Trust
  - UCAN (re)delegation support
  - UCAN invocation support
  - Optimistic verification
    - Staking
    - Slashing
  - Later: wasm-on-snark
- Payment
  - Delegated payment (channels)
    - e.g. ucan-chan and/or interledger
  - Settlement
