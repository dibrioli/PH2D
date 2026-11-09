/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-09_pastel_motor.md

Estado (09/11, tarde): linha `line/PainterWatercolor`, base `main` `f254e42bf1`, por integrar. Feito nesta janela:
- **Selo de progresso** no canto da prévia do cartão Animation (pedido do dono 09/11): gira enquanto a tinta/água anda,
  o mesmo arco cresce até fechar o anel pelos `QUADROS_PARADOS`, vira check verde que vive 1,2 s de relógio e some; dica
  «The animation updates when the paint settles». `8b6b4a253a` + `394a84dc09` (+ fotos `7029da236c`). Peça
  `ProgressPhase`/`paint_progress_badge` em `ph2d-editor-core/src/widget/spinner.rs`; estado `SheetCache::sync` em
  `ph2d-anim-sheets/src/sheet_cache.rs` (`follow_live` recebe `agora_ns`); pintor `ph2d-panel-inspector/src/sections/anim_card.rs`.
  Fotos em `docs/Painter/handoffs/material_2026-11-09_selo/` (tira `8_…depois_zoom.png` = as 4 curas). ⚠️ Com o Painter
  ligado o painel direito vira Brush: o cartão Animation vê-se clicando **Inspector** (topo direito). **Smoke do dono 09/11: OK**
  (`PH2D_GAME_SMOKE=5`, Two Sheets ▸ Painter ▸ Inspector ▸ pintar).
- **Pastel, fase 1 e 2** (dono: «com os três e veja o mais realista»; depois «siga como achar melhor, buscando o estado da
  arte»). LEIA `docs/Painter/handoffs/material_2026-11-09_pastel/README.md` (as duas rodadas, tabelas) e veja
  `resumo_para_o_dono.png` e `folha_variantes.png`. Oráculo mais realista: **Krita h) Chalk Soft**. O nosso = spray
  (`spray_count 20`, grãos 2 px sorteados). Só por receita, **H_d08_j07** (sem spray/jitter/dab_jitter, papel próprio =
  `PAPEL_ASPERO` com `lei {mode: 4 (Height), depth_pressure, depth_jitter 0.7, depth_min 0}`, `grain_depth 0.8`,
  `hardness 0.85`, `falloff Smooth`, `opacity 1.0`) casa com o Chalk Soft na pressão 1 e pousa nos picos à pressão baixa.
  Sonda: `sonda_csp_v3_revisao` com `PH2D_V3_PRESSAO=rampa`, `PH2D_V3_PASSADAS=<n>` (`ea0a341fea`; `PH2D_V3_VARIANTES` ABSOLUTO;
  nunca duas corridas por `ph2d-run.sh` ao mesmo tempo).

PRÓXIMO — o Pastel do estado da arte, num BLOCO (Sousa & Buchanan 1999; Rudolf, Mould & Neufeld 2003/2005): o que a receita
não faz e precisa de MOTOR (`ph2d-painter-brush/src/texture/lei.rs` modos; `dab_spec.rs:112-146` papel/mordida):
1. **Limiar suave e estreito** (largura ≈ 0,1) no lugar do `Height` binário: grãos com borda macia a meia pressão.
2. **Penetração = pressão × queda do dab**: a borda do traço quebra-se pelo papel (hoje a borda é o disco redondo, 2 px).
3. **Enchimento do dente**: altura efectiva por CAMADA `h + k·desfoque(alfa pousado, ~1,5 px)`, k ≈ 1 — a 2.ª passada cobre
   os vales (simulado: vales a 0,5/0,65/0,8 de pressão vão de 0,00/0,16/0,68 a 0,33/0,77/0,95; o motor hoje repinta os
   mesmos píxeis). Cuidado com o undo da camada e com o custo (medir antes de limitar).
4. **Grão fino**: os buracos de hoje são manchas do `Noise` de 48 px; o Chalk Soft tem grão de 3 px — escala do papel do Pastel.
Método (memórias `feedback_brush_method_owner_triage_and_simulate_before_building`, `feedback_brush_oracle_must_include_blank_canvas`):
receita H_d08_j07 primeiro, cada peça de motor medida contra o Krita (pressão 1, rampa, 2 passadas, 3 fundos) na mesma sonda,
folha lado a lado ao dono. Os alarmes de pincel por nome vão reprovar no Pastel: relatório ao smoke, regravar SÓ com
`PH2D_PINCEIS_APROVA=1` depois do sim do dono. Que outros pincéis secos (Chalk em `csp_secos.rs:81-105`, Crayon) herdam a lei
é decisão a mostrar ao dono, não a tomar sozinho.

Fila depois: memória do 1.º traço (`TECTO_DA_MEMORIA_MIB`, dono 07/11) · em bloco: marca do Wet por camada · Wet Wash Mix ·
Splash · `SPRAY_FIXO_MAX` com Wet blend · painel a 245 px · dicas longas sem quebra · papel de imagem sem dente com Grain.
Smoke ainda não confirmado: Tooth com Grain no Digital · «Color mixing» dobrado · Clear glass (o dono perguntou o que ver:
explicado; perguntado se a Transmission sai do Tuning — sem resposta).

Inegociáveis: alarmes de pincel por nome; Watercolor Basic ao byte; shell com folga ~43 linhas (`the_shell_only_shrinks`);
fotos em Xvfb puro + xdotool (XTEST chega) ou kwin próprio, `import`, nunca `spectacle`/`ydotool`; UI em inglês; não integre
nem faça push sem ordem. Fecho da linha: bateria por nome inclui `ph2d-anim-sheets`, `ph2d-panel-painter-layers`,
`ph2d-editor-core` (spinner), `ph2d-panel-inspector` (`as_folhas_do_animated`) e `ph2d-tool-painter` (sonda).
