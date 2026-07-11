# Handoff de integração — linha `line/anim` (Timeline W4.T5 + fix auto-key/play)

**Data:** 2026-07-11 · **Regime:** Modo L (workstation) · DIRETRIZ §1.5.9
**Status:** linha pronta — **NÃO integrada, NÃO shipada** (aguarda ordem explícita do Enio).

> Escopo desta rodada: **(a)** cauda da W4 → T5 (aposentar os scaffolds de debug da timeline);
> **(b)** bug fix — **auto-key criava um keyframe por quadro no Play** (§7). W4.T8 (gate batched)
> rodado aqui embaixo. A Timeline v1 (W0–W3, W4.T1–T3, Summary) já estava **integrada em main**
> antes desta linha reabrir — nada a re-landar.

---

## 1. Identidade

- **Branch:** `line/anim`
- **Commits à frente de main (base `1c7c9a22` == main HEAD):**
  1. `b7f62238` — `chore(timeline): W4.T5 — aposenta timeline_smoke + hook KeyB; tecla B livre`
  2. `50211c5d` — `docs(anim): handoff de integração W4.T5` (este doc)
  3. `bd412e01` — `fix(timeline): auto-key inerte durante o play` (§7)
  4. + o commit deste update do handoff
- **Base do fork:** `1c7c9a22` — **== main HEAD atual** → integração é **fast-forward puro** (`--ff-only` trivial, sem rebase, sem drift).

## 2. Foundational / compartilhado tocado + por quê

Mudança confinada ao **shell** (`shells/desktop`) + docs. **Nenhuma crate foundational de código-núcleo** (`ph2d-core`/`editor-core`/`tokens`/`host`) tocada; **nenhuma crate da timeline** (`ph2d-anim`/`ph2d-timeline`/`ph2d-panel-timeline`) tocada.

| Arquivo | Mudança | Natureza |
|---|---|---|
| `shells/desktop/src/render_loop/timeline_smoke.rs` | **DELETADO** (169 LOC) | remoção |
| `shells/desktop/src/render_loop/mod.rs` | remove `mod timeline_smoke;` + desembrulha o `if timeline_smoke::enabled() {…} else {…}` (o corpo do `else` — caminho off-by-default — vira incondicional) + 1 comentário | remoção / reindent |
| `shells/desktop/src/input_handlers.rs` | remove o arm `KeyCode::KeyB` (bind demo `SpriteAnimation`) + helper `demo_spin_clip` (43 LOC) | remoção |
| `shells/desktop/Cargo.toml` | reescreve 2 comentários de dep (`ph2d-anim`/`ph2d-timeline`) que citavam o smoke/KeyB; **deps inalteradas** | comentário |
| `shells/desktop/src/app_state.rs` | doc-comment `See render_loop::timeline_smoke` → `timeline_bridge` | comentário |
| `shells/desktop/src/render_loop/timeline_bridge.rs` | doc-comment sobre KeyB → "bind programático" | comentário |
| `shells/desktop/src/render_loop/autokey_pass.rs` | **§7 fix** — `let armed = armed && !playhead.is_playing()` (1 linha) + comentário + teste de regressão | 1-liner + teste |
| `CLAUDE.md` §5 · `docs/Timeline/01_plano_timeline_ui.md` | W4.T5 marcada ✅ + gotcha auto-key/play + prosa histórica | doc |

**CLAUDE.md tocado** (arquivo compartilhado): edição **aditiva pontual** na entrada §5 Timeline (nota "W4.T5 landou"), sem colidir com outras seções. Mergiraf resolve resíduo textual se outra linha também editar §5.

**Comportamento default preservado byte-a-byte:** o smoke era `PH2D_TIMELINE_SMOKE=1` (off por default), então produção sempre executou o corpo do `else`. O `apply_sprite_animations` **permanece** (caminho por-componente programático, no-op quando nada carrega `SpriteAnimation`).

## 3. Símbolos que podem COLIDIR com outra linha

**Nenhum.** Zero id/const/variant/token **novo** — esta rodada só **remove**. `mod timeline_smoke` retirado; `KeyCode::KeyB` liberado (cai no `_ => {}`). Nada a grepar por mesmo-símbolo.

## 4. Contratos congelados encostados (§4)

**Nenhum.** `Tool`/`NodeOp`/`AnimValue`/`PanelEvent`/vector-doc intactos. Nenhum ADR necessário.

## 5. O que SÓ o `ship.sh` pega (o gate de integração não roda)

- **fmt:** rodei `rustup run 1.95 cargo fmt -p ph2d-host-desktop` (pin canônico); os arquivos tocados estão canônicos. A linha forkou de main atual (1c7c9a22), **sem** drift pré-fork.
- **machete/deny/RUSTSEC:** **nenhuma dep nova** (só removi código e comentários) → sem risco de dep órfã. `ph2d-anim`/`ph2d-timeline` seguem usados no shell (mod.rs/timeline_bridge/apply). advisory-db pode ter RUSTSEC novo desde o fork — o ship reconfirma.
- **clippy latente:** rodei `clippy -p ph2d-host-desktop --all-targets -- -D warnings` = **verde** (pega o dead-code/unused que o `check` plain não eleva). Como é remoção, o risco residual é ~nulo.
- **typos:** só toquei prosa em pt-BR + código; sem strings de UI novas.

## 6. Ordem/dependências + o que smoke-testar

- **Ordem:** 1 commit único, sem dependência entre commits. FF trivial.
- **Smoke do Enio (DoD da T5):**
  1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop`
  2. Abrir o painel Timeline, criar track + keys, **play/scrub** → a cena anima (prova viva substituiu o smoke).
  3. Pressionar **B** → **nada acontece** (tecla liberada; antes fazia "bound spin to N sprites").
  4. `PH2D_TIMELINE_SMOKE=1 cargo run …` → o env-flag **não faz mais nada** (não substitui a cena demo).
  5. **(§7)** objeto animado + **AutoKey armado** + **Play** → a timeline **não** cria keyframes por quadro; pausar e mover ainda grava normalmente.
- **O que NÃO foi smokado por mim:** o run visual (headless não cobre a tela). Confiança alta: comportamento default byte-idêntico + 206/206 testes do shell verdes (o fix da §7 tem teste + mutação dirigida).

---

## Gate batched W4.T8 (rodado nesta linha)

- ✅ `cargo check -p ph2d-panel-timeline` · `cargo check -p ph2d-host-desktop` (verde, sem warnings)
- ✅ `cargo clippy -p ph2d-host-desktop --all-targets -- -D warnings` (verde)
- ✅ `cargo nextest run -p ph2d-host-desktop` → **206/206** (impacted-set = só o shell; nenhuma crate timeline tocada; +1 = teste de regressão da §7)
- ✅ Gates de arquitetura relevantes ao diff, explícitos:
  `file_loc_caps::{shell_files_respect_hr18_loc_cap, loc_cap_exceptions_inventory}` ·
  `architecture_no_per_tool_branch_in_render_loop` (o desembrulho **não** criou branch por-tool) ·
  `architecture_no_downcast_to_concrete_tool_in_shell`
- ⏭️ Perf da cena de referência: **N/A** — remoção não adiciona custo de paint (mod.rs encolheu).

### Audit ≥2 lentes (DIRETIVA §3)

**LENTE 1 — correção da remoção (nenhum caminho vivo perdido).**
CLAIM: remover o smoke/KeyB não altera nenhum comportamento de produção.
TRAÇO: `render_loop/mod.rs` — `timeline_smoke::enabled()` lê `PH2D_TIMELINE_SMOKE` (off por default) → produção sempre rodou o `else`; agora incondicional (mesma ordem: ppm → default_filter → cooked_texture_bridge → apply_sprite_animations → timeline_bridge::run → sim_extract::run, lidas linhas 690–816). `input_handlers.rs` — `KeyCode::KeyB` era bind de DEBUG (toast "bound spin to N"), não produção; removido → `_ => {}`.
ASSERÇÃO-VERMELHA: se eu tivesse cortado o caminho errado, `cargo check` passaria mas a cena pararia de animar — pego pelos testes headless de apply (`ph2d-anim/tests/playhead_drive.rs`, INTACTOS) + gate `no_per_tool_branch_in_render_loop` (verde) + seam tests do painel (INTACTOS, crate não tocada).
NÃO-CHECADO-PELA-COMPILAÇÃO: que B ficou de fato inerte + timeline anima via painel → smoke do Enio (§6).
LOC LIDAS: `timeline_smoke.rs` (169) + `input_handlers.rs` 400–590 + `mod.rs` 690–856 + 3 arquivos de comentário.

**LENTE 2 — wiring / órfãos (nenhuma referência pendente, nenhum símbolo morto).**
CLAIM: zero referência dangling ao smoke/KeyB/`demo_spin_clip`; nenhum import/dep órfão.
TRAÇO: grep workspace `timeline_smoke|demo_spin_clip|PH2D_TIMELINE_SMOKE|KeyCode::KeyB` em `*.rs`/`*.toml` = **vazio**. `mod timeline_smoke;` removido. clippy `--all-targets -D warnings` verde (pegaria unused import / dead_code).
ASSERÇÃO-VERMELHA: `mod timeline_smoke;` sem o arquivo → E0583 (compilação vermelha); `demo_spin_clip` órfã → clippy `-D warnings` vermelho. Ambos verdes.
NÃO-CHECADO: menções em `.md` são prosa histórica intencional ("aposentado na W4.T5").
LOC LIDAS: grep completo + os sites de edição.

---

## 7. Bug fix — auto-key criava keyframe por quadro no Play (`bd412e01`)

**Sintoma (reportado pelo Enio):** objeto animado + botão **AutoKey** marcado → ao dar **Play**, a timeline gravava **um keyframe por quadro**.

**Causa:** o passe `autokey_pass` grava quando a pose "saiu da curva" (`world != curve(t)`). Mas o apply escreve `world = curve(t **raw** do playhead)` a cada frame, enquanto o diff compara contra `curve(t **snapado**)`; sob frame-snap / drift de float os dois divergem e a comparação exata dispara — key espúria por frame tocado. O invariante anti-feedback (`world==curve → não grava`) só vale em `t` FIXO, não durante o play (t avança).

**Fix (1 linha, `apply_samples`):** `let armed = armed && !playhead.is_playing();` — durante o play a pose é dirigida pela animação, não pelo usuário; auto-key fica inerte. O `baseline` continua avançando (como no caso disarmed) → pausar no meio do play não lê a pose como salto. Performing (gravar durante o play) é **W5**, não v1.

**Superfície:** só `shells/desktop/src/render_loop/autokey_pass.rs` (crate do shell). **Zero símbolo novo, zero contrato tocado.** `Playhead::is_playing()` já existia.

**Prova:** teste `playing_does_not_auto_key_even_when_the_pose_looks_off_its_curve` (pose off-curve + armado + PLAYING → doc intocado; controle pausado → grava). **Mutação dirigida** (remover `&& !playhead.is_playing()`) reproduz o bug: insere a key espúria no play → teste vermelho. Restaurado.

---

## Cauda da W4 ainda aberta (para a próxima rodada — decisão do Enio)

- **W4.T4** — docar a timeline no `motion_timeline_slot` quando o split do Motion está ativo (coordenação leve com Motion).
- **W4.T7** — unificar o relógio: `MotionTransport` derivar do `Playhead` + remover transporte duplicado em `motion_bridge.rs` (coordenação leve com Motion).
- **W4.T6 (= B5)** — save de projeto unificado cena+timeline + id estável de entity (**deferido** — cross-cutting, esforço coordenado, não landar solo).
- **W5** — backlog pós-v1 (Enio prioriza).
