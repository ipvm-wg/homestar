# Homestar Roadmap

I can't even begin to describe just how WIP this is at the moment:

Dependency Graph

```mermaid
flowchart
    subgraph Services
        account-system
        odd-jobs
        dashboard
        task-registry
        ucan-pay
    end

    subgraph Payment
        delegated-payment-channels-ucan-or-interlegder
        settlement-rails
    end

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

    subgraph Wasm
        deterministic-Wasm-runtime
        Wasm_Standard_Library
    end

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

    subgraph Trust
        ucan-delegation
        ucan-invocation

        attestation

        subgraph Optimistic-Verification
            direction LR

            verify
            adjudicate
            stake
            slash
        end

        wasm-snarks
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
            Nix
            Brew
            Crate-SDK
        end

        docs
    end

    content-handles --> wasm-load
    wasm-snarks -.->|after| Optimistic-Verification
    attestation --> ucan-invocation --> ucan-delegation
    block-streaming --> wasm-load
    matchmaking-coordination --> pubsub

    matchmaking-coordination --> announce-capabilities
    matchmaking-coordination --> request-tasks

    delegated-payment-channels-ucan-or-interlegder --> settlement-rails

    Crypto --> Randomness_Oracle
    HTTPS --> DNS

    dashboard --> odd-jobs --> account-system
    dashboard --> task-registry --> account-system

    odd-jobs --> deterministic-Wasm-runtime
    odd-jobs --> wasm-load
    odd-jobs --> scheduling

    ucan-pay --> account-system
    ucan-pay --> Payment

    Wasm_Standard_Library --> deterministic-Wasm-runtime
    odd-jobs --> Nix --> static-binary
    Docker --> static-binary
    Brew --> static-binary
    static-binary --> Crate-SDK

    adjudicate --> verify
    adjudicate --> slash
    slash --> stake
    stake --> settlement-rails

    slash --> attestation
    ```
