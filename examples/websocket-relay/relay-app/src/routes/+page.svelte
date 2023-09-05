<script lang="ts">
  import type { NodeType } from "svelvet";
  import { onDestroy } from "svelte";
  import Svelvet from "svelvet";

  import { connect } from "$lib/channel";
  import { edgeStore, nodeStore } from "../stores";
  import Controls from "$components/Controls.svelte";
  import Header from "$components/Header.svelte";
  import WorkflowDetail from "$components/WorkflowDetail.svelte";

  let nodes: any[] = [];
  let edges: any[] = [];
  let showWorkflowModal = false;
  let windowHeight = window.innerHeight;
  let windowWidth = window.innerWidth;

  const unsubscribeNodeStore = nodeStore.subscribe((store) => {
    nodes = store;
  });

  const unsubscribeEdgeStore = edgeStore.subscribe((store) => {
    edges = store;
  });

  function handleWindowResize(event: Event) {
    windowHeight = window.innerHeight;
    windowWidth = window.innerWidth;
  }

  function toggleWorflowModal() {
    showWorkflowModal = !showWorkflowModal;
  }

  // Connect to websocket server
  connect();

  onDestroy(() => {
    unsubscribeNodeStore();
    unsubscribeEdgeStore();
  });
</script>

<svelte:window on:resize={handleWindowResize} />

<Header on:workflow={toggleWorflowModal} />
{#if showWorkflowModal}
  <WorkflowDetail />
{/if}
<Controls />
<Svelvet
  {nodes}
  {edges}
  width={windowWidth}
  height={windowHeight}
  initialZoom={1.25}
  initialLocation={{ x: 0, y: 0 }}
  boundary={{ x: windowWidth + 200, y: windowHeight + 200 }}
/>
