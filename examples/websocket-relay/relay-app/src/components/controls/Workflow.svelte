<script lang="ts">
  import { createEventDispatcher } from "svelte";

  import type { Workflow } from "$lib/workflow";
  import { workflowStore } from "../../stores";
  import PlayIcon from "$components/icons/Play.svelte";
  import Spinner from "$components/icons/Spinner.svelte";

  export let workflow: Workflow;

  const dispatch = createEventDispatcher();
  let disabled = false;

  function run(workflowId: string) {
    dispatch("run", { workflowId });
  }

  $: {
    const otherId = workflow.id === "one" ? "two" : "one";
    disabled = $workflowStore[otherId].status === "working";
  }
</script>

<div
  class="flex flex-row content-center items-center px-2 py-1.5 bg-black text-white"
>
  <span class="capitalize">Workflow {workflow.id}</span>
  {#if disabled}
    <span class="ml-auto cursor-not-allowed">
      <PlayIcon disabled={true} />
    </span>
  {:else if workflow.status === "waiting"}
    <span
      class="ml-auto cursor-pointer"
      on:click={() => run(workflow.id)}
      on:keypress={() => run(workflow.id)}
    >
      <PlayIcon />
    </span>
  {:else if workflow.status === "working"}
    <span class="ml-auto cursor-progress">
      <Spinner />
    </span>
  {/if}
</div>
