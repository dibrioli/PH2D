# Plano de waves — isolamento de ferramentas (tool-as-crate)

**Data:** 2026-05-22
**Status:** T1 (neck) + T2 (bgremoval) + T3a/b (trim, padding) + **T-close (brush/move + register_all_tools codegen)** FEITOS e verificados headless; **smoke das 4 image tools OK pelo Enio**. Pendente: smoke do palette pós-T-close (Brush default + on-boot/fallback) + canal de ação genérico (T1.2/T1.3, smoke-dependente).
**Arquitetura:** [ADR-0040](../architecture/decisions/0040-tool-as-isolated-feature-crate.md) (contrato `Tool`/`ImageEditTool` + canal de ação genérico + `ph2d-tool-sync` codegen) · estende ADR-0031 (tool como unidade de feature) · espelha o fan-out de nós (ADR-0030/0039, [`node-waves.md`](2026-05-node-waves.md)).
**Norte:** editor-core vira **pura foundation** (só o contrato). Cada tool é satélite drop-in. Escala pra N tools com **zero edit central por tool**.

Os tags `T1.x`/`T2.x` que aparecerem em comentários de código referenciam este doc.

---

## ✅ Estado de execução (run autônomo 2026-05-22)

**Re-staging consciente do plano original:** o plano previa generalizar-o-canal-primeiro (T1.2/T1.3) e relocar depois. O run autônomo INVERTEU para **relocar-primeiro** (behavior-preserving, 100% verificável headless) e deixar o canal genérico (smoke-dependente) por último — porque sem o smoke do Enio o canal genérico não pode ser provado, e a relocação entrega o grosso do valor (tools fora de editor-core) com prova headless. A relocação mantém o **vocabulário** (UiEdit/UiSnapshot) em editor-core temporariamente (re-exportado via `crate::params`); ele migra pro crate quando o canal genérico landar.

| Fase | Estado | Verificação |
|------|--------|-------------|
| **T1.1** contrato `ImageEditTool` + `as_image_edit_mut` upcast | ✅ feito | unit tests; auditado (zero Crít/Alto) |
| **T1.4** `ph2d-tool-sync` codegen + staleness gate | ✅ feito | idempotente; staleness+alfabético verdes; auditado |
| **T1.5** gate de ciclo `editor_core_has_no_concrete_tool_deps` + make_square 100% no crate | ✅ feito | gate provado (injeção→FAILED); machete limpo |
| **T2** BgRemoval relocado (~4.4k LOC) | ✅ feito | 98 testes do crate; auditado **zero achados Crít/Alto/Médio** |
| **T3a** trim_transparency relocado (isolamento completo, `from_trim` removido) | ✅ feito | 19 testes; auditado (só doc) |
| **T3b** padding relocado (PaddingTool→crate) | ✅ feito | 19 testes; auditado (só doc) |
| **T-close** brush+move relocados (Tool+make, sem manifest) + `register_all_tools` codegen + `Tool::is_default` + 5 fallbacks → `default_tool_id` | ✅ feito | 183 binários verdes; auditado **1 Alto + 3 Médios corrigidos** (gate alfabético cobre tools_body; needles `pub fn make(` estritos; `register` pure-push, ativa via `activate_default`) |
| **T1.2/T1.3** canal de ação genérico (mata os variants por-tool da `EditorAction`) | ❌ **NÃO feito** | smoke-dependente (rewire de preview/eyedropper/protect interativos + painel). É o que torna "zero edit central" COMPLETO. Os 512 LOC de vocab em editor-core/tools/ migram aqui. |
| **🔒 FREEZE** | ❌ não feito | só após smoke do palette pós-T-close + o canal genérico landar |

**Resultado pós-T-close:** TODOS os tools (make_square, bgremoval, trim, padding, brush, move + real_size + grid_snap) estão isolados em crates; `editor-core/src/tools/` tem apenas 512 LOC (os módulos de vocabulário `params` do bgremoval/padding — migram quando o canal genérico landar). **Init.rs não tem mais bloco manual de registro de tools.** Adicionar tool agora = largar crate + rodar `cargo run -p ph2d-tool-sync`, **zero edit central** — paridade total com o fan-out de nós. **Workspace verde: 183 binários de teste.** Smoke do Enio: ✅ 4 image tools (T2/T3); ⏳ pendente palette pós-T-close (Brush default + fallback ao deactivar image tool).

---

## Forma: funil (idêntico ao dos nós)

**Neck serial** (o contrato + a fiação genérica, Coordenador-only — nenhum nº de agentes acelera) → **🔒 FREEZE** → **fan-out paralelo** (N agentes, um tool-crate cada, sem colisão). A velocidade total depende de congelar o contrato cedo e **não** abrir o fan-out antes de uma vertical (BgRemoval) provar end-to-end.

## Padrão-ouro + loop de auditoria (vale para TODA fase)

Cada fase segue o loop do [`HANDOFF_node_system.md`](../HANDOFF_node_system.md) §1, sem economias (§0):

1. **Build isolado:** `CARGO_TARGET_DIR="$PWD/target/slot-coord" cargo ...`.
2. **Implementar no padrão-ouro** — forma definitiva, sem `unwrap`/falha silenciosa, sem `TODO: depois`, toda superfície pública documentada.
3. **Auto-verificar verde:** `cargo test -p <crate>` + `clippy --all-targets -- -D warnings` + `fmt --check`.
4. **AUDITAR — ≥2 auditores adversariais em paralelo** (Agent `general-purpose`), lentes distintas (corretude/edge · acoplamento/ciclo · paridade-comportamental antes↔depois · consistência docs↔código). Instruídos a serem **duros**, achar bugs/lacunas, dar severidade, **não validar por cortesia**.
5. **CORRIGIR todos os achados** (Crítico→Baixo). Nada adiado salvo follow-up não-bloqueante registrado aqui no §Follow-ups.
6. **RE-AUDITAR até erro zero.**
7. **Commit** (`--no-verify`, background, local; um commit limpo por fase) + atualizar este plano + todos.
8. **Próxima fase.**

**Gate transversal de TODA fase de refactor:** paridade comportamental. Antes de declarar uma fase pronta, provar que o app se comporta **idêntico** ao anterior (smoke do Enio quando a fase toca pixels/input/tela; teste de paridade quando dá pra automatizar). Isolamento que muda comportamento é regressão, não refactor.

**Quando PARAR e chamar o Enio:** fase que precisa de **smoke visual** (`./play.command`); a decisão de **FREEZE**; bloqueio que exija mudar foundational além do previsto. Push/CI é o **ship do Enio** — acumular commits locais.

---

## T0 — Baseline + convenção de slot · ✅ pré-requisito
- [ ] Baseline verde no HEAD (`cargo test --workspace --exclude ph2d-asset`).
- [ ] Build sempre em `target/slot-coord` (Gargalo 1; não contende no lock).
- **Exit:** baseline conhecido-verde antes de tocar foundational.

---

## T1 — O NECK (contrato + fiação genérica) · SERIAL · Coordenador-only

Objetivo: estabelecer o contrato e a fiação genérica **com os tools ainda dentro de editor-core**. Estratégia anti-risco: **generalizar in-loco primeiro, relocar depois** (T2+). Ao fim do T1 o app funciona idêntico, mas a `EditorAction` não tem mais variant por-tool e o registro é codegen-ready.

### T1.1 — Tipos do contrato
- [ ] `ImageEditTool: Tool` em `editor-core/src/tool.rs` (`set_source` / `preview` / `take_pending_commit` / `run_full`), com os tipos de view/buffer (reusar o tipo de imagem existente do shell/asset, não inventar um novo sem necessidade).
- [ ] `ToolKind` (`ImageEdit | Gizmo | Paint | OneShot`) no `ToolManifest` (`ph2d-tool-registry`) + no design TOML + gate `tool_manifest_design_sync` atualizado.
- **Auditoria:** revisão de superfície (o contrato é mínimo? cobre preview+commit+source sem vazar estado?). **Exit:** `cargo test -p ph2d-editor-core -p ph2d-tool-registry` verde; contrato documentado.

### T1.2 — Driver genérico de ImageEditTool no shell
- [ ] O loop único em `shells/desktop/src/render_loop/` que dirige **qualquer** `ImageEditTool` (set_source on selection-change, recompute via handle_panel_event, commit via take_pending_commit→run_full→swap texture, draw preview).
- [ ] Eyedropper/protect-brush: mantêm-se por downcast ao tipo concreto (exceção pragmática documentada — o shell pode depender do tool-crate; generalizar só se um 2º tool precisar do mesmo padrão).
- **Auditoria:** paridade de comportamento do caminho image-edit (preview/apply/cancel idênticos). **Exit:** smoke do Enio (bgremoval + padding funcionam igual).

### T1.3 — Matar os variants por-tool da EditorAction
- [ ] Converter os tools image-edit existentes (`bgremoval`, `padding`) pra rotear edits via `handle_panel_event` (mapeamento slider→edit semântico vai pra DENTRO do tool); os enums `BgRemovalUiEdit`/`PaddingUiEdit` deixam de aparecer em `action_bus.rs`.
- [ ] OneShot (`Trim`/`MakeSquare`/`RealSize`) viram `ToolKind::OneShot` dirigidos pelo mesmo driver (run_full uma vez, sem preview interativo).
- [ ] Remover os variants por-tool de `EditorAction` + o `match` correspondente em `render_loop/mod.rs`. Sobram só ações genéricas (hierarquia/inspector/view/present/reimport/undo).
- **Auditoria:** grep prova zero `EditorAction::{Bgremoval*,Padding*,Trim,MakeSquare,RealSize}`; paridade de todos os caminhos de ativação (pill/palette/atalho/bus) — DIRETRIZ §4.1.2. **Exit:** smoke do Enio (todas as image tools idênticas).

### T1.4 — Codegen do registro (`ph2d-tool-sync`)
- [ ] `tools/ph2d-tool-sync` (espelha `ph2d-node-sync`): scan de `crates/ph2d-tool-*` → gera `register_all` (manifests) + `register_all_tools(&mut ToolRegistry)` (chama `ph2d_tool_<slug>::make()`) em `ph2d-tool-registry-init`.
- [ ] `pub fn make() -> Box<dyn Tool>` em cada tool-crate que tiver behavior.
- [ ] Shell: substituir os N `tools.register(Box::new(...))` no `init.rs` por `register_all_tools(&mut tools)`.
- [ ] **Staleness gate** (espelha `ph2d-node-registry-init/tests/staleness.rs`): falha se o gerado divergir do scan.
- **Auditoria:** determinismo do gerado (ordem estável/alfabética); o gate pega esquecimento de sync. **Exit:** `cargo test -p ph2d-tool-registry-init` verde; adicionar tool-crate fake + sync registra sem edit manual.

### T1.5 — Gate de ciclo estendido
- [ ] `architecture_cycle_prevention`: asserir **`ph2d-editor-core` não depende de nenhum `ph2d-tool-*`**.
- **Auditoria:** o gate falha de propósito se eu adicionar uma dep tool em editor-core. **Exit:** gate verde + provado que barra a inversão.

**T1 EXIT (neck fechado):** contrato `Tool`/`ImageEditTool` estável; `EditorAction` sem variant por-tool; registro 100% codegen; gate de ciclo ativo; **app idêntico** (smoke do Enio). Tools ainda em editor-core — relocação é T2+.

---

## T2 — PROVAR A VERTICAL: BgRemoval · SERIAL · precisa do smoke do Enio

Objetivo: relocar **BgRemoval inteiro** pra `crates/ph2d-tool-bgremoval/` pelo contrato novo. É a vertical que **prova o contrato de tool**, como a Motion provou o de nó. Agora é relocação pura (o contrato já é genérico).

- [ ] **T2.1** — Mover algoritmo puro (`algorithm/` + `scratch`) pro crate (sem dep editor-core).
- [ ] **T2.2** — Mover `BgRemovalTool` (tool.rs) + `BgRemovalParams` + edits semânticos (agora privados) + icon pro crate; o crate ganha dep `ph2d-editor-core` (direção permitida) e larga a dep do shim `ph2d-editor`. `make()` + `MANIFEST` + `register` no `lib.rs`.
- [ ] **T2.3** — `cargo run -p ph2d-tool-sync`; o shell passa a falar `ph2d_tool_bgremoval::*` (resolve parte do sweep do shim de quebra).
- [ ] **T2.4** — Resolver eyedropper/protect-brush no contrato real (hook genérico se limpo, senão downcast ao crate, documentado).
- **Auditoria (≥2 lentes):** algoritmo bit-idêntico ao anterior (golden); paridade do preview/apply/eyedropper/protect; zero símbolo bgremoval órfão em editor-core; gate de ciclo verde.
- **Exit:** `cargo test -p ph2d-tool-bgremoval -p ph2d-tool-registry-init` verde; **smoke do Enio** (bg removal idêntico de ponta a ponta); editor-core ~4.9k LOC menor.

---

## 🔒 FREEZE — gate do fan-out

Depois que a vertical BgRemoval passar (smoke incluso):
- [ ] Capar a superfície de `Tool`/`ImageEditTool`/`ToolManifest` por arch-gate (espelha `architecture_contract_surface.rs` dos nós; caps apertados ao tamanho atual).
- [ ] Marcadores 🔒 nos `lib.rs`/`tool.rs` do contrato. Mudança vira evento raro Coordenador-only + ADR.
- [ ] **Atualizar as docs canônicas pós-freeze** (só agora, contrato estável): DIRETRIZ novo balde "tool fan-out" (irmão do §3.8) + briefing pronto-pra-colar; SKILL_Stack §"Adicionar uma tool" reescrito pro fluxo codegen; ADR-0040 → `Status: Accepted (implementado)`.
- **Exit:** contrato congelado + gateado + documentado. **Fan-out aberto.**

---

## T3+ — FAN-OUT · PARALELO (pós-freeze)

Relocar os tools restantes, **um agente por tool-crate**, governados pelo briefing do balde novo. Sem colisão (cada um sua pasta; registro gerado; glob members).

- [ ] **T3.1** — Padding (já tem algoritmo no crate; relocar o `Tool` + params).
- [ ] **T3.2** — Trim Transparency (algoritmo ~661 LOC em editor-core → crate; é OneShot).
- [ ] **T3.3** — Make Square (consolidar o shim; algoritmo já no crate).
- [ ] **T3.4** — Real Size (já no crate; conformar ao contrato `make()`).
- [ ] **T3.5** — Brush (sem crate hoje; criar `crates/ph2d-tool-brush/`).
- [ ] **T3.6** — Move (sem crate hoje; criar `crates/ph2d-tool-move/`; é `ToolKind::Gizmo`, exercita a categoria não-raster do contrato).
- Cada um: loop padrão-ouro + auditoria + smoke se tocar tela. **Exit por tool:** crate isolado, `register_all_tools` gerado, gate de ciclo verde, paridade provada.

**CONCLUSÃO TOTAL:** `editor-core/src/tools/` vazio (ou só re-exports de compat a deletar); `EditorAction` sem nada por-tool; `init.rs` sem registro manual; todo tool é crate satélite drop-in. editor-core = pura foundation. Tool novo daí em diante = largar crate + `ph2d-tool-sync`, zero edit central — paridade total com o fan-out de nós.

---

## Follow-ups diferidos (não-bloqueantes)
- Dualidade `FloatingPanel` (tool-owned) vs. crates `ph2d-panel-*` (`Panel<State>` docado) — bgremoval usa ambos; racionalizar é fora de escopo do ADR-0040 (item próprio).
- Generalizar eyedropper/protect-brush além do downcast — só se um 2º tool pedir o mesmo padrão.
- Generalizar as ações não-tool da `EditorAction` (hierarquia/inspector) — trabalho separado; o ADR-0040 só remove o que é por-tool.
- Deletar o shim `ph2d-editor` — o T2/T3 reduzem consumidores de `ph2d_editor::*`; o sweep final + deleção é a outra frente de limpeza (matar-o-shim), destravada por esta.

---

## Riscos (do ADR-0040 §3, vivos aqui)
- **Congelar cedo demais** — só após T2 exercitar o contrato. (lição da vertical Motion).
- **`ImageEditTool` não cobrir caminho interativo** (eyedropper/protect) — decidir na vertical T2, não às cegas.
- **Ripple de rebuild** tool→editor-core — custo aceito do modelo satélite (mitigado por slot).
- **Paridade comportamental** — todo refactor prova idêntico antes de fechar; smoke do Enio nos pontos de tela/input.
