---
name: feedback_a_cost_probe_without_the_owners_paper_has_no_phenomenon
description: Sonda de custo sem o papel/relevo do dono não tem o fenómeno (1,6–7 ms na sonda × 1,7 s no app, pen-up a 4K); medir no app real com mede_pen_up_4k.sh
metadata:
  type: feedback
---

Em 10/11 o dono disse «trava no mouse-up numa imagem 4K». `sonda_pen_up_4k` e `measure_o_quadro_do_pen_up_em_4k` liam
1,6–7 ms e concluíram «não é a ferramenta»; no app real, com papel de relevo, era um quadro de 1,7 s (o passe de luz do impasto
sobre a tela inteira, 1 776,5 → 153,9 ms depois das faixas). A fixture das sondas não tinha relevo de papel: o fenómeno não existia nela.

**Why:** «não é» de uma sonda vale só para o que a fixture contém; pôr de fora o papel/relevo/camadas do dono é pôr de fora a causa.

**How to apply:** antes de dizer «não é», confira que a fixture tem o que o dono tem (papel com relevo, tamanho, pilha). Para
travamentos medir no app real: `docs/Painter/ferramentas/mede_pen_up_4k.sh` + `PH2D_QUADRO_LENTO_MS` (ver
`reference_perf_on_this_pc_opens_the_app_no_callgraph.md`; Bug #85 em `docs/Painter/BUGS_painter.md`).
