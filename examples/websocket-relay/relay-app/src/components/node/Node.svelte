<script lang="ts">
  import { Anchor, Node } from "svelvet";
  import type { Position } from "$lib/node";
  import type { Task } from "$lib/task";
  import Edge from "./Edge.svelte";

  export let id: string;
  export let position: Position;
  export let task: Task | null = null;
  export let tempcat = false;

  let base64Cat = "";

  async function getSpacecatOriginal() {
    const catResponse = await fetch("./spacecat");
    base64Cat = await catResponse.text();
  }

  getSpacecatOriginal();
</script>

<Node
  {id}
  width={150}
  height={150}
  bgColor="white"
  borderColor="transparent"
  label="Default Node"
  {position}
>
  {#if task === null}
    {#if !tempcat}
      <img
        src={`data:image/png;base64,${base64Cat}`}
        draggable="false"
        alt="A cat in space chilling on a synth."
      />
      <Anchor
        id="1-east"
        multiple
        invisible
        direction="east"
        connections={[
          ["2", "2-west"],
          ["3", "3-west"],
        ]}
      >
        <Edge slot="edge" />
      </Anchor>
    {:else}
      <img
        src={`data:image/png;base64,${base64Cat}`}
        draggable="false"
        alt="A cat in space chilling on a synth."
      />
      <Anchor
        id={`${id}-west`}
        multiple
        invisible
        direction="west"
        connections={[["1", "1-east"]]}
      />
    {/if}
  {:else if task.status === "replayed"}
    <img
      src="data:image/png;base64,${task.receipt?.out[1]}"
      draggable="false"
      style="filter: opacity(75%)"
      alt="A cat image after a ${task.operation} performed by Homestar. The operation was replayed."
    />
  {:else}
    <img
      src="data:image/png;base64,${task.receipt?.out[1]}"
      draggable="false"
      alt="A cat image after a ${task.operation} performed by Homestar"
    />
  {/if}
</Node>
