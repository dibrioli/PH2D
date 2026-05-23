# ADR-0040 — Ferramenta como crate de feature isolado (contrato `Tool`/`ImageEditTool` + canal de ação genérico + registro por codegen)

**Status:** Accepted (implementado e FREEZE ratificado 2026-05-22 — vide §7 Histórico de execução)
**Data:** 2026-05-22
**Decisor(es):** Enio + Claude (arquiteto).
**Estende:** ADR-0031 (nó **e** ferramenta como unidade de feature) — especifica o contrato concreto da família *ferramenta*, que o 0031 declarou mas não definiu. Espelha o que ADR-0032 + ADR-0039 fizeram pelo contrato de **nó**.
**Relaciona:** ADR-0027 (convention-by-discovery / tool-as-crate), ADR-0029 (trait-driven panel host — o mesmo padrão satélite aplicado a painéis), ADR-0038 (tools terminais artista-primeiro).
**Espelha o mecanismo de:** o fan-out de nós (`ph2d-node-sync` + glob members + contrato congelado), DIRETRIZ §3.8.

---

## 1. Contexto

O projeto vai ter **uma quantidade enorme de ferramentas** (bgremoval é só a primeira pesada). O objetivo é o mesmo do sistema de nós: **crescer por adição de crate isolado, com zero edit central por feature** (isolamento FBP = unidade multi-agente, ADR-0030/0031).

Hoje as ferramentas **não** atingem isso — e por **inércia**, não por princípio. A migração convention-by-discovery (ADR-0027) puxou os `ToolManifest` (dado) + alguns algoritmos puros pros crates `ph2d-tool-*`, mas deixou os `Tool` impls *stateful* dentro de `ph2d-editor-core/src/tools/`. Estado em 2026-05-22:

| Tool | Onde está | Tipo | Crate |
|------|-----------|------|-------|
| BgRemoval | `editor-core/tools/bgremoval/` (~4.9k LOC) | stateful | stub (só manifest) |
| Padding | algoritmo no crate; `Tool` em `editor-core/tools/padding/` | stateful | parcial |
| Move / Brush | `editor-core/tools/{move_tool,brush}.rs` | stateful | inexistente |
| Trim Transparency | `editor-core/tools/trim_transparency/` | OneShot | manifest no crate |
| Make Square / Real Size | algoritmo no crate | OneShot | completo |

A inconsistência é o sintoma. **Não há gate** que force "tool stateful vive em editor-core" — o `architecture_cycle_prevention` só proíbe `editor-core → panel-*` e `panel-* → ph2d-editor`. A direção `tool → editor-core` é **permitida** (é o padrão satélite, igual `panel-* → editor-core`). As ferramentas ficaram no god-crate só porque ninguém adicionou a dep de editor-core num tool-crate.

### 1.1 Os três pontos de edit central que prendem um tool a editor-core

1. **O trait `Tool`** (`editor-core/src/tool.rs`): `id/label/icon_slug/build_panel/on_activate/on_deactivate/handle_panel_event/as_any_mut`. **Isto não é o problema** — é o contrato, e um tool-crate dependendo de editor-core pra implementá-lo é o padrão correto.
2. **A `ToolRegistry` é construída na mão no shell** (`shells/desktop/src/init.rs`): `tools.register(Box::new(BrushTool::default()))` × N. Adicionar tool ⇒ editar o shell.
3. **O `EditorAction` é um enum-deus** (`editor-core/src/action_bus.rs`) com variants por-tool — `ActivateBgRemoval`, `BgremovalUiEdit(BgRemovalUiEdit)`, `Bgremoval`, `BgremovalCancel`, `ActivatePadding`, `PaddingUiEdit`, … — e o shell tem um `match` gigante (`render_loop/mod.rs`) que despacha cada um. Adicionar tool ⇒ editar o **enum** + o **match**. É **edit central triplo por tool** — o pior, e o que não escala pra "enorme quantidade de tools".

O sistema de nós **já resolveu exatamente esse problema**: um node-crate é drop-in (`MANIFEST` + `eval` + `register`), e `ph2d-node-sync` **gera** o wiring central a partir de um scan de `crates/ph2d-node-*`; o `workspace.members` é glob. Zero edit central. Esta ADR aplica o mesmo mecanismo às ferramentas.

---

## 2. Decisão

A unidade de ferramenta é **`crates/ph2d-tool-<slug>/` contendo TUDO**:

```
crates/ph2d-tool-<slug>/
  src/lib.rs        # pub fn make() -> Box<dyn Tool> ; pub const MANIFEST ; pub fn register(reg)
  src/manifest.rs   # ToolManifest (dado: id/icon/zone/cluster/order/kind/budget)
  src/tool.rs       # impl Tool [+ impl ImageEditTool] — comportamento (dep de editor-core)
  src/algorithm/    # lógica pura (sem dep de editor-core)
  src/icon.rs       # BezPath
```

`ph2d-editor-core` volta a ser **pura foundation**: define o **contrato** (traits `Tool`/`ImageEditTool`, `FloatingPanel`, `PanelEvent`, `widget::*`, `ToolRegistry`, chrome) e **não conhece nenhum tool concreto**. Três mecanismos tornam isso real:

### 2.1 O contrato `Tool` + sub-trait `ImageEditTool` (congelável)

`Tool` (já existe, mantém-se) é a base genérica: identidade + painel Procreate-style + ciclo de vida + `handle_panel_event(PanelEvent)` (PanelEvent já é **genérico**: `Click/SetValue/Toggle/SelectOption` por `NodeId`).

Tools que transformam um raster implementam o **sub-trait** novo:

```rust
/// Tool que produz novos pixels para a entidade ativa (bg removal,
/// padding, trim, make-square, real-size). O shell dirige QUALQUER
/// ImageEditTool por um caminho único — sem variant por-tool.
pub trait ImageEditTool: Tool {
    /// Alimenta os pixels-fonte (straight alpha) da entidade ativa quando
    /// a seleção muda. O tool cacheia + computa o preview downscale internamente.
    fn set_source(&mut self, src: ImageView<'_>);
    /// Preview corrente (downscale), se houver — desenhado pelo shell.
    fn preview(&self) -> Option<ImageView<'_>>;
    /// O usuário pediu commit (Apply)? Drena a flag.
    fn take_pending_commit(&mut self) -> bool;
    /// Roda em resolução cheia → novos pixels. Chamado pelo shell no commit.
    fn run_full(&mut self) -> ImageBuf;
}
```

O `ToolManifest` ganha um `kind: ToolKind` (`ImageEdit | Gizmo | Paint | OneShot`) pra o shell despachar genericamente sem conhecer o tipo concreto.

**Edits semânticos viram internos ao tool-crate.** O mapeamento "slider movido → `BgRemovalUiEdit::Tolerance(v)`" sai de `editor-core` e entra no `handle_panel_event` do tool: o tool recebe `SetValue(TOLERANCE_ID, v)` genérico e mapeia pro seu enum interno. Os enums semânticos (`BgRemovalUiEdit`, `PaddingUiEdit`) **deixam de existir em editor-core** — viram detalhe privado do crate. O painel emite `PanelEvent` chaveado pelos `NodeId`s que o tool publica (via manifest/const compartilhada).

### 2.2 Canal de ação genérico (mata o enum-deus)

`EditorAction` perde **todos** os variants por-tool (`ActivateBgRemoval`, `Bgremoval*`, `Padding*`, e os OneShot `Trim`/`MakeSquare`/`RealSize` viram `ImageEditTool kind=OneShot`). Fica só com ações **genéricas cross-tool**: hierarquia, inspector, view, present-mode, reimport, undo. A interação com tools passa **inteiramente** pelo contrato:

- **Ativação:** `ToolRegistry::set_active(id)` — genérico, por id (já existe). A pill/palette/atalho/bus chamam isso; o gate "Image Tools off" continua reconciliando por frame (Image Tools Bugs §2, DIRETRIZ §4.1.2).
- **Edits de UI:** `tools.active_mut().handle_panel_event(ev)` — genérico.
- **Commit/preview de raster:** o loop único do shell sobre `ImageEditTool`:

```rust
// shell, caminho ÚNICO para todo ImageEditTool — zero conhecimento de tool concreto
if let Some(tool) = tools.active_image_edit_mut() {
    if selection_changed { tool.set_source(active_entity_pixels()); }
    // (panel events já roteados via handle_panel_event → tool recomputa preview)
    if tool.take_pending_commit() {
        swap_entity_texture(active_entity, tool.run_full());
    }
    draw_preview(tool.preview());
}
```

Adicionar um image-edit tool ⇒ **nada** muda no shell nem na `EditorAction`.

### 2.3 Registro por codegen (`ph2d-tool-sync`, espelha `ph2d-node-sync`)

Um gerador escaneia `crates/ph2d-tool-*` e gera, em `ph2d-tool-registry-init`:
- `register_all(reg: &mut Registry)` — os `ToolManifest` (dado; já existe, passa a ser **gerado**).
- `register_all_tools(reg: &mut ToolRegistry)` — chama `ph2d_tool_<slug>::make() -> Box<dyn Tool>` de cada crate (behavior).

O shell chama **uma linha** (`register_all_tools(&mut tools)`) no lugar dos N `register(Box::new(...))`. `workspace.members` já é glob (`crates/*`). Um **staleness gate** (espelha `ph2d-node-registry-init/tests/staleness.rs`) falha se o arquivo gerado estiver dessincronizado do scan. Adicionar tool ⇒ largar crate + `cargo run -p ph2d-tool-sync`. Zero edit central.

### 2.4 editor-core = foundation; gate de ciclo estendido

`architecture_cycle_prevention` ganha a asserção: **`ph2d-editor-core` não depende de nenhum `ph2d-tool-*`** (é a foundation). `tool-* → editor-core` segue permitido. Isso prova por gate que a inversão não regride.

---

## 3. Consequências

**Aceitas:**
- editor-core encolhe materialmente (só a vertical bgremoval são ~4.9k LOC) e volta a ser **contrato, não catálogo**. O peso migra pros satélites.
- Escala pra N tools com **zero edit central por tool** — o que motivou a ADR ("enorme quantidade de tools").
- Cada tool vira testável em isolamento (algoritmo puro = golden test; `ImageEditTool` = teste source→run_full), no slot do agente, como os node-crates.
- A `EditorAction` deixa de crescer por tool; o `match` do shell para de inchar.
- Habilita o fan-out paralelo de tools (vários agentes, um tool-crate cada) — novo balde na DIRETRIZ, irmão do §3.8 de nós.

**Riscos / custos:**
- **Ripple de rebuild:** `tool-* → editor-core` significa que mudança em editor-core recompila todos os tool-crates. Já é verdade pros panels; é o custo aceito do modelo satélite (mitigado por build-por-slot, Gargalo 1).
- **`ImageEditTool` precisa cobrir os caminhos interativos hoje encapsulados** (preview no thumbnail, protect-mask brush, eyedropper). O contrato acima cobre preview+commit; **eyedropper/protect-brush** (que hoje chamam métodos diretos no `BgRemovalTool` via downcast) precisam de hooks genéricos OU continuam por downcast até o contrato amadurecer. **Decidido na vertical** (T2), não às cegas.
- **Congelar cedo demais** repetiria o erro que o sistema de nós evitou: o contrato `Tool`/`ImageEditTool` **só congela depois** que a vertical bgremoval o exercitar end-to-end (§5).

**Neutras:**
- O vocabulário semântico de cada tool (ex.: `BgRemovalUiEdit`) deixa de ser compartilhado e vira privado do crate — o painel passa a falar `PanelEvent` genérico chaveado por `NodeId`.

---

## 4. Alternativas consideradas

- **Manter tools stateful em editor-core (status quo):** rejeitado — é a inércia que esta ADR corrige; não escala e contradiz o norte FBP (ADR-0030/0031).
- **Mover só o algoritmo puro pra um crate-folha, deixar o `Tool` em editor-core** (a "Opção B" discutida): rejeitado como solução final — encolhe menos, mantém o tool no god-crate, e não ataca os pontos 2/3 (registro manual + enum-deus). Aceitável só como passo intermediário se a vertical revelar bloqueio no contrato.
- **Mover o `Tool` pra o crate mas manter a `EditorAction` com variants por-tool:** rejeitado — resolve o ponto 2 mas não o 3 (o pior); o shell continuaria editado por tool.
- **Dispatch por `linkme`/`inventory` em vez de codegen:** rejeitado pelo mesmo motivo dos nós (ADR-0031 §4) — ordem dependente de link-order atrita com determinismo; codegen dá lista explícita/diffável/determinística.
- **Carregamento dinâmico (WASM/dylib) de tools:** rejeitado (sem ABI Rust estável; UI cruzando boundary GPU/AccessKit; atrito com HR-3). A lane sandboxed é o Luau (gameplay), não a UI first-party.

---

## 5. Plano de implementação (funil: neck → freeze → fan-out, espelha as waves de nó)

**T1 — NECK (Coordenador-only, serial).** O contrato + a fiação genérica:
- Finalizar `Tool` + `ImageEditTool` + `ToolKind` em editor-core.
- Canal de ação genérico: remover os variants por-tool da `EditorAction`; o loop único de `ImageEditTool` no shell; ativação por id.
- `ph2d-tool-sync` (gerador) + `register_all_tools` gerado + staleness gate.
- Estender `architecture_cycle_prevention` (editor-core ⊥ tool-*).

**T2 — PROVAR A VERTICAL (serial, precisa do smoke do Enio).** Migrar **BgRemoval** inteiro pro `crates/ph2d-tool-bgremoval/` através do contrato novo (manifest + `BgRemovalTool` + algoritmo + icon + edits internos). Resolve eyedropper/protect-brush no contrato real. É a vertical que **prova o contrato de tool**, como a Motion provou o de nó.

**🔒 FREEZE.** Depois que a vertical bgremoval passar (smoke do Enio incluso), capar a superfície de `Tool`/`ImageEditTool` por arch-gate (espelha `architecture_contract_surface` dos nós) e declarar estável. Mudança de contrato vira evento raro Coordenador-only + ADR.

**FAN-OUT (paralelo, pós-freeze).** Migrar padding/brush/move/trim/make_square/real_size, cada um pro seu crate, um agente por tool. Daí em diante, **tool novo = crate drop-in** via briefing irmão do §3.8 (a criar na DIRETRIZ: balde "tool fan-out").

---

## 6. Notas

- Esta ADR **não** generaliza a `EditorAction` inteira — só remove o que é por-tool. As ações genéricas (hierarquia/inspector/view) seguem como estão; uma eventual generalização delas é trabalho separado.
- A dualidade `FloatingPanel` (painel Procreate-style que o tool constrói) vs. crates `ph2d-panel-*` (painéis docados `Panel<State>`) **permanece** — bgremoval usa ambos hoje. Racionalizar essa dualidade é fora de escopo; esta ADR só garante que o tool e seu vocabulário saem de editor-core.
- Atualizar a DIRETRIZ (novo balde de fan-out de tool, irmão do §3.8) **após o FREEZE**, não antes — pra não documentar contrato instável.

---

## 7. Histórico de execução

Implementação fechou em 2026-05-22 numa única jornada de cinco fases (TG-A..TG-E),
seguindo o plano funil neck → freeze → fan-out de §5. Smoke do Enio entre fases
(bgremoval interativo e padding interativo) passaram sem regressão. Commits
locais (não-pushados; ship é do Enio):

| Fase | Commit | Resumo |
|---|---|---|
| Pré-TG-B (T-close) | `1484a49` | Handoff executável dos substeps + plano sincronizado. |
| TG-A | `5be7541` | `EditorAction::ActivateTool { tool_id }` + `OneShotImageOp { tool_id, entity_bits }` genéricos; 7 dos 11 variants per-tool eliminados (Trim/MakeSquare/RealSize/ActivateBgRemoval/ActivatePadding/dead-code). |
| TG-A close | `42438be` | Register pure-push + activate_default data-driven + codegen `register_all_tools` (3 staleness gates: manifests + Box<dyn Tool> + Cargo deps). |
| TG-B | `7676793` | `ToolPanelEvent(PanelEvent)` + `CancelActiveTool` genéricos; `BgRemovalTool::handle_panel_event` cobre os 15 `BGR_*` NodeIds via `apply_ui_edit`. `take_params_dirty` substitui `!bgremoval_ui_edits.is_empty()` no canvas-preview gate. Vocab `BgRemovalUiEdit`/`UiSnapshot`/`BrushFalloff`/`BgRemovalParams` migrou de `editor-core/src/tools/bgremoval/params.rs` para `ph2d-tool-bgremoval/src/params.rs`. `on_deactivate` zera `pending_apply` + `params_dirty` (fix de bake fantasma latente). |
| TG-C | `4a15d9b` | Espelho mecânico de TG-B em padding. `PaddingTool::handle_panel_event` cobre 10 `PAD_*` NodeIds (sliders + chips + Pivot + Apply). Vocab migrou de `editor-core/src/tools/padding/params.rs` para `ph2d-tool-padding/src/params.rs`. `take_params_dirty` não aplicado em padding — o bridge não tem cache de preview a invalidar (snapshot/overlay incondicional cada frame). |
| TG-D | `c4063b7` | `editor-core/src/tools/` deletado + `pub mod tools;` removido de `editor-core/src/lib.rs`. Doc-comment atualizado. |
| TG-E | (este) | **FREEZE.** Arch-gate `architecture_tool_contract_surface` caps: `Tool=10` métodos, `ImageEditTool=4`, `PanelEvent=4` variants. 🔒 markers nos doc-comments de `Tool`/`ImageEditTool`/`PanelEvent`. `panel_crate_tomls()` agora auto-descobre `crates/ph2d-panel-*` e o gate `panel_crates_depend_only_on_editor_core` ganhou ban explícito de cross-panel-dep (excetuando `panel-registry-init`) — codifica a edge panel→tool permitida. DIRETRIZ §3.1 neutralizada como referência histórica; novo §3.9 "Tool crate — fan-out" espelha §3.8 (briefing pronto-pra-colar + garantia sem-colisão + checklist do revisor). SKILL_Stack §"Adicionar uma tool" reescrita para 3 passos (largar crate + `cargo run -p ph2d-tool-sync` + verificar 3 gates). |

**Métricas:**
- `EditorAction`: **11 variants per-tool removidos** (TG-A 7 + TG-B 2 + TG-C 2); restam apenas os 4 genéricos (`ActivateTool`, `OneShotImageOp`, `ToolPanelEvent`, `CancelActiveTool`).
- LOC saídos de `editor-core`: ~512 LOC de vocab (bgremoval + padding `params.rs`) + ~22 LOC de stubs/decls (`tools/mod.rs`, `pub mod tools;`).
- Tools em crates satélite: **10 de 10** (bgremoval, brush, grid-snap, make-square, move, padding, real-size, trim-transparency + registry + registry-init).
- Arch-gates ativos pós-FREEZE: 4 em `architecture_cycle_prevention` + 3 em `architecture_tool_contract_surface` + 3 staleness em `ph2d-tool-registry-init`.

**Auditorias adversariais** (≥2 agentes paralelos com lentes distintas — paridade comportamental + arquitetura/cycles — em TG-B, TG-C). Achados Médio/Alto remediados pré-commit. Follow-ups documentados (gate de panel auto-discover) endereçados no próprio TG-E. Stutter de mouse VSync = trade-off documentado em `docs/perf/mouse-stutter.md` (não-regressão).
