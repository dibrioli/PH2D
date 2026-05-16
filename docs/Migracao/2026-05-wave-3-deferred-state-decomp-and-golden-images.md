# Wave 3 — HeroScreen state decomp + Action Bus migrations + golden images

**Status:** Planejado, aguardando sessão dedicada.

Origem: PRs deferidos da Wave 2.5 (vide
[`2026-05-wave-2-5-deferred-splits.md`](2026-05-wave-2-5-deferred-splits.md))
após audit honesta em 2026-05-16: o trabalho remanescente é grande
demais para uma sessão pós-Wave-2.5 já longa, e cada PR demanda
~2-4h de mecânica cuidadosa + smoke visual.

## Contexto pós Wave 2.5

Wave 2.5 entregou:

- PR 11.7a (`3f42972`) — grid_snap panel.rs split (2869 → 7 files).
- PR 11.11 (`863a2ca`) — lib.rs trim (84 re-exports removidos).
- PR 11.8 foundation (`8cbfe4e`) — `action_bus.rs`: EditorAction
  enum + ActionBus queue + 7 unit tests. Infra-only; consumers
  ainda em pending_X pattern.

Estado de débito conhecido (HR-18 exceptions ainda ativas):

- `shells/desktop/src/main.rs` — 2421 LOC.
- `shells/desktop/src/hero_intents.rs` — 696 LOC.

Ambos resolvem quando os ~20 `pending_X` migrarem para o
ActionBus (PR 11.8b/c/d).

## PRs Wave 3

### PR 11.7d — HeroScreen state decomp

**Status:** Postponed após audit honesta. O custo (~3-4h, 138 call
sites) excede o valor: hero.rs NÃO está sob HR-18 (cap só aplica a
`shells/<plat>/src/`), então o decomp é puro architectural hygiene
sem unlock funcional.

**Plano original (mantido para referência):**

Extrair sub-structs cohesivos:

```rust
pub struct HeroScreen {
    pub id: NodeId,
    pub theme: Theme,
    pub inspector: InspectorState,        // ~12 fields
    pub hierarchy: HierarchyState,        // ~10 fields
    pub image_edit: ImageEditState,       // ~7 fields
    pub view: ViewState,                  // ~6 fields
    pub gizmo: GizmoStateGroup,           // ~3 fields
    pub store: WidgetStore,
    pub hit_index: HitIndex,
    pub bus: ActionBus,                   // PR 11.8 foundation
    // ...
}
```

**Recommendation:** só fazer se um benefício concreto aparecer
(e.g., multi-agente colidir em hero.rs durante feature dev). Caso
contrário, hero.rs flat fica como está.

### PR 11.8b — migrate image-edit intents into ActionBus

**Pré-condição:** PR 11.8 foundation já mergeada (`8cbfe4e`).

**Plano:** lift os 4 image-edit `pending_X` para `EditorAction`:

- `pending_trim_transparency` → `EditorAction::Trim`
- `pending_make_square` → `EditorAction::MakeSquare`
- `pending_bgremoval` → `EditorAction::Bgremoval`
- `pending_activate_bgremoval` → `EditorAction::ActivateBgRemoval`
- `pending_reimport` → `EditorAction::Reimport`

Adicionar variants ao enum (Reimport ainda não existe). Atualizar
`HeroScreen::apply_event` para `self.bus.push(...)` em lugar de
`self.pending_X = Some(...)`. Atualizar `shells/desktop/src/main.rs`
+ `hero_intents.rs` para drenar via `for action in
hero.bus.drain() { match action { ... } }`.

**Impacto:** remove ~300 LOC de `hero_intents.rs` (de 696 → ~400),
remove a HR-18 exception. Reduz `main.rs` em ~50 LOC.

**Esforço:** ~2h. **Risco:** MÉDIO — refatora dispatch path.

### PR 11.8c — migrate hierarchy intents

Lift `pending_visibility_toggle`, `pending_reparent`,
`pending_duplicate`, `pending_delete`, `pending_reset_transform`,
`pending_add_child`, `pending_hierarchy_row_click`,
`pending_rename_seed`, `pending_rename_commit` para
`EditorAction::Hierarchy*` variants.

**Esforço:** ~1.5h. **Risco:** MÉDIO.

### PR 11.8d — migrate inspector intents

Lift `pending_transform_edit`, `pending_visibility_edit`,
`pending_sprite_source_change`, `pending_name_edit`,
`pending_view_focus`, `pending_undo_image_edit` para
`EditorAction::Inspector*`.

**Esforço:** ~1.5h. **Risco:** MÉDIO.

**Resultado conjunto de 11.8b+c+d:** todas as 20 pending_X fields
removidas; `main.rs` < 600 LOC e HR-18 exception removida;
`hero_intents.rs` colapsa ou desaparece.

### PR 11.10 — Golden image tests

Vello headless rendering + SSIM compare contra baselines PNG. ~30
widgets × ~5 estados = ~150 baselines committed (~1.5MB total).

**Esforço:** ~2h. **Risco:** MÉDIO (setup headless Vello quirks).

Defer alternativo: nunca implementar — manter validação visual
manual via `play.command` smoke.

## Ordem recomendada

```
PR 11.8b  (image-edit migrations)    ← unlock 1: remove ~300 LOC de hero_intents.rs
PR 11.8c  (hierarchy migrations)
PR 11.8d  (inspector migrations)     ← unlock 2: HR-18 exceptions removidas
PR 11.10  (golden images)            ← independente, baixa prioridade
PR 11.7d  (HeroScreen state decomp)  ← só se valor concreto emergir
```

**Total estimado:** 5-7h de sessão dedicada (não cabe em uma
janela LLM padrão).

## Critério de fechamento Wave 3

- 11.8b/c/d todos mergeados → HR-18 file_loc_caps inventory test
  emite "exceptions inventory: NONE (cap fully active)".
- ADR-0028 status atualizado para incluir Wave 2.5 + Wave 3
  conclusão.
- STATE.md sha bom atualizado.

## Não-prioridade

- PR 11.7d é decomp puro de hygiene; só executar se houver
  conflito multi-agente concreto em hero.rs (até 2026-05-16
  nenhum).
- PR 11.10 é validação visual sintética; manter smoke manual via
  `play.command` é cost-effective enquanto não houver designer
  contribuindo PRs de UI diretamente.
