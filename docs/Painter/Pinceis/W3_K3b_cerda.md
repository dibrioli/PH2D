# W3 K3b — a geometria da cerda e da marca (27/10)

Agente K3b da MISSÃO W3 do Óleo Real. Família «geometria da cerda e da marca» (CLAUDE.md §0.10), cinco casos da W2:
Fan (fios através do traço), Metallic Flat (pontas que esfarelam), Metallic Round (relevo grumoso), Dabber 3 (lascas
angulares), Metallic Splats (duas populações). Folha: [`W3_K3b_cerda.jpg`](W3_K3b_cerda.jpg) (antes | depois |
Rebelle, faixa de 200 px da cena `tamanhos`; o Flat na ponta direita). Régua:
[`W2_Dabber_candidatos/k3b/cerda_regua.py`](W2_Dabber_candidatos/k3b/cerda_regua.py) (`fan|flat|round|d3|splats`).

## Duas leis novas no motor (nenhum knob novo)

1. **As cerdas do Shape crescem com o pincel** (`wetpaint/lote.rs::escala_as_cerdas`, chamado em `dab_route.rs` ao lado
   do `escala_o_grao` da K2). Causa: o tipo `Bristles` (o tapete — todo pincel de cerdas do Wet o é, pelo
   `cerdas_no_shape`) é uma lei em PÍXEIS (1 texel = 1 px a todo tamanho), então os fios do leque a 200 px eram os de
   40. Lei: o mesmo factor `(D ÷ textureSizeRef)^textureSizeExp` da K2 divide o `Size` do Shape Bristles, em qualquer
   mapeamento. `γ = 0` = 1, ao bit.
2. **A ponta que ESFARELA** (`ponta_das_cerdas.rs::recorta_pelo_grao`, ligada em `dab_route.rs`). Causa: o
   `bristleTipRagged` desloca a pegada por FIO (uma faixa através do traço), então a ponta só sabe abrir em fios. Lei:
   com um grão armado em `Tiled` (preso à tela, sem Flow), o desvio de cada célula é `ρ·(1 − 2u)` com `u` o grão na
   célula — a faixa do `comprimento_do_fio`, mas 2-D: a ponta acaba pelas curvas de nível do grão. Sem grão `Tiled`, a
   desfiada de sempre (o `recorta` foi partido em `desloca`, mesma conta, ao bit). Censo da fábrica: os três com
   `bristleTipRagged > 0` (Ink Brush, Oil Bristle, Filbert) não têm grão ⇒ neutro.

## Números (régua K3b, ÷ D; foto A da cena `tamanhos`)

| item | métrica | Rebelle 40 · 200 | antes | depois | cura |
|---|---|---|---|---|---|
| 1 Fan | anisotropia (transições em x ÷ em y; fios verticais ⇒ > 1) | 1,13 · 1,84 | 0,74 · 0,94 | 0,98 · 1,21 | composição: Shape `Bristles` + **Flow** + Size `[0,6; 0,1]` (os fios estendidos através, em fase com o arco). Com `γ` 0,6–1 os fios crescem mas ENGROSSAM também (1,01 · 1,01): parcial |
| 2 Flat | anisotropia das pontas (fios ⇒ ≪ 1) | 0,89 · 1,00 | 0,61 · 0,50 | **1,14 · 0,96** | lei 2 + grão `Noise Tiled` 60, Contrast 1, Depth 0,1 |
| 3 Round (com Óleo Real) | p5 da luz ÷ mediana (vãos fundos ⇒ baixo) | 0,69 · 0,55 | 0,58 · 0,73 | 0,62 · 0,67 | lei 1, `γ` 0,5 sem grão: os grumos crescem a 200 px; ainda ondulado, não os vãos finos do Rebelle |
| 4 Dabber 3 | buracos: fracção · tamanho ÷ D · alongamento (200) | 0,25 · 0,104 · 1,91 | 0,38 · 0,083 · 2,02 | sem cura | ver «o que falhou» |
| 5 Splats | massa nas manchas grandes · tom (200) | 0,147 · 0,052 | 0,00 · 0,052 | 0,76 · 0,175 | composição: `spray_count` 4, `particle` 0,1, `across` 1, `jitter` 0,5 + o pó — DUAS populações (a marca do spray pousa desde a K2); valores para a P (excesso 5×) |

Olhado (folha): o Flat passa de riscas a grumos (o Rebelle tem ainda uma rede de fendas que esbate — fica para o
material); o Fan mostra os fios verticais mas em blocos e mais baixo (`dab_flatten` 0,78 achata o leque: P); o Round
cresce o grumo; o Splats tem as duas populações mas as grandes são DISCOS PÁLIDOS (no Rebelle são pontos pequenos e
escuros).

## Gates

- Neutro: `presets_de_fabrica::a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` verde; `textura_do_tamanho` (K2) verde.
- `tests/it/cerda_e_marca.rs`: `as_cerdas_crescem_com_o_pincel` (γ 0 escrito = sem a chave ao byte; 96÷24 com γ 1 >
  1,5 × o de γ 0 = 1,39) · `a_ponta_esfarela_pelo_grao` (Filbert de fábrica, recto de 120 px: sem grão 0,72, com
  grão `Tiled` 1,02; limiares 0,8 / 0,9).
- Mutações, todas VERMELHAS sobre o verde (ficheiro restaurado e conferido por sha256): cerdas não escalam →
  `as_cerdas_crescem_com_o_pincel` · γ 0 não neutro nas cerdas (×1,25) → a miniatura da fábrica · a ponta não
  esfarela → `a_ponta_esfarela_pelo_grao` · a desfiada refeita muda (`− 0,9`) → a miniatura da fábrica.

## O que falhou / fila

- **Dabber 3 (lascas):** NENHUMA composição dá as lascas. Medido em duas rondas: Voronoi (Random e Tiled, Edges 1,
  Chebyshev/Manhattan, Subtract) = GRELHA de buracos quadrados (fracção 0,16–0,20, tamanho 0,04 ÷ D); Musgrave Tiled
  `γ` 1,2 = buracos pequenos e raros (0,08 · 0,038); Marble = riscas finas; o tapete esticado `[1; 0,33]` com Jitter
  Rotate = cheio (os dabs sobrepostos tapam). As lascas do Rebelle são CARIMBOS discretos de flocos angulosos. Precisa
  de um tipo de textura novo **«Lascas»** em `ph2d-painter-brush` (`TextureKind`, `patterns`, `specs`, rótulo i18n):
  polígonos aleatórios alongados (2–3:1) por célula, no referencial do dab (View/Random: escala com o raio), Subtract.
  Fora dos ficheiros deste agente — a janela agenda.
- **Splats escurece a 200:** a marca do spray espalha o MESMO pigmento numa área ∝ raio² ⇒ empalidece a 200 (tom das
  grandes 0,52 → 0,43). Lei candidata (motor, `rota_da_lane`/`dab_route`): o pigmento da marca do spray ∝ a área dela.
  Não feita (sem ronda que sobrasse).
- **Fan:** os fios do Rebelle crescem em COMPRIMENTO e quase não em largura; o `γ` escala os dois eixos do Size.
  Pede um expoente por eixo — knob `textureSizeExpAcross` (0–1,5, neutro = o `textureSizeExp`) em `tuning/defs.rs`
  (fora deste agente).
- **Round:** os vãos finos e escuros pedem o relevo do material (K4), não a geometria.
- Os ficheiros dos candidatos NÃO foram regravados (valores = etapa P); as variantes estão acima.

## ⛔ Recusas MEDIDAS

| tentativa | medido | porquê não |
|---|---|---|
| Voronoi (Edges/Metric, Random e Tiled) como lasca | grelha de buracos quadrados | padrão de rede (a recusa da W2, confirmada com os 4 params) |
| grão `Noise`/`Stucci`/`Voronoi Tiled` no Round com Óleo Real | ONDAS de 20–40 px (desvio 0,33) | o grão vira ondulação no óleo (a recusa da W2, confirmada com `γ`) |
| `Noise` Contrast 0,5 na ponta esfarelada | pontas 1,11 · 0,55 | o ruído de valor concentra-se perto de 0,5: desvio pequeno; Contrast 1 abre a faixa |
| spray sem `across` (Splats) | as marcas caem todas no eixo (uma linha) | o spray precisa do espalhamento |
