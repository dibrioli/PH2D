═══════════════════════════════════════════════════════════════════
BRIEFING — Implementador · módulo VECTOR (continuação W1)
Autor: Coordenador (sessão 2026-05-28) · você é o Vector, 5º dos 5 implementadores
═══════════════════════════════════════════════════════════════════

VOCÊ É: o ÚNICO implementador do módulo Vector. Os outros 4 estão em módulos
disjuntos (Sprite/render, imageio-avif, KTX2, Painter) — você não os vê.

DIRETRIZ DO PROJETO (Enio, 2026-05-28): "o melhor possível, sem pensar em
custos". Atenção redobrada AQUI: a sessão Vector anterior fechou com baixa
confiança do Enio (10 R-rounds = falha de design upfront). A auditoria 6-lente
(docs/AUDIT_vector_module_W1_results.md) confirmou data-model sólido, SEM
redesign — mas o padrão é padrão-ouro absoluto: T1.4 sai como implementação
REAL (não stub), com golden tests, e fecha com re-audit. Sem corner-cut.

───────────────────────────────────────────────────────────────────
CONTEXTO (leia nesta ordem)
───────────────────────────────────────────────────────────────────
  1. docs/AUDIT_vector_module_W1_results.md (achados das 6 lentes).
  2. docs/HANDOFF_vector_module_W1_continuation.md (estado; §2 o que está FECHADO).
  3. docs/Vector Module/17_plano_de_implementacao.md (plano W1, T1.4/T1.6).
  4. DIRETRIZ §7 (anti-colisão git) + §6 (codificação rápida).
  5. Memórias: project-vector-module-w1-audit, feedback-audit-lens-diversity,
     feedback-scoped-commit-shared-index, feedback-git-stash-multiagent-danger,
     feedback-app-ui-english-only, feedback-perfection-no-deferrals.

  Blocos 0/1/2 JÁ commitados (8b60f8c, 3617672, 2732962) — NÃO re-faça.
  Persistência (C1-C3) foi removida (auto-save) e DEFERE pra W2 AssetDb —
  decisão adjacente ratificada pelo Enio, NÃO é gap in-scope teu.

───────────────────────────────────────────────────────────────────
SANITY CHECK (rode primeiro — baseline já validado por mim)
───────────────────────────────────────────────────────────────────
  source scripts/slot-env.sh impl-vector   # ou CARGO_TARGET_DIR=target/<slot>
  git log --oneline -3                      # HEAD=e5fb811; contém 2732962/3617672/8b60f8c
  git status -sb -- crates/ph2d-vector-doc/ crates/ph2d-vector-traits/ \
    crates/ph2d-brush-traits/ crates/ph2d-tool-vector-pen/
    # esperado: limpo EXCETO 3 untracked _audit_send_sync.rs (vide TASK 0)
  CARGO_TARGET_DIR=target/<slot> cargo test -p ph2d-vector-doc      # 21 unit + 12 arch-gate
  CARGO_TARGET_DIR=target/<slot> cargo test -p ph2d-tool-vector-pen # 28 tests

  ⚠️ HEAD=e5fb811, 83 ahead. Working tree tem WIP de outras 4 sessões. NADA é seu.

───────────────────────────────────────────────────────────────────
SUA PASTA EXCLUSIVA (zero colisão hoje — edite SÓ aqui)
───────────────────────────────────────────────────────────────────
  crates/ph2d-vector-doc/   crates/ph2d-vector-traits/
  crates/ph2d-brush-traits/   crates/ph2d-tool-vector-pen/
  shells/desktop/src/render_loop/vector_pen_bridge.rs   ← SEU (tool-bridge, §3.A.4)
  shells/desktop/src/input_dispatch/vector_pen_input.rs ← SEU

NÃO TOQUE:
  - crates/ph2d-render/ — RESERVADO pela sessão Sprite (bump v3→v4). H5/M3
    (mover world_to_screen_affine pra Camera2d) está BLOQUEADO até eu liberar.
    NÃO comece H5/M3 até eu te avisar "Sprite soltou ph2d-render".
  - shells/desktop/src/render_loop/mod.rs (intent-drain), keybinds, painter_bridge.rs
    — são plumbing compartilhado, MEUS (Coord). Precisa de algo lá? PARE e reporte.
  - Qualquer foundational fora da sua pasta → PARE e reporte.
  - UI strings sempre em INGLÊS (feedback-app-ui-english-only).

───────────────────────────────────────────────────────────────────
TASK 0 — os 3 _audit_send_sync.rs untracked (decida primeiro)
───────────────────────────────────────────────────────────────────
  crates/ph2d-brush-traits/tests/_audit_send_sync.rs
  crates/ph2d-brush-traits/tests/_audit_dyn_send_sync.rs
  crates/ph2d-vector-traits/tests/_audit_send_sync.rs
  Sobraram untracked da sessão de auditoria. Decida (são sua pasta):
  - Se enforçam invariante real (Send+Sync nas traits) → padrão-ouro = formalize
    como arch-gate nomeado + comente o porquê + COMITE (git add -- <esses paths>).
  - Se eram scratch descartável → remova.
  Não os deixe pendurados como untracked órfão.

───────────────────────────────────────────────────────────────────
TASK 1 = T1.4 — Levien cubic fit (stub → real)
───────────────────────────────────────────────────────────────────
  crates/ph2d-vector-doc/src/cubic_fit.rs é stub de 38 linhas (retorna input
  sem mudança). Implemente fit_cubic_levien REAL (Raph Levien 2021):
  https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html
  - DoD do próprio stub: < 0.5 px max error nas 5 fixtures canônicas.
  - Golden test (feliz + edge: poucos samples / colinear-degenera-pra-reta /
    cusp). Determinismo HR-5: se usar transcendentais, via libm (grep
    sin/cos/tan/atan2/sqrt/pow — não f32:: nativo).
  - Pasta isolada → baixo risco de colisão.

DEPOIS (PERGUNTE-ME antes):
  - H5/M3 affine → SÓ quando eu liberar ph2d-render (Sprite session).
  - T1.6 CRDT replay → re-escopar pós scene-ownership; exige custom Deserialize
    depth-bounded + gate (Lente A do relatório). Não comece sem meu OK.
  - LOW items §3.4 do handoff (dedup 12px, crosshair, atalho P, cap segments) —
    eu priorizo.

───────────────────────────────────────────────────────────────────
DISCIPLINA GIT (colisões ativas — 5 implementadores no índice compartilhado)
───────────────────────────────────────────────────────────────────
  - NUNCA git stash (na sessão anterior um pop injetou conflict markers no
    arquivo de OUTRO agente — proibido; isole por raciocínio estático de paths).
  - NUNCA git add -A / -a / git add . / reset --hard / restore / clean.
  - git add -- <só seus paths>  ;  git commit --no-verify -m "msg" -- <seus paths>
  - RACE-GUARD antes do commit:
      git diff --cached --name-only          # só seus arquivos?
      git diff --name-only --diff-filter=U   # algum unmerged? → aborte, me avise
  - --no-verify legítimo SÓ se o hook falhar em drift alheio (imageio-svg clippy,
    asset-cooker fmt, render_loop/mod.rs fmt — todos MEUS p/ limpar no ship). Seu
    diff deve passar rustfmt --check + cargo test -p <crate> isolado.
  - Commits LOCAIS, sem push (eu faço ship+push 1× por jornada).

───────────────────────────────────────────────────────────────────
FECHAMENTO (mandato padrão-ouro + alta cadência DIRETRIZ §6.6) / REPORT
───────────────────────────────────────────────────────────────────
  INNER LOOP por task = SÓ `cargo check -p ph2d-vector-doc` (ou cargo-check-narrow.sh).
  NADA de test / clippy --all-targets / auditor POR TASK.
  NO FECHAMENTO do módulo (1×, NÃO por task) — sobre o diff ACUMULADO:
  cargo nextest (scripts/nextest-impacted.sh — impacto + golden determinismo) +
  clippy --all-targets + ≥2 auditorias adversariais (lentes ROTACIONADAS, não reuse
  as 6 do relatório) → remediar CRITICAL/HIGH/MEDIUM → re-audit erro-zero.
  T1.8 audit formal: a auditoria existente cobre a maior parte; 1 mini-round
  pós-T1.4 confirmando (lentes que pegaram alvo grande), não round do zero.
  SMOKE (Enio, fim de W1): vide handoff §3.5 (triângulo persiste, Esc cancela/
  limpa, sem .ph2d-vector no root, click rejeitado → toast).
  Reporte por task: "T1.X pronto, commit local <sha>, ph2d-vector-doc +
  tool-vector-pen verdes, audit <K lentes> erro-zero."
═══════════════════════════════════════════════════════════════════
