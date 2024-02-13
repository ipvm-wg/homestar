<div align="center">
  <a href="https://github.com/ipvm-wg/homestar" target="_blank">
    <img src="https://raw.githubusercontent.com/ipvm-wg/homestar/main/assets/mascot_full_transparent.png" alt="Homestar logo" width="400"></img>
  </a>

  <h1 align="center">homestar-runtime</h1>

  <p>
    <a href="https://crates.io/crates/homestar-runtime">
      <img src="https://img.shields.io/crates/v/homestar-runtime?label=crates" alt="Crate">
    </a>
    <a href="https://github.com/ipvm-wg/homestar/blob/main/homestar-runtime/LICENSE">
      <img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License">
    </a>
    <a href="https://docs.rs/homestar-runtime">
      <img src="https://img.shields.io/static/v1?label=Docs&message=runtime.docs.rs&color=pink" alt="Docs">
    </a>
    <a href="https://fission.codes/discord">
      <img src="https://img.shields.io/static/v1?label=Discord&message=join%20us!&color=mediumslateblue" alt="Discord">
    </a>
  </p>
</div>

##

## Description

The *runtime* is responsible for bootstrapping and running nodes, scheduling
and executing workflows as well as tasks within workflows, handling retries
and failure modes, etc.

For more information, please go to our [Homestar Readme][homestar-readme].

[homestar-readme]: https://github.com/ipvm-wg/homestar/blob/main/README.md

## API

The runtime provides a JSON-RPC API to run workflows, request node information, health, and metrics, and to subscribe to network notifications. The OpenRPC API is documented in [api.json][api] and is available on the `rpc_discover` endpoint.

JSON Schemas for [workflow][workflow-schema], [receipt][receipt-schema], [network notifications][network-schema], [health checks][health-schema], [node info][node-info-schema], and [metrics][metrics-schema] are also available inidividually.

[api]: ./schemas/api.json
[health-schema]: ./schemas/health.json
[metrics-schema]: ./schemas/metrics.json
[network-schema]: ./schemas/network.json
[node-info-schema]: ./schemas/node_info.json
[receipt-schema]: ./schemas/receipt.json
[workflow-schema]: ./schemas/workflow.json
