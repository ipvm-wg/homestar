# Homestar Roadmap

I can't even begin to describe just how WIP this is at the moment:

```mermaid
flowchart LR
    subgraph Networking
        Receipt-DHT
        pubsub
    end

    subgraph scheduling
        matchmaking-coordination
        announce-capabilities
        request-tasks
    end

    subgraph Data
        subgraph Host
            wasm-load[Load Wasm & other data by CID]
            content-handles
        end

        subgraph Guest
            block-streaming[Block streaming API in & out]
        end
    end

    Wasm_Standard_Library

    subgraph Effects
      Effect_Plugin_system

      subgraph first-party-effs
        HTTPS
        DNS
        Randomness_Oracle
        Crypto[crypto: encryption & signing]
      end
    end

    subgraph Dynamic limits
        Fuel
        Memory
        Storage
    end

    subgraph Safety
      dataflow-checks
    end

    subgraph Trust
        ucan-delegation
        ucan-invocation

        attestation

        subgraph Optimistic-Verification
            fair-scheduling
            verifying
            adjudicating
            staking
            slashing
        end

        wasm-snarks
    end

    subgraph Payment
        delegated-payment-channels-ucan-or-interlegder
        settlement-rails
    end

    subgraph DX
        subgraph DSLs
            Rust
            JS
            Python
            others-eg-apache-beam
        end

        subgraph Packaging
            static-binary
            Docker
            brew
            crate
        end

        docs
    end

    subgraph Services
        account-system
        odd-jobs
        dashboard
        task-registry
    end

    content-handles --> wasm-load
    ucan-delegation --> ucan-invocation --> attestation --> Optimistic-Verification -.->|ahead of| wasm-snarks
    block-streaming --> wasm-load
    matchmaking-coordination --> pubsub

    matchmaking-coordination --> announce-capabilities
    matchmaking-coordination --> request-tasks
```
