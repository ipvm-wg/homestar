<script lang="ts">
  import { onDestroy } from "svelte";

  import type { Task } from "$lib/task";
  import type { Workflow, WorkflowId } from "$lib/workflow";

  import { run as runWorkflow } from "$lib/workflow";
  import { taskStore, workflowStore } from "../stores";
  import TaskEntry from "$components/controls/Task.svelte";
  import WorkflowEntry from "$components/controls/Workflow.svelte";

  let tasks: Record<string, Task[]> = {};
  let workflows: Record<string, Workflow> = {};

  const unsubscribeWorkflowStore = workflowStore.subscribe((store) => {
    workflows = store;
  });

  const unsubscribeTaskStore = taskStore.subscribe((store) => {
    tasks = store;
  });

  function expand(event: CustomEvent<{ task: Task }>) {
    const { task } = event.detail;

    tasks[task.workflowId] = tasks[task.workflowId].map((t) => {
      t.active = false;

      if (t.id === task.id) {
        t.active = true;
      }

      return t;
    });
  }

  function collapse(event: CustomEvent<{ task: Task }>) {
    const { task } = event.detail;

    tasks[task.workflowId] = tasks[task.workflowId].map((t) => {
      t.active = false;
      return t;
    });
  }

  function run(event: CustomEvent<{ workflowId: WorkflowId }>) {
    const { workflowId } = event.detail;

    runWorkflow(workflowId);
  }

  onDestroy(() => {
    unsubscribeTaskStore();
    unsubscribeWorkflowStore();
  });
</script>

<div
  class="absolute z-10 left-3 top-16 w-60 bg-white border border-black shadow-md"
>
  <WorkflowEntry workflow={workflows.one} on:run={run} />
  {#each tasks.one as task}
    <TaskEntry {task} on:expand={expand} on:collapse={collapse} />
  {/each}
  <WorkflowEntry workflow={workflows.two} on:run={run} />
  {#each tasks.two as task}
    <TaskEntry {task} on:expand={expand} on:collapse={collapse} />
  {/each}
</div>
