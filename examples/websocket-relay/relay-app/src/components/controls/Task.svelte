<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { slide } from "svelte/transition";
  import { quartOut } from "svelte/easing";

  import type { Task } from "$lib/task";
  import CircleIcon from "$components/icons/Circle.svelte";
  import CheckCircleIcon from "$components/icons/CheckCircle.svelte";
  import ChevronDownIcon from "$components/icons/ChevronDown.svelte";
  import ChevronUpIcon from "$components/icons/ChevronUp.svelte";
  import XCircleIcon from "$components/icons/XCircle.svelte";
  import TaskValue from "$components/controls/TaskValue.svelte";

  export let task: Task;

  const dispatch = createEventDispatcher();

  function expand(task: Task) {
    dispatch("expand", { task });
  }

  function collapse(task: Task) {
    dispatch("collapse", { task });
  }
</script>

<div class="px-2 py-1.5">
  <div class="flex flex-cols gap-2 items-center">
    {#if task.status === "waiting"}
      <CircleIcon />
    {:else if task.status === "executed" || task.status === "replayed"}
      <CheckCircleIcon />
    {:else if task.status === "failure"}
      <XCircleIcon />
    {/if}
    <span class="capitalize">
      {task.operation}
    </span>
    {#if task.selected}
      <span
        class="ml-auto cursor-pointer"
        on:click={() => collapse(task)}
        on:keypress={() => collapse(task)}
      >
        <ChevronUpIcon />
      </span>
    {:else}
      <span
        class="ml-auto cursor-pointer"
        on:click={() => expand(task)}
        on:keypress={() => expand(task)}
      >
        <ChevronDownIcon />
      </span>
    {/if}
  </div>
  {#if task.selected}
    <div transition:slide={{ easing: quartOut }}>
      {#if task.receipt}
        <p class="w-52 pl-1 pt-2 text-sm text-slate-700">{task.message}</p>
        <TaskValue label="Ran" val={task.receipt.ran} />
      {:else}
        <p class="w-52 pl-1 pt-2 text-sm text-slate-700">{task.message}</p>
      {/if}
    </div>
  {/if}
</div>
