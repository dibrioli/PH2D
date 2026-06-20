# Examples — fan-out drop-crate (paste-ready)

> **Companheiro do [DIRETRIZ.md §3.8](DIRETRIZ.md)**.
>
> §3.8.2 traz o briefing **parametrizado** com `<family>`/`<slug>`/`<domínio>`.
> Este doc instancia esse briefing **fim-a-fim para dois slugs concretos** —
> um node (`shader.blur`) e um tool sabor-(1) (`grayscale`) — com **todos os
> arquivos** que o agente deve criar, sem placeholder algum. Cole o briefing
> abaixo na sessão Implementador; ela só renomeia o slug e segue.

---

## Intent

Auditoria multi-agente (2026-05-24) identificou: o briefing parametrizado de
§3.8.2 é suficiente para um agente já familiarizado com o domínio, mas exige
que o agente faça substituições (`<slug>` → algo concreto) **e** decida sozinho
o que vai em cada arquivo. Para reduzir esse atrito a zero, este doc apresenta
dois exemplos **completamente instantiated**:

- **Example A** — `ph2d-node-shader-blur`: node Temporal com 1 entrada, 1 saída,
  2 params, golden test. ~90 LOC totais.
- **Example B** — `ph2d-tool-grayscale`: tool sabor-(1) one-shot stateless,
  algorithm puro, manifest, ícone, golden test. ~140 LOC totais.

Cada exemplo é um **template literal** — substitua `shader-blur` ou `grayscale`
pelo seu slug em todo o doc e o resultado é o crate fim-a-fim.

> **Não compilamos esses templates no workspace.** Eles são pedagógicos. As
> referências mecânicas (templates "vivos" no workspace) continuam sendo:
> - Node Pure: `crates/ph2d-node-debug-const/` (59 LOC)
> - Node Temporal + ph2d-expr: `crates/ph2d-node-debug-wave/`
> - Node vertical (3 nodes): `crates/ph2d-node-motion-{grid,clone,transform}/`
> - Tool sabor (1) one-shot: `crates/ph2d-tool-make-square/`
> - Tool sabor (2) palette modal: `crates/ph2d-tool-move/`
> - Tool sabor (3) stateful + panel: `crates/ph2d-tool-padding/` (leve) ou
>   `crates/ph2d-tool-bgremoval/` (completo)

---

## Example A — node Temporal: `ph2d-node-shader-blur`

Hipotético: uma node Temporal que recebe um stream de cores RGBA e produz
versão borrada. `radius` e `kernel` (gaussian/box) como params.

### Briefing (cole direto na sessão)

```
═══════════════════════════════════════════════════════════════════
BRIEFING — node-crate · slug: shader-blur · domínio: shader
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA: crates/ph2d-node-shader-blur/

ANTES DE CODAR: leia o mapa node↔tool em DIRETRIZ §3.8.1 (entry points,
contrato, vocab, templates). Sua família é "node Temporal" — copie a
estrutura de crates/ph2d-node-debug-wave/ como base.

O QUE VOCÊ FAZ (só dentro da sua pasta):
0. PRIMEIRO arquivo: src/lib.rs (mesmo com 1 linha — destrava o
   workspace pras outras sessões paralelas).
1. Cargo.toml: deps = ph2d-nodegraph, ph2d-node-registry.
   (Não precisa de ph2d-expr — o blur usa loop fechado, sem expr-IR.)
2. src/lib.rs:
   - pub const MANIFEST: NodeManifest com:
       id      = NodeTypeId::of("shader.blur")
       inputs  = [PortSpec{name:"in",  ty: Field+Vec4+Frame (RGBA)}]
       outputs = [PortSpec{name:"out", ty: Field+Vec4+Frame (RGBA)}]
       effect  = Effect::Temporal
       params  = [ParamSpec{name:"radius", default: 1.0_f32},
                  ParamSpec{name:"kernel", default: 0.0_f32}]
       (ParamSpec stores f32 só; enums se encodam como float: kernel
        0.0=Gaussian, 1.0=Box, e o eval faz a rounding/match. Para cap
        de alocação use param_as_count(ctx.param("radius"), 16) no eval.)
       lowerings = [LoweringKind::Cpu]   // Wgsl é fan-out futuro
   - struct ShaderBlur; impl NodeOp { manifest(); eval(ctx) }
     eval lê ctx.param("radius") (cape com param_as_count(v, 16) se
     usar pra alocação) + ctx.param("kernel"), aplica blur por-pixel
     no stream de entrada, ctx.emit do stream resultado.
   - pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError>
3. Teste golden:
   - constrói grafo: const_rgba → blur(radius=2, kernel=Box) → sink
   - register, g.validate(&ops), cook
   - asserta saída tem mesmas dim e valores conhecidos (3x3 com
     centro=1 → blur box r=1 → centro=1/9).

O QUE VOCÊ NÃO TOCA:
- 🔒 ph2d-nodegraph (contrato congelado em ADR-0039), ph2d-expr,
  ph2d-node-registry, ph2d-node-registry-init/ (gerado), Cargo.toml raiz.

WIRING (sem colisão):
  cargo run -p ph2d-node-sync       # regenera register_all_nodes + deps
  cargo test -p ph2d-node-registry-init   # staleness gate fecha

VALIDAÇÃO (codificação rápida):
  cargo check -p ph2d-node-shader-blur
  cargo test  -p ph2d-node-shader-blur
  cargo clippy -p ph2d-node-shader-blur --all-targets -- -D warnings
  cargo fmt -p ph2d-node-shader-blur

NOMES (gates ativos):
  type name canônico = "shader.blur" (único cross-crate; colisão pega no
  boot por RegistryError::Collision). Param names: identificadores
  simples ("radius", "kernel") sem ponto/espaço.

SE PRECISAR DE ALGO FORA DA PASTA (port-type novo, variant em Effect,
domain novo): PARE e reporte ao Enio — não é fan-out puro, é mudança
em contrato congelado (Coordenador-only + ADR amendment).

QUANDO TERMINAR, reporte:
  "Node shader-blur pronto. Commit local: <sha>. cargo test
   -p ph2d-node-shader-blur e -p ph2d-node-registry-init verdes."
═══════════════════════════════════════════════════════════════════
```

### Arquivos a criar

#### `crates/ph2d-node-shader-blur/Cargo.toml`

```toml
[package]
name = "ph2d-node-shader-blur"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true
license.workspace = true
authors.workspace = true

[lib]

[dependencies]
ph2d-nodegraph = { path = "../ph2d-nodegraph" }
ph2d-node-registry = { path = "../ph2d-node-registry" }
```

#### `crates/ph2d-node-shader-blur/src/lib.rs` (esqueleto — preencher `eval`)

```rust
#![forbid(unsafe_code)]
//! `shader.blur` — Temporal node, blurs an RGBA Frame stream by `radius`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("shader.blur"),
    name: "shader.blur",
    inputs: &[PortSpec {
        name: "in",
        ty: PortType::new(Domain::Field, Dim::Vec4, Clock::Frame),
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: PortType::new(Domain::Field, Dim::Vec4, Clock::Frame),
    }],
    effect: Effect::Temporal,
    clock: Clock::Frame,
    // ParamSpec stores `f32` only (autoria-side). Discrete choices —
    // here `kernel` (0.0 = Gaussian, 1.0 = Box) — encode as floats and
    // the `eval` does the rounding/match. The radius is naturally
    // float; cape it at `eval` time via `param_as_count(v, 16)` so an
    // untrusted override can't blow up allocation.
    params: &[
        ParamSpec {
            name: "radius",
            default: 1.0,
        },
        ParamSpec {
            name: "kernel",
            default: 0.0, // 0.0 = Gaussian, 1.0 = Box
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct ShaderBlur;

impl NodeOp for ShaderBlur {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Lê o stream de entrada, aplica blur por-pixel, emite o saído.
        // (Corpo elidido — siga o padrão de motion.transform; cape
        //  alocação via param_as_count se o blur usa buffer auxiliar.)
        let radius: f32 = ctx.param("radius");
        let kernel: f32 = ctx.param("kernel"); // 0.0=Gaussian, 1.0=Box
        let _ = (radius, kernel);
        // ctx.emit(Stream::new(width * height).with("rgba", Column::Vec4(out)));
        let _ = ctx;
    }
}

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ShaderBlur))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    // Golden test concreto: const_rgba(3x3, centro=1) → blur(r=1, Box)
    // → centro deve virar 1/9. Implemente seguindo o padrão de
    // crates/ph2d-node-debug-wave/src/lib.rs::tests.
}
```

### Sync + validação

```bash
cargo run  -p ph2d-node-sync               # regenera wiring
cargo test -p ph2d-node-shader-blur        # crate-local
cargo test -p ph2d-node-registry-init      # staleness + collision gates
```

Saída esperada do sync:

```
ph2d-node-sync: <N+1> node crate(s) (incl. shader-blur).
```

---

## Example B — tool sabor (1) one-shot: `ph2d-tool-grayscale`

Hipotético: pill no chrome dispara conversão luminância da bitmap do sprite
ativo. Sem panel, sem behavior (`make`) — sabor (1) puro.

### Briefing (cole direto na sessão)

```
═══════════════════════════════════════════════════════════════════
BRIEFING — tool-crate · slug: grayscale · sabor: (1) one-shot
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA: crates/ph2d-tool-grayscale/

ANTES DE CODAR: leia o mapa node↔tool em DIRETRIZ §3.8.1 e a tabela
de sabores §3.8.3 — você é sabor (1), template = ph2d-tool-make-square/.

O QUE VOCÊ FAZ (só dentro da sua pasta):
0. PRIMEIRO arquivo: src/lib.rs (1 linha qualquer destrava o workspace
   para sessões paralelas).
1. Cargo.toml: deps = ph2d-tool-registry, ph2d-a11y, ph2d-core,
   ph2d-color (para SrgbRgba — entrada/saída tipada), ph2d-vector
   (para o BezPath do ícone).
2. src/algorithm.rs:
   - pub fn grayscale(pixels: &[SrgbRgba]) -> Vec<SrgbRgba>
   - Luma per-pixel (Y = 0.2126R + 0.7152G + 0.0722B); preserva alpha.
   - Testes: 1 pixel branco → cinza claro; vermelho puro → 0x36; alpha
     preservado.
3. src/icon.rs:
   - pub fn grayscale_bezpath() -> BezPath
   - Porte do SVG docs/design/icons/grayscale.svg (24×24, Lucide-style,
     stroke="currentColor"). Se não tem SVG source, peça ao Enio antes
     de continuar — sem o asset, o IconId variant não pode entrar em
     ordem alfabética e o gate enum_order_matches_svgs falha.
4. src/manifest.rs:
   - pub const MANIFEST: ToolManifest com id="grayscale",
     label_key="tool.grayscale.label", cluster="image_tools",
     zone=Zone::TopRight, order=<próximo livre — vide
     docs/design/tools/*.toml para os existentes>,
     a11y_role=Role::Button, handler=OneShot{on_click: shadow_handler},
     touches_sim=false, memory_budget=MemoryBudget::new(0,0,0).
5. src/lib.rs:
   pub mod algorithm; pub mod icon; pub mod manifest;
   pub use algorithm::grayscale;
   pub use manifest::MANIFEST;
   pub fn register(reg: &mut Registry) { reg.register(&MANIFEST); }
   #[test] register_attaches_manifest_to_registry.
6. docs/design/tools/grayscale.toml:
   [tool] id="grayscale" cluster="image_tools" zone="top_right"
          order=<mesmo> a11y_role="Button" icon_slug="grayscale"
          touches_sim=false
   [label] fluent_key="tool.grayscale.label"
   [memory_budget] vram_mb=0 ram_mb=0 heap_script_mb=0
   (PRECISA existir — gate every_registered_manifest_has_matching_design_toml.)
7. ph2d-editor-core/src/icons.rs: adicione IconId::Grayscale em
   ORDEM ALFABÉTICA. NÃO pule via --no-verify — quebra TODOS os ícones
   (o gate enum_order_matches_svgs ordena por ordinal e bate com o
   índice de SVGs).
   Avise o Coord-A antes de fazer (pode haver outro agente paralelo
   adicionando IconId — sincroniza pra evitar conflito de ordinal).

   ↑ ESSE arquivo + a SVG e o TOML acima são as ÚNICAS 3 edições fora
   da sua pasta que o sabor (1) exige. Nenhuma delas é em código
   foundational congelado — são índices alfabéticos / assets.

O QUE VOCÊ NÃO TOCA:
- 🔒 crates/ph2d-editor-core/src/tool.rs (Tool/RasterEditTool/
  PanelEvent — contrato congelado ADR-0040+0041).
- crates/ph2d-editor-core/src/action_bus.rs::EditorAction (use os 4
  genéricos: ActivateTool/OneShotImageOp/ToolPanelEvent/CancelActiveTool;
  NÃO crie variant per-tool).
- ph2d-tool-registry, ph2d-tool-registry-init/ (gerado),
  Cargo.toml raiz.

WIRING (sem colisão):
  cargo run -p ph2d-tool-sync             # regenera 5 superfícies
                                          # (incl. image_tools order
                                          #  list + icon_slug match arms
                                          #  derivados do design TOML)
  cargo test -p ph2d-tool-registry-init   # 6 staleness gates fecham

VALIDAÇÃO:
  cargo check -p ph2d-tool-grayscale
  cargo test  -p ph2d-tool-grayscale
  cargo clippy -p ph2d-tool-grayscale --all-targets -- -D warnings

QUANDO TERMINAR, reporte:
  "Tool grayscale (sabor 1) pronto. Commit local: <sha>. cargo test
   -p ph2d-tool-grayscale e -p ph2d-tool-registry-init verdes."
═══════════════════════════════════════════════════════════════════
```

### Arquivos a criar (estrutura mínima)

```
crates/ph2d-tool-grayscale/
├── Cargo.toml
└── src/
    ├── lib.rs        # mod + register fn (~30 LOC)
    ├── algorithm.rs  # grayscale fn + testes (~50 LOC)
    ├── icon.rs       # BezPath do ícone (~40 LOC)
    └── manifest.rs   # ToolManifest const + testes (~50 LOC)

docs/design/tools/grayscale.toml   # 12 linhas — fonte canônica do design

docs/design/icons/grayscale.svg    # 24×24 Lucide-style (Enio fornece)

crates/ph2d-editor-core/src/icons.rs   # +1 variant IconId::Grayscale
                                       # em ordem alfabética
```

#### `crates/ph2d-tool-grayscale/Cargo.toml`

```toml
[package]
name = "ph2d-tool-grayscale"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true
license.workspace = true
authors.workspace = true

[lib]

[dependencies]
ph2d-a11y           = { path = "../ph2d-a11y" }
ph2d-color          = { path = "../ph2d-color" }
ph2d-core           = { path = "../ph2d-core" }
ph2d-tool-registry  = { path = "../ph2d-tool-registry" }
ph2d-vector         = { path = "../ph2d-vector" }
```

#### `crates/ph2d-tool-grayscale/src/manifest.rs`

```rust
//! Grayscale — ToolManifest declaration (sabor 1 one-shot).

use ph2d_a11y::Role;
use ph2d_core::MemoryBudget;
use ph2d_tool_registry::{HandlerFn, McpExposure, ToolHandler, ToolManifest, Zone};

use crate::icon::grayscale_bezpath;

fn shadow_handler() {} // shell drena via EditorAction::OneShotImageOp

pub const MANIFEST: ToolManifest = ToolManifest {
    id: "grayscale",
    label_key: "tool.grayscale.label",
    icon_fn: grayscale_bezpath,
    zone: Zone::TopRight,
    cluster: "image_tools",
    order: 130, // próximo livre após upscale=120. Confira com
                //   grep -h "^order" docs/design/tools/*.toml | sort -n
                // antes de assumir (outro agente paralelo pode ter
                // reservado 130 enquanto você editava).
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot {
        on_click: shadow_handler as HandlerFn,
    },
    memory_budget: MemoryBudget::new(0, 0, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_id_matches_label_key_slug() {
        assert_eq!(MANIFEST.label_key, "tool.grayscale.label");
    }

    #[test]
    fn manifest_lives_in_image_tools_cluster() {
        assert_eq!(MANIFEST.cluster, "image_tools");
        assert_eq!(MANIFEST.zone, Zone::TopRight);
    }
}
```

#### `crates/ph2d-tool-grayscale/src/lib.rs`

```rust
#![forbid(unsafe_code)]
//! ph2d-tool-grayscale — Grayscale one-shot image action.
//!
//! Per-pixel luma (Y = 0.2126R + 0.7152G + 0.0722B); alpha preserved.

pub mod algorithm;
pub mod icon;
pub mod manifest;

pub use algorithm::grayscale;
pub use icon::grayscale_bezpath;
pub use manifest::MANIFEST;

pub fn register(reg: &mut ph2d_tool_registry::Registry) {
    reg.register(&MANIFEST);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_attaches_manifest_to_registry() {
        let mut reg = ph2d_tool_registry::Registry::default();
        register(&mut reg);
        reg.build().expect("registry should build with grayscale");
        let found = reg.by_id("grayscale").expect("registered by id");
        assert_eq!(found.id, "grayscale");
    }
}
```

#### `crates/ph2d-tool-grayscale/src/algorithm.rs`

```rust
//! Per-pixel luma conversion: Y = 0.2126R + 0.7152G + 0.0722B; alpha preserved.

use ph2d_color::SrgbRgba;

pub fn grayscale(pixels: &[SrgbRgba]) -> Vec<SrgbRgba> {
    pixels
        .iter()
        .map(|p| {
            let [r, g, b, a] = p.0;
            let y = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u8;
            SrgbRgba([y, y, y, a])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_red_becomes_dark_gray() {
        let out = grayscale(&[SrgbRgba([255, 0, 0, 255])]);
        // 0.2126 * 255 ≈ 54 = 0x36.
        assert_eq!(out[0].0, [54, 54, 54, 255]);
    }

    #[test]
    fn preserves_alpha() {
        let out = grayscale(&[SrgbRgba([128, 128, 128, 200])]);
        assert_eq!(out[0].0[3], 200);
    }

    #[test]
    fn empty_input_is_total() {
        assert!(grayscale(&[]).is_empty());
    }
}
```

#### `crates/ph2d-tool-grayscale/src/icon.rs`

```rust
//! Grayscale icon — half-filled circle (Lucide-style placeholder).
//!
//! In a real fan-out, port `docs/design/icons/grayscale.svg` byte-for-byte
//! into the BezPath calls (this stub matches the manifest signature so the
//! contract holds; replace before merge).

use ph2d_vector::BezPath;

pub fn grayscale_bezpath() -> BezPath {
    // Placeholder: empty path. Replace with the SVG port — see
    // crates/ph2d-tool-make-square/src/icon.rs for a worked example
    // (BezPath::move_to + line_to + curve_to wiring).
    BezPath::new()
}
```

#### `docs/design/tools/grayscale.toml`

```toml
# Grayscale — one-shot luma conversion. Image Tools cluster.

[tool]
id          = "grayscale"
cluster     = "image_tools"
zone        = "top_right"
order       = 130
a11y_role   = "Button"
icon_slug   = "grayscale"
touches_sim = false

[label]
fluent_key = "tool.grayscale.label"

[memory_budget]
vram_mb        = 0
ram_mb         = 0
heap_script_mb = 0
```

### Sync + validação

```bash
cargo run  -p ph2d-tool-sync               # regenera 5 superfícies
cargo test -p ph2d-tool-grayscale          # crate-local (algorithm + manifest)
cargo test -p ph2d-tool-registry-init      # 6 staleness gates + 3 design-sync
```

Saída esperada do sync:

```
ph2d-tool-sync: <N+1> crate(s) total; <M+1> manifest, <K> modal (make);
                <P+1> design TOML(s) regenerated.
```

> **A partir de `b8495e7` (Friction-B, 2026-05-24):** o
> `cargo test -p ph2d-tool-registry-init` regenera automaticamente os 2
> testes antes mantidos à mão (`image_tools_cluster_in_canonical_order`
> + `expected_icon_slug`). Você não precisa estendê-los manualmente; o
> `ph2d-tool-sync` regenera a partir do `docs/design/tools/*.toml`.

---

## Resumo: o que muda entre os dois exemplos

| Aspecto | Node (`shader-blur`) | Tool (`grayscale` sabor 1) |
|---|---|---|
| Pasta | `crates/ph2d-node-shader-blur/` | `crates/ph2d-tool-grayscale/` |
| Sync | `cargo run -p ph2d-node-sync` | `cargo run -p ph2d-tool-sync` |
| Superfícies regen | 1 (register_all_nodes) | 5 (register_all + register_all_tools + 2 testes + deps) |
| Staleness gate | `-p ph2d-node-registry-init` | `-p ph2d-tool-registry-init` |
| Entry point | `pub fn register(reg) -> Result<…>` | `pub fn register(reg)` |
| Contrato congelado | NodeOp ≤ 2 / NodeManifest ≤ 8 (ADR-0039) | Tool ≤ 11 / RasterEditTool ≤ 5 / PanelEvent ≤ 4 (ADR-0040+0041) |
| Edição fora da pasta | Nenhuma | 3 touches: IconId variant em `editor-core/src/icons.rs` (ordem alfabética) + design TOML em `docs/design/tools/` + SVG em `docs/design/icons/` (Enio fornece) |
| Templates vivos | `ph2d-node-debug-const` / `-debug-wave` / `-motion-*` | `ph2d-tool-make-square` / `-move` / `-padding` |

---

## Quando este doc precisa de atualização

- Mudança em §3.8.2 (briefing parametrizado): reflita aqui.
- Cap arch-gate bumpado em ADR amendment: atualize a linha de contrato.
- Sabor novo de tool ou domínio novo de node: adicione um terceiro exemplo.
- Friction novo descoberto em fan-out: documente no exemplo afetado.
