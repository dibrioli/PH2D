# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 18/10 — F20 (seleção múltipla «misto» e espessura na unidade do documento) feita

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, rebaseada no início da sessão sobre `main` `bdc7a38639` (13 commits, limpo, zero sobreposição). Anterior:
  [F19](HANDOFF_CONTINUACAO_line_UIUX_2026-10-18_F19_O_PAINEL_LE_A_FORMA.md).
- Fecha os **ABERTO 3** (Width em px de tela) e **4** (seleção múltipla mostra o primário) do F19. Ambos com decisão do dono (18/10, opção recomendada nos dois).
- Plano: [`31_plano_F20_misto_e_espessura_do_documento.md`](../vetor_nova_ui/31_plano_F20_misto_e_espessura_do_documento.md) (§8 = desvios).
- Defeito medido (plano §1): com 2 formas selecionadas, tocar na Width reescrevia a cor de preenchimento e o traço inteiro do primário em todas (`apply_fill_colour(rgba(fill))` + `StrokeStyle::onto` em laço). Width lia 2,952 a 98 % (foto F19 28).

## 1 — Commits

| commit | o quê |
|---|---|
| `e6fbdfacbc` | docs: plano 31 + decisões do dono. Revoga a recusa «mostra o primário» de `crates/ph2d-app-vec/src/appearance.rs` |
| `d2d8c2d7d7` | feat: ver §2 (desenho). Mensagem do commit diz estágios `225..=228`: **errado**, são `225..=229` |
| `2027a500b6` | fix: três censos da suíte (teste do «misto» pintado → `tests/it`, aresta widget→interaction; `skin/number.rs` delega a11y ao pintor canónico; `tool_tests.rs` ≤ 700) |

## 2 — Superfície de colisão e decisões que um merge pode partir

**Portas únicas**
- `ph2d_tool_vector::StyleFields` (`crates/ph2d-tool-vector/src/style_fields.rs`, máscara `u32`: FILL_RGB, FILL_A, STROKE_RGB, STROKE_A, WIDTH, CAP, JOIN, ALIGN, MARKER_START/END, MARKER_SCALE_START/END, MARKER_ROUND, dash(k,gap)×6). Campo `authored` da ferramenta substitui `apply_to_selected`+`width_authored`; cada setter arma o SEU bit; cores armam só na mudança (RGB e alfa em separado); drenado por `take_authored()`.
- Escritor = detetor: `StrokeStyle::onto_fields(old, width, campos)` + `fill_merge` + `restyle_selected_fields` em `crates/ph2d-app-vec/src/vector_bridge_fields.rs`; `onto`/`differs_from`/`restyle_selected_strokes` viraram wrappers. O passo 4 de `vector_bridge::dispatch` escreve só `stroke_campos`/`fill_campos`; `onto_fields(s) != *s` é o detetor de mudança. `note_style_seen` continua DEPOIS do passo 4 (F19).
- Leitura: `mixed_of(pen, scene)` (mesma máscara) → `VectorStyleSnapshot.mixed` (+ `agrees(f)`).
- `ph2d_editor_core::DocPx` (`crates/ph2d-editor-core/src/length.rs`; embrulha `pixels_per_meter`; `to_world`/`of_world`; `DEFAULT`). A Width da ferramenta guarda px do DOCUMENTO; `PenStyle.stroke_w_px` → **`stroke_w` em MUNDO** (8 produtores: `shape.rs`, `pencil.rs`×2, `lib.rs`, `pencil_edit.rs`, `stroke_present.rs`, `ph2d-vec-text` `glyph.rs`, shell `connector_gesture.rs`). Campos Px das formas (raios/cantos) por `shapes_units::to_world/to_ui(…, DocPx)`; alças (`range_world/step_world`, `handle_grip`, `track`, `drag`) recebem `doc`. `VecState.doc` escrito a cada quadro por `shape_params::frame_fields(…, doc)`; `Gate.doc`. Px de TELA ficou onde é tela (raio de clique, fidelidade do lápis, tolerância de imã, realces) — recusado mudar.
- Removidos por não terem mais uso (a shell encolheu): parâmetro de zoom em `stroke_present::toggle`, `resolve_style`, `sync_active_text_style`, `fase_text_panel`, `fase_shape_fields`, `fase_connector_and_shape_params`.

**Pintura do «misto»**
- Fill/Stroke: `paint_color_row(mixed)` no painel; `ToolRailEntry::Swatch.mixed` (+ `.mixed()`) na barra ②; `BarFace::Swatch.mixed` na coluna ①. Cap/Join/Align sem segmento aceso; chips Width/opacidade em branco (`mixed_or`); tracejado e Head Size/Round em branco (`blank_number_input` na semente); dropdown de ponta e mistura do objeto dizem «Mixed» (chave i18n nova `panel.vector.mixed`); opacidade/mistura do objeto por `Appearance.mixed_opacity/mixed_blend` calculados em `paint_stack::published`. O slider fica no primário (lei do BulkSelect do Inspector). `blank_mixed_numbers` esvazia `VECTOR_WIDTH_NUM` e restaura quando a seleção volta a concordar (`thread_local LAST_MIXED`).
- **Defeito preexistente achado e curado** (foto da cena 31: Dash 1 mostrava 0): `blank_number_input` limpava o buffer, mas todo pintor mostra `format_number(value)` sem foco — o BulkSelect em branco do Inspector também nunca aparecia. Agora `WidgetStore.mixed_numbers: BTreeSet<NodeId>` + `number_input_mixed(id)`; marcado por `blank_number_input`; limpo por `set_number_value`, `apply_chip_value_with_mirror` (digitar/stepper/arrasto), setas, register/unregister. Pintores `paint_number_input_with_buffer`, `paint_number_chip(_flat)` ganharam `mixed: bool`. Comportamento: campo misto abre VAZIO ao focar (sair sem digitar não escreve e continua misto); digitar o número do próprio primário agora escreve em todas (antes: «sem mudança», ignorado). `widget/skin.rs` partido em `widget/skin/number.rs` (tecto de 500 LOC do widget). ~30 ficheiros de painéis ganharam o argumento (inspector, grid-snap, equalize-sizes, flip-frames, motion-graph, painter-layers, timeline, tokens).

**O que um merge pode partir**
- Qualquer linha que adicione chamada a pintor de NumberInput/chip precisa passar `mixed: bool`.
- `ToolRailEntry::Swatch` e `BarFace::Swatch` ganharam campo (construtores literais quebram).
- `PenStyle.stroke_w_px` → `stroke_w` (mundo): produtor novo de `PenStyle` NÃO multiplica por `px_to_world`.
- `shapes_units::to_world/to_ui`, `shape_handles::{handle_grip, track, drag}`, `shape_params::{apply_shape_field, ui_values_of, frame_fields}` recebem `DocPx`.
- `VectorTool::take_apply_to_selected` / `take_width_authored` removidos → `take_authored`.
- Gates da shell `the_width_reaches_the_selection_when_it_is_authored` e `the_stroke_checkbox_is_wired` reancorados em `vector_bridge_fields.rs`.
- Testes migrados: os `alheia` do F19 passaram de PX 0.02 → 0.01 (régua do documento); testes de alças via `doc_of(px)` (helper `cfg(test)` em `shape_handles.rs` e no `lib.rs` do app-vec); testes da ferramenta afirmam bits exatos.
- Cenas 31/32: `crates/ph2d-app-vec/src/vector_mode_misto.rs`, estágios **225..=229** (FIRST 225; formas 225–226; ESTILO 227 também clica Select; PICK 228; LAST 229 só na 32); `const assert` contra vizinhos `alinhar`/`controle`. Tabela `vector_mode_scenes.rs` agora 9..=32, `handles` 17/18; braço do roteador em `vector_mode.rs`, que está em **697/700 LOC**.
- `PROJECT_SCHEMA` 191 intocado. Contratos congelados: `git diff bdc7a38639..HEAD --stat -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-editor-core/tests/it/architecture_tool_contract_surface.rs crates/ph2d-vector-doc/src/primitives.rs` → **saída vazia** (nada tocado).

## 3 — Smoke (por fazer pelo dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=31 ./target/smoke/ph2d-host-desktop`
   — quadrado vermelho e azul selecionados. Barra de cima: FILL e LINE mostram um traço «—», W vazio. Painel da direita: Fill/Stroke com «—», Width vazio, Cap/Join/Align sem nada aceso, caixa esquerda de Dash/Gap 1 vazia. Errado = uma cor ou um número ali.
2. Idem `=32` — as duas mais grossas, a vermelha mantém o contorno preto tracejado, a azul mantém o branco; W = 9. Errado = as duas da mesma cor.
3. Manual, na 31: arraste a Width — as duas recebem, as cores ficam; escolha uma cor de Fill — as duas ganham o preenchimento, contornos intactos; clique na caixa Width vazia e saia sem digitar — nada muda.
4. Manual: selecione uma forma e dê zoom com a roda — o número W não muda; desenhe um retângulo com zoom alto e outro com zoom baixo com W 3 — o contorno tem a mesma grossura a 100 %.
5. Manual: Inspector de uma multi-seleção de sprites com opacidades diferentes — a caixa de percentagem fica em branco (antes mostrava a do primário).

Fotos (`docs/UI_New_and_Simple/vetor_nova_ui/fotos/`): `F20_cena29_show_width_inteira.png` (barra W = 6; era 5,904), `F20_cena30_width_9.png` (W = 9), `F20_cena31_misto.png` (FILL/LINE «—», W vazio; painel Fill/Stroke «—», Width vazio, Cap/Join/Align apagados, Dash 1 vazio, Gap 1 = 1 por serem iguais, Head Size 1/1), `F20_cena32_width_das_duas.png` (vermelha continua vermelha com traço preto tracejado, azul/branco; W = 9; Dash 1 ainda vazio). Zoom das cenas 101 % no rodapé. Os `.log` irmãos ficam não rastreados. Tempo de quadro: **não medido**.

## 4 — Verificação

- Testes novos: `crates/ph2d-app-vec/src/vector_bridge_misto_tests.rs` (6: `mexer_na_width_de_duas_formas_so_muda_a_width`, `cada_controlo_escreve_so_o_seu_campo` (11 controlos), `o_painel_diz_misto_onde_as_formas_diferem`, `a_width_e_a_mesma_a_qualquer_zoom`, `desenhar_com_3_da_a_mesma_linha_a_qualquer_zoom`, `com_uma_forma_o_gesto_escreve_como_antes`); `crates/ph2d-editor-core/tests/it/o_misto_pinta_em_branco.rs`; `dispatch/tests/blur.rs` `o_misto_entra_vazio_e_digitar_o_numero_do_primario_escreve`; teste dos bits de `StyleFields`; ida-e-volta de `DocPx`; teste de `vector_mode_misto`.
- **Vermelho primeiro NÃO foi observado** (testes escritos depois da cura). Substituído por prova de mutação **10/10 sangrou**: M1/M2 gravar todos os campos → 2 vermelhos em `misto_tests` · M3 `onto_fields` ignora WIDTH → 2 · M4 `mixed_of` desligado · M5 adoção por zoom · M6 `PenStyle` por zoom · M7 `set_cap` arma JOIN · M8 sem esvaziar · M9 `shapes_units` por zoom → `shape_params` + ida-e-volta vermelhos · M10 `blank_number_input` não marca → `o_misto_pinta` vermelho.
- clippy `-D warnings --all-targets` em 16 crates tocadas: verde. `fmt` limpo.
- `nextest-impacted` (`PH2D_SUITE="motivo: editor-core"`) em `d2d8c2d7d7`: 20 027 corridos, 20 024 verdes, 3 vermelhos = censos (aresta widget→interaction 44 > 43; `tool_tests.rs` 703 > 700; a11y de `skin/number.rs`), curados em `2027a500b6`; esses 3 gates + o teste movido reexecutados: 7/7 verdes. **Suíte inteira NÃO reexecutada depois do fix.** 816 s; o mais lento, `ph2d-tool-painter` tutorial `macro_track` 417 s.

## 5 — ABERTO

1. Smoke do dono (§3).
2. **Fechados aqui:** F19 ABERTO 3 (Width em unidades do documento) e 4 (seleção múltipla «misto»).
3. `paint_stack::published` mostra as camadas de `sel.first()`, não do primário da caneta: a pilha de camadas não sabe «misto» (só opacidade/mistura sabem).
4. Alfa de preenchimento por campo: numa multi-seleção, escolher uma cor RGB não dá preenchimento a forma sem preenchimento (alfa 0 permanece; «sem preenchimento» = alfa 0). O Illustrator preencheria.
5. Seletor sobre Fill misto: escolher exatamente a cor do primário é no-op (a cor só arma na mudança).
6. `bindings::token_world` ainda divide por `ppm` à mão; poderia usar `DocPx` (deixado: devolve `None` com ppm ≤ 0, `DocPx` limita).
7. `vector_mode.rs` em 697/700 LOC — a próxima cena tem de mover o roteamento para fora.
8. Suíte inteira não reexecutada depois de `2027a500b6` (§4).
9. Herdados do F19 §5: 2 (isenção das abas de slot sem gate), 5 (edição durante *Show* sobrescrita pelo tween), 6 (bloqueio do Preview não cobre Hierarquia/Timeline), 7 (opacidade dirigida/cor por token são valores de vista), 8 (herdados do F18).
10. `CLAUDE.md` §5.1 UI/UX: **não** alterado nesta janela; ao fechar a linha, trocar o link do último handoff e o intervalo do smoke (`…26` → `…32`).
