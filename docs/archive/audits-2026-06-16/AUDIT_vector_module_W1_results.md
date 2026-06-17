# AUDIT — Vector Module W1 (resultados da auditoria multi-lens)

**Data:** 2026-05-28
**Auditor:** sessão Coord-A + Implementador (a pedido do Enio)
**Mandato:** [`HANDOFF_vector_module_audit.md`](HANDOFF_vector_module_audit.md) §4 — auditoria adversarial completa, read-only, fix proposto (não aplicado) até ratificação.
**Método:** 6 lentes paralelas (A arch/ADR/HR · D correctness · G security · B+C UX/file-mgmt · E code-quality · F+H testing/i18n/a11y), cada uma escopada SÓ aos arquivos Vector, claims verificados por grep/read (não memória). Top-4 críticos re-verificados pelo coordenador.

**Disciplina de escopo:** achados em `painter_*`, chrome cluster a11y, e i18n `.ftl` são **adjacentes / project-wide** — handoff ao owner, NÃO fix nesta sessão (`feedback-audit-scope-discipline`).

---

## §0 — Veredito honesto (resposta à pergunta do §4 do handoff)

**NÃO precisa de um R4-style redesign do zero.** A auditoria foi *mais favorável* que o handoff temia em duas camadas e *confirmou* os problemas na terceira:

| Camada | Estado real | Veredito |
|---|---|---|
| **Data model** (`ph2d-vector-doc`, `-traits`, `-brush-traits`, `vector_network.rs`) | Sólido. HR-5 limpo (BTreeMap, zero transcendental), HR-14 versionado (outer+inner), `bounded_decode` minucioso, arch-gate **strict** nos enums congelados, **66 tests** em vector-doc (handoff dizia 35), cobertura de edge real. | **OK pós-cleanup.** Sem refactor. |
| **Correctness core** (`tool.rs` `on_canvas_click`, close-path, chirality) | Verificado robusto: guard NaN/Inf é first-statement e completo, degenerate close double-guarded, `world_to_screen_affine` algebricamente idêntico a `Camera2d`, zoom/resize OK. | **OK.** A preocupação de redesign do handoff é sobre UX, não sobre a matemática. |
| **Shell bridge + persistência** (`vector_pen_bridge.rs`, `vector_pen_input.rs`) + **doc-vs-reality** | **NÃO** OK. É onde os 10 R-rounds deixaram dívida. Concentrada e nomeada, não difusa. | **Cleanup focado (~1 dia) + 1 decisão de design**, não redesign. |

**A raiz dos 10 R-rounds:** falta um *conceito de ownership de cena* — committed paths viram um `Vec` in-memory no shell que ninguém limpa, nada relê do disco, e a persistência é um efeito-colateral por-frame no render loop. Os R-rounds foram patch-de-sintoma de um modelo de dados ausente. **Decidir quem é dono dos vector paths committed (documento/AssetDb W2) é o pré-requisito antes de qualquer fan-out W2 construir sobre este bridge.**

---

## §1 — CRITICAL

### C1 — 26 arquivos `.ph2d-vector` no root, NÃO gitignored
`vector_pen_bridge.rs:255-256` escreve no CWD (root do repo em dev). `git check-ignore` = não ignorado; `.gitignore` sem entry. Cada close-path larga 1 arquivo untracked, sem cleanup. **Podem ser commitados por acidente.** É a queixa original do Enio.
**Fix:** `rm vector_pen_*.ph2d-vector` + `.gitignore` entry; e ver C3/C2 (mover ou remover o save).

### C2 — Assets salvos são dead data write-only — nada relê
`vector_pen_bridge.rs:248` comenta "pode recarregar via `load_and_validate_vector_asset`", mas grep confirma: as únicas referências a `load_*_vector_asset` fora do crate vector-doc são doc-comments/tests **dele mesmo**. Nenhum path de startup/AssetDb carrega. A cena vive só em memória (`committed_vector_pen_paths`); no restart os 26 arquivos ficam órfãos e o canvas abre vazio. **Save custa 1 arquivo poluente por path e entrega zero round-trip.**
**Fix:** remover o auto-save até persistência+load-on-open real (W2), OU rotear via AssetDb. Hoje é pura liability.

### C3 — Data-loss silenciosa: timestamp segundo + write truncante
`vector_pen_bridge.rs:253` usa `as_secs()` (granularidade de segundo); `:256` `std::fs::write` trunca+sobrescreve sem `create_new`. Dois close-paths no mesmo segundo → **nome de arquivo idêntico, primeiro asset destruído silenciosamente** (o toast "Vector saved" dispara para ambos). Verificado.
**Fix:** `as_nanos()` ou counter monotônico + `OpenOptions::create_new(true)` (no-clobber). Resolvido de graça se C2 remover o save.

### C4 — Mentira doc-vs-reality: gate `vello_kurbo_only_in_ph2d_vector` não existe
CLAUDE.md e ADR-0059 §2.8 apresentam como gate congelado ativo ("homestead gate L6F1"). Grep: **nenhum `#[test]` o implementa** — só doc-comments em `vector_network.rs:31` e `lib.rs:32` que dizem *"planned for W2+"*. Nada impede `use vello`/`use kurbo` vazar pra outro crate. **Confirmado pelas lentes A e F + coordenador.** É o achado mais importante: engana todo agente futuro que confiar no CLAUDE.md.
**Fix:** implementar o gate agora (walk `crates/*/src`, allowlist só `ph2d-vector` + os ~20 crates pré-existentes que já importam), OU corrigir CLAUDE.md + ADR-0059 pra marcar como deferred-W2 com task rastreada. **Não deixar o doc afirmar gate inexistente.**

---

## §2 — HIGH

- **H1 — HR-3 violado no dispatch per-frame.** `vector_pen_bridge.rs`: `NetworkLookup::build` reconstrói 2 BTreeMaps por path committed por frame (`:97`); `BezPath::new()` por segmento (`:154`,`:212`); overlay loop faz `vertices.iter().find()` O(N²) (`:147-153`) — ignora o `NetworkLookup` que o committed-renderer já usa um arquivo ao lado. Composto com H2 (Vec cresce sem limite) → alloc por-frame não-limitada. Handoff §2.6 já suspeitava; confirmado.
  **Fix:** cachear lookup/scratch BezPath com dirty-flag; indexar o overlay loop; batch dos guidelines num único BezPath.
- **H2 — `committed_vector_pen_paths` nunca limpo; sem modelo de ownership.** `app_state.rs:415` só recebe `.push` (`:119`), nunca clear/pop/cap. Sem Clear Scene, sem delete, sem Esc. Raiz arquitetural de C2 + crescimento de H1.
  **Fix:** **decisão de design** — documento/AssetDb (W2) OU container de cena bounded com clear/delete. Patch de UX sozinho não resolve.
- **H3 — `bounded_decode` capa POST-decode; doc diz "before decoding".** `:300` `from_bytes` materializa tudo; caps só em `:312+`. Ceiling real = `MAX_ASSET_SIZE=100MB`, que permite amplificação heap multi-GB (cada Vertex/Segment packed é muito menor que em memória; snapshots clonam VectorNetwork inteira).
  **Fix:** corrigir o doc (mentira de segurança), baixar `MAX_ASSET_SIZE` p/ valor realista, OU reader `take`-bounded; + teste com blob adversarial de densidade máxima medindo RSS.
- **H4 — 3 caps de segurança W0-ratificados AUSENTES.** `MAX_VERTICES_PER_LLM_GEN=1000`, `MAX_POLYGON_SIDES=128`, `MAX_SPIRAL_TURNS=64` — zero hits em qualquer crate. Sem LLM-gen path no W1, mas é gap silencioso (viola `feedback-perfection-no-deferrals`). Latente: 1000 < 2048 (cap interativo), então um path LLM-gen poderia exceder o que o tool permite.
  **Fix:** adicionar as `const` em ph2d-vector-doc + gate que asserta presença/valor; aplicar no site de *generation* quando landar.
- **H5 — `world_to_screen_affine` duplica a projeção de `Camera2d` (2 fontes de verdade).** `vector_pen_bridge.rs:233-240` reimplementa por mão o que `Camera2d::world_to_screen` (`camera.rs:151`) já faz. Se a projeção da câmera mudar (DPI, pixel não-quadrado, viewport offset), o bridge diverge dos cliques silenciosamente → vértices renderizam fora de onde foram colocados.
  **Fix (foundational, Coord-A):** adicionar `Camera2d::world_to_screen_affine(&self, window) -> Affine` em ph2d-render + teste de round-trip; bridge chama; deletar a cópia do shell.
- **H6 — Pill PEN não emite nó AccessKit (HR-12).** TopBar clusters não têm `*_a11y_nodes` (só a row de Image action tem). PEN, Open, etc. são invisíveis pra screen reader. **Pré-existente chrome-wide, herdado, não introduzido pelo Vector.**
  **Fix (adjacente — chrome owner):** `topbar_cluster_a11y_nodes` espelhando `image_action_a11y_nodes`. Handoff, não fix nesta sessão.
- **H7 — Esc não cancela path em progresso; deactivate descarta silenciosamente.** Grep: zero handling de Escape em `input_dispatch`. `tool.rs:138` expõe `has_in_progress_path()` explicitamente "pra decidir emitir toast destrutivo" — toast documentado, nunca emitido. Toggle Pen off → `on_deactivate → reset_path` (`:418`) joga fora geometria sem aviso.
  **Fix:** wire Esc → `reset_path()`; emitir o toast destrutivo documentado no cancel/deactivate.
- **H8 — i18n: zero `.ftl` no repo; HR-15 gate é shape-only.** `tool.vector_pen.label` tem shape correto mas resolve pra nada — igual a todo tool. `ph2d-i18n` é stub. **Project-wide, não Vector.**
  **Fix (adjacente — i18n owner):** landar Fluent bundles + estender o gate p/ asserir presença no catálogo.

---

## §3 — MEDIUM

- **M1** — arch-gate não asserta os SmallVec inline caps (`[Vertex;32]`/`[Segment;64]`/`[Region;8]`/`segments;16`); refactor mudaria ABI/postcard silenciosamente. Fix: assert textual dos tamanhos inline.
- **M2** — magnitude de coordenada não-bounded em `on_canvas_click` (`tool.rs:207` rejeita NaN mas não `1e30` finito); coords gigantes são armazenadas/serializadas. Fix: rejeitar `pos.abs().max_element() > 1e7`.
- **M3** — afim depende de assumption pixel-quadrado nunca asserida (foot-gun se câmera virar anisotrópica). Resolvido por H5 (derivar do camera).
- **M4** — literais mágicos: "Pen blue" `(80,130,255)` 3× com alphas 230/200/120, `5.0`/`2.0`/`1.6` sizes (`vector_pen_bridge.rs:141-144,172,211`). Fix: `const` no topo (idealmente tokens).
- **M5** — ruído de comentário R1..R10 (changelog-as-comments) em 5 arquivos; piores: `vector_pen_bridge.rs:59-78`, `:192-199`, `vector_pen_input.rs:42-50`, `vector_pen_toggle.rs:22-28`, `tool.rs:501-509`. Fix: substituir pela *invariante* em 1 linha, dropar a *história*.
- **M6** — toast spam: 1 toast por close-path sem throttle (`vector_pen_bridge.rs:114-118`); ainda vaza o filename poluente ao user.
- **M7** — `Rejected`/`NoOpNearExistingVertex` descartados (`vector_pen_input.rs:79-82`); click silenciosamente no-op sem feedback.
- **M8** — write-path (`save_vector_asset:464`) não enforça `AssetBounds` (assimetria: produtor escreve o que o loader rejeita). Fix: `bounded_encode`/debug-assert.
- **M9** — arch-gate usa `assert_capped` não-strict pra `StampSpec`/`AnimValue` (rename → gate vira no-op). Fix: usar `*_strict`.

---

## §4 — LOW / INFO (registro de honestidade)

- Downcast `as_any_mut().downcast_mut::<VectorPenTool>()` em 2 sites (`bridge:105`, `input:76`): **INFO, não defeito** — todo tool bridge faz isso (painter 6×, bgremoval 7×); é o padrão ADR-0040 §3 corretamente espelhado. Helper Vector-only *divergiria* da house style → seria refactor cross-cutting fora de escopo.
- Tolerância de dedup 12px compartilhada com close-path tolerance: rejeita vértice legítimo a <12px (limitação de design, não bug; raro no MVP straight-line).
- `AssistModeStub` (spiro.rs) sem referente — slot forward-compat de 16 LOC; verificar se W2 usa.
- Cursor crosshair no Pen ativo: provavelmente ausente (fora de escopo verificar).
- Sem atalho de teclado pro Pen (Illustrator = P). Ergonomia, não-bloqueante.
- Cap interativo conta só `vertices.len()`, não segments/regions — benigno (muito abaixo do global 100k).
- HR-4 no-panic: **limpo** (zero unwrap/expect/panic fora de `#[cfg(test)]`).

---

## §5 — Plano de fix por owner (NÃO aplicado — aguarda ratificação do Enio)

### Bloco 0 — cleanup imediato seguro (Coord-A, baixo risco)
- `rm vector_pen_*.ph2d-vector` (26 untracked, scratch deste módulo).
- `.gitignore += vector_pen_*.ph2d-vector`.
- (resolve C1; coordenar com working tree dos outros agentes — escopar `git add -- .gitignore`).

### Bloco 1 — Vector module (modify existing, caminho A/D, sessão Implementador isolada)
Pasta: `crates/ph2d-tool-vector-pen/` + `shells/desktop/src/render_loop/vector_pen_bridge.rs` + `.../input_dispatch/vector_pen_input.rs`.
- Tirar disk-save do bridge per-frame → defer p/ ação explícita OU `target/vector-scratch/` + nanos + `create_new` (C2, C3, M6, + HR-4 I/O síncrona no render loop).
- Indexar overlay loop + scratch BezPath + extrair consts (H1, M4).
- Strip comentários R-history → invariante 1-linha (M5).
- Esc → `reset_path()` + toast destrutivo no deactivate (H7).
- Clamp de magnitude em `on_canvas_click` (M2).
- Surface `Rejected`/`NoOp` via toast (M7).
- Clear-scene affordance + cap em committed_paths (H2 — **depende da decisão §6**).

### Bloco 2 — Foundational (Coord-A only)
- `Camera2d::world_to_screen_affine` + teste; deletar cópia do shell (H5, M3).
- Implementar `vello_kurbo_only_in_ph2d_vector` gate OU corrigir CLAUDE.md/ADR-0059 (C4).
- `const MAX_VERTICES_PER_LLM_GEN/POLYGON_SIDES/SPIRAL_TURNS` + gate (H4).
- Corrigir doc de `bounded_decode` + bound real pré-decode (H3); `bounded_encode` (M8).
- Estender arch-gate: inline SmallVec caps + strict p/ StampSpec/AnimValue (M1, M9).

### Bloco 3 — Adjacente / handoff (NÃO Vector session)
- TopBar cluster a11y nodes (HR-12) → chrome owner (H6).
- Fluent `.ftl` catalogs → i18n owner, project-wide (H8).

---

## §6 — A decisão que precisa do Enio

**Quem é dono dos committed vector paths?** Hoje: `Vec` in-memory no shell, write-only, sem clear. Opções:
1. **Defer persistência inteira p/ W2 AssetDb** — remover auto-save agora, manter cena só in-memory com Clear/Esc. (Mais simples, honesto p/ smoke.)
2. **Scene-document model agora** — container bounded com clear/delete/load-on-open. (Mais trabalho, destranca W2.)
3. **Scratch dir gitignored + load-on-startup** — meio-termo: salva em `target/vector-scratch/`, relê no boot.

Recomendação: **(1)** — remove a liability (C1/C2/C3) imediatamente, e o scene-document real entra estruturado no W2 junto do AssetDb, evitando um terceiro design improvisado.

> **RATIFICADO 2026-06-01 (Enio): opção (1).** Nota: a opção (1) **já foi a
> implementada** no commit `3617672` ("remove auto-save, HR-3 overlay, Esc") —
> auto-save removido, cena só in-memory, Esc-quando-ocioso → `committed_vector_pen_paths.clear()`
> com doc "no persistence until W2 AssetDb". A ratificação confirma o que já está
> em `origin/main`; nada a mudar. Persistência real = W2 AssetDb.

---

## §7 — Sobre T1.4 / T1.6 / T1.8 (pergunta do §4 do handoff)
- **T1.4 (Levien cubic fit)**: ~~stub straight-line~~ **CORRIGIDO 2026-06-01: já IMPLEMENTADO.** `cubic_fit.rs` é um fit Levien moment-matching completo (`fit_cubic_levien`, ~244 LOC, bracket-and-bisect, HR-5-clean, HR-4 NaN/Inf-guarded, Hausdorff scorer two-sided) + 13 testes in-module + `tests/cubic_fit_levien.rs` com fixtures reais (arcos 60°/90°, S-curve `<0.5px`, round-trip exato `<0.25px`, equivariância rotação/translação, controles negativos 180°/closed-loop). A afirmação "stub" acima estava desatualizada. Falta só o subdivisor multi-cubic (split em ≤90° chords) = W2.
- **T1.6 (CRDT)**: `crdt.rs` stub (42 LOC, `CrdtReplay{site_id, peer_clocks}` sem `apply/merge/replay`) — forward-compat correto, `crdt_state = None` no W1. Quando landar, exige custom `Deserialize` depth-bounded + gate. Genuinamente W2.
- **T1.8 (audit formal)**: **executado 2026-06-01** — ver §8.

---

## §8 — T1.8: mini-auditoria de confirmação + FECHAMENTO W1 (2026-06-01)

**Método:** 3 lentes adversariais paralelas read-only (A+G arch/security · D+B correctness/UX · F+E testing/quality) sobre a working tree viva pós-remediação. Cada achado classificado CONFIRMED-FIXED / STILL-OPEN / NEW-REGRESSION com file:line.

**Resultado:** remediação **CONFIRMADA**. Tudo do escopo Bloco 1 + Bloco 2-foundational verificado FIXED em código (não verbal):

| Finding | Verdito T1.8 | Evidência |
|---|---|---|
| C1 (lixo .ph2d-vector) | ✅ FIXED | `.gitignore:57` + zero arquivos no root |
| C2/C3/M6 (auto-save) | ✅ FIXED | zero `fs::write`/`as_secs` nos shell files; cena in-memory |
| C4 (gate vello_kurbo "mentira-doc") | ✅ FIXED (2026-06-01) | CLAUDE.md §6 + ADR-0059 §2.8 + README §7.1/L6F1 todos marcados "W2-deferred — não existe ainda" (`69febf7`) |
| H1 (HR-3 hot-path) | ✅ FIXED | overlay indexado O(N) via BTreeMap lookup; scratch BezPath reusado |
| H2 (clear-scene) | ✅ FIXED | Esc-idle → `committed_vector_pen_paths.clear()` in-memory = **opção (1)** |
| H3 (bounded_decode pré-decode) | ✅ FIXED | `bytes.len() > MAX_ASSET_SIZE` antes de `from_bytes`; doc honesto |
| H4 (3 consts LLM-gen) | ✅ FIXED | consts + gate value-asserting em `architecture_vector_contract_surface` |
| H5/M3 (Camera2d affine) | ✅ FIXED | `world_to_screen_affine` fonte única + round-trip test; cópia shell deletada |
| H7(a) (Esc cancela path) | ✅ FIXED | `try_vector_pen_escape` → `reset_path`, wired em keyboard.rs |
| H7(b) (toast destrutivo no deactivate) | ✅ FIXED (2026-06-01) | `pen_has_in_progress_path` (downcast no bridge allowlisted) + `Toast::warning` no drain `CancelActiveTool` (`69febf7`) |
| M1/M9 (gate strict) | ✅ FIXED | `assert_capped_strict`/`assert_exact_strict` nos enums congelados + inline SmallVec caps |
| M2 (clamp magnitude) | ✅ FIXED | `MAX_COORD_MAGNITUDE=1e7` + NaN/Inf guard, 2 tests |
| M7 (Rejected toast) | ✅ FIXED | `Toast::warning` no Rejected; NoOp-near-vertex silencioso documentado |
| M8 (write-path bounds) | ✅ FIXED (2026-06-01) | `check_asset_bounds` shared + `bounded_encode` (valida→encode→MAX_ASSET_SIZE); `save_vector_asset` = wrapper default-bounds; sem bypass `to_allocvec` em produção; +2 tests (`b3b2f00`) |
| M5 (R-history comments) | ✅ FIXED | shell files limpos; só invariantes 1-linha |

**Residuais aceitos (não-bloqueantes, rastreados p/ W2):**
- **H3 transiente:** `MAX_ASSET_SIZE=100MB` permite pico de heap multi-GB durante `from_bytes` antes dos caps post-decode. Fix-doc tomado (opção OR da auditoria); reader `take`-bounded ou valor menor = W2 hardening. **Honestamente documentado, não escondido.**
- **Persistência real (§6 opção 1):** `save/load_vector_asset` + `bounded_encode/decode` sem caller de produção; cena vive em `committed_vector_pen_paths` (in-memory). AssetDb + load-on-open = W2.
- **T1.6 CRDT + subdivisor multi-cubic + `AssistModeStub`** = W2.

**VEREDITO: Vector Module W1 FECHADA (2026-06-01).** Data model sólido, correctness verificada, bridge/persistência limpos pós-remediação, 4 CRITICAL + todos HIGH/MEDIUM in-scope fechados, gates executáveis verdes. Carry-overs para W2 são genuínos (AssetDb, CRDT, subdivisor, gate vello_kurbo), nenhum silenciosamente quebrado. Commits da remediação: `3617672` (Bloco 1) + `172eff2` (H5/M3) + `b3b2f00` (M8) + `69febf7` (T1.8 close).
