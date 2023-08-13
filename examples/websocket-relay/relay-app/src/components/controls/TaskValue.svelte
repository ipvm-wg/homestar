<script lang="ts">
  import clipboardCopy from "clipboard-copy";
  import { quintIn } from "svelte/easing";
  import { fade } from "svelte/transition";

  import Check from "$components/icons/Check.svelte";
  import CopyIcon from "$components/icons/Copy.svelte";

  export let label: string;
  export let val: string | null;

  let state: "copy" | "check" = "copy";

  async function copy(val: string | null) {
    if (!val) return;

    await clipboardCopy(val);
    state = "check";

    setTimeout(() => {
      state = "copy";
    }, 2000);
  }
</script>

<div class="grid grid-flow-row pl-1 pt-1.5 text-sm text-slate-700">
  <span class="flex flex-row items-center gap-1 w-52">
    {label}
    {#if val}
      {#if state === "copy"}
        <span
          class="cursor-pointer"
          on:click={() => copy(val)}
          on:keypress={() => copy(val)}
          in:fade={{ duration: 80, easing: quintIn }}
          out:fade={{ duration: 40, easing: quintIn }}
        >
          <CopyIcon />
        </span>
      {:else}
        <span out:fade={{ duration: 40, easing: quintIn }}>
          <Check />
        </span>
      {/if}
    {/if}
  </span>
  <span class="val text-xs font-light">{val}</span>
</div>

<style>
  .val {
    width: 13rem;
    word-wrap: break-word;
    display: inline-block;
  }
</style>
