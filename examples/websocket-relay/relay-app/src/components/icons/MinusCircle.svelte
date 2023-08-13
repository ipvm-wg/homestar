<script lang="ts">
  import { draw } from "svelte/transition";
  import { quartIn } from "svelte/easing";
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width="16"
  height="16"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class="feather feather-minus-circle"
>
  <circle cx="12" cy="12" r="10" />
  <line
    in:draw={{ delay: 0, duration: 125, easing: quartIn }}
    x1="8"
    y1="12"
    x2="16"
    y2="12"
  />
</svg>
