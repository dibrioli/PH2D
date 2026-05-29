# Handoff — Sprite Inspector v2 (W1) → **novo Coordenador único** (3 implementadores)

**Data:** 2026-05-28
**De:** sessão Coord-A+Implementador que fechou a auditoria de continuação + T1.1/T1.2/T1.3.
**Para:** o **Coordenador único** que vai orquestrar 3 implementadores em paralelo.
**Por quê este doc existe:** tivemos **colisões entre implementadores**. Este handoff (a) te entrega o estado exato do módulo Sprite, (b) te dá as regras anti-colisão que NÃO podem ser violadas, e (c) te pede para **escrever um briefing focado para o implementador que continua o Sprite** (template pronto no §6).

> **Leia antes:** [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md) v7.0 (papéis, caminhos A/B/C, contratos congelados, §7 anti-colisão git) + o handoff técnico profundo [`docs/HANDOFF_sprite_inspector_v2.md`](HANDOFF_sprite_inspector_v2.md) (§0 mandato, §2 PONTO DE ENTRADA, §3 mapa de pastas, §7 gates).

---

## 1. O que é este módulo e em que caminho ele cai

**Sprite Inspector v2** = expansão do `Sprite` (sucessor do inspector de sprite do Godot/Unity), W0 ratificada (7 ADRs Accepted 0069..0074 + 0025-amendment-1). Estamos na **W1 — schema bump strategic-only** (zero feature visível ainda; só fundação de schema + ABI + extract + shader).

**CAMINHO (C) — Coord-A foundational. NÃO é drop-crate (A).** O trabalho vive em `crates/ph2d-render/` e `crates/ph2d-ecs/`, que são **crates foundational**. Isso significa:

- **NÃO paraleliza dentro do módulo.** As tasks W1 (T1.4..T1.14) são uma cadeia sequencial: migrator → ABI `RenderInstance` → extract phase → shader WGSL → arch-gate. Um único implementador toca o Sprite por vez.
- **O implementador do Sprite precisa de POSSE EXCLUSIVA de `crates/ph2d-render/`** (e leitura/escrita pontual em `crates/ph2d-ecs/` só se reabrir o skew em W2). Se outro implementador encostar em ph2d-render, colide na hora.

**Implicação direta pra você (Coord):** dos seus 3 implementadores, **só 1 pode estar no Sprite**. Os outros 2 têm que estar em trabalho fisicamente isolado (vide §3).

---

## 2. Estado exato do módulo (verificado, commitado local)

**HEAD local:** `3fd0b80` (docs) → `4591f7e` (T1.1) em cima da cadeia anterior. **Nada pushado — push é do Enio.**

| Item | Status |
|---|---|
| **Auditoria de continuação** (4 commits da sessão anterior `cef1959`/`e3ad19f`/`5974a84`/`f9850bf`) | ✅ **GO** — postcard 22/22, determinism 4/4 (golden hash + libm pin), zero ≥HIGH. 1 LOW informacional (sim_populate velocity-sin, demo-only, não-fixado). |
| **T1.1** Sprite struct v3→v4 (20 campos) | ✅ commit `4591f7e` |
| **T1.2** Default helpers (`default_white`/`default_one`/…) | ✅ **incluído em `4591f7e`** (o struct não compila sem eles) |
| **T1.3** Constructors `atlas()`/`individual()` v4 | ✅ **incluído em `4591f7e`** (idem) |
| **T1.3.5** libm sweep (ph2d-ecs + 4 crates) | ✅ sessão anterior (`5974a84`+`f9850bf`) |
| **T1.5** 5 fixtures v3 | ✅ movido pra W0.T0.12 (`cef1959`) |
| `SpriteVersioned::V4(Sprite)` disc 0x01 + 2 testes V4 (pin + round-trip) | ✅ em `4591f7e` |
| Drift-gate `spritev3_struct_wire_matches_live_sprite_v3` | ✅ aposentado (premissa live==v3 fechou; baseline v3 ainda pinada por `fixtures_match_canonical_serialization`) |

**Verificação verde:** `cargo test -p ph2d-render` = 85 lib + 23 postcard + binários auxiliares, todos verdes; clippy/fmt limpos no código Sprite.

### ▶ PRÓXIMA TASK = **T1.4** (migrator `migrate_v3_to_v4`)

⚠️ **Drift de numeração entre docs:** o [plano §15.2](Sprite_projeto/15_plano_de_implementacao.md) chama o migrator de **T1.4**; o [handoff técnico §2](HANDOFF_sprite_inspector_v2.md) chama de **T1.6**. **São a mesma coisa.** O **contrato canônico está escrito no stub `#[ignore]`d** em [`crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs`](../crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs) (linha ~57):
- `crate::sprite_versioned::load_sprite(&[u8]) -> Result<Sprite, LoadError>` (ADR-0070-amendment-2 §4) — entry point de dispatch do wrapper enum.
- `Sprite::migrate_v3_to_v4(SpriteV3) -> Sprite` (spec §10.2) — transform puro, com branch `region_filter_clip` (Atlas→true, Individual→false) + `premultiplied` rebuild de texture-store context.
- Un-ignore o stub + per-fixture assertions (spec §10.6) sobre as 5 fixtures v3 + um round-trip v4.

Depois: T1.7a/b (ABI `RenderInstance` 144B/11 attrs + criterion bench) → T1.8..T1.11 (extract tint cascade / per-corner / flip_uv / shader WGSL) → T1.12 (arch-gate `architecture_sprite_inspector_surface` cap 20 fields) → T1.13 (audit) → T1.14 (commit).

**Critério de fechamento W1** (handoff técnico §5): `ph2d-render` + `ph2d-ecs` verdes, 5 fixtures v3→v4 carregam, `vertex_attr_offsets_match_struct` com 11 attrs, bench T1.7b <8ms M-series, e **smoke do Enio: cena atual renderiza IDÊNTICA (zero regressão)**.

---

## 3. Anti-colisão — as regras que NÃO podem ser violadas (a causa-raiz dos conflitos)

As colisões vêm de 3 mecanismos (DIRETRIZ §7 + memórias do projeto). Faça os 3 implementadores obedecerem:

1. **Índice git é compartilhado.** Dois implementadores com arquivos staged ao mesmo tempo → um `git commit` agarra os arquivos do outro. **Regra:** NUNCA `git add -A`/`-a`/`git add .`. Sempre `git add -- <paths-específicos>`. E **commit escopado**: `git commit -m "msg" -- <só meus paths>` (o `-- paths` no commit garante que arquivos staged por outro agente NÃO entrem). *Eu já usei isso nesta sessão: o agente KTX2 tinha `asset-ktx2/src/lib.rs` staged no índice; meu commit `4591f7e` saiu com exatamente 5 arquivos meus sem varrê-lo.*

2. **`git reset --hard` / `git restore` alheio destrói WIP.** Um agente que dá `git reset --hard HEAD` apaga WIP **tracked+uncommitted** de outro. **Regra:** ninguém roda reset/restore destrutivo na árvore compartilhada. **Defesa:** cada implementador faz `git add -- <my-paths>` **cedo** (vira fence — staged sobrevive; e untracked também sobrevive a reset --hard). Proibido reset/checkout/clean fora da própria pasta.

3. **Lock de `target/` serializa cargo.** Dois `cargo` simultâneos sem isolamento → o 2º espera silenciosamente. **Regra:** cada sessão roda `source scripts/slot-env.sh <slot-id>` (slots: `coord`, `impl-1..3`) OU exporta `CARGO_TARGET_DIR` próprio. **RAM 8 GiB = máx 2-3 slots cargo-ativos simultâneos** — não rode os 3 implementadores compilando ao mesmo tempo se a máquina é a mesma.

**Protocolo SESSION_ACTIVE** ([`docs/SESSION_ACTIVE.md`](SESSION_ACTIVE.md)): você (Coord) mantém a verdade de quem possui o quê. **Cada implementador lê antes de cada burst.** A entrada Coord-A atual já reflete: **Sprite reserva `crates/ph2d-render/`** + lista os pre-existing failures.

---

## 4. Particionamento sugerido dos 3 implementadores (zero sobreposição física)

Como o Sprite é (C) foundational e não paraleliza internamente, distribua assim:

| Impl | Trabalho | Pasta exclusiva | Caminho |
|---|---|---|---|
| **Impl-1 (Sprite)** | Continua W1 a partir de **T1.4** | `crates/ph2d-render/` (+ leitura `ph2d-ecs/`) | (C) foundational |
| **Impl-2** | Tool/node nova OU modificar feature existente | `crates/ph2d-tool-<slug>/` OU `crates/ph2d-node-<dom>-<slug>/` | **(A) drop-crate** — não toca arquivo central, paraleliza por construção |
| **Impl-3** | Outra tool/node OU painel via Coord-scaffold | `crates/ph2d-tool-<slug2>/` OU painel `crates/ph2d-panel-<slug>/` (você scaffolda antes) | (A) ou (B) |

**Regra de ouro do particionamento:** dois implementadores **(A) drop-crate** nunca colidem (pasta nova cada + wiring gerado por sync entre marcadores; garantia formal DIRETRIZ §3.A.5). O perigo é **qualquer um deles encostar em foundational** (`ph2d-render`, `ph2d-editor-core`, `shells/`, contratos congelados). Se um (A) precisar de algo central → **ele PARA e reporta a você**, não edita (DIRETRIZ §1.4). **Só o Impl-1 (Sprite) e você (Coord) tocam foundational.**

> **NÃO** ponha 2 implementadores no Sprite "pra ir mais rápido" — a cadeia T1.4→T1.11 é sequencial (migrator antes de ABI antes de extract antes de shader). Paralelizar dentro dela = exatamente a colisão que você está tentando evitar.

---

## 5. Pendências que precisam da sua decisão / do Enio

- **🚩 BLOQUEADOR DE SHIP (pre-existing, não-meu-crate):** `cargo clippy --all-targets` falha em [`crates/ph2d-imageio-svg/src/lib.rs:84`](../crates/ph2d-imageio-svg/src/lib.rs#L84) — `field_reassign_with_default` (rust-1.95.0). Surge porque o clippy-driver linta path-deps do workspace. **Vai barrar `ship.sh` e o CI lint.** Fix trivial 1-linha (`let opts = usvg::Options { ..Default::default() };`). Não fixei (audit-scope-discipline: bug em crate adjacente → owner). **Decisão tua:** designar um implementador pra corrigir antes do ship, ou tratar como Coord-only.
- **Outros pre-existing failures** (documentados em SESSION_ACTIVE + handoff técnico §9.1, não-fixados): `panel-hierarchy/paint.rs` 388 LOC > cap (owner: hierarchy session); `ph2d-tool-painter` `PanelEvent::Activated` missing (owner: Painter session, bloqueia link do desktop binary).
- **Untracked W0 ride-along:** as specs `docs/Sprite_projeto/*` + ADRs `0069..0074` + `0025-amendment-1` ainda são **untracked** (sobem na cadeia de commit do Enio, handoff técnico §10). **Eu apliquei edições nelas** (reconciliação do nome `default_region_filter_clip` em anatomia/schema/ADR-0070) que estão na working tree untracked — **elas sobem junto quando o Enio commitar os artefatos W0**. Não as commite num commit de código.

---

## 6. 📋 SUA TAREFA: escreva o briefing do Implementador-1 (Sprite) — template pronto

Cole o bloco abaixo (ajustando o que precisar) pro implementador que continua o Sprite. Ele é auto-suficiente: aponta pro mandato, pro loop, pro entry-point e pras regras anti-colisão.

```
═══════════════════════════════════════════════════════════════════
BRIEFING — Implementador-1 · módulo SPRITE INSPECTOR v2 · W1 (a partir de T1.4)
═══════════════════════════════════════════════════════════════════

VOCÊ É: o ÚNICO implementador deste módulo. Posse EXCLUSIVA de
  crates/ph2d-render/  (+ leitura em crates/ph2d-ecs/).
Não há outro agente no ph2d-render. Os outros 2 implementadores estão
em drop-crates isolados (tool-*/node-*) — você não os vê.

LEIA PRIMEIRO (nesta ordem):
  1. docs/HANDOFF_sprite_inspector_v2.md §0 (MANDATO padrão-ouro) + §1 (O LOOP)
     + §2 (PONTO DE ENTRADA) + §3 (mapa de pastas) + §7 (gates).
  2. docs/Sprite_projeto/15_plano_de_implementacao.md §15.2 (tasks W1).
  3. docs/IntegracaoMultiAgente/DIRETRIZ.md §7 (anti-colisão git) + §6 (codificação rápida).
  4. Memórias: feedback-audit-lens-diversity, feedback-scoped-commit-shared-index,
     feedback-destructive-reset-collision, feedback-audit-scope-discipline,
     feedback-app-ui-english-only, feedback-perfection-no-deferrals.

ESTADO: T1.1+T1.2+T1.3+T1.3.5 FECHADOS (commit 4591f7e + cadeia anterior).
Sprite já é v4 (20 campos), SpriteVersioned::V4 existe (disc 0x01),
85 lib + 23 postcard verdes. NÃO refaça isso.

PRÓXIMA TASK = T1.4 (migrator). Contrato canônico no stub #[ignore]d:
  crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs (linha ~57).
  - implemente Sprite::migrate_v3_to_v4(SpriteV3) -> Sprite (spec §10.2:
    branch region_filter_clip Atlas=true/Individual=false; premultiplied
    rebuild de texture-store context, NÃO matches!(source, Individual)).
  - implemente crate::sprite_versioned::load_sprite(&[u8]) -> Result<Sprite, LoadError>
    (ADR-0070-amendment-2 §4): dispatch V3→migrate→v4, V4→direto.
  - un-ignore o stub + per-fixture assertions (spec §10.6) sobre as 5
    fixtures v3 + 1 round-trip v4.
Depois, na ordem: T1.7a/b (ABI RenderInstance 144B/11 attrs + bench
criterion <8ms) → T1.8..T1.11 (extract tint cascade / per-corner /
flip_uv / shader WGSL) → T1.12 (arch-gate cap 20 fields) → T1.13 audit
→ T1.14 commit.

O LOOP (por task, sem parar até precisar de smoke):
  1. Build isolado: source scripts/slot-env.sh impl-1  (ou
     CARGO_TARGET_DIR próprio). Sem contender no lock do target/.
  2. Implemente padrão-ouro (mandato §0): zero corner-cut, zero
     "TODO depois", contratos minúsculos+gateados, toda superfície
     pública documentada, testes feliz+edge+classe-de-bug.
  3. Auto-verifique: cargo test/clippy --all-targets/fmt -p ph2d-render.
  4. AUDITE: >=2 auditores adversariais paralelos, LENTES ROTACIONADAS
     (A escopo · B ABI/grep · C determinism/HR-5 · D UX/i18n · E
     security/perf/coverage). Duros, sem validar por cortesia.
  5. CORRIJA TODOS os achados (Crítico→Baixo). RE-AUDITE até erro-zero.
  6. Commit ESCOPADO: git add -- <só meus paths em ph2d-render>;
     git commit --no-verify -m "msg" -- <mesmos paths>. Em background.

ANTI-COLISÃO (a máquina pode ter outros agentes):
  - NUNCA git add -A / -a / git add .  → só git add -- <paths>.
  - NUNCA git reset --hard / git restore / git clean na árvore.
  - git status ANTES de stage; se há M/?? que não são seus → NÃO comite,
    reporte ao Coord.
  - Stage cedo (fence contra reset alheio). Commit escopado com -- paths
    (não varra arquivos staged por outro agente).

NÃO TOQUE / PARE-E-REPORTE ao Coord se precisar:
  - qualquer arquivo fora de crates/ph2d-render/ (exceto leitura ph2d-ecs).
  - contratos congelados (tool.rs/PanelEvent, nodegraph) — exige ADR.
  - crates/ph2d-host/ (MemoryBudget) — só no FIM de W1, é Coord-A.
  - imageio-svg clippy fail e outros pre-existing (§9.1) — são de outros
    owners; reporte, não fixe.
  - UI strings sempre em INGLÊS (feedback-app-ui-english-only).

QUANDO PARAR: quando a task precisar de smoke visual (./play.command —
fim de W1: "cena atual renderiza idêntica, zero regressão") OU mudança
foundational fora de escopo. Aí: relatório curto pro Coord.
NÃO faça git push / CI (é o ship do Enio).
═══════════════════════════════════════════════════════════════════
```

---

## 7. Referências canônicas

- **Mandato + loop + gates + entry-point:** [`docs/HANDOFF_sprite_inspector_v2.md`](HANDOFF_sprite_inspector_v2.md)
- **Plano executável W1..W8:** [`docs/Sprite_projeto/15_plano_de_implementacao.md`](Sprite_projeto/15_plano_de_implementacao.md)
- **Spec normativa (17 arquivos):** [`docs/Sprite_projeto/`](Sprite_projeto/)
- **ADRs:** [0069](architecture/decisions/0069-sprite-inspector-v2.md)..[0074](architecture/decisions/0074-sprite-component-boundary.md) + [0070-amendment-2](architecture/decisions/0070-amendment-2.md) + [0025-amendment-1](architecture/decisions/0025-amendment-1.md)
- **Papéis / caminhos A·B·C / contratos congelados / anti-colisão:** [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md)
- **Sincronização entre sessões:** [`docs/SESSION_ACTIVE.md`](SESSION_ACTIVE.md)
- **Memória do projeto:** `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`

---

**TL;DR pro novo Coord:** Sprite W1 é (C) foundational → **1 implementador exclusivo no `ph2d-render`, começando em T1.4 (migrator)**; os outros 2 em drop-crates (A) isolados. Faça os 3 obedecerem stage/commit escopado + slot isolado + zero reset destrutivo. Estado limpo e verde em `4591f7e`/`3fd0b80` (local, não-pushado). Bloqueador de ship pendente: clippy do imageio-svg. Cole o §6 pro implementador do Sprite.
