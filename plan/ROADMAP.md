# Homestar Roadmap

I can't even begin to describe just how WIP this is at the moment:

Dependency Graph

```mermaid
flowchart
    subgraph Networking
        pubsub
        car
        quic
        webt
        https
    end
    
    subgraph ca_storage
        layered_bs
        pub_obj_store
        priv_obj_store
    end

    subgraph task_storage
        receipt_store
        task_registry
        wasm_retrieval
        host_session_storage
    end

    subgraph execution
        wasm_based_plugin_system
        resource_limits
        wasm_execution
    end

    subgraph scheduling
        coordinator
        dag_injector
        workflow_static_analyser
        workflow
        affinity_probe
        match_maker
    end

    subgraph tracking
        host_logging_metrics_traces
        workflow_progress_tracker
    end

    subgraph capabilities
        ucan
        did
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
```
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    times function run?
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
Trust

Networking Data
