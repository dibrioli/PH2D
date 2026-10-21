# Exame da família `aerografo` (J3, catálogo J)

Receitas e portões: `tests/it/familias/aerografo.rs`. Pranchas: `pranchas/aerografo/`. Réguas e fotos novas:
`exames/aerografo/`; a régua do portão por foto: `aerografo_foto.py`. Custo: `--custo` smoke, p95 a 5 · 40 · 200 px.

**Aprovados: nenhum.** **O Airbrush foi curado e fica na fábrica, mas está reprovado no grão (motor).**
**Reprovados: Spray Can, Stipple, Splatter, Spray Drips. Grainy Shader fundido no Airbrush.** Os cinco saíram da fábrica.

## J-1 Airbrush (Wet; já estava na fábrica): curado, REPROVADO no grão

- **Receita:** a do P43/P51 com a cura J3: `pigment` 9 → **10,5**, `pressureOpacityCurve` 1,5 → **1,2**. O `dust`
  fica em 0,8. Com 9/1,5 o pó pousava 12 % menos que o Spray 2 (opacidade 0,49 × 0,557, a cor mais clara em 9 traços)
  e as pontas da rampa 0,15→1→0,15 sumiam (0 × 0,61).
- **Oráculo:** Rebelle `AIRBRUSH/default_spray_2`. Régua **412 → 444/454**; ficam de fora a textura e a cor do recto de
  5 px e a largura a 200 px (156 × 161). O 2.º oráculo, Krita `b) Airbrush Soft`, dá 166/454: é um disco macio liso, que
  já é o **Soft Round (A-2)**.
- **Vizinho:** Square Hard **0,706** (depois Grainy Shader 0,87–1,00, antes de este sair). Razão de uso: névoa de pó fino
  para degradês e luz.
- **«Acumula enquanto se segura» — REFUTADO pelos oráculos.** Pus uma cena parada nova (`exames/aerografo/bateria_parado.json`)
  e corri os três oráculos (cobertura do miolo a 10 · 60 · 200 eventos):
  - Rebelle Spray 2, foto nova `default_spray_2_parado_A.png`: **0,053 · 0,060 · 0,057**, UM sopro.
  - MyPaint `deevad/airbrush`: 0 · 0 · 0.
  - Krita Airbrush Soft: 0 · 0,10 · 0,10.
  - O nosso: 0,017 · 0,017.
  - Quem acumula é o spray do Krita: 0,24 · 0,92 · 1,00.
- **Revisão cega r1** (chave `chaves/airbrush.json`): escolheu o NOSSO como original nas 6 linhas, com confiança baixa.
  *«a imitação tem um spray grosso e contrastado … uma orla de pontos soltos; o original é uma névoa fina, contínua e lisa»*.
  **r2:** achou o Rebelle nas 6, com confiança baixa: *«o original é feito de gotas grossas, irregulares e de contraste alto;
  a imitação é uma névoa fina, uniforme»*. E as bordas esfiapadas.
- **Cruzamento (decisão 12):** dois revisores repetem a mesma diferença, o grão do Rebelle mais GROSSO. Tentei a cura com
  `dust` 0,85 e 0,9: a textura a 200 px sobe de 21 para 25–28 (Rebelle 31). Mas isso tira as pontas de
  `midias_de_fabrica::cada_preset_parece_a_sua_midia` da barra 0,6 (dá 0,53–0,59, mediana de 7 tiragens), e nenhum
  `pressureSize` ou curva casa as duas coisas. O pó só fica mais RALO, a gota não cresce: `GOTA_DO_PO` é uma constante do
  motor (`ph2d-wet-paint/src/trail/deposit.rs`). ⇒ **fila: a gota do pó por preset.** Fica na fábrica (pincel do dono,
  P44/P51): sair é decisão do coordenador.
- **Custo:** p95 **0,86 · 4,81 · 10,93 ms**, RSS 71–93 MiB, load 12–13.
- **Portões:**
  - `cada_aerografo_se_reconhece_pela_foto`: largura, cobertura, vão, pontos, gota, matiz e rampa contra a foto. Os
    IRMÃOS são as fotos do spray e do pontilhismo do Krita. Mutação `pressureOpacity` 0 → rampa 0,537 × 0,131 ± 0,12:
    **SANGRA**.
  - `o_aerografo_parado_pousa_um_sopro` (10 × 200 eventos, |Δ| ≤ 0,02). Sem mutação de produto: nenhum knob faz o Wet
    acumular parado. Sobrevive, e está declarado.
  - O `midias_de_fabrica` de sempre fica verde.

## J-2 Spray Can (Digital): REPROVADO (motor)

- **Receita medida (saiu):** `spray_count` 16, `spacing` 0,015, `jitter` 0,7, partícula 0,09, `jitter_scale` 0,95,
  através 0,4, opacidade por gota ±0,5, dureza 0,2, `flow` 0,65; a pressão vai para a opacidade.
- **Oráculo:** Krita `y) Texture Spray`, régua **430/454**, escala 1,055. Vizinho: Airbrush **1,901**.
- **Porque não o Rebelle Spray 3:** dá 218/454. Dentro do Rebelle, Spray 2 → Spray 3 = **0,44** (`matriz/oraculos.md`), um
  degrau do Airbrush e não a lata de núcleo com gotículas que o catálogo descreve. A identidade vem do Krita.
- **Revisão cega r1:** achou o Krita nas 6 linhas, com confiança ALTA. As três diferenças:
  1. *«não há véu de fundo na imitação»* (o Krita pinta névoa contínua e pontos por cima);
  2. *«o traço de 200 px é outro»* (o Krita fica quase sólido, de gotas pequenas);
  3. *«os pontos da imitação são uniformes e espalham mais»*.
- **A medição confirma, e nada disto se exprime com o motor de hoje.** Variantes tentadas: partícula 0,035–0,09, passo
  0,006–0,05, forma de ruído por dab. Isto vai para a **fila**:
  - **(a) Gota em px.** A gotícula do Krita não escala com o pincel: a 5 px é fina e clara, a 200 px é uma massa de gotas
    pequenas. A nossa partícula é uma fracção do pincel.
  - **(b) Duas populações.** O véu macio e as gotas duras.
  - **(c) Emissão pelo tempo parado.** Krita 0,24 → 1,00; o nosso `Space` dá 0.

## J-5 Stipple (Digital): REPROVADO (motor)

- **Receita medida (saiu):**
  - `spray_count` 2, `spacing` 0,28, `jitter` 0,42, partícula 0,42, `jitter_scale` 0,45, redondeza 0,3, rotação 1;
  - matiz/sat/valor ±0,02 · 0,05 · 0,2, `pressureSize` 0,3.
- **Oráculos:** Krita `v) Texture Pointillism`, régua **294/454**. MyPaint `classic/puantilism`, régua 262/454 (receita 1).
  Vizinho: Wax Scrape **2,07**.
- **r1:** escolheu o NOSSO como original nas 6 linhas, com confiança média. Listou o tamanho, a densidade, o tom e a pressão.
- **r2:** achou o Krita nas 6, com confiança média.
  1. *«o original mantém a partícula fixa e só alarga a dispersão: a 5 px um cordão grosso, a 200 px tufos esparsos»*;
  2. a pressão (o Krita não responde);
  3. a cor: o Krita é ultramar mais claro, o nosso petróleo mais escuro, e as elipses opacas contra círculos translúcidos.
- **Leitura:** a lei do tamanho repete-se em r1 e r2. Pede a **gota em px** (fila, a mesma do Spray Can). O tom do Krita é
  um sorteio enviesado (média (48,110,210) sobre a base (23,101,179)); o nosso jitter de cor é simétrico.

## J-3 Splatter (Digital): REPROVADO (motor)

- **Receita medida (saiu):** `spray_count` 8, `spacing` 0,6, `jitter` 0,45, partícula 0,25, `jitter_scale` 1, através
  0,35, contagem sorteada 0,6, dureza 0,95, `pressureSize` 1.
- **Oráculos do catálogo e o que não deu:** MyPaint `tanda/splatter-04`, régua 100/442 à escala 1. À escala da bateria
  é um COLAR de discos com uma gota gigante no pouso de cada traço. `speedSize` 0,9 não a reproduz e apaga o rápido.
  O `deevad/spray2` a 40 px cobre a tela inteira. Por isso o exame foi **pela descrição** (Procreate Splatter /
  PS Kyle's Spatter).
- **Vizinho:** Acrylic Round **3,06**.
- **r1 e r2** (pela descrição, contra o Stipple/Acrylic Round e o MyPaint): escolheram o nosso nas 3 cenas, com confiança
  ALTA, e *«claramente diferente dos outros dois»*. As duas listaram a mesma falta: *«num mesmo traço não há gama do pó às
  gotas grandes»*. A dispersão contradiz-se: r1 achou-a larga demais, r2 estreita. Na espiral os vãos fecham.
- **Leitura:** o `jitter_scale` sorteia o tamanho UNIFORME, e os salpicos pedem uma lei de potência (muitas gotículas,
  poucas gotas). Variantes com partícula 0,25–0,7 e contagem 6–16 dão aglomerados. ⇒ **fila:** a distribuição de tamanho
  da partícula (log/potência), ou a mangueira de N imagens (L19) numa receita.
- **Portão** (saiu com o preset): `o_salpico_pinta_gotas_opacas_de_tamanhos_varios`. Mutação `jitter_scale` 0 → p90/p10
  4,1: **SANGRAVA**.

## J-4 Spray Drips (Wet): REPROVADO (motor grande)

- Os Rebelle `default_spray_4`/`_5` não escorrem: são variantes do pó, mais grossas e mais claras.
- 8 variantes sobre o Wet Drip (pó 0,8, pigmento 30–60, `gravity` 0,02–0,05, `extFingering` 0–1, `extRivulets` 15–60,
  água 0,3–3): a gravidade arrasta a folha molhada INTEIRA 20–45 px, o pó vira aguada, os dedos ficam ≤ ~15 px na borda.
  Nunca há o fio comprido que desce do traço.
- ⇒ **fila:** escorridos por limiar (a tinta que se acumula além de um limite corre em fio estreito, o resto fica).

## J-6 Grainy Shader (Digital): FUNDIDO no Airbrush

- **Fontes:** Handbook do Procreate (biblioteca Classic: «Airbrushing», «Spraypaints», «Materials … noise»; sem o Bonobo
  Chalk, que saiu no Procreate 5 — nota na Skillshare); Tuts+ (o Noise Brush sombreia em Multiply).
- **A lei que o separava existe e está medida:** o grão PRESO À TELA. No mesmo traço pintado duas vezes, ficam 0,58–1,00
  dos vãos; no pó do Airbrush ficam 0,04.
- **A revisão cega pela descrição não sustenta o pincel** (vizinho Airbrush 0,98, depois Tilted Shade 0,93 e Wax Scrape
  1,38, todos ≥ 0,591):
  - **r1 e r2** (Multiply, fluxo 0,3 e 0,6) escolheram o nosso, com confiança média a alta, e repetiram três faltas:
    sombra fraca, borda esfarelada e o grão que enche cerca de 30 % no cruzamento.
  - **r3** (fluxo 1) e **r4** (modo Height, que guarda os vãos: 0,33 → 0,31) escolheram o **Airbrush** como «o
    sombreador granulado». Leram o nosso como *«salpico denso» / «pontilhado grosso com buracos brancos duros»*.
- **Leitura:** forte, vira giz; fraco, é o Airbrush com faltas. ⇒ nada novo: o Airbrush já faz este papel.
- Portão `o_grao_do_sombreador_fica_no_papel` (mutação ViewPlane → 0,00: SANGRAVA): saiu com o preset.
- **Custo** da última receita: p95 0,18 · 0,36 · 2,21 ms, 24–26 MiB.

## Para a fila (coordenador)

1. **Gota do pó por preset** (Wet, `GOTA_DO_PO`), para o grão grosso do Rebelle no Airbrush.
2. **Partícula em px** (o tamanho da gota não escala com o pincel), para o Spray Can e o Stipple.
3. **Duas populações no spray** (véu e gotas), para o Spray Can.
4. **Emissão pelo tempo com o traço parado**, num `Space` (o Krita acumula 0,24 → 1,00), para o Spray Can.
5. **Lei de potência do tamanho da partícula**, para o Splatter.
6. **Escorridos por limiar no Wet**, para o Spray Drips.
7. **Sorteio de cor enviesado** (o Krita sobe valor e saturação), para o Stipple.
8. Os MyPaint `splatter-04`/`spray2`/`spray` não servem à escala da bateria (o raio multiplica o espalhamento).
