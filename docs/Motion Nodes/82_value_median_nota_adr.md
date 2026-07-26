# Doc 82 — `value.median`: o filtro de MEDIANA (não-linear) do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..81.

## O que é

O filtro **não-linear** — troca cada elemento pela MEDIANA da sua janela de índice,
uma estatística de ordem sobre a ordem das instâncias. É o irmão do `value.smooth`
(doc 77) que o smooth **não pode ser**: o box blur é LINEAR (faz a média, então um
outlier vaza para os vizinhos e toda borda amacia); a mediana é NÃO-LINEAR (escolhe
o valor do meio, então um spike é DELETADO e uma borda é MANTIDA). É o removedor de
ruído sal-e-pimenta / impulso — a Median do processamento de imagem, o de-spike de
um filtro de sinal.

- **input** `in` : o campo de valor (`v`)
- **output** `out` : VALUE, a `r`-ésima estatística de ordem da janela `2r+1`
- **params** `radius` (meia-janela; 0 = passthrough, 1 = mediana-de-3)
- **Effect** `Pure`; mapa unário, comprimento preservado

## Decisões

1. **A única coisa que o smooth não faz, e por isso ambos existem:** um campo com um
   SPIKE isolado (uma amostra ruim, um glitch de `value.noise`). O `value.smooth`
   espalha o spike numa corcova e arredonda toda borda; o `value.median` deleta o
   spike e deixa as bordas afiadas. Linear vs estatística de ordem — a razão de um
   toolset trazer um Blur E uma Median.

2. **Lê os VIZINHOS off `in_v`, como o `value.smooth`** — binding `ReadWrite`
   (buffers separados; a escrita nunca corrompe uma leitura de janela). As bordas
   **estendem** (índice clampado repete a fronteira), e devolve a `r`-ésima
   estatística de ordem da janela `2r+1`.

3. **Saída é SEMPRE uma amostra EXISTENTE** (nunca uma média) ⇒ **paridade BIT-exata**:
   a CPU ordena a janela, o device conta ranks, e ambos escolhem o MESMO valor. Sem
   aritmética para divergir. ⚠️ **Mas só sobre uma entrada BIT-idêntica** — se dois
   valores da janela são ε-próximos e cercam a fronteira do rank, os dois devices
   escolhem amostras DIFERENTES, divergindo pelo VÃO entre estatísticas de ordem
   (>> ε), não por ε. Por isso o gate de paridade alimenta `value.pattern`
   (passthrough de param, exato), nunca `value.noise` (que tem FMA).

4. **`radius` capado em 16** (janela 33): a seleção do device roda num array de
   registradores fixo, e o custo é `O(w²)` por elemento — uma mediana é um
   de-spiker de janela PEQUENA por natureza, e um filtro de rank de janela larga é
   outra ferramenta. Default `0` (passthrough neutro, a convenção do smooth).

## Rejeitados

- **Um sort completo no kernel** — a mediana precisa só do rank `r`, então a seleção
  é uma CONTAGEM de rank (quantos elementos da janela estão abaixo, empate por
  POSIÇÃO), sem array mutável de ordenação. O CPU ordena (código limpo, mesmo
  resultado — a `r`-ésima estatística de ordem é única).
- **`value.median` NÃO é o `value.smooth` com outro kernel** — smooth é a MÉDIA
  (linear, borra bordas), median é a MEDIANA (não-linear, preserva bordas); a
  diferença é a razão de existirem os dois.
- **Um filtro de rank geral (percentil `p`)** — a mediana é `p = 0.5`; um percentil
  arbitrário é uma generalização (o mesmo motor de seleção, rank = `p·w`), fica para
  se houver pedido.

## Preço / cobertura

Kernel WGSL = colhe a janela num array de registradores (`array<f32, 33>`, capado em
`radius ≤ 16`), depois seleção por contagem de rank (empate por posição ⇒ exatamente
um candidato por rank). Binding `ReadWrite` na coluna `v`, `count_law: None`
(unário). `vmd_round` casa com `f32::round` (o raio dimensiona a janela, um inteiro).
Sem `applicable` (sem fallback de CPU). Paridade de dispositivo **bit-exata** (sobre
entrada bit-idêntica).

**Gates:** raio 0 é passthrough · spike isolado é DELETADO, não espalhado
(`[0,0,9,0,0]` → tudo zero; um box blur daria `[0,3,3,3,0]`) · borda é MANTIDA afiada
(`[0,0,0,1,1,1]` intacto; o blur rampearia) · campo constante intacto · saída é
sempre uma amostra EXISTENTE, finita, comprimento preservado, e o raio é capado em
`MAX_RADIUS` · cook end-to-end (spike `[2,2,8,2,2]` → `[2,2,2,2,2]`) · registro —
7 unit tests verdes. Paridade de dispositivo (`#[ignore]`, RTX, **entrada
`value.pattern`** — bit-idêntica, exercita o empate; `radius 2`; `max|d| == 0`,
BIT-exato).

## Demo — `PH2D_VALUE_MEDIAN_SMOKE=1`

Três fileiras de 24, a MESMA `value.pattern` de 8 valores repetida 3× — platô baixo
com um SPIKE (sal, 0.9), a borda 0.2→0.7, e um POÇO (pimenta, 0.1): de cima **RAW**
(os spikes e a borda), do meio **SMOOTH** (os spikes viram corcovas, a borda vira
RAMPA — o passa-baixa borra tudo), de baixo **MEDIAN** (os spikes SOMEM, a borda fica
AFIADA — estatística de ordem, preservando as bordas; marcada `>> EVALUATE <<`). O
par smooth/median fica óbvio sobre o mesmo campo. Selecione a de baixo → suba o
**Radius** (spikes largos somem, a borda resiste). Estático, sem play.
