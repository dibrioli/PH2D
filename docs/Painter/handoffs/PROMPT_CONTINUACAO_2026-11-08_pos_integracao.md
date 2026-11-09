/pd-linha-assumir PainterWatercolor

Estado (09/11): a linha `line/PainterWatercolor` foi integrada no main (rodada de 09/11) e REABERTA a partir dele
(`git reset --keep main`, base `f254e42bf1`). Desde então, na linha (sem integrar):
- `02341d1b0a` — o «×» limpa a busca do Preset (peça `cards::busca`); teste `o_x_do_campo_limpa_a_busca_e_o_teclado_fica_nele`.
- `aeefe44d21` + `02341d1b0a` — sondas do rabisco: `PH2D_COMPOSITE_PILHA=0`, `PH2D_COMPOSITE_RELEVO=<v>`, `PH2D_COMPOSITE_PAPEL=1`.
- `0126220dfa` — **regressão do Wet curada** (dono: raw 170 → 14 no Textured Paper 1024, pior com o rato): o
  `sheet_cache::follow_live` (GEF, folha ao vivo do Animated Sprite) lia `imagem_por_assar` (tela inteira composta e
  acesa pelo relevo do papel, ~60 ms) a cada passo da água do Wet; agora só com a tinta parada `QUADROS_PARADOS = 30`.
  **Smoke do dono 09/11: OK** (× e velocidade).
- Ferramenta: `docs/Painter/ferramentas/mede_rato_xvfb.sh` — rato REAL (XTEST) no Xvfb + `PH2D_PAINT_PERF` + perf da linha
  principal. O rabisco automático não passa pela entrada da shell e o kwin --virtual não recebe o xdotool.

LEIA ANTES DE TOCAR EM CÓDIGO: a §6 de `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-08_MISTURA_E_PAPEL.md`
e as memórias `project_pastel_after_integration_2026_11_08.md`, `feedback_brush_method_owner_triage_and_simulate_before_building.md`,
`feedback_the_owners_reference_product_decides_the_law.md`, `feedback_brush_oracle_must_include_blank_canvas.md`,
`project_first_stroke_undo_memory_cut_next_round_2026_11_07.md`.

Primeiro — pedido do dono 09/11, à espera do SIM dele ao desenho: **ícone circular de progresso** discreto no canto do
preview animado do editor do Animated Sprite (acima das folhas) enquanto a folha pintada ainda não foi atualizada.
Desenho proposto: aparece só quando a tinta do Painter tem revisão diferente da que a folha ao vivo mostra; a GIRAR
enquanto a tinta/água anda (não se sabe quando assenta); quando pára, ENCHE de verdade pelos `QUADROS_PARADOS` (o
contador `parado` do `SheetCache`) e some quando a folha atualiza; tokens, sem texto, dica em inglês («The animation
updates when the paint settles»). Mora em `ph2d-anim-sheets` (estado) + o editor (`ph2d-app-sprite-anim`/`editor_ui`).

Smoke ainda NÃO confirmado pelo dono: Tooth aceso com Grain no Digital (§7 B do handoff) · «Color mixing» nasce dobrado ·
Clear glass (Brush ▸ cartão Wet Paint ▸ caixa **Tuning** ▸ Material ▸ Transmission ▸ Transmission mode — o dono não a
achou sozinho: a Transmission escondida atrás do Tuning é candidata a pergunta de UX).

Fila, por ordem:
1. **Pastel parece spray** (dono 08/11: «parece ruim»). Foto: `material_2026-11-08_pastel/pastel_hoje.png`.
   PERGUNTE o produto de referência dele para pastel (ainda sem resposta; instalados: Krita, MyPaint; antes comparou
   com o Clip Studio). Depois: oráculo CORRIDO (licença por artefacto, entradas nossas, 3 fundos), simular o conserto
   antes de construir, antes/depois lado a lado, o dono faz a triagem.
2. **Memória do 1.º traço** (dono 07/11, «depois do CI»): corte = undo só na caixa do traço, baixar
   `TECTO_DA_MEMORIA_MIB` em `tests/it/presets_de_fabrica/memoria.rs`.
3. Em bloco: marca do Wet por CAMADA (§6.2) · Wet Wash Mix (grão + ~4× lento) · Splash · `SPRAY_FIXO_MAX` com Wet blend ·
   painel antigo a 245 px · dicas longas sem quebra · papel de IMAGEM sem dente com Grain próprio.

Inegociáveis: os 2 alarmes de pincel por nome (regravar só com o dono, `PH2D_PINCEIS_APROVA=1`); Watercolor Basic ao
byte; a shell tem folga de ~44 linhas (`the_shell_only_shrinks`) — código de família vai para `ph2d-app-painter`;
fotos no app por `ph2d-run.sh` em Xvfb/kwin próprio, `import -window`, nunca `spectacle`; texto de UI em inglês;
não integre nem faça push sem ordem do dono. Fecho da linha (antes de integrar): a bateria por nome inclui
`ph2d-anim-sheets` e `ph2d-panel-painter-layers` (tocados nesta reabertura).
