# HANDOFF DE INTEGRAÇÃO — `line/Painter`: a jornada WET PAINT (2026-07-21)

> Para o **agente integrador** (por ordem EXPLÍCITA do Enio — a linha NÃO integra sozinha,
> §0.7). Cobre a jornada Wet Paint inteira (ADR-0134 → doc 21), **49 commits**, todos os
> smokes aprovados pelo Enio. Os chunks ANTERIORES desta mesma linha (lag do Rake · Shape
> Flow/modelo de rotação · Impasto unified tools · gizmo/hover) têm handoffs próprios de
> 2026-07-18/19 — leia-os junto; a branch carrega tudo.

## 0. Estado

| | |
|---|---|
| Branch | `line/Painter`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| Jornada | `f36a533a` (ADR-0134) → HEAD — 49 commits; diff 99 files, +13,6k/−1,4k |
| Árvore | limpa · `cargo check --workspace --all-targets` 0 warnings |
| Gate batched | `nextest-impacted` vs `origin/main` **5015/5015 verdes** (185 skip) · clippy `--all-targets -D warnings` limpo nas 5 crates tocadas · fmt pinado (1.95) limpo na árvore · typos limpo · `workspace_src_files_under_loc_cap` **verde (zero ofensores)** · `no_magic_numeric` verde |
| Smokes | W1 · Circular re-smoke · lote W2.4/2.5 · checkbox+Paper · paper-tone · W3 · **doc 21** — todos OK (Enio, 2026-07-20/21) |
| Auditoria | 2 lentes sobre o diff acumulado — ver §5 |

## 1. O que a jornada entrega (mapa por camada)

- **`ph2d-wet-paint` (crate NOVA)** — porte 1:1 do reference JS (`docs/Painter/ph2d_wet_paint/js/engine/`),
  16 módulos; lei do porte no doc do `lib.rs` (f64 aritmética / f32 storage · `jsmath` · `libm`).
  Aceitação §18 completa + **fingerprint de sessão pinado** (`tests/fingerprint.rs`) + perf gates
  `#[ignore]`. Portas de produto: `dispatch_pressure_dab_lane` (pressão real + raio real + silhueta +
  grain por closure) · `begin/segment/end_direct_stroke` · `render_pigment_only_region` ·
  `dispatch_pressure_dab_erase` · `seed_paper_with`/`rebake_paper` · `set_stroke_color` ·
  `reset_knob_group` · raias `painter/doors.rs` (Symmetry/Tiling).
- **`ph2d-tool-painter`** — `PaintMode::WetPaint` (slot 11, `PAINT_MODE_COUNT` 12); módulos novos
  `wetpaint.rs` (sessão display-state + guards) · `wetpaint_settings.rs` (arm autorado + `WetKnobs` f64 +
  rotas) · `wetpaint_commit.rs` (a porta de commit do doc 21) + `wetpaint/tests.rs` e
  `wetpaint_commit/tests.rs` (gates). Costuras pequenas em `stamp_route.rs` (rota + altura pulada),
  `stamp_preview.rs` (stash + peel door), `stencil.rs` (fronteira de modo), `layers/undo.rs`
  (kill EAGER da água), `stroke_lifecycle.rs`/`lifecycle.rs` (clears do stash).
- **`ph2d-painter-brush`** — `StrokeMethod::is_incremental()` (porta única da pergunta que vivia em 3 cópias).
- **`ph2d-panel-painter-layers`** — seção Wet Paint (`paint_wetpaint.rs`), `stroke_method_offer.rs`
  (função pura do menu Method), esconderijos em `paint_brush_sections.rs` (Watercolor some sob wet;
  Paper aparece com `watercolor || wetpaint`), allowlist `PAINTER_WETPAINT_CLICKS` no `event.rs`,
  fields no `number_field.rs`/`populate.rs`; gates de seam em `tests/seam_wetpaint.rs` + `tests/seam.rs`.
- **`ph2d-editor-core`** — ids novos `ids/chrome/painter_wetpaint.rs` (11 ids + 2 arrays). Hasheados,
  sem colisão (gate `node_id_collisions` cobre).
- **`shells/desktop`** — cena de smoke `wetpaint_smoke.rs` (`PH2D_WETPAINT_SMOKE=1`) + wiring mínimo
  no `render_loop`/`painter_bridge`/`app_state`/`main`.
- **Docs** — ADR-0134 · design multiagêntico [`docs/Painter/21_wet_stroke_integration_design.md`]
  (as 4 leis, 13 seams file:line, tabela por método, G0–G18, §F análise do vazamento de
  `deposit_pass`, §G riscos) · handoffs da jornada · memória
  `feedback_a_nonidempotent_target_excludes_nothing_split_authoring_from_deposit.md` + linha no índice.
- **`.typos.toml`** — 2 palavras pt-BR (`TRANSFERE`, `dilema`).

## 2. As decisões que o integrador NÃO deve re-litigar

- **O solver é serial POR SEMÂNTICA** (brake lê `wet` vivo; drying lê o vizinho esquerdo pós-update)
  — ADR-0109 (bandas) é inaplicável. O fingerprint pina.
- **Sessão = display-state; undo mata a água** (grid fora do `ModelSnapshot` — ~235 MB/passo a 2048²,
  ADR-0117). Encerrar a sessão É o bake.
- **Doc 21 (as 4 leis):** autoria não-incremental é UN-owned (pipeline flat NORMAL) · a cauda do
  `stamp_drag_preview` stasha o batch exato (`pending_deposit`) · UMA porta de commit
  (peel → re-arm → replay único sob `deposit_pass`; escrito em exatamente 2 statements adjacentes —
  vazá-lo é I2 ressuscitado, doc 21 §F) · **wet-on-wet**: writes próprios re-armam o guard, a água
  CONGELA sob autoria (hold DERIVADO), o depósito funde na MESMA sessão. Esc devolve a água;
  eraser × re-stamp = flat erase honesto (água morre, W2.6 estendido).
- **Reverts deliberados do W3:** narrowing do menu Method DELETADO (lista cheia sob wet) · coerção de
  entrada DELETADA (shapes cruzam a fronteira abertos, o método viaja junto) · o belt da rota FICA.
- **2 sobreviventes de mutação POR PROJETO, documentados no código:** o gate de relevo do seam-9
  (`impasto_applies` já é Paint-only — contrato honrado 2×) e o `!eraser` do rearm (nenhum chamador
  atual alcança; 2ª camada).

## 3. Conflitos prováveis no merge (e como resolver)

- **`CLAUDE.md §5`** — a linha ADICIONOU o bloco "🌊 O WET PAINT" dentro da entrada Painter. Lei da
  lista compartilhada: só adição; fundir mantendo os blocos de outras linhas.
- **`ph2d-editor-core/src/ids/chrome/mod.rs`** — +2 linhas (mod + re-export `painter_wetpaint`).
  Append-only; conflito textual com outra linha que adicionou módulo de ids resolve mantendo ambos.
- **`ph2d-panel-painter-layers/src/event.rs`** — allowlist ganhou `PAINTER_WETPAINT_CLICKS`; o arquivo
  está em **600 LOC exatos** (teto do painel). Se outra linha somou linhas ali, o merge estoura o cap —
  a válvula usada foi comprimir comentário (o aviso mora no doc de `PAINTER_WETPAINT_CLICKS`).
- **`paint.rs` do tool está em 700 EXATOS** — qualquer linha paralela que some 1 linha estoura; a
  válvula desta linha foi fundir pares de `pub use` adjacentes.
- **`Cargo.toml` workspace** — membro novo `ph2d-wet-paint`; `ph2d-tool-painter` e o shell ganharam a
  dep. Append-only.
- **`stroke_method.rs`** (`ph2d-painter-brush`) — método novo `is_incremental` + doc de enumeração de
  leitores. Conflito só se outra linha tocou o mesmo arquivo.
- **Nenhum contrato congelado foi tocado** (§6): `Tool`/`CanvasPaintTool`/`PanelEvent` intactos;
  `NodeOp`/vector intactos; `PROJECT_SCHEMA`/`DOC_VERSION` intactos (a sessão wet não é serializada).

## 4. Gates que o integrador deve ver verdes na árvore combinada

`foundational-integrate.sh` (gate da árvore combinada) e, nominalmente:
- `ph2d-wet-paint`: suite inteira (aceitação §18 + `fingerprint` + `product_doors`).
- `ph2d-tool-painter`: 790 verdes — em particular `wetpaint/tests.rs` (G0 fingerprint do modo Paint —
  a lei #1 da jornada, pinada em literais) + `wetpaint_commit/tests.rs` (14 gates do doc 21).
- `ph2d-panel-painter-layers`: `seam_wetpaint.rs` + os 2 seams do wet em `seam.rs`.
- `ph2d-editor-core`: `workspace_src_files_under_loc_cap` · `no_magic_numeric` · `node_id_collisions`.
- Perf (manual, `--release -- --ignored`): `ph2d-wet-paint/tests/perf.rs` (mediana por classe).

## 5. Auditoria de fechamento (2 lentes sobre o diff acumulado)

- **Lente SEAMS (costura de UI): SEM ACHADOS.** Os 11 ids `PAINTER_WETPAINT_*` fecham o circuito
  pintado→populado→forward→consumido (Enable/Reset via allowlist + `route_brush_wetpaint_event`;
  knobs via `PAINTER_WETPAINT_FIELDS` + `is_param_field`); menu Method completo sob wet com gate;
  Watercolor/Paper com presença E ausência gateadas; Reset devolve DEFAULT e desarma; zero widget
  morto. Nota registrada: os ranges dos knobs vivem em **2 cópias deliberadas** (consts do painel ·
  clamps do tool), ambas declarando o SPEC §16 — hoje batem valor a valor; se um lado mover, o outro
  não acompanha sozinho.
- **Lente CLAIMS (afirmações load-bearing vs código, state-grep): SEM ACHADOS.** Os 9 claims
  verificados com evidência file:line: `deposit_pass` escrito só nos 2 statements adjacentes da porta
  (`wetpaint_commit.rs:72/74`, nascimento `Default`) · undo mata a água EAGER antes do
  `restore_shape_overlay` (`layers/undo.rs:114→118`) · stash limpo em TODOS os caminhos de morte
  (paint_begin · reset_transient · end_session · restore_shape_overlay · peel) · `wet_owns_the_dabs`
  porta única com os 2 perguntadores (`stamp_route.rs:104/279`; o `is_incremental` do belt interno é
  muro documentado, não cópia) · hold derivado perguntado DEPOIS do guard; knobs reconciliam nas 2
  portas · nenhum outro modo alcança o engine (gate varre 10 modos + controle positivo) e a altura é
  pulada em wet · os 5 sites de cancelamento usam `peel_drag_preview` · §G fiel (`TRAIL_HALF=61`;
  cauda ≤2 dabs/lane provada em `trail.rs:178-183/381/410`) · braço wet PRIMEIRO no
  `commit_drag_preview` com o caminho não-wet byte-idêntico ao pré-doc-21 (diff conferido).
  **Duas notas menores registradas (não-achados):** o único caminho que dropa os shape editors sem limpar o
  stash é `discard_open_shape` (camada vira não-pintável) e **nenhum consumidor alcança o órfão**
  (todo chamador de `commit_drag_preview` exige editor vivo ou passou por `paint_begin`; o próximo
  `stamp_drag_preview` faz clear+refill) — é a "higiene aceita sem pin" do handoff do doc 21; e o
  `drag_preview = None` em `stamp_preview.rs:156` é redundante (a porta já fez `take()`), inofensivo.
  Selection/Deform não estão na lista do gate de 10 modos porque roteiam ANTES de `stamp_dabs`.

## 6. ⛔ NÃO integrei nem pushei (protocolo §0.7 / §0.2)

A linha fecha aqui: handoff entregue, worktree limpo, aguardando a ordem EXPLÍCITA do Enio e o
agente integrador dedicado. Pós-integração, o que segue vivo está nomeado no doc 21 §G e
na entrada §5 do CLAUDE.md (pool de lanes · `TRAIL_HALF=61` · cauda ≤2 dabs/lane · água congela sob
shape aberto **por design** · emenda de perf do ADR-0134 pendente de veto · gatilho de commit por
secagem completa = decisão de produto aberta).
