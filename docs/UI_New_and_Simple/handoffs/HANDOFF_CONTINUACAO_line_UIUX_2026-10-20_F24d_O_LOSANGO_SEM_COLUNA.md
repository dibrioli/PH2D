# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 20/10 — F24d: o losango nas linhas SEM coluna de decorator

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (não andou; rebase no-op). Anterior:
  [F24c](HANDOFF_CONTINUACAO_line_UIUX_2026-10-20_F24c_O_LOSANGO_EM_TODO_CAMPO.md) (smoke aprovado pelo dono 20/10; §6 item 3 + a parte do item 2 cujo canal JÁ existia = esta onda).
- Plano: [`35_plano_F24_o_auto_key_grava_tudo.md`](../vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md) §5.

## 1 — Commits desta onda

| commit | o quê |
|---|---|
| `ba192a17eb` | o losango nos campos sem coluna e nos ids derivados (32 ficheiros, +1144/−229) + a lei da COLUNA no gate + cena `PH2D_AUTOKEY_SMOKE=8\|9\|10` |
| (este) | este handoff |

## 2 — Como foi construído

### 2.1 Tabela id → GRUPO (`crates/ph2d-app-vec/src/marcas.rs`)

- `CAMPOS` += `VECTOR_OBJ_BLEND`→MISTURA · `VECTOR_FILL_KIND_{SOLID,LINEAR,RADIAL,MULTI,PATTERN}`→PREENCHIMENTO (`OFF_TIPO`, membro do grupo da tinta) · `VECTOR_STROKE_PRESENT` e `VECTOR_BAR_KIND`→COR_DO_TRACO (`OFF_EXISTE`/`OFF_TIPO`, i=0) · `VECTOR_MARKER_START_DD/END_DD`→TRACO_MARCA_INICIO/FIM. `DA_CAMADA_ABERTA` += `VECTOR_PAINT_BLEND`→PILHA_MISTURA. Por entrada k: `vector_paint_eye_id(k)`→PILHA_LIGADA `i=k+1` (ao lado da swatch).
- `derivados()` = `LazyLock<Vec<(NodeId, PropKind)>>` calculado UMA vez (os ids são `format!`+hash): padrão de textura `texpat_id(slot,k)` para os 21 botões do const `PADRAO` (Tile 0..3, Mode 0..2, Width/Height/Gap/GapY/Angle/ShiftX/ShiftY + `*Num`) → slot 0 = PREENCHIMENTO, slot 1 = COR_DO_TRACO (o bloco inteiro é membro do grupo da tinta). Fora: Source/PickShape (verbos), Offset/OffsetNum (desfasamento: sem campo no vocabulário), Lock/GapLink (gestos de sessão). Efeitos: por linha r<`MAX_FX_ROWS`(4) `vector_fx_hide_id(r)`→EFEITO_LIGADO(r); por p<`MAX_FX_ROW_PARAMS`(6) `vector_fx_param_id/param_num_id/toggle_id(r,p)`→EFEITO_PARAM(i=r,j=p) (linha do painel r == índice do efeito, confirmado em `fx_bridge_dispatch`). Forma: j<`MAX_SHAPE_FIELDS` `vector_shape_field_id(j)` e `vector_shape_choice_id(j)`→PARAM_VALOR(0,j). `ids(ctx)` encadeia derivados + olhos.
- EFEITO_PARAM é grupo por efeito (`por_i`): chave em QUALQUER param acende o losango de todos os params daquele efeito.

### 2.2 `crates/ph2d-panel-vector`

- `paint_rows.rs`: `BodyCtx::decorator(dot, fields)` (`paint_decorator_for` + registo do alvo) e `dropdown_row(id, a11y, face, chip) -> Option<Rect>` (a linha de menu: o chip encolhe para fora da coluna, o losango nela; devolve o chip quando aberto, para o popover diferido). `checkbox_row` agora reserva a coluna (`form_row_columns`) + decorator.
- Os 7 sítios de chip de menu passam por `dropdown_row`: mistura do objecto (`paint_appearance.rs`), da entrada (`paint_stack_rows.rs`), pontas (`paint_markers.rs` `marker_row`), token (`paint_tokens.rs` `token_row`), fonte (`paint_text_sections.rs`; o botão `>` fica na largura do controlo, fora da coluna), mistura do filtro (`paint_filters.rs`), ícone do widget (`paint_widget.rs`).
- Fill Type (`paint_fill_type.rs`): segmentado na largura do controlo + decorator na 1.ª linha do reflow com `[VECTOR_FILL_KIND_SOLID]`.
- Olho da entrada (`layer_row`): losango próprio numa coluna `DECORATOR_W` colada à direita do olho (a coluna da direita segue a da tinta); o rótulo desloca-se `DECORATOR_W`.
- Efeitos (`paint_effects.rs`): `effect_header` reserva a coluna → losango do olho, ícones deslocam-se à esquerda; param booleano («Smooth: Off») pintado na largura do controlo + losango (save/restore de `inner_w`, idioma do cartão).
- `bars_control.rs`: closure `list` → fn `list_chip(store, …)` (o gate lê os ids dela).

### 2.3 `crates/ph2d-editor-core`

`widget/tool_rail/paint.rs` `chip_with_key_mark(scene, theme, id, slot) -> Rect` = a ÚNICA porta dos chips Number e Compound do rail (Compound só ganha losango quando o campo tem marca; compostos sem marca inalterados). `screens/hero/tool_bar.rs` regista o alvo para `Number | Compound`. Teste `tool_rail::tests::o_chip_de_um_campo_com_canal_deixa_a_ponta_ao_losango`.

### 2.4 Gate `tests/it/architecture_every_editable_field_is_a_channel.rs` (agora 5 testes)

- Lei nova da COLUNA: `SEM_COLUNA` = `paint_checkbox(` · `paint_dropdown_chip(` · `paint_segmented_group_adaptive(` · `IconId::Eye` · `eye_icon(` (olho só conta em fn que pinta ícone) só pode aparecer em fn que escreva um marcador `COLUNA` (`paint_decorator_for(` · `.decorator()`); ficheiros `DIVIDA_FICHEIRO` isentos (o olho do filtro em `paint_filters.rs` cai quando a dívida for paga). Teste `todo_campo_sem_coluna_propria_ganha_a_do_losango` com controlo do leitor (5 sítios conhecidos).
- `PORTAS` += `compound`; `DO_RAIL` = number/compound só depois de `ToolRailEntry::`. `LINHAS` += `checkbox_row`, `dropdown_row`, `decorator`, `chip`, `list_chip`. `nome_em` lê `Tipo::Variante` qualificado em ids e tabelas.
- Dívidas pagas: `paint_texture_pattern.rs` (dívida de ficheiro removida); POR_FUNCAO `shape_field_chip`, `fx_param` → Canal. Canal POR_FUNCAO novos: `layer_row` (+`vector_paint_eye_id`), `effect_header`, `marker_row`, `texture_pattern_section`, `size_and_gap_rows`. NaoAnimavel novos: POR_ID `VECTOR_CUT_`, `VECTOR_BAR_STROKE`, `VECTOR_TRANSFORM_RESIZE_BOX` (o que a alça do gizmo faz: comportamento de edição), `TexPatKnob::Lock`, `TexPatKnob::GapLink`; POR_FUNCAO `token_row`, `gesture_chips`, `symmetry_chips`. Divida novos: `TexPatKnob::Offset`, `VECTOR_BAR_PROFILE` (Power Stroke), `VECTOR_TEXT_FONT_DD` (família da fonte). Lista de controlo de `a_varredura` estendida (`VECTOR_OBJ_BLEND`, `VECTOR_PAINT_BLEND`, `VECTOR_STROKE_PRESENT`, `VECTOR_FILL_KIND_SOLID`, `VECTOR_BAR_KIND`, `TexPatKnob::Angle`, `TexPatKnob::Lock`).

### 2.5 Cena de smoke `crates/ph2d-app-vec/src/autokey_smoke_campos.rs` (NOVO, `mod` em `lib.rs`)

Despachante `autokey_smoke.rs`: DESENHO 6/7/8 = `=8|9|10`; `autokey_smoke_tudo::trilhas` passou a `pub(crate)`. Estrela paramétrica (params mantidos), traço azul, entrada 1 da pilha = traço amarelo, efeito Zig Zag, Auto Key ligado, quadro 12. `=9`: q12 olho da entrada desligado, mistura do objecto Multiply, Zig Zag p0 = meio da faixa (50), estrela com 8 pontas; q24 tudo de volta (p0 = 10 % da faixa, 5 pontas); para em q18. `=10` = `=9` + painel do vetor rolado (`set_panel_scroll VECTOR_PANEL, 900`) até ao cartão do efeito. (A 1.ª corrida tinha Zig Zag p0 por omissão 0 → dobrar não fazia nada; corrigido para fracções da faixa.)

## 3 — Superfície de colisão (um merge pode partir)

- `paint_rows.rs` ganha `decorator`/`dropdown_row` (nomes também em `LINHAS` do gate); `checkbox_row` e os chips de menu de largura inteira ficam `DECORATOR_W` mais estreitos no redesenho; `layer_row`: x do rótulo +`DECORATOR_W`; ícones do cabeçalho do efeito deslocados à esquerda `DECORATOR_W`; botão `>` da fonte movido; chip Compound do rail encolhe só quando marcado; `tool_bar` casa `Number | Compound`; `ph2d-app-vec/lib.rs` com `mod` novo; dispatcher `autokey_smoke.rs`; `PH2D_AUTOKEY_SMOKE` passa a `1..10` (+3).
- Sem mudança de `Cargo.toml`/`Cargo.lock`, sem mudança de vocabulário (`canal_vetor` intocado), shell delta 0 linhas (intocada), nenhum contrato congelado tocado. Gate: +1 teste (5).

## 4 — Smoke do dono (binário `smoke` já compilado: 2.ª build `Finished smoke profile in 1.04s`, zero `Compiling`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=8 ./target/smoke/ph2d-host-desktop` — estrela azul de 5 pontas com contorno amarelo extra (a entrada da pilha), efeito Zig Zag ligado, Auto Key ligado, relógio no quadro 12.
2. No painel Vector → Appearance: clique no olho à esquerda da linha amarela «Stroke»; abra o menu «Normal» sob Opacity e escolha Multiply; na barra de cima mude «Points» para 8; role até Effects → Zig Zag, mude «Size» e clique «Smooth: Off»; em Fill Type clique «Linear».
3. Tem de acontecer: cada campo mudado ganha um losango cheio na coluna da direita (o do olho é o colado ao olho; os campos da estrela na barra acendem todos juntos — a forma é um só grupo) e aparece uma linha no Timeline (Layer On, Blend, Shape Settings, Effect, Fill). Vá ao quadro 24 e desfaça as mudanças; no quadro 18 os losangos estão vazios, a estrela mostra pontas intermédias (6,5), o tamanho do Zig Zag fica a meio; olho e mistura seguram o valor do quadro 12. Clicar num losango cheio apaga a chave daquele quadro; clicar no ponto de um campo parado cria uma (Resize Box é um ponto que nunca vira losango — não animável).
4. Errado = checkbox/menu/Fill Type/olho/campo de efeito sem ponto nem losango; o losango do olho acender quando só a cor mudou (ou vice-versa); chip da barra de cima pintado por baixo do losango.

Fotos (`../vetor_nova_ui/fotos/`): `F24d_cena_autokey_8.png` (início: pontos em todo o lado) · `F24d_cena_autokey_9.png` (q18: Points 6,50 com losango vazio e todos os campos da estrela na barra com losango; menu Multiply com losango; olho da entrada fechado com o seu losango; Resize Box/Stroke On/Fill Type com ponto; dope-sheet Layer On 0,00 · Blend 1,00 · Shape Settings) · `F24d_cena_autokey_10.png` (cartão do efeito: Size 30,00 e Ridges com losango, Smooth: Off / Rough: Off com losango, olho do cartão com ponto; menus Start/End das pontas com ponto).

Sonda (`F24d_cena_autokey_9.log`): q18 «olho da entrada 1 false · mistura Multiply · Zig Zag p0 30.000 · pontas 6.50 · trilhas: efeito[0] 4×2ch · forma_param[0] 8×2ch · mistura[0] 1×2ch · pilha_ligada[1] 1×2ch».

## 5 — Verificação

- `cargo check` / `clippy -D warnings` limpos em `ph2d-app-vec`, `ph2d-panel-vector`, `ph2d-editor-core` (`--all-targets`); `cargo fmt --all`.
- `ph2d-panel-vector` crate INTEIRA (`PH2D_SUITE` motivo: gate de registo de painel): 49 + 282 passaram, 3 ignorados (loadavg 168). editor-core por nome: `tool_rail` 9→10, `tool_bar` 11, `key_mark` 3, `architecture_every_editable_field_is_a_channel` 5, `escolha` 6, `literal` 11, `hardcoded` 1, `the_shell_only_shrinks` 1, `workspace_file_loc_cap` 2, `no_magic_numeric` 3. app-vec: `marcas` 9, `canais` 17.
- Testes novos: `marcas_tests::{os_campos_sem_coluna_mostram_o_canal_deles, os_ids_derivados_voltam_ao_canal_deles, a_chave_de_um_efeito_acende_os_parametros_dele}`; `seam_key_mark_sem_coluna.rs` (5: `a_caixa_de_marcar_tem_losango`, `o_menu_de_mistura_tem_losango`, `o_fill_type_tem_losango`, `o_olho_da_entrada_tem_losango`, `o_efeito_tem_losango_no_olho_e_na_caixinha`); `o_losango_do_chip_da_barra::o_chip_de_opcao_da_barra_tem_losango`; `tool_rail::tests::o_chip_de_um_campo_com_canal_deixa_a_ponta_ao_losango`.
- Mutação (12), cada teste visto VERDE antes: 11 sangraram à 1.ª; M7 (ramo Compound a pintar o chip na slot inteira) SOBREVIVEU (nenhum teste observava a largura pintada) → curada pela porta única `chip_with_key_mark` + teste unitário, re-provada a sangrar (`tests.rs:183`). Árvore restaurada (`cmp` igual).
- NÃO corrido: `nextest-impacted` (cadência: < 10 commits desde a verde de F24c; a mudança do editor-core é só o ramo de pintura do rail, compostos sem marca inalterados) e clippy `--workspace` (fecho da linha).
- `agent-loop-profile`: paralelismo 1,19/passo (alvo ≥ 1,5) ✗ · respostas/sessão mediana 142 ✓ · test:check 340:155 = 2,2× ✗ · edições sem conferir 0,3 % ✓ · contexto relido 309 mil ✗ (alvo ≤ 250 mil) · contexto inicial 64 mil ✓.

## 6 — ABERTO, por ordem

1. **Smoke do dono** (§4).
2. **Dívida restante nomeada no gate** (zera no bloco 2): filtros do vetor (ficheiro inteiro, olho incluído) · padrão ao longo do caminho · texto no caminho · pele do widget · auto layout/âncoras/moldura (Interface) · Blend/Morph/Envelope vivos · Brush stroke · perfil de largura (Power Stroke, incl. o chip PROFILE da barra) · modo da quebra do texto · família da fonte (menu) · desfasamento do padrão de textura (`Offset`: precisa de campo no vocabulário). Inspector: skew, emissão, tint/cantos, motor (eixo/modo/força), quebra/limites/mola da junta, ficheiros fora do ESCOPO. Crates: áudio, Flip, Painter layers, Física.
3. **Perf:** `marcas()` agora percorre `CAMPOS` + derivados (2×21 texpat + 4×19 fx + 2×`MAX_SHAPE_FIELDS`) + 2 por entrada, com `canal_do_campo` linear a cada quadro — medir no T0(b) antes de otimizar (um `BTreeMap` id→grupo é a cura óbvia se aparecer).
4. **Seguem de F24b/F24c, inalterados:** menu do botão direito/valor na linha de grupo do dope-sheet · perf de `linhas::layout` · lacunas de vocabulário (conteúdo/família do texto, fonte do padrão, Brush) · Bloco 2 (plano 35 §10; antes `git worktree list`) · ADR «o canal de qualquer propriedade» · gate de fecho da linha (clippy `--workspace --all-targets`, `nextest-impacted`, mutação do F24b §6.10, `paint_decorator_for` sem registrar, `set_key_marks` sem limpar, `clique` com `Escopo::Tudo`) · `CLAUDE.md` §5.1 UI/UX ao fechar (link do handoff; `PH2D_AUTOKEY_SMOKE=1..10`; não alterado nesta onda) · undo da timeline separado do undo do vetor (conhecido).

## 7 — Adenda 21/10: dois bugs do dono, curados nesta janela

1. **«Com a Timeline aberta os gizmos de ajuste das shapes não aceitam ajustes.»** Mecanismo: o Auto
   Key só arma com o painel da timeline aberto (`autokey_pass.rs` `armed = panel_open && auto_key`);
   as mãos que o apply poupa e que o Auto Key trata como arrasto eram só `hero.gizmo.drag` e a pose do
   osso. O arrasto de uma alça de forma (e os outros 10 gestos vivos do vetor) era «edição discreta»
   por quadro: a chave nascia e o apply do quadro seguinte reescrevia `VecShape` com a curva — o
   dedo perdia. Cura (família inteira, uma porta): `ph2d_ecs::MaosDasFamilias` (recurso NOVO, append
   em `canal.rs`, re-exportado; por família) · `ph2d_app_vec::maos` (NOVO: `gesto_vivo` = shape_handles.grab,
   grad_drag, envelope_drag, patternpath_handle, width_grab, textpath_handle_drag, conn_handle,
   connect, pen.is_dragging, shape/pencil activos; `maos` = alvo explícito + selecção; `publicar`)
   · shell: `fase_timeline_view.rs` publica antes do apply, `timeline_bridge::maos_do_quadro` lê as
   famílias, `autokey_pass` `drag_now` conta-as (um passo de undo, como o gizmo). Shell +9 linhas.
   Testes: `maos_tests` (3), `timeline_bridge_maos_tests::a_forma_que_a_mao_do_vetor_segura_nao_recebe_a_curva`
   (com controlo; mutação «sem o extend das famílias» ⇒ RED, visto). ⚠️ Não fotografável (o XTest não
   chega à Xwayland da tela virtual) — o smoke do dono é o juiz.
2. **«L é o atalho da Timeline, mas no modo vetor tem conflito.»** A cadeia do vetor corre antes do
   global e `L` era a Elipse. Decisão do dono (21/10): **`L` = Timeline em todo modo; a Elipse passa a
   `Shift+L`** (`ph2d-panel-vector/src/tool_keys.rs`). Gate `tool_keys_tests::o_l_da_timeline_nao_tem_dono_no_vetor`.
   ⚠️ Aberto: não há gate geral «tecla de modo × tecla global» (as letras do vetor sobrepõem-se a
   várias globais por decisão de 07/10 — `HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md`
   :38-40); só o `L` ficou reservado.

Smoke: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=8 ./target/smoke/ph2d-host-desktop`
→ arraste uma alça da estrela (o ponto do raio interno): ela segue o rato com a Timeline aberta; ao
soltar nasce UMA chave «Shape Settings». Com a ferramenta de vetor na mão, `L` abre/fecha a Timeline;
`Shift+L` pega a Elipse. Binário: 2.ª build sem «Compiling».

**Smoke do dono (§4 + §7): APROVADO em 21/10 («smoke ok»).** A próxima onda começa pelo §6 item 2.
