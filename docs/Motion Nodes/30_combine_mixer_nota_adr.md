# 30 — Nota-ADR: Combine + Mixer (M1 — os operadores de STREAM, branch-and-merge)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: 2/1/8). Adiciona os dois operadores de **stream** do lote M1 (doc 01
§1.7): **`motion.combine`** (concatena) e **`motion.mixer`** (mistura element-wise). São os **primeiros nós
multi-input** — até aqui todo grafo era uma **cadeia linear única**; agora dá pra **ramificar e mesclar**.

---

## 1. O problema

Nenhum nó **fundia** streams. Cada grafo era uma linha reta (fonte → modifiers → output). Sem merge não existe
"grid E anel juntos", "duas distribuições numa cena", nem morph entre dois layouts. É o buraco topológico do
M1 (o sort/cull da fatia 27 reordena/poda UM stream; estes operam ENTRE streams).

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.combine` (concatenar):** o padrão-ouro é o **Merge SOP** do Houdini / o **Join Geometry** do Blender
— empilha várias geometrias numa só. Veredito:

- Até 4 inputs; a saída tem `Σ counts`. **União de colunas**: pra cada nome que aparece em QUALQUER input,
  concatena na ordem dos inputs; um input que não tem a coluna contribui **zeros** (a convenção do Merge — um
  atributo ausente lê como default), então os streams se alinham. HR-5: cópia pura. `Effect::Pure`, Utility.
  Testado por: soma as contagens + empilha (falsifica sobrescrever) · coluna presente em 1 só input é
  zero-filled no outro · cook funde 2 fontes na contagem somada.

**`motion.mixer` (misturar element-wise):** o padrão-ouro é o **Attribute Interpolate** / **Sequence Blend** do
Houdini — interpola atributos entre geometrias. Veredito:

- Até 4 inputs + um `blend` value input. A contagem é o **mínimo** entre os contribuintes (o excedente do
  input mais longo é descartado — convenção do Sequence Blend). Cada coluna presente em TODOS os contribuintes
  é reduzida: **Avg** (média) · **Add** (soma) sobre todos os não-vazios; **Blend** = `lerp(in0, in1, blend)`
  (o value input, 0..1, desconectado → 0.5). Blend de dois layouts + `value.lfo` = **morph** (quadrado ↔
  círculo). HR-5: aritmética de componente. `Effect::Pure`, Utility. Testado por: Avg é o ponto-médio · Add
  soma · **Blend faz lerp in0→in1** (falsifica média que ignora o peso) · contagem = mínimo · cook.

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-combine` (drop-crate, CONCATENAR):** `(in0..in3) → out`. União de colunas com zero-fill,
contagem somada. `Pure`, Utility. Display "Combine".

**`ph2d-node-motion-mixer` (drop-crate, MISTURAR):** `(in0..in3, blend?) → out`. Avg/Add/Blend element-wise,
contagem = mínimo. `Pure`, Utility. Display "Mixer".

**Cena boot — DUAS cenas, os primeiros grafos branch-and-merge** (`motion_demo_strobe.rs`, 14 nós):

```
ESQUERDA (combine):  grid ┐→ combine → tint → move(−6) → output      ring(spin) ┘
DIREITA  (mixer):    grid ┐→ mixer(Blend) → tint → move(+6) → output  circle ┘   lfo(sine) → blend
```

- **combine** (x≈−6): um grid 10×10 + um **anel de 40 pontos girando** → **um** stream de **140** dots
  (concatenação). Cada cena é um **Y** (duas fontes convergindo no nó novo).
- **mixer** (x≈+6): um grid 8×8 (64) blendado num **círculo de 64** → um **morph quadrado↔círculo**; o `blend`
  (`value.lfo` sine) mistura ida e volta.

**Testes (8 unit + 3 integração):** combine (3: soma-concatena, zero-fill, cook-funde); mixer (5: avg-midpoint,
add-soma, blend-lerp, count-mínimo, cook). Integração no shell: `the_grid_and_ring_combine` (**140** = 100+40 +
anel gira + esquerda) · `the_grid_morphs_into_the_circle` (**64** = min + o blend viaja muito + direita) ·
`the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-combine`, tipo `motion.combine` | nova | nome novo |
| crate `ph2d-node-motion-mixer`, tipo `motion.mixer` | nova | nome novo |
| `ph2d-node-registry-init` regenerado (66 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas, 14 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo (só path crates). **Primeiros nós multi-input** (4
input ports) — o contrato de nós já suportava (`inputs: &[PortSpec]`), sem mudança. Machete verde. Zero tofu.

## 5. O que fica (a cauda M1)

Streams fecham. A cauda M1 self-contained que resta:
- **Expressão:** `motion.expression` (parser VEX-lite → `ph2d-expr`; `i,n,t` virtuais; erro → badge) — o mais
  poderoso do lote M1, merece fatia própria. Self-contained.
- **Adapters:** `adapt-value-to-color` · `-luminance` · `-threshold` · `-gate` · `-make-point` — self-contained.
- **Cor:** `color-range-to-color` subsumido pelo `color_ramp` Custom (opcional).

Fora da cauda M1: M3 completo · **M4 (Rig+FX, necks)** · **M5 (GPU, ADR)** · deferidos cross-module
(`distribute-path`, `slit-scan`/`delay`) · pendências M2 (scrub-back wiring, `delay`, `buoyancy`).

> Com combine + mixer o grafo deixa de ser uma linha reta: **ramifica e mescla**. É a peça topológica que
> faltava. A cauda M1 vira essencialmente a **expressão** (o escape-hatch de fórmula) + adapters — ainda tudo
> self-contained. Ou **integrar** as 15 fatias.
