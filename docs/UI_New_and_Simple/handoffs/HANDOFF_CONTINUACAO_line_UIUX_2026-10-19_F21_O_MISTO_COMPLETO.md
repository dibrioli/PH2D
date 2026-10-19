# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 19/10 — F21 (o «misto» completo: seletor, forma vazia, pilha de camadas) feita

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, rebaseada no início da sessão sobre `main` `100d34c8e0` (18 commits, limpo). Anterior:
  [F20](HANDOFF_CONTINUACAO_line_UIUX_2026-10-18_F20_MISTO_E_ESPESSURA.md).
- Fecha os **ABERTO 3** (a pilha lia `sel.first()` e não sabia «misto»), **4** (alfa de Fill: forma vazia não ganhava cor), **5** (escolher a cor do primário no seletor era no-op) e **7** (`vector_mode.rs` 697/700) do F20. O **ABERTO 6** (`bindings::token_world`) fica de propósito: devolve `None` com `ppm ≤ 0`, lei outra que a do `DocPx`.
- Plano: [`32_plano_F21_o_misto_completo.md`](../vetor_nova_ui/32_plano_F21_o_misto_completo.md) (§3 = desenho, §8 = desvios).

## 1 — Commits

| commit | o quê |
|---|---|
| `5d574d16c6` | docs: plano 32 |
| `e9d2a1ddaf` | feat: ver §2. Cenas 33 e 34 |
| `486f80e4be` | fix: o campo `picks` no seletor do teste dos tokens da shell (`tokens_bridge_tests.rs`); teste-controlo do cursor; plano §8 |

## 2 — Superfície de colisão e decisões que um merge pode partir

**O sinal de toque (editor-core)**
- `InteractiveState::BlenderPicker` ganhou o campo `picks: u64`: construtor literal tem de pôr `picks: 0` (7 sítios atualizados, incl. `shells/desktop/src/render_loop/tokens_bridge_tests.rs` e testes de `app-motion` e `app-vec`).
- `WidgetStore::colour_picks(id)` / `note_colour_pick(id)` em `interaction/state/blender_ops.rs`. Incrementado em `dispatch/blender.rs`: `apply_blender_hit` (Wheel, ValueSlider, ChannelSlider com `idx < 3` pela `const ALPHA = 3`, PaletteSwatch com clique esquerdo, HarmonySwatch — helper `picked`), `apply_blender_channel_value` (`idx < 3`) e `dispatch/mod.rs` `commit_hex_buffer`. Alfa e `set_blender_value` (re-semeia) NÃO contam. Recusado: contar no fim do `dispatch` por `ValueChanged` (a alfa também o emite e armaria o RGB).

**Ferramenta e ponte (vetor)**
- `tool-vector` `tool.rs`: `pick_fill_rgba` / `pick_stroke_rgba` armam `FILL_RGB`/`STROKE_RGB` ao toque (+ `colour_authored`); `visible_pick` (alfa 0 → 255). `set_*_rgba` não mudam (conta-gotas, fábrica, swap).
- `app-vec` `vector_bridge.rs` passo 2: `fields::colour_picked(&store)` (cursor `thread_local` `PICKS_SEEN`, avançado a TODO quadro antes do teste do alvo do seletor), escolhe `pick_*` ou `set_*` e devolve a cor visível ao seletor no mesmo quadro (`set_blender_value`) quando o valor da ferramenta difere do escolhido. `vector_bridge_fields.rs`: `pick_merge(cur, new, campos)` (RGB armado sobre alfa 0 toma a alfa nova); `fill_merge` delega-lhe; `pub use fields::pick_merge` em `vector_bridge`.

**A pilha de camadas**
- `paint_stack.rs` + novo irmão `paint_stack_mixed.rs`: `ordered(pen)` (primário à frente — a porta única de «de quem é a pilha»; usada por `vector_bridge_publish.rs` e por `shells/desktop/src/render_loop/fase_transform_ops.rs`), `layers_differ`, `row_mixed`, `toggle` (todas ficam o oposto do primário; o flip por forma saiu), `set_offset_axes(scene, sel, i, dx, dy)` (só o eixo comitado), `apply_picker(store, scene, sel)` (saiu da shell; cursor `thread_local` próprio; RGB ao toque ou se difere do primário, alfa se difere; `pick_merge` por forma).
- Defeito achado e curado: com o seletor aberto numa amostra de camada, a shell escrevia a cor dele na camada `i` de TODAS as formas A CADA QUADRO. A shell encolheu (~-30 linhas).
- `panel-vector`: `PaintRow.mixed: RowMixed { color, width, opacity, blend, offset, dilate, dilate_join }` e `Appearance.layers_differ: bool` (construtores literais quebram; 3 testes atualizados); `state::current_appearance` passou a `pub`. `paint_stack_rows.rs`: `layers_differ` ⇒ uma linha «Mixed layers» (chave i18n nova `panel.vector.appearance.mixed_layers`) + botões de adicionar, sem linhas por índice; amostra mista por `paint_swatch_or_mixed`, etiqueta de opacidade em branco, mistura «Mixed», `dilate_join` sem segmento aceso. `paint.rs` `seed_number_fields`: width/dx/dy/dilate mistos → `blank_number_input` depois de `set_number_value`.
- O olho NÃO tem estado misto pintado (não há ícone); o clique iguala.

**Cenas e roteador**
- Novo `crates/ph2d-app-vec/src/vector_mode_cor.rs` (cenas 33/34, estágios **235..=239**: FIRST 235; formas 235–236; ESTILO 237 também clica Select; PICK 238; LAST 239 só na 34 — abre o seletor em `VECTOR_FILL_SWATCH`, põe o vermelho exato e chama `note_colour_pick`).
- Novo `vector_mode_route.rs`: o roteador das famílias de estágios 21..=34 (`FAIXAS`, `owns`, `step(…, h)`) saiu de `vector_mode.rs` (697 → **655** LOC). `vector_mode_scenes.rs` agora 9..=34, `handles` 19|20 → `cor::FIRST`. O teste `as_faixas_de_estagios_nao_se_cruzam` lê `route::FAIXAS` (o misto 225..=229 faltava antes). Estágios livres: 248..=249, 254..=255.

**O que um merge pode partir**
- Qualquer construtor literal de `BlenderPicker` (campo `picks`), de `PaintRow` / `Appearance` (campos novos).
- Quem leia as camadas por `sel.first()` em vez de `paint_stack::ordered`.
- Quem adicione estágio de cena fora de `route::FAIXAS`.
- `PROJECT_SCHEMA` intocado. Contratos congelados: `git diff 3ee22f984b..HEAD --stat -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-editor-core/tests/it/architecture_tool_contract_surface.rs crates/ph2d-vector-doc/src/primitives.rs` → **saída vazia**.

## 3 — Smoke (por fazer pelo dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=33 ./target/smoke/ph2d-host-desktop`
   — quadrado vermelho (contorno externo amarelo) e quadrado vazio (contorno azul) selecionados. Barra de cima: FILL «—». Painel: Fill «—», Fill Opacity em branco, Stroke «—», e em Appearance a linha «Mixed layers» com os botões + Fill / + Stroke. Errado = uma cor ou as linhas de camada aparecerem.
2. Idem `=34` — os dois quadrados do mesmo vermelho, o seletor aberto mostrando #DC2828FF, FILL vermelho. Errado = o quadrado da direita continuar vazio.
3. Manual, na 33: clique na amostra Fill e escolha uma cor — os dois ganham a cor e o vazio passa a aparecer; arraste só a Alfa — as cores ficam. Selecione uma forma vazia sozinha, abra Fill e mexa no matiz — a forma ganha a cor, opaca.
4. Manual: duas formas com as mesmas camadas mas cores de camada diferentes — a amostra da camada mostra «—»; clique no olho — as duas ficam iguais.

Fotos (`docs/UI_New_and_Simple/vetor_nova_ui/fotos/`): `F21_cena33.png`, `F21_cena34.png` (zoom 98 %, 1930×1012). Os `.log` irmãos ficam não rastreados. Tempo de quadro: **não medido**.

## 4 — Verificação

- **Vermelho primeiro OBSERVADO** em 4 testes antes da cura (`uma_cor_escolhida_preenche_quem_nao_tinha`, `escolher_a_cor_do_primario_escreve_em_todas`, `uma_forma_vazia_ganha_cor_ao_mexer_no_matiz`, `a_pilha_le_o_primario`); controlos verdes antes e depois.
- Testes novos: `crates/ph2d-app-vec/src/vector_bridge_misto_cor_tests.rs` (11, incl. `mod pilha`: `o_olho…`, `um_eixo…`, `o_seletor_numa_camada_mista…`); `crates/ph2d-editor-core/src/interaction/dispatch/tests/picks.rs`; `panel-vector` `seam_paint_stack.rs` `mixed_layers_paint_no_row_and_keep_the_add_buttons`; teste de `vector_mode_cor`.
- Prova de mutação **13/13 sangrou**: M1 pick sem armar · M2 contador no-op · M3 `pick_merge` sem a regra da alfa · M4 `visible_pick` identidade · M5 cursor dentro do `if` · M6 `ordered` · M7 `layers_differ` · M8 `row_mixed` · M9 `toggle` por forma · M10 deslocamento do primeiro · M11 `apply_picker` com `campos = FILL` · M12 pintor ignora `differ` · M13 alfa conta. Nota: M3 só fez sangrar 1 teste, porque `visible_pick` já eleva a alfa no caminho de uma forma.
- `nextest-impacted` (`PH2D_SUITE="motivo: editor-core"`) em `486f80e4be`: 19 716 corridos, 19 715 verdes, 1 vermelho = `ph2d-tool-painter` `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (gate de relógio, crate fora do diff, `loadavg` 79/157/107; sozinho 3/3 verde) ⇒ flake de carga.
- clippy `fecho` `-D warnings --all-targets` em 7 crates: verde. `fmt` limpo. Censos `the_shell_only_shrinks`, tecto de LOC, `panel_wiring_parity`, `tool_contract_surface`: verdes.

## 5 — ABERTO

1. Smoke do dono (§3).
2. **Fechados aqui:** F20 ABERTO 3, 4, 5 e 7.
3. F20 ABERTO 6 (`bindings::token_world` divide por `ppm` à mão): deixado de propósito, devolve `None` com `ppm ≤ 0`.
4. O olho não tem estado «misto» pintado (sem ícone); o clique iguala.
5. Herdados do F19 §5: 2, 5, 6, 7, 8.
6. `CLAUDE.md` §5.1 UI/UX: **não** alterado; ao fechar a linha, trocar o link do último handoff e o intervalo do smoke (`…26` → `…34`).
7. O flake de `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` é candidato a membro da família: conferir [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md) (não promovido aqui).

> ✅ **Smoke do dono: OK (19/10)** — cenas 33 e 34 e os passos manuais (cor nas duas e na vazia, só a alfa, matiz numa forma vazia, olho e amostra das camadas).
