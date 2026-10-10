# HRC — descrição para implementar (W5c)

> Fonte: Freeman, Sannikov, Margel, *Holographic Radiance Cascades for 2D Global Illumination*,
> arXiv 2505.02041v1 (só existe a v1; lido o PDF, conferidas as equações, o Algoritmo 1, a Tabela 1 e a
> Fig. 8 nas páginas renderizadas). Nenhum código de terceiros lido (não há público). «INFERÊNCIA» =
> o artigo não diz; é a leitura mais plausível. § = secção, Eq. = equação, Fig. = figura do artigo.

## 0. Primitivas (§3)

- Fluência `F(p) = ∫₀^{2π} L(p, ω(θ)) dθ`, sem cosseno (Eq. 1).
- Composição de pontos colineares `p, q, r`: `Tr(p←r) = Tr(p←q)·Tr(q←r)` (Eq. 5);
  `Lr(p←r) = Lr(p←q) + Tr(p←q)·Lr(q←r)` (Eq. 6).
- `Merge(⟨rn,tn⟩, ⟨rf,tf⟩) = ⟨rn + tn·rf, tn·tf⟩` (Eq. 7); `Merge_r(⟨rn,tn⟩, rf) = rn + tn·rf` (Eq. 8)
  — o alfa pré-multiplicado de Porter-Duff com `t = 1 − α`.
- `Trace(p,q) = ⟨Lr(p←q), Tr(p←q)⟩` (Eq. 9). `A × ⟨r,t⟩ = ⟨A·r, t⟩`: radiância → fluência angular de
  um arco `A` (§3.1).
- Referência do artigo (§5): DDA de Amanatides–Woo, pixel = quadrado de extinção/emissão uniformes;
  radiância e transmitância em 3 × f16 (V_r = V_t = 6 B).
- O artigo NÃO salta espaço vazio (§5.0.4): só T0–T2 são traçados, raios ≤ 4√2 texels (§4.3.1).
- INFERÊNCIA (a nossa semântica «× (1−α) a cada ENTRADA»): cada `Trace(p,q)` cobre um intervalo
  SEMIABERTO — um texel cruzado pertence a um só dos dois segmentos que partilham um ponto, senão a
  entrada conta duas vezes.

## 1. Quadrantes (§4, Algoritmo 1)

- A fluência de cada sonda divide-se em 4 quadrantes somados no fim. Canónico: a luz que chega de
  **+x**, ângulos `[−π/4, +π/4]`. Os outros por ROTAÇÃO de 90° (o sentido não importa se consistente):
  ```
  for dir in 0..4:
      R0 := HRC(X, Y)
      for x, y:  L([x,y]) += R0([x+1, y], 0)
      rodar o mundo 90°; rodar L 90°; trocar X, Y
  ```
- INFERÊNCIA: em vez de rodar texturas, aplicar a mesma transformação nas leituras da cena e na
  escrita de L.

## 2. Sondas por cascata (§4, Fig. 4, §4.3)

- Cascata `n`: sondas em `p = (x·2ⁿ, y)` — espaçamento `2ⁿ` no eixo principal, **1** no transversal.
- Deslocamento `v_n(k) = (2ⁿ, 2k − 2ⁿ)`, `k = 0..2ⁿ` (de −45° a +45°). `angle([x,y]) = atan(y/x)`.
- **R_n** (radiância por cone): `i` semi-inteiro `½ … 2ⁿ − ½` ⇒ `2ⁿ` cones por sonda; o cone `i` tem
  arestas por `p + v_n(i±½)`; arco `A_n(i) = angle(v_n(i+½)) − angle(v_n(i−½))` (Eq. 13).
- **T_n** (intervalos, a aceleração, §4.1): `k` inteiro `0..2ⁿ` ⇒ `2ⁿ + 1` direções;
  `T_n(p,k) ≈ Trace(p, p + v_n(k))`.
- Contagens (X = 2^N): cada R_n tem X² valores; cada T_n `X²·(1 + 2^{−n}) ≤ 2X²`. N cascatas de R
  (R_0..R_{N−1}) e N+1 de T (T_0..T_N), `N = ⌈log₂ X⌉` (§4.2).
- Memória (§4.3.2): R em pingue-pongue (2 camadas, `2·X²·V_r`); TODAS as T guardadas
  (R_n lê T_n e T_{n+1}): `≈ N·X²·(V_r + V_t)`. O T domina. O artigo não MEDE memória.

## 3. Traçado direto e Merge Up (§4.1, Eq. 18–20, Fig. 7)

- T0, T1, T2 traçados direto (raios curtos; o erro da fusão de direções ímpares cresce com raios
  curtos): `19·X²` Traces nos 4 quadrantes (§4.3.1).
- Para n = 3..N, `T_{n+1}` de `T_n` (p = (x·2^{n+1}, y)):
  - índice par `2k`: `T_{n+1}(p, 2k) = Merge(T_n(p,k), T_n(p + v_n(k), k))` (Eq. 18) — exato;
  - índice ímpar (`k` semi-inteiro): `F± = Merge(T_n(p, k±½), T_n(p + v_n(k±½), k∓½))`,
    `T_{n+1}(p, 2k) = (F₋ + F₊)/2` nas duas componentes (Eq. 19–20).
- Índices inteiros (INFERÊNCIA de layout): `T_n[xi][y][k]`, `h = 2^{n−1}`:
  - k par, `c = k/2`: `Merge(T_{n−1}[2xi][y][c], T_{n−1}[2xi+1][y + 2c − h][c])`;
  - k ímpar, `a = (k−1)/2`, `b = (k+1)/2`:
    `F₊ = Merge(T_{n−1}[2xi][y][b], T_{n−1}[2xi+1][y + 2b − h][a])`,
    `F₋ = Merge(T_{n−1}[2xi][y][a], T_{n−1}[2xi+1][y + 2a − h][b])`, `T_n = (F₊ + F₋)/2`.

## 4. Merge Down (§4, Eq. 14–17, Fig. 6)

- `R_n(p,i) = F₊ + F₋` (metades do cone), cada uma pelo cone da cascata n+1 de índice `j± = 2i ± ½`.
- **x ímpar** (sem sonda n+1 em p), Eq. 14: `q± = p + v_n(i±½)`;
  `F± = Merge_r(A_{n+1}(j±) × T_n(p, i±½), R_{n+1}(q±, j±))`.
- **x par**, Eq. 15–17: `q± = p + 2·v_n(i±½)`; `F± = (F±⁰ + F±¹)/2`, `F±⁰ = R_{n+1}(p, j±)`,
  `F±¹ = Merge_r(A_{n+1}(j±) × T_{n+1}(p, 2i±1), R_{n+1}(q±, j±))` — usar só `F±⁰` puxa para o meio
  do cone e dá artefactos.
- Índices inteiros (INFERÊNCIA): `R_n[xi][y][m]`, `i = m + ½`; `j₊ = 2m+1`, `j₋ = 2m`; arestas
  `k₊ = m+1`, `k₋ = m`. xi ímpar: `F_s = A·T_n[xi][y][k_s].r + T_n[xi][y][k_s].t ·
  R_{n+1}[(xi+1)/2][y + 2k_s − 2ⁿ][m'_s]`. xi par, `K₊ = 2m+2`, `K₋ = 2m`: `F_s⁰ = R_{n+1}[xi/2][y][m'_s]`,
  `F_s¹ = A·T_{n+1}[xi/2][y][K_s].r + T_{n+1}[xi/2][y][K_s].t · R_{n+1}[xi/2+1][y + 2K_s − 2^{n+1}][m'_s]`.
  `A_{n+1}(m'+½) = atan((2m'+2−2^{n+1})/2^{n+1}) − atan((2m'−2^{n+1})/2^{n+1})`.
- **Fluência por texel:** `L[x,y] = Σ_quadrantes R₀([x+1, y], 0)`. O **+1** evita «cruzes» de brilho
  (raios diagonais de quadrantes vizinhos sobrepostos, Fig. 12).
- **Filtro (Eq. 21):** cruz de 1 px `(1/8)·[[0,1,0],[1,4,1],[0,1,0]]`, ignorando vizinhos de opacidade
  muito diferente — para n ≥ 1, `v_n(k)` desloca y por um PAR: sondas de y par e ímpar não conversam
  (tabuleiro de xadrez). Limiar e renormalização: o artigo não diz.
- Multi-rebote (§4.2): realimentar a fluência de saída, ao longo dos quadros (Fig. 14: 6 rebotes).

## 5. Contorno (§4.2, nota 1)

- O artigo assume todas as luzes DENTRO da grade: R_N ≡ 0. Luzes fora: R_N por outro método; céu sem
  oclusores fora: `R_N(i)` = o mapa na direção `v_N(i)`. Fora de `[0,Y)` em y: não diz (INFERÊNCIA:
  vácuo ⟨0,1⟩; R fora = 0).

## 6. Ordem dos passes

Por quadrante: (a) T0–T2 (um dispatch); (b) T_3..T_N em série (Merge Up); (c) R_{N−1}..R_0 em série,
pingue-pongue (Merge Down); (d) `L += R₀[x+1, y]`. Depois o filtro em cruz. Custo constante para um
tamanho, O(X² log X).

## 7. O que o artigo MEDE (RTX 3080 Laptop, Rust + LuisaCompute/CUDA)

| Grade | Merge Up | Merge Down | Total |
|---|---|---|---|
| 256² | 0,30 ms | 0,25 ms | 0,55 ms |
| 512² | 1,00 ms | 0,85 ms | 1,85 ms |
| 1024² | 4,10 ms | 3,57 ms | 7,67 ms |
| 2048² | 18,1 ms | 15,8 ms | 33,9 ms |

Fig. 8 (512², oclusor 14 px, luz 10 px): HRC 1,85 ms RMSE 0,0184 · RC otimizado 1,96 ms 0,0354 · PT+NEE
10 spp 1,55 ms 0,00711 (ruído) · PT 600 spp 109 ms 0,0260. Limitações: borra ~4 px; xadrez junto à
borda da sombra (Fig. 10); Moiré com luzes < 8× o espaçamento base (Fig. 11); em 3D memória ∝ N⁴.
