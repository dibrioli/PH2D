# Handoff de integração — linha `line/anim` (Timeline W4.T5 + fix auto-key/play + speed graph)

**Data:** 2026-07-11 · **Regime:** Modo L (workstation) · DIRETRIZ §1.5.9
**Status:** linha pronta — **NÃO integrada, NÃO shipada** (aguarda ordem explícita do Enio).

> Escopo desta rodada: **(a)** cauda da W4 → T5 (aposentar os scaffolds de debug da timeline);
> **(b)** bug fix — **auto-key criava um keyframe por quadro no Play** (§7); **(c)** feature W5 —
> **speed graph** (vista de velocidade + edição, §8). W4.T8 (gate batched) rodado aqui embaixo. A
> Timeline v1 (W0–W3, W4.T1–T3, Summary) já estava **integrada em main** antes desta linha reabrir.

---

## 1. Identidade

- **Branch:** `line/anim`
- **Commits à frente de main (base `1c7c9a22` == main HEAD):**
  1. `b7f62238` — `chore(timeline): W4.T5 — aposenta timeline_smoke + hook KeyB; tecla B livre`
  2. `50211c5d` — `docs(anim): handoff de integração W4.T5`
  3. `bd412e01` — `fix(timeline): auto-key inerte durante o play` (§7)
  4. `0cfadd02` — `docs(anim): handoff — adiciona fix auto-key/play`
  5. `e78c0394` — `feat(timeline): speed graph — velocity view + editable speed handles` (§8)
  6. + o commit deste update do handoff
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

Da W4.T5 + fix: **nenhum** (só remoção + 1-liner). Do **speed graph (§8)**, símbolos NOVOS a grepar por mesmo-símbolo:

- **`TIMELINE_SPEED`** — chrome id novo em `ph2d-editor-core::ids::chrome::timeline` (`hash_node_id("timeline.speed")`), append-only após `TIMELINE_SNAP`. Re-export glob (`pub use timeline::*`) → também na lista explícita do painel `ids.rs`.
- **i18n `panel.timeline.speed` = "Speed"** — nova arm em `ph2d-i18n/src/lib.rs` após `panel.timeline.snap`.
- **`ph2d-timeline::speed`** — módulo irmão NOVO (`sample_speed`/`speed_extent`/`segment_endpoint_speed`/`out_handle_y_for_speed`/`in_handle_y_for_speed`) + re-exports no `lib.rs`. Nome de módulo isolado, sem colisão.
- **`ph2d_anim::Interp::slope(u)`** — método novo (append-only) + helpers privados `solve_bezier_param`/`bezier_slope`/`bezier_start_gradient`/`bezier_end_gradient`/`eased_slope` em `curve.rs` (§8.1 fix A). `Interp` NÃO é contrato gateado; o solver do `remap` foi refatorado **byte-idêntico** (goldens 62/62).
- **`TimelinePanelState.speed_view: bool`** — campo novo (panel-local). Reusa o `CurveHandle`/`HandleDrag` existentes (NÃO adiciona variant em `TimelineHitKind` — contrato de dispatch intacto).

Nenhum contrato congelado (§4) tocado (o `TimelineHitKind` não é gateado; nenhum `NodeOp`/`Tool`/`AnimValue`).

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
  6. **(§8)** anime uma faixa, expanda o graph (twirl), marque **Speed** na barra de transporte → a band mostra a curva de **velocidade** (linha-zero no meio); selecione uma key e arraste um handle na vertical → a velocidade daquele trecho muda ao vivo (a tangente/easing por baixo se reafina).
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

## 8. Feature W5 — speed graph (`e78c0394`)

Uma **2ª vista do graph editor** que plota a **velocidade** (`d(value)/dt`) da curva (padrão AE/Cavalry/Blender). Toggle **Speed** panel-local na barra de transporte alterna toda band expandida entre valor e velocidade; arrastar um speed-handle reafina a tangente.

**Superfície:** `ph2d-timeline` (novo módulo `speed.rs` + 4 re-exports) · `ph2d-panel-timeline` (`state.speed_view`, `transport`/`event`/`populate` p/ o toggle, `graph_paint`/`graph` branch view+edit, testes) · `ph2d-editor-core` (id `TIMELINE_SPEED`, append-only) · `ph2d-i18n` (`panel.timeline.speed`). **Sem tocar o shell** (o painel republica o snapshot como sempre). Símbolos novos listados em §3.

**Modelo (o que auditar):**
- `sample_speed` = `dv·P'(u)/span`, diferenciando a **easing pura** `Interp::remap` em espaço-u normalizado — a MESMA fn que o runtime toca (WYSIWYG), uniforme p/ Hold/Linear/Bezier/Eased, e **nunca cruza uma key** (Hold lê zero, sem spike de fronteira). `speed_extent` sempre inclui a linha-zero.
- Edição = inverso EXATO velocidade→inclinação da tangente (`y1/x1` no início, `(1-y2)/(1-x2)` no fim), mantendo a **influência x fixa**; segmento flat (`dv=0`) → sem velocidade a escalar → mantém o handle. Reusa `CurveHandle`/`HandleDrag` (só um conjunto de handles pinta por frame — sem colisão de id).

**Prova (DIRETIVA §3–5):** 9 goldens de math em `speed.rs` (linear=rate const; ease slow/fast/slow; hold=0 sem spike; inverso exato dos dois handles; round-trip no sampler numérico) + **3 seam comportamentais** — `speed_toggle_flips_the_view_locally` (toggle flipa `speed_view`, zero evento de shell), `dragging_a_speed_handle_retunes_the_tangent_to_that_velocity` (drive real do gesto → `SetInterp` com slope na velocidade-alvo + influência preservada), `a_speed_drag_on_a_flat_segment_keeps_the_handle`. **Mutação dirigida** nos 2 invariantes (neutralizar o branch de `resolve_drag` → retune falha; neutralizar o flip do toggle → toggle test falha). ASSERÇÃO-VERMELHA presente em cada claim.

**O que NÃO foi smokado por mim:** o render visual do painel (headless não pinta a tela). Ver §6.6.

### §8.1 Auditoria padrão-ouro (a pedido do Enio, 2026-07-11) — 3 achados, 3 fixes

Referências: **Chromium `ui/gfx/geometry/cubic_bezier.cc`** (a implementação de referência do CSS
cubic-bezier: derivada por chain rule paramétrica `y'(s)/x'(s)` em `SlopeWithEpsilon` + cascata
`InitGradients` p/ endpoints degenerados) e **semântica do speed graph do After Effects** (Adobe
helpx + Creative COW: para propriedade 1D/dimensão separada o gráfico é **com sinal**; magnitude
só em posição espacial combinada).

| # | Achado | Fix |
|---|---|---|
| A | `sample_speed` usava **diferença finita** (`DIFF_U=1e-3`) onde a referência computa a derivada **analiticamente** — e o repo já tinha `bezier_axis_deriv`. Violação da regra "porte o algoritmo de referência" (DIRETIVA §1) | `Interp::slope(u)` novo em `ph2d-anim` (port do Chromium: chain rule no MESMO solver Newton do `remap` — `solve_bezier_param` extraído byte-idêntico — + cascata `InitGradients` p/ 0/0 + ±∞ em tangente vertical; `Eased` = diferença central de `eval` sem o clamp de handle). `sample_speed` = `dv·slope(u)/span` |
| B | O dot de speed do **Hold** lia **3·rate** no lado in (derivado do chord dos `tangent_handles`, que é convenção de DESENHO do value view, não derivada) — um Hold é flat e vale **0** nas duas pontas | `segment_endpoint_speed(k0,k1,which)` via `slope(0\|1)` — Hold = 0 ✓; teste red-assertion `a_holds_endpoint_dots_read_zero_speed` + mutação (Hold→1.0 no slope → vermelho) |
| C | Dots de speed flutuavam a 1/3–2/3 do segmento (posição dos handles de VALOR); a convenção AE ancora os speed handles **nas keyframes**. E tangente vertical (`x1=0`) mostrava dot em 0 (mentira) com a curva espicando | Dots relocados p/ `(t0, out)` / `(t1, in)`; non-finite (vertical) não pinta dot nem envenena o fit (`speed_extent`/curva filtram); `speed_extent` agora inclui os dots de segmento selecionado (espelho do `drawn_extent`) — pega endpoint íngreme que o grid de 2px perde |

**Confirmados pela auditoria (sem mudança):** velocidade **com sinal** = comportamento AE p/ 1D ✓ ·
inversos `out/in_handle_y_for_speed` = álgebra exata dos endpoint slopes (`y1/x1`, `(1-y2)/(1-x2)` —
os mesmos do Chromium) ✓ · derivação per-segmento (Hold sem spike de fronteira) ✓ · edição mantém
influência x (weighted = W5) ✓. Extra: alternar a vista derruba drag em curso (fecha o bracket).
Verificação: goldens de `ph2d-anim` **byte-idênticos** pós-refactor do solver (62/62) + 4 goldens
novos de slope + 154 anim+timeline + 290 total com painel + clippy `--all-targets` verdes.

---

## 9. Rodada de bugs de UX (Enio, smoke 2026-07-11) — 4 fixes

| # | Bug (reportado) | Causa | Fix |
|---|---|---|---|
| B1 | Multi-seleção via Summary + R-click easing não aplicava a todos | `column()` em `timeline_presets.rs` fazia `ClearSelection` incondicional — descartava a seleção e aplicava só na coluna clicada | Coluna cujas keys **já estão todas selecionadas** → `SetSelectedInterp` na seleção INTEIRA (espelho exato do scope `Key`); fora da seleção → comportamento antigo. Teste `a_selected_column_retunes_the_whole_selection_not_just_itself` + mutação |
| B2 | Deletar objeto animado não removia as curvas da timeline | Design antigo: binding `missing` ganhava badge e a row FICAVA | Rows de binding missing **somem do snapshot** (dado fica dormente no doc) + **heal por nome**: `refresh_and_heal_bindings` (ph2d-timeline) + `timeline_persist::upkeep` por-frame (zero-alloc se nada missing) re-stampa `wire_id` por `Name` e reconecta quando um objeto de mesmo nome reaparece — **delete + undo global cura a animação** (o undo respawna com bits NOVOS e mesmo Name; antes ficava quebrada pra sempre). Testes: persist unit + snapshot filter + upkeep e2e |
| B3 | Sem AutoKey, impossível posar o objeto pra criar o 2º key (snap-back) | O apply reescreve `world = curve(t)` todo frame; só a entidade EM drag era pulada — ao soltar o gizmo a pose voltava | **Pin de pose deslocada** (Blender-style): desarmado+pausado, pose bound off-curve entra em `AutokeyState.displaced` e o apply pula (via `apply_from_doc_except` agora com predicado). Pin morre quando o playhead move ou a pose volta à curva (K/undo). Teste `a_disarmed_displaced_pose_pins_the_entity_for_a_manual_k` + mutação |
| B4 | Selecionar key individual selecionava a coluna/summary inteira | `column_lock` **fechado** por default roteava todo clique de key pro gesto de coluna | Default **aberto** (clique = só a key); o padlock fecha p/ quem quer colunas alinhadas. Testes do interact ajustados |

**Superfície:** `ph2d-timeline` (`apply_from_doc_except` virou predicado `skip: impl Fn(u64)->bool` — API pública minha, único caller externo era o bridge; `persist::refresh_and_heal_bindings` novo; snapshot filtra missing) · `ph2d-panel-timeline` (default do lock + testes) · shell (`AutokeyState` agrupa baseline/drag/displaced — campos do `App` consolidados; bridge limpa o pin na mudança de tempo e chama `upkeep`; `timeline_presets.rs` dividido — `pick_tests` extraído p/ `timeline_presets_tests.rs` pelo LOC cap 600). **Zero contrato congelado; zero id novo.** dhat: `apply_from_doc_is_zero_alloc_steady_state` segue verde (upkeep só aloca com binding missing).

---

## 10. Feature W5 — weighted / value-space tangents

**O quê:** handles com **peso** — `Interp::BezierW{x1,dy1,x2,dy2}`, bézier no plano `(u, valor)`: x = influência (fração, clamp CSS, MESMO solver Newton do `remap`), **dy = offset ABSOLUTO em valor** (semântica AE keyframe-velocity / Blender F-curve). Fecha o gap documentado do `handle_coords` ("Value-space tangents... W5 backlog"): **segmento flat agora curva**, e speed-edit em flat funciona.

**Compat (o ponto de auditoria nº 1):** variant **apendado por ÚLTIMO** no enum → índices postcard estáveis → **saves v1 seguem legíveis** (DOC_VERSION inalterado); um arquivo NOVO com keys W não abre em build antigo (forward-incompat esperada). O caminho de sampling **legado é byte-idêntico**: `interpolate` ramifica SÓ no W (`lerp(remap)` intocado); goldens de `ph2d-anim` + o golden bit-a-bit `sample_keys ↔ Track::sample` provam.

**Superfície:** `ph2d-anim` (variant + motor no módulo irmão `curve_weighted.rs` + `Interp::value/value_slope` + `slope_tests` extraído p/ `curve_slope_tests.rs` pelo LOC cap) · `ph2d-timeline` (`segment_handle_points` — posições value-space p/ QUALQUER interp; producers `weighted_with_handle`/`weighted_with_endpoint_speed`; `sample_keys` lockstep; speed.rs migrado pro funil `value_slope`; `out/in_handle_y_for_speed` REMOVIDOS — substituídos pelo producer) · painel (`resolve_drag` produz W nos dois modos; paint via `segment_handle_points`; o freeze do flat morreu). **Zero contrato congelado** (`AnimValue`/`sample(t)` intocados; `Interp` não é gateado); zero id novo.

**Semântica de UX:** todo drag de tangente (valor E speed) converte pro W — **lossless** (o lado não-arrastado mantém a posição exata em que é desenhado; equivalência N↔W provada por teste). Presets (Hold/Linear/famílias/Custom) seguem normalizados — um preset é uma FORMA, independente dos valores.

**Prova:** 5 testes no motor (flat-bulge — a red-assertion da feature · equivalência lossless · derivada vs fd · cascata degenerada/vertical · legado == caminho antigo) + golden `sample_keys` estendido com key W + drags do painel re-especificados (flat toma o valor · overshoot em dy · speed em flat) + round-trip serde com W + **2 mutações dirigidas**: (a) remover o branch W do `interpolate` → o golden bit-a-bit acusa o desalinhamento painel↔runtime; (b) zerar o dy no motor → flat-bulge vermelho. 506/506 nas 4 crates; clippy `--all-targets` verde. O produtor de speed tinha um bug pego pelo compilador durante o build (retunava as DUAS pontas) — corrigido + teste reforçado (a outra ponta não se move).

**§10.1 — Speed handles AE-style (braço de influência, mesmo dia; smoke do Enio pediu paridade com o AE):** cada endpoint no speed graph agora desenha **âncora** no ponto de velocidade do key + **braço horizontal** cujo comprimento É a influência + **ponta arrastável** (`speed_handle_tip` em ph2d-timeline dá a posição; hit na PONTA, mesmo id `CurveHandle`). A ponta edita em **2D como no AE**: vertical = velocidade, horizontal = influência — um único producer `weighted_with_speed_handle(k0,k1,which,t,v)` (influência clampada ao segmento e longe do zero degenerado, `MIN_INFLUENCE = 1e-3` = os 0.1% do AE); o lado oposto NUNCA se move. `weighted_with_endpoint_speed` (vertical-only, existiu por ~1 commit) foi **removido** — superseded. Testes: round-trip da ponta (influência mantida em drag vertical) · drag horizontal seta a influência · mutação no eixo horizontal morde. 507/507.

---

## 11. Feature W5 — time remap (modelo AE, por objeto)

**O quê:** `PropKind::TimeRemap = 6` — uma track **"Time"** keyável por entidade mapeando tempo do playhead → tempo-fonte (segundos→segundos): slope < 1 = slow-mo, > 1 = acelera, flat = **freeze**, descendo = **reverse**. `apply_from_doc` computa `remapped_time` por entidade (scan linear **zero-alloc** — o gate dhat foi estendido com bindings de remap e segue verde) e TODAS as outras tracks da entidade amostram nesse relógio. A track de remap nunca escreve cena (consumida como clock).

**Autoria:** a track é UI **normal** — dope-sheet, graph editor, weighted tangents e speed graph funcionam nela de graça (o valor é segundos). **+Track** ganhou "Time" (`TIMELINE_ADDPROP_TIME`, i18n `panel.timeline.prop.time`; popup/populate/wiring derivam do array — 7º botão automático). **K semeia identidade** (`valor = t`) em track vazia — bindar Time não muda nada até editar — e **na-curva** quando há keys (`key_value_for` no bridge). `sample_prop_value(TimeRemap) = None` → **auto-key nunca toca** o remap (`ALL` continua sendo a pose de 6; `PoseSample=[;6]` intacto).

**Símbolos novos (grep de colisão):** `PropKind::TimeRemap = 6` (discriminante wire, apendado) · `TIMELINE_ADDPROP_TIME` (`hash("timeline.addprop.time")`) · i18n `panel.timeline.prop.time` · `ADDPROP_BUTTONS` 6→7 · `key_value_for`/`remapped_time`. Zero contrato congelado. **NÃO confundir** com `motion.time_remap` dos Motion Nodes (escopo de cook de sub-árvore — outro sistema, outra crate).

**Prova:** 4 testes de apply (2× speed · freeze · reverse · identidade/per-entity/nunca-escreve-cena) + seed do K (identidade + na-curva) + id novo no shell + dhat estendido zero-alloc + **mutação dirigida** (neutralizar `remapped_time` → freeze/reverse vermelhos). 1259/1259 nas 5 crates; clippy `--all-targets` verde.

**⚠️ §11 + §11.1 eram INSUFICIENTES (Enio, 2026-07-11): o Time remap ainda anulava a animação de posição.** Causa-raiz + fix definitivo na **§13** (commit `72803d18`) — o seed do K agora é a MESMA transform da amostragem. **Smoke do Enio pendente** (§13.3); até lá o fechamento é condicional.

**§11.1 — Fix "Time bugado" (smoke do Enio: criar key Time travava a autoria de pose):** duas metades. **(a)** `remapped_time` extrapolava por HOLD fora dos keys — UM key semeado pelo K congelava o relógio da entidade pra sempre (0 keys = identidade, 1 key = freeze: descontinuidade). Agora **extrapola em slope 1** (identidade deslocada pelo key de borda); exceção: **último key `Hold` = freeze-frame** (o freeze do AE sobrevive — o teste antigo de freeze usava Hold e passou intacto). **(b)** autoria em tempo cru vs apply em tempo remapeado: auto-key (diff + insert), pin de pose deslocada e K agora usam o **relógio da entidade** (`remapped_time`, agora `pub`; `key_insert_time` no bridge) — o key landa no tempo FONTE, onde o apply amostra, e a pose gruda. A track Time em si segue keyando no tempo do playhead. Consequência de exibição: sob remap ≠ identidade, um K no playhead `t` desenha o key da cena na régua no tempo-fonte (as tracks são autoradas em tempo-fonte — modelo precomp do AE). **4 mutantes dirigidos** (hold-extrapolation · diff armado em t cru · pin em t cru · K em t cru) → cada um derruba seu teste. 51 + 205 verdes; clippy + LOC ok. Commit `56217b44`.

---

## 12. Feature W5 — roving keys (AE "rove across time")

**O quê:** key marcado **roving** perde o tempo autorado — só o VALOR é autorado; `Track::resolve_roving` (novo módulo `rove.rs`, FILHO de `track.rs`) deriva o tempo pra **velocidade de valor constante** entre os vizinhos pinados (tempo ∝ |Δv| acumulado — total travel, não displacement; travel zero = uniforme; **keys de borda nunca rovam** — flag mantida, ignorada). Idempotente, sem alloc.

**Dado:** flags = **vec paralelo persistente** no `Track` (padrão dos `ids`, mas entra no `PartialEq` + serde via `TrackData.roving` apendado com pad-on-load) — `Key` fica literal puro nos ~54 sites. **`DOC_VERSION` 1→2** (postcard posicional: v1 é REJEITADO pelo gate de versão, não deslido; nenhum writer de produção existia). Mantido em 6 sites de mutação (insert/remove/merge-moved/remove_keys/resort zip3/clone).

**Choke points:** `edit()` no intent.rs re-resolve após TODA mutação por intent (mesmo undo step — undo restaura tempo autorado E flag em 1 passo) + `doc.upsert_key` re-resolve (caminho direto do auto-key). Efeito: editar valor de key roving, arrastar vizinho pinado, K/auto-key na track — tudo refloa na hora.

**UI:** menu R-click ganhou a 7ª linha **"Rove Across Time"** (`CTX_MENU_TL_ROVE`, tabela única `TIMELINE_SEGMENT_MENU` 6→7 — overlay/pre_populate/gate anti-item-morto andam juntos; `is_leaf` no `timeline_segment.rs`). Shell: `Preset::Rove` = **toggle** (all-roving → off; misto → converge ON), escopo espelha os presets (key solto = per-key `SetRove` sem mexer na seleção · key selecionado = `SetSelectedRove` em massa · coluna Summary = seleciona e aplica). Painel: roving desenha **circulinho** no dope-sheet (vs diamond, convenção AE) e **âncora menor** no graph; hit targets inalterados; `KeyView.roving` no snapshot (zip a 3, buffer reuse — dhat verde). Arrastar a âncora de um key roving no graph: o valor pega, o tempo re-deriva no mesmo frame (a componente horizontal "volta" — tempo não é seu pra arrastar; pra pinar, desmarque no menu — un-rove **pina no tempo derivado**, nada se move).

**Símbolos novos (grep de colisão):** `Track::{roving,is_roving,set_roving,resolve_roving}` · `Clip::resolve_roving` · `TimelineIntent::{SetRove,SetSelectedRove}` · `CTX_MENU_TL_ROVE` (`hash("ctx_menu_tl_rove")`) · `KeyView.roving` · `Preset::Rove` · consts `ROVE_DOT_R`/`ROVE_ANCHOR_R` · arquivo novo `timeline_presets_menu_tests.rs` (preset_tests extraído pelo LOC cap). Zero contrato congelado.

**Prova:** 5 testes no motor (proporcional · run acumulado · reversing=travel · zero-travel uniforme + bordas pinadas · flags seguem keys por edits+serde) + 3 de intents (reflow por valor/vizinho/upsert · 1 undo step + un-rove pina · bulk) + 3 do menu (resolve+toggle · misto converge · coluna) + **mutação dirigida 4/4** (resolvedor morto → 4 vermelhos · choke edit() · hook upsert · direção do toggle). Suítes das 5 crates verdes + clippy `--all-targets` + LOC caps + dhat. Commit `5b8b6e7f`.

**Fila do Enio: vazia — aguardando smoke (time remap §13 + roving) e próxima ordem.**

---

## 13. Bug fix — Time remap anulava a animação (3º e definitivo, `72803d18`) — SUPERSEDE a metade (b) do problema da §11.1

**Sintoma (Enio, 2026-07-11):** bindar **Time** e criar keyframes (K em t=0, scrub, K de novo — o fluxo natural de 2 âncoras) **congelava** a posição e todas as outras tracks da entidade.

**Causa-raiz (a inconsistência que a §11.1 deixou passar):** o **seed do K** (`key_value_for`, branch `TimeRemap`) usava `tr.sample(t)` = **flat-clamp** fora do intervalo de keys, enquanto a **amostragem** (`remapped_time`) extrapola em **slope 1** (§11.1a). K@0 numa track vazia semeia identidade `(0,0)`; K@2 com 1 key flat-clampava em `0` → track `{(0,0),(2,0)}` = **remap PLANO** = todas as tracks da entidade amostram na fonte 0 = "animação anulada".

**Fix (`timeline_bridge.rs`, `key_value_for`):** o seed do K virou **a MESMA função** da amostragem — `ph2d_timeline::remapped_time` (identidade em track vazia · na-curva entre keys · **slope-1 fora** · Hold = freeze deliberado · skip de binding missing · clamp ≥ 0 — tudo herdado por construção, [[feedback_derived_coordinate_seed_must_match_sample]]). Zero símbolo novo, zero contrato tocado; superfície = 1 função no shell.

**Prova (o alvo irrefutável que faltou nos 2 fixes anteriores):**
1. `time_remap_double_k_must_not_freeze_position` — dirige o caminho REAL do K do shell (`key_value_for` + `key_insert_time` + `apply_from_doc`): **vermelho antes** (`x@1 = 0`, congelado) → **verde depois** (`x@1 = 2.5`, `x@3 = 7.5`, identidade). O run vermelho pré-fix é a mutação dirigida (restaurar o flat-clamp reprova).
2. `k_past_a_hold_freeze_seeds_the_frozen_clock` — o freeze do **Hold** segue deliberado: K além de um último key Hold semeia a fonte congelada e os keys de cena landam no relógio congelado.
3. In-range **não regride**: `k_seeds_a_time_remap_key_on_its_curve_or_at_the_identity` (na-curva + identidade em track vazia) passou intacto.
4. **Varredura dos outros autores de valor de Time:** graph editor **nunca emite `AddKey`** (só Move/SetKeyValue/SetInterp — valor user-driven) · auto-key **duplamente guardado** (`PropKind::ALL` = pose de 6 sem TimeRemap E `sample_prop_value(TimeRemap)=None`) · Duplicate/Paste preservam valor **verbatim** (`duplicate_keys`/`upsert_key(t, ck.value, ck.interp)`) — nenhuma outra fonte de seed sintetizado.

**Gate:** `rustup run 1.95 cargo fmt` ✓ · nextest `ph2d-timeline`+`ph2d-host-desktop` **320/320** (inclui dhat `apply_from_doc_is_zero_alloc_steady_state`) ✓ · clippy `--all-targets -D warnings` ✓ · LOC 568/600 ✓.

**§13.3 — Smoke do Enio (pendente, o DoD):** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-anim && cargo run -p ph2d-host-desktop` → animar X de um sprite (2 keys, 0→4s) → **+Track → Time** → **K em t=0**, scrub p/ t=2, **K de novo** → a posição deve **continuar tocando** (antes congelava); então arrastar as âncoras de Time no graph pra ver slow-mo/freeze/reverse reais.

---

## 14. Feature — deletar track pelo botão direito (`10cb7771`)

**O quê (pedido do Enio, 2026-07-11):** R-click na **LABEL** de uma row de track (coluna de nomes, depois do twirl) abre um menu de contexto de track novo com **"Delete Track"** → remove o binding + a track + os keys em **1 passo de undo** (`TimelineIntent::Unbind`, que já existia no motor **sem nenhuma cobertura** — ganhou o teste `unbind_removes_the_binding_and_its_track_in_one_undo_step`).

**Arquitetura (espelha o menu de presets + o menu da hierarquia):** o hit da label é `TimelineHitKind::Row { target }`; o Secondary Down (em `pointer_down_menus.rs`) abre `ContextMenuKind::TimelineTrack { target }`, cuja tabela `TIMELINE_TRACK_MENU` vive em `ids/menus.rs` (uma tabela, três consumidores: overlay pinta, `pre_populate` registra, o `event.rs` do painel resolve). O painel consome o request (`context_menu().or_else(last_context_menu())` — o gotcha do menu-fechado-no-Down), resolve `target → (entity, prop)` pelo `current_snapshot()` e empurra o `Unbind` pelo canal `drain_intents()` existente — **zero mudança no shell**. Primary na label é **deliberadamente inerte** (arm comentado no `interact.rs`); o splitter continua ganhando na fronteira (hit index back-to-front, splitter registra depois).

**Símbolos novos (grep de colisão):** `CTX_MENU_TL_DELETE_TRACK` (`hash("ctx_menu_tl_delete_track")`) · `TIMELINE_TRACK_MENU` · `TimelineHitKind::Row` · `ContextMenuKind::TimelineTrack` · `timeline_row_id` (`dynamic_id("timeline.row")`) · dev-dep novo `ph2d-core` no painel (Playhead nos seam tests). Zero contrato congelado (`TimelineHitKind`/`ContextMenuKind` não são gateados).

**Prova:** dispatch test (R-click na Row → menu de track, **sem** gesture/captura) · seam test com o fluxo de produção (request parkeado → Click → `Unbind` exato + request gasto) · **expiração** (row sumiu do snapshot → zero intent) · **gate anti-item-morto executável** (`every_track_menu_row_is_handled_by_the_panel` — row nova na tabela sem arm no event.rs fica vermelha) · **2 mutações dirigidas** (mapear Row→None no dispatch → teste vermelho; neutralizar o push_intent → seam vermelho). Gate: 1274/1274 nas 4 crates + clippy `--all-targets` + fmt pin + LOC caps.

**Smoke do Enio (pendente):** row de track → botão direito na **label** (nome) → "Delete Track" → a track some (curvas incluídas); **Ctrl+Z do painel** (undo da timeline) traz binding+keys de volta. R-click num **diamond** continua abrindo o menu de presets; arrastar o **splitter** na borda da coluna continua funcionando.

---

## 15. Rodada de bugs do smoke do rove (Enio, 2026-07-11) — 3 relatos, 2 causas-raiz

| # | Relato | Causa | Fix |
|---|---|---|---|
| B5 | Menu de easing não fecha ao clicar fora | As capturas de graph/dope-sheet em `pointer_down.rs` rodavam ANTES do close-on-outside: o menu flutua SOBRE o dope-sheet, o clique "fora" caía na superfície de baixo, capturava e `return`ava — menu aberto + box-select começado embaixo | Com menu aberto, Down não-Secondary numa superfície graph/timeline = **dismissal puro** (fecha e consome; Secondary segue relocando o menu). Teste `a_primary_down_over_the_dope_sheet_closes_an_open_menu_instead_of_dragging` |
| B6 | "Autoplay criando frames" (keyframes surgindo sem tocar em nada) | O apply escreve `world = curve(t_raw)`, mas o diff do autokey comparava `curve(t_snap)`: **pausar no meio do play** deixa o playhead fora da fronteira de frame (dt da sim 1/60 × display 1/24) e a pose intocada lia como "arrastada" → armado (default!) mintava key do nada | Diff no **t_src CRU** (a MESMA transform da escrita — lição do §13/[[feedback_derived_coordinate_seed_must_match_sample]]); o insert segue no tempo snapado. Testes `a_pose_on_its_curve_at_an_off_frame_pause_keys_nothing` (vermelho antes) |
| B7 | Undo "não volta completamente, resíduos" | MESMA causa do B6, ramo desarmado: a pose intocada lia off-curve → entidade **pinada** (`displaced`) → o apply parava de escrevê-la → undo mudava o doc e o sprite não seguia (até o playhead mexer — por isso "nem sempre") | Idem B6 (pin usa o mesmo diff). Teste `an_untouched_pose_at_an_off_frame_pause_is_not_pinned` (vermelho antes) |

**Superfície:** `pointer_down.rs` (bloco dismissal) + `autokey_pass.rs` (`t_diff` cru) + testes; `autokey_pass` tests extraídos p/ **`autokey_pass_tests.rs`** (LOC cap, 621→210). Zero símbolo novo, zero contrato. Gate: 1277/1277 nas 4 crates + clippy `--all-targets` + fmt pin.

**Confirmado como DESIGN (não bug):** rove com interp **Eased** — o rove iguala a velocidade **MÉDIA** (`|Δv|/Δt`) entre vizinhos pinados redistribuindo os TEMPOS; a forma instantânea (sinos) é autorada pelo easing de cada segmento e fica. Velocidade constante plana = interp Linear (smoke OK do Enio).

**Smoke do Enio (pendente):** (a) menu de presets/easing aberto → clique no dope-sheet fora → fecha, sem box-select fantasma; (b) play → pause no meio → **nenhum** key novo aparece (AutoKey ON) e, com AutoKey OFF, editar/undo na timeline move o sprite imediatamente (sem resíduo).

---

## 16. Feature W5 — Performing / Record (gravar durante o play, `26a77af4`)

**O quê (pedido do Enio, 2026-07-11):** um toggle **Record** novo na barra de transporte (ao lado do AutoKey). Com Record armado **e tocando**, arrastar um objeto **grava a pose ao vivo** ao longo do playhead — mocap na mão, 1 key por frame de display, o traço inteiro em **1 passo de undo**.

**O AVISO DO ENIO (autoplay+play gravava keys) honrado por CONSTRUÇÃO.** O guard de captura:
```
let capturing = if playing { performing && drag_now } else { armed };
```
No play, só grava com **gesto de gizmo ATIVO** (`drag_now`). A pose passiva que a animação está dirigindo **nunca** minta key — um Play puro, mesmo com **AutoKey armado**, grava zero. Segunda barreira independente: o diff do autokey compara no **relógio CRU do apply** (o fix da §15/B6) — sem movimento real, `autokey_props` retorna vazio. Duas camadas, mesmo resultado: **nada grava sem o usuário arrastar**.

**Por que é limpo:** o `autokey_pass::run` roda DEPOIS do `timeline_bridge::run`, que pula a entidade em drag no apply (`live_entity`) — então a pose lida da entidade arrastada é a do usuário, e as não-arrastadas ficam on-curve (o apply as escreveu) → o diff só pega a arrastada. O bracket de undo reusa o de gizmo-drag: um record atravessa N frames de play mas commita **1 step** quando o drag solta.

**Costura (7 pontas, DIRETIVA §2):** `TimelineFlags.performing` + `TimelineIntent::SetPerforming` + `TimelineViewSnapshot.performing` (motor, `ph2d-timeline`) · `TIMELINE_RECORD` id + i18n `panel.timeline.record="Record"` (editor-core/i18n) · populate + paint na barra + `is_toggle` no event + re-export (painel) · `intent_for_transport` → `SetPerforming` + o guard `capturing` (shell). **Zero contrato congelado; `DOC_VERSION` intacto** (`TimelineFlags` não é serializado). Método novo `MockPanelHost::set_toggle_on` no testkit (append-only, espelha `set_slider_value`).

**Símbolos novos (grep de colisão):** `TimelineFlags.performing` · `TimelineIntent::SetPerforming` · `TimelineViewSnapshot.performing` · `TIMELINE_RECORD` (`hash("timeline.record")`) · i18n `panel.timeline.record` · `apply_samples` ganhou o param `performing: bool` · `MockPanelHost::set_toggle_on`.

**Prova IRREFUTÁVEL (5 testes + mutação):** `a_plain_play_with_autokey_armed_records_nothing` (o aviso exato do Enio) · `performing_without_a_drag_records_nothing` (é o GESTO que grava, não a pose) · `performing_with_a_drag_records_the_dragged_pose` (a feature) · `performing_is_inert_when_paused` (só modo de play) · `a_performing_session_is_one_undo_step`. **Mutação dirigida** (`capturing = if playing { true }`, capturar sempre no play) → os **2 testes do aviso ficam vermelhos**, o de gravar-com-drag segue verde — prova de que o guard é o que impede o autoplay de gravar. Mais o seam do toggle Record (→ shell) + o intent test. Gate: **1283/1283** nas 5 crates + clippy `--all-targets` + fmt pin + LOC caps (autokey_pass 234/600).

**Smoke do Enio (pendente):** anime um sprite (2 keys) → **arme Record** → **Play** → durante o play **arraste o objeto** → a trajetória é gravada como keys ao longo do tempo; solte → 1 Ctrl+Z desfaz a sessão inteira. **Prova do aviso:** Play **sem arrastar** (com AutoKey OU Record armado) → **nenhum** key novo. Record é modal (off por default).

---

## 17. Feature W5 — simplificação de keyframes do record (Schneider F-curve fit, `b09f7003`)

**O quê (pedido do Enio, 2026-07-11):** o record (§16) gravava **1 key por frame** (denso, ineditável). Agora, no fim de cada sessão de record, o autokey **simplifica** cada track gravada numa curva Bézier **limpíssima de pouquíssimos keys**, precisa a **0.5% do range de valor**. O algoritmo é o **padrão-ouro** — Schneider ("An Algorithm for Automatically Fitting Digitized Curves", Graphics Gems I 1990): least-squares cubic + reparameterização de Newton + split adaptativo no pior ponto.

**Núcleo em `ph2d-anim/curve_fit.rs`** (`fit_fcurve(samples, tol) -> Vec<FitKey>`), f64, foundational, testável isolado. Duas adaptações que o tornam um fit de **F-curve** (não de curva 2D genérica) — validadas por pesquisa contra `FitCurves.c`, Inkscape `bezier-utils.cpp`, Blender `correct_bezpart`/`curve_fit_nd`:
1. **Erro medido em VALOR no tempo correto:** inverte `x(u)=t` via Newton e compara `V(u)` vs `v` — não a distância euclidiana 2D clássica, que cobraria por estar adiantado/atrasado no TEMPO (irrelevante p/ F-curve; o sample ESTÁ no seu tempo). É o que dá "muito preciso em valor".
2. **Eixos normalizados p/ `[0,1]`** → tolerância = fração do range (mesma leitura em pixels ou radianos); **handle-x clampado a `[0,1]`** → o resultado continua função do tempo (handle não corre pra trás). Wu–Barsky fallback com guard `ε·chord` (não `α<0`, que a referência do livro erra); Newton com guard `den≤0` (Inkscape).

A saída é **`Interp::BezierW`** — um par de tangentes ponderadas É exatamente uma cúbica Bézier no plano `(u, valor)`, então a conversão é **exata**.

**`Track::simplify_range(t_min, t_max, tol)`** (ph2d-anim) troca os keys densos do range pelos fitted, **pinando os endpoints** (keys fora do range intactos, o vizinho mantém seu interp), pulando roving. **Integração no shell** (`autokey_pass`): a sessão de record rastreia por `(entity, prop)` o span `[t,v]`; no **release** simplifica no **MESMO undo step** (1 Ctrl+Z desfaz record+cleanup). **Record grava em tempo REAL sub-frame** (não snapado): com frame-snap o valor do key descasava do seu tempo snapado e o fit perseguia o erro (49→26 keys); em tempo real, 49→~7.

**Símbolos novos (grep de colisão):** `ph2d_anim::{fit_fcurve, FitKey}` (módulo `curve_fit`) · `Track::simplify_range` · `apply_samples` param `performing` já existia; `AutokeyState.record: BTreeMap<(u64,PropKind),RecSpan>` + `RecSpan` (shell-privado) · consts `REC_SIMPLIFY_REL`/`REC_SIMPLIFY_FLOOR`. Zero contrato congelado, zero dep nova.

**§17.1 — Pré-filtro passa-baixa (`e2df085d`, smoke do Enio "reduziu um pouco"):** o record em tempo real captura o **tremor de mão/mouse** (alta frequência); o fit a 0.5% preservava cada tremorzinho > tolerância → **over-subdivide** (avisado na pesquisa como "noise amplification"). Solução padrão de mocap: **`smooth_values`** (kernel binomial `[1,2,1]/4` ×N passes, endpoints pinados, tempos intocados) ANTES do fit. `simplify_range` ganhou `smooth_passes`; o record usa **8 passes** (~janela 9 samples ~150ms a 60fps) + tolerância subida p/ **1%**. Calibrado empiricamente: a 60fps uma gravação ruidosa de 120 samples cai de ~215 keys (sem smooth) p/ **~5-6 keys**, com o erro ficando só no nível do ruído removido (não perde sinal). Testes: `low_pass_lets_a_noisy_recording_reduce_far_more` (smooth mais que HALVA + segue o SINAL não o ruído) + `smooth_values_pins_the_endpoints`.

**§17.2 — Fit KEY-POR-EXTREMO (`7ee4c1fa`, smoke do Enio "keys só nos vales e picos, ≤13"):** o Schneider recursivo **over-subdividia massivamente** (senoide de 6 extremos → ~41 keys espalhados pelas rampas — não é o que um animador desenha). Nova estratégia: **detectar os extremos prominentes** (`anchor_indices`, swinging-door por prominência) e ajustar **UMA cúbica least-squares por trecho monótono** entre extremos (`fit_run`/`one_cubic`) — sem subdivisão. Um key em cada curva, nada nas rampas. Trecho genuinamente complexo (não onda) ganha splits **limitados** (profundidade 2 = ≤4 cúbicas), nunca o blow-up. Resultados (60fps, ruidoso): senoide 6-ext **41→9 keys**; onda 5-ext → 7; bump → 3; rampa → 2. **Fidelidade vira APROXIMADA** (~1-3% do range) — o trade que o Enio pediu (poucos keys > precisão sub-percentual). Testes atualizados p/ o contrato aproximado + `a_wave_gets_a_key_at_each_turn` (maioria dos keys interiores num turn + contagem ≤ extremos+4). Schneider recursivo removido; `min_prom`/`SPLIT_SLACK`/`RUN_SPLIT_DEPTH` documentados.

**§17.3 — COLUNAS ALINHADAS (`267ae2a2`, pedido do Enio):** todas as tracks de uma sessão de record keyam nos **mesmos tempos** — é o que a animação feita à mão parece (o animador pega a coluna e retima TODOS os canais do objeto de uma vez). Pipeline **por ENTIDADE** (dois objetos gravados juntos mantêm timing independente): **(1)** cada track propõe os tempos que quer (seus extremos, via `fit_fcurve`); **(2)** os tempos são unidos e os quase-coincidentes fundidos numa coluna (`COLUMN_MERGE_S` ~2 frames — dois canais que viram quase junto compartilham a coluna); **(3)** cada track é **RE-fitada exatamente nessas colunas** (`fit_fcurve_at` / `Track::simplify_range_at`) — 1 cúbica least-squares por trecho entre colunas, **SEM split** (um split cairia fora da grade e quebraria o alinhamento). `curve_fit` refatorado: `Prep` + `build_keys` compartilhados entre `fit_fcurve` (âncoras = extremos) e `fit_fcurve_at` (âncoras = tempos dados); `Track` ganhou `range_samples`/`simplify_range_at`. **Custo:** cada track fica com a UNIÃO dos tempos (mais keys por track que o fit isolado) — é o preço do alinhamento; se ficar denso demais, aumentar `COLUMN_MERGE_S`. Prova: `a_session_aligns_every_track_of_an_object_on_shared_key_times` (grava X 2-ciclos e Y 3-ciclos, extremos NÃO coincidentes → mesma contagem, mesmos tempos).

**Deferido (documentado no topo de `curve_fit.rs`):** corner pre-pass (cusps viram tangentes BROKEN, não suavizadas) · value-overshoot clamp p/ canais limitados (opacity já é clampada no runtime) · rotation unwrap p/ spins multi-volta.

**Prova:** 11 testes de fit (fidelidade dentro do tol · redução dramática · retas/flat colapsam a 2 keys · canto preservado por split · handles nunca correm pra trás · scale-independence pixel↔radiano · endpoints exatos · dedup de tempo) + 3 de `simplify_range` (redução+fidelidade · keys fora do range intactos · no-op <3 keys) + 2 e2e no record (sessão densa simplifica no release em 1 undo step · não dispara em drag pausado). **416/416** no shell + suítes ph2d-anim/timeline + clippy `--all-targets` + fmt pin + LOC caps (tests do autokey divididos p/ `autokey_test_helpers.rs` + `autokey_performing_tests.rs`).

**Smoke do Enio (pendente, RE-testar pós-§17.1):** arme Record → Play → arraste o objeto por alguns segundos → solte → a track deve ter **poucos keys Bézier limpos** (~5-15, não 1 por frame), e a curva no graph deve seguir de perto o gesto (o tremor de mão fica suavizado — é desejado). 1 Ctrl+Z desfaz o record inteiro. Se AINDA ficar denso, o problema é a delimitação da sessão (drag_now oscilando no play) — instrumentar `ak.record` no release; se ficar liso DEMAIS (achatou um movimento rápido genuíno), baixar `REC_SMOOTH_PASSES`.

---

## Cauda da W4 ainda aberta (para a próxima rodada — decisão do Enio)

- **W4.T4** — docar a timeline no `motion_timeline_slot` quando o split do Motion está ativo (coordenação leve com Motion).
- **W4.T7** — unificar o relógio: `MotionTransport` derivar do `Playhead` + remover transporte duplicado em `motion_bridge.rs` (coordenação leve com Motion).
- **W4.T6 (= B5)** — save de projeto unificado cena+timeline + id estável de entity (**deferido** — cross-cutting, esforço coordenado, não landar solo).
- **W5 restante** (Performing §16 + record-simplify §17 **landaram**): NLA / multi-clip UI (dado já é `Vec<NamedClip>`, só a UI falta) · markers→signals · MCP/Luau · bake curves→keyframes · export · (refinamentos do fit: corner broken-tangents · overshoot clamp p/ opacity · rotation unwrap · low-pass — §17).
