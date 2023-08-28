<script lang="ts">
  export let disabled = false;
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width="16"
  height="16"
  viewBox="0 0 24 24"
  fill="none"
  stroke={disabled ? "#999999" : "currentcolor"}
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class="feather feather-play"
>
  <polygon points="5 3 19 12 5 21 5 3" />
</svg>
