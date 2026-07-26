# Doc 81 — `value.slope`: a DERIVADA (Slope CHOP) do domínio de valor (nota-ADR)

> Motion Nodes M2, domínio de VALOR (doc 12). Segue 68..80.

## O que é

A **derivada** do campo — a taxa de mudança `d(value)/d(index)` ao longo da ordem
das instâncias. É o irmão EXATO do `value.smooth` (doc 77): onde o smooth faz a
MÉDIA de cada elemento com os vizinhos (passa-baixa, amacia o campo), este os
SUBTRAI (passa-alta, acha onde o campo MUDA) — zero nos platôs, um pico em cada
borda. O Slope CHOP do TouchDesigner, o `np.gradient` do NumPy, o gradiente de
imagem como operação de primeira classe do valor.

- **input** `in` : o campo de valor (`v`)
- **output** `out` : VALUE, a derivada discreta × `scale`
- **params** `scale` (amplifica; negativo inverte o sinal)
- **Effect** `Pure`; mapa unário, comprimento preservado

## Decisões

1. **Central no interior, unilateral nas pontas** (a regra do `np.gradient`):
   `out[i] = (v[i+1] − v[i−1]) / span · scale`, onde `span` é a distância de índice
   REAL entre os dois vizinhos lidos — `2` no interior (diferença centrada), `1` na
   borda (diferença adiante/atrás contra o índice clampado). Dividir pelo span
   verdadeiro mantém a inclinação da borda HONESTA (um `/2` fixo em toda parte
   halvaria as pontas unilaterais).

2. **Lê os VIZINHOS off `in_v`, como o `value.smooth`** — binding `ReadWrite` (lê a
   entrada, escreve um `out_v` fresco; buffers separados, então uma escrita nunca
   corrompe uma leitura de vizinho). `params.count` dá o `N` para o clamp de borda.
   A ordem é significativa quando as instâncias estão em sequência (uma fileira, uma
   grade), o caso comum — e a derivada é *ao longo dessa ordem*.

3. **Só o `scale`, e ele é o certo.** Uma inclinação é pequena (a mudança de uma
   instância para a próxima) e você quer dirigir algo visível com ela; negativo
   inverte o sinal. A **magnitude** da inclinação (força da borda, sem direção) é um
   `value.unary` Abs adiante, então este nó fica a inclinação COM SINAL,
   single-purpose.

4. **Campo de `≤ 1` elemento não tem inclinação → `0`** (o guard do span — um span
   zero dividiria). Finito sempre. `Pure`, transcendental-free (subtração e
   divisão), **device-resident** (sem fallback; canal de kernel existente, sem
   redução, sem scan) — a divisão por `1.0`/`2.0` é exata, a única divergência de
   dispositivo é um FMA em `· scale`, ε abaixo do orçamento.

## Rejeitados

- **Um toggle `abs` (magnitude da borda)** — é `value.unary(Abs)` adiante; embutir
  duplicaria um nó que já existe. O slope fica COM SINAL.
- **`radius` (uma derivada de janela larga)** — a derivada de escala é
  `smooth(radius) → slope` (borra primeiro, depois diferencia): composição, não um
  param. O slope é a diferença de vizinho imediato.
- **`value.slope` NÃO é o `value.smooth` invertido** — smooth é a MÉDIA (passa-baixa),
  slope é a DIFERENÇA (passa-alta); irmãos, não inversos (a integral de um slope é o
  `value.accumulate`, que é foundational e fica deferido à linha GPU).

## Preço / cobertura

Kernel WGSL = clamp dos dois índices vizinhos, `(read_v(hi) − read_v(lo)) / span ·
scale`, com o guard de `count ≤ 1 → 0`. Binding `ReadWrite` na coluna `v`,
`count_law: None` (unário). Sem `wgsl_lib` (nenhum helper), sem `applicable` (sem
fallback de CPU). Paridade de dispositivo bit-comparável.

**Gates:** rampa tem inclinação CONSTANTE (a derivada de uma reta é a taxa;
falsificável — um `/2` cru daria `0.5` nas pontas) · campo constante tem inclinação
ZERO · **borda é um PICO** (`[0,0,1,1]` → `[0, 0.5, 0.5, 0]`: zero nos platôs,
nonzero no salto — detecção de arestas) · `scale` amplifica e inverte · campo
degenerado (`≤1`) → `0`, finito · cook end-to-end · registro — 7 unit tests verdes.
Paridade de dispositivo (`#[ignore]`, RTX, **entrada `value.noise`** — uma rampa
teria inclinação CHATA que um produtor-constante passaria; o noise faz a inclinação
VARIAR por instância; `scale 2.3`; `max|d| < 1e-4`).

## Demo — `PH2D_VALUE_SLOPE_SMOKE=1`

Três fileiras de 24, a MESMA `value.pattern` escalonada em cada uma (quatro valores
distintos, degraus com saltos claros): de cima **RAW** (o campo cru — degraus), do
meio **SMOOTH** (os degraus amaciados, o passa-baixa), de baixo **SLOPE** (zero nos
platôs, um PICO em cada salto — as bordas; marcada `>> EVALUATE <<`). O par
smooth/slope fica óbvio sobre o mesmo campo. Selecione a de baixo → suba o **Scale**
(os picos crescem, o platô fica em zero) ou inverta-o (negativo).
