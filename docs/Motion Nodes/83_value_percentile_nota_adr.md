# Doc 83 — `value.percentile`: o filtro MORFOLÓGICO / de rank do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..82.

## O que é

O filtro **morfológico / de rank** — troca cada elemento pela `p`-ésima estatística
de ordem da sua janela de índice. Onde o `value.median` (doc 82) pega o MEIO da
janela (`p = 0.5`), este pega qualquer rank — e as **pontas** são as operações
genuinamente novas, não uma mediana com botão:

- **`p = 0` → o filtro MÍNIMO → EROSÃO cinza:** cada elemento vira o MENOR da
  vizinhança, então regiões altas ENCOLHEM — o *Minimum* do Photoshop.
- **`p = 1` → o filtro MÁXIMO → DILATAÇÃO cinza:** o MAIOR, regiões altas CRESCEM —
  o *Maximum* do Photoshop.
- **`p = 0.5` → a mediana** (o de-spike; o `value.median` é o atalho dedicado sem
  botão para o caso mais comum).

Erosão e dilatação são os primitivos morfológicos (abertura/fechamento são as
composições), e um min/max de JANELA **não é** exprimível dos nós existentes (o
`value.reduce` Min/Max é o agregado GLOBAL, não uma janela por-elemento).

- **input** `in` : o campo de valor (`v`)
- **output** `out` : VALUE, a `round(p·(w−1))`-ésima estatística de ordem da janela
- **params** `radius` (meia-janela; 0 = passthrough, cap 16) · `percentile` (`p`, 0..1)
- **Effect** `Pure`; mapa unário, comprimento preservado

## Decisões

1. **As PONTAS são o valor** (erosão/dilatação), não o meio. Um Median dedicado já
   cobre `p=0.5`; este destrava min/max/rank. Photoshop shipa Median + Minimum +
   Maximum como três filtros; este os unifica num `p` contínuo.

2. **Lê os VIZINHOS off `in_v`, como `value.median`/`value.smooth`** — binding
   `ReadWrite`, bordas estendem. O rank é `round(p·(w−1))` (0 = min, `w−1` = max,
   `(w−1)/2` = mediana), clampado. `p·(w−1)` é o MESMO f32 nos dois devices e o
   `round` (half-away) é o MESMO, então o rank-alvo é bit-idêntico.

3. **Saída é SEMPRE uma amostra EXISTENTE** ⇒ **paridade BIT-exata** (a CPU ordena,
   o device conta ranks; ambos pegam a mesma estatística de ordem). Concorda com o
   `value.median` em `p=0.5` pela **MATEMÁTICA** (uma estatística de ordem é única),
   **não** por código compartilhado — implementações independentes, zero drift.
   ⚠️ Como no median: só sobre entrada BIT-idêntica (um input ε-diferente poderia
   escolher outra amostra, divergindo pelo VÃO entre estatísticas de ordem >> ε), e
   o gate de paridade alimenta `value.pattern` (exato), não `value.noise`.

4. **`radius` capado em 16** (janela 33, o array de registradores do device; `O(w²)`).
   Default `radius 0` (passthrough neutro), `percentile 0.5` (mediana).

## Rejeitados

- **Erode e Dilate como nós separados** — Photoshop os separa, mas um `p` contínuo é
  um nó só que também dá quartis (p10/p90 = de-noise assimétrico) e a mediana; dois
  nós seriam duas cópias do mesmo motor de seleção.
- **Compartilhar o motor de contagem-de-rank com o `value.median`** — desnecessário:
  as duas concordam por DEFINIÇÃO (estatística de ordem única), não por código, então
  implementações independentes não driftam. Um crate compartilhado seria acoplamento
  sem ganho.
- **Abertura/fechamento (erode∘dilate) como modos** — são a COMPOSIÇÃO de dois
  percentiles (`p=0` depois `p=1`): dois nós na pilha, não um param.

## Preço / cobertura

Kernel WGSL = colhe a janela num `array<f32, 33>` (cap `radius ≤ 16`), computa o rank
`round(p·(w−1))`, seleção por contagem de rank (empate por posição). Binding
`ReadWrite`, `count_law: None` (unário), `vpc_round` = `f32::round` (raio E rank). Sem
`applicable` (sem fallback de CPU). Paridade **bit-exata** (sobre entrada bit-idêntica).

**Gates:** raio 0 passthrough em qualquer `p` · `p=0` é o MÍNIMO/erosão
(`[5,5,5,1,5,5]` → o `1` erode nos vizinhos) · `p=1` é o MÁXIMO/dilatação
(`[0,0,0,9,0,0]` → o `9` dilata) · `p=0.5` é a mediana (deleta o spike) · os ranks
são ORDENADOS (`min ≤ median ≤ max` por elemento) e a constante é ponto-fixo · saída
é amostra EXISTENTE, finita, comprimento preservado, raio capado · cook end-to-end
(dilatação `[0,0,5,0,0]` → `[0,5,5,5,0]`) · registro — 8 unit tests verdes. Paridade
de dispositivo (`#[ignore]`, RTX, **entrada `value.pattern`** bit-idêntica,
**`p=0.25`** — um quartil, rank ≠ mediana, pina a aritmética do rank; `radius 2`;
`max|d| == 0`).

## Demo — `PH2D_VALUE_PERCENTILE_SMOKE=1`

Quatro fileiras de 24, a MESMA `value.pattern` de 8 valores repetida 3× — platô médio
(0.5) com um SPIKE (0.9) e um POÇO (0.1): de cima **RAW**, **ERODE** (`p=0` — o spike
some, o poço engorda), **MEDIAN** (`p=0.5` — spike E poço somem; marcada
`>> EVALUATE <<`), **DILATE** (`p=1` — o spike engorda, o poço some). Selecione a do
meio → deslize o **Percentile** de `0` a `1` e veja a MESMA fileira MORFAR erosão →
mediana → dilatação ao vivo. Estático, sem play.
