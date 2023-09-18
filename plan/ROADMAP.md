# Homestar Roadmap

I can't even begin to describe just how WIP this is at the moment:

Dependency Graph

```mermaid
flowchart
    subgraph Networking
        pubsub[PubSub]
        car[CAR Files]
        quic[QUIC]
        webt[WebTransport]
        https[HTTPS]
    end
    
    subgraph ca_storage[Content Addressed Storage]
        layered_bs[Layered Blockstore]
        pub_obj_store[Public Object Store]
        priv_obj_store[Secret Object Store]
    end

    subgraph task_storage[Task Storage]
        receipt_store[Receipt Retrieval]
        task_registry[Task Retrieval]
        wasm_retrieval[Wasm Retrieval]
        host_session_storage[Host Session Storage]
    end

    subgraph execution[Execution]
        wasm_based_plugin_system[Wasm-based Effects Plugin System]
        resource_limits[Resource Limits]
        wasm_execution[Wasm Runtime]
    end

    subgraph scheduling[Scheduling]
        coordinator[Coordinator]
        dag_injector[DAG Injector]
        workflow_static_analyser[Workflow Analyzer]
        workflow[Workflow]
        affinity_probe[Affinity Probe]
        match_maker[Match Maker]
    end

    subgraph tracking[Tracking]
        host_logging_metrics_traces[Host: Logging, Metrics, Traces]
        workflow_progress_tracker[Workflow Progress Tracker]
    end

    subgraph capabilities
        ucan[UCAN]
        did[DID]
    end

    subgraph trust
        reputation
        optimistic_verification
        zk_wasm
        validator
        content_handle
    end

    subgraph payment
        staking_escrow
        payment_channels
        settlement
        stripe
        eth
    end

    subgraph first_party_effects
        subgraph cryptographic_effects
            randomness_oracle_fx
            encryption_fx
            signature_fx
        end

        subgraph networking_effects
            https_fx
            dns_fx
            email_fx
        end

        subgraph store_effects
            block_object_reader_fx
            block_object_writer_fx
            block_object_reader_fx
            persistence_fx
        end
    end

    subgraph services
        user_account
        swarm_federation
        hosted_bootstrap
        managed_homestar
        self_hostable_homestar
    end

    subgraph sdk
        javascript_sdk
        python_sdk
        rust_sdk
    end

    subgraph ui
        dashboard
        cli
    end

    pubsub --> car
    pubsub --> quic
    pubsub --> webt
    pubsub --> https

    pub_obj_store --> layered_bs
    priv_obj_store --> layered_bs

    layered_bs --> pubsub

    wasm_retrieval --> ca_storage
    host_session_storage ----> ca_storage
    receipt_store --> ca_storage
    task_registry --> wasm_retrieval

    wasm_based_plugin_system --> wasm_execution
    resource_limits --> wasm_execution
    wasm_execution --> task_registry
    wasm_execution --> receipt_store

    coordinator --> match_maker
    coordinator --> dag_injector
    workflow_static_analyser --> workflow
    match_maker --> affinity_probe
    match_maker --> workflow

    affinity_probe ---> wasm_based_plugin_system

    host_logging_metrics_traces --> workflow_progress_tracker
    workflow_progress_tracker --> coordinator

    ucan --> did

    reputation --> optimistic_verification
    optimistic_verification --> coordinator
    optimistic_verification --> content_handle
    optimistic_verification --> validator
    validator -.-> receipt_store

    dag_injector --> workflow_static_analyser
    dag_injector -...-> content_handle
    workflow --> ucan

    optimistic_verification --> payment_channels

    payment_channels --> staking_escrow
    staking_escrow --> settlement
    settlement --> stripe
    settlement --> eth

    first_party_effects -.......-> wasm_based_plugin_system

    user_account
    swarm_federation --> self_hostable_homestar
    swarm_federation --> hosted_bootstrap
    managed_homestar --> self_hostable_homestar
        
    dashboard -.-> javascript_sdk
    cli --> rust_sdk
    ui ----> user_account

    %% zk_wasm --> resource_limits
    stripe --> user_account

    https_fx ~~~ block_object_reader_fx
    randomness_oracle_fx ~~~ https_fx
```
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    times function run?
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
Trust

Networking Data
