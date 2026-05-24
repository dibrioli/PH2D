# Proposta v4 FINAL — Estrutura PH2D Multi-Agente (escopo máximo + deltas adversariais)

**Decisões ratificadas pelo Enio (2026-05-23):**
1. Escopo: v2 completo (7 Etapas, todas)
2. Rename `ImageEditTool → RasterEditTool`: SIM
3. RAM: 2-3 slots simultâneos (sem upgrade)
4. Pre-flight: push já feito
5. 2 Coords paralelos: SIM
6. CI paralelo + merge-on-green: SIM

**Cronograma realista pós-correções adversariais: 11-13 semanas (não 10).** RAM 8 GiB limita 2 paralelos efetivos em batches; estimativas v2 eram otimistas em 2-3× para Tier 1.3 (tool-runtime 800-1200 LOC, não 200-300), Tier 5.4 (panel-canonical-template AST = standalone 1-2 sem), Tier 1.4 (apagar shell tem ripple em callers).

---

## I. EMENDA ADR-0040 CONSOLIDADA (single commit, ~6-8h, Coord-A)

**Mudanças no `crates/ph2d-editor-core/src/tool.rs`:**

> **Correção pós-implementação (Etapa 1.A executada 2026-05-23, commit `a03d830`):** este bloco originalmente prometia `RasterFrame { pixels: Arc<[u8]>, layout: PixelLayout }` typed wrapper + `Tool` cap 10→11. A implementação real (e o ADR-0041 §2.4) **mantém assinaturas crus** (`Vec<u8>` / `(&[u8], u32, u32)`) porque os tipos típicos esperam o crate `ph2d-color` (Etapa 5). Cap `Tool` permanece em **10** (o método `as_image_edit_mut` foi renomeado para `as_raster_edit_mut`, mesmo slot). Cap `RasterEditTool` sobe de 4 → 5 (adicionando `deactivate`). Verdade canônica está em [ADR-0041](../architecture/decisions/0041-rasteredit-rename-and-deactivate.md).

```rust
// Renomeado de ImageEditTool → RasterEditTool (ADR-0041)
// 🔒 FREEZE ADR-0040 TG-E + ADR-0041

pub trait Tool {
    // ... 9 métodos atuais mantidos + as_raster_edit_mut (renomeado de
    //     as_image_edit_mut, slot 9 — cap permanece 10)
    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> { None }
}

pub trait RasterEditTool: Tool {
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32);
    /// Drains o dirty flag interno; retorna slice do buffer cacheado
    /// dentro do tool se houve mudança. `None` se nada novo.
    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)>;
    fn take_pending_commit(&mut self) -> bool;
    fn run_full(&mut self) -> (Vec<u8>, u32, u32);
    fn deactivate(&mut self);  // NOVO — lifecycle hook (ADR-0041)
}
```

**Caps reais pós-Etapa 1.A:**
- `Tool` = 10 métodos (rename de slot 9, sem bump)
- `RasterEditTool` = 5 métodos (4 antigos + `deactivate`)
- `PanelEvent` = 4 (sem mudança nesta wave)

**`RasterFrame` typed wrapper / `Arc<[u8]>` interno / `PixelLayout`** ⇒ DIFERIDOS para Etapa 5 quando `ph2d-color` chegar. ADR-0041 §2.4 documenta a justificativa.

**Rename ripple:** 17 hits em 6 arquivos (verificado via `grep -rn ImageEditTool`):
- `crates/ph2d-editor-core/src/tool.rs` (definição + doctest)
- `crates/ph2d-editor-core/tests/architecture_cycle_prevention.rs`
- `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` (string-needle)
- `crates/ph2d-tool-equalize-sizes/src/tool.rs`
- `crates/ph2d-tool-color-equalization/src/tool.rs` + `manifest.rs`
- DIRETRIZ §3.8.3.1 (reescrita pra remover heads-up legacy)

**Smoke da emenda:** `cargo test --workspace` + arch-gate verde + DIRETRIZ atualizada commitada junto.

---

## II. As 7 Etapas (com deltas críticos incorporados)

### ETAPA 0 — Infraestrutura completa (3-4 dias, Coord-A + Coord-B setup)

**0.1 `scripts/slot-env.sh`** — `CARGO_TARGET_DIR=$PROJ/target/slot-$SLOT_ID`. Coord-A em SSD local (124 GiB livre); demais em MAC_EXTERNO (1.8 TB). Hook via `source` no início de cada sessão Claude Code, documentado em CLAUDE.md projeto-local (NÃO em settings global).

**0.2 `scripts/git-stage-guard.sh`** — pre-commit guard:
- `git diff --cached --name-only` mostra fora da pasta do slot ⇒ alerta
- `git add -A`/`-a` ⇒ aborta
- Edge cases: ~80 LOC (não 30, corrigindo adversarial B)
- Bypass declarado para Coords (env `COORD_OVERRIDE=1`)

**0.3 Política "2 Coords" (decisão #5 Enio) — amendment DIRETRIZ §1.1:**
- Coord-A: foundational (contratos, ADRs, arch-gates, tool-runtime crate, render_loop apaga, ph2d-color crate)
- Coord-B: baldes (painel/widget/chrome scaffolds, sweep panel-*, smoke organization, gate-meta UI_Bugs)
- **Protocolo de sincronização explícito:** ambos comitam locais; Coord-A faz merge final por jornada. Coord-B notifica Coord-A no início de sessão sobre arquivos que vai tocar (via `docs/SESSION_ACTIVE.md` editado por cada Coord).
- Ship-de-jornada continua serial em Coord-A.

**0.4 CI paralelo feature-branch (decisão #6 Enio):**
- Reescrever `.github/workflows/spike.yml`: jobs caros (matrix 3-OS + replay) só em `refs/heads/main`; jobs leves (`cargo check --workspace` + arch-tests + clippy) em qualquer `feat/*` branch.
- Habilitar GH merge-queue (exige admin do Enio no repo settings).
- Implementador pusha branch `feat/<slot>-<feature>` → CI subset corre em ~5min.
- Coord-A faz `gh merge queue add` → CI full matrix → auto-merge.
- Amendment DIRETRIZ §7 documentando o novo fluxo.

**0.5 Cap LOC arch-gate budget** — wrapper `tools/arch-gate-time-budget/` que avisa se `cargo test --workspace --tests` > 90s. Detecção, não falha (corrigindo adversarial — gate falhando em 60s pode quebrar perfil de testes lentos legítimos como GPU init).

**0.6 Tag `pre-perfection-2026-05-24`** + confirma push verde origin/main.

**0.7 Arquivar HANDOFF_image_tools_slots_2_3_4.md** → `docs/archive/handoffs-completed/`.

**Pré-flight Dia 1 — checklist obrigatório:**
```
[ ] cargo run -p ph2d-tool-sync && cargo test -p ph2d-tool-registry-init verde
[ ] top -o MEM idle confirma ≥4 GiB RAM livre
[ ] df -h confirmar MAC_EXTERNO ≥1.5 TB e / ≥100 GiB livres
[ ] Enio + Coord-A: 30min revisando docs/plans/2026-05-wave-10-decisoes.md
[ ] Coord-A dedica jornada inteira para 0.1+0.2+0.3+0.4. Sem Etapa 1 no Dia 1.
```

---

### ETAPA 1 — Vertical BgRemoval canônica (1-1.5 semanas, Coord-A + 1 Implementador)

**Sub-etapa 1.A — Emenda + impl BgRemoval (Dias 4-7):**

1. Coord-A: emenda ADR-0040 §7 (Seção I acima). Smoke isolado da emenda. Commit `emenda-adr-0040-w10`.
2. Coord-A inicia `crates/ph2d-tool-runtime/` em paralelo (esqueleto, ~200 LOC, sem usar ainda):
   - `pub fn drive_active_raster_edit_tool(tools, sim, scene, asset_db, ...) -> RuntimeReport`
   - Loop genérico set_source/current_preview/take_pending_commit/run_full
   - Cap LOC `architecture_runtime_loc_cap` ≤500 ativo desde dia 1 (corrigindo RA2 do adversarial A)
3. Implementador (Coord-A-dirigido) faz BgRemoval:
   - `impl RasterEditTool for BgRemovalTool`
   - Migra cache de preview de fields em `app_state.rs` → `Arc<[u8]>` interno do tool
   - Mover NodeIds: `editor-core/src/ids.rs` → `crates/ph2d-tool-bgremoval/src/ids.rs` + `crates/ph2d-panel-bgremoval/src/ids.rs`
   - Mover strings i18n: `ph2d-i18n/src/lib.rs::tr` BgRemoval → `crates/ph2d-tool-bgremoval/src/strings.rs`
   - Marcadores `// SHELL_DELETE_AFTER_MIGRATION: bgremoval_preview` no `crates/ph2d-tool-bgremoval/src/lib.rs`

**Sub-etapa 1.B — Estender `ph2d-tool-sync` para apagar shell (Dias 8-9, Coord-A):**

4. Coord-A estende `tools/ph2d-tool-sync/`:
   - Detecta `// SHELL_DELETE_AFTER_MIGRATION: <bridge_filename>` em `crates/ph2d-tool-<slug>/src/lib.rs`
   - Verifica que tool implementa `RasterEditTool` (via syn AST parse)
   - Apaga o arquivo bridge correspondente no `shells/desktop/src/render_loop/<bridge>.rs`
   - Substitui blocos no `render_loop/mod.rs` por chamada genérica `tool_runtime::drive_active_raster_edit_tool(...)` (idempotente — só uma chamada para todos)
   - Tests: `tools/ph2d-tool-sync/tests/shell_delete.rs` cobre

5. Implementador integra `tool_runtime::drive_active_raster_edit_tool(...)` no `render_loop/mod.rs`.

**Sub-etapa 1.C — Smoke do Enio (Dia 10):**

Gate da Etapa 2. Verificar:
- BgRemoval preview live ao mover sliders
- Apply produz bake correto
- Eyedropper pick (downcast com exceção documentada ADR-0040 §3 — allowlist explícita no gate da Etapa 3)
- Protect-brush dab (idem)
- Cancel/deactivate limpa estado
- `render_loop/mod.rs` perdeu ≥200 LOC

**Backout plan:**
- Tag `pre-emenda-adr-w10` antes de 1.A
- Tag `pre-bgremoval-vertical-w10` antes do impl
- Tag `pre-tool-runtime-w10` antes da 1.B
- Cada um permite `git reset --hard` cirúrgico

**Delta crítico 1 incorporado:** o shell delete é parte do codegen `tool-sync`, NÃO trabalho serial do Coord-A para cada Implementador. Implementador comita marker → `tool-sync` apaga automaticamente → Coord-A só revisa.

**Custo realista: 10-12 dias úteis (não 5-7 como v2 prometeu).**

---

### ETAPA 2 — Fan-out 4 tools paralelas (1.5-2 semanas, batches de 2)

**Pré-requisito:** Etapa 1 fechada + `docs/MIGRATION_TEMPLATE.md` redigido por Coord-A (1 dia pós-1.C).

**Batches de 2 (RAM 8 GiB limita, decisão #3 Enio):**
- Batch 1 (Semana 3): Padding + Color Equalization
- Batch 2 (Semana 4): Upscale + Equalize Sizes

**Cada Implementador, na pasta exclusiva:**
1. `impl RasterEditTool for <Tool>`
2. Mover ids locais
3. Mover strings locais
4. Verificar setup canônico Widget Gallery em populate (link_slider_number + storage 0..1)
5. Marker `// SHELL_DELETE_AFTER_MIGRATION:<bridge>` em `lib.rs`
6. Pusha branch `feat/<slot>-<tool>` → CI paralelo subset roda → Coord-A revisa diff → merge queue

**Padding caso especial:** flow geométrico (gizmo push, snap content-bbox) — `current_preview()` retorna `None`; `run_full()` aplica bake (pixels não mudam, só dimensões mudam). `tool-runtime` aceita None graciously.

**`render_loop/mod.rs` shrinkage acumulada por tool-sync:** após cada batch, `tool-sync` apaga 2 bridges + 2 blocos activate + 2 blocos Apply-teardown. Coord-A só comita o resultado.

**Smoke do Enio (gate da Etapa 3):** 4 tools funcionais via novo caminho.

**Custo realista: 10-14 dias úteis (2-3 paralelos efetivos em batches de 2).**

---

### ETAPA 3 — Gates de tool + apaga shell legacy (3-5 dias, Coord-A)

**3.1 Gates novos:**
- `arch_no_per_tool_branch_in_render_loop` — regex `last_<tool>_pushed`, `<slug>_preview`, `_apply\\b` em `shells/desktop/src/`
- `arch_tool_has_raster_edit_or_oneshot` — manifest `kind=Stateful` exige `as_raster_edit_mut() = Some` (verificável via AST scan)
- `arch_no_downcast_to_concrete_tool_in_shell` com allowlist EXPLÍCITA:
  ```rust
  const DOWNCAST_ALLOWED: &[&str] = &[
      "shells/desktop/src/input_dispatch/eyedropper.rs",
      "shells/desktop/src/input_dispatch/protect_brush.rs",
      "crates/ph2d-tool-runtime/src/",  // único lugar legítimo
  ];
  ```

**3.2 Apagar via tool-sync (já feito incrementalmente em Etapa 2):**
- Confirmar que `app_state.rs` perdeu 6 fields `<Slug>Preview` + `last_<slug>_pushed_entity`
- Confirmar 5 blocos Apply-teardown + 5 blocos activate removidos
- Confirmar match arms por tool_id em `ActivateTool/OneShotImageOp` substituídos por Registry kind lookup

**3.3 Resultado mensurável:**
- `render_loop/mod.rs`: 1002 → ~650-700 LOC (não promessa de ≤600; honestidade adversarial)
- Se ainda exceder cap HR-18, extrair `render_loop/legacy_drain.rs` na mesma Etapa
- Escape-hatches HR-18: 0

---

### ETAPA 4 — Codegen panel-sync + chrome-sync + widget-sync (1.5-2 semanas, Coord-B + 3 Implementadores)

**Coord-B dirige (decisão #5 — 2 Coords). 3 Implementadores em paralelo entre si (não com Etapa 2/3).**

**4.1 `tools/ph2d-panel-sync`** (Coord-B + Implementador A):
- Varre `crates/ph2d-panel-*`
- Regenera entre marcadores codegen:
  - `panel-registry-init/Cargo.toml` (features `panel-X` + optional deps)
  - `panel-registry-init/src/lib.rs::build_typed_registry` (push ErasedPanel)
  - `shells/desktop/Cargo.toml` (features `panel-X` + optional deps)
- Staleness gate `crates/ph2d-panel-registry-init/tests/staleness.rs`
- Custo: ~300 LOC (espelha tool-sync). 3-4 dias.

**4.2 `tools/ph2d-chrome-sync`** (Coord-B + Implementador B):
- **Refactor PRIMEIRO** (corrigindo adversarial A — ordem é load-bearing, não comutativa):
  - Handlers retornam `ChromeOutcome { handled: bool, side_effects: SmallVec<Effect> }`
  - Cada handler declara prioridade via atributo proc-macro `#[chrome_order(N)]` (NÃO comment marker — adversarial mostrou que comment é frágil)
- Codegen `dispatch_all` respeitando ordem explícita
- Staleness gate
- Custo: ~400 LOC + refactor 13 handlers existentes. 4-5 dias.

**4.3 `tools/ph2d-widget-sync`** (Coord-B + Implementador C):
- Varre `widget/*.rs`
- Codegen `widget/mod.rs`
- Codegen `widget/showcase/mod.rs` agrupando por atributo `#[showcase_group("atomic")]` (proc-macro)
- Staleness gate
- Custo: ~250 LOC. 3 dias.

**Os 3 syncs em paralelo entre si** (crates diferentes, tools/ pasta diferente, sem colisão). Coord-B sincroniza no merge final.

**Resultado:** painel/widget/chrome novos viram fan-out drop-file (igual tool/node). DIRETRIZ §1.2 caminho (B) reduz para apenas foundational genuíno.

---

### ETAPA 5 — Gates UI + classes de bug + ph2d-color (2 semanas, Coord-A + Coord-B + 2 Implementadores)

**Coord-A: ph2d-color crate + gates ortogonais.**
**Coord-B: sweep panel-* + estender scan_roots.**

**5.1 Sweep + estender scan_roots para `crates/ph2d-panel-*/src/**` (Coord-B + Impl X):**

Estender de `editor-core/src/{widget,screens}/` para também `panel-*`:
- `no_literal_color`
- `no_magic_numeric`
- `hr12_widgets_a11y` ← correção adversarial C gap A.4.1
- `hr15_no_hardcoded_ui_strings`

**Protocolo SWEEP PRÉ-ATIVAÇÃO:**
1. Dia 1: estender scan_roots em modo REPORT-ONLY (`PH2D_GATE_REPORT=1` env var). Rodar, capturar TODAS as violations em `docs/plans/wave-10-violations-snapshot.md`. Estimativa: 50-150 violations.
2. Dias 2-5: Coord-B + Impl X fix em paralelo (sweep por panel).
3. Dia 6: validar ZERO violations.
4. Dia 7: ATIVAR gate em deny mode.

**5.2 Cap LOC em `crates/ph2d-panel-*/src/**` (Coord-B):**
- 600 LOC/arquivo (paridade HR-18 — corrigindo v1 que propunha 400; 3 panels já passam de 400)
- 200 LOC/função
- Sweep prévia: extrair 3-5 arquivos que já passam de 600

**5.3 Gates ortogonais (Coord-A + Impl Y):**

| Gate | O que barra | Bug que mata |
|---|---|---|
| `arch_no_absolute_drag_pattern` | regex `event\.[xy] - .*start_[xy]` em `interaction/dispatch/` | Burning 4 CEQ |
| `arch_mode_has_reconcile` | `pub fn set_<X>_mode` exige `reconcile_<X>_state` companion no shell | Image Tools Bug §2 |
| `arch_safe_clamp_only` | proíbe `.clamp(min, max)` literal; força `safe_clamp` em ph2d-core | UI_Bugs §8 |
| `arch_no_char_count_widths` | regex `\.chars\(\)\.count\(\) \*` em widget/screens/panel-* | UI_Bugs §3.3, §9.16, §10.1 |
| `no_tofu_glyphs` ampliado | U+2000–FFFF menos faixa Inter pinned-version + cobre `panel-*`, `tool-*` | UI_Bugs §9.19 |
| `arch_color_space_typed` ← **delta crítico 3** | proíbe `Vec<u8>` literal sem `ImageBuf \| LinearRgba \| Premultiplied` typing em `crates/ph2d-{tool,render,panel}-*/src/**` | UI_Bugs §4 + Image Tools §1 |
| `tests/docs_bugs_have_gates.rs` | toda entrada em UI_Bugs/Image Tools Bugs precisa `**Gate:**` apontando teste OU `gate-deferred:<reason>` revisado mensal | Reincidência geral |

**5.4 Crate `ph2d-color`** (Coord-A):
- Novo crate `crates/ph2d-color/` (~500-800 LOC realista)
- Tipos: `LinearRgba`, `SrgbRgba`, `Premultiplied<T>`, `OklchColor`, `HsvCache`
- Conversões EXPLÍCITAS (`linear.to_srgb()` não implícito)
- `RasterFrame.layout: PixelLayout` da emenda I usa esses tipos
- Migração concentrada nos 5 tools image + ph2d-render + ph2d-tool-runtime
- Custo realista: 1-2 semanas (não 1 — adversarial mostrou que migração atravessa render+tool+panel+shells)

**5.5 `panel-canonical-template` gate AST-aware** (Coord-A, DIFERIDO para Etapa 6):
- Movido para Etapa 6 porque é 1-2 semanas standalone (adversarial A)
- Etapa 5 não inclui — Etapa 6 sim

---

### ETAPA 6 — Golden-image SSIM + drift detection + panel template (2-3 semanas)

**6.1 Golden-image SSIM gate (Coord-A):**
- Renderizar offscreen via Vello headless
- Comparar widgets + painéis contra snapshots PNG em `tests/golden/widget/`, `tests/golden/panel/`
- SSIM ≥ 0.995 (rebaseline manual via flag `--update-baselines` em PR)
- Reduz smoke do Enio em ~70%

**6.2 `panel-canonical-template` AST gate (Coord-B + Implementador):**
- Codegen de `crates/ph2d-panel-canonical/src/__template__.rs` derivado do Speed setup em `pre_populate.rs`
- Gate verifica via `syn` AST que cada `panel-*/src/populate.rs` respeita:
  1. Para cada `(slider_id, chip_id)`, existe `store.link_slider_number(...)`
  2. `slider.value` e `chip.value` são o mesmo símbolo (storage paridade)
  3. Não há `store.set_slider_value` / `store.set_number_value` em `event.rs` (mirror manual = bug)
- Aplicar SÓ a painéis que declaram `paint_slider_with_chip*` (filtro)
- 1-2 semanas standalone

**6.3 LOC trend detector (Coord-A):**
- Registra LOC por arquivo crítico em `metrics/loc-trend.json` cada commit
- Gate falha se >10% growth em 30 dias sem ADR
- Previne god-files emergentes

**6.4 Memory GC tool (Coord-A):**
- `tools/memory-gc/` valida que paths em MEMORY.md ainda existem no código
- Auto-flag entries obsoletas pra revisão

---

### ETAPA 7 — Política escala 10+ agentes + DIRETRIZ amendments (1 semana)

**7.1 Merge-on-green auto-aceite (decisão #6 Enio):**
- GH action: PR com (i) diff só em `crates/ph2d-{node,tool,panel}-<slug>/`, (ii) todos gates verdes, (iii) sem mudança em foundational ⇒ merge automático sem Coord review
- Coord review obrigatório para: foundational, contrato congelado, multi-crate
- Detecção "multi-crate" via `cargo metadata --format-version 1` + script `scripts/auto-merge-eligibility.sh` (fail-safe pra coord-review se ambíguo)

**7.2 Ship paralelo:**
- Etapa 0.4 (CI feature-branch) torna ship não-serial
- Implementador termina → pusha branch → CI subset → merge automático se verde
- Coord ship-de-jornada vira "rebase mass + push main" ao final

**7.3 Auto-squash by crate-path:**
- Script `tools/auto-squash/` agrega commits do mesmo `crates/ph2d-*` em um, mantém commit messages
- Reduz revisão Coord

**7.4 DIRETRIZ §1.4 reescrita pós-Etapa 4:**
- Triagem vira tabela executável `tools/ph2d-triagem` que lê path tocado, sugere caminho
- Caminho (A) drop-crate cobre 95% dos casos (tool/node/panel/widget/chrome — todos fan-out via sync)
- Caminho (B) só pra foundational genuíno
- Caminho (C) só pra contrato congelado

**7.5 DIRETRIZ §3.8.3.1 reescrita** — pós-rename: "RasterEditTool é o canal canônico desde Wave 10. Eyedropper/protect-brush exceção via downcast com allowlist."

**7.6 Documentação final:**
- ADR-0041 Wave 10 closure (cita commits-âncora, decisões tomadas, próximos passos)
- SKILL_Stack §11.9 atualizada
- HANDOFF_node_system §protocolo de fan-out atualizado

---

## III. DAG explícito + cronograma realista

```
Semana 1:  Etapa 0 (infra + 2 Coords setup + CI paralelo + emenda ADR redigida)
Semana 2:  Etapa 1.A (BgRemoval impl + tool-runtime skeleton)
Semana 3:  Etapa 1.B + 1.C (extend tool-sync + smoke BgRemoval)
Semana 4-5: Etapa 2 (4 tools em batches de 2)
Semana 6:  Etapa 3 (gates + apaga shell)
Semana 7-8: Etapa 4 (3 syncs paralelos: panel-sync + chrome-sync + widget-sync)
Semana 9-10: Etapa 5 (sweep panel-* + gates ortogonais + ph2d-color)
Semana 11-12: Etapa 6 (golden-image + panel-template AST + LOC trend)
Semana 13: Etapa 7 (política CI/merge + DIRETRIZ amendments + closure ADR)
```

**Total: 13 semanas calendar (não 10 como v2 prometeu).**

Realismo: 8 GiB RAM + 2-3 paralelos efetivos + smokes do Enio gating + Coord-A + Coord-B trabalhando em paralelo mas precisando sync nos merges = 13 semanas.

---

## IV. Métricas-alvo finais

| Métrica | Hoje | Pós Etapa 3 | Pós Etapa 5 | Pós Etapa 7 |
|---|---|---|---|---|
| Maior LOC em `shells/desktop/` | 1002 (escape) | ≤700 | ≤700 | ≤600 |
| Escape-hatches HR-18 ativos | 2 | 0 | 0 | 0 |
| Downcasts `<ConcreteTool>` no shell | 33 | ≤2 (allowlist) | ≤2 | ≤2 |
| Tools sabor (3): edits centrais | 12-16 arquivos | 1-2 | 0 | 0 |
| Painéis novos: edits centrais | 4-5 | 4-5 (E4 fixa) | 0 (codegen panel-sync) | 0 |
| Bugs UI por classe sem gate | 11+ | 11+ (não tocado) | 1-2 (color/hit-test parciais) | 0-1 |
| Cargo lock waits entre sessões | manual | auto via slot-env | auto | auto |
| Coord ships/jornada | 1 (serial) | 1 | 1 | parallel via merge-on-green |
| Agentes simultâneos saudáveis | 2-3 | 2-3 (RAM) | 2-3 | 2-3 hoje; 8+ se RAM upgrade |
| Smoke do Enio: tempo médio | 30min | 30min | 30min | 10min (golden-image cobre 70%) |

---

## V. Os 3 deltas críticos dos adversariais (incorporados)

**Delta 1 — Shell delete via tool-sync codegen (NÃO Coord-A serial):**
Incorporado em Sub-etapa 1.B. Cada `crates/ph2d-tool-<slug>/` declara `// SHELL_DELETE_AFTER_MIGRATION:<bridge>` marker; `tool-sync` apaga automaticamente quando tool impl `RasterEditTool`. Coord-A só revisa diff. **Não há gargalo serial de deletes.**

**Delta 2 — Não renomear `ImageEditTool`:** 
~~Diferido~~ → **Cancelado pela decisão #2 do Enio (renomeia agora pra RasterEditTool).** Custo absorvido na emenda ADR.

**Delta 3 — Gate `arch_color_space_typed` adicional ao typing:**
Incorporado em Etapa 5.3. Typing por si só (`ph2d-color`) é insuficiente — `Vec<u8>` cru escapa. Gate regex bane `Vec<u8>` sem typing wrapper em `crates/ph2d-{tool,render,panel}-*/src/**`.

---

## VI. Riscos remanescentes + mitigações

**R1 — Smoke do Enio é serial gargalo (8 smokes na wave).**
Mitigação: Etapa 6 golden-image gate reduz 70%. Tabelas explícitas de "o que verificar" em cada smoke.

**R2 — `tool-runtime` cap LOC ≤500 pode estourar.**
Mitigação: cap explícito desde dia 1 + `arch_runtime_loc_cap` gate. Se estourar legítimo, refactor em sub-modules antes do cap-bust.

**R3 — Sweep panel-* (Etapa 5.1) pode ter 50-150 violations.**
Mitigação: modo REPORT-ONLY primeiro + 2 trabalhadores em paralelo (Coord-B + Impl X) + estimativa concreta antes de ativar deny.

**R4 — CI paralelo feature-branch quebra fluxo atual.**
Mitigação: Etapa 0.4 é trabalho explícito de Coord-A com smoke próprio antes de Etapa 1 começar.

**R5 — 2 Coords pisam um no outro em foundational.**
Mitigação: `docs/SESSION_ACTIVE.md` editado por cada Coord antes de tocar foundational. Protocolo de sincronização explícito em Etapa 0.3.

**R6 — Rename `RasterEditTool` causa ripple em sessões em vôo.**
Mitigação: rename é parte da emenda ADR, single commit, antes de qualquer outra mudança Wave 10.

**R7 — Auto-merge on green com falso positivo.**
Mitigação: script `auto-merge-eligibility.sh` fail-safe pra coord-review se ambíguo (multi-crate, foundational, contrato).

---

## VII. Decisões politicamente sensíveis (já ratificadas pelo Enio)

| Decisão | Status |
|---|---|
| 2 Coords | ✅ SIM (decisão #5) |
| Rename ImageEditTool → RasterEditTool | ✅ SIM (decisão #2) |
| Escopo v2 completo (7 Etapas) | ✅ SIM (decisão #1) |
| CI paralelo + merge-on-green | ✅ SIM (decisão #6) |
| RAM atual (2-3 paralelos) | ✅ Aceito (decisão #3) |
| Pre-flight push | ✅ Feito (decisão #4) |

---

## VIII. Compatibilidade com norte node-centric (ADR-0030..0039)

- `RasterEditTool` (renomeado) abre espaço pra `VectorEditTool`, `NodeEmitTool`, `PhysicsEditTool` futuros sem cap-bust.
- `tool-runtime` opera fora do `Render::Cook` phase — boundary explícito documentado em ADR-0041 (closure da Wave 10).
- `ph2d-color` crate compartilhado entre tool/node/render/panel — substrato comum.
- Codegens (tool-sync, node-sync, panel-sync, chrome-sync, widget-sync) — família simétrica simétrica, fan-out idêntico nas 5 famílias.

---

## IX. Plano de execução — Dia 1 concreto

```
Dia 1 manhã (Coord-A em sessão dedicada):
  1. Source scripts/slot-env.sh (criar primeiro)
  2. Implementar 0.1 (slot-env.sh) — 1h
  3. Implementar 0.2 (git-stage-guard.sh) — 1h
  4. Tag pre-perfection-2026-05-24 + confirmar push
  
Dia 1 tarde (Coord-A + Enio 30min):
  5. Revisar v4 + ratificar
  6. Coord-A inicia amendment DIRETRIZ §1.1 (2 Coords) + §7 (CI paralelo)
  
Dia 2 (Coord-A):
  7. Reescrever .github/workflows/spike.yml para 0.4 CI paralelo
  8. Habilitar GH merge-queue (precisa admin do Enio — 5min)
  
Dia 3 (Coord-A + Coord-B se aberto):
  9. 0.3 protocolo 2 Coords documentado
  10. 0.5 cap LOC arch-gate budget
  11. 0.7 arquivar HANDOFF_image_tools_slots_2_3_4
  12. Smoke Etapa 0 completa
  
Dia 4: Etapa 1.A começa.
```

---

**Esta é a versão DEFINITIVA da proposta. Padrão ouro, escopo máximo, deltas adversariais incorporados, decisões do Enio ratificadas, riscos mitigados, cronograma realista de 13 semanas calendar.**

