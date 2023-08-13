<script lang="ts">
  import { draw } from "svelte/transition";
  import { quartIn } from "svelte/easing";
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width="12"
  height="12"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class="feather feather-check"
>
  <polyline
    in:draw={{ delay: 100, duration: 200, easing: quartIn }}
    points="4 12 9 17 20 6"
  />
</svg>
