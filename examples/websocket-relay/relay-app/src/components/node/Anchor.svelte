<script lang="ts">
  import { Anchor, type Connections, type Direction } from "svelvet";

  import Edge from "./Edge.svelte";

  export let connections: Connections | null = null
  export let direction: Direction
  export let edgeLabel: string | null = null
  export let id: string
</script>

<Anchor
  {direction}
  dynamic
  {...(connections ? { connections }: {})}
  {...(edgeLabel ? { edgeLabel }: {})}
  {id}
  invisible
  multiple
>
  <Edge slot="edge" />
</Anchor>
