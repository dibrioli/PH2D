# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 47: a W6, a caneta no pincel (2026-10-18)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 47](../NewFeaturesWetPaint/47_PROMPT_W6_a_pressao_e_a_inclinacao_do_pincel.md)
· a medição e as leis: [47_medicao](../NewFeaturesWetPaint/47_medicao_a_caneta_no_pincel.md) · o 05 §3 com a W6 ✅ · lista 26 C31 ◐.
Anterior: [P46](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P46_CANETA.md). Commits `52e24e9439` → `61ff16137c` (+ este).

## §2 — Decidido e medido (o detalhe e as tabelas no 47_medicao)

- **A rampa** ⛔ REVOGADA pelo dono no smoke de 18/10 («apertar muito e pouco variam pouco a espessura; antes funcionava
  melhor»): o piso que punha as pontas nas do Rebelle (9 pincéis) comia a resposta da mão dele (0,24–0,86 ⇒ ~1,3× contra
  ~2,7×). Os 9 voltaram aos ficheiros de `8ff6591c79` (as 22 cenas ao byte); `Metrica::Pontas` fica só no aerógrafo e nos
  Flats. ⚠️ Não reabrir as pontas do Rebelle pelo piso: se voltarem, é por outra via (a curva do tamanho, `pressureSizeCurve`,
  ou o tecto), e medida contra a faixa da mão do dono.
- **O lápis deitado** — o Rebelle CORRIDO a 0–63° ao longo/através/diagonal (`fixtures/rebelle_tilt/`, sessão nova): a ponta
  é uma ELIPSE ao longo do azimute; nada abaixo de ~39°, inteira a 45°; atraso ~170 px de caminho. Lei `pen::Tilt` +
  `DabTilt::compose` (SVD 2×2, exacta com pincel achatado); o 8B: 39–45°, 3,3 × 1,05, lag 170, `tiltSmooth` 0 (o grão
  alisa-se pela sobreposição — medido; o 0,43 alargava 1,58×). Ao longo 1,27× / textura 0,62× / +74 px; através 3,46×;
  diagonal 2,54× (Rebelle 1,24 / 0,65 / 79 / 3,45 / 2,59).
- **Suavização**: `MAX_SMOOTHING` 0,9 (45 ms = o tempo mediano da mão do dono ao pico; os 180 s).
- **Pulled String** substitui o lazy-mouse quando ligado; o pen-up acaba onde o pincel está (sem catch-up).
- **Rotação** `None` (os slots decidem, como antes) · `FollowTrajectory` (entra no `follows_stroke` e no aquecimento) ·
  `PenTilt` (o azimute composto no `Dab::rotation`). Sem barril (dono 08/10).
- **C31**: o piso/tecto/curva são do PINCEL (como o `pressure_max` dos presets do Rebelle), não duplicam a curva do KDE.
  Medido event-a-event com o dono: o KDE entrega a pressão crua (`medicoes_2026-10-18/`).

## §3 — O que existe

- `ph2d-painter-brush`: `pen.rs` (Pen, Tilt, DabTilt, DabRotation, `pulled_string_step`, `any_tilted`), `stroke/pen_input.rs`
  (`Stroke::set_tilt`, a pressão LIDA no `begin`/`extend`, `deita`), `Dynamics::size_curve` (a lei `dynamics::curve`, a
  MESMA da opacidade), `Dab::tilt`, `BrushSpec::pen_tilt`; as portas `dab_footprint_of` (pegada) e `at_dab` (pincel por dab;
  o grão `× (1 − smooth)`), `tilt_scale`; `FootprintDeform::frame/with_frame`. Todas as rotas passaram às duas portas.
- **Caches**: decidem pelo LOTE (`pen::any_tilted(dabs)`), nunca pelo pincel — com o rato as rotas são as de sempre ao bit
  mesmo com a lei ligada (decidir pelo pincel mudava bits no Digital; medido e recusado).
- `ph2d-wet-paint`: 12 knobs (`pressureSizeCurve`, `pressureFloor`, `pressureCeiling`, `pressureSmoothing`, `pulledString`,
  `dabRotation`, `tiltStart`, `tiltFull`, `tiltWidth`, `tiltLength`, `tiltSmooth`, `tiltLag`), `KNOB_COUNT` 71; i18n.
  Hospedeiro: `dinamica_do_traco` (Pen) e `inclinacao_do_pincel` (no `authored_spec`). Censo: `pen_tilt` em `CAMPOS_FORA`
  (67) e na macro (`json/meio.rs`).
- ⚠️ O gerador de presets MANTÉM uma chave antiga que sai da receita: escreva o neutro explícito (o `tiltSmooth` 0 do 8B).

## §4 — Portões

`pen_tests` 16 (neutros ao bit; piso/tecto; a curva do tamanho = a da opacidade; a suavização no ORÁCULO dos 180 s —
`pressao_por_toque.txt`, 105 toques; a corda; a composição dos eixos; a zona morta; a elipse; o grão só ao comprido; o
PenTilt roda; o atraso cresce pelo caminho) · `tests/caneta_deitada` (lib: os 5 meios + o rato ao byte com a lei ligada) ·
`it/a_caneta_deitada` (o 8B contra o Rebelle; o rato e a caneta de pé ao byte) · `Metrica::Pontas` +
`a_regua_das_pontas_e_a_do_python` · identidade verde: `os_outros_meios_nao_mudam`, miniaturas, as 22 cenas.
**Mutação 12/12** (10 de 12 na 1.ª ronda; os 2 sobreviventes — a rotação do PenTilt e o atraso — ganharam gate próprio).
Sondas: `sonda_o_lapis_deitado_por_variante` (PH2D_SONDA_KNOBS), `sonda_as_metricas_das_variantes` (já existia).

## §5 — Fecho

fmt ✓ · `cargo check --workspace --all-targets` ✓ · clippy `-D warnings --all-targets` (6 crates) ✓ · `nextest-impacted`
(`BASE=8ff6591c79`) **18 920/18 927**: 4 defeitos desta onda, curados em `6212cc6270` e refeitos por nome ✓ — 3 rótulos com a
unidade no texto (`no_row_label_carries_its_own_unit`), `tuning/defs.rs` 768 e `stamp_banded_tests.rs` 701 linhas (o tecto),
o painel a contar 33 linhas (agora 45), e o `sem_po_o_fluxo_do_pincel_nao_anda` (usa o Soft Pastel, cujo piso mudou: o portão
fixa agora o piso de quando foi pinado e o hash é o de antes, provando que só o piso o mudou); 3 timeouts de carga
(`load 77`) verdes sozinhos: `sem_glaze_…` 12,7 s, `o_glaze_do_pincel_…` 8,9 s, `a_largura_e_a_prometida` 73,5 s.
Binário do smoke (`cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida): `Finished \`smoke\` profile [optimized]
target(s) in 0.53s`.

## §6 — Aberto

- ✅ C31 medida com o dono (18/10): o KDE não aplica curva (máx 0,0051 = o arredondamento do libinput; 47_medicao §2).
- Os knobs da caneta só se ligam no Wet Paint (o mapa `engine` do preset, painel Tuning): os outros 4 meios têm a lei (o
  gate dos 5) e nenhum sítio onde a armar — chega com A15 (presets em todos os modos) / W8 (o painel).
- Aquarela com «Automatic»: carimbo redondo dela, a elipse só com ele off (a regra do Flatten).
- O degrau 55→0 do Rebelle corta em ~15 px (o nosso atraso é simétrico): não explicado, não replicado.
- Suavização/corda sem pincel de fábrica (a escala do `pressure_smoothing_strength` do Rebelle está por medir).
- Na fila (dono 17/10, NÃO juntados): A13–A15, C29, C30, C32; depois W2, W3–W5, W7, W8.
