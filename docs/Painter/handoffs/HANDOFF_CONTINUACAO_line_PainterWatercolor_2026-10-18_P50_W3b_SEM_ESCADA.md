# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 50: a W3b, o relevo sem escada (2026-10-18)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 50](../NewFeaturesWetPaint/50_PROMPT_W3b_o_relevo_sem_escada.md)
· a medição, a lei e as recusas: [50_medicao](../NewFeaturesWetPaint/50_medicao_o_relevo_sem_escada.md) ·
[prancha](../NewFeaturesWetPaint/50_prancha_escada.png) · [foto do smoke](../NewFeaturesWetPaint/50_smoke_cena.png) ·
05 §3 (W3 ◐ com a W3b) · lista 26 C35 actualizado, C36, C37 novos. Anterior: [P49](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P49_W3_RELEVO.md).
Commits `cdaedcaae9` → `04fda78c26` (+ os docs).

## §2 — Decidido e medido (o detalhe no 50_medicao)

- **A régua da escada** (`ferramentas/escada.py` + a gémea `tests/it/regua_escada.rs`, iguais a 0,01): o salto da
  LUMINÂNCIA ao longo dos fios a 1 px, na direcção local deles (tensor de estrutura), no miolo; o controlo é o
  recto horizontal e um padrão sintético (o degrau lê 3,0× o liso). Cenas: a curva em S e `--cena inclinados`
  (8° e 25°, `sonda_gestos::INCLINADOS`). O oráculo corrido: `fixtures/rebelle/{oil103,oil40c,incl103,incl40}`.
- **18/10 lia 15,24 · 12,33** no inclinado a 103 px (Rebelle 2,48 · 2,30; ontem 1,74 · 1,25) e **3,1× o contraste**
  do Rebelle — a textura pela cobertura (o canal mais longe do papel) é cega à sombra na tinta escura e leu 5,87 =
  5,86. A foto do dono lê do lado mau. A cor por fio da W2 (degrau) NÃO é escada (1,81 → 1,98; Rebelle 2,50).
- **O achado de lei:** a luz/sombra do Rebelle soma o MESMO cinza aos três canais (eixo RGB do resíduo 0,9 · 1 · 0,9);
  a luz do Impasto multiplicava a cor (0,13 · 0,6 · 1 = o albedo). ⇒ a altura entra no emboss da tinta.
- **A lei:** cada fio deixa o alvo no CENTRO; entre dois centros, o meio-cosseno (`bristles_relevo::entre`,
  `FiosNoDab::entre_fios`); o campo guarda px ÷ px por célula; `render.rs` soma `(∂x h·½ + ∂y h) × LUZ 20 × Emboss`
  ao emboss da massa (o mesmo clamp ±40).
- **Fábrica:** Oil Bristle `bristleHeight` 0,75 (emboss 0,7, fios 1,5 px como estavam). Textura 3,85 → 5,65 (5,86),
  forte 3,31 → 4,83 (5,04), contraste 8,7 → 13,4 (12,1), borda 0,887, carris 3,07, CV 0,016; escada 1,90 · 1,43 a 103
  px e 2,18 · 1,90 a 41. A largura dos fios medida contra o Rebelle a 103 px: 1,5 px fica (2,5 px tira a faixa de
  2,5–3,5 px que ele tem: 5,8 % contra 24 %; 1,5 px dá 18,6).
- ⛔ Recusas medidas (tabela no 50): o degrau; a luz do Impasto no relevo molhado; a cobertura sozinha como barra;
  altura ≥ 1 (contraste 1,35–1,5×); fios do relevo de 2,5 px; emboss 1 com o relevo.

## §3 — O que existe

- `ph2d-wet-paint`: `bristles_relevo::{entre, LUZ}` (o `alvo(p: &Ponto)` saiu; fica `alvo_do_fio`),
  `FiosNoDab::{entre_fios, fronteira}` (o `fio_no_ponto` saiu — único chamador era o pente), `render.rs` (o termo da
  altura no emboss do `render_pigment_row_visual`), `Grid::altura` em px ÷ px por célula.
- `ph2d-tool-painter`: o pente (`dab_route.rs`) com a tabela por fio alargada de ±1 e o alvo contínuo; **saíram**
  `impasto_light_porta.rs` e `wetpaint/relevo_luz.rs` (o compositor e o `impasto_light.rs` voltam ao de antes da W3,
  ao byte). ⚠️ O cartão Lighting do Impasto já NÃO toca no relevo molhado: o knob é o Emboss do pincel.
- `ph2d-app-painter`: `wetpaint_smoke` nível 2 (`NIVEIS` 2, `arm_hero`, `pinta_o_relevo` pela porta do rato no bind);
  a shell chama `wetpaint_smoke::arm_hero` no lugar de 4 linhas.
- Sondas: `o_relevo_dos_fios::sonda_a_escada` (PPM para a régua; `PH2D_SONDA_KNOBS`, `PH2D_SONDA_RAIOS`),
  `sonda_o_custo_do_relevo` (2 variantes); `regua_escada::a_regua_da_escada_e_a_do_python`.

## §4 — Portões

`o_relevo_dos_fios`: `o_relevo_nao_tem_escada` (8°, 25° e curva a 103 e 41 px ≤ Rebelle + 0,3; contraste do recto no
vale a 41 e ≤ 1,3× a 103) · `o_relevo_e_do_emboss` (Emboss 0 = sem relevo ao byte; as lâmpadas do Impasto não lhe
tocam) · `a_altura_e_por_px_de_tela` (grade 2 px/célula = metade) · `o_oleo_tem_o_relevo_de_cada_fio` (com a fábrica)
· `cada_knob_do_relevo_morde` · `a_borracha_tira_o_relevo`. `regua_escada::a_regua_da_escada_morde_o_degrau`.
Unitários `entre_dois_fios_a_altura_e_continua`, `entre_fios_anda_continuo_de_centro_a_centro`. Identidade: 21 das 22
cenas de `a_lei_da_tinta` (a do Oil Bristle repinada: `0x7041…` → `0x363fa46c11df0df7`) e as outras 19 miniaturas.
**Mutação 7/7** (o degrau, os centros, `LUZ` ×4, sem o Emboss, sem o ÷ px por célula, o relevo apagado, sem o
declive vertical). ⚠️ `LUZ`/Emboss/vertical só os veem os portões de produto (não há unitário do termo no render).

## §5 — Fecho

fmt ✓ · clippy `-D warnings --all-targets` (`ph2d-wet-paint`, `ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-host-desktop`) ✓ ·
`nextest-impacted` (`BASE=e3a8858603`) **18 479/18 479** (962 s) · binário do smoke: `Finished \`smoke\` profile [optimized]
target(s) in 40.78s`. **Custo** (`sonda_o_custo_do_relevo`, 2048², raio 40, mínimo de 7 intercaladas, load 2–5):
move 0,68 → 1,44 ms com o relevo; tick 0,001 → 0,002 ms. **Depois do smoke do dono («o traço ficou lento», `04fda78c26`):** a 103 px o move
era 1,07 → 2,37 ms — o pente recalculava a silhueta do depósito, em série; agora a silhueta é guardada por dab
(`wetpaint/sil_memo.rs`) e o pente anda por linhas (`MIN_CELLS_PENTE` 768, medido): **1,28 ms** (sem relevo 1,12),
ao byte. Medido em release e em smoke (1,53 · 1,35); o smoke do dono continua por refazer.

## §6 — Aberto

**Smoke do dono por fazer** (`PH2D_WETPAINT_SMOKE=2`, os passos estão na resposta ao dono e no roteiro que o app
imprime). Depois: C35 (o leve 5,72 contra 5,01) · C36 (a borda a 103 px 0,73 contra 0,87, de antes; o contraste a
103 px 1,25×) · C37 (as linhas escuras do Rebelle: assimetria −2,2 · −1,1 contra +0,2) · C33 · A13–A15, C29, C30,
C32. Depois da W3: W4 (papel), W5 (mistura), W7, W8 (o painel — inclui os 14 knobs dos fios e do relevo; o Lighting
do Impasto já não é a lâmpada do relevo molhado).
