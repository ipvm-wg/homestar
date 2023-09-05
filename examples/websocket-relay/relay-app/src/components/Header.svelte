<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import GitHubIcon from "$components/icons/GitHub.svelte";
  import LayersIcon from "./icons/Layers.svelte";
  import XIcon from "./icons/X.svelte";

  const dispatch = createEventDispatcher();

  let showWorfklowDetail = false;

  function toggleWorkflowDetail() {
    showWorfklowDetail = !showWorfklowDetail;
    dispatch("workflow");
  }
</script>

<div
  class="absolute z-10 top-0 grid grid-cols-[1fr_auto_auto] content-center w-full h-12 px-4 py-2.5 bg-black/95 backdrop-opacity-100"
>
  <span class="text-xl text-white font-[500]">Example: Websocket Relay</span>
  <span class="flex flex-row gap-4">
    <a
      class="text-white"
      href="https://github.com/ipvm-wg/homestar"
      target="__blank"
      rel="noreferrer"
    >
      <GitHubIcon />
    </a>
    {#if !showWorfklowDetail}
      <span
        class="cursor-pointer pl-0.5"
        on:click={toggleWorkflowDetail}
        on:keypress={toggleWorkflowDetail}
      >
        <LayersIcon />
      </span>
    {:else}
      <span
        class="cursor-pointer"
        on:click={toggleWorkflowDetail}
        on:keypress={toggleWorkflowDetail}
      >
        <XIcon />
      </span>
    {/if}
  </span>
</div>
