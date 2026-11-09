# Handoff de continuação — `line/ScriptEditor`, plano 14 Ondas Q1 + Q2 (09/11)

> Plano: [14_plano_quadros.md](../14_plano_quadros.md) · decisão do dono 13 · maquete
> [14_quadros_do_depurador.html](../maquetes/14_quadros_do_depurador.html). Base: Onda B fechada
> ([handoff](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_ONDA_B.md), não reaberta). **Não integrado, sem push.**
> Smoke do dono de Q1+Q2: **pendente** (cena 14).

## 1. O que entrou

Q1 e Q2 num só commit: a tabela sem um quadro que a use não se fotografa nem se prova ao dono.

- **Motor (`ph2d-script-debug`)** — `Var { name, value, kind: ValueKind, fields, field_count }`; `ValueKind` = o tipo
  Luau (re-exportado por `ph2d_script`). A paragem desce `Var::MAX_DEPTH = 3` níveis de tabela, `Var::MAX_FIELDS = 24`
  campos por tabela (nomes por ordem, depois `[i]` por número; o recurso é o tempo do gancho com o jogo parado). Os
  valores ao vivo NÃO descem (`locals_at(l, 0, 0)`). ⛔ **Defeito achado e curado:** descer a uma tabela no fio parado
  por um ERRO empurrava valores dentro dos registos da função (o Luau não desenrola a pilha) e pisava as locais
  seguintes (`hp` lia-se como o `self.id`) — a descida corre agora numa cópia no fio PRINCIPAL (`ffi::on_aux`, `lua_xpush`).
  Gate: `depurador_pause_on_error_para_no_erro_com_as_variaveis` (+ `dt == 0.5`); prova de mutação: com `aux = t` reprova.
- **`DebugStop.queued: usize` → `waiting: Vec<Entity>`** (quem espera, pela ordem; `waiting.len()` é a contagem de antes).
- **Crate nova `ph2d-ui-table` (Folha)** — a tabela da casa: `Column`/`Row`/`Cell`/`Ink`, `TableState` (larguras
  arrastadas, colunas escondidas, dobras por chave, rolagem, arrasto), `layout()` → `Layout` (a mesma geometria pinta e
  recebe: `Layout::hit` → `Resize | Header | Toggle | Cell`), `natural_widths`, `natural_height`, `paint` (listra →
  âmbar/rato → texto; divisórias; cabeçalho; barra de rolagem), `paint_spark`/`spark_points` (para a Q3),
  `changed_tint` (o âmbar a `ROW_SELECTED_ALPHA`: o `WarnSoft` é superfície neutra neste tema — foto). O texto vem pelo
  trait `CellText` do anfitrião (`UiText` na casa; `ph2d_panel_code::table_text::CodeCells` na mono do código, com a cor
  do realce e o âmbar por `Palette::tint`). Testes: `ph2d-ui-table/tests/it` (8).
- **Crate nova `ph2d-code-debugcard` (Folha)** — o modelo saiu da `ph2d-code-assist` (`debug_view.rs` → `model.rs`):
  `DebugCard { state, title, line, function, who: WhoTag, waiting: Vec<WhoTag>, error, locals, this: Vec<VarRow>, stack,
  can_step }`, `VarRow` achatada com caminho estável (`self.pos.x`, `self.trail[1]`), `StepMemory` (o «mudou» = valores
  diferentes do MESMO objecto+script; outro objecto recomeça), `CardStyle` (Sheet por omissão), `sheet_rows` (grupos
  `Locals · N`/`self · N fields`; uma tabela com filha mudada abre-se sozinha), `columns`, `value_ink`, `type_text`.
  Tecto na catraca da família: 663 (medido 563).
- **Pintura `ph2d-panel-code::debug_card`** (`mod.rs` estado/clique, `head.rs` faixa, `sheet.rs`, `boxes.rs` Cards e Strip)
  — `DebugCardView { style, table, painted, revealed }`: `height`, `paint`, `press → CardClick (Ask | Jump | Copy |
  Taken)`, `drag_to`, `wheel`, `header_at`, `column_menu`, `chevron_of` (sonda). Um passo novo com uma linha âmbar fora
  de vista rola até ela (`revealed` = `linha:chave`). Quadro até ½ da aba (`CARD_SHARE`), mínimo 4 linhas.
- **Cola `ph2d-app-code`** — `View ▸ Card Style: Sheet · Cards · Strip` (`CMD_CARD_SHEET/CARDS/STRIP`, a escolhida com ✓,
  estado `CodeDocs::card_style`, como o Peek/Map: em memória); `debug.rs` monta o `WhoTag` com a cor da Bancada
  (`bancada.who.holders` pelos bits) e a `StepMemory`; o clique na pilha vai à linha; clicar num valor copia
  (`copy_out`); o botão direito no cabeçalho abre o menu das colunas (`context_menu::Row::Column`). `debug_card.rs`
  ficou cola (50 linhas): a `ph2d-app-code` estava a 33 linhas do tecto — está em 26 627 / 26 628.
- **Cena `PH2D_CODE_SMOKE=14`** — o MESMO `depura.luau` da 12 (ganhou `self.pos`, `self.trail`, `self.label`, `self.alive`
  e a linha `self.pos.x = …`; a cena 12 não muda de passos), parado em `self.t = self.t + dt` (`QUADROS_BREAK`).
  `STEPS_14`: 1 Sheet · 2 Next line (`self.t` âmbar) · 3 Next line ×2 (`self.pos` abre, `x` âmbar) · 4 Cards · 5 Strip ·
  6 Sheet + menu das colunas. Gestos novos: `ClickTabMenuRow`, `ClickDebugRow`, `RightClickDebugHeader`; `Until::Changed`.

## 2. Provas

| Prova | Resultado |
|---|---|
| `ph2d-ui-table` it (geometria: dobras, flex, puxador com mínimo, esconder Type, ▸ vs célula, rolagem, traço) | 8/8 |
| `ph2d-code-debugcard` it (caminhos estáveis, «mudou», abrir sozinha, `· 3 fields`) | 2/2 |
| `ph2d-script` `depurador_*` (24, com o novo `…_tipo_e_os_campos_das_tabelas_aninhadas`) | 24/24 |
| `ph2d-app-code` `depurador`, `barra_da_aba`, `onda1`, `botao_direito`, `o_nivel_le_se` — inclui o seam `o_card_style_troca_o_stopped_e_o_cabecalho_esconde_a_coluna_type` (cliques reais: ▸, botão direito, «Type») | verdes |
| `ph2d-app-components` `os_scripts_da_cena`, `every_word`, `a_cena` · `ph2d-panel-code` `bancada` | verdes |
| `ph2d-editor-core`: `shell_only_shrinks`, `workspace_file_loc_cap`, `panel_loc_cap`, `panel_wiring_parity`, `no_dependency_climbs_a_layer`, `every_member_inherits`, `hr15`, `no_literal_color`, `no_magic_numeric` | 23/23 |
| Fotos `ferramentas/smoke/cena14/passo_0..6.png` + `maquete_lado_a_lado.png` (A, B, C da maquete × app) | feitas e conferidas |

Não corrido (fecho da linha): clippy das crates tocadas, `PH2D_SUITE=fecho`, auditoria.

## 3. Diferenças conhecidas da maquete (decididas, não esquecidas)

- O âmbar do tema é pálido (`Warn` = 212,199,158 medido na foto); a maquete usou um amarelo mais quente. É o token.
- A coluna Type diz o tipo Luau; a maquete escreveu `object · Circle` para o `self.id` (um número) — o motor não sabe
  que um número é um objecto. Fica para quando o `ph2d.get`/ids tiverem tipo próprio.
- Strip só mostra o nível de cima (sem `self.pos.x`), como a maquete.

## 4. O que falta (plano 14 §5)

- **Q3** — Live values em tabela (Line · Name · Value · Last 4 s) + anel de histórico por linha+nome no `Debugger`
  (amostrado a 2,5 Hz, `Duration::Slow`, como os fantasmas), pairar = mín/máx/actual, clicar = gráfico grande por baixo
  da linha. Peças prontas: `Column::spark()`, `Cell::spark`, `paint_spark`, `spark_points`. Hoje o Live mostra só a faixa
  (estado verde + objecto).
- **Q4** — balão ◆ (`ph2d-panel-code/src/bancada/popover.rs`), Script options (`ph2d-app-code/src/options_card.rs`) e
  Problems (`ph2d-panel-code/src/console/problems.rs`) na mesma família; a grelha da Bancada (`bancada/grid.rs`) migra
  para a `ph2d-ui-table`.
- ⚠️ **Tecto da `ph2d-app-code`: 1 linha de folga.** Q3/Q4 têm de pôr o código novo no `ph2d-panel-code` (sem tecto de
  crate, 600/ficheiro) ou numa Folha, e cortar na cola.

## 5. Rodada do smoke 09/11

O dono (cena 14): o Stopped tomava mais área que o código, não rolava nos Cards, sem puxador, e «o texto do script não rola».

- **Altura** (`ph2d-panel-code::debug_card`): sozinho o quadro vai até `CARD_SHARE = 1/3` da aba (era ½), mínimo
  faixa + 4 linhas. **Puxador** na borda de cima (`painted.grip`, faixa `PANEL_RESIZE_HANDLE_SIZE_PX`, traço `Text3` ao
  meio): carregar · arrastar · largar muda `user_h`, preso entre o mínimo e `USER_SHARE = 2/3`. Ponteiro: `Grab` /
  `Grabbing` (`DebugCardView::look`, não há `ResizeNs` no `PointerLook`).
- **Roda**: `DebugCardView::wheel` toma a roda sobre o quadro (Sheet → `TableState::wheel`; Cards → `cards_scroll`,
  preso a `painted.cards_max`; os títulos ficam, as linhas rolam por baixo e as parciais não se pintam).
- **«O código não rola»**: o teste de costura PROVA que a roda sobre o texto rola com o quadro aberto e fica rolada
  nos quadros seguintes (verde já antes da cura). A causa do que o dono viu: a vista do código **não tinha barra de
  rolagem nenhuma** e o quadro a ½ deixava 9 linhas. Cura: a barra da casa (`paint_scrollbar`, só indica) em
  `CodeView::paint` + o tecto de ⅓.
- **Tecto da `ph2d-app-code`** (26 619 / 26 628): o teste novo pagou-se movendo `ghosts_due` (+ teste) e `wheel_rows`
  para `ph2d-panel-code::debug_marks`, as linhas em texto do quadro para `DebugCard::text_rows`, e tirando
  `debug_buttons_of` (o teste lê `debug_view_of(..).painted.buttons`).
- **Provas**: `ph2d-panel-code` `quadro_do_depurador` (2: tecto ⅓, puxador até ⅔ e ao mínimo, ponteiro; Cards rolam e a
  roda fora do quadro não é dele) e `os_valores_ao_vivo_amostram…`; `ph2d-app-code`
  `parado_o_codigo_rola_e_o_quadro_tem_tecto_roda_e_puxador` (cliques reais: roda no código, na Sheet, puxador, Cards)
  + `depurador barra_da_aba onda1 botao_direito completar` verdes; `ph2d-ui-table` 8/8; `ph2d-code-debugcard` 2/2;
  `ph2d-editor-core` `shell_only_shrinks file_loc_cap panel_loc_cap no_magic_numeric no_literal_color hr15
  no_dependency_climbs` verdes.
- **Fotos** `ferramentas/smoke/cena14/passo_1|3|4.png`: código maior que o quadro, barra do código, puxador, Sheet e
  Cards cortados com rolagem. ⚠️ Os Cards não têm barra de rolagem própria (só a roda) — por decidir no próximo smoke.

## 6. Cards: rolagem por cartão (09/11)

Decisão do dono (smoke 09/11): nos **Cards** cada cartão rola por si.

- `DebugCardView::cards_scroll: [f32; 3]` (Locals · self · Call stack) substitui a rolagem única; `CardPainted::cards`
  (`CardBox { rect, max, bar }`) substitui `cards_max`. `wheel` rola só o cartão debaixo do ponteiro (preso ao `max`
  dele); dentro do quadro mas fora dos cartões a roda é tomada sem efeito. A rolagem fica entre Next line (sem reset).
- `boxes::scroll_card`: cabem `floor((h − título) / linha)` linhas; o `max` conta-se em linhas inteiras e o
  deslocamento pinta-se arredondado à linha — nenhuma linha a meio, o título fica. Se transborda, pinta o indicador
  da casa (`paint_scrollbar`, só indica) na borda direita, e as caixas de valor / a Call stack encolhem para não
  passar por baixo dele. Os rects de copiar e de saltar na pilha só existem para as linhas visíveis.
- `ph2d-app-code`: zero linhas em `src` (só o teste).
- **Provas**: `ph2d-panel-code` `quadro_do_depurador` (2; o novo `os_cards_rolam_cada_um_por_si…`: só o self tem
  indicador, a roda no self não mexe nos Locals, os Locals que cabem não rolam, preso ao fim, nenhuma linha a meio);
  `ph2d-app-code` `parado_o_codigo_rola_e_o_quadro_tem_tecto_roda_e_puxador` (roda real sobre o cartão do self:
  `cards_scroll[1] > 0`, `[0] == 0`) + `depurador` 12 · `barra_da_aba` 6 · `botao_direito` 6 verdes;
  `ph2d-editor-core` `shell_only_shrinks file_loc_cap panel_loc_cap no_magic_numeric no_literal_color hr15` verdes.
  ⚠️ O filtro de testes da Bancada em `ph2d-panel-code` não correu: o guarda dos subagentes recusa a palavra (pesado).
- **Foto** `ferramentas/smoke/cena14/passo_4.png`: Locals 2 · self 8 (5 à vista, com indicador) · Call stack.

## 7. Barras arrastáveis (09/11)

O dono (smoke 09/11): «não consigo arrastar o novo scroll com o mouse». As três barras da aba eram só indicador.

- **Uma lei, a da casa** (`ph2d-editor-core::widget::scroll_area::{OwnBar, BarDrag}`, ficheiro novo `own_bar.rs`):
  para a barra cuja rolagem o DONO guarda (fora do `WidgetStore`). Mesma conta do despachante (`begin_bar_drag`):
  regista a TRILHA (nunca o polegar); carregar no polegar agarra sem rolar; carregar na trilha fora dele salta o
  polegar para debaixo do dedo (`scroll_for_track_press`, a lei GTK/Blender da casa — não é página); arrastar é
  proporcional (`scrollbar_delta_for_drag`), preso a 0 e ao fim, com as medidas fotografadas no Down. Zona de
  toque = a trilha até à borda direita do corpo (12 px), nunca menos que `Spacing::Sm`.
- **Código** (`CodeView::{bar, bar_press, bar_drag_to, bar_release}`, campo `bar_drag`): a mesma `OwnBar` pinta e agarra;
  a arrastar o polegar fica `Accent` (`ScrollbarState::Dragging`).
- **Cards** (`CardBox::track: OwnBar`, `DebugCardView::cards_drag`, `bar_dragging`): só o cartão agarrado rola; o
  puxador de cima não mexe.
- **Sheet** (`ph2d-ui-table`): `Hit::Bar` (antes das células, depois do cabeçalho), `Layout::bar()`,
  `TableState::press(hit, (x, y), ..)` / `drag_to((x, y), ..)` (assinaturas com o `y`), `bar_dragging`. ⚠️ A barra
  pintada à mão (fina, `BorderStrong`) passou a ser o `paint_scrollbar` da casa (10 px, trilha `Bg3`) — a mesma
  geometria pinta e agarra. Durante o arrasto nenhuma coluna muda de largura.
- **Bancada**: listas/grelha da `ph2d-panel-code` não têm barra (só `paint.rs` e `boxes.rs` chamam `paint_scrollbar`).
- **`ph2d-app-code`** só encaminha (`pointer.rs`: 3 linhas reescritas, 0 novas). Tecto: o teste de costura pagou-se
  em `tests/it`: `Mesa::arrasta_com` (carregar · mover · largar; o `clica_com` passa a ser um caso dele) e os laços de
  clique à mão de `ajuda.rs::clica_aba` e `botao_direito.rs::direito` passaram a usá-lo ⇒ catraca 26 628 / 26 628.
- **Provas**: `ph2d-ui-table` 9/9 (novo `a_barra_do_corpo_arrasta_pelo_polegar_e_salta_pela_trilha`);
  `ph2d-panel-code` `quadro_do_depurador` 3 (novo `a_barra_de_um_cartao_arrasta_pelo_polegar_e_salta_pela_trilha`);
  `ph2d-app-code` `parado_o_codigo_rola_e_o_quadro_tem_tecto_roda_e_puxador` estendido com arrastos reais (Sheet ao
  fim sem mexer em colunas; código rola e a seleção fica; puxador; self dos Cards ao fim, Locals a 0) +
  `depurador` 12 · `barra_da_aba` 6 · `botao_direito` 6 · `selec` 5 · `arrastar` 17 · `ajuda` 5 verdes;
  `ph2d-editor-core` `shell_only_shrinks workspace_file_loc_cap panel_loc_cap no_magic_numeric no_literal_color hr15
  panel_wiring_parity` verdes.
- **Fotos** `ferramentas/smoke/cena14/passo_1|4.png` refeitas (a barra da Sheet é agora a da casa). O arrasto não se
  fotografa (a tela virtual ignora eventos sintéticos): prova-se pelo teste de costura.

## 8. Acabamento ao nível da maquete (09/11)

O dono: «a referência ficou mais bela que o nosso app». Três rodadas foto × maquete (Firefox headless a 1×,
`ferramentas/smoke/cena14/maquete_lado_a_lado.png`: A/B/C da maquete à esquerda, Sheet/Cards/Strip à direita).

- **Âmbar** — o `warn` do tema `dark` é o `warning` do Godot `(0,83 0,78 0,62)` = `212,199,158` (caqui; a 0,35 sobre
  o cartão dava `99,100,96`, cinzento). Novo `ph2d_tokens::changed` (`changed_ink`, `changed_band`): **matiz e croma
  do âmbar canónico do `tokens.json` (fábrica `forge`: `warn` 0,795/0,125/80, `warn-soft` 0,310/0,036/80) à
  luminosidade do tema activo** — no `forge` é a identidade; no `dark` dá `≈242,190,98` (maquete `#e6b04c`) e a
  banda opaca `≈ forge warn-soft` (maquete `#3d3422`). `ph2d_ui_table::changed_tint` passou a ser a banda opaca.
  Usado: ■ Paused, valor novo, borda da caixa nos Cards, rótulo e célula da Strip.
- **Números** — `ph2d_tokens::value_ink`: luminosidade e croma do `info` do tema no **matiz do acento** → `168,208,253`
  no `dark` (maquete `#9fd0ff`). `Ink::Number` deixa o laranja do realce do código (`table_text::code_ink` → `None`,
  pintado por `ph2d_ui_table::ink_color`). Texto continua verde do realce, booleano/nil a constante.
- **Faixa** — ■ de quina 2 px; «line» e «·» em `Text2`, número e função em título (`code.debug.line_word`, `code.debug.sep`);
  pílula mais baixa; chip em `Xs`; **a tecla DENTRO do botão** (Continue cheio no acento, tecla `AccentFg`; Next line
  `Bg3` + moldura `Border`, tecla `Text3` `Xs`) — pintura própria em `head::buttons` (deixa o `paint_button`).
- **Tabela** (`ph2d-ui-table::paint`, assinatura `(base, head)`): cabeçalho e linhas de grupo no tom do chão
  (`cards::ground_tone`, novo), títulos do cabeçalho `Xs` `Text3`, grupos «▾ Locals · 1» em `Xs` (título `Text2`,
  contagem `Text3`), fio por baixo de cada linha, coluna Type (`Ink::Muted`) em `Xs` centrada pela altura real
  (`ui_font`). Coluna lateral: títulos e `ln N` em `Xs`, Waiting em `Text2`; pé em `Xs`.
- **Cards** — cada cartão no tom do chão (`Radius::Md`), título `Xs` `Text2`, contagem `Text3`; caixa de valor no tom
  do cartão com moldura `Border` (âmbar quando mudou).
- `ph2d-app-code`: zero linhas.

**Diferenças que ficam:** o peso do texto (o `paint_text` é MEDIUM com o reforço de nitidez da casa; a maquete é 400 —
mexer é lei do `ph2d-editor-core`, para todo o app); o raio da pílula é o `frame_radius` do tema (quinas quase rectas
no `dark`); a Type diz `number` onde a maquete escreveu `object · Circle` (§3); o chão dos Cards é mais escuro que o
`--a-head` da maquete (é o chão real dos cartões da casa).

### 8.1 Segunda rodada (revisão do lado a lado)

- **Linha cortada por baixo do cabeçalho** (passo 2: «4294967290 number» lia-se dentro de Name/Value/Type): o recorte
  funcionava; a causa era a rolagem contínua deixar meia linha no topo do corpo. Cura na raiz, `ph2d-ui-table`:
  `layout` arredonda a rolagem a LINHAS INTEIRAS (roda, revelar, fim do arrasto da barra) e o máximo arredonda para
  cima (`max_scroll_rows`: a última linha fica inteira); só se pintam/recebem linhas inteiras (nem meia no topo, nem
  cortada no pé); a roda anda pelo menos uma linha; o cabeçalho pinta-se opaco (`base` + `head`) DEPOIS do corpo; a
  barra mede `max_scroll + body.h`. Abrir um ▸ rola até as filhas caberem (`reveal_children`) — sem isto a filha
  aberta podia ficar fora (o seam `o_card_style_troca_o_stopped…` lia-a na meia linha do pé). Teste:
  `a_rolagem_anda_em_linhas_inteiras_e_nada_fica_por_cima_do_cabecalho`.
- **Números**: medido na foto, TODOS os `Number/Vector` (Sheet, Cards, Strip) já saem `168,208,253` (`value_ink`) —
  o «1» parecia branco por ser um glifo fino e claro. `N fields` (`Ink::Muted`) deixou o tom de comentário do realce
  e passou ao `Text3` da casa.
- **Lado a lado**: cada cartão recortado pelos próprios limites (a linha do Continue → a faixa de chão de ≥4 linhas
  acima e abaixo), topos alinhados, escala 1:1; Sheet = passo 3 (âmbar em `self.pos.x`).
- A Sheet mostra 5 linhas no tecto de ⅓ a 1930×1040; não se arrastou o puxador na cena (custaria linhas na cola).

### 8.2 A barra da Sheet tem calha própria (smoke do dono 09/11)

O dono: a coluna Type cortada («numbe», «boolea») — a barra de 10 px pintava-se POR CIMA da última coluna, e o Name
(`flex`) nunca descia abaixo da largura natural, empurrando a Type para fora. Cura em `ph2d-ui-table::layout`:
corpo que transborda ⇒ calha à direita `bar_gutter() = SCROLLBAR_W + Spacing::Sm` (a trilha da casa, `track_rect`, vive
nos últimos `SCROLLBAR_W + 2` px do corpo, dentro da calha); sem transbordo, sem calha. A `flex` toma
`room − outras` e encolhe primeiro (até `min_col`); as outras nunca abaixo da natural (título e célula mais longa + recuo
dos dois lados) — o recuo da última coluna é o mesmo das outras. Teste
`o_corpo_que_transborda_da_a_barra_uma_calha_so_dela` (Σ larguras + calha = área; a trilha não toca coluna; Type ≥
natural; sem transbordo Σ = área). Foto `passo_1` a 1:1: «number», «boolean», «number · changed» inteiros com recuo,
a barra sozinha na calha; Cards: as caixas param antes da barra do self; o código não passa por baixo da barra dele.
