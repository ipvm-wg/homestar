import { derived, writable } from "svelte/store";
import type { Readable, Writable } from "svelte/store";
import type { NodeProps } from "svelvet";

import type { Channel } from "$lib/channel";
import type { Workflow, WorkflowId, WorkflowState } from "$lib/workflow";
import type { Maybe } from "$lib";
import type { Task } from "$lib/task";

// Initialized in +page.svelte
export const base64CatStore: Writable<string> = writable("")

export const channelStore: Writable<Maybe<Channel>> = writable(null);

export const workflowStore: Writable<Record<string, Workflow>> = writable({
  one: {
    id: "one",
    status: "waiting",
  },
  two: {
    id: "two",
    status: "waiting",
  },
});

export const activeWorkflowStore: Writable<Maybe<WorkflowState>> =
  writable(null);

export const firstWorkflowToRunStore: Writable<'one' | 'two'| null> = writable(null)

export const taskStore: Writable<Record<WorkflowId, Task[]>> = writable({
  one: [
    {
      id: 1,
      workflowId: "one",
      operation: "crop",
      message: "Waiting for task to complete",
      selected: false,
      status: "waiting",
    },
    {
      id: 2,
      workflowId: "one",
      operation: "rotate90",
      message: "Waiting for task to complete.",
      selected: false,
      status: "waiting",
    },
    {
      id: 3,
      workflowId: "one",
      operation: "blur",
      message: "Waiting for task to complete.",
      selected: false,
      status: "waiting",
    },
  ],
  two: [
    {
      id: 1,
      workflowId: "two",
      operation: "crop",
      message: "Waiting for task to complete.",
      selected: false,
      status: "waiting",
    },
    {
      id: 2,
      workflowId: "two",
      operation: "rotate90",
      message: "Waiting for task to complete.",
      selected: false,
      status: "waiting",
    },
    {
      id: 3,
      workflowId: "two",
      operation: "grayscale",
      message: "Waiting for task to complete.",
      selected: false,
      status: "waiting",
    },
  ],
});

export const nodeStore: Readable<NodeProps[]> = derived(
  [firstWorkflowToRunStore, taskStore],
  ($stores) => {
    const [firstWorkflowToRunStore, taskStore] = $stores;
    const workflowOneTasks = taskStore["one"];
    const workflowOneNodes = workflowOneTasks.reduce((nodes, task, index) => {
      const previous = index !== 0 ? workflowOneTasks[index - 1] : null;

      if (
        (task.status === "executed" || task.status === "replayed") &&
        (previous
          ? previous.status !== "waiting" && previous.status !== "failure"
          : true)
      ) {
        const idOffset = 2;
        const id = String(index + idOffset)

        // @ts-ignore
        nodes = [
          ...nodes,
          {
            id,
            position: {
              x: 500 + (index + 1) * 275,
              y:
                firstWorkflowToRunStore === 'two' && (id === "2" || id === "3")
                  ? 450
                  : 150,
            },
            task,
          },
        ];
      }
      return nodes;
    }, []);

    const workflowTwoTasks = taskStore["two"];
    const workflowTwoNodes = workflowTwoTasks.reduce((nodes, task, index) => {
      const previous = index !== 0 ? workflowTwoTasks[index - 1] : null;

      if (
        (task.status === "executed" || task.status === "replayed") &&
        (previous
          ? previous.status !== "waiting" && previous.status !== "failure"
          : true)
      ) {
        const idOffset = 5;

        // Check for a matching task in workflow one
        const matchingOneTask = taskStore.one.find(
          (t) => t.operation === task.operation
        );

        if (
          matchingOneTask &&
          (matchingOneTask.status === "executed" ||
            matchingOneTask.status === "replayed")
        ) {
          const nodeIndex = matchingOneTask.id - 1;

          // Update node in workflow one with opacity to indicate the replayed
          // task
          workflowOneNodes[nodeIndex] = {
            ...workflowOneNodes[nodeIndex],
          };

          // Skip adding new nodes to workflow two
          return nodes;
        }

        const id = String(index + idOffset);

        // @ts-ignore
        nodes = [
          ...nodes,
          {
            id,
            position: { x: 500 + (index + 1) * 275, y: 450 },
            task,
          },
        ];
      }

      return nodes;
    }, []);

    return [
      {
        id: "1",
        position: { x: 500, y: 300 },
      },
      ...workflowOneNodes,
      ...workflowTwoNodes,
    ];
  }
);

export const edgeStore = derived(nodeStore, ($nodeStore) => {
  let edges: any[] = [];
  const nodeIds = $nodeStore.map((node) => node.id);

  // Workflow One

  if (nodeIds.includes("1") && nodeIds.includes("2")) {
    edges = [
      ...edges,
      { id: "e1-2", source: "1", target: "2", label: "Crop", arrow: true },
    ];
  }

  if (nodeIds.includes("2") && nodeIds.includes("3")) {
    edges = [
      ...edges,
      { id: "e2-3", source: "2", target: "3", label: "Rotate90", arrow: true },
    ];
  }

  if (nodeIds.includes("3") && nodeIds.includes("4")) {
    edges = [
      ...edges,
      { id: "e3-4", source: "3", target: "4", label: "Blur", arrow: true },
    ];
  }

  // Workflow Two

  if (
    nodeIds.includes("1") &&
    nodeIds.includes("2") &&
    nodeIds.includes("3") &&
    nodeIds.includes("7")
  ) {
    edges = [
      ...edges,
      { id: "e3-7", source: "3", target: "7", label: "Grayscale", arrow: true },
    ];
  } else {
    if (nodeIds.includes("1") && nodeIds.includes("5")) {
      edges = [
        ...edges,
        { id: "e1-5", source: "1", target: "5", label: "Crop", arrow: true },
      ];
    }

    if (nodeIds.includes("5") && nodeIds.includes("6")) {
      edges = [
        ...edges,
        {
          id: "e5-6",
          source: "5",
          target: "6",
          label: "Rotate90",
          arrow: true,
        },
      ];
    }

    if (nodeIds.includes("6") && nodeIds.includes("7")) {
      edges = [
        ...edges,
        {
          id: "e6-7",
          source: "6",
          target: "7",
          label: "Grayscale",
          arrow: true,
        },
      ];
    }
  }

  return edges;
});
