═══════════════════════════════════════════════════════════════════
BRIEFING — Implementador-1 · módulo SPRITE INSPECTOR v2 · W1 (a partir de T1.4)
Autor: Coordenador (sessão 2026-05-28) · você é o 1º (Sprite) dos 5 implementadores
═══════════════════════════════════════════════════════════════════

VOCÊ É: o ÚNICO implementador deste módulo. Posse EXCLUSIVA de
  crates/ph2d-render/  (+ leitura em crates/ph2d-ecs/).
Caminho (C) foundational — NÃO paraleliza internamente. A cadeia
T1.4→T1.11 é sequencial (migrator → ABI → extract → shader). Não há
outro agente no ph2d-render. Os outros 4 implementadores estão em
módulos fisicamente isolados (imageio-avif, KTX2, Painter, Vector) —
você não os vê e não os toca.

DIRETRIZ DO PROJETO (Enio, 2026-05-28): "o melhor possível, sem pensar
em custos". Isso já é o mandato §0 do Sprite (padrão-ouro absoluto).
Custo de build/CI/footprint NÃO é razão pra cortar caminho. Onde houver
fork qualidade-vs-custo, escolha a mais correta/completa.

───────────────────────────────────────────────────────────────────
LEIA PRIMEIRO (nesta ordem)
───────────────────────────────────────────────────────────────────
  1. docs/HANDOFF_sprite_inspector_v2.md §0 (MANDATO padrão-ouro) + §1
     (O LOOP) + §2 (PONTO DE ENTRADA) + §3 (mapa de pastas) + §7 (gates).
  2. docs/Sprite_projeto/15_plano_de_implementacao.md §15.2 (tasks W1).
  3. docs/IntegracaoMultiAgente/DIRETRIZ.md §7 (anti-colisão git) + §6
     (codificação rápida).
  4. Memórias: feedback-audit-lens-diversity, feedback-scoped-commit-shared-index,
     feedback-destructive-reset-collision, feedback-audit-scope-discipline,
     feedback-app-ui-english-only, feedback-perfection-no-deferrals.

───────────────────────────────────────────────────────────────────
SANITY CHECK (rode primeiro — baseline já validado por mim)
───────────────────────────────────────────────────────────────────
  git log --oneline -3
    # HEAD = e5fb811. A história contém 4591f7e (T1.1) + 3fd0b80 (docs).
  git status -sb -- crates/ph2d-render/ crates/ph2d-ecs/
    # esperado: NADA pendente nesses 2 crates (working tree do Sprite limpo).
  source scripts/slot-env.sh impl-1   # isola target/ (ou CARGO_TARGET_DIR próprio)
  cargo test -p ph2d-render           # baseline: 85 lib + 23 postcard verdes

  ⚠️ O working tree TEM WIP de outras 4 sessões (editor-core dispatch, shells,
  docs untracked Sprite/Painter). HEAD = e5fb811, 83 ahead de origin. NADA disso
  é seu. NÃO comite misturado.

───────────────────────────────────────────────────────────────────
ESTADO (não refaça)
───────────────────────────────────────────────────────────────────
  T1.1+T1.2+T1.3+T1.3.5 FECHADOS (commit 4591f7e + cadeia anterior). Sprite já é
  v4 (20 campos), SpriteVersioned::V4 existe (disc 0x01), 85 lib + 23 postcard
  verdes. Auditoria de continuação = GO (postcard 22/22, determinism 4/4).

───────────────────────────────────────────────────────────────────
PRÓXIMA TASK = T1.4 (migrator)
───────────────────────────────────────────────────────────────────
  ⚠️ Drift de numeração: o plano §15.2 chama de T1.4; o handoff técnico §2
  chama de T1.6. SÃO A MESMA COISA. Contrato canônico no stub #[ignore]d:
    crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs (linha ~57).

  - Implemente Sprite::migrate_v3_to_v4(SpriteV3) -> Sprite (spec §10.2):
    branch region_filter_clip Atlas=true / Individual=false; premultiplied
    rebuild de texture-store context (NÃO matches!(source, Individual)).
  - Implemente crate::sprite_versioned::load_sprite(&[u8]) -> Result<Sprite, LoadError>
    (ADR-0070-amendment-2 §4): dispatch V3→migrate→v4, V4→direto.
  - Un-ignore o stub + per-fixture assertions (spec §10.6) sobre as 5 fixtures
    v3 + 1 round-trip v4.

  Depois, na ordem (sequencial): T1.7a/b (ABI RenderInstance 144B/11 attrs +
  bench criterion <8ms M-series) → T1.8..T1.11 (extract tint cascade / per-corner
  / flip_uv / shader WGSL) → T1.12 (arch-gate architecture_sprite_inspector_surface
  cap 20 fields) → T1.13 (audit) → T1.14 (commit).

  CRITÉRIO DE FECHAMENTO W1 (handoff técnico §5): ph2d-render + ph2d-ecs verdes;
  5 fixtures v3→v4 carregam; vertex_attr_offsets_match_struct com 11 attrs;
  bench T1.7b <8ms; e SMOKE do Enio: cena atual renderiza IDÊNTICA (zero regressão).

───────────────────────────────────────────────────────────────────
O LOOP (por task, sem parar até precisar de smoke)
───────────────────────────────────────────────────────────────────
  1. Build isolado (slot impl-1 — sem contender no lock do target/).
  2. Implemente padrão-ouro: zero corner-cut, zero "TODO depois", contratos
     minúsculos+gateados, toda superfície pública documentada, testes
     feliz+edge+classe-de-bug.
  3. Auto-verifique: cargo test/clippy --all-targets/fmt -p ph2d-render.
  4. AUDITE: ≥2 auditores adversariais paralelos, LENTES ROTACIONADAS (A escopo ·
     B ABI/grep · C determinism/HR-5 · D UX/i18n · E security/perf/coverage).
     Duros, sem validar por cortesia (feedback-audit-lens-diversity).
  5. CORRIJA TODOS os achados (Crítico→Baixo). RE-AUDITE até erro-zero
     (feedback-perfection-no-deferrals — gaps in-scope fecham agora).
  6. Commit ESCOPADO em background:
       git add -- <só meus paths em ph2d-render>
       git commit --no-verify -m "msg" -- <mesmos paths>

───────────────────────────────────────────────────────────────────
ANTI-COLISÃO (a máquina tem outros 4 agentes ativos)
───────────────────────────────────────────────────────────────────
  - NUNCA git add -A / -a / git add .  → só git add -- <paths>.
  - NUNCA git reset --hard / git restore / git clean na árvore compartilhada.
  - git status ANTES de stage; se há M/?? que não são seus → NÃO comite, reporte.
  - Stage CEDO (fence contra reset alheio — staged + untracked sobrevivem a
    reset --hard de outro agente; tracked+uncommitted NÃO).
  - Commit escopado com `-- paths` no próprio commit (não varre o índice alheio).

───────────────────────────────────────────────────────────────────
NÃO TOQUE / PARE-E-REPORTE ao Coord (sou eu) se precisar
───────────────────────────────────────────────────────────────────
  - Qualquer arquivo fora de crates/ph2d-render/ (exceto LEITURA de ph2d-ecs).
  - Contratos congelados (tool.rs/PanelEvent, nodegraph) — exige ADR (Coord-only).
  - crates/ph2d-host/ (MemoryBudget) — só no FIM de W1, eu faço (Coord-A).
  - imageio-svg clippy fail (lib.rs:84) e outros pre-existing — NÃO são seus.
    EU (Coord) cuido disso no ship. Só reporte se cruzar seu caminho; não fixe.
  - UI strings sempre em INGLÊS (feedback-app-ui-english-only).

───────────────────────────────────────────────────────────────────
QUANDO PARAR / REPORTAR
───────────────────────────────────────────────────────────────────
  - Quando a task precisar de smoke visual (./play.command — fim de W1: "cena
    atual renderiza idêntica, zero regressão") OU mudança foundational fora de
    escopo. Aí: relatório curto pro Coord.
  - NÃO faça git push / CI (é o ship do Enio, via mim).
  - Ao fechar cada task: "T1.X pronto. Commit local <sha>. ph2d-render verde
    (N lib + M postcard). Audit: <K lentes, L findings, todos fechados>."
═══════════════════════════════════════════════════════════════════
