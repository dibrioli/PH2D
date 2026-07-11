# 29 — Nota-ADR: Color Ramp + Color Array (M1 — a família COR, cauda self-contained)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: 2/1/8). Abre a **família de cor do M1** — os nós que o plano lista
(doc 01 §1.7, lote "Cor") e que faltavam: **`motion.color_ramp`** (gradiente contínuo) e
**`motion.color_array`** (paleta discreta). Até aqui a única fonte de cor era o `motion.tint` (um sólido).

> **Nota de escopo (correção honesta):** eu vinha dizendo que "o poço self-contained secou". Consultar o plano
> mostrou que secou **só** nas distribuições/deformers do M3 — a **cauda M1 (cor/expressão/adapters/streams)**
> continua self-contained e não foi feita. Esta fatia começa essa cauda pela cor. Deps: `ph2d-color` (lib
> **foundational**, já no workspace — não é outro módulo-feature; o color_ramp depende dela pelo `ColorRamp`).

---

## 1. O problema

Instâncias sem cor variável são metade de um sistema de motion graphics. O `motion.tint` só pinta um sólido
(× o `tint` existente). Faltavam os dois nós de cor universais: **colorir por um valor** (gradiente — cor por
índice/distância/velocidade) e **ciclar uma paleta** (o "colorArray" do gate M1 do próprio plano, doc 01 §3).

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.color_ramp` (gradiente contínuo):** o padrão-ouro é o **"Color Ramp" do Blender** / o ramp-parameter
do Houdini — um escalar `t ∈ [0,1]` mapeado por um gradiente multi-stop. Veredito:

- Reusa o **`ph2d-color::ColorRamp`** (a lib foundational já tem o modelo de stops + `eval(t)→[f32;4]` em RGBA
  linear — a MESMA base do color ramp do painter). O `t` de cada instância é o value input `v[i]` (clamp 0..1)
  quando conectado, senão o **índice normalizado** `i/(n−1)`. `preset` (Rainbow/Heat/Ice/Grayscale/Custom) +
  `interp` (Linear/Ease). Avaliado em **RGB linear** — o mesmo espaço da coluna `tint` e do compositor → zero
  conversão de cor no nó. HR-5: o nó só faz aritmética + chama `ramp.eval` (a matemática de cor é da
  `ph2d-color`). `Effect::Pure`, categoria Fx. Testado por: grayscale espalha preto→branco por índice · o `t`
  field sobrepõe o índice · o Rainbow **abrange muitas cores** (falsifica um sólido) · cook escreve `tint`.

**`motion.color_array` (paleta discreta):** o padrão-ouro é o **MoGraph "Color"** do C4D — uma paleta ciclada
por índice. Veredito:

- Até 4 slots de cor (RGB linear), `colors` (2–4) ativos; elemento `i` pega o slot `(i + offset) mod colors`.
  Um `offset` value input marcha a paleta (um `value.lfo` cicla as listras). O contraponto **discreto** do
  ramp (listras duras vs gradiente). HR-5: um lookup por módulo, sem matemática. `Effect::Pure`, Fx. Testado
  por: cicla por índice (`i mod 3`) · offset marcha · `colors` limita os slots ativos · cook escreve `tint`.

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-color-ramp` (drop-crate, GRADIENTE):** `(in, t?) → out`. Colore por `t` (ou índice) via
`ColorRamp` de preset. `Pure`, Fx, dep `ph2d-color`. Display "Color Ramp".

**`ph2d-node-motion-color-array` (drop-crate, PALETA):** `(in, offset?) → out`. Cicla 2–4 cores por índice,
`offset` marcha. `Pure`, Fx, sem deps extra. Display "Color Array".

**Cena boot — DUAS cenas** (`motion_demo_strobe.rs`, 10 nós):

```
ESQUERDA (rainbow):  distribute_radial → color_ramp(Rainbow) → move(−6) → output   lfo(sine) → spin
DIREITA  (palette):  grid → color_array(4 cores) → move(+6) → output               lfo(saw)  → offset
```

- **rainbow** (x≈−6): 60 pontos radiais em 4 anéis, coloridos por índice no ramp Rainbow → **anéis espectrais
  concêntricos**; o sunburst **gira** (spin ← sine lfo).
- **palette** (x≈+6): um grid 10×10 com a paleta default (vermelho/verde/azul/amarelo) por `i mod 4`; o
  **offset saw lfo marcha** a paleta pelo grid (listras ciclando).

**Testes (8 unit + 3 integração):** color_ramp (4: grayscale-por-índice, t-field-sobrepõe, rainbow-abrange,
cook-escreve-tint); color_array (4: cicla-por-índice, offset-marcha, colors-limita, cook). Integração no shell
(lê a coluna `tint` Vec4): `the_rainbow_sunburst_spins_and_is_colourful` (60 pts + gira + **>20 cores
distintas** + esquerda) · `the_palette_grid_marches` (100 pts + **exatamente 4 cores** + a cor do índice-0 muda
no tempo + direita) · `the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-color-ramp`, tipo `motion.color_ramp` | nova | nome novo |
| crate `ph2d-node-motion-color-array`, tipo `motion.color_array` | nova | nome novo |
| **dep `ph2d-color`** em color-ramp (lib foundational, JÁ no workspace) | Cargo.toml | nenhum (sem dep externa nova; sem RUSTSEC) |
| `ph2d-node-registry-init` regenerado (64 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas, 10 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token novo. A dep `ph2d-color` é o **primeiro** node-crate da linha
a depender de uma lib foundational além de `ph2d-nodegraph`/`-node-registry` — mas é foundational (não módulo-
feature), já membro do workspace, então nada novo no lockfile. Machete verde (ColorRamp usado). **Nota:**
`no_tofu_glyphs` pegou 1 seta `→` em string literal de teste (color-ramp) — trocada por texto.

## 5. O que fica (a cauda M1 continua)

A cor abre; o resto da **cauda M1 self-contained** (doc 01 §3, lote M1) segue como fan-out puro:
- **Cor:** `color-range-to-color` — subsumido pelo `color_ramp` Custom (2-stop); um nó dedicado é opcional.
- **Expressão:** `motion.expression` (parser VEX-lite → `ph2d-expr`, já no workspace; `i,n,t` virtuais; erro →
  badge) — self-contained, é o próximo grande da cauda.
- **Adapters:** `adapt-value-to-color` · `-luminance` · `-threshold` · `-gate` · `-make-point` — self-contained.
- **Streams:** `motion-mixer` (avg/add ≤4) · `motion-combine` (concat) — self-contained.

Fora da cauda M1 (inalterado): distribuições/deformers M3 completos · fronteiras **M4 (Rig+FX, exige necks)**
e **M5 (GPU, exige ADR)** · deferidos cross-module (`distribute-path` vetor, `slit-scan`/`delay` time-scope,
`pin_constraint`).

> A cor era o buraco mais visível do M1. Com ramp + array, o vocabulário de **aparência** ganha as duas peças
> universais. O próximo passo natural da cauda é a **expressão** (o nó mais poderoso do lote M1), depois
> adapters/streams — tudo ainda self-contained.
