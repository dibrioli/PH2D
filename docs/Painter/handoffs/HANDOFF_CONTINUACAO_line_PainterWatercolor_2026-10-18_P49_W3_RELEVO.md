# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 49: a W3, o relevo de cada fio (2026-10-18)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 49](../NewFeaturesWetPaint/49_PROMPT_W3_o_relevo_de_cada_fio.md)
· a medição, as leis e as recusas: [49_medicao](../NewFeaturesWetPaint/49_medicao_o_relevo_de_cada_fio.md) · o 05 §3 com a
W3 ✅ · lista 26 C34 fechado, C35 novo. Anterior: [P48](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P48_W2_FIOS.md).
Commits `1ced4b253b` → `13a54a7618` (+ este).

## §2 — Decidido e medido (o detalhe no 49_medicao)

- **A barra da W3 de 08/10 estava velha:** o «borda/miolo hoje 0,52» era o Wet Round; o Oil Bristle já estava em
  0,83–0,90 (Rebelle 0,872). O número que faltava era a TEXTURA (3,85 contra 5,86) — as sombras dos fios.
- **O relevo de hoje seguia o TAPETE, não os fios:** o emboss (`render.rs`) é o gradiente da massa, e a massa do óleo
  varia 18,9 % através SEM os fios (18,5 % com). Emboss 2 sozinho dava 6,97 com carris 4,49 ⛔.
- **A lei nova:** cada fio deixa `bristleHeight × ganho × contacto²` px, para cima se constrói e para baixo se escava
  (`bristleRemove`, a fracção, por hash do fio); o dab penteia a célula (taxa 0,5 × cobertura). A luz é a do
  Impasto (o rig do artista) por uma porta livre; a luz satura a ~2 px em fios de 1,5 px.
- **Fábrica:** Oil Bristle `bristleHeight` 1,25 e emboss 0,7 → 1: textura 5,87 (5,86), carris 3,26 (3,32), forte
  3,31 → 5,29 (5,04), borda 0,895, CV 0,017, período 2 %. O leve sobe 5,36 → 6,32 (5,01): C35.
- ⛔ Recusas medidas: a oleosidade como knob (largura molhada = seca em todos os óleos, viscosidade 0–2, até com água
  1); o relevo no Express Oil (7,94 → 8,79 dentro do ruído da foto, 9,0 · 9,9); alturas > 2 px; escavar 0,65; o
  contacto¹; o emboss da massa sozinho.
- A régua ganhou a textura ATRAVÉS × AO LONGO (fotos: óleo 5,84 · 1,09); os 5,48 · 0,98 do 48 não se reproduzem.

## §3 — O que existe

- `ph2d-wet-paint`: `bristles_relevo.rs` (+ testes), `Grid::altura` PREGUIÇOSO (`grid/altura.rs`: `garante`, `em`,
  `restaura_janela`; o instantâneo, os recortes e o limpar), `painter/relevo.rs` (`Engine::pentear_relevo`),
  `Fios::resolvido` (as faixas mesmo no neutro). Knobs Hidden `bristleHeight` (0 = a lei não é chamada) e
  `bristleRemove` (0,5), `KNOB_COUNT` 85.
- `ph2d-tool-painter`: `impasto_light_porta.rs` (`LuzLivre`: `Rig::shade` + `paint_body`/`gloss_body`/`light_pixel`
  — `apply_impasto_light` intacto); `wetpaint/relevo_luz.rs` (a linha de pigmento acesa antes do `over`);
  `PainterTool::relevo_do_pincel`; o dab penteia e a borracha alisa (`dab_route.rs`).
- ⚠️ O cartão **Lighting** só aparece no modo Impasto: a lâmpada do relevo molhado regula-se lá (W8: o painel).
  O que já foi pintado guarda a luz com que foi aceso (como o emboss).
- Sondas: `o_relevo_dos_fios.rs::sonda_a_barra_da_w3` (a imagem + o CAMPO), `sonda_a_largura_molhada_e_seca`,
  `sonda_o_custo_do_relevo`.

## §4 — Portões

`o_relevo_dos_fios`: `o_oleo_tem_o_relevo_de_cada_fio` (vermelho antes: 6 falhas com o preset de ontem) ·
`cada_knob_do_relevo_morde` · `o_relevo_e_so_da_luz` · `a_borracha_tira_o_relevo` · 6 unitários. Identidade: 21 das
22 cenas de `a_lei_da_tinta` (a do Oil Bristle repinada no commit da lei), as outras 19 miniaturas, Express e Acrylic ao
byte. Os veículos da W2 (`os_fios::sem_fios`, `sem_os_fios`) tiram também o relevo. **Mutação 15/15** sangram
(o tipo do fio, o contacto², o ganho, a cobertura do pente, a luz, o declive em y, o filtro do compositor, a borracha,
o hospedeiro sem relevo, os recortes/instantâneo/limpar do campo, a régua através, e a ESCALA da altura — esta
sobrevivia aos portões de produto (a luz satura) e ganhou `relevo_luz::tests::a_altura_e_em_px_de_tela`).

## §5 — Fecho

fmt ✓ · `cargo check --workspace --all-targets` ✓ · clippy `-D warnings --all-targets` (`ph2d-wet-paint`,
`ph2d-tool-painter`) ✓ · `nextest-impacted` (`BASE=8723538a73`) **18 475/18 475** · binário do smoke: `Finished
\`smoke\` profile [optimized] target(s) in 41.99s`. **Custo** (`sonda_o_custo_do_relevo`, 2048², raio 40, 7 rodadas
intercaladas, o mínimo, load ~22): move 0,99 → 1,55 ms com o relevo (a luz ≈ 0; o pente pela caixa da PEGADA e o ganho
tabelado por fio tiraram 2,1 → 1,55, ao byte); o tick não muda (0,002 ms).

## §6 — Aberto

**⛔ Smoke do dono 18/10: REPROVADO** — «ficou pior, muito pixelado, talvez porque as cerdas tenham ficado mais finas»
([foto](../NewFeaturesWetPaint/49_smoke_dono_pixelado.png)). O relevo SAIU da fábrica (o Oil Bristle de ontem ao byte,
a cena `0x7041…` de volta); a lei fica no motor, desligada; os portões leem-na por `com_relevo()`. A bancada horizontal
não via a escada (49 §7). **Próxima janela: o [prompt 50](../NewFeaturesWetPaint/50_PROMPT_W3b_o_relevo_sem_escada.md).**
Os números da fábrica em §2 são os da lei com os knobs, não os da fábrica. Depois da W3: W4 (papel), W5 (mistura), W7 (sem água), W8 (o painel — inclui os 14 knobs
dos fios e do relevo e o Lighting no Wet Paint).

- C35 (o leve do Oil Bristle) · B15 (o Acrylic liso?) · C33 (o Dry Brush sem régua).
- Na fila (dono 17/10, NÃO juntados): A13–A15, C29, C30, C32.
