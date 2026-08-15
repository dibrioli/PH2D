# ADR-0158 — O laço de vértices de um dab é um MAP disjunto: exceção `rayon` na `ph2d-sculpt3d`

- **Status:** aceito (ordem do Enio, 2026-08-13: *"ambos"*, sobre a avaliação que
  nomeou esta alavanca e a cerca que a bloqueava).
- **Escopo:** habilita `rayon` na `ph2d-sculpt3d`, **restrito ao laço de vértices
  do `SculptStroke::dab`**. Não abre `rayon` para o resto da crate; um sítio novo
  exige ADR novo, como o [ADR-0109](0109-watercolor-optical-composite-rayon-exception.md)
  determina e como o [ADR-0145](0145-wet-paint-solver-row-parallel-passes-rayon-exception.md),
  o [ADR-0147](0147-wet-paint-order-invariant-solver.md) e o
  [ADR-0156](0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md)
  já fizeram cada um por si.
- **Número:** ⚠️ **PROVISÓRIO.** Ele foi contado contra o `main` do dia (o último
  é o 0157) numa linha paralela, e um número escolhido assim **se re-conta na
  integração** — foi o que aconteceu oito vezes neste repo.

## O problema, com o número

O report do Enio (2026-08-13) é *"o resultado ficou muito bom do modo L mas com
um pouco de queda de FPS"*. A medição
(`ph2d-sculpt3d/tests/measure_field_cost.rs`, pela porta do produto, na malha
que o módulo abre — `sculpt_sphere`, 196 608 triângulos) decompõe um dab do
`l-mode` a 30 % de raio, **6,406 ms**, assim:

| parte | ms | % | natureza |
|---|---|---|---|
| normais + curvatura | 3,166 | 49 % | **já `rayon`** (na `ph2d-mesh`) |
| **o laço de vértices** | **2,649** | **41 %** | **serial, um núcleo de 32** |
| a consulta do octree | 0,591 | 9 % | — |

⛔ **Três alavancas foram medidas e MORTAS antes desta** (§7.14 do
[plano 21](../../3D/21_plano_modos_e_ferramentas.md)):

1. a **família de escalas** rende zero — `Tri` custa **1,00×** o `Bi` (o 3º tap
   é grátis: o kernel é limitado por latência, não por vazão), e só o `Mono` é
   barato, mas ele é o *look*;
2. **paralelizar a descoberta do `refresh_region`**: ela é **3 / 6 / 10 %** do
   passe, ou seja ≤ 5 % do dab. ⚠️ Isto **refuta o número que o `CLAUDE.md` §5
   carrega da W1** (*"88 % do refresh"*), que era do `apply_dab`, numa
   UV-sphere de 5 M, antes de o refresh paralelizar as duas metades;
3. **pular a curvatura** mudaria a **aparência** — a `ph2d-sculpt3d` não a lê em
   lugar nenhum, mas quem a lê é o **renderer**, todo quadro
   (`pipeline_upload`: cavidade e sombreado).

Sobra o laço de vértices.

## A decisão

O laço interno do `SculptStroke::dab` percorre índices independentes e é
paralelizado por `rayon`, com a **acumulação e o espalhamento SERIAIS**.

## Por que ele é elegível — as três condições do ADR-0109, verificadas

1. **O laço NÃO escreve posições.** Ele escreve `self.target[s]` e `self.accum[s]`
   — buffers por-slot — e o `apply_positions(mesh)` roda **depois**, fora dele.
   A forma já é *computa-contíguo-depois-espalha*, a mesma que o
   `Mesh::refresh_region` usa e pelo mesmo motivo (o comentário dele diz:
   *"calcula para um vetor CONTÍGUO e espalha — é o que deixa a leitura pura e
   permite o `rayon`"*).
2. **As leituras são puras dentro de um passe**, inclusive as `from_live`: elas
   leem `mesh.positions()`, que ninguém escreve enquanto o laço corre.
3. **Os slots são disjuntos** (um por vértice) e o `compute_target` é `&self` e
   **não lê o `accum`** — conferido, e era o único acoplamento de ordem que
   poderia existir.

## O que a exceção NÃO cobre, e é o que a torna segura

⚠️ **Nada em ponto flutuante é ACUMULADO em paralelo.** O `fit_plane` (mínimos
quadrados sobre a pegada) e o banco do `Grip` somam `f32`, e ordem de soma entre
threads move bits — os dois ficam **fora** do laço paralelo, onde já estavam. A
condição 3 do ADR-0145 é honrada por construção, não por promessa.

⚠️ **`self.moved` é reconstruído na ORDEM dos índices**, não na ordem de
chegada das threads. Ele alimenta o `refresh_region`, cujo resultado é
independente de ordem — mas a lista é publicada e um consumidor futuro pode
não ser, então ela não vira um conjunto não-ordenado por acidente de
implementação.

## O preço e o teto

Teto medido: dab **6,4 → ~4,0 ms (1,6×)** no raio de 30 %. ⚠️ **Menor que o
coalescing** do `Grip::Hold` (15×, byte-idêntico, sem ADR — commit
`63c856aa4`), que atacou o mesmo report por outro eixo: *quantos dabs*, em vez
de *quanto custa um*. Os dois compõem.

## A prova exigida

**Byte-identidade contra a rota serial CONGELADA** sob `cfg(test)` — o
precedente do `serial_side` do Painter e do `sim_step` do wet paint: comparar
contra uma reimplementação seria razão entre dois doentes. Mais o gate de
escalonamento (a mesma resposta com o pool em 1 e em N threads) e a suíte de
paridade ULP com o SculptGL, que é o oráculo que já existe.

## Alternativas recusadas

- **GPU**: o `sculpt_kernel_device` do [ADR-0150](0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)
  segue não construído de propósito, e o K1 (8 ms) **não é estourado** no ponto
  de operação medido — a porta nasce com o caminho, se ele nascer.
- **Não fazer nada**: legítimo enquanto o coalescing bastar. Este ADR fica
  aceito e a construção é incremental; se o smoke do `63c856aa4` já devolver o
  FPS, o 1,6× é ganho sem urgência.
