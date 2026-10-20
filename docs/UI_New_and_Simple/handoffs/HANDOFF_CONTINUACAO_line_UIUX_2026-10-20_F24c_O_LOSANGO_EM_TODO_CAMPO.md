# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 20/10 — F24c: o losango em todo campo que tem canal

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (não andou; sem rebase). Anterior:
  [F24b](HANDOFF_CONTINUACAO_line_UIUX_2026-10-20_F24b_O_GRUPO_E_O_LOSANGO.md) (smoke aprovado pelo dono 20/10; §6 itens 2 e 3 = esta onda).
- Plano: [`35_plano_F24_o_auto_key_grava_tudo.md`](../vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md) §5.

## 1 — Commits desta onda

| commit | o quê |
|---|---|
| `716c5e8de7` | o losango em todo campo com canal (pilha, degradê, Contour, Offset, texto, pontas, escolhas, barra de controle, Inspector) + o gate `architecture_every_editable_field_is_a_channel` |
| `17203c8055` | cena `PH2D_AUTOKEY_SMOKE=5\|6\|7` + fotos (`F24c_cena_autokey_5/6/7.png` + `.log`); a paleta só pinta o losango com canal |
| `ca23f2d6c2` | achados da suíte: cenas `autokey_cena*` → `autokey_smoke*` (HR-15), sonda `PH2D_AUTOKEY_LOG` em NOT_LANGUAGE, um só emissor de `AddPathKey` |
| `7f15664a7e` | clippy: `const` asserts em `ph2d-ecs` `canal_tests`, iterador em `ph2d-panel-timeline` `linhas.rs` (herdados de F24a/b) |
| `3c551ef15e` | testes das 2 mutações sobreviventes |
| (este) | este handoff |

## 2 — Como foi construído

### 2.1 Porta única do GRUPO (`crates/ph2d-timeline/src/channel_group.rs`, NOVO, re-exportado em `lib.rs`)

`Instante{t_cut,solo}` · `Forca{Livre<Animado<Chave<Mudado}` · `estado_do_grupo(doc, entidades, reps, inst, valor)` (`:53`) · `clique_no_grupo(doc, reps, entidades, inst, lidos, valor, autorar)` (`:86`): com chave no quadro apaga-as; senão grava cada membro pela porta da K. Grupos de família alargam por `FieldKey::mesmo_grupo`; os outros são a lista dada. `marcas::Instante` agora é `ph2d_timeline::Instante` (re-exportado).

### 2.2 Vocabulário (APPEND, `crates/ph2d-ecs/src/canal_vetor.rs`)

`CONTORNO_PASSOS 200 · _DISTANCIA 201 · _ACEL 202 · _JUNCAO 203 · _LADO 204 · _COR 205..208` (VecContour); `OFFSET_DISTANCIA 210 · _JUNCAO 211 · _LADO 212` (VecOffset). Info/membro/rótulos i18n em `ph2d-i18n/src/canais.rs`; gate `o_vocabulario_do_vetor_nao_renumera` estendido; teste `canais_campos_tests::todo_campo_do_vocabulario_tem_rotulo_traduzido`.

### 2.3 Efeitos como canais (`crates/ph2d-app-vec/src/canais_efeitos.rs`, NOVO)

`Efeitos{contorno,offset}` lê/escreve os componentes da ENTIDADE (passos `1..=MAX_CONTOUR_STEPS`, códigos `0..=2`, cor `0..=255`). Componente ausente fica ausente: criar/remover é estrutura (plano 35 §8). `canais.rs` encaminha por `e_de_efeito` (`:15`) em publicar/aplicar/`ConducaoVetor`. Teste `canais_tests::os_efeitos_vivos_publicam_se_e_a_curva_os_conduz`.

### 2.4 Tabela id → GRUPO (`crates/ph2d-app-vec/src/marcas.rs`, `CAMPOS` `:46`)

Cobre agora degradê (GRAD_ANGLE/INFLUENCE/JITTER → tinta do preenchimento), Cap/Join/Align, fill rule, tipo do traço, W (`VECTOR_WIDTH_NUM`) e Opacity (`VECTOR_OBJ_OPACITY_NUM`) da barra, pontas (`MARKER_ROUND`, `MARKER_SCALE_IDS` → `[ESCALA_INICIO, ESCALA_FIM]`), texto (tamanho, peso, entrelinha, tracking, alinhamento, largura da quebra), Contour e Offset. `DA_CAMADA_ABERTA` (`:173`; `VECTOR_PAINT_WIDTH/DX/DY/DILATE/JOIN_*/OPACITY(+_NUM)`) → campo com `i = aberta+1`. A swatch da entrada k (`vector_paint_swatch_id(k)`) → `[TINTA_PREENCH+OFF_SOLIDA, TINTA_TRACO+OFF_SOLIDA]` com `i=k+1`. `Contexto{aberta, camadas}` (`:188`; `Contexto::do_painel()` `:196` lê `ph2d_panel_vector::state`); `marcas`/`clique` ganharam o parâmetro `ctx`.

### 2.5 Inspector (`crates/ph2d-app-components/src/marcas_inspector.rs`, NOVO)

Position X/Y → `[TranslationX, TranslationY, Position]`, Rotation, Scale X/Y → `[ScaleX, ScaleY]`, Opacity do sprite (+chip), `JointMotorTarget/Speed`, `JointRestLength`, `JointMaxLength`. O clique grava a posição no modo que a K escolheria (`TimelineDoc::position_key_mode(e, true)`: objecto novo → trajectória/Position; eixos separados → X/Y). `ph2d-panel-inspector` e `ph2d-timeline` passaram de dev-deps a deps em `ph2d-app-components/Cargo.toml` (família → painel); `Cargo.lock` não mudou.

### 2.6 Widgets

- editor-core `property_row/escolha.rs`: `paint_choice_row` (`:126`) pinta o losango ao lado E em paleta (`decorador` `:112`; na paleta só com canal, na coluna do decorator do cabeçalho).
- `widget/tool_rail/geometry.rs` `number_with_key_mark(id, slot) -> (chip, Option<losango>)` (`:241`), re-exportado em `widget`; `tool_rail/paint.rs` pinta o chip encolhido + o losango; `screens/hero/tool_bar.rs` regista `key_mark_id(id)` (`key_mark.rs:54`) para `ToolRailEntry::Number`.
- panel-inspector `sections/rows.rs`: `num_row_unit` e `seg_row` com `paint_decorator_for`.
- panel-vector `paint_stack_rows.rs` `layer_row`: o losango da TINTA da entrada fica na coluna do decorator (`form_row_columns`); lixo/↓/↑ e a swatch começam ANTES dela (layout visível mudou).

### 2.7 Shell

`fase_autokey.rs` compõe vetor + Inspector (marcas e clique; o do Inspector só se o do vetor vier vazio). `render_loop/timeline_bridge_keys.rs`: `chave_do_losango` (`:344`, = `chave_da_k`; Position → `ancora_da_k`) e `ancora_da_k` (`:370`) = ÚNICO emissor de `AddPathKey` (a K em `fase_timeline_key_insert.rs` chama-o; agulha do controlo positivo em `fase_timeline_key_insert_tests.rs` = `timeline_bridge::ancora_da_k(`). Shell +47/−7 (sobravam 401 no F24b).

### 2.8 Gate `crates/ph2d-editor-core/tests/it/architecture_every_editable_field_is_a_channel.rs` (4 testes)

- População: cada chamada de PORTA com coluna de losango (`paint_fields_row`, `paint_field_row(_value)`, `paint_text_row`, `paint_color_row`, `paint_slider_with_chip(_layout_adaptive)`, `paint_choice_row`, `paint_decorator_for`, `ToolRailEntry::number`) ou de um helper de LINHA (`LINHAS`), no ESCOPO: `ph2d-panel-vector` inteiro; `ph2d-panel-inspector` só `transform`/`color_tint`/`emissive_row`/`joint_cards`/`joint_kind_rows`.
- Classes: **canal** (id numa TABELA: `marcas.rs`, `marcas_inspector.rs`) · **POR_ID** (exacto ou prefixo `_`) · **POR_FUNCAO** (id em variável; Canal exige o nome no ficheiro E numa tabela) · **DIVIDA_FICHEIRO** · **DIVIDA_CRATE** (as outras `ph2d-panel-*`).
- Censos de obsolescência em todas as tabelas; controlo do leitor `a_varredura_acha_os_campos_que_se_conhecem`.
- Teste de seam novo: `a_linha_de_escolha_tem_losango_nos_dois_feitios` (editor-core `tests/it`). Seams que cobrem a onda: `seam_key_mark` (largura, escolha Cap, entrada da pilha), `o_losango_do_chip_da_barra`, `o_losango_do_transform` (Inspector).

## 3 — Superfície de colisão (um merge pode partir)

- `marcas::marcas`/`clique` ganharam `ctx: Contexto`; `Instante` passou a ser `ph2d_timeline::Instante`; `CAMPOS` mudou de tipo.
- `paint_choice_row` regista um alvo extra (id derivado) quando há canal; `ToolRailEntry::Number` encolhe com canal; `layer_row` mudou o x dos verbos.
- `ph2d-app-components/Cargo.toml` (deps normais). Sem mudança de `Cargo.lock`.
- Ficheiros RENOMEADOS `ph2d-app-vec` `autokey_cena*.rs` → `autokey_smoke*.rs`; `fase_object_mode.rs:36` chama `ph2d_app_vec::autokey_smoke::step`. `fase_timeline_key_insert.rs` já não contém `AddPathKey`.
- NOT_LANGUAGE da shell ganhou 2 linhas (`autokey_pass.rs`; a sonda `PH2D_AUTOKEY_LOG`, com a 2.ª frase numa linha só).
- Vocabulário `canal_vetor` só APPEND (200–212); `PropKind::Field` e `DOC_VERSION 21` seguem do F24a §3.
- Contadores como DELTA: shell +47/−7 linhas; gate novo +4 testes; cena de smoke +3 (`PH2D_AUTOKEY_SMOKE` passa a `1..7`).

## 4 — Smoke do dono (binário `smoke` já compilado: 2.ª build `Finished smoke profile in 0.75s`, zero `Compiling`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=5 ./target/smoke/ph2d-host-desktop` — estrela com degradê laranja→roxo, contorno amarelo extra (2.ª entrada da pilha, já aberta no painel Vector), anéis brancos à volta (Contour), Auto Key ligado, relógio no quadro 12.
2. No painel Vector → Appearance, mude o «Opacity» por baixo de Width/X/Y/Offset (é o da entrada amarela) para 40.
3. Tem de acontecer: no Timeline aparece «Path 0 · Layer Opacity» com losango no quadro 12 e o ponto ao lado do Opacity vira losango cheio. Leve o relógio ao quadro 24 e ponha 100; no quadro 18 vale 70 e o losango fica vazio. Na secção Contour mude Steps (ou Offset): nasce «Path 0 · Contour Steps» e o losango. Em Fill Type mude o Angle: losango ao lado. Na aba «Inspector» (ao lado de «Vector») mude Rotation: losango ao lado e linha «Path 0 · Rotation». Clique no losango cheio de um campo: a chave daquele quadro some. Clique no ponto de um campo parado (ex. Opacity da barra de cima): nasce a chave.
4. Errado = campo sem ponto nem losango na coluna da direita (salvo a dívida de §6.2), losango que não muda ao clicar, Opacity da entrada amarela a gravar na estrela inteira, rotação gravada como trajectória quando o objecto tem X/Y separados.

`=6` faz sozinho e para no quadro 18 (`../vetor_nova_ui/fotos/F24c_cena_autokey_6.png`: losango vazio em Opacity da entrada (70), Fill, Fill Opacity, Angle; dope-sheet Layer Opacity 0.70 · Contour 6.00 · Fill · Rotation). `=7` = `=6` com o Inspector à frente (`F24c_cena_autokey_7.png`: Rotation 8,594° com losango vazio). `=5` = início (`F24c_cena_autokey_5.png`: ponto em todo campo, dope-sheet vazio).

Sonda (`F24c_cena_autokey_6.log`): q18 entrada 1 = 0,700 (meio de 0,4 e 1,0) · anéis 6 (degrau, Hold) · fim do degradê (0,00; 0,90) · rotação 0,1500 (meio de 0,3 e 0); trilhas Rotation 1×2 · `contorno_passos[0]` 1×2 · `pilha_opacidade[1]` 1×2 · `preenchimento[0]` 17×1.

## 5 — Verificação

- `cargo check --workspace --all-targets` verde.
- `nextest-impacted` (`PH2D_SUITE="motivo: deps de Cargo.toml + editor-core"`) 20338/20343: 3 reais curadas em `ca23f2d6c2` (HR-15 cenas app-vec; HR-15 sonda shell; 2 emissores de `AddPathKey`) e re-corridas por nome verdes; 2 flakes de carga (load 45): `motion_path_perf::the_cost_of_sampling_a_path_is_flat_in_its_anchors` (3/3 sozinho) e `wetpaint …a_wet_move_costs_what_the_footprint_costs…` (2/3 sozinho; outra linha).
- clippy `-D warnings` limpo em `ph2d-timeline`, `-ecs`, `-i18n`, `-app-vec`, `-app-components`, `-editor-core`, `-panel-vector`, `-panel-inspector`, `-panel-timeline`, `-host-desktop`.
- Mutação (10): 8 sangraram à 1.ª; 2 sobreviveram (escolha no ramo «ao lado» sem observador; publicação dos efeitos sem teste) → testes novos em `3c551ef15e`, ambas re-provadas a sangrar.
- **NÃO corrido** como lote de fecho da linha: clippy `--all-targets` do workspace inteiro (só as 10 crates acima) e a mutação da lista do F24b §6.10.

## 6 — ABERTO, por ordem

1. **Smoke do dono** (§4).
2. **A DÍVIDA nomeada no gate** (zera no bloco 2). Vetor: filtros · padrão ao longo do caminho · texto no caminho · padrão de textura (canal `OFF_PADRAO` existe; falta a inversa dos ids `TexPatKnob`) · pele do widget · auto layout/âncoras/moldura (Interface) · Blend/Morph/Envelope vivos · Brush stroke · parâmetros dos efeitos (`EFEITO_PARAM` existe; falta inversa de `vector_fx_param_id`) · campos da forma paramétrica na fila ② (`PARAM_VALOR` existe; falta inversa de `vector_shape_field_id`) · perfil de largura (Power Stroke) · modo da quebra do texto. Inspector: skew, emissão, tint/cantos, motor (eixo/modo/força), quebra/limites/mola da junta, e todos os ficheiros fora do ESCOPO (componentes, física, navegação…). Crates: áudio, Flip, Painter layers, Física.
3. **Linhas SEM coluna de losango** (fora do gate, que só vê linhas com a coluna): checkboxes, dropdowns (mistura da entrada `VECTOR_PAINT_BLEND`, mistura do objecto), o seletor Solid/Linear/… do Fill Type (pinta `paint_segmented_group_adaptive` direto), o olho da entrada da pilha.
4. **Seguem do F24b:** menu do botão direito/valor na linha de grupo do dope-sheet · perf de `linhas::layout` e `marcas()` (agora por grupo único por quadro, mais o Inspector) — medir no T0(b) antes de otimizar · lacunas de vocabulário (conteúdo/família do texto, fonte do padrão, Brush) · Bloco 2 (plano 35 §10; antes `git worktree list` por colisão com a missão dos pincéis) · ADR «o canal de qualquer propriedade» · gate de fecho da linha (clippy `--all-targets`, `nextest-impacted`, mutação, **mais** `paint_decorator_for` sem registrar, `set_key_marks` sem limpar, `clique` com `Escopo::Tudo`) · `CLAUDE.md` §5.1 UI/UX ao fechar (trocar o link do handoff; intervalo `PH2D_AUTOKEY_SMOKE=1..7`; não alterado nesta onda) · undo da timeline separado do undo do vetor (conhecido).
