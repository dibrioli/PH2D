# Solving Parallel-Agent Collisions in the PH2D Editor

Narrativa completa do problema → diagnóstico → solução. Para quem
chega novo ao projeto: leia ISTO primeiro. Depois aprofunde nas
ADRs (links no fim).

**Última atualização:** 2026-05-16 — Wave 2 + parte de Wave 2.5
mergeadas em `origin/main`. Wave 2.5 ActionBus migrations em
progresso (2 de 20 fields migrados, 18 restantes documentados em
[Wave 3 plan](2026-05-wave-3-deferred-state-decomp-and-golden-images.md)).

---

## TL;DR

> **Problema:** múltiplas LLM operando em paralelo no mesmo
> codebase colidiam silenciosamente em ~10 arquivos centrais
> (icons.rs, lib.rs, fixture.rs, ids.rs, color.rs, ...). Cada nova
> tool ou widget exigia editar 4-6 arquivos compartilhados.
>
> **Solução:** *convention-by-discovery + codegen-from-design +
> lint-as-spec*. Cada tool vira um crate isolado com `MANIFEST`
> const; o engine descobre tools no boot. Designer edita
> `docs/design/{tokens.json, icons/*.svg, tools/*.toml}`; `build.rs`
> gera Rust. CI rejeita violações via 5 architecture tests + HR-18
> cap.
>
> **Resultado:** adicionar uma tool nova = 1 crate novo + 1 linha
> em `register_all`. Zero edit em arquivos centrais. ~1000 LOC de
> código manual ↔ design eliminados. 6 silent NodeId collisions
> caçados em uma só PR.

---

## 1. O problema concreto

Em 2026-05-15, uma sessão multi-agente ficou catastroficamente
lenta: 3 LLMs paralelas tentando adicionar features (grid-snap,
bg-removal, make-square) precisavam **todas** editar:

| Arquivo | Por quê |
|---------|---------|
| `crates/ph2d-editor/src/icons.rs` | Adicionar variant ao `IconId` enum + 715 LOC de match-arms manuais lendo cmds SVG |
| `crates/ph2d-editor/src/lib.rs` | `pub use widget::Button` style — 84 re-exports criando zona de merge a cada commit |
| `crates/ph2d-editor/src/tools/mod.rs` | Registrar o mod novo |
| `crates/ph2d-editor/src/screens/hero/fixture.rs` | `topbar_clusters()` hard-coded |
| `crates/ph2d-editor/src/screens/hero/ids.rs` | Alocar manualmente um `NodeId(N)` num range disponível |
| `crates/ph2d-tokens/src/color.rs` | Hex hardcoded → re-typar 4-temas OKLCH |
| `Cargo.toml` workspace | Adicionar member |
| `shells/desktop/src/main.rs` | 20 `pending_X` drains inline |

**Colisões silenciosas observadas no audit:**

- **6 NodeId duplicados** em `ids.rs` (`INSP_TRANSFORM_RESET` e
  `INSP_BLENDER_PICKER` ambos em `NodeId(380)`;
  `INSP_NOTE_BODY_3` e `CTX_MENU_SETTINGS_UNIT` ambos em
  `NodeId(853)`; e 4 outros). Cada um era um bug latente: clicar
  o widget A acionaria também a lógica do B.
- **715 LOC de SVG-portado-manualmente** em `icons.rs::cmds()`.
  Cada novo ícone exigia o autor ler o SVG e re-escrever em
  `IconCmd` literals à mão.
- **`tokens.json` divergia** dos valores hard-coded em
  `color.rs`. Designer editava JSON sem efeito; Rust ignorava.
- **Sem source-of-truth canônica** para tool functionality —
  `topbar.rs` + `fixture.rs` + `icons.rs` + `color.rs` precisavam
  ser editados em sync à mão.

A pasta de cada agente colidia inevitavelmente com a do outro. O
merge custava ~30min de rebase por colisão, sem garantia de não
re-introduzir o NodeId duplicado.

---

## 2. Diagnóstico (audit em 2026-05-16)

Cruzamento de duas opiniões de LLM (Agente A + Agente B) +
auditoria do Coordenador → **10 pontos de colisão identificados**:

| # | Sintoma | Causa-raiz |
|---|---------|------------|
| 1 | tokens.json ↔ color.rs drift | Sem codegen |
| 2 | icons.rs grande + edit central | Sem codegen SVG |
| 3 | NodeId silent collisions | Allocação manual em ranges |
| 4 | fixture::topbar_clusters() hardcoded | Chrome não derivado do Registry |
| 5 | Sem TOML canônico para tools | Drift TOML ↔ Rust inevitável |
| 6 | Sem lint anti-`0xRRGGBB` | Hex regrediam silenciosamente |
| 7 | HR-18 declarada mas inativa | Sem CI gate |
| 8 | grid_snap/panel.rs 2869 LOC | Multi-agente colidia em 4119 LOC do grid_snap subsistema |
| 9 | screens/hero.rs 3300 LOC + 50 fields | Toda interação tocava god-struct |
| 10 | main.rs 2416 LOC com 20 pending_X drains | Action Bus pendente |

A LLM precisava **subir um nível**: deixar de ser refactor de
arquivo e virar refactor de **arquitetura de discovery**.

---

## 3. A estratégia: três pilares

### 3.1 *Convention-by-discovery* — uma tool = um crate

Em vez de "uma tool = mods espalhados em 4 arquivos", cada tool
vira um crate isolado em `crates/ph2d-tool-<slug>/`. Ele exporta:

```rust
pub const MANIFEST: ToolManifest = ToolManifest {
    id: "trim_transparency",
    label_key: "tool.trim_transparency.label",
    icon_fn: icon,
    zone: Zone::TopRight,
    cluster: "image_tools",
    order: 40,
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot { on_click: shadow_handler },
    memory_budget: MemoryBudget::new(0, 0, 0),
    touches_sim: false,
    mcp: McpExposure::reserved(),
};

pub fn register(reg: &mut Registry) {
    reg.register(&MANIFEST);
}
```

Adicionar a tool ao engine = **uma única linha** em
`crates/ph2d-tool-registry-init/src/lib.rs::register_all`:

```rust
ph2d_tool_trim_transparency::register(reg);
```

O `register_all` é a **única zona de merge possível** — e ela é
append-only, então conflitos de merge são triviais (resolver = pick
both).

### 3.2 *Codegen-from-design* — `docs/design/` é canonical

Designer (humano ou Claude Design) edita:

- `docs/design/tokens.json` — palette + spacing + radii + typography
- `docs/design/icons/<slug>.svg` — Lucide-derived 24×24 SVGs
- `docs/design/tools/<slug>.toml` — functional spec de cada tool

`build.rs` em `ph2d-tokens` + `ph2d-editor` lê esses arquivos e
**gera Rust no `OUT_DIR`**:

- `crates/ph2d-tokens/build.rs` → `tokens_generated.rs` com 4
  themes × 33 OKLCH colors resolvidos
- `crates/ph2d-editor/build.rs` → `icons_generated.rs` com
  per-icon `IconCmd` arrays + `ICON_CMDS_BY_ID` + `lookup_cmds` +
  `ALL_ICON_SLUGS`

`cargo:rerun-if-changed=` garante rebuild automático quando
designer edita.

Resultado: **715 LOC de cmds manuais eliminados**. **Drift entre
JSON e Rust matematicamente impossível.**

### 3.3 *Lint-as-spec* — arquitetura testada em CI

Cinco architecture tests rodam em CI:

| Test | Garante |
|------|---------|
| `tests/node_id_collisions.rs` | NodeId chrome consts são pairwise únicos + nenhum colide com NodeId::ROOT ou com bits de eye/expand companion |
| `tests/chrome_manifest_coverage.rs` | Cada chrome const NodeId hash-bate com o `manifest.id` da tool correspondente |
| `tests/tool_manifest_design_sync.rs` | Cada `docs/design/tools/<slug>.toml` ↔ MANIFEST const field-by-field |
| `tests/no_literal_color.rs` | Zero `0xRRGGBB` em `widget/**` ou `screens/**` (allowlist `// LITERAL-COLOR-OK: <reason>`) |
| `shells/desktop/tests/file_loc_caps.rs` | `shells/<plat>/src/*.rs` ≤ 600 LOC (HR-18; exceções via `// ph2d-loc-cap: <reason>` nos primeiros 20 linhas) |

CI verde = especificação respeitada. Regressão silenciosa fica
impossível.

---

## 4. Execução em 3 Waves

### Wave 1 — Foundation (mergeada origin/main 2026-05-16 manhã, sha bom `a5343f9`)

- Criação de `ph2d-tool-registry` crate (Registry + ToolManifest +
  hash NodeId FNV-1a + IconHandle + ActionInvocation).
- `ph2d-tool-registry-init` crate (`register_all` append-only).
- 4 piloto tool-crates: `make-square` (Action one-shot completo),
  `grid-snap` + `bgremoval` (manifest-thin; conteúdo continua em
  `ph2d-editor/src/`).
- Shell decomposition: `main.rs` 3463 → 2421 LOC (extracted
  `init.rs`, `forwarding.rs`, `hero_intents.rs`).
- **ADR-0027 Accepted, SKILL 2.4. HR-18 declarada (inativa).**
- CI matrix verde 10/10 jobs.

Detalhes em
[`docs/Migracao/2026-05-convention-by-discovery.md`](2026-05-convention-by-discovery.md).

### Wave 2 — Codegen + Registry chrome + Lints (12 commits, mergeada 2026-05-16 noite, sha bom `6336e89`)

| PR | O que entregou |
|----|----------------|
| 11.1.0 | tokens.json reset to Round 9 canonical (sincronizou JSON com valores Rust) |
| 11.1 | `build.rs` tokens codegen — 4 themes resolvidos |
| 11.2 | `build.rs` SVG codegen — 715 LOC manuais removidos; 11 SVGs perdidos recuperados via git |
| 11.3 | NodeId hash universal — 250 consts migradas; **6 colisões silenciosas pré-Wave-2 eliminadas**; companion-bit guard contra hash mis-detection |
| 11.4 | Chrome derived from Registry — `image_action_pills` consome `Registry::cluster("image_tools")`; novo `ph2d-tool-trim-transparency` crate; `paint_icon_path` helper |
| 11.5 | 4 design canonical TOMLs + `tool_manifest_design_sync` test (3 cases) |
| 11.6 | `no_literal_color` lint (+ Windows path-sep fix em PR 11.6 follow-up) |
| 11.7c | `topbar.rs` 727 → 482 LOC split |
| 11.7b | `hierarchy.rs` 998 → 414 LOC split |
| 11.9 | **HR-18 cap ATIVO** em `shells/desktop/tests/file_loc_caps.rs` com 2 exceções declaradas |
| 11.12 | ADR-0028 + SKILL 2.5 + Wave 2.5 plan stub |

CI matrix verde 10/10 jobs em todos. **Workspace ~1296 tests pass.**

### Wave 2.5 — File hygiene + ActionBus (em progresso 2026-05-16 noite)

| PR | Status | O que entregou |
|----|--------|----------------|
| 11.7a (`3f42972`) | ✅ mergeada | `grid_snap/panel.rs` 2869 LOC → 7 sibling files |
| 11.11 (`863a2ca`) | ✅ mergeada | `lib.rs` trim — 84 widget/tools/image_edit re-exports removidos |
| 11.8 foundation (`8cbfe4e`) | ✅ mergeada | `action_bus.rs` — `EditorAction` enum + `ActionBus` queue + 7 tests |
| 11.8b1 (`ca429ec`) | ✅ mergeada | `pending_trim_transparency` → `EditorAction::Trim` (1 of 20) |
| 11.8b2 (`017a5cf`) | ✅ mergeada CI 10/10 | `pending_make_square` → `EditorAction::MakeSquare` (2 of 20) |
| 11.8b3/b4 | ⏳ pendente | bgremoval / reimport / undo_image_edit / activate_bgremoval |
| 11.8c | ⏳ pendente | 9 hierarchy intents |
| 11.8d | ⏳ pendente | 5 inspector intents |
| 11.8 closeout | ⏳ pendente | Remove os 2 `// ph2d-loc-cap:` markers de main.rs + hero_intents.rs |

Após o closeout, HR-18 fica clean (inventory test emite "NONE").

Detalhes em
[`docs/Migracao/2026-05-wave-2-5-deferred-splits.md`](2026-05-wave-2-5-deferred-splits.md).

### Wave 3 — Pending (deferidos por baixo valor/alto custo)

| PR | Recomendação |
|----|--------------|
| 11.7d — HeroScreen state decomp | Defer. hero.rs não está sob HR-18 (cap só vale para `shells/`). Decomp é puro architectural hygiene — 138 call sites, 3-4h, zero unlock funcional. Só executar se conflito multi-agente concreto em hero.rs surgir. |
| 11.10 — Golden image tests (Vello headless) | Defer. Validação visual sintética. Manter smoke manual via `play.command` é cost-effective enquanto designer não contribui PRs de UI diretamente. |

Detalhes em
[`docs/Migracao/2026-05-wave-3-deferred-state-decomp-and-golden-images.md`](2026-05-wave-3-deferred-state-decomp-and-golden-images.md).

---

## 5. Estado de hoje — como adicionar uma tool nova

```text
1. Designer (humano ou Claude Design) cria docs/design/tools/<slug>.toml
   com a spec funcional:

       [tool]
       id          = "my_tool"
       cluster     = "image_tools"
       zone        = "top_right"
       order       = 70
       a11y_role   = "Button"
       icon_slug   = "my-icon"
       touches_sim = false

       [label]
       fluent_key = "tool.my_tool.label"
       pt_br_inline = "My Tool"
       en_us_inline = "My Tool"

       [memory_budget]
       vram_mb = 0
       ram_mb = 0
       heap_script_mb = 0

2. Designer dropa docs/design/icons/my-icon.svg (Lucide-derived,
   24×24, currentColor).

3. Dev cria crates/ph2d-tool-my-tool/ replicando o TOML como
   ToolManifest const. Quatro arquivos:

       Cargo.toml      — package + 4 deps (registry / vector / a11y / core)
       src/lib.rs      — pub const MANIFEST + pub fn register
       src/icon.rs     — BezPath glyph (~30 LOC)
       (algorithm.rs   — opcional, se a tool tiver lógica pura)

4. Dev adiciona UMA linha em crates/ph2d-tool-registry-init/src/lib.rs:

       ph2d_tool_my_tool::register(reg);

5. CI roda 5 architecture tests:
   - tool_manifest_design_sync valida o TOML ↔ MANIFEST
   - chrome_manifest_coverage valida que se essa tool aparece na
     UI, o chrome NodeId hash-bate com manifest.id
   - node_id_collisions valida que nenhum NodeId colide
   - no_literal_color valida que widget/screens não regrediram
   - file_loc_caps valida que main.rs ainda está sob 600 LOC

6. Chrome aparece automaticamente. Painter consome
   Registry::cluster e renderiza o novo pill.
```

**Não toca:** `lib.rs`, `icons.rs`, `tools/mod.rs`,
`screens/hero/fixture.rs::topbar_clusters()`, `screens/hero/ids.rs`,
`Cargo.toml` raiz (além de adicionar o member).

Conflito multi-agente: zero. Cada tool vive em seu próprio crate;
register_all é append-only.

---

## 6. Hard rules cementadas

| HR | O quê | Onde |
|----|-------|------|
| HR-3 | Zero-alloc no dispatcher hot path | `interaction_dispatch_no_alloc` test |
| HR-5 | Determinism (cross-platform replay hash) | CI replay-hash matrix |
| HR-12 | A11y (`Role::Button` etc. em todo widget) | `hr12_widgets_a11y` test |
| HR-13 | Memory budget (HR-13 boot check) | Manifest `memory_budget` field |
| HR-15 | Zero hardcoded UI string (Fluent stub OK) | `hr15_no_hardcoded_ui_strings` + `no_literal_color` tests |
| HR-18 | shells/`.rs` ≤ 600 LOC | `file_loc_caps` test em `shells/desktop/tests/` |

---

## 7. Métricas

| Métrica | Pré-Wave-1 | Pós-Wave-2 | Pós-Wave-2.5 |
|---------|-----------|-----------|-------------|
| Manual NodeId consts em ids.rs | 253 | 0 (hash) | 0 |
| Silent NodeId collisions | 6 | 0 | 0 |
| Manual icon match-arms (icons.rs) | 715 LOC | 0 (codegen) | 0 |
| Manual color resolve fns | 200 LOC | 0 (codegen) | 0 |
| Sources of truth para tools | 4 fragmented | 1 canonical | 1 canonical |
| Architecture tests | 1 | 6 | 6 |
| Files em shells > 600 LOC | 2 ungated | 2 com exceções declaradas | 2 (closeout em 11.8c+d remove) |
| Widget re-exports em lib.rs | 84 | 84 | 0 |
| god-files crates/ acima de 600 | 5 | 3 (todos não-shell) | 2 (panel.rs e state.rs splittados) |
| pending_X scattered fields | 20 | 20 | 18 (e descendo a cada PR 11.8b) |
| Workspace tests pass | 1098 | 1296 | ~1300 |
| Commits batched para CI única | — | 11 (Wave 2) | 5+ (Wave 2.5) |

---

## 8. Lições aprendidas

1. **Multi-agente paralelo é diferente de multi-developer
   paralelo.** Humans tolerate merge churn; LLMs perdem contexto
   na resolução. Engineering pré-multi-agent precisa **eliminar**
   colisão arquitetonicamente, não gerenciá-la.

2. **Source-of-truth fragmentada é dívida invisível.** Cada novo
   developer (LLM ou humano) acreditava que estava editando "o"
   canonical, sem saber que os outros 3 arquivos também
   precisavam mudar. Codegen + cross-validation test elimina o
   fragmento.

3. **Lint-as-spec > documentação prosaica.** "Não use hex em
   widget/" como regra prosaica regrediu em 3 PRs. Como
   `no_literal_color.rs` test, falha CI no momento da regressão.

4. **HR-18 com exceções declaradas > HR-18 desligada.** Ativar o
   cap com 2 exceções `// ph2d-loc-cap:` documenta a dívida e
   surfaces-a em CI inventory. Desligar até "todos os arquivos
   couberam" deixa o cap eternamente inativo.

5. **Hash-derived NodeId tem caveat.** FNV-1a output cobre 64
   bits uniformemente; existem bit-flag patterns no codebase
   (`EYE_TOGGLE_BIT`, `EXPAND_TOGGLE_BIT`) que assumem high bits
   livres. Companion-bit threshold (`COMPANION_ROW_ID_MAX = 2^32`)
   resolve. Documentado em
   `crates/ph2d-editor/tests/node_id_collisions.rs`.

6. **Batching CI runs > Per-PR CI runs.** Para Wave-style
   migrations, commit local + push único + babysit CI (uma vez)
   é 10× mais barato que rodar CI 12× (~30min cada). Feedback
   accumulated em memory:
   `~/.claude/projects/.../feedback_ci_batching.md`.

---

## 8a. O problema específico do UI/hero

A solução das seções 1-7 resolveu a colisão **em tools/widgets
novos** (adicionar é 1 crate + 1 linha). Mas existe um **segundo
foco de colisão** que merece tratamento próprio: a god-struct
`HeroScreen` em `crates/ph2d-editor/src/screens/hero.rs`.

### 8a.1 Por que `hero.rs` é problema

`HeroScreen` é a struct raiz da tela de edição. Hoje (pós Wave 2.5
em progresso, sha `017a5cf`) tem **48 campos públicos** em uma só
struct + **3300 LOC** no arquivo:

```rust
pub struct HeroScreen {
    pub id: NodeId,
    pub theme: Theme,
    pub selection: Option<HeroSelection>,
    pub store: WidgetStore,
    pub hit_index: HitIndex,
    pub bus: ActionBus,            // ← Wave 2.5 PR 11.8 foundation

    // 6 view fields:  ui_mirrored, inspector_visible, hierarchy_visible,
    //                 stats_visible, widget_gallery_visible, grid_visible
    // 7 inspector fields:  inspector_sprite, inspector_transform, inspector_visibility,
    //                      inspector_name, last_inspector_entity, ...
    // 3 hierarchy fields:  live_hierarchy_entries, rename_target_row, ...
    // 4 image-edit fields: image_tools_mode, has_undoable_image_edit, ...
    // 3 gizmo fields:      gizmo_selection, gizmo_view, gizmo_drag
    // 18 pending_X fields: pending_visibility_toggle, pending_reparent, ...
    //                      (já em migração via ActionBus — 2 de 20 movidos)
    // ... mais 5 fields:    grid_view, grid_config, grid_snap_state, project, stats, ...
}
```

**Conflitos multi-agente concretos observados no UI/hero:**

| Quem | Quer tocar |
|------|-----------|
| Agente A adicionando feature de Hierarchy | `pending_duplicate`, `pending_delete`, `rename_target_row`, ... |
| Agente B adicionando feature de Inspector | `pending_transform_edit`, `pending_visibility_edit`, `inspector_sprite`, ... |
| Agente C ajustando Image Tools | `image_tools_mode`, `pending_trim_transparency`, `pending_make_square`, ... |
| Agente D refinando Gizmo | `gizmo_selection`, `gizmo_view`, `gizmo_drag`, ... |

Todos editam a **mesma struct definition**, todos editam o **mesmo
`Default::default()`**, todos editam o **mesmo `apply_event`** com
~30 match arms aninhados. Conflito de merge inevitável.

### 8a.2 Por que NÃO foi resolvido em Wave 2

Wave 2 atacou os arquivos que **bloqueavam adicionar tool nova**:
icons.rs, lib.rs, fixture.rs, ids.rs. Foram os arquivos mais
visíveis no audit.

`hero.rs` é diferente: ele bloqueia **modificar a UI existente**,
não adicionar tool. E **hero.rs não está sob HR-18** (cap só vale
para `shells/<plat>/src/`, vide §HR-18 do SKILL). Logo o CI nunca
falhou por hero.rs estar grande. A pressão para arrumá-lo é
puramente arquitetural.

Wave 2.5 ataca **parcialmente**: a ActionBus (PR 11.8 foundation
+ migrations b1/b2/b3/b4/c/d) elimina os 20 `pending_X` fields —
o maior subgroup. Quando todos os 20 estiverem migrados, a struct
perde **40%** dos seus campos.

Mas restam ~28 campos não-`pending_*` que continuam scatter. Isso
é o que **PR 11.7d** atacaria (e foi deferido).

### 8a.3 As soluções definitivas (a aplicar quando custo justificar)

Três soluções complementares. Quando todas executadas, a colisão
multi-agente no hero.rs vira **estruturalmente impossível**.

#### Solução 1 — ActionBus completa (Wave 2.5 PR 11.8b3..d + closeout)

**Status:** em progresso. 2 de 20 fields migrados.

**Resultado quando completa:**

- Os 20 `pending_X: Option<T>` somem do `HeroScreen` struct.
- `apply_event` deixa de ter "set pending field" branches —
  passa a fazer `self.bus.push(EditorAction::X)`.
- Shell deixa de ter 20 hand-written drain blocks — substitui
  por um `for action in hero.bus.drain() { match action { ... } }`.
- `main.rs` cai abaixo de 600 LOC.
- `hero_intents.rs` colapsa para ~50 LOC ou desaparece.
- HR-18 exceptions removidas (`// ph2d-loc-cap:` markers saem).

**Custo:** ~3-4h, mecânico, padrão pinned em PRs 11.8b1/b2.
Detalhes em
[hand-off prompt para outro agente](2026-05-wave-2-5-deferred-splits.md).

**Por que é definitivo:** elimina toda uma categoria de field —
*outbound intents* — substituindo por uma queue tipada que NÃO é
zona de merge (push é append-only, drain é match exhaustive).
Conflito multi-agente em intent novo = adicionar variant no enum
+ push site + drain arm. Cada um é local; zero zona compartilhada.

#### Solução 2 — HeroScreen state decomp (PR 11.7d, deferido)

**Status:** deferido. Honesta avaliação: zero unlock funcional,
~3-4h de churn em 138 call sites.

**Plano:** decompor `HeroScreen` em sub-structs cohesivos por
domínio:

```rust
pub struct HeroScreen {
    pub id: NodeId,
    pub theme: Theme,
    pub inspector: InspectorState,         // todos os inspector_X
    pub hierarchy: HierarchyState,         // todos os hierarchy_X + live_entries
    pub image_edit: ImageEditState,        // image_tools_mode, has_undoable_*
    pub view: ViewState,                   // ui_mirrored, *_visible
    pub gizmo: GizmoStateGroup,            // gizmo_selection / view / drag
    pub project: ProjectSettings,
    pub grid: GridStateGroup,              // grid_view, grid_config, grid_snap_state
    pub store: WidgetStore,
    pub hit_index: HitIndex,
    pub bus: ActionBus,
    pub selection: Option<HeroSelection>,
    pub stats: BottomHudStats,
    pub dragging_files: Option<(...)>,
    // ...meta state que não cabe em nenhum sub-struct
}
```

**Resultado:** cada sub-struct ganha `Default + apply_event`
próprio. `HeroScreen::apply_event` deixa de ser um mega-match —
delega para `self.inspector.apply_event` / `self.hierarchy.apply_event`
/ etc. Multi-agente trabalhando em inspector vs hierarchy NÃO
toca os mesmos arquivos.

**Por que é definitivo:** cada sub-struct vive em seu próprio
arquivo (`screens/hero/state/inspector.rs`,
`screens/hero/state/hierarchy.rs`, ...). Adicionar campo em
InspectorState = editar 1 arquivo. Zona de merge entre dois
agentes = zero (se eles trabalham em domínios diferentes).

**Custo:** ~3-4h mecânico, mas com ALTO risco de typo silencioso
(138 call sites de `hero.X` → `hero.<group>.X`). Smoke visual
obrigatório.

**Quando executar:** se um conflito multi-agente concreto em
hero.rs surgir entre agentes (até 2026-05-16 não aconteceu pós
Wave 2). Defer enquanto Wave 2.5 ActionBus completar — depois
disso, a contagem cai de 48 → 28 fields e talvez nem precise.

#### Solução 3 — Painel-as-canonical-source (futuro Wave 4+)

**Status:** não planejado ainda, mas é o "endgame" lógico.

A mesma receita que tornou *tools* discoverable (TOML + crate + 1
linha em register_all) pode tornar *painéis* discoverable:

```toml
# docs/design/panels/inspector.toml
[panel]
id        = "inspector"
zone      = "top_right"
default_visible = true
collapsible = true
sections  = ["transform", "render_strategy", "name", "samples"]

# docs/design/panels/hierarchy.toml
[panel]
id        = "hierarchy"
zone      = "top_left"
features  = ["search", "rename", "drag_reparent", "context_menu"]
```

Cada painel vira um crate `ph2d-panel-<slug>/` com:
- `pub const PANEL_MANIFEST: PanelManifest`
- `pub fn paint(...)` + `pub fn apply_event(...)`
- `pub fn populate(store: &mut WidgetStore)`
- Sub-state struct (vide solução 2)

O `paint_hero_screen` itera o `PanelRegistry` e pinta cada um.

**Resultado:** Inspector, Hierarchy, ImageTools, Gallery, GridSnap
viram **5 crates isolados**. `hero.rs` colapsa para ~200 LOC de
orquestração. Multi-agente trabalhando em painel novo (ex:
Animator) = drop um crate `ph2d-panel-animator/`, adicionar TOML,
adicionar 1 linha em `panel_register_all`. **Zero contato com
hero.rs.**

**Por que é definitivo:** simétrico ao tool-as-crate. Resolve
de raiz, não por palliative.

**Custo estimado:** 1-2 sessões dedicadas. Sem urgência — Wave 2.5
+ Wave 3 ActionBus já dão folga arquitetural significativa.

### 8a.4 Resumo: ordem recomendada de execução

| Quando | O quê | Custo | Bloqueia o quê |
|--------|-------|-------|-----------------|
| Já feito | Wave 1 + Wave 2 (códegen + Registry chrome + lints + HR-18) | — | Nada |
| Em progresso | Wave 2.5 PR 11.8b3/b4/c/d/closeout (ActionBus completa) | 3-4h | HR-18 fica clean; -20 fields no HeroScreen |
| Quando custo justificar | Wave 3 PR 11.7d (state decomp) | 3-4h | Multi-agente em domínios diferentes do hero |
| Endgame | Wave 4+ (painel-as-canonical-source) | 1-2 sessões | Multi-agente em painéis inteiros; hero.rs colapsa para 200 LOC |

A regra: **só executar a próxima solução quando a anterior parar
de ser suficiente**. Não over-engineering — cada step ganha
justificativa empírica.

---

## 9. Referências

- [ADR-0027 — Convention-by-discovery (Wave 1)](../architecture/decisions/0027-convention-by-discovery.md)
- [ADR-0028 — Wave 2 codegen + design canonical + lint guards + HR-18 ativo](../architecture/decisions/0028-wave-2-codegen-design-canonical.md)
- [SKILL_Stack_PH2D_Definitiva.md §HR-18](../../SKILL_Stack_PH2D_Definitiva.md#hr-18--crescimento-bounded-em-shell-binaries)
- [Wave 2 plan canonical](2026-05-wave-2-eliminating-all-collisions.md)
- [Wave 2.5 plan](2026-05-wave-2-5-deferred-splits.md)
- [Wave 3 plan (deferred)](2026-05-wave-3-deferred-state-decomp-and-golden-images.md)
- [STATE.md (Coordenador's operational state)](../IntegracaoMultiAgente/STATE.md)
- [Convention-by-discovery migration plan (Wave 1 source)](2026-05-convention-by-discovery.md)

## 10. Para quem vai operar daqui pra frente

- Leia esta narrativa.
- Leia ADR-0027 + ADR-0028.
- Leia SKILL §HR-1..HR-18 (toda a Hard Rules section).
- Memory persistente em
  `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`
  tem perfil do Enio, communication preferences, e estado
  acumulado entre sessões. **Leia antes de tomar ações.**
- CLAUDE.md tem workflow operacional + PRCI loop policy (CI
  babysit).
