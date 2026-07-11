# 19 — Nota-ADR: Scatter + Morph (M3.2 — blue-noise + crossfade)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia M3.2 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Segunda fatia do
M3 (doc 01 §3): a distribuição **blue-noise** e o deformer **crossfade** — que juntos fazem um girassol
**dissolver numa nuvem e voltar**.

---

## 1. O problema (doc 01 §3 / doc 18 §5 — M3)

O doc 18 abriu o M3 com fibonacci (distribuição ordenada) + twist (deformer rotacional). Faltavam duas
peças complementares e icônicas: a **distribuição oposta** (aleatória-mas-uniforme, o blue-noise) e um
deformer que **interpola entre duas formas** (o morph/blend-shape) — juntos, o contraste
ordem ⇄ aleatoriedade numa transição contínua.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.scatter` (blue-noise):** os geradores blue-noise de referência são **Bridson** (dart-throwing
Poisson-disk, 2007 — enche um domínio até um raio mínimo, **count implícito**) e **Mitchell
best-candidate** (1991 — coloca exatamente `N`, cada um o mais distante de `k` dardos aleatórios dos já
colocados, **count exato**). Veredito:

- **Escolhi best-candidate**, não Bridson. Racional: um NÓ tem um param `count`, então **count exato**
  encaixa muito melhor (Bridson daria um count variável em função do raio/domínio); best-candidate é mais
  simples, allocation-bounded, e sua qualidade blue-noise é excelente pra scatter (cada ponto maximiza a
  distância aos vizinhos). É uma decisão de FIT, documentada — não um chute.
- White-noise (random puro) aglomera e deixa buracos; blue-noise o olho lê como "natural". O nó é o
  oposto exato da `motion.fibonacci` ordenada. Stateless (Jarzynski/Olano: cada dardo é `hash(seed,
  index, candidate, lane)`) → `Pure`, scrub-stable. HR-5: hash + distâncias AO QUADRADO (compara `d²`,
  sem `√`) + min/max. Source node.

**`motion.morph` (crossfade):** o **morph target / blend-shape** clássico (Maya/Blender/AE), reduzido a
pontos. Veredito:

- **Interpola posição-por-posição** por um `blend`: `P_i = lerp(a_i, b_i, blend_i)`. `blend = 0` → tudo
  `a`, `1` → tudo `b`. É o deformer que reesculpe por INTERPOLAÇÃO (vs `twist`, por função espacial). O
  `blend` é um **input de valor** (animável por `value.lfo`) — o domínio de valor (docs 12–17) dirigindo
  outro deformer do M3. Per-element (um `blend` length-N faz um dissolve escalonado). Saída = `min(len_a,
  len_b)` (só os pares alinhados por índice morfam). `Pure`, HR-5 (um lerp).

## 3. O que foi adicionado (fatia M3.2)

**`ph2d-node-motion-scatter` (drop-crate, a DISTRIBUIÇÃO):** `() → stream`. `count` pontos blue-noise num
retângulo `w×h` (centrado), best-candidate (K=12 dardos/ponto). Params `count`/`width`/`height`/`seed`.
`Pure`. `NodeUiCategory::Source`. Display "Scatter".

**`ph2d-node-motion-morph` (drop-crate, o DEFORMER):** `(a, b, blend?) → stream`. `lerp(a_i, b_i,
blend_i)`; `blend` value input (desconectado → 0 = `a`), per-element, clamp `[0,1]`; saída `min(len)`.
`Pure`. `NodeUiCategory::Transform`. Display "Morph".

**Cena boot — um GIRASSOL QUE DISSOLVE NUMA NUVEM** (`motion_demo_strobe.rs`, ~9 nós):

```
fibonacci ─┐
scatter ───┼→ morph → tint → drive_size → output
lfo ───────┘ (blend)
morph → instance_field → size_range → drive_size.value
```

- `fibonacci` (180 sementes, golden angle) = shape `a` (a espiral ordenada).
- `scatter` (180 pontos blue-noise, 4×4) = shape `b` (a nuvem uniforme-aleatória).
- `morph` faz o crossfade; o `blend` é um `value.lfo` lento (0..1) → o girassol **derrete na nuvem e se
  reforma**. Ordem ⇄ aleatoriedade.
- `instance_field(Ramp) → size_range → drive_size` dimensiona as sementes por índice (cada uma mantém o
  tamanho enquanto migra).

Duas distribuições do M3 ligadas por um deformer dirigido por valor. Cena playhead-pura (sem estado `pre`).

**Testes (12 unit + 2 integração):** scatter (6: dentro do retângulo, **blue-noise vs white-noise** [o gap
mínimo do best-candidate >4× o do primeiro-dardo], determinístico/re-roll por seed, através do cook, +
hash — falsificados); morph (6: crossfade a→b/midpoint, empty=a & clamp, per-element escalonado, mismatch
= prefixo pareado, através do cook, resolve — falsificados). Integração no shell:
`the_morph_dissolves_the_spiral_into_the_scatter` (**no vale [blend≈0] é a espiral [raio cresce por
índice]; no pico [blend≈1] é uma forma muito diferente [deslocamento total grande] que enche o campo do
scatter e NÃO tem a ordem índice→raio** — morph morto casaria os dois frames; scatter morto deixaria o
pico como espiral). Determinismo (playhead-puro).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-scatter`, tipo `motion.scatter` | nova | nome novo |
| crate `ph2d-node-motion-morph`, tipo `motion.morph` | nova | nome novo |
| `hash.rs` (copiado de `value.instance_field`, `pub(crate) hash3`) em scatter | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (44 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita p/ o girassol↔nuvem, ~9 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem (o loop-replay do doc 11 JÁ tem doc próprio desde a M3.1) |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de
`ph2d-nodegraph` + `ph2d-node-registry` (machete verde).

## 5. O que fica (M3, doc 01 §3)

- **Distribuições:** `motion.distribute-voronoi` (Lloyd) · `-path` (integra vector.*) · `motion.lattice`.
  (Bridson Poisson-disk fica como variante *fill-a-domain* futura, se um caso precisar de count implícito.)
- **Deformers:** `motion.bend` · `-four-point-warp` · `motion.look-at` · `-slit-scan`.
- **Sim/agentes:** `motion.boids` (spatial hash) · `-verlet-rope` · `-soft-body` (XPBD) · `-pin-constraint`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

> M3.1 (fibonacci+twist) + M3.2 (scatter+morph) estabelecem o padrão do M3: **distribuições ricas** (ordem
> e blue-noise) + **deformers dirigidos por valor** (rotacional, interpolação). O resto do M3 (voronoi,
> bend, boids, ropes) segue o mesmo molde.
