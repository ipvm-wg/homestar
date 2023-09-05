<script>
  import { Edge } from "svelvet";

  const edgeColor = "#777777";
</script>

<defs>
  <marker
    id="arrow"
    markerWidth="9"
    markerHeight="9"
    refX="6.5"
    refY="3.5"
    orient="auto"
  >
    <polygon points="0 0, 7 3.5, 0 7" fill={edgeColor} />
  </marker>
</defs>

<Edge let:path>
  <path d={path} />
</Edge>

<style>
  path {
    stroke: #999;
    stroke-width: 2px;
    marker-end: "url(#arrow)";
  }
</style>
