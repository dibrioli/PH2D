# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 48: a W2, os fios e a textura do traço (2026-10-18)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 48](../NewFeaturesWetPaint/48_PROMPT_W2_os_fios_e_a_textura_do_traco.md)
· a medição, as leis e as recusas: [48_medicao](../NewFeaturesWetPaint/48_medicao_os_fios.md) · [prancha](../NewFeaturesWetPaint/48_prancha_os_fios.png)
· o 05 §3 com a W2 ✅ · lista 26 B15, C33, C34. Anterior: [P47](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P47_W6.md).
Commits `de54c0b9a7` → `a68ad40cab` (+ este).

## §2 — Decidido e medido (o detalhe no 48_medicao)

- **A barra da W2 hoje** (os 19 pincéis com cerdas ou grão, à largura da foto e a 40 px): o **período < 15 %** já passava
  nos 19. A barra 5–10 % / CV ≤ 0,02 é do ÓLEO — o Rebelle dos outros meios já a fura; por pincel, a barra é a foto dele.
- **Só os `PARTICLE` do Rebelle têm fios** (15 de 281; o Bristle Flat entre eles). O Round Hard, o Hairy, os redondos de
  aquarela/tinta/marcador são carimbos ⇒ os números pedem fios ao **Oil Bristle** e ao **Express Oil**.
- **Os carris medidos** (`Medida::carril`, o desvio da cobertura AO LONGO de cada linha do miolo): Rebelle oil 3,32 · o
  nosso 0,00; express 0,85 · 0,03 (o ruído do Rebelle na régua: 1,38 · 0,81 ⇒ ela só distingue o óleo).
- **Dois mecanismos:** o óleo do Rebelle varia PELO caminho e em claro/escuro com cobertura cheia (0,909) ⇒ a cor de
  cada fio (saturação e luz HSLuv) + a pressão própria; o Hairy tem faixas quase CONSTANTES de contraste forte ⇒ a
  pressão por fio forte com ruído de 2 000 px.
- **Fábrica:** Oil Bristle textura 2,75 → 3,85 (5,86), carris 0,00 → 3,03 (3,32), CV 0,017 · Express Oil textura 3,20 →
  7,82 (9,00), CV 0,006 (0,008), carris 0,86 (0,85), leve 11,4 · forte 7,0 (10,2 · 7,3). O resto do óleo são as sombras
  do relevo de cada fio: W3 (C34).
- ⛔ Recusas medidas: a pressão forte no óleo para bater 5,9 (falhas brancas no miolo); o ruído curto no Express (carris
  4,95); o matiz por fio (berrante a 0,08 volta); copiar o período 47 % do Hairy; o Perlin sem fase (149 px a 57 %).
- **Acrylic Round** (carris 30,5 contra 3,1): o tapete esparso que o dono afinou em 11/10; nenhuma lei dos fios o cura
  (30 → 21) ⇒ pergunta ao dono (B15). **Wet Dry Brush**: sem foto honesta (C33).

## §3 — O que existe

- `ph2d-wet-paint`: `bristles_fios.rs` (Fios/FiosNoDab/Ponto; `ruido` = Perlin com a FASE do fio + 2.ª oitava; a fronteira
  ondulante, o span e a oscilação na normal; o ganho `max(0, 1 + pressão·ruído − levanta·(1 − contacto))`) +
  `bristles_fios_tests.rs`; `bristles_cor.rs` (HSLuv da especificação pública, valores de referência no teste). 12 knobs
  **Hidden** por nome (`bristlePressure`, `bristleNoiseLength`, `bristleLaneWidth`, `bristleWave`, `bristleWaveLength`,
  `bristleOscillation`, `bristleSpan`, `bristleSpanLength`, `bristleNoiseHue/Sat/Light`, `bristleLift`), `KNOB_COUNT` 83.
- Hospedeiro: `PainterTool::fios_do_pincel` (`wetpaint_settings.rs`); `dab_route.rs` lê o tapete no ponto dos fios × o
  ganho; a cor do fio entra pela porta da Shape Color (sobre ela ou a cor do dab); contacto = raio ÷ nominal. O neutro
  não chama a lei.
- Réguas: `regua_midia` ganhou o PERÍODO (DFT com Hann) e os CARRIS, iguais ao Python nos 19 PPM
  (`ferramentas/carril.py` novo); a régua da ponta (Rust e `midias_regua.py`) não rebenta numa falha entre fios.
- Sonda `tests/it/os_fios.rs::sonda_a_barra_da_w2` (`PH2D_SONDA_PRESETS`, `PH2D_SONDA_KNOBS="n:k=v|…"`, `PH2D_SONDA_RAIOS`).

## §4 — Portões

`os_fios::os_oleos_tem_fios_e_nao_carris` (vermelho antes: 12 falhas; o controlo = o preset sem os fios; CV, período,
carris do Oil no vale, textura e leve/forte do Express no vale e 5–10) · `os_fios::cada_knob_dos_fios_morde` (os 12
nomes; os comprimentos sozinhos ao byte) · 11 unitários · identidade: 18 miniaturas, 20 das 22 cenas de `a_lei_da_tinta`
(as 2 do óleo repinadas no mesmo commit), `os_outros_meios_nao_mudam`, `cada_preset_parece_a_sua_midia`, `a_largura_e_a_prometida`.
**Mutação 11/11** sangram.

## §5 — Fecho

fmt ✓ · `cargo check --workspace --all-targets` ✓ · clippy `-D warnings --all-targets` (`ph2d-wet-paint`,
`ph2d-tool-painter`) ✓ depois de 1 achado (o tipo da variante da sonda num alias, `a68ad40cab`) · `nextest-impacted`
(`BASE=8723538a73`) **18 462/18 464**: as 2 falhas eram REAIS (3/3 sozinhas, load 6,7–13): `o_escuro_fino_sobre_o_claro_seco_…`
(Bug #50) e `a_cor_sobrevive_a_agua_nos_pinceis_de_fabrica` usam o Oil Bristle como VEÍCULO e a cor de cada fio entrava
na régua deles (o controlo sem rampa deixou de ser liso: ×16,5 → ×2,4; só 157 px «cheios» do claro) ⇒ os dois tiram os
fios (`verde_no_soltar_tests::sem_os_fios`, como já tiravam o Emboss) e ficaram verdes por nome. ⚠️ O commit
`de54c0b9a7` diz «13 knobs»: são 12. Binário do smoke: `Finished \`smoke\` profile [optimized] target(s) in 29.77s`.

## §6 — Aberto

**Smoke do dono 18/10: OK.** Próxima janela: o [prompt 49](../NewFeaturesWetPaint/49_PROMPT_W3_o_relevo_de_cada_fio.md) (a W3, o relevo de cada fio).

- B15 (o Acrylic liso?) · C33 (o Dry Brush sem régua) · C34 (o relevo por fio, W3).
- Os 12 knobs dos fios sem painel: W8. Span, oscilação e onda existem e mordem, mas nenhum pincel de fábrica os usa
  (medido: não fecham os carris do Acrylic). Thin/Thick/Edges: nenhum número pediu.
- Na fila (dono 17/10, NÃO juntados): A13–A15, C29, C30, C32; depois W3–W5, W7, W8.
