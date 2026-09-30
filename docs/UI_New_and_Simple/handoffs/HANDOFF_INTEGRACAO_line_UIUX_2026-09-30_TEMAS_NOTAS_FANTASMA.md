# Handoff de integração — `line/UIUX`, 2026-09-30 — OS QUATRO TEMAS COLORIDOS, AS CORES VIVAS, O FANTASMA DO ARRASTO E AS NOTAS COM PEGA

> Leitor: o agente integrador e a próxima janela desta linha. Estado vivo: `CLAUDE.md` §5.
> Base: merge-base `912a9652e` (= `main`, que não andou). Continua o handoff dos CARTÕES
> (`HANDOFF_INTEGRACAO_line_UIUX_2026-09-30_OS_CARTOES.md`), que segue válido para o que descreve.

## §0 — A ordem do dono (2026-09-30, depois do smoke OK dos cartões)

*«Crie 4 outros themes, dessa vez bem coloridos. um com as cores do tema de Mario, outro com as
cores de Luigi. Escolha outros 2 themes. Coloque mais 4 cores vivas para outline (vermelho, azul,
verde e amarelo). Ao arrastar um card para organizar o painel, permita ver o card sendo arrastado,
menor e meio transparente. Veja que podemos criar notas com o botão direito do mouse. Coloque os
10 pontinhos de arrastar também nas notas. Botão direito sobre as notas devem ter no menu a
mudança de cor das notas, opções de apagar e duplicar.»*

## §1 — O que mudou, por item

### 1. Quatro temas coloridos — DERIVADOS, nenhum slot à mão

`Theme::{PlumberRed, PlumberGreen, Sunset, Candy}` **no FIM do enum** (`Theme::ALL` 8 → 12,
`MODERN` 4 → 8). Cada um são as **cinco entradas** de `derive::Inputs::of` (base · acento ·
contraste · escuro · bordas extra) e mais nada — a mesma derivação dos quatro do Godot, e o mesmo
gate WCAG (`contrast_tests.rs`) a medi-los. ⚠️ **Os nomes não levam a marca registada** (Mario e
Luigi são da Nintendo): *Plumber Red* e *Plumber Green* vivem na tabela de traduções e mudam-se lá.
⚠️ `Candy` é o único CLARO e usa o contraste negativo do `light`.

⭐ **O byte do ficheiro do projecto** (`shells/desktop/src/project_tokens.rs::theme_from_u8`)
ganhou `8..=11`, também no fim — nenhum byte anterior se mexe. O gate
`the_mode_round_trips_through_its_byte` varre `Theme::ALL` e **reprovou** até isto entrar (um
projecto com override no tema vermelho reabriria no `Forge`, em silêncio).

⛔⛔ **E o 1.º tema COLORIDO derrubou uma RÉGUA, não o produto:** o
`a_marca_nunca_e_a_cor_da_caixa_em_que_assenta::e_o_pintor_da_linha_de_marcar_usa_essa_porta`
empacotava as cores como `argb` e o `draw_data` do Vello guarda `[r, g, b, a]` little-endian
(`a<<24 | b<<16 | g<<8 | r`). Nos temas cinzentos `R == B` e as duas ordens coincidem, logo a régua
**nunca tinha medido a ordem**; no `PlumberRed` ela leu `FF480409` procurado contra `FF090448`
pintado — a caixa estava lá. Curado na régua, com o porquê escrito ao lado.

### 2. Quatro cores VIVAS no marcador

`HIGHLIGHTER_RGBA` 5 → **9** (vermelho · azul · verde · amarelo vivos), **acrescentadas no FIM**
(o índice é o que a nota e o contorno guardam). Porta nova `highlighter_rgba(idx)`: um índice fora
da paleta cai na ÚLTIMA cor, nunca estoura. Menu de contorno e de nota com 9 cores + «nenhuma».
A paleta mudou-se para `widget/panel_chrome/highlighter.rs` (corte por assunto: o `panel_chrome.rs`
foi a 511 contra o tecto de 500).

### 3. O FANTASMA do cartão arrastado

`widget::drag_ghost` (`paint_card_ghost`, `afim_do_fantasma`): o conteúdo do cartão é pintado numa
`VectorScene` própria e reacrescentado com um afim — **escala `DRAG_GHOST_SCALE = 0,75` à volta do
PONTO DA PEGA, e esse ponto posto sob o cursor** — dentro de uma camada de alfa
`DRAG_GHOST_ALPHA = 0,55`. ⚠️ Escalar à volta do centro deixaria a pega a fugir do dedo.
Serve os dois arrastos: a secção do Inspector (o `plano` pinta a secção arrastada numa sub-cena e
devolve um `Fantasma`, pintado DEPOIS das notas do fim) e a nota (`paint_note_drag_ghost`).
`SectionDrag` ganhou `down_x`/`cursor_x` (o fantasma precisa do X).

### 4. As NOTAS: pega de 10 pontos, arrastar, menu com cor / duplicar / apagar

⭐⭐ **A nota deixou de guardar uma POSIÇÃO e passou a guardar a SECÇÃO a que pertence**:
`NoteData.before_section: Option<u8>` → **`section: Option<NodeId>`** (`None` = no fim do painel).
A posição por índice partia-se em dois sítios: ao reordenar secções (a nota ficava presa ao índice
e mudava de vizinha) e em secções opcionais (o índice contava secções que não estavam lá). O
`WidgetEvent::CreateNote` leva `section` (a secção sob o clique, lida do hit-index DESTE quadro por
`note_drag::seccoes_do_painel`/`seccao_sob`).

⛔ **E o menu do botão direito sobre a nota nunca abria** (pré-existente): o despacho reconhecia a
nota por uma faixa de ids `800..=811`, e os ids das notas são hashes. Hoje é `ids::note_index_of`,
uma porta.

- **A pega** (`NOTE_GRIP_IDS`) é pintada no título, registada como `Plain` com dica
  (`chrome.note.grip_hint`) e **vence o título** no `pointer_down` (antes da porta do foco). O
  arrasto (`NoteDrag`) arma depois do limiar e larga pela MESMA lei que o fantasma mostra
  (`note_drag::lugar_da_queda`: a secção sob o cursor e o lugar entre as notas dela).
- **Menu**: as 9 cores + «nenhuma», **Duplicate Note** (`CTX_MENU_NOTE_DUPLICATE`) e **Delete
  Note** (`CTX_MENU_NOTE_DELETE`). Tecto `NOTES_PER_PANEL` respeitado no duplicar e no criar.
- ⚠️⚠️ **O texto de uma nota vive no `WidgetStore` por RANHURA** (`NOTE_TITLE_IDS[i]` /
  `NOTE_BODY_IDS[i]`), não na `NoteData`. Mover, apagar ou duplicar permuta as ranhuras ⇒
  `permute_note_texts(ordem)` leva o texto com a nota (e larga o foco de uma nota, senão o caret
  ficava a escrever na ranhura errada). Gates em `notes_ops_tests.rs`.
- ⏳ **DÍVIDA PRÉ-EXISTENTE, nomeada e não curada:** esses ids de texto são **partilhados entre a
  Galeria e o Inspector** (uma nota com o mesmo índice nos dois painéis lê o mesmo texto). Curar
  pede ids por painel — é modelo, não fiação.

⭐ **A geometria das secções e das notas passou a sair do hit-index do quadro anterior** e não de
uma tabela paralela: `LAST_SECTION_TOPS_Y`, `LAST_BODY_TOP_SCREEN_Y`, `push_section_top_y` e
`section_index_below_body_y` **morreram** (eram a segunda resposta à pergunta «onde está a secção
N?»). O `begin_section`/`finish_section` do Inspector perderam o encanamento de `tops`/`body_top_y`
(~30 pintores tocados só para tirar o parâmetro).

## §2 — Superfície de colisão (o que a fusão pode partir)

- **Assinaturas mudadas** (quem as chama noutra linha não compila — falha ALTO): `NoteData.section`,
  `WidgetEvent::CreateNote { panel, section }`, `begin_section_drag(section, x, y)`,
  `update_section_drag(x, y)`, `section_drag::seed(store, id, x, y)`,
  `handle_down_menus(store, hit_index, hit, event)`, o `begin_section`/`finish_section`/`emoldurada`
  do Inspector, `plano::run` devolve `(f32, Option<Fantasma>)`.
- **Apagados:** `widget::showcase::state::{LAST_SECTION_TOPS_Y, LAST_BODY_TOP_SCREEN_Y,
  push_section_top_y, section_index_below_body_y}`; os arrays de ids de nota mudaram de
  `widget::showcase` para `ids::notes` (re-exportados na galeria com o mesmo nome).
- **Enums append-only:** `Theme` (+4 no fim), `HIGHLIGHTER_RGBA` (+4 no fim), ids de menu novos.
  ⚠️ Uma linha que tenha acrescentado ao `Theme` no mesmo dia escreve no MESMO fim — reconte a
  ordem e o byte do `theme_from_u8`.
- **Catraca do DAG** `widget → interaction`: **46 → 44** (a galeria juntou num `use` as importações
  que repetia; as notas trouxeram uma aresta nova dentro do `use` que já existia). Outra linha que
  acrescente referências reprova em cima de 44, não de 46.
- **Tectos de LOC curados por CORTE**: `panel_chrome.rs` (511 → 484, paleta para `highlighter.rs`)
  e `screens/hero/pre_populate.rs` (703 → 670, notas para `pre_populate_notes.rs`).
- **Opt-outs novos, com razão**: `drag_ghost` na `WIDGET_OPT_OUT` e na `A11Y_OPT_OUT`,
  `panel_chrome/highlighter.rs` na `A11Y_OPT_OUT`.
- **Contadores partilhados:** `PROJECT_SCHEMA` **0**, registos **0**, zero contrato (§6), zero ADR.
  ⚠️ O `NoteData` vive no `WidgetStore` (estado de painel), **não** no ficheiro do projecto.

## §3 — Prova

- `nextest-impacted`: **19 084 / 19 084** (a 1.ª corrida deu 8 vermelhos, todos curados acima).
- `censos-da-arvore-combinada.sh`: **127 / 127**, controlo do filtro 12 de 12.
- `clippy --all-targets -D warnings` nas 7 crates tocadas: zero. `cargo fmt --check`: limpo.
- Gates de costura novos: `ph2d-panel-inspector/tests/it/as_notas_tem_pega_e_menu.rs` (a nota
  nasce na secção sob o clique; a pega vence o título; o botão direito no título abre o menu da
  nota; duplicar/apagar; arrastar a pega para a Sprite Sheet muda a secção e o fantasma tem a
  geometria certa) e a metade do fantasma em `a_pega_reordena_as_seccoes.rs`.
- ⚠️ **O fantasma NÃO foi fotografado** — ele só existe com o dedo a segurar a pega, e o XTest é
  ignorado na Xwayland virtual (o `ydotool` moveria o rato real do dono). A geometria dele prova-se
  nos gates de costura, que medem a cena pintada; o que se vê é do smoke do dono.

## §4 — Adenda (mesmo dia): os OITO temas originais ganharam cor

Ordem do dono, depois do smoke dos quatro coloridos: *«Tão bons que os originais ficaram um pouco
sem graça. Tente melhorar um pouco os originais de modo que sejam interessantes como os novos.»*

- **Os quatro modernos** (`derive::Inputs::of`): cada um ganhou uma MATIZ na base e um acento mais
  vivo, guardando o carácter do nome — `Dark` `#1e2433` + `#4aa3ff` (era `#292929` + `#569eff`) ·
  `Gray` `#3d3833` + `#2ec4b0` · `Light` `#e6dccb` + `#4150d8` · `Oled` preto + `#b57bff`. O Godot
  fica como a ORIGEM da regra (cinco entradas e a derivação), não das cores.
  ⚠️ A base do `Light` **não pode ser mais clara que a do Godot**: o painel sobe acima dela e satura
  no `255`, e o degrau cartão/painel encolhe (`#ede4d3` mediu `10/255` contra a barra de `12`).
- **Os quatro clássicos** (`docs/design/tokens.json`): a escada neutra (`bg-*`, `border*`,
  `text-*`, `canvas`, `panel-bg`, `window-ground`, `rail-bg`, `graph-bg`, `graph-inert`) teve o
  croma multiplicado — ×4 no fundo e ×2 no texto no `forge` (ameixa), ×2,5/×1,5 no `sunstone`
  (creme) e no `blueprint` (azul-papel), com tecto de `0,045` — e os acentos subiram `+0,02`–`+0,035`.
  ⭐ O **`workshop` ganhou escada PRÓPRIA** (a do `forge` com matiz `220`, petróleo): antes herdava
  o fundo do forge, e herdar a ameixa punha o acento ciano sobre roxo.
- ⛔ **Um gate teve a premissa MORTA por esta ordem e foi reescrito com a morte à vista:**
  `the_dark_preset_is_godots_default` (as entradas do Godot, byte a byte) passou a
  `o_dark_guarda_o_caracter_do_nome` (base e acento azuis, o mais escuro dos escuros com base
  não-preta, e os dois chegam ao app intactos).
- ⚠️ **E uma régua lia só o VERDE:** `a_card_stands_off_its_panel` media o degrau cartão/painel no
  canal `g`, que numa base cinzenta é igual aos outros dois; com uma base tingida ele mede a matiz.
  Passou à média dos três canais.
- ⚠️ **O fundo do canvas muda de cor em todos os temas** (o `Bg1` é o fundo do canvas, e o dono já
  devolveu uma vez *«mudou a cor do canvas»*, em 05/09). Aqui é o pedido: os coloridos fazem o
  mesmo desde que nasceram. Se ele o recusar, a alavanca é a escada de fundo (`bg-0`/`bg-1`) e não
  o acento.
- Prova: `nextest-impacted` **19 084/19 084** · censos **127/127** · clippy e fmt limpos · as oito
  cenas FOTOGRAFADAS com `PH2D_THEME=<id>` (`fotografa_cena.sh`, a cena do script).

## §5 — Adenda (mesmo dia): quinas redondas entre botões vizinhos

Report do dono: *«houve uma regressão na forma de alguns widgets como os botões que em alguns
casos ficaram com quinas redondas mesmo se ao lado de outros. corrija»*. Eram DUAS coisas:

1. ⛔⛔ **O CONTORNO de uma peça de grupo ignorava a posição.** O `paint_button` e o
   `paint_segmented_button_in_group` já enchiam o FUNDO com `cell.radii(r)` (só as bordas de fora
   arredondam — wave 10/20), e traçavam a MOLDURA e o anel de FOCO com o `radius` uniforme. Num tema
   moderno sem moldura em repouso isso não se via; **em todo tema que traça bordas (os quatro
   clássicos, o OLED) e no foco de qualquer tema, a quina redonda voltava entre vizinhos** — e
   passou a ver-se quando o dono foi experimentar os temas. Portas novas
   `paint::stroke_rounded_rect_radii` e `paint::stroke_frame_radii` (quatro raios iguais delegam
   nas de um raio ⇒ um botão sozinho é **byte-idêntico**). Gate
   `o_contorno_de_uma_peca_de_grupo_segue_as_quinas_do_fundo` (Forge · Sunstone · OLED, botão E
   segmento): a régua é o PONTO da quina no `path_data` — o traço de uma peça do meio passa por
   `(x, y)`, uma quina redonda nunca; o controlo é a peça sozinha (zero pousos). ⚠️ O gate irmão
   de sempre media só o `Dark`, que é exactamente o tema sem moldura. **Mutação 2 de 2** (o traço
   do botão e o do segmento de volta ao raio uniforme ⇒ vermelho).
2. **O transporte da Timeline e o `+ ⧉ T 🗑` dos clipes eram botões de ÍCONE soltos** — cinco (e
   três ou quatro) peças encostadas com quatro quinas cada, em TODOS os temas. O `Button` e o
   segmento conheciam a lei do grupo; o botão de ícone não. ⇒ `paint_icon_button_in_group` (o
   `paint_icon_button` delega com o neutro `GroupCell::ONLY`, byte-idêntico), `icon_button_in` na
   timeline, as peças a um `SEGMENT_HAIRLINE` e a MEDIDA de cada fileira com o mesmo fio (a
   contagem do corpo dos clipes numa porta só, `botoes_do_grupo`, lida pela medida e pelo pintor).
   Fotografado em Dark, OLED e Forge.

⚠️ **A régua da porta das molduras ganhou a irmã:** `every_frame_goes_through_the_theme_door`
passa a contar `stroke_rounded_rect_radii(` como traço cru e o `stroke_frame_radii` como porta —
sem isso a função nova seria uma fuga ao censo. ⚠️ `transport::paint_item` passou do tecto de
200 LOC ⇒ a fileira de transporte virou `transport_row` (corte por assunto).
Prova: `nextest-impacted` **19 085/19 085** · censos **127/127** · clippy e fmt limpos.
