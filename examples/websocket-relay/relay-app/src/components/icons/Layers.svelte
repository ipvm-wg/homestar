<script lang="ts">
  import { draw } from "svelte/transition";
  import { quintIn } from "svelte/easing";
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width="24"
  height="24"
  viewBox="0 0 24 24"
  fill="none"
  stroke="white"
  stroke-width="1.75"
  stroke-linecap="round"
  stroke-linejoin="round"
  class="feather feather-layers"
>
  <polygon
    in:draw={{ delay: 250, duration: 150, easing: quintIn }}
    points="12 2 2 7 12 12 22 7 12 2"
  />
  <polyline
    in:draw={{ delay: 50, duration: 150, easing: quintIn }}
    points="2 17 12 22 22 17"
  />
  <polyline
    in:draw={{ delay: 150, duration: 150, easing: quintIn }}
    points="2 12 12 17 22 12"
  /></svg
>
