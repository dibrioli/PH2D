# Plano de Blindagem da Implementação — PH2D

**Aberto:** 2026-06-20 (Enio) · **Dono:** Coordenador · **Status:** Fase 0 em curso

> Diagnóstico forense (2026-06-20, 5 frentes paralelas) concluiu: **todo o aparato de
> qualidade mede correção ESTRUTURAL (compila? contrato congelado? arquivo < N LOC? deps
> acíclicas?) e quase nada mede correção COMPORTAMENTAL (o botão dispara? a ferramenta faz
> o que promete?).** Pior, "compile-green" foi treinado como sinal de sucesso (inner loop =
> só `cargo check`) e a definição de "pronto"/"auditoria" herdou isso. Todos os sintomas
> (3 ciclos por botão, god-files, manutenção quebradiça, auditoria que só compila) saem dessa
> única assimetria.

**Princípio:** cada fase converte uma *disciplina em prosa* numa *gate executável* e move o
sinal de "pronto" de compila-verde para **comporta-verde**. Cada fase segue a DIRETIVA §5:
conjunto de aceitação concreto + kill-criterion **antes** do build.

**Ordem inegociável:** 0 → 1 → (2 ∥ 3) → 4. A Fase 3 (decompor god-files) NÃO começa antes da
Fase 1, porque os comentários das gates atuais provam que *split às cegas de paint sem cobertura
causa regressão visual que nenhuma gate pega* (`architecture_panel_loc_cap.rs:90,107`).

---

## Evidências-âncora (do diagnóstico)

| Queixa | Causa-raiz com file:line |
|---|---|
| "3 ciclos por botão" | Um slider+chip toca **13 sites em 3 crates, 100% manuais, 0 codegen** (rastreado em `padding`). As 2 gates que a DIRETIVA manda confiar (`architecture_studio_slider_wiring`/`_cycler_wiring`) **não existem** (eram do brush studio, deletadas). Nenhuma gate verifica populate↔event. |
| Auditoria cega | 3.354 `#[test]` mas ~2 harnesses comportamentais; o do painter foi deletado. Gates `*_contract_surface` contam `.matches("fn ")`/`.matches("pub ")` — ABI, nunca comportamento. |
| God-files | 6 arquivos >1000 LOC; cap de LOC cobre <15% das crates (só painel/widget/shell/1 helper); núcleo (`render`, `editor-core`, `effects`, 20 tools) sem cap. |
| Docs enganam | 418 `.md`, 89k LOC; DIRETIVA referencia gates fantasma. |

---

## Fase 0 — Fundação: tornar o silêncio impossível (barato, retroativo)

| # | Entrega | Caminho | Gate nova |
|---|---|---|---|
| 0.1 | Testkit comportamental headless | `crates/ph2d-ui-testkit/` (`MockPanelHost` impl `PanelHostInternal` sobre `WidgetStore`+`ActionBus` reais + helpers + teste-exemplo no padding) | — (habilita 1.x) |
| 0.2 | Paridade de wiring por painel | `crates/ph2d-editor-core/tests/architecture_panel_wiring_parity.rs` (generaliza `architecture_topbar_registration_parity`) | id **hit-indexado no paint ⟹ registrado** em `populate.rs`/global (focável) |
| 0.3 | Docs não citam gate morta | `crates/ph2d-editor-core/tests/architecture_docs_reference_live_gates.rs` | token `architecture_*` em doc instrucional ⟹ existe teste real |
| 0.4 | Cap LOC por-arquivo, workspace | `crates/ph2d-editor-core/tests/architecture_workspace_file_loc_cap.rs` | todo `.rs` de `crates/` ≤ 600 LOC (allowlist = dívida congelada) |

> **Registro de decisão (0.2):** a 1ª versão checava `populate ⟹ event.rs`. O **kill-criterion
> do plano disparou** (>30% falso-positivo: dispatch via tabela-de-mapeamento indireta no grid-snap,
> handling multi-arquivo no inspector, chrome genérico no editor-core). Pivotei — como o plano mandava —
> para **hit↔register** (modelo do topbar, provado): zero falso-positivo por construção. A classe
> "evento despachado mas dropado no `_ => false`" passa a ser coberta pelo **teste comportamental de
> seam (0.1)**, que segue a indireção. Divisão limpa: gate estático = focabilidade; teste comportamental = dispatch.

- **Nomenclatura:** o testkit NÃO pode ser `ph2d-panel-*`/`ph2d-tool-*`/`ph2d-node-*` (seria
  coletado por codegen/cap — [node-sync glob gotcha]). Nome: `ph2d-ui-testkit`.
- **Posição do teste de seam:** no próprio `ph2d-panel-*` (já pode depender da sua tool); o
  testkit entra como `[dev-dependencies]` — permitido pela `architecture_cycle_prevention`
  (lê só `[dependencies]`, docstring linhas 127-129).
- **Aceitação Fase 0:** removo 1 arm de `event.rs` → 0.2 vermelha; renomeio uma gate citada
  no doc → 0.3 vermelha; teste de seam do padding prova `populate → ValueChanged → apply_event
  → bus → handle_panel_event → tool.spec() mudou`.
- **Kill-criterion 0.2:** se gerar >30% falso-positivo, vira allowlist explícita em vez de heurística.
- **Cap por-função:** adiado para a Fase 3 (depende do fix do parser comment-aware).

## Fase 1 — "Comporta-verde = pronto" vira lei

- **1.1** Backfill: cada `ph2d-tool-*` com `handle_panel_event` e `ph2d-panel-*` com
  `apply_event` ganha ≥1 teste de seam via testkit.
- **1.2** Gate `architecture_interactive_crate_has_behavioral_test` (allowlist = dívida → zero).
- **1.3** DoD + contrato de auditoria: DIRETIVA §3 vira **template de output obrigatório**
  (por claim: trace file:line + asserção-que-fica-vermelha + LOC lidas). Addendum à doutrina
  do inner loop: **check-verde é velocidade, nunca é done.**

## Fase 2 — Codegen do widget (a correção definitiva; só após Fase 1)

- Macro `panel_widget! { id, kind, edit, project }` emite os 13 sites de 1 declaração.
- **Kill-criterion (antes de codar):** se não cobrir ≥80% dos widgets reais (2D-livres como
  curve editor/XY pad são exceção conhecida), paramos e ficamos com a gate 0.2 como blindagem.

## Fase 3 — Estancar a podridão (atrás da Fase 1)

- 3.1 fix parser comment-aware → liga cap por-função workspace.
- 3.2 decompor os 6 god-files >1000 LOC (compositor 2396, compute 1636, pointer 1261, atlas
  1159, layers 1148, chrome 1027) — com a rede comportamental da Fase 1.
- 3.3 congelar allowlist de ~30 `downcast_mut` para tools concretas no shell (sem nova exceção
  sem ADR).

## Fase 4 — Docs que não enganam (contínuo)

- Reescrever DIRETIVA §2 ("como adicionar widget") apontando à macro/gate; reconciliar DIRETIVA/
  DIRETRIZ contra a realidade; arquivar agressivamente.

---

## Status

- [x] **0.1** `ph2d-ui-testkit` (`MockPanelHost`) + seam test no padding — **2 testes verdes**
- [x] **0.2** `architecture_panel_wiring_parity` (hit↔register; 3 ids dinâmicos allowlistados c/ mecanismo verificado) — **verde**
- [x] **0.3** `architecture_docs_reference_live_gates` — caçou 4 refs mortas reais (DIRETIVA/DIRETRIZ/CLAUDE corrigidos) — **verde**
- [x] **0.4** `architecture_workspace_file_loc_cap` (53 offenders congelados em baseline 2026-06-20 + guard de stale) — **verde**
- [x] **Fase 1** — comporta-verde vira lei:
  - [x] 1.2 `architecture_interactive_crate_has_behavioral_test` (12 painéis interativos; 6 cobertos, 6 em `BEHAVIORAL_TEST_DEBT` drive-to-zero + guard anti-stale) — **verde**
  - [x] 1.1 backfill seam tests: bgremoval, color-equalization, upscale, equalize-sizes, grid-snap (+ padding) = **6 painéis, 13 testes verdes** (cada um dirige evento real → afirma efeito observável no tool/state, com guard anti-vacuidade)
  - [x] 1.3 DoD + **rubrica executável de auditoria** na DIRETIVA §3 (template obrigatório: LENTE/CLAIM/TRAÇO/ASSERÇÃO-VERMELHA/LOC) + §5 (DoD = seam verde + smoke; compile/gate-verde nunca é done)
  - [ ] **Dívida Fase 1 (drive-to-zero):** seam tests para inspector, hierarchy, painter-layers, vector-graph, vector-inspector, widget-gallery (genéricos/multi-arquivo — backfill incremental; a gate trava regressão)
- [ ] **Fase 2 / 3 / 4** (não iniciadas)

[node-sync glob gotcha]: ../../CLAUDE.md
