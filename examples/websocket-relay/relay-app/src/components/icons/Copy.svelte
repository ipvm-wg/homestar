<script lang="ts">
  import { draw } from "svelte/transition";
  import { quartIn, quartOut } from "svelte/easing";
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
  class="feather feather-copy"
>
  <rect
    in:draw={{ delay: 50, duration: 160, easing: quartOut }}
    x="9"
    y="9"
    width="13"
    height="13"
    rx="2"
    ry="2"
  />
  <path
    in:draw={{ delay: 60, duration: 150, easing: quartIn }}
    d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"
  />
</svg>
