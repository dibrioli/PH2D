# 26 — Nota-ADR: Kaleidoscope + Collide (M3 — simetria N-fold + empacotamento)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: 2/1/8). Adiciona a **simetria N-fold** (a generalização do
`motion.mirror`, que era só D₁) e a **colisão/empacotamento** (o modifier de não-penetração — distinto do
CVT do voronoi). A cena boot vira um **mandala girando** + um **grid que se empacota respirando**.

---

## 1. O problema

Tínhamos o `motion.mirror` (reflexão única = D₁) mas nenhuma **simetria N-fold** (mandala/caleidoscópio — o
radial cloner-com-rotação do C4D, o "CC Kaleida" do AE, o Kaleidoscope do Blender). E tínhamos o
`motion.voronoi` (CVT — espalha para densidade *uniforme*) mas nenhuma **colisão/empacotamento** (o hard-radius
que impede sobreposição — o "Push Apart Effector" do C4D). Duas capacidades genuinamente ausentes e ambas
**muito visuais** (mandala girando · nuvem que se empacota) — bons demos auto-play.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.kaleidoscope` (a simetria N-fold):** o padrão-ouro é a **órbita da fonte sob o grupo diedral Dₙ**.
Veredito:

- Para uma fonte de `n` elementos e `segments = k`, a saída tem `k · n`: a fatia `s` é a fonte **rotacionada**
  em torno de `(pivot_x, pivot_y)` por `s · (1/k)` de volta (+ `spin` global). Com **rotacional** (`reflect`
  off) é o grupo cíclico Cₖ — `k` cópias giradas. Com **refletido** (`reflect` on) toda fatia ímpar é
  **espelhada** antes de girar (o `y` local negado) → fatias vizinhas se encontram como imagem-espelho: o
  grupo diedral Dₖ, o visual caleidoscópio. Só `P` é transformado; as outras colunas duplicam por fatia. HR-5:
  a rotação usa `cos_sin_cycles` (seno parabólico copiado de `motion.orbit`, ~0.09% erro), **sem `sin`/`cos`**,
  sem `sqrt`. `Effect::Pure` (o `spin` chega pelo value input). Testado por: **conta × segments** · cópias
  rotacionais no anel (¼ de volta leva +x → +y) · **reflect espelha a fatia ímpar** (o x flipa vs a rotacional
  — falsifica ignorar `reflect`) · spin gira · o **pivô é o ponto fixo**.

**`motion.collide` (o empacotamento):** o padrão-ouro é a **restrição de não-penetração da Position Based
Dynamics** (Müller et al., *Position Based Dynamics*, 2007; a relaxação é Jakobsen, *Advanced Character
Physics*, 2001), exposta como o **"Push Apart Effector"** do Cinema 4D (que tem justamente um parâmetro
`Iterations`). Veredito:

- Cada instância é um disco de raio `radius`. Para cada par mais perto que `2·radius`, o gradiente da restrição
  é a **normal de contato** (o vetor unitário entre eles) e a correção é **metade da penetração cada**,
  afastando ao longo da normal → o par acaba **encostando** com o ponto-médio preservado. A projeção varre
  todos os pares `iterations` vezes (Gauss–Seidel), o que deixa uma nuvem apertada assentar num empacotamento.
  É uma **relaxação PURA da entrada a cada cook** (sem estado, como o Lloyd do voronoi), então um `spread`
  (multiplicador de raio) animado faz o empacotamento **respirar** — determinístico e replay-safe. HR-5:
  aritmética + `sqrt` (permitido), sem trig. `Effect::Pure`. Par coincidente (normal indefinida) é aberto por
  eixo determinístico (x/y por paridade de `i+j`) — sem rng. Testado por: **par sobreposto separa até
  encostar** (2·radius) · **separados ficam intactos** · ponto-médio preservado · **cluster empacota sem
  sobreposição** · coincidentes abrem determinístico · `strength`/`radius` 0 = identidade. Distinto do voronoi
  (CVT = densidade uniforme; collide = hard-radius, empacotamento). O(n²·iterations): ok no budget do nó; a
  rota de escala (spatial hash / GPU) é `docs/plans/2026-07-gpu-resident-node-pipeline.md`.

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-kaleidoscope` (drop-crate, a SIMETRIA N-fold):** `(in, spin?) → out`. Replica em
`segments` fatias giradas por `pivot`, `reflect` espelha as ímpares. `Pure`, Transform, `trig.rs`
(cos_sin_cycles). Display "Kaleidoscope".

**`ph2d-node-motion-collide` (drop-crate, o EMPACOTAMENTO):** `(in, spread?) → out`. Empurra pares que penetram
o raio `radius` (× `spread`), `iterations` varreduras Gauss–Seidel. `Pure`, Transform, aritmética + `sqrt`.
Display "Collide".

**Cena boot — DUAS cenas** (`motion_demo_strobe.rs`, 12 nós):

```
ESQUERDA (mandala): fibonacci → kaleidoscope → move(−6) → tint(âmbar) → output   lfo → spin
DIREITA  (packing): grid → collide → move(+6) → tint(ciano) → output             lfo → spread
```

- **mandala** (x≈−6): uma espiral Fibonacci de 6 sementes (r≈0..2) dobrada em **8 fatias espelhadas** (48
  dots); o `spin` (`value.lfo` ±180°) gira o mandala. A semente 0 fica no pivô (ponto fixo, ancora o centro).
- **packing** (x≈+6): um grid **8×8 com espaçamento 0.45 mais apertado que o diâmetro 0.6** → todo mundo
  sobrepõe → `motion.collide` empurra para um empacotamento (64 dots); o `spread` (`value.lfo` [0.3, 1.5])
  respira o raio → a nuvem infla e assenta. **É o Push-Apart ao vivo.**

**Testes (15 unit + 3 integração):** kaleidoscope (8, inclui 2 do trig: conta×segments, cópias-no-anel,
reflect-espelha, spin-gira, pivô-fixo, cook duplica colunas); collide (7: separa-até-encostar, separados-
intactos, ponto-médio, cluster-empacota, coincidentes-determinístico, strength/radius-0-identidade, cook).
Integração no shell: `the_mandala_spins` (48 dots = 6×8 + viaja + esquerda) · `the_grid_packs_apart_and_
breathes` (64 dots + **collide quebra o grid 0.45** [min-pair > 0.45 no frame mais largo] + respira + direita)
· `the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-kaleidoscope`, tipo `motion.kaleidoscope` | nova | nome novo |
| crate `ph2d-node-motion-collide`, tipo `motion.collide` | nova | nome novo |
| `trig.rs` (copiado de `motion.orbit`) em kaleidoscope | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (58 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas, 12 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo (só path crates). Machete verde. **Nota:** o gate
`no_tofu_glyphs` pegou 2 setas `→` latentes em string literals de teste (uma pré-existente da fatia 25, uma
nova) — trocadas por `->`. Latente que só o ship/gate completo pega (per memória do integrador).

## 5. O que fica

O vocabulário de **arranjo, deformação, simetria e empacotamento** do M3 fecha: distribuições (retangular,
espiral, blue-noise, hex, CVT, radial), deformers (rotação, interpolação, arc, orientação, corner-pin, lente),
os modifiers de simetria (mirror D₁, **kaleidoscope Dₙ**) e o **empacotamento (collide)**. O que resta depende
de outros módulos ou é a fronteira estratégica — todos deferidos com razão anotada:
- **Distribuição:** `motion.distribute-path` (curva — integra `vector.*`; DEFERIDO).
- **Deformer:** `motion.slit-scan` (amostragem temporal; DEFERIDO).
- **Simulação:** `pin_constraint` (port de constraints; DEFERIDO) · spatial hash (acelera collide/boids) ·
  colisão contra bordas/paredes · **motor GPU** (`docs/plans/2026-07-gpu-resident-node-pipeline.md` — a
  próxima grande alavanca de capacidade, exige ADR + toca foundational).
- Straggler do M2: `motion.delay` (precisa do time-scope do editor, como `trail`/`time_remap`; DEFERIDO).

> Com kaleidoscope + collide, o poço de nós **auto-contidos** (que dependem só de `ph2d-nodegraph`) do M3 está
> essencialmente **esgotado** — o próximo salto real é o motor GPU-residente (linha foundational dedicada) ou a
> integração com outros módulos (path/vetor, timeline/keyframes). Hora natural de **integrar** as fatias
> acumuladas.
