# HANDOFF — KTX2 Fase 2 W1 → **novo Coordenador** (2026-05-28)

**Audiência:** o **Coordenador único** que vai dirigir 3 implementadores (modelo
novo decidido pelo Enio após colisões git entre implementadores paralelos).
**Módulo deste handoff:** **KTX2 / Texture Compression (Fase 2, ADR-0055-v4)**.
**Autor:** sessão Coord-A texture (slot `impl-texture`), que fechou o audit
meta-W1.T9 e está encerrando.

> **Sua tarefa, Coordenador:** ler isto, ler a DIRETRIZ, e então **escrever um
> sub-handoff enxuto para o implementador** deste módulo (esqueleto pronto na
> §8) para ele continuar **de onde paramos** — sem reabrir o que já está fechado.

---

## §0 TL;DR de 5 linhas

- **W1 da KTX2 está ~85% completo.** Último audit pendente do batch B (W1.T9) **foi fechado nesta sessão** — commit `bf39eb6` (HEAD), 39 lib + 2 doctests verdes.
- **Próxima task do implementador = W1.T15** (audit 5-lente final = gate antes da W2). Baixo risco, mesmo crate.
- **NÃO há push pendente seu agora.** Há **80 commits locais ahead de origin/main** (de TODAS as sessões), mas push é decisão do Enio e exige `./scripts/ship.sh` antes (vide §6).
- **⚠️ O working tree está SUJO por um `git stash pop` cross-agente em conflito** (não causado pela KTX2). Detalhe na §7 — o implementador KTX2 **não deve** mexer nisso; é coordenação sua + das outras sessões.
- Pastas do módulo: `crates/ph2d-asset-ktx2/`, `tools/asset-cooker/`, `crates/ph2d-asset/`. Isoladas — não colidem com Painter/Sprite/Vector se cada implementador ficar na sua.

---

## §1 O que é este módulo

Pipeline de **texture compression KTX2** (ADR-0055-v4 Accepted, strategic-only).
Cooking offline nativo per-platform (sem Basis runtime — ADR-0055 v3 abortado).
Três crates, todos **caminho (A)/(D)** da DIRETRIZ (modificar feature isolada,
sem tocar contrato congelado):

| Crate | Papel | LOC novos na Fase 2 |
|---|---|---|
| `tools/asset-cooker/` (texture/ + tests/) | cook lib API + multi-tier batch + target_matrix + mip pyramid + fixtures | ~1.5k |
| `crates/ph2d-asset/` | `Asset::TextureKtx2` variant + `TierIndex` + `LogicalTextureMap` | ~480 |
| `crates/ph2d-asset-ktx2/` | parser KTX2 (Fase 1) + W1.T9 kvd preservation + `PremulIntent` | ~180 (Fase 2) |

Plano vivo: [`docs/plans/2026-05-texture-compression-waves.md`](plans/2026-05-texture-compression-waves.md).
Handoff de origem (estado pré-W1.T9): [`docs/HANDOFF_ktx2_w1_session_continuation.md`](HANDOFF_ktx2_w1_session_continuation.md).
ADR: [`docs/architecture/decisions/0055-*.md`](architecture/decisions/).

---

## §2 Estado das tasks W1 (atualizado 2026-05-28)

| Task | Estado | Commit |
|---|---|---|
| W1.T0..T7 | ✅ | vide handoff de origem §2 |
| W1.T9 (kvd preservation) | ✅ **+ AUDITADA esta sessão** | `9c31822` + `bf39eb6` (audit ν+ξ) |
| W1.T11, W1.T14 | ✅ | `aa6766b` + `d4644ff` |
| **W1.T8** (cooker emit kvd) | ⏳ **DEFERRED** | `ctt 0.4.0` + `ktx2 0.5` ambos READ-ONLY; vide handoff origem §6.2 |
| **W1.T15** (audit 5-lente final) | ⏳ **← PRÓXIMA** | gate antes da W2 |
| W1.T8.1 (patcher post-hoc PH2D_PREMUL) | ⏳ opcional | ~200-400 LOC; destrava bg-removal premul tag |
| W1.T10 + T11.5 (canonical runner CI + LFS) | ⏳ **ALTO RISCO** | toca `.github/workflows/` — **PERGUNTA Enio** |
| W1.T12, W1.T13 | ⏳ | dependem T10 |

### O que esta sessão entregou (commit `bf39eb6`)

Audit meta-W1.T9, **2 lentes ortogonais round único** (anti-Goodhart — não
recriou o padrão R1→R4):

- **ν (Fase 1 contract preservation): PASS** — additividade confirmada (Cargo.toml intocado, sem serde/postcard em `Ktx2Image`, zero consumidor externo).
- **ξ (bounds/DOS): PASS_WITH_FINDINGS** — ordem `count→size→alloc` correta, sem alloc-before-check.
- **4 findings in-scope fechados a erro-zero** (per [[feedback-perfection-no-deferrals]]):
  - ξ-F1 (HIGH): `build_fixture` agora emite seção KVD real + 6 testes de parse-path (round-trip, PH2D_PREMUL e2e, reject too-many/oversized-value, boundary at-cap).
  - ξ-F2: conta **iterações** em vez de `kvd.len()` → fecha bypass de duplicate-key flood.
  - ξ-F3: `MAX_KVD_KEY_BYTES=256` + `KvdKeyTooLong` (defesa simétrica ao value cap).
  - ν-7: `#[non_exhaustive]` em `Ktx2Image` + `Ktx2Error` (aditividade por construção).
- **ν-6** (doc drift em `crates/ph2d-asset/src/asset.rs`) **NÃO fixado** — é adjacent, owner = quem mexer em `ph2d-asset`, per [[feedback-audit-scope-discipline]]. Repassar como nota ao implementador.
- Deliverables: `docs/audits/w1-t9-lens-{nu,xi}-*.md`.

---

## §3 Pastas reservadas do módulo (limites anti-colisão)

**O implementador KTX2 edita SÓ:**
- `crates/ph2d-asset-ktx2/`
- `tools/asset-cooker/`
- `crates/ph2d-asset/`
- `docs/audits/ctt-source-audit-*.md` + `w1-t*-lens-*.md`
- `docs/architecture/decisions/0055-*.md` · `docs/plans/2026-05-texture-compression-waves.md`
- `docs/HANDOFF_ktx2_*.md`

**NÃO tocar (outras sessões — verifique SESSION_ACTIVE.md ao iniciar):**
- `crates/ph2d-render/` · `crates/ph2d-ecs/` · `crates/ph2d-editor-core/` (Sprite Inspector v2)
- `crates/ph2d-tool-painter/` · `crates/ph2d-painter-*` · `crates/ph2d-panel-painter-sidebar/` (Painter)
- `crates/ph2d-tool-vector-pen/` · `crates/ph2d-vector-*` (Vector Module)
- `.github/workflows/` (compartilhado — alto risco; W1.T10 renegocia com Enio: provável `spike-texture-cook.yml` separado em vez de mexer no `spike.yml`)

**Nota cross-pasta benigna:** a sessão Sprite já commitou um pin `libm` em
`tools/asset-cooker/Cargo.toml` (alinhamento workspace) — está em HEAD, não é
conflito vivo.

---

## §4 Known issues / armadilhas do módulo (repassar ao implementador)

1. **ISPC parallel SIGBUS:** `cargo test -p ph2d-asset-cooker` em paralelo **crasha determinísticamente** (encoders ISPC vendored = global state não-thread-safe). **SEMPRE** `RUST_TEST_THREADS=1 cargo test -p ph2d-asset-cooker`. `asset-ktx2` é parser puro (sem ISPC) — não sofre disso, mas use `RUST_TEST_THREADS=1` por hábito.
2. **slot-env.sh dentro do Bash tool:** `source scripts/slot-env.sh <slot>` num `zsh -c` **não é detectado como sourced** e aborta (cargo não roda). Para isolar `CARGO_TARGET_DIR` use export direto ou rode num shell interativo. `asset-ktx2` tem deps mínimas, então target dir default é seguro mesmo com outra sessão buildando.
3. **W1.T8 deferred honesto:** `ktx2 0.5` e `ctt 0.4.0` são READ-ONLY (zero `pub struct Writer`). `Ktx2Image::premul_intent()` sempre retorna `Unspecified` em KTX2 cooked hoje. 3 paths documentados (patcher / upstream PR / custom writer) na handoff origem §6.2.
4. **Pin hash `gradient_64x64` desligado** em fixtures.rs (assert comentado) — espera W1.T10 canonical runner estabelecer valor cross-platform.

---

## §5 Pre-existing failures cross-session (NÃO fixar — reportar ao owner)

Per [[feedback-audit-scope-discipline]]:
1. `cargo test -p ph2d-editor-core --test architecture_panel_loc_cap` → hierarchy session (`paint_hierarchy_body` 388 > 200 cap).
2. `cargo check -p ph2d-host-desktop` → Painter `PanelEvent::Activated` missing.
3. `crates/ph2d-panel-painter-sidebar` member sem lib.rs (em alguns pontos do dia).

---

## §6 Ship / push (decisão do Enio, via você Coordenador)

- **80 commits locais ahead de origin/main** (todas as sessões somadas). Nenhum push nesta jornada.
- **⚠️ fmt drift workspace-wide:** ~10 arquivos de várias sessões commitados com `--no-verify` ao longo do dia (asset, painel-*, shells/desktop, asset-cooker). `./scripts/ship.sh` (`cargo fmt --all --check`) vai reprovar até alguém rodar `cargo fmt --all`. **Antes de qualquer push, o Coordenador roda `./scripts/ship.sh` e corrige TUDO até verde** (DIRETRIZ §8.1). Push só depois, com babysit do CI.
- O implementador KTX2 **não pusha** — só reporta commit local pronto (CLAUDE.md).

---

## §7 ⚠️ Estado git ATUAL do working tree (você precisa saber)

No momento em que escrevo, o working tree está **sujo por um `git stash pop`
cross-agente em conflito** (NÃO originado pela KTX2):

- **HEAD = `bf39eb6`** (meu commit W1.T9) — íntegro, nada perdido.
- `crates/ph2d-asset-ktx2/src/lib.rs` está em estado **`UU` (unmerged)** com conflict markers `Updated upstream`/`Stashed changes`. Inspeção: o lado "Stashed" é **redundante** com o que já está em `bf39eb6` (mesmo trabalho W1.T9). Resolver para "ours" não perde nada do módulo KTX2.
- **7 arquivos da sessão Painter** (`panel-painter-sidebar` + `tool-painter`) aparecem **staged** no índice — **não são do módulo KTX2**.
- WIP unstaged de outras sessões (editor-core, render/sprite, vector-doc).

**Recomendação de recuperação (você decide / coordena com as sessões donas):**
1. **Ninguém deve `git commit`** com o índice neste estado (misturaria os 7 arquivos Painter).
2. Para o arquivo do MÓDULO KTX2, é seguro resolver mantendo `bf39eb6`:
   `git checkout --ours -- crates/ph2d-asset-ktx2/src/lib.rs && git add crates/ph2d-asset-ktx2/src/lib.rs`
   (não toca nos arquivos das outras sessões).
3. O resto (7 painter staged + o stash) é da sessão que rodou o pop — coordene com ela; **não** rode `git reset`/`stash drop` cego ([[feedback-destructive-git-outside-pasta]]).

---

## §8 ESQUELETO do sub-handoff que VOCÊ (Coordenador) escreve para o implementador

Per pedido do Enio, gere um handoff curto pro implementador KTX2 continuar. Use:

```
═══════════════════════════════════════════════════════════════════
HANDOFF — Implementador KTX2 / Texture Compression · continuação W1
═══════════════════════════════════════════════════════════════════

SANITY CHECK (rode primeiro):
  git log --oneline -3            # HEAD deve conter bf39eb6 (W1.T9 audit)
  git status -sb                  # se houver UU/stash conflict, PARE e me
                                  # avise — NÃO resolva git fora da sua pasta
  RUST_TEST_THREADS=1 cargo test -p ph2d-asset-ktx2   # 39 lib + 2 doctests

SUA PASTA EXCLUSIVA (edite SÓ aqui):
  crates/ph2d-asset-ktx2/  ·  tools/asset-cooker/  ·  crates/ph2d-asset/
  + docs/audits/w1-t*-lens-*.md  +  docs/plans/2026-05-texture-compression-waves.md

NÃO TOQUE: render/ecs/editor-core (Sprite), painter-* (Painter),
  vector-* (Vector), .github/workflows/. Precisou de algo fora? PARE e
  me reporte (sou o Coordenador) — não edite.

TASK: W1.T15 — audit 5-lente final de toda a W1 (gate antes da W2).
  - Catalogue os 6 ciclos audit já feitos (10+ lentes gregas α..ξ).
  - Final integration check do pipeline cook → asset → ktx2.
  - Lentes ainda não usadas: ο (omicron), π (pi), ρ (rho), σ (sigma)...
  - 2 lentes paralelas por round, máx; não recriar R1→R4 (anti-Goodhart).
  - Fixes in-scope inline (feedback-perfection-no-deferrals); adjacent →
    me reporta com owner. Inclui ν-6 (doc drift em ph2d-asset/src/asset.rs:38
    refs ph2d_asset_ktx2::parse inexistente) — pode fixar (é sua pasta).
  Deliverables: docs/audits/w1-t15-lens-{X,Y}-*.md.

VALIDAÇÃO: RUST_TEST_THREADS=1 cargo test -p ph2d-asset-ktx2
           RUST_TEST_THREADS=1 cargo test -p ph2d-asset-cooker  (NUNCA sem a env!)
           cargo test -p ph2d-asset

COMMIT: escopado (git add -- <só seus paths>), nunca -A. Eu (Coord) faço
  ship/push no fim da jornada. Você só reporta "commit local <sha> pronto".

DEPOIS de W1.T15 (me pergunte antes de iniciar):
  - W1.T8.1 (patcher post-hoc PH2D_PREMUL, ~200-400 LOC) — baixo risco.
  - W1.T10/T12/T13 (CI canonical runner) — ALTO RISCO, toca workflows,
    eu renegocio com Enio antes.
═══════════════════════════════════════════════════════════════════
```

---

## §9 Boundaries — o que o Coordenador decide vs PERGUNTA Enio

**Coordenador decide:** atribuição de pastas aos 3 implementadores; ordem das
tasks W1 restantes; resolução do git conflict do working tree (§7); lentes
específicas do W1.T15.

**Coordenador PERGUNTA Enio:** push para origin (80 commits ahead); estratégia
do W1.T10 workflow (`spike.yml` vs separado); escolha entre os 3 paths do
W1.T8.1; qualquer amendment a contrato FROZEN.

---

## §10 Memórias-âncora (releia antes de agir)

- [[project-ktx2-phase2-v4-accepted-2026-05-27]] — ADR-0055-v4 Accepted, escopo W1.
- [[feedback-perfection-no-deferrals]] — gaps in-scope viram trabalho da sessão atual.
- [[feedback-audit-scope-discipline]] — bug em crate adjacent → handoff ao owner, não fixo.
- [[feedback-audit-lens-diversity]] — rotacionar lentes adversariais; gates > claims verbais.
- [[feedback-parallel-agent-commit-collision]] + [[feedback-scoped-commit-shared-index]] — `git status` antes de stage; `git add -- <paths>` específico; nunca `-A`.
- [[feedback-destructive-git-outside-pasta]] — nunca git destrutivo fora da sua pasta sem coordenar.
- [[feedback-fanout-registry-init-friction]] — fmt drift de agentes paralelos = caso `--no-verify` legítimo de dia (ship.sh pega no fim).

---

**Resumo de uma linha:** KTX2 W1 ~85% pronto, último audit (W1.T9) fechado em
`bf39eb6`, próxima task = W1.T15 audit final; working tree sujo por stash-pop
alheio (§7) que NÃO é do módulo; escreva o sub-handoff §8 pro implementador.
