<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { Workflow } from "$lib/workflow";
  import PlayIcon from "$components/icons/Play.svelte";
  import Spinner from "$components/icons/Spinner.svelte";

  export let workflow: Workflow;

  const dispatch = createEventDispatcher();

  function run(workflowId: string) {
    dispatch("run", { workflowId });
  }
</script>

<div
  class="flex flex-row content-center items-center px-2 py-1.5 bg-black text-white"
>
  <span class="capitalize">Workflow {workflow.id}</span>
  <span
    class="ml-auto cursor-pointer"
    on:click={() => run(workflow.id)}
    on:keypress={() => run(workflow.id)}
  >
    {#if workflow.status === "waiting"}
      <PlayIcon />
    {:else if workflow.status === "working"}
      <Spinner />
    {/if}
  </span>
</div>
