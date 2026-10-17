# 44 — As formas: a ponta reta do Flat, o corpo da guache e a pressão do pincel — o que se MEDIU (17/10)

> Prompt [44](44_PROMPT_as_formas_flat_e_guache.md). Barra: a do [18 §2](18_mapa_das_midias.md) (o vale entre o
> Rebelle e o controlo Wet Round à mesma largura, só métricas a > 3× o ruído do Rebelle) + a largura ± 2 px + a
> métrica nova **ponta**. Sonda `sonda_midias::as_midias_com_os_knobs_de_hoje` (receitas JSON, `PH2D_SONDA_SECA=1320`,
> `--profile smoke`); medidor `ferramentas/midias_lote.py`.

## §1 — O oráculo, das fotos que já havia (não foi preciso correr o Rebelle)

| foto (Rebelle, 40 px) | cob | tex | borda | cruz | **ponta** | larg | rampa larg / cob |
|---|---|---|---|---|---|---|---|
| `wcflat` (Watercolor/Flat) | 0,315 | 5,1 | 0,966 | 0,953 | **0,096** | 34 | 0,83 / 0,42 |
| `gouacheflat` (Gouache Flat) | 0,504 | 2,5 | 1,020 | 0,847 | **0,083** | 39 | 0,87 / 0,43 |
| `wc` (Flat 2) | 0,905 | 1,0 | 0,830 | 0,998 | 0,042 | 42 | — |
| `wcround` (Round) | 0,778 | 2,3 | 0,881 | 0,926 | 0,258 | 32 | 0,44 / 0,59 |
| `gouache` (Gouache Round) | 0,218 | 4,5 | 0,989 | 0,986 | 0,289 | 38 | 0,53 / 0,45 |
| `wcliner` · `wcwash` | | | | | 0,316 · 0,219 | | |

- **A forma do Flat é a da PONTA, não do traço**: o recto deitado acaba RETO (recuo a 0,7 da meia-largura 0,03–0,12,
  o redondo teórico 0,29), o vertical acaba arredondado (0,20–0,27) e é mais estreito (30 contra 34; a guache 31
  contra 39). A onda a 30° alarga 1,22× (o quadrado FIXO daria 1,58; o rodar/redondo 1,15): a ponta não gira com o
  traço (`brush_rotation_mode` PEN_ROTATION, o rato manda rotação 0). ⇒ uma pegada de lados RETOS no eixo do traço
  deitado, um pouco mais alta que larga.
- **A ponta não se mede pelo «além do último carimbo»**: o Rebelle não pousa a ponta onde o gesto acaba (o Round
  dá 9 px à esquerda e 1 px à direita — o atraso do traço). Mede-se a FORMA: **ponta** = quanto a ponta a ±0,7 da
  meia-largura fica atrás da do eixo, ÷ a meia-largura, média das 4 pontas isoladas (o traço rápido e o vermelho
  recto) — `midias_regua.py::ponta` e `regua_midia::ponta` (a mesma conta; gate `a_regua_da_ponta_e_a_do_python`
  sobre tampas sintéticas). Ruído do Rebelle (rápido × vermelho, mesma foto): ~0,045.
- **A guache Round do Rebelle continua a ser o nosso Round** (cob 0,22 · borda 0,99 contra 0,18–0,19 · 1,00; o 18
  §4.3): nenhuma métrica a distingue acima do ruído. **A guache que é outra mídia é a FLAT**: o dobro da tinta do
  Flat da aquarela (cob 0,50 × 0,32) e o cruzamento que escurece (0,85 × 0,95) — `use_glazing` desligado e
  `splat_opacity_multiplier` 7 × 2 no preset dela.
- **A pressão** (a rampa 0,15 → 1 → 0,15, a largura e a cobertura na pressão ~0,46 ÷ na 1): o Round do Rebelle
  ESTREITA (0,44 / 0,59) — **e o nosso Wet Round também** (0,45 / 0,64): bate. Os Flats quase não estreitam e
  ESMAECEM (0,83–0,87 / 0,42–0,43; `pressure_size_influence` 0,2–0,3, `pressure_opacity_influence` 1).

## §2 — O que a composição já dava (perguntado ao CÓDIGO e medido)

- **Nenhuma ponta procedural é reta**: a silhueta é `falloff(t)` com `t = |pegada|₂` (`dab::silhouette_at`,
  `FootprintDeform::falloff_t`); o Shape procedural MULTIPLICA o falloff (lei B) e o Flatten é uma ELIPSE
  (`dab_flatten`, recuo da elipse = o do disco no referencial dela). A ponta de IMAGEM é reta, mas o preset não leva
  imagem (os pixels moram no `PaintState`, vêm do asset do artista) — um pincel de fábrica não a pode trazer.
- Medido no lote 1 (`wcflat`, controlo Wet Round seco 34 px: ponta 0,432): elipse 0,12 @ 90° 0,432 · dureza 1
  0,333 — ninguém abaixo de 0,3.
- **A pressão → opacidade existe no motor** (`Dynamics::strength_pressure` → `d.coverage` → o `b` do dab molhado,
  `dab_route.rs`), mas NINGUÉM a liga: `PaintState::dynamics` é o `Dynamics::default()` fixo (o raio a 10 % na
  pressão 0, a opacidade parada), sem painel nem preset.

## §3 — As leis (3 lotes, ~120 cenas)

- **A QUINA da pegada** (`BrushSpec::dab_quina`, `0..1`): o falloff lê `t = (1 − q)·|f|₂ + q·|f|∞` no
  referencial do Flatten/Angle (`FootprintDeform::with_quina`) — `0` o `sqrt` de sempre, ao bit; `1` o quadrado.
  Sem transcendentes (HR-5). O contorno (`outline_at`: o anel e o gizmo) é exacto (`1/((1−q)+q·max|c|,|s|)`); a
  caixa é a menor das duas que a contêm (o quadrado, a norma dual 1; a elipse × o maior raio). Vai no preset por
  NOME (`CAMPOS_DO_PINCEL` 31; omissão = 0). **`SCHEMA` fica 1** (o leitor ignora chaves que não conhece).
- ⚠️ **A ponta reta só se LÊ com a borda dura**: lote 1, quina 1 com a dureza 0 do Wet Round → ponta 0,439 (o
  controlo 0,432). Com a queda macia as linhas de fora do eixo recebem menos tinta e acabam antes; e as cerdas
  recortam a ponta em dentes. Lote 2: dureza 0,6 → 0,286; **0,85–1 → 0,11–0,21**.
- **A pressão do PINCEL** (knobs `pressureSize`, `pressureOpacity`, Tuning › Paint): o `Dynamics` do traço
  (`PainterTool::dinamica_do_traco`, lido no `Stroke::new`) — `size_min = 1 − pressureSize` (0,9 → 0,1, ao bit),
  `pressureOpacity > 0` liga a opacidade (`strength_min = 1 − ela`). Omissão = a lei de sempre.
- **Varrido por consumidor** (a pegada é partilhada): caches de carimbo (chave), o device do Digital (inelegível
  com quina — o kernel lê `|M·p|₂`), blur/clone/inércia (`pegada_anisotropica`), o anel do cursor e o gizmo do
  Shape (`outline_at`), o alcance do motor molhado (a caixa do dab DESTE dab; a janela, a do canto). O controlo
  «Square tip» mostra-se só no Wet Paint (Shape, abaixo do gizmo).

## §4 — Os pincéis de fábrica (18 → 20)

Seco, à largura da foto, contra o Rebelle (R) e o controlo Wet Round (C):

| | cob | ponta | chato | rampa larg | rampa cob | larg |
|---|---|---|---|---|---|---|
| **Wet Flat** (Watercolor › Fluid): pigmento 50, dura 0,9, quina 1, achatado 0,12 @ 90°, pressão 0,3/1 | 0,322 | 0,164 | 0,89 | 0,77 | 0,54 | 35 |
| R `wcflat` | 0,315 | 0,096 | 0,86 | 0,83 | 0,42 | 34 |
| **Gouache Flat** (Watercolor › Gouache): pigmento 100, o resto igual, achatado 0,2 @ 90° | 0,525 | 0,171 | 0,80 | 0,82 | 0,62 | 40 |
| R `gouacheflat` | 0,504 | 0,083 | 0,78 | 0,87 | 0,43 | 39 |
| C (Wet Round) | 0,18–0,20 | 0,43–0,53 | 0,96–0,99 | 0,45 | 0,64 | |

Receitas DIFERENTES (o Flat pinta 0,32 onde a guache pinta 0,53; o cruzamento 0,95 × 0,85): não é um preset com
dois nomes. A **Gouache Round não entra** (§1). Portão `cada_preset_parece_a_sua_midia` (as métricas acima de 3× o
ruído: a cobertura, a ponta, o chato da guache, a largura da rampa e a largura).

## §5 — Portões, custo e mutação

(no fecho)

## §6 — Aberto

- **A cobertura da rampa**: com a opacidade inteira ligada a nossa vai a 0,54–0,62 contra 0,42–0,43 do Rebelle
  (o controlo 0,64) — a rampa linear do `Dynamics` (`min + (1 − min)·p`); o Rebelle esmaece mais depressa que a
  pressão. Uma curva de pressão por pincel é a próxima porta — não medida.
- **O Airbrush** (P43 §6): o Spray 2 também esmaece com a pressão (rampa 0,63 / 0,34); o nosso estreita. A lei
  agora existe (os knobs), mas mudar o Airbrush mexe num pincel de fábrica (a identidade das 18) — decisão no smoke.
- A 20 px o Airbrush lê textura 28,0 contra 22,5 (lote 1, `air13`): a janela 5×5 da régua num traço de 10–13 px —
  não investigado.

## ⛔ Recusas MEDIDAS

| proposta | medido | porquê não |
|---|---|---|
| a ponta reta pela composição (elipse, dureza 1, ponta de imagem) | ponta 0,432 · 0,333; a imagem não viaja no preset | a silhueta procedural é sempre `falloff(|f|₂)` |
| a quina sobre a queda macia do Wet Round | 0,27–0,44 (= o redondo) | a ponta só se lê com a dureza ≥ 0,85 |
| uma «Gouache Round» | cob 0,22 · borda 0,99 = o Round | um nome, não uma mídia (18 §4.3) |
| o Glaze na guache para o cruzamento | 0,49–0,50 (o Rebelle 0,85) | escurece de mais; o pigmento 100 já dá 0,85 |
| a ponta pelo «comprimento além do último carimbo» | o Round do Rebelle: 9 px × 1 px | o Rebelle não pousa a ponta onde o gesto acaba |
