import { derived, writable } from "svelte/store";
import type { Readable, Writable } from "svelte/store";
import type { NodeType } from "svelvet";

import type { Channel } from "$lib/channel";
import type { Workflow, WorkflowId, WorkflowState } from "$lib/workflow";
import type { Maybe } from "$lib";
import type { Task } from "$lib/task";

const catResponse = await fetch("./spacecat");
const base64Cat = await catResponse.text();

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

export const taskStore: Writable<Record<WorkflowId, Task[]>> = writable({
  one: [
    {
      id: 1,
      workflowId: "one",
      operation: "crop",
      message: "Waiting for task to complete",
      active: false,
      status: "waiting",
    },
    {
      id: 2,
      workflowId: "one",
      operation: "rotate90",
      message: "Waiting for task to complete.",
      active: false,
      status: "waiting",
    },
    {
      id: 3,
      workflowId: "one",
      operation: "blur",
      message: "Waiting for task to complete.",
      active: false,
      status: "waiting",
    },
  ],
  two: [
    {
      id: 1,
      workflowId: "two",
      operation: "crop",
      message: "Waiting for task to complete.",
      active: false,
      status: "waiting",
    },
    {
      id: 2,
      workflowId: "two",
      operation: "rotate90",
      message: "Waiting for task to complete.",
      active: false,
      status: "waiting",
    },
    {
      id: 3,
      workflowId: "two",
      operation: "grayscale",
      message: "Waiting for task to complete.",
      active: false,
      status: "waiting",
    },
  ],
});

export const nodeStore: Readable<NodeType[]> = derived(
  taskStore,
  ($taskStore) => {
    const workflowOneNodes = $taskStore["one"].reduce((nodes, task, index) => {
      if (task.status === "executed" || task.status === "replayed") {
        const idOffset = 2;

        // @ts-ignore
        nodes = [
          ...nodes,
          {
            id: String(index + idOffset),
            position: { x: 500 + (index + 1) * 250, y: 150 },
            data: {
              html:
                task.status === "replayed"
                  ? `<img src="data:image/png;base64,${task.receipt?.out[1]}" draggable="false" style="filter: opacity(75%)" />`
                  : `<img src="data:image/png;base64,${task.receipt?.out[1]}" draggable="false" />`,
            },
            width: 150,
            height: 150,
            bgColor: "white",
            borderColor: "transparent",
          },
        ];
      }
      return nodes;
    }, []);

    const workflowTwoNodes = $taskStore["two"].reduce((nodes, task, index) => {
      if (task.status === "executed" || task.status === "replayed") {
        const idOffset = 5;

        // Check for a matching task in workflow one
        const matchingOneTask = $taskStore.one.find(
          (t) => t.operation === task.operation
        );

        if (
          matchingOneTask &&
          (matchingOneTask.status === "executed" ||
            matchingOneTask.status === "replayed")
        ) {
          const nodeIndex = matchingOneTask.id - 1;
          const updatedHtml = `${workflowOneNodes[nodeIndex].data.html.slice(
            0,
            -2
          )} style="filter: opacity(75%)" />`;

          // Update node in workflow one with opacity to indicate the replayed
          // task
          workflowOneNodes[nodeIndex] = {
            ...workflowOneNodes[nodeIndex],
            data: { html: updatedHtml },
          };

          // Skip adding new nodes to workflow two
          return nodes;
        }

        // @ts-ignore
        nodes = [
          ...nodes,
          {
            id: String(index + idOffset),
            position: { x: 500 + (index + 1) * 250, y: 450 },
            data: {
              html:
                task.status === "replayed"
                  ? `<img src="data:image/png;base64,${task.receipt?.out[1]}" draggable="false" style="filter: opacity(75%)" />`
                  : `<img src="data:image/png;base64,${task.receipt?.out[1]}" draggable="false" />`,
            },
            width: 150,
            height: 150,
            bgColor: "white",
            borderColor: "transparent",
          },
        ];
      }
      return nodes;
    }, []);

    return [
      {
        id: "1",
        position: { x: 500, y: 300 },
        data: {
          html: `<img src="data:image/png;base64,${base64Cat}" draggable="false" />`,
        },
        width: 150,
        height: 150,
        bgColor: "white",
        borderColor: "transparent",
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
