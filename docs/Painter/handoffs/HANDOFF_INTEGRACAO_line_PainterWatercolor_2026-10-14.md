# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor` no `main` (2026-10-14)

> Ordem do dono (13/10): *«quando ela [a Macros] finalizar vc assume e integra essa linha»*. Smoke do dono ✓.
> O que a linha fez: [P34 — o dab-cápsula](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_P34_CAPSULA.md)
> (e, antes dela, W0/W1/W2 do plano 27, P32 e o smoke do Continuous). Aberto: [26](../NewFeaturesWetPaint/26_ABERTOS.md) (A1, B5, C13–C15).

## §1 — A rodada

- Uma linha só ⇒ a ordem é trivial. Merge-base `757afebd58`; o `main` andou 88 commits (Luz 2D W5o/W5p, Macros W2–W4,
  UIUX F14/F14b, MiroClone W20/W21). 13 ficheiros mexidos pelos dois lados; 8 na `ph2d-tool-painter` (a trilha
  do Painter da Macros W2).
- Rebase de 17 commits; dois conflitos, ambos no índice da memória (`project-memory/MEMORY.md`), resolvidos
  pelos estágios: (1) a entrada nova da linha ao lado das contagens que o `main` subiu; (2) a contagem da
  família «Ship / CI» **CONTADA no ficheiro fundido**: 25 na base + 1 do `main` (o cache de shaders aparado) + 1
  da linha (a suíte 1× por linha) = **27** — nenhum dos dois lados dizia 27. Marcadores varridos em cada commit:
  zero. `range-diff`: só contexto.

## §2 — O que a árvore combinada achou (`foundational-integrate.sh`)

- 1.ª corrida: 18 389/18 390 — `macro_portas::toda_porta_do_painter_tem_decisao` (Macros W2): duas portas desta
  linha sem decisão de gravabilidade — `toggle_brush_continuous_spacing` (a caixa Continuous, P32) e
  `set_brush_texture_param_shown` (os sliders do Shape nas unidades do Tuning, W1). Cada uma tinha UM leitor
  isolado (a linha não conhecia o censo; a Macros não conhecia as portas). Ambas só o painel alcança
  (`handle_panel_event`, gravado) ⇒ **INTERNAS** (`macro_track/portas_internas.rs`), como as irmãs
  `toggle_brush_accumulate` e `set_brush_texture_param_norm`. O censo: 5/5 ✓.
- 2.ª corrida: ver a mensagem do commit de integração.

## §3 — Números que somam

Registo de componentes 107/108/108 (a linha não toca) · ADR: nenhum · `PROJECT_SCHEMA`: a linha não toca ·
`Cargo.lock`: nenhum pacote externo novo · a altura de abertura do painel `painter_layers` 1 630 (a fileira
Continuous, conta escrita no portão).
