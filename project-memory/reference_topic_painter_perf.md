---
name: reference-topic-painter-perf
description: "Perf do Painter (fatos medidos vivos)"
metadata:
  node_type: memory
  type: reference
---

- [[project_painter_w3_block2_persist_ktx2_2026_06_01]] — composite bandwidth-bound: 50×4K = 23ms; gate dirty-rect
- [[project_painter_texture_brush_stamp_cache]] — re-amostrar falloff×tex derruba FPS; cache o stamp
- [[project_painter_composite_perf_2026_06_03]] — GPU compositor 1.7ms vs 55ms; meça em `--release`
