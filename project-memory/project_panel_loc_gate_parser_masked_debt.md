---
name: project-panel-loc-gate-parser-masked-debt
description: RESOLVIDO 2026-07-10 — o parser do panel LOC-cap gate sub-contava fns (apóstrofo em comentário); fix comment-aware + re-baseline honesto; falta só o split das 14 fns agora visíveis
metadata: 
  node_type: memory
  type: project
  originSessionId: f72fd562-e393-4e8e-953f-5a10ba8f3d6c
---

**Estado: o parser foi CONSERTADO em 2026-07-10** (commit `06706243`). O que segue
vale como lição de classe, não como dívida aberta.

## O bug (2026-05-31 → 2026-07-10)

O `architecture_panel_loc_cap` (`crates/ph2d-editor-core/tests/`) andava as chaves
com um walker que alternava um flag de char-literal em **todo** `'`, inclusive
dentro de `//`. Um apóstrofo em prosa ("doesn't", "sprite's") ou um tick de
lifetime (`&'a`) deixava `in_char` preso e o walk **fechava a função cedo**.

Resultado contra-intuitivo: o gate **sub-contava**, não super-contava. Ele
media `ph2d-panel-inspector/src/event.rs::apply_event_impl` como 353 LOC quando
a função tem **477** — 124 linhas escondidas. E ao sub-contar uma fn, as
seguintes eram medidas a partir do lugar errado, mascarando mais violações.

## O conserto

`find_matching_brace` novo: pula `//` e `/* */`, strings normais e raw
(`r#"…"#`), e distingue char literal (`'x'`, `b'{'`) de tick de lifetime
(`&'a`, `'static`) — um lifetime não tem aspa de fechamento. O
`strip_test_modules` usa o mesmo walker.

**Re-baseline do `FN_OVERAGE_OK` contra a medição real** (14 fns acima do teto
de 200): 3 entradas eram **fósseis** (a fn já estava sob o teto: grid-snap
`populate` = 126 congelada em 235; inspector `paint_color_tint_section` = 124 em
289; painter-layers `paint_adjustment_params` = 54 em 227) — cada uma uma
licença silenciosa de triplicar. 2 mentiam para baixo e subiram para a verdade.
2 apertaram. 8 fns nunca antes contadas foram declaradas.

**Guarda nova** `fn_overage_allowlist_has_no_stale_entries`: entrada cuja fn
sumiu, caiu sob o teto, ou encolheu abaixo do número congelado = gate vermelho.
Espelha a guarda equivalente do `architecture_workspace_file_loc_cap`. É o que
impede o fóssil de voltar.

## Lições duráveis

1. **Um gate pode mentir para BAIXO.** Verde não prova ausência de dívida — prova
   que o medidor não a viu. Antes de confiar num contador, dê-lhe um caso cuja
   resposta você sabe (aqui: `awk` achando o `}` na coluna 0 → 477, não 353).
2. **Allowlist numérica precisa de guarda de obsolescência.** Sem ela, uma
   entrada sobrevive ao próprio motivo e vira permissão de crescer.
3. Re-baselinear um medidor quebrado **não é** violar
   [[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]: corrigir a
   MEDIÇÃO ≠ licenciar o crescimento. Os números só encolhem daqui pra frente.

## Aberto (o que NÃO foi feito)

O **split** das 14 fns acima do teto — `apply_event_impl` (477), `paint_inspector`
(431), `paint_hierarchy_body` (388), `paint_render_source_section` (307),
`paint_hierarchy_row` (291), `paint_transform_section` (281), painter-layers
`apply_event_impl` (281) e `paint` (273), `paint_body_sections` (255),
audio-mixer `paint` (225), hierarchy `apply_event` (216), `paint_brush_body`
(215), color-eq `populate` (203), `sync_sprite_fields` (202).

É código de paint/dispatch **sem cobertura de unit** — um split cego arrisca
regressão visual que o gate não pega. Cada um deve landar com smoke próprio, um
painel de cada vez. O split por-cluster `try_*` do `apply_event_impl` do
inspector já estava pronto e agora está desbloqueado.
