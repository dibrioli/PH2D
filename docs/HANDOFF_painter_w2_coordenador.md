# HANDOFF — Painter W2 → **COORDENADOR NOVO** (modelo 1 Coord + 3 Implementadores)

**Data:** 2026-05-28
**De:** Coord+Implementador da sessão T2.1 (auditoria 4-lente + remediação).
**Para:** o **novo Coordenador único** que vai orquestrar 3 implementadores em paralelo.
**Leitura obrigatória antes de agir:** [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md) (v7.0 — papéis, 3 caminhos A/B/C, anti-colisão, ship).

> **Este documento é PRA VOCÊ, Coordenador.** No fim (§6) há um template do
> handoff DESCENDENTE que você deve escrever pro implementador do Painter,
> pra ele continuar de onde paramos (T2.5).

---

## §0. Estado atual do módulo Painter (verdade do git, não da memória)

- **Última entrega:** commit local **`c82293c`** — `fix(painter-sidebar): T2.1 audit W/X/Y/Z remediation`.
  - 9 arquivos, +362/−76. Auditoria adversarial 4-lente (W widget-canon / X shell-coupling / Y state-machine / Z determinism) executada; **todos CRITICAL+HIGH+MEDIUM remediados in-code**; LOW + tarefas-futuras → [`docs/plans/2026-05-wave-11-carry-overs.md` §Painter T2.1](plans/2026-05-wave-11-carry-overs.md).
- **Commits são LOCAIS.** ~80 ahead de `origin/main`, **zero push** (fast-mode). Push/CI = sua decisão de fim-de-jornada (DIRETRIZ §8), uma vez só.
- **Smoke Enio aprovado em T2.1:** stroke aparece, sidebar takeover, Size/Opacity sliders real-time, drag/resize/close.

### O que a remediação T2.1 fechou (pra você não re-auditar)
- 🔴 Size/Opacity chips agora `link_slider_number_mapped_integer` (eram identity → split-brain digitar-clampa-no-max).
- `PainterUiSnapshot::Default` espelha `PainterParams::default()` (era all-zero → brush size-0 no 1º frame).
- Helpers SSOT `size01_to_px`/`px_to_size01`/`size_chip_mapping`/`opacity01_to_pct`/`opacity_chip_mapping` em [`params.rs`](../crates/ph2d-tool-painter/src/params.rs) — `tool.rs`/`paint.rs`/`populate.rs` todos chamam (magic `2048` não duplicado).
- Layout adaptive (iPad portrait).
- Roteamento morto undo/redo/modifier REMOVIDO (volta com paint em T2.2/T2.4).
- Opacity refresh mid-stroke.
- **Gate novo:** [`crates/ph2d-panel-painter-sidebar/tests/populate_mapped_link.rs`](../crates/ph2d-panel-painter-sidebar/tests/populate_mapped_link.rs) (3 tests) + 3 tests affine/default em `params.rs`.

---

## §1. ⚠️ INCIDENTES DE COLISÃO DESTA SESSÃO — leia antes de distribuir trabalho

Foi exatamente por isso que o Enio está centralizando em 1 Coord. Dois problemas reais:

1. **4 testes falhando em [`crates/ph2d-tool-painter/tests/history_integration_t19.rs`](../crates/ph2d-tool-painter/tests/history_integration_t19.rs) NÃO são do Painter.**
   Provado via stash escopado: falham idênticos (`samples 1 vs 2`, tilt index OOB, recovery `1 vs 0`) mesmo com o WIP do Painter revertido. Origem = **WIP não-commitado de outro implementador** em `crates/ph2d-editor-core/src/interaction/dispatch/{number_input,tick}.rs` (mudou geração de samples consumida por esses testes T1.9). **Ação sua:** identificar o dono do dispatch, fechar/commitar ou reverter esse WIP, e re-rodar — senão qualquer ship do Painter trava no hook/CI por culpa alheia.

2. **`git stash` é VENENO em sessão multi-agente com índice compartilhado sujo.**
   Meu `git stash pop` (escopado às minhas 2 pastas) injetou conflict markers em `crates/ph2d-asset-ktx2/src/lib.rs` (arquivo de OUTRO agente) porque o HEAD moveu entre push e pop. Restaurei sem perda, mas **proíba stash entre os 3 implementadores.** Regra: pra isolar "é minha mudança ou de outro?", raciocinar estaticamente (quais paths toquei vs o que o teste exercita), nunca stash. Detalhe em [`memory/feedback_git_stash_multiagent_danger.md`].

**Disciplina anti-colisão que VOCÊ deve impor aos 3 (DIRETRIZ §7 + §1.2/§1.3):**
- Cada implementador roda `source scripts/slot-env.sh <impl-N>` no início (target isolado — RAM 8 GiB ⇒ máx 2-3 cargo simultâneos).
- **NUNCA** `git add -A`/`-a`/`.`. Sempre `git add -- <só meus paths>` + `git commit -m "msg" -- <só meus paths>` (`-m` ANTES do `--`).
- `git status` antes de qualquer stage; se houver `M`/`??` alheio, NÃO commitar — reportar a você.
- Conferir `git diff --cached --name-only` antes do commit (índice compartilhado vaza arquivo alheio — aconteceu comigo com `asset-ktx2`).

---

## §2. Próximas tasks Painter — triagem de CAMINHO (pra distribuir sem colisão)

Ordem recomendada (do handoff T2.1 §4) + classificação A/B/C pra você saber **quem pode pegar o quê em paralelo**:

| # | Task | Caminho (DIRETRIZ §2) | Toca arquivo central? | Paraleliza? |
|---|------|----------------------|------------------------|-------------|
| **T2.5** | **Commit-to-sprite** (`request_commit()` via `on_deactivate` + keybind Cmd+Enter) | **(A) modificar** `ph2d-tool-painter` + **(C) shell** `painter_bridge.rs`/keybind | **SIM** (shell foundational) | ⚠️ shell parte é **Coord-A only** |
| T2.3 | Color picker wire (BlenderColorPicker popover na topbar → primary color) | **(A)** tool + **(C)** topbar/shell wire (BlenderColorPicker já existe em editor-core) | Provável (topbar/shell) | parcial |
| T2.4 | Modifier square (eyedropper-while-held) + re-add paint do botão | **(A)** dentro de `ph2d-panel-painter-sidebar` + `ph2d-tool-painter` | Não (só minhas 2 pastas) | ✅ isolado |
| T2.2 | Undo/redo replay engine + re-add buttons | **(A)** `ph2d-tool-painter` (+ `ph2d-painter-stroke`?) | Talvez stroke crate | parcial |
| T2.6 | A11y nodes (gate `hr12_widgets_a11y`) sliders/color/modifier | **(A)** `ph2d-panel-painter-sidebar` | Não | ✅ isolado |
| T2.7 | Smoke W2 + audit final | — | — | fecha a wave |

**Recomendação de distribuição:** o módulo Painter tem **2 pastas isoladas** (`crates/ph2d-tool-painter/` + `crates/ph2d-panel-painter-sidebar/`). Para evitar colisão, **um único implementador deve segurar o Painter** (T2.5→T2.7 em série dentro dessas pastas) enquanto os outros 2 implementadores trabalham em **módulos disjuntos** (Vector / Sprite Inspector / asset-cooker / dispatch). Se você quiser paralelizar DENTRO do Painter, só T2.4 e T2.6 são seguramente isolados — mas o ganho é pequeno e o risco de pisar em `tool.rs`/`paint.rs` é alto. **Não recomendo 2 implementadores na mesma pasta.**

**Pontos que exigem VOCÊ (Coord-A), não delegáveis:**
- T2.5 mexe em `shells/desktop/src/render_loop/painter_bridge.rs` + keybind (foundational/shell = caminho C). O implementador faz a parte do tool crate (`request_commit()` + `take_pending_commit`); **você** faz o wire no shell.
- Cap `PainterUiEdit ≤ 24` / `PanelEvent ≤ 4` são contrato congelado (ADR-0043 / ADR-0040). Nenhuma task abaixo precisa bumpar — se precisar, é (C)+ADR e passa por você.

---

## §3. Surfaces disponíveis (o que já existe pro implementador reusar)

**`crates/ph2d-tool-painter`** (`PainterTool`):
- `ui_snapshot() -> PainterUiSnapshot` (projeção read-only, ADR-0043 §2.3).
- `apply_ui_edit(PainterUiEdit)` — single source of truth de clamps/maps. Já trata Size/Opacity/SetColor/Toggle*/Undo/Redo/Reset/Symmetry.
- `handle_panel_event(PanelEvent)` — hoje roteia **só** `*_SIZE_SLIDER`/`*_OPACITY_SLIDER` (botões removidos; T2.2/T2.4 re-adicionam).
- Helpers SSOT públicos: `size01_to_px`, `px_to_size01`, `size_chip_mapping`, `opacity01_to_pct`, `opacity_chip_mapping`.
- `impl RasterEditTool` (5 métodos congelados ADR-0041): `set_source`/`current_preview`/`take_pending_commit`/`run_full`/`deactivate`. **T2.5 usa `take_pending_commit` + `on_deactivate`.**

**`crates/ph2d-panel-painter-sidebar`** (`PainterSidebarPanel`):
- typed Panel (ADR-0029). `populate`/`paint`/`apply_event`/`state`.
- `set_current_painter_snapshot(Option<PainterUiSnapshot>)` — shell publica por frame.
- NodeIds canon em `ph2d_editor_core::ids::PAINTER_SIDEBAR_*` (incl. `UNDO_BUTTON`/`REDO_BUTTON`/`MODIFIER_SQUARE` re-exportados em `ids.rs` — prontos pra T2.2/T2.4 re-pintar).
- Chrome completo (surface + corner dots BR/BL + title + close + drag + 2 resize).

**Shell:** `shells/desktop/src/render_loop/painter_bridge.rs` (espelha `bgremoval_preview.rs`); feature `panel-painter-sidebar` no default.

**Doc canônico:** [`docs/Painter_projeto/15_plano_de_implementacao.md` §5 (W2)](Painter_projeto/15_plano_de_implementacao.md).

---

## §4. Gates + validação (o implementador roda; você confere no ship)

```bash
source scripts/slot-env.sh impl-<N>          # SEMPRE no início (target isolado)
cargo check -p ph2d-tool-painter -p ph2d-panel-painter-sidebar
cargo test  -p ph2d-tool-painter -p ph2d-panel-painter-sidebar   # NOTA: 4 falhas em
   # history_integration_t19.rs são ALHEIAS (§1.1) até o dispatch WIP fechar
cargo clippy -p ... --all-targets -- -D warnings
```
Gates relevantes: `architecture_panel_chip_pill_no_stepper`, `hr12_widgets_a11y` (T2.6), `no_literal_color`/`no_magic_numeric`, `architecture_painter_contract_surface` (caps congelados). **Antes do push:** `./scripts/ship.sh` (paridade-CI, DIRETRIZ §8.1).

---

## §5. Mandato §0 do plano (não-negociável, repassar ao implementador)

[`15_plano_de_implementacao.md` §0](Painter_projeto/15_plano_de_implementacao.md) — **padrão-ouro absoluto, sem gambiarras / sem deferral aceitável** ([`memory/feedback_perfection_no_deferrals.md`]). Toda task fecha com ≥2 auditorias paralelas (lentes rotacionadas, [`memory/feedback_audit_lens_diversity.md`]) → findings remediadas → re-audit erro-zero. UI strings em inglês ([`memory/feedback_app_ui_english_only.md`]).

---

## §6. 📋 TEMPLATE — handoff DESCENDENTE que VOCÊ (Coord) escreve pro implementador Painter

Copie, preencha os `<...>` e entregue ao implementador que pegar o Painter:

```
═══════════════════════════════════════════════════════════════════
BRIEFING — Implementador Painter · continua de T2.1 (commit c82293c)
═══════════════════════════════════════════════════════════════════

PASTAS EXCLUSIVAS SUAS (edite SÓ aqui):
  crates/ph2d-tool-painter/
  crates/ph2d-panel-painter-sidebar/
Precisa de algo no shell (painter_bridge.rs, keybind) OU em editor-core
(BlenderColorPicker, ids novos) OU contrato congelado? PARE e me reporte
(Coord) — eu faço a parte central. Você NÃO toca arquivo fora dessas 2 pastas.

ANTES DE CODAR:
  1. source scripts/slot-env.sh impl-<N>     # target isolado
  2. git log --oneline -3                     # confirme c82293c na história
  3. git status                               # se houver M/?? alheio, me avise
  4. Leia docs/HANDOFF_painter_w2_coordenador.md §0–§3 (estado + surfaces)

SUA TASK: T2.5 — commit-to-sprite
  - No tool crate: garanta `request_commit()` enfileira pending commit;
    `take_pending_commit` + `on_deactivate` disparam o commit do stroke
    buffer pro Sprite ativo (carry-over R3-LE-4 do T1.5).
  - Keybind Cmd+Enter → request_commit: a parte do KEYBIND/shell é MINHA
    (Coord) — você expõe o método público, eu faço o wire em
    painter_bridge.rs/input. Me entregue a assinatura quando pronto.
  - DoD: ativar Painter → desenhar → trocar de tool (ou Cmd+Enter) → o
    stroke "cola" no sprite (não some). Smoke do Enio confirma.

DISCIPLINA GIT (índice compartilhado entre 3 implementadores):
  - NUNCA git add -A / git stash. (stash injetou conflito em arquivo
    alheio nesta sessão — proibido.)
  - git add -- <só seus arquivos>  ;  git commit -m "msg" -- <seus paths>
  - git diff --cached --name-only ANTES do commit (cheque vazamento).
  - Commits LOCAIS, sem push (eu faço o ship no fim).

VALIDAÇÃO: cargo check/test/clippy -p das suas 2 crates. As 4 falhas em
  history_integration_t19.rs são ALHEIAS (WIP dispatch) — ignore até eu
  fechar; não tente "consertar".

FECHAMENTO (mandato §0): ≥2 auditorias adversariais (lentes rotacionadas,
  não reuse W/X/Y/Z de T2.1) → remediar CRITICAL/HIGH/MEDIUM → re-audit
  erro-zero. Reporte: "T2.5 pronto, commit local <sha>, audit erro-zero."

PRÓXIMAS APÓS T2.5: T2.3 → T2.4 → T2.2 → T2.6 → T2.7 (eu redijo o
  briefing de cada uma quando T2.5 fechar).
═══════════════════════════════════════════════════════════════════
```

---

## §7. Quick start pra você, Coordenador

1. Resolva o débito de colisão §1.1 (WIP dispatch quebrando testes do Painter) — descubra o dono, feche.
2. Decida a distribuição §2 (recomendo: 1 implementador segura o Painter inteiro; os outros 2 em módulos disjuntos).
3. Escreva o handoff descendente §6 pro implementador Painter (preencha `<N>`, entregue).
4. Quando os implementadores reportarem commits locais + audits erro-zero, faça `./scripts/ship.sh` → push único → babysit CI (DIRETRIZ §8).

**Padrão-ouro. Sem colisão.**
