# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (depois da retirada da secção Rendering)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md e ESTE ficheiro inteiro. Delegue leitura e verificação.
```

## O que a janela 5 fez (commit «secção Rendering retirada + Hardness», ver `git log -3`)

1. **Secção Rendering do Digital retirada** (ordem do dono 24/10): saem Glaze/Wet edges/Burnt edges/Edge width
   (`TracoComposto`, `traco.rs`, `traco_orla*.rs`, `traco_aguada.rs`, 8 ficheiros de teste das orlas, a sonda
   descartável `sonda_opacidade_tests.rs`). O «On empty» (`PAINTER_BRUSH_BLEND_ON_EMPTY`, mesmo hash de antes) mora
   logo abaixo do Blend (`paint_brush_rows::paint_blend_on_empty_row`); porta em `blend_vazio_settings.rs`.
   - O **plano do traço** fica, sem botão: arma num modo de cor (`BrushBlend::e_de_cor`), com a mistura de pigmento
     (`pigmento().mix > 0`), com a cauda (L5) ou o dual (L6) — `traco_plano::pede_o_plano`. Num Multiply/Burn/Dodge
     (Marker Liner, efeitos) e no pigmento (Ballpoint) o traço não escurece a si mesmo.
   - `BrushSpec::tecto_por_dab` (era `traco.tecto_por_dab`); JSON grava `"tecto_por_dab": true` só ligado; o legado
     `traco.tecto_por_dab` lê-se em `json.rs::pincel_de_json`; tecto/orlas legados ignoram-se.
   - Fábrica Digital regravada (20 ficheiros). Glaze Round: Accumulate off, **Strength 0,65 · Flow 0,05**
     (5 variantes medidas em `cada_basico_se_reconhece_pela_foto`: opacidade 0,239/foto 0,231, tecto 1,77/1,85).
     Dry Marker (em estudo): tecto → Strength 0,9.
   - Sonda `sonda_secao_render` (antes × depois): Ballpoint, Burn, Dodge, Color Glaze, Soft Round ao byte; Marker
     Liner ≤ 2/255; miniaturas iguais salvo Glaze Round e 4–11 px nos efeitos/Liner.
2. **Hardness à vista** (dono 24/10: o Glaze Round de fábrica, dureza 0,8, pintava um disco chapado e o preview
   mostrava o Smooth inteiro): linha `panel.painter_layers.shape.hardness` logo abaixo do Falloff
   (`PAINTER_BRUSH_HARDNESS`, `set_brush_hardness`, `BrushSettings::{hardness, dureza_do_dab}`); o preview da silhueta
   usa `brush_dab_weight_at` (= `BrushSpec::falloff_weight` ao bit, gate `dureza_tests`); o editor da curva continua
   `brush_falloff_weight_at`. Gate do painel `seam_hardness`.
3. **Diagnóstico medido (sondas `sonda_falloff_do_traco`)**: o corte de um traço é mais duro/largo que o falloff de
   um dab (lei da cobertura `1−Π(1−w·flow)` desde 25/07 — MEDIDO igual no código de 18/10, não nasceu com Glaze/Flow);
   o toque único é exacto (40/150/300 px, Flow 1/0,5, com/sem airbrush, depois de trocas). Accumulate off obedece
   (nunca passa a Strength), mas com Flow baixo a volta por cima escurece até ela (0,24 → 0,39 → 0,55 no Glaze Round).

## ⚠️ POR FAZER antes de mais nada

- **A bateria por crate NÃO terminou** (o verificador foi interrompido): `ph2d-painter-brush`, `ph2d-tool-painter`
  (lib + it), `ph2d-panel-painter-layers`, `ph2d-app-painter`, `ph2d-i18n`, clippy `--all-targets -D warnings` e fmt
  das 5 crates. `motivo:` válido: formato gravado (BrushSpec mudou). Só passaram, por nome: `dureza_tests`,
  `seam_hardness`, `cada_basico_se_reconhece_pela_foto`, as sondas.
- A sonda `sonda_cartao_pen.rs` (do auditor) está no commit; `foto_pen_tests.rs` está em
  `/tmp/claude-1000/.../scratchpad/` (fotos `pen_340/260/220.png`).

## Decisões do dono por pedir (curto, sem jargão)

1. **Lei do Accumulate desligado = a borda EXATA do falloff e o traço nunca escurece a si mesmo** (o «Wash» do
   Krita, `max` em vez do produto; as «contas» de 25/07 curam-se com o espaçamento contínuo da fábrica). Muda o
   aspecto de todos os pincéis do Digital (re-medir os aprovados) e o Flow perde o papel de taxa com Accumulate off.
2. **Parâmetros escondidos** (censo do explorador, 25/10) — mostrar no painel ou fixar e esconder de propósito:
   - Digital, sem linha: `pigment_espectral` (Ballpoint), `passo_pela_pressao` (Bokeh, Stars), `borda_px` (Hard
     Round 1, Flat 1, Hard Eraser 1, Halftone 3), `tecto_por_dab` (Marker Liner), `sub_passo_fino` (Flat),
     `particle_max_px` (Stars 56).
   - Wet Paint: 58 knobs no grupo `Hidden` (`ph2d-wet-paint/src/tuning/defs.rs`), ~30 fora do neutro nos presets
     (`wetOpen`, `tipReservoir`, `pressureGrain`, `levelingOwnStroke`, `strokeFilm`, `dust`, cerdas…).
     Scripts do censo: `scratchpad/census*.py`.
3. **Reset do cartão Pen**: volta ao neutro da app (hoje) ou ao preset?

## Auditoria do cartão Pen (25/10) — defeitos a corrigir (técnicos, decisão delegada)

1. ALTA — Min/Max pressure e Smoothing esmaecem («Acts when Pressure size or Pressure opacity is above 0») mas agem
   via Pen hardness, Light scatter, grão/imagens pela pressão: predicado `inercia.rs:648-651`, ids `:198-202`;
   Pressure curve (id 18) fora da lista.
2. ALTA — **Speed softness endurece** abaixo de 300 px/s (`dynamics.rs:164-171`: `k·ln((17,6+300)/(17,6+v))`, no
   clique +2,89·k; Soft Round borda 19 px → 0 com k=1); a chave `speedHardness` diz o contrário do texto. Cura: zero
   em v=0, só subtrair (`−k·ln((Γ+v)/Γ)`), clique = Hardness; nome alinhado com a chave.
3. ALTA — o arranque de todo traço rápido sai duro com Speed softness: os dabs guardados até à 1.ª medição da
   velocidade (`speed.rs:80-96`) recebem a dureza com v=0 (`dab_build.rs:268-274`) e `solta_a_velocidade`
   (`speed.rs:126-148`) não a recalcula.
4. MÉDIA — Pen hardness (`pressure_hardness·p`, `dynamics.rs:176-183`) age com o RATO (p=1 sempre; quebra «o rato
   não muda nada»). Cura `−k·(1−p)`.
5. MÉDIA — Tilt smooth e Rotation sem efeito neste pincel e não esmaecem; mexer neles acende Tilt start/full/lag
   inertes (`pen.rs:165`, `inercia.rs:579-582`).
6. MÉDIA — faixa do Speed softness 0..1 satura o clique a ~0,35 (glow do MyPaint 0,038): tecto ~0,3.
7. BAIXA — nomes: «Pen hardness» → «Pressure hardness»; «Light scatter» → «Light-touch scatter»; «Pressure curve»
   → «Pressure S-curve»; «Press. opacity» por extenso (`painter_layers.rs:545-553`).
8. BAIXA — casas decimais incoerentes («0.700» ao lado de «1»): a caixa usa `format_number`
   (`ph2d-editor-core/src/widget/number_input/mod.rs:458-462`) e ignora `casas()` (`paint_pen.rs:59-61, 108`).
9. BAIXA — sem unidades (Pulled string px, Tilt start/full °, Tilt lag px, Speed lag s).
10. BAIXA — a linha Rotation não alinha (menu até à borda, sem o ponto do Auto Key, `paint_pen.rs:116-140`).
OK: Pressure size/curve, Press. opacity/curve, Min/Max pressure, Smoothing, Pulled string, Tilt width/length/
opacity/start/full/lag, Speed size/opacity/lag, Pressure curve, Light scatter, Reset; nenhum rótulo corta a 220 px.

## Herdado (da janela 4, ainda aberto)

Acelerar a água do Wet (dono escolheu); Borracha da Watercolor que apaga mantendo a sessão (dono escolheu); a
família «mundo» das macros lê a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da Borracha no Wet;
Texture Eraser à Strength 1; a shell a 23 linhas do tecto.
