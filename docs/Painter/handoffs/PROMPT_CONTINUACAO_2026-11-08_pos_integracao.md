/pd-linha-assumir PainterWatercolor

Estado: a linha `line/PainterWatercolor` FECHOU em 08/11 com o handoff de integração
`docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-08_MISTURA_E_PAPEL.md`. Use este prompt só
DEPOIS de o integrador a pôr no main (confira com `git cherry main HEAD`: 0 `+` ⇒ `git reset --keep main`).

LEIA ANTES DE TOCAR EM CÓDIGO: a §6 (ABERTO) desse handoff e as memórias
`project_pastel_after_integration_2026_11_08.md`, `feedback_brush_method_owner_triage_and_simulate_before_building.md`,
`feedback_the_owners_reference_product_decides_the_law.md`, `feedback_brush_oracle_must_include_blank_canvas.md`,
`project_first_stroke_undo_memory_cut_next_round_2026_11_07.md`.

Primeiro: o smoke do dono da integração (§7 do handoff), se ele ainda não o fez.

Fila, por ordem:
1. **Pastel parece spray** (dono 08/11: «parece ruim»). Foto: `material_2026-11-08_pastel/pastel_hoje.png`.
   PERGUNTE o produto de referência dele para pastel (instalados: Krita, MyPaint; antes comparou com o Clip Studio).
   Depois: oráculo CORRIDO (licença por artefacto, entradas nossas, 3 fundos: branco, blocos, tela vazia), simular o
   conserto antes de construir, antes/depois lado a lado, o dono faz a triagem.
2. **Memória do 1.º traço** (dono 07/11, «depois do CI»): confira o `noite` do main; corte = undo só na caixa do
   traço, baixar `TECTO_DA_MEMORIA_MIB` em `tests/it/presets_de_fabrica/memoria.rs`.
3. Em bloco: marca do Wet por CAMADA (§6.2) · Wet Wash Mix (grão + ~4× lento) · Splash · `SPRAY_FIXO_MAX` com Wet blend ·
   painel antigo a 245 px · dicas longas sem quebra · papel de IMAGEM sem dente com Grain próprio.

Inegociáveis: os 2 alarmes de pincel por nome (regravar só com o dono, `PH2D_PINCEIS_APROVA=1`); Watercolor Basic ao
byte; a shell tem folga de ~44 linhas (`the_shell_only_shrinks`) — código de família vai para `ph2d-app-painter`;
fotos no app por `ph2d-run.sh` em Xvfb próprio (`PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PRESET=<nome>`), `import -window`,
nunca `spectacle`; texto de UI em inglês; não integre nem faça push sem ordem do dono.
