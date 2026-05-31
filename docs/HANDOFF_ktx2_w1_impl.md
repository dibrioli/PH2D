═══════════════════════════════════════════════════════════════════
HANDOFF — Implementador KTX2 / Texture Compression · continuação W1
Autor: Coordenador (sessão 2026-05-28) · você é o 1º dos 5 implementadores
═══════════════════════════════════════════════════════════════════

CONTEXTO (1 linha): KTX2 Fase 2 W1 está ~85% pronto. Último audit pendente
do batch B (W1.T9) JÁ foi fechado e auditado (`bf39eb6`). Sua próxima task
é o gate final da W1: W1.T15.

───────────────────────────────────────────────────────────────────
SANITY CHECK (rode primeiro — eu já validei, mas confirme localmente)
───────────────────────────────────────────────────────────────────
  git log --oneline -3
    # HEAD = e5fb811. A história contém bf39eb6 (W1.T9 audit ν+ξ).
  git status -sb -- crates/ph2d-asset-ktx2/ tools/asset-cooker/ crates/ph2d-asset/
    # Esperado: NENHUMA mudança pendente no módulo (working tree do KTX2 limpo).
  RUST_TEST_THREADS=1 cargo test -p ph2d-asset-ktx2
    # Esperado: 39 lib + 2 doctests verdes.

  ⚠️ O working tree TEM modificações de OUTRAS sessões (editor-core dispatch,
  shells/desktop, untracked docs Sprite/Painter). NÃO são suas, NÃO toque.
  O conflito UU/stash-pop que o handoff de origem mencionava JÁ FOI resolvido
  (HEAD avançou limpo). Se aparecer QUALQUER UU/conflict no SEU módulo: PARE
  e me avise — não resolva git fora da sua pasta.

───────────────────────────────────────────────────────────────────
SUA PASTA EXCLUSIVA (edite SÓ aqui)
───────────────────────────────────────────────────────────────────
  crates/ph2d-asset-ktx2/   ·   tools/asset-cooker/   ·   crates/ph2d-asset/
  docs/audits/w1-t*-lens-*.md   ·   docs/plans/2026-05-texture-compression-waves.md
  docs/HANDOFF_ktx2_*.md

NÃO TOQUE (outros 4 implementadores ativos):
  crates/ph2d-render/ · ph2d-ecs/ · ph2d-editor-core/  (Sprite Inspector v2)
  crates/ph2d-tool-painter/ · ph2d-painter-* · ph2d-panel-painter-sidebar/  (Painter)
  crates/ph2d-tool-vector-pen/ · ph2d-vector-*  (Vector)
  .github/workflows/  (compartilhado — alto risco)
  Precisou de algo fora? PARE e me reporte (Coordenador). Não edite.

───────────────────────────────────────────────────────────────────
TASK — W1.T15: audit 5-lente final de toda a W1 (gate antes da W2)
───────────────────────────────────────────────────────────────────
  - Catalogue os 6 ciclos de audit já feitos (10+ lentes gregas α..ξ; vide
    docs/audits/ + handoff de origem §3) num índice consolidado.
  - Final integration check do pipeline: cook → asset → ktx2 (end-to-end).
  - Lentes ainda NÃO usadas (escolha 2): ο (omicron), π (pi), ρ (rho), σ (sigma).
  - ANTI-GOODHART: máx 2 lentes paralelas por round, round único. NÃO recriar
    o padrão R1→R4 (rotacione a LENTE, não repita a mesma — vide memória
    [[feedback-audit-lens-diversity]]). Gates executáveis > claims verbais.
  - Fixes IN-SCOPE → inline na sessão ([[feedback-perfection-no-deferrals]]).
    Adjacent (fora das suas 3 crates) → me reporta com owner, não fixe.
  - ✅ INCLUI fix de ν-6 (é SUA pasta): docs drift em
    crates/ph2d-asset/src/asset.rs:38 — o `///` cita `ph2d_asset_ktx2::parse(&blob)`
    mas a API pública real é `decode_ktx2_bytes` (confirmado nos doctests de
    asset-ktx2/src/lib.rs). Corrija o doc-comment.

  Deliverables: docs/audits/w1-t15-lens-{X,Y}-*.md (X,Y = as 2 letras escolhidas).

───────────────────────────────────────────────────────────────────
ARMADILHAS DO MÓDULO (decoradas — já queimaram)
───────────────────────────────────────────────────────────────────
  1. ISPC parallel SIGBUS: `cargo test -p ph2d-asset-cooker` em paralelo crasha
     determinísticamente (encoders ISPC vendored = global state não-thread-safe).
     SEMPRE `RUST_TEST_THREADS=1`. asset-ktx2 é parser puro (sem ISPC) mas use
     a env por hábito.
  2. slot-env.sh dentro do Bash tool não é detectado como sourced (aborta cargo).
     Use export direto de CARGO_TARGET_DIR se precisar isolar; asset-ktx2 tem
     deps mínimas, target default é seguro.
  3. W1.T8 deferred honesto: ktx2 0.5 e ctt 0.4.0 são READ-ONLY (zero Writer).
     `Ktx2Image::premul_intent()` retorna sempre Unspecified em KTX2 cooked hoje.
     NÃO é bug — é limitação documentada (handoff origem §6.2).
  4. Pin hash `gradient_64x64` desligado em fixtures.rs (assert comentado) —
     espera W1.T10 canonical runner. NÃO re-habilite agora.

PRE-EXISTING FAILURES cross-session (NÃO fixar — não são seus):
  - ph2d-editor-core --test architecture_panel_loc_cap (hierarchy session)
  - cargo check -p ph2d-host-desktop (Painter PanelEvent::Activated missing)

───────────────────────────────────────────────────────────────────
VALIDAÇÃO
───────────────────────────────────────────────────────────────────
  RUST_TEST_THREADS=1 cargo test -p ph2d-asset-ktx2
  RUST_TEST_THREADS=1 cargo test -p ph2d-asset-cooker    # NUNCA sem a env!
  cargo test -p ph2d-asset

───────────────────────────────────────────────────────────────────
COMMIT / REPORT
───────────────────────────────────────────────────────────────────
  - Commit ESCOPADO: `git add -- <só seus paths>` (nunca -A / -a / git add .).
    `git status` antes de stage; se houver M/?? que não são seus, NÃO comite.
  - Fast-mode de dia OK: `git commit --no-verify` (eu rodo ship.sh no fim).
  - Você NÃO pusha. Eu (Coordenador) faço ship.sh + push + babysit do CI no
    fim da jornada (há 83 commits locais ahead de origin; push é decisão do Enio).
  - Ao terminar, reporte: "W1.T15 pronto. Commit local <sha>. asset-ktx2 +
    asset-cooker (RUST_TEST_THREADS=1) + asset verdes. Findings: <N in-scope
    fixados, M adjacent reportados>."

DEPOIS de W1.T15 (PERGUNTE-ME antes de iniciar qualquer um):
  - W1.T8.1 (patcher post-hoc PH2D_PREMUL, ~200-400 LOC) — ✅ FEITO 2026-05-31.
    `ph2d_asset_ktx2::patch_premul_intent` (insere KV PH2D_PREMUL, reescreve
    kvd/sgd/level offsets, rebuild do tail; insert-only; SGD align(8) coberto).
    Cooker wired via `cook_tagged`/`cook_all_tagged` + CLI; asset-ktx2 virou dep
    de produção do cooker. ~14 tests (11 unit + 2 seam end-to-end via ctt real).
    Detalhe completo: docs/plans/2026-05-texture-compression-waves.md §W1.T8.1.
  - W1.T10/T12/T13 (CI canonical runner + LFS) — ALTO RISCO, toca workflows.
    Eu renegocio estratégia com o Enio antes (provável spike-texture-cook.yml
    separado em vez de mexer no spike.yml).
═══════════════════════════════════════════════════════════════════
