# ADR-0058-amendment-1 — Vector domain cook payload: type-erased opaque geometry value

**Status:** Accepted (2026-06-03)
**Amends:** [ADR-0058 §2.1](0058-vector-geometry-graph.md) (Vector geometry graph — domain `vector` no `ph2d-nodegraph`).
**Decisor(es):** Enio + Claude (sessão Vector W3, onboarding T3.2).
**Trigger:** T3.2 (`ph2d-node-vector-source`) bloqueou no arranque: a ADR-0058 §2.1 declara o domain `vector` mas **não especifica o mecanismo concreto** pelo qual um nó emite/consome um `VectorNetwork`. A spec normativa [`02_geometry_graph.md` §2.2.1](../Vector%20Module/02_geometry_graph.md) mostra `fn eval(ctx) -> Result<VectorNetwork>` + `Output::path("network")` — **pseudocódigo aspiracional que nunca existiu no substrato**. O contrato real (`NodeOp::eval(&self, ctx) -> ()` emitindo `Stream` de `Scalar/Vec2/Vec3`) não tem carrier para geometria. Esta amendment fecha o gap.

---

## 1. Contexto

[ADR-0058](0058-vector-geometry-graph.md) ratificou `vector` como o 2º domain do `ph2d-nodegraph` (após `motion`), com fan-out drop-crate de 18 nós. §2.1 afirma "caps ADR-0039 (`NodeOp=2 / OpResolver=1 / NodeManifest=8`) continuam válidos … sem mexer contrato base nodegraph" — **correto**, mas §2.1 nunca disse **como** um `VectorNetwork` trafega numa edge.

Auditoria do substrato real no arranque do W3 (sessão T3.2):

| Spec `02_geometry_graph.md` §2.2.1 (pseudocódigo) | Substrato real `ph2d-nodegraph` |
|---|---|
| `fn eval(ctx) -> Result<VectorNetwork>` | `fn eval(&self, ctx: &mut EvalCtx) -> ()` emite `Stream` ([`node.rs`](../../../crates/ph2d-nodegraph/src/node.rs), [`cook.rs:86`](../../../crates/ph2d-nodegraph/src/cook.rs)) |
| `Output::path("network")` carrega VectorNetwork | `Column` só tem `Scalar/Vec2/Vec3` ([`attr.rs`](../../../crates/ph2d-nodegraph/src/attr.rs)) — **zero carrier de geometria** |
| `Param::enum_var / u32 / f32(range)` | `ParamSpec { name, default: f32 }` — só f32 |
| `Clock::None` | `Clock` = `Static / Frame / Audio / Event` — **sem `None`** |
| `ctx.param_enum / param_f32 / param_u32` | só `ctx.param(name) -> f32` |

Grep de `crates/` + `shells/`: **0 hits** para `-> VectorNetwork`, `Output::path`, `param_f32`, `Param::enum_var`, `Clock::None`. Nenhum node-crate referencia `VectorNetwork`; o W2 renderiza **tool-direct** (bridges → `committed_vector_*_paths` → `vector_scene::reconcile` → ECS), **fora** do nodegraph. Logo: nem produtor (nó) nem consumidor (graph) plugados.

**Raiz:** o `motion` domain cabe no substrato porque dados de motion *são* `Scalar/Vec2/Vec3` SoA por-instância. Um `VectorNetwork` (grafo de `vertices`/`segments`/`regions`, cubics, `VertexKind` — ADR-0056) é um **valor rico de topologia variável**, não um stream SoA por-elemento. Não decompõe em colunas flat sem encoding lossy. Precisa de um carrier novo.

---

## 2. Decisão

**Generalizar o valor que trafega na edge do cook — hoje hard-coded como `Stream` — para um payload type-erased, mantendo o substrato `ph2d-nodegraph` domain-agnostic (zero deps; leaf).** Uma edge de geometria carrega um `VectorNetwork` embrulhado em `Arc<dyn Any + Send + Sync>`.

### 2.1 `CookValue` — o valor por-output do cook

```rust
// ph2d-nodegraph/src/attr.rs (ou value.rs)
#[derive(Clone, Default)]
pub enum CookValue {
    #[default]
    Empty,
    Instances(Stream),                       // motion / field — o caminho existente
    Opaque(Arc<dyn Any + Send + Sync>),      // valor rico domain-específico (e.g. VectorNetwork)
}
```

- `EvalCtx.inputs/outputs` e `Cook::{cache, prev_outputs}` passam de `Stream`/`Vec<Stream>` para `CookValue`/`Vec<CookValue>`.
- `Arc` mantém o design clone-happy do cook (`cur_output`/`prev_output` clonam por-tick) **barato**: refcount, não deep-copy da geometria.
- O substrato **não conhece `VectorNetwork`** — só `Arc<dyn Any>`. `ph2d-nodegraph` continua com `[dependencies]` vazio.

### 2.2 `EvalCtx` ganha acessores (NÃO é superfície gated)

```rust
// motion — inalterado (conveniência sobre CookValue::Instances):
pub fn input(&self, port: usize) -> &Stream { … }   // Empty/Opaque → &Stream vazio
pub fn emit(&mut self, stream: Stream) { … }         // → CookValue::Instances

// novo (canal opaco; tipagem fica na borda do domain):
pub fn input_any(&self, port: usize) -> Option<&Arc<dyn Any + Send + Sync>> { … }
pub fn emit_any(&mut self, value: Arc<dyn Any + Send + Sync>) { … }
```

`EvalCtx` **não** é coberto pelo gate `architecture_contract_surface` (só `NodeOp`/`OpResolver`/`NodeManifest` são). Adicionar métodos ao `EvalCtx` é evolução de substrato livre.

### 2.3 `Domain::Vector` + `Clock::Static`

- `Domain::Vector` adicionado ao enum `port.rs` (enum variant count **não** é gated). O comentário existente em `Domain::Instances` já antecipava "geometry … lowers to … vector" — agora é domain próprio, pois o *valor* difere (instance-stream vs. network).
- Geometria estática (source/boolean/offset sem animação) mapeia em **`Clock::Static`** que já existe ("cooked once, never re-evaluated" — [`port.rs:41`](../../../crates/ph2d-nodegraph/src/port.rs)). **Não** se adiciona `Clock::None`.

### 2.4 Params enum/u32 viajam como discriminante f32

O vocabulário de params **não muda** (continua `ParamSpec { name, default: f32 }`):
- `kind` (Rect/Ellipse/Polygon/Star/Spiral) → discriminante f32 `0.0..=4.0`, lido com `param_as_count` + match.
- `sides`, `turns` → f32 → `param_as_count` ([`node.rs`](../../../crates/ph2d-nodegraph/src/node.rs) **já provê** essa conversão total/saturating, desenhada exatamente para "f32 não-confiável → count").
- `width`, `height`, `inner_radius` → já são f32.

A spec mostrar `Param::u32`/`enum_var` é açúcar; o discriminante f32 cobre semanticamente sem tocar contrato. (Um vocabulário tipado de params é refinamento futuro, **fora** desta amendment.)

### 2.5 Glue crate `ph2d-vector-graph` (camada de borda)

Crate novo `crates/ph2d-vector-graph/` (deps: `ph2d-nodegraph` + `ph2d-vector-doc`) provê a tipagem na borda do domain, mantendo nodegraph leaf **e** vector-doc data-model puro:

```rust
pub trait VectorEvalExt {
    fn input_network(&self, port: usize) -> Option<&VectorNetwork>;  // downcast do Arc<dyn Any>
    fn emit_network(&mut self, net: VectorNetwork);                  // Arc::new + emit_any
}
impl VectorEvalExt for EvalCtx<'_> { … }
```

Os 18 node-crates vetoriais dependem de `ph2d-vector-graph`, não fazem downcast à mão.

### 2.6 Caps congelados — **intactos**

| Cap (ADR-0039 / gate `architecture_contract_surface`) | Antes | Depois |
|---|---|---|
| `NodeOp` métodos | 2 | **2** (manifest + eval) |
| `OpResolver` métodos | 1 | **1** |
| `NodeManifest` campos | 8 | **8** |

O carrier vive nos **internos ungated** (`EvalCtx`, valor do cook, `Domain`/`Clock` enums). Por isso **não é amendment de ADR-0039** — é a evolução de substrato que o freeze foi desenhado pra permitir (o freeze protege a *superfície que todo node-crate implementa*, não a representação interna de valor do cook).

---

## 3. Consequências

### 3.1 Positivas

- **Um substrato só.** Cook/memo/scheduler/determinismo/serialização únicos. Cross-domain edges que a ADR-0058 §2.7/§2.9 exige (motion→param vetorial, vetor→path de motion, Field/SDF→preview vetor via ADR-0065) viram **membrane crossings nativos in-engine**, não pontes inter-engine.
- **Substrato domain-agnostic preservado.** `Arc<dyn Any>` mantém `ph2d-nodegraph` com zero deps; motion/field/signal/control não pagam por geometria.
- **Freeze verde.** Zero cap bump; o fan-out W3/W4 (18 nós) constrói contra o mesmo contrato congelado.
- **Destrava T3.2..T3.5 + W4.** `vector-source` vira o 1º nó sobre o substrato real.

### 3.2 Negativas

- **`Stream` → `CookValue` toca `cook.rs` + `EvalCtx`** (substrato compartilhado). Nós motion existentes (transform/grid/clone) e os testes do cook **devem permanecer verdes** — `ctx.input(0)->&Stream` / `ctx.emit(stream)` mantidos como conveniência sobre `Instances`. Risco mitigado: a mudança é aditiva na borda da API do nó; só o tipo interno do cook muda.
- **`Arc<dyn Any>` perde checagem estática na edge.** Um downcast errado (e.g. ligar saída de motion num input vetorial) falha em runtime, não em compile-time. Mitigação: a validação de `PortType` (`Domain` mismatch ⇒ não conecta direto, [`port.rs`](../../../crates/ph2d-nodegraph/src/port.rs) `connects_directly`) já barra a edge no editor antes do cook; o downcast é a 2ª linha.
- **+1 crate** (`ph2d-vector-graph`). Aceito — mantém layering limpo (DIRETRIZ §3.A).

### 3.3 Neutras

- A spec `02_geometry_graph.md` §2.2.1 deve ser atualizada (o pseudocódigo `eval -> Result<VectorNetwork>` reflete intenção, não a API real `eval(&self, ctx)` + `emit_network`). Débito de doc rastreado no handoff W3.
- ADR-0058 §2.1 permanece válido (caps preservados); esta amendment é puramente aditiva — especifica o carrier que §2.1 deixou implícito.

---

## 4. Alternativas consideradas

### 4.1 Engine de grafo vetorial separado (rejeitada)

Substrato vetor-native próprio (`eval -> VectorNetwork`), fora do `ph2d-nodegraph`. **Por que rejeitada:** dois cooks/memo/cache/determinismo/serialização; cross-domain edges (motion↔vector, Field/SDF→vector — requisito de 1ª classe ADR-0058 §2.7 + ADR-0065) viram pontes inter-engine, duplicando exatamente o que as membranas tipadas do nodegraph (ADR-0030/0032) já entregam. A razão de existir do nodegraph é unificação multi-domain.

### 4.2 `Column::Network(VectorNetwork)` tipado no nodegraph (rejeitada)

Carrier tipado dentro do substrato. **Por que rejeitada:** força o substrato leaf (zero deps) a depender de `ph2d-vector-doc`, invertendo a camada e poluindo o substrato compartilhado por motion/field/signal com o tipo de um único domain. O canal opaco (`Arc<dyn Any>`) mantém o substrato domain-agnostic — padrão de Houdini/Blender-geo-nodes/USD (handle type-erased + acessor tipado na borda).

### 4.3 Encodar geometria em colunas Vec2 (rejeitada)

`P` = vertices Vec2 + colunas-índice para segments/regions. **Por que rejeitada:** lossy para cubics/regions/`VertexKind`; o encoding é ele próprio uma decisão de contrato; briga com o data model ADR-0056.

### 4.4 Estender `ParamSpec` para vocabulário tipado agora (adiada)

`Param::enum/u32/range`. **Por que adiada:** o discriminante f32 + `param_as_count` cobre os params do `vector-source`/`boolean`/`offset` sem tocar contrato. Vocabulário tipado é refinamento ortogonal; entra por ADR própria se/quando um nó exigir param que f32 não expresse (e.g. `String`/`Color`/`Path`-ref).

---

## 5. Implementação (Vector W3, T3.2 foundation)

1. **`ph2d-nodegraph`** (substrato):
   - `attr.rs` (ou `value.rs`): `CookValue { Empty, Instances(Stream), Opaque(Arc<dyn Any+Send+Sync>) }`.
   - `cook.rs`: `EvalCtx.inputs/outputs` + `Cook::{cache, prev_outputs, cur_output, prev_output}` sobre `CookValue`; `input`/`emit` mantidos (conveniência `Instances`); `input_any`/`emit_any` novos.
   - `port.rs`: `Domain::Vector`.
2. **`ph2d-vector-graph`** (crate novo, deps nodegraph + vector-doc): trait `VectorEvalExt` (`input_network`/`emit_network` via downcast).
3. **`ph2d-node-vector-source`** (T3.2): `NodeOp` multi-variant (kind discriminante f32) envolvendo `ph2d-vector-doc/src/primitives.rs`; golden tests bit-identical cross-OS por variant.
4. **`ph2d-node-registry-init`**: registrar `vector-source` (test hand-maintido cluster/staleness).
5. **Gates novos:** `vector_node_golden_source` per-variant.

Caps ADR-0039 inalterados (§2.6). Gate `architecture_contract_surface` permanece verde — **regredir os caps é fora-de-escopo desta amendment**.

---

## 6. Referências

- [ADR-0058 — Vector geometry graph](0058-vector-geometry-graph.md) (parent).
- [ADR-0039 — Nodegraph contract freeze](0039-nodegraph-contract-freeze-w2t4.md) (caps preservados; **não** amendado).
- [ADR-0056 — Vector Network data model](0056-vector-network-data-model.md) (`VectorNetwork` carregado).
- [ADR-0030/0031/0032 — Nodegraph substrate, FBP black box, typed membranes](0030-nodegraph-substrate.md).
- [ADR-0065 — Vector-SDF Hybrid GPU](0065-vector-sdf-hybrid.md) (cross-domain Field→vector preview).
- Spec normativa: [`02_geometry_graph.md`](../Vector%20Module/02_geometry_graph.md) §2.1.2 + §2.2.1 (pseudocódigo a atualizar).
- Memory: [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md) (padrão-ouro vence custo), [`feedback-tool-unit-green-integration-dead`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_tool_unit_green_integration_dead.md) (traçar caminho produtor→consumidor completo).
