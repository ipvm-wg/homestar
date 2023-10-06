<script lang="ts">
  import { onDestroy } from "svelte";
  import { Svelvet } from "svelvet";

  import { connect } from "$lib/channel";
  import { base64CatStore, nodeStore } from "../stores";
  import Controls from "$components/Controls.svelte";
  import Header from "$components/Header.svelte";
  import WorkflowDetail from "$components/WorkflowDetail.svelte";
  import Node from "$components/node/Node.svelte";

  let nodes: any[] = [];
  let showWorkflowModal = false;
  let windowHeight = window.innerHeight;
  let windowWidth = window.innerWidth;

  const unsubscribeNodeStore = nodeStore.subscribe((store) => {
    nodes = store;
  });

  function handleWindowResize() {
    windowHeight = window.innerHeight;
    windowWidth = window.innerWidth;
  }

  function toggleWorflowModal() {
    showWorkflowModal = !showWorkflowModal;
  }

  async function initializeSpaceCat() {
    const catResponse = await fetch("./spacecat");
    const cat = await catResponse.text();

    base64CatStore.set(cat);
  }

  // Set spacecat unmodified image
  const fetchCat = initializeSpaceCat();

  // Connect to websocket server
  connect();

  onDestroy(() => {
    unsubscribeNodeStore();
  });
</script>

<svelte:window on:resize={handleWindowResize} />

<Header on:workflow={toggleWorflowModal} />

{#if showWorkflowModal}
  <WorkflowDetail />
{/if}

<Controls />

{#await fetchCat then _}
  <Svelvet width={windowWidth} height={windowHeight} zoom={1.25}>
    {#each nodes as node}
      {#key node}
        <Node {...node} />
      {/key}
    {/each}
  </Svelvet>
{/await}
