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
- J13 (02/11): a óptica do Real Oil fora do fio da tela, quadro parado 72–82 → 16,5–17,0 ms; cena `PH2D_WETPAINT_SMOKE=3`; `docs/Painter/Arquitetura/J13_fps_do_real_oil.md`, com as recusas no §5
- Placa (31/10): submissões fatiadas, maior 93,3 → 22,6 ms na Radeon; ⛔ a NVIDIA travou o PC 2×; `docs/Painter/49_realshader/PLACA_degrau6.md`
