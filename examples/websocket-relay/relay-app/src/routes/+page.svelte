<script lang="ts">
    import { onDestroy } from "svelte";
    import { Svelvet } from "svelvet";

    import { connect } from "$lib/channel";
    import { base64CatStore, nodeStore } from "../stores";
    // import { base64CatStore, edgeStore, nodeStore } from "../stores";
    import Controls from "$components/Controls.svelte";
    import Header from "$components/Header.svelte";
    import WorkflowDetail from "$components/WorkflowDetail.svelte";
    import Node from "$components/node/Node.svelte";

    let nodes: any[] = [];
    let edges: any[] = [];
    let showWorkflowModal = false;
    let windowHeight = window.innerHeight;
    let windowWidth = window.innerWidth;

    // const unsubscribeNodeStore = nodeStore.subscribe((store) => {
    //     nodes = store;
    // });

    // const unsubscribeEdgeStore = edgeStore.subscribe((store) => {
    //   edges = store;
    // });

    function handleWindowResize(event: Event) {
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
    initializeSpaceCat();

    // Connect to websocket server
    // connect();

    // onDestroy(() => {
    // unsubscribeNodeStore();
    // unsubscribeEdgeStore();
    // });
</script>

<svelte:window on:resize={handleWindowResize} />

<Header on:workflow={toggleWorflowModal} />
{#if showWorkflowModal}
    <!-- <WorkflowDetail /> -->
{/if}
<!-- <Controls /> -->
<Svelvet width={windowWidth} height={windowHeight} zoom={1}>
    <Node id="1" position={{ x: 400, y: 200 }} />
    <Node id="2" position={{ x: 600, y: 200 }} tempcat />
    <Node id="3" position={{ x: 600, y: 400 }} tempcat />
</Svelvet>

<!-- <Svelvet
  {nodes}
  {edges}
  width={windowWidth}
  height={windowHeight}
  initialZoom={1.25}
  initialLocation={{ x: 0, y: 0 }}
  boundary={{ x: windowWidth + 200, y: windowHeight + 200 }}
/> -->
