<script lang="ts">
  import { Node } from "svelvet";
  import type { CSSColorString } from "svelvet";

  import { base64CatStore, edgeStore } from "../../stores";
  import type { Position } from "$lib/node";
  import type { Task } from "$lib/task";
  import Anchor from "./Anchor.svelte";

  export let base64Cat: string = $base64CatStore;
  export let bgColor: CSSColorString = "white";
  export let borderColor: CSSColorString = "transparent";
  export let dimensions = {
    height: 150,
    width: 150,
  };
  export let label = "Default Node";
  export let tempcat = false;
  export let id: string;
  export let position: Position;
  export let task: Task | null = null;

  const matchingEdge = $edgeStore.find((edge) => edge?.target === id)
</script>

<Node
  {id}
  {...dimensions}
  {bgColor}
  {borderColor}
  {label}
  {position}
>
  <div class="relative w-full h-full">
    {#if task === null}
      {#if !tempcat}
        <img
          src={`data:image/png;base64,${base64Cat}`}
          draggable="false"
          alt="A cat in space chilling on a synth."
        />
        <Anchor
          id="1-east"
          direction="east"
        />
      {:else}
        <img
          src={`data:image/png;base64,${base64Cat}`}
          draggable="false"
          alt="A cat in space chilling on a synth."
        />
        <Anchor
          id="1-east"
          direction="east"
        />
      {/if}
    {:else if task.status === "replayed"}
      <img
        src={`data:image/png;base64,${task.receipt?.out[1]}`}
        draggable="false"
        style="filter: opacity(75%)"
        alt={`A cat image after a ${task.operation} performed by Homestar. The operation was replayed.`}
      />
      {#if matchingEdge}
        <!-- If difference between `target` and `source` is greater than 1, we're breaking to a new row, so we'll use north/south directions -->
        {#if (matchingEdge.target - matchingEdge.source) > 1}
          <Anchor
            id={`${id}-north`}
            direction="north"
            connections={[[`${matchingEdge.source}`, `${matchingEdge.source}-south`]]}
            edgeLabel={matchingEdge.label}
          />
        {:else}
          <Anchor
            id={`${id}-west`}
            direction="west"
            connections={[[`${matchingEdge.source}`, `${matchingEdge.source}-east`]]}
            edgeLabel={matchingEdge.label}
          />
        {/if}
      {/if}
    {:else}
      <img
        src={`data:image/png;base64,${task.receipt?.out[1]}`}
        draggable="false"
        alt={`A cat image after a ${task.operation} performed by Homestar`}
      />
      {#if matchingEdge}
        <!-- If difference between `target` and `source` is greater than 1, we're breaking to a new row, so we'll use north/south directions -->
        {#if (matchingEdge.target - matchingEdge.source) > 1}
          <Anchor
            id={`${id}-north`}
            direction="north"
            connections={[[`${matchingEdge.source}`, `${matchingEdge.source}-south`]]}
            edgeLabel={matchingEdge.label}
          />
        {:else}
          <Anchor
            id={`${id}-west`}
            direction="west"
            connections={[[`${matchingEdge.source}`, `${matchingEdge.source}-east`]]}
            edgeLabel={matchingEdge.label}
          />
        {/if}
      {/if}
    {/if}
    <Anchor
      id={`${id}-east`}
      direction="east"
    />
    <Anchor
      id={`${id}-south`}
      direction="south"
    />
  </div>
</Node>
