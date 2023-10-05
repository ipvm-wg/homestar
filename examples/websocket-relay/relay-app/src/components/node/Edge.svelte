<script lang="ts">
  import { Edge } from "svelvet";

  export let arrowDirection: string = 'left'
</script>

<Edge let:path labelColor="white">
	<defs>
		<marker
			id="arrow"
			viewBox="0 0 10 10"
			refX="11"
			refY="5"
			markerWidth="6"
			markerHeight="6"
			orient="auto-start-reverse"
      fill="#000000"
		>
			<path d="M 0 0 L 10 5 L 0 10 z" />
		</marker>
	</defs>
	<path
		d={path}
    {...(arrowDirection === 'right' ? { 'marker-end': "url(#arrow)" } : { 'marker-start': "url(#arrow)" })}
	/>
</Edge>

<style>
  path {
    stroke: #000000;
    stroke-width: 2px;
  }
</style>
