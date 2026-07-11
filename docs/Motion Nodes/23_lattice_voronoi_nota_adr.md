# 23 — Nota-ADR: Lattice + Voronoi (M3 — as distribuições que faltavam)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Fecha as duas
distribuições restantes do M3 (doc 22 §5): a **rede cristalina** e a **relaxação organizada**, completando o
vocabulário de distribuições ao lado de `grid` (quadrada), `fibonacci` (phyllotaxis) e `scatter` (blue-noise).

---

## 1. O problema (doc 01 §3 / doc 22 §5 — distribuições)

Tínhamos a grade quadrada (`grid`), a espiral (`fibonacci`) e o blue-noise (`scatter`). Faltavam as duas
distribuições clássicas que todo pacote tem e que os anteriores não cobrem: o **empacotamento hexagonal**
(a rede triangular — colmeia, empacotamento denso de círculos) e a **tesselação de Voronoi centroidal (CVT)**
por **relaxação de Lloyd** (o método iterativo que organiza um ruído em espaçamento uniforme — stippling,
amostragem, dart-board).

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.lattice` (a rede hexagonal):** o padrão-ouro do empacotamento 2D mais denso é a **rede triangular**.
Veredito:

- Linhas pares num passo quadrado `spacing`; linhas ímpares deslocadas meia-célula, com passo de linha
  `spacing·√3/2` → **toda distância ao vizinho mais próximo é exatamente `spacing`** (triângulos equiláteros /
  hexágonos regulares). É o empacotamento de círculos mais denso do plano. HR-5: `√3/2` é constante, sem
  chamada. Testado por **empacotamento hexagonal** (NN = spacing p/ todos; um grid quadrado daria `spacing·√2`
  na diagonal) + **ímpares meia-deslocadas**.
- Um **`jitter` value input** desloca cada ponto por um offset hasheado (splitmix) escalado por `jitter` → um
  `value.lfo` faz a colmeia **cintilar e reformar**. Stateless, `Effect::Pure`.

**`motion.voronoi` (a relaxação de Lloyd):** o padrão-ouro da distribuição uniforme *iterativa* é o
**algoritmo de Lloyd (iteração de Voronoi)** rumo a uma CVT. **Confirmei via pesquisa** (Wikipedia/CVT +
stippling ponderado 2002 + jump-flood na GPU). Veredito:

- Lloyd: repetidamente **move cada ponto pro centroide da sua célula de Voronoi** → converge pra CVT (mais
  regular que blue-noise, quase-hexagonal). O método exato usa Voronoi de Fortune + centroide do polígono; a
  **forma prática padrão** (stippling ponderado 2002; primo-CPU do jump-flood na GPU) **discretiza o domínio
  numa grade `res²`**, atribui cada amostra ao ponto mais próximo, e move cada ponto pra média das suas
  amostras — repetido `iterations` vezes. **Deterministico e HR-5** (distâncias ao quadrado, sem `sqrt`).
- Um **`relax` value input** toca a relaxação (lerp raw-seed → CVT relaxada) → um `value.lfo` faz a nuvem
  **se organizar numa colmeia e dissolver** (mostra o próprio algoritmo). Seed hasheado, `Effect::Pure`.
  Testado por **Lloyd empareja a nuvem** (min-gap da CVT ≫ ruído branco), **relax lerpa** (0=seed, 0.5=meio,
  1=CVT), pontos dentro do domínio, replay + seed re-rola.
- **Custo (Enio reportou queda de fps — CORRIGIDO 2×):** um cook é `O(iterations · res² · count)`. Com
  `relax` **desconectado** (CVT estático) o memo cacheia → livre. Mas **animar `relax`** re-roda a relaxação
  INTEIRA todo frame (raw+CVT são fixos, mas um nó `Pure` não cacheia entre frames). (1) O `RESOLUTION=64²`
  fixo custava **12.8 ms/frame em debug** → **`res` adaptativa ao count** (`√(count·16)`, clamp [20,96]) + cena
  mais leve → ~1.2 ms. (2) Enio reportou count 180 → 20fps; o cook era **single-thread num core** (o walk do
  `Cook` chama `op.eval` em série; a máquina tem 32). Fix: a **busca do ponto mais próximo paraleliza com
  rayon** (o `O(res²·count)`), acumulação segue ordenada → **bit-idêntico ao serial** (replay-hash intacto).
  count 180 caiu de ~20ms → **3 ms** (debug, ~8×; bandwidth-bound). A cena subiu p/ count 96. **`O(count²)` do
  Lloyd NÃO muda** — count ~2000 animado ainda custa ~64ms; pra milhões, é lowering GPU (`LoweringKind::Wgsl`,
  desenhado mas não implementado — decisão estratégica do Enio).

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-lattice` (drop-crate, a REDE HEXAGONAL):** `(jitter?) → out`. `rows×cols` pontos no
empacotamento triangular (passo de linha `√3/2`, ímpares meia-deslocadas); `jitter` value input melta a
colmeia. `Effect::Pure`, Source, `hash.rs` (splitmix). Display "Lattice".

**`ph2d-node-motion-voronoi` (drop-crate, a RELAXAÇÃO DE LLOYD):** `(relax?) → out`. `count` pontos relaxados
por `iterations` passos de Lloyd (grade `res²` adaptativa ao count), `relax` value input lerpa raw→CVT. `Effect::Pure`,
Source, `hash.rs`, `sqrt`-free. Display "Voronoi".

**Cena boot — DUAS cenas pequenas** (`motion_demo_strobe.rs`, 10 nós, 2 sinks):

```
ESQUERDA (lattice): lattice → move(−6) → tint(âmbar) → output   lfo_jitter → lattice.jitter
DIREITA  (voronoi): voronoi → move(+6) → tint(ciano) → output   lfo_relax  → voronoi.relax
```

- **lattice** (x≈−6): o `jitter` (`value.lfo` ∈ [0, 0.5]) desloca a colmeia → **cintila e reforma**.
- **voronoi** (centro x≈+6): o `relax` (`value.lfo` ∈ [0, 1]) toca a relaxação de Lloyd → a nuvem **se
  organiza numa colmeia uniforme e dissolve** de volta pro ruído.

Cada distribuição é `Pure` (função dos params + input de valor) — a animação vem 100% do domínio de valor.
Pequena e legível (regra do doc 12 / feedback "simplifique"). O `motion.move` desloca cada cena (as
distribuições nascem centradas na origem). Vários `motion.output` compõem num só draw.

**Testes (14 unit + 3 integração):** lattice (7, inclui 2 do hash: **empacotamento hexagonal**, **ímpares
meia-deslocadas**, centrado, **jitter melta deterministicamente**, cook); voronoi (7, inclui 2 do hash:
**Lloyd empareja** [min-gap CVT ≫ ruído], **relax lerpa** [0/0.5/1], dentro-do-domínio, replay + seed,
cook). Integração no shell: `the_lattice_shimmers_under_the_jitter_lfo` (ponto varre + na esquerda) ·
`the_voronoi_relaxes_into_an_even_honeycomb` (min-gap cresce quando relaxado + na direita) ·
`the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-lattice`, tipo `motion.lattice` | nova | nome novo |
| crate `ph2d-node-motion-voronoi`, tipo `motion.voronoi` | nova | nome novo |
| `hash.rs` (copiado de `motion.scatter`) em ambas | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (52 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita p/ lattice+voronoi, 2 cenas, 10 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates só dependem de `ph2d-nodegraph` +
`ph2d-node-registry` (machete verde).

## 5. O que fica (doc 22 §5)

- **Distribuições (M3):** `motion.distribute-path` (distribui ao longo de uma curva — **integra vector.***,
  cross-module; DEFERIDO até uma crate-satélite que leia o contrato de vetor).
- **Deformers (M3):** `motion.four-point-warp` (corner-pin / bilinear) · `motion.slit-scan` (offset temporal
  por-coluna — **precisa de amostragem temporal do stream**, arquiteturalmente mais pesado; DEFERIDO).
- **Simulação:** `motion.pin_constraint` (precisa de port de cadeia de constraints; DEFERIDO) · spatial hash
  no boids (só-perf) · colisão.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

> As distribuições do M3 estão COMPLETAS na parte auto-contida (grid, fibonacci, scatter, lattice, voronoi);
> só `path` fica, e depende do módulo de vetor. O que resta do M3 são deformers (four-point-warp, slit-scan) e
> o straggler `delay`.
