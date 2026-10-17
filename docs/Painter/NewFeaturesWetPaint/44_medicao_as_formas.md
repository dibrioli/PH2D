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

## §3 — As leis

(em curso — lote 2)
