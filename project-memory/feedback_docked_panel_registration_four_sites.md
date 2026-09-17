---
name: feedback_docked_panel_registration_four_sites
description: Registrar painel docado novo exige sítios além da crate; em 2026-07 nenhum tinha gate e o painel ficava morto com tudo verde — hoje três têm gate, um deixou de ser defeito
metadata:
  type: feedback
---

Um `ph2d-panel-*` novo **compilava, linkava e passava nos testes** enquanto estava **morto na tela** — a UI só aparecia quando TODOS estes sítios estavam feitos (diagnosticado 2026-07-08 no `ph2d-panel-timeline`, 3 fixes até aparecer). **Estado verificado contra o código em 2026-09-13** (processo vivo: DIRETRIZ §3.B.1, passos 4–9):

1. **crate + `cargo run -p ph2d-panel-sync` + `EXPECTED_TYPED`** em `ph2d-panel-registry-init` (feature, dep e `reg.push` GERADOS entre marcadores; o `default` e o bloco `#[cfg]` do contador são à mão). Os consts `DEFAULT_VISIBLE`, `ICON` e `DEFAULT_SLOT` do trait são obrigatórios — faltar um é erro de compilação, não silêncio. Os ids do painel moram no `src/ids.rs` da própria crate (A5b, 12/09).
2. **Feature-proxy no SHELL** (`shells/desktop/Cargo.toml`): `panel-<slug> = ["ph2d-panel-registry-init/panel-<slug>"]` + entrada no `default` do shell. Sem ela o `reg.push` é compilado fora. Uma dep DIRETA do painel no shell NÃO liga a feature (resolver-v2). ✅ **Gate:** `every_panel_the_registry_ships_reaches_the_binary` (`shells/desktop/tests/it/`).
3. **`PANEL_Z_ORDER_FALLBACK`** em `crates/ph2d-editor-core/src/screens/hero/paint.rs`, caminhado pelo `screens/hero/panel_walk.rs` junto com os painéis já trazidos à frente. Registado+visível fora dos dois ⇒ **nunca pintado**. ✅ **Gate:** `architecture_every_panel_is_painted` (`ph2d-editor-core/tests/it/`), que imprime a cura.
4. ~~**Visibilidade** em `hero.rs`~~ — ⚠️ **deixou de ser sítio de defeito.** `default_panel_visibility()`/`canonical_panel_id()` vivem hoje em `screens/hero/panel_ids.rs`; sem a entrada o `set_panel_visible` cai num `Box::leak` e FUNCIONA (o comentário lá diz «funciona e mesmo assim é errado»). A visibilidade de omissão de um painel novo é o `DEFAULT_VISIBLE` do trait.
5. **`cursor_over_hero_panel`** em `shells/desktop/src/forwarding.rs` — allowlist de ids. Ausente dela, a roda sobre o painel dá **zoom na CÂMERA** por baixo e o painel aparece e clica normal (não parece bug de registo). Descoberto 2026-07-09 no zoom da timeline. ✅ **Gates** (para painéis que publicam polegar de barra): `every_scrollable_panel_intercepts_the_wheel` + `the_allowlist_has_no_stale_entries` (`shells/desktop/tests/it/`).

**Por que mascarava (e onde a lição sobrevive aos gates):** o `EXPECTED_TYPED` compila com os defaults da própria crate e o `cargo check -p <shell>` passa porque a crate é dep direta — só o run visual revelava. Os gates dos sítios 2, 3 e 5 vivem em **`tests/it/`** da `ph2d-editor-core` e da shell: um `cargo test --bins` ou uma suíte só da crate do painel **não lhes chega** ([[feedback_a_bins_run_never_reaches_the_gates_that_live_in_tests]]). Ao debugar «painel não aparece»: 2 → 3 → geometria; se **aparece mas a roda não rola**, é o 5.

**Escala do wheel:** o shell converte `LineDelta` em **px lógicos (×16 por notch)** antes do `WheelEvent`. Um divisor de sensibilidade calibrado pra "notches" fica ~16× rápido demais — o motion-graph usa `ZOOM_WHEEL_DIV = 240` para essa mesma entrada; copie a escala, não o número mental.

**Widgets:** pinte pela **fonte da verdade = Widget Gallery** (`ph2d-editor-core/src/widget/showcase/*.rs`), não improvise dimensões. Toggle canônico = switch `TypeToken::Xl3` largura × `Density::Compact` altura (pílula 2:1) + label à esquerda; `Toggle::new(id,label).on().state()`. Rect quadrado → vira círculo sem trilho. Ver [[feedback_ui_source_of_truth_gallery_inspector]].
