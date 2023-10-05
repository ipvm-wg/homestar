<script lang="ts">
  // import { JsonView } from "@zerodevx/svelte-json-view";
  import { slide } from "svelte/transition";
  import { quartOut } from "svelte/easing";

  // import { workflowOneJson, workflowTwoJson } from "../lib/workflow";
  // import * as workflows from "$lib/workflow";
</script>

<div
  transition:slide={{ delay: 50, duration: 500, easing: quartOut }}
  class="absolute w-screen h-[calc(100vh-48px)] top-12 z-50 bg-white"
>
  <div class="w-full h-full grid grid-flow-col grid-cols-2">
    <div class="p-4 overflow-y-auto scrollbar-hide">
      <div class="text-lg uppercase font-medium">Workflow One</div>
      <div class="jsonview">
        <!-- <JsonView json={workflowOneJson} /> -->
      </div>
    </div>
    <div class="p-4 overflow-y-auto scrollbar-hide">
      <div class="text-lg uppercase font-medium">Workflow Two</div>
      <div class="jsonview">
        <!--  <JsonView json={workflowTwoJson} /> -->
      </div>
    </div>
  </div>
</div>

<style>
  .jsonview {
    font-family: "IBM Plex Mono";
    font-size: 14px;
    --jsonBorderLeft: 1px solid #ddd;
    --jsonValStringColor: #6649f8;
    --jsonValNumberColor: #0f9162;
    --jsonValColor: #a6163a;
  }

  /* Hide scrollbar for Chrome, Safari and Opera */
  .scrollbar-hide::-webkit-scrollbar {
    display: none;
  }

  /* Hide scrollbar for IE, Edge and Firefox */
  .scrollbar-hide {
    -ms-overflow-style: none; /* IE and Edge */
    scrollbar-width: none; /* Firefox */
  }
</style>
