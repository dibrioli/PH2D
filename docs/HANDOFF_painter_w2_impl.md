═══════════════════════════════════════════════════════════════════
BRIEFING — Implementador Painter · W2 (continua de T2.1, commit c82293c)
Autor: Coordenador (sessão 2026-05-28) · você é o Painter, 4º dos 5 implementadores
═══════════════════════════════════════════════════════════════════

VOCÊ É: o ÚNICO implementador do módulo Painter. Posse EXCLUSIVA de
  crates/ph2d-tool-painter/  +  crates/ph2d-panel-painter-sidebar/
  (e, por dependência, crates/ph2d-painter-stroke/ + crates/ph2d-painter-brush/
   são do MESMO módulo — você os toca se a regressão abaixo exigir).
Os outros 4 implementadores estão em módulos disjuntos (Sprite/render,
imageio-avif, KTX2, Vector) — você não os vê.

DIRETRIZ DO PROJETO (Enio, 2026-05-28): "o melhor possível, sem pensar em
custos" = mandato §0 do plano Painter (padrão-ouro absoluto, sem gambiarra,
sem deferral aceitável). Custo de build/footprint não justifica cortar caminho.

───────────────────────────────────────────────────────────────────
⚠️ CORREÇÃO ao handoff Coord de origem (§1.1) — leia, mudou
───────────────────────────────────────────────────────────────────
O handoff de origem dizia que os 4 testes falhando em
  crates/ph2d-tool-painter/tests/history_integration_t19.rs
eram culpa de "WIP não-commitado de outro implementador no dispatch
(number_input/tick) do editor-core" e mandava você IGNORAR.

EU (Coord) verifiquei e isso está ERRADO:
  - O WIP atual em dispatch/number_input.rs + tick.rs é PURO `cargo fmt`
    (zero mudança de lógica) — eu limpo no ship; não te afeta.
  - O teste history_integration_t19.rs NÃO importa dispatch nenhum — só
    ph2d_painter_brush + ph2d_painter_stroke + ph2d_tool_painter + os traits
    Tool/RasterEditTool. Reverter o WIP de dispatch NÃO conserta nada.
  - Os 4 testes falham no HEAD COMMITADO (e5fb811), determinístico:
      current_samples_len_tracks_pushed_samples  (samples: left 1, right 2)
      deactivate_cancels_active_stroke_in_wal
      detach_journal_cancels_active_stroke        ("cancelled não vira recovered": 1 vs 0)
      u7_tilt_unavailable_flag_set_for_zero_tilt
  - São uma REGRESSÃO REAL nas SUAS crates (painter-stroke / painter-brush /
    tool-painter). Foram verdes em W1 closure (1485471, per memória); algo
    commitado depois quebrou. NÃO são "alheias". NÃO ignore.

→ SUA TASK 0 (antes de T2.5): root-cause + fix dos 4. Padrão-ouro = ship
  verde, não red ignorado. Provável janela de regressão: commits T2.1
  (28b4a27 / 4d71324 / c82293c) ou um bump em painter-stroke/brush. Use
  `git log --oneline -- crates/ph2d-painter-stroke/ crates/ph2d-painter-brush/`
  pra achar o que mudou pós-1485471. Se a causa-raiz cair FORA das suas crates
  (improvável — o teste não toca foundational), PARE e me reporte.

───────────────────────────────────────────────────────────────────
SANITY CHECK (rode primeiro — eu já validei o baseline)
───────────────────────────────────────────────────────────────────
  source scripts/slot-env.sh impl-painter    # target isolado (RAM 8GiB: máx 2-3 cargo simultâneos)
  git log --oneline -3                        # HEAD=e5fb811; história contém c82293c (T2.1)
  git status -sb -- crates/ph2d-tool-painter/ crates/ph2d-panel-painter-sidebar/
    # esperado: NADA pendente nessas 2 (Painter limpo no working tree)
  cargo test -p ph2d-tool-painter --test history_integration_t19
    # 31 passa, 4 falha (a regressão da TASK 0). Confirme antes de mexer.

  ⚠️ HEAD=e5fb811, 83 ahead. Working tree tem WIP de outras 4 sessões
  (editor-core fmt drift, shells, docs untracked). NADA disso é seu.

───────────────────────────────────────────────────────────────────
TASK 1 = T2.5 — commit-to-sprite (depois da TASK 0)
───────────────────────────────────────────────────────────────────
  - No tool crate: `request_commit()` enfileira pending commit; `take_pending_commit`
    + `on_deactivate` disparam o commit do stroke buffer pro Sprite ativo
    (carry-over R3-LE-4 do T1.5).
  - Keybind Cmd+Enter → request_commit: a parte do KEYBIND/shell
    (shells/desktop/src/render_loop/painter_bridge.rs + input) é MINHA (Coord,
    caminho C foundational). Você expõe o método público e me entrega a
    assinatura; eu faço o wire no shell.
  - DoD: ativar Painter → desenhar → trocar de tool (ou Cmd+Enter) → o stroke
    "cola" no sprite (não some). Smoke do Enio confirma.

  Surfaces prontas pra reusar (handoff Coord §3): ui_snapshot / apply_ui_edit
  (SSOT de clamps) / handle_panel_event / helpers size01_to_px·px_to_size01·
  opacity01_to_pct / impl RasterEditTool (take_pending_commit + on_deactivate).

PRÓXIMAS após T2.5 (eu redijo cada briefing quando fechar): T2.3 color picker
  wire → T2.4 modifier square + re-add paint → T2.2 undo/redo replay → T2.6 a11y
  nodes (gate hr12_widgets_a11y) → T2.7 smoke W2 + audit final.

───────────────────────────────────────────────────────────────────
NÃO TOQUE / PARE-E-REPORTE ao Coord (sou eu)
───────────────────────────────────────────────────────────────────
  - shells/ (painter_bridge.rs, keybind/input) — foundational, é MINHA (C).
  - crates/ph2d-editor-core/ (BlenderColorPicker, ids novos, dispatch) — MINHA.
    O fmt drift em dispatch/number_input.rs+tick.rs é meu p/ limpar no ship —
    NÃO mexa.
  - Contratos congelados: caps PainterUiEdit ≤ 24 / PanelEvent ≤ 4 (ADR-0043/0040).
    Nenhuma task atual precisa bumpar; se precisar = (C)+ADR via mim.
  - Pre-existing alheios: ph2d-host-desktop PanelEvent::Activated missing,
    panel_loc_cap hierarchy, imageio-svg clippy — NÃO são seus; reporte se cruzar.
  - UI strings sempre em INGLÊS (feedback-app-ui-english-only).

───────────────────────────────────────────────────────────────────
DISCIPLINA GIT (índice compartilhado entre 5 implementadores)
───────────────────────────────────────────────────────────────────
  - NUNCA git stash. (Na sessão anterior um stash pop injetou conflict markers
    no arquivo de OUTRO agente — proibido. Pra isolar "minha mudança vs alheia",
    raciocine estaticamente sobre paths, não com stash.)
  - NUNCA git add -A / -a / git add . / reset --hard / restore / clean.
  - git add -- <só seus paths>  ;  git commit --no-verify -m "msg" -- <seus paths>
    (`-m` ANTES do `--`).
  - git diff --cached --name-only ANTES do commit (índice vaza arquivo alheio).
  - Stage CEDO (fence contra reset alheio). Commits LOCAIS, sem push (eu faço ship).

───────────────────────────────────────────────────────────────────
VALIDAÇÃO + FECHAMENTO (mandato §0)
───────────────────────────────────────────────────────────────────
  cargo check/test/clippy -p ph2d-tool-painter -p ph2d-panel-painter-sidebar --all-targets -- -D warnings
  (+ -p ph2d-painter-stroke -p ph2d-painter-brush se a TASK 0 tocá-las)
  Gates: architecture_panel_chip_pill_no_stepper, hr12_widgets_a11y (T2.6),
         no_literal_color / no_magic_numeric, architecture_painter_contract_surface.
  Cada task fecha com ≥2 auditorias adversariais (lentes ROTACIONADAS — NÃO reuse
  W/X/Y/Z de T2.1) → remediar CRITICAL/HIGH/MEDIUM in-code → re-audit erro-zero.
  Reporte por task: "T2.X pronto, commit local <sha>, <N lib + M> verdes
  (incl. os 4 t19 da TASK 0), audit <K lentes> erro-zero."
═══════════════════════════════════════════════════════════════════
