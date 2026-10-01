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

## §6 — Adenda (mesmo dia): o cartão no Black, e o TEMA CUSTOM dos cartões

Ordem do dono: *«No tema Black o fundo do painel e o fundo dos cards é igual e desse modo os cards
não são visíveis. Ainda não temos um botão para resetar todos os themes dos cards e nem um botão
para salvar o theme custom. Corrija tudo.»*

### 6.1 O cartão no `Oled`

- **Causa:** base preta com `contrast = 0` ⇒ `bg-1`, `bg-2` e `panel-bg` são os três `#000000`, e a
  *Draw Extra Borders* que o gate `the_oled_theme_separates_by_border` dava como a separação **nunca
  chegava ao cartão** — `section_cards::paint_into` só PREENCHE. *O gate media a porta
  `frame(Oled, Rest)`, não o pintor.*
- ⛔ **O `bg-1` NÃO sobe:** é também o fundo do CANVAS (`hero::canvas_backdrop`), preso pelo gate
  `the_canvas_ground_is_the_one_the_owner_approved` (report de 05/09, *«mudou a cor do canvas»*) —
  a 1.ª tentativa desta adenda subiu-o e esse gate reprovou.
- ⇒ `derive::card_surfaces` (degrau ABSOLUTO acima do painel quando o cartão está a menos de um
  `SURFACE_STEP` dele: cartão `+2` degraus `#141414`, sub-cartão `+3` `#1f1f1f`) exposto pela porta
  `derive::card_surface(theme, sub) -> Option<Color>` (`None` = pinte o token, byte a byte o de
  sempre) e lido pelo `CardDepth::fill`, que o `paint_into` usa. `Roles` ganhou `card`/`subcard`.
- ⭐ **O gate `a_card_stands_off_its_panel` varre agora os OITO modernos** (varria três) e mede a
  tinta do cartão pela mesma porta. Ele apanhou o **`Candy`** a `9/255`: base `#f7cfe3 → #e3bdd1` e
  contraste `−0,06 → −0,08` (medido: cada metade sozinha dá `10`/`11`; as duas juntas passam). As
  amostras de cor do Candy nos dois menus seguiram a base.
- Gate de PINTOR novo: `section_cards::tests::in_black_the_card_is_not_the_panel_colour` (régua é o
  `draw_data`).
- ⚠️ Os ~26 sítios que pedem `CardDepth::X.token()` para saber *em que superfície assentam* (Painter,
  Inspector, Áudio) continuam a ler `bg-1`/`bg-2` (preto no Oled) — as marcas deles leem-se pela
  moldura extra; só o PREENCHIMENTO do cartão mudou.

### 6.2 O TEMA CUSTOM

- Três linhas novas no menu do botão direito do título de uma secção (nas duas famílias), entre os
  temas e os contornos: **Reset All Card Themes** · **Save Custom Theme** · **Load Custom Theme**.
  Agem sobre o PAINEL inteiro. Ids `CTX_MENU_SECTION_THEMES_{RESET,SAVE_CUSTOM,LOAD_CUSTOM}`
  (append em `ids/menus.rs`), chaves `chrome.menu.section_themes_*`, registo no `pre_populate`,
  despacho em `theme_menu::apply_custom_theme_click` (chamado pelo `apply_section_theme_click` — o
  `hero.rs` está a 696/700 e não foi tocado).
- Lei em `interaction/state/section_prefs_ops/tema_custom.rs` (filho do módulo das secções: o
  `state/mod.rs` está a 696/700). `SectionPrefs.custom: Option<BTreeMap<NodeId, Theme>>` — `None`
  nunca gravou, `Some(vazio)` gravou «todas no app». O *Reset* não toca no custom.
- ⚠️ **A escolha de cada secção já se gravava sozinha** desde 29/09; o custom é uma cópia SEPARADA,
  «um sítio para onde voltar». Viaja no mesmo `~/.ph2d/sections.txt` (`custom=1` + `custom.<id>=`),
  com as duas chaves isentas no censo HR-15 com o mecanismo.
- A marca do menu acende *Load Custom Theme* quando a combinação de agora É a guardada.
- ⏳ **Aberto:** *Load* sem nada gravado não faz nada e a linha não se desliga (o menu não tem linhas
  desactivadas, e a tabela é estática por `ContextMenuKind`).

### 6.3 Prova

- Mutação **4 de 4** a sangrar: `paint_into` de volta ao token · `reset` a apagar o custom · o braço
  do *Reset* trocado pelo do *Save* · a marca `custom=1` só com entradas.
- `nextest-impacted` **19 089/19 089** · clippy `-D warnings` zero nos crates tocados · `fmt` limpo ·
  censos da árvore COMBINADA **127/127**. Fotos `Oled` e `Candy` (cartões do Inspector visíveis).

## §7 — Adenda (mesmo dia): o slider `Tinted` é o padrão do app

Ordem do dono, com foto do Widget Lab: *«Escolhi o tipo 1. Mas agora quero um slider que mistura o
tipo 1 com o tipo 4. Isso para todos os sliders do app.»*

- `SliderDesign::Tinted` (1.ª variante e `#[default]`; `ALL` 4 → 5, com ele à frente porque o
  selector abre na primeira): a caixa e a linha de valor de 2 px do `Underline` + o preenchimento
  `AccentSoft` do `Ghost` por baixo do texto (só as quinas da esquerda arredondam, excepto cheio).
  Pintado no `property_box::paint_surface`, o único sítio onde os desenhos divergem.
- ⚠️ **Os DOIS defaults mudaram juntos** (`SliderStyle::default()` e o `thread_local` const do
  `published.rs`), e o gate `the_paint_default_matches_the_token_default` segura-os. O gate do
  dono virou `the_default_is_tinted_radius_four_row_twentytwo` e a lista de nomes
  `the_customisation_offers_exactly_the_chosen_designs` (5 nomes; a citação no `catalogue.rs` do
  lab seguiu o nome novo). Raio 4 e linha 22 não mudaram.
- Gate de PINTOR novo: `tinted_paints_the_ghost_fill_and_the_underline_line` (as duas tintas na
  cena, com o `Underline` e o `Ghost` como controlos). Mutação 1/1 a sangrar.
- ⚠️ O desenho **não se grava em disco** (só o Widget Lab o muda, por sessão) — logo não há
  migração: o app inteiro passa ao `Tinted` no arranque.
- Prova: `nextest-impacted` **19 090/19 090** · clippy `-D warnings` zero · `fmt` · censos
  **127/127** · foto do Widget Lab e do painel Vector com o desenho novo.

## §8 — Adenda (mesmo dia): contraste da caixa, amostras de 4 cores, e o menu de tema + arrasto em TODOS os painéis de secções

Ordem do dono, com três fotos: *«Veja como o checkbox tem pouco contraste em Light e Candy.
Corrija! … em vez de uma única cor no retângulo do theme, melhor 4 retângulos pequenos com as
principais cores de cada theme. Depois siga com os outros painéis.»*

### 8.1 O contraste e as amostras (`a684a2591`)

- `on_field_fill` afasta a marca da caixa até `CONTRASTE_DA_MARCA` (`36/255`) com UM ganho tirado do
  repouso (o eixo do hover mantém a ordem). Só Light e Candy (`10/255`) se mexem; os outros temas
  ficam byte-idênticos (gate com as duas metades e o controlo da lista).
- A amostra das linhas de tema (selector do topo e menu da secção, App Theme incluído) passa a ser
  quatro ladrilhos `2×2` tirados do tema: painel · cartão (pela porta do cartão) · acento · texto.
  Os `24` hex das tabelas saíram (HR-15).

### 8.2 A lei sai do Inspector e o LAÇO fica partilhado

- `750c7b52a`: a lei (ordem · tema da secção · corredor · marca de queda · fantasma) muda-se para
  `section_plan`; o `HitIndex` ganha o LIVRO das secções do quadro (`register_section`: secção ·
  pega · cabeçalho recortado), lido pelo despacho do botão direito e do arrasto. A pega é
  `ids::grip_de(secção)`. O `Corredor` só fecha o cartão anterior **se ele pintou**. Vector = 2.º.
- `da18c1224`: o laço para os painéis de `PaintCtx` é `panel::section_plan_ctx::PlanoCtx`, com três
  espécies — `seccao` (arrasta-se e muda de tema), `fixa` (muda de tema, sem pega, fora dos alvos
  de queda: `register_fixed_section`) e `bloco` (linhas sem título). Fixas e blocos pintam-se
  primeiro, pela ordem declarada.
- `6b66f2b11`: a lei muda-se de `panel` para `widget` (a galeria é `widget` e `widget` não pode ler
  `panel` — DAG); `panel::section_plan` fica como re-exportação.
- ⛔⛔ **Nesta adenda, o `PlanoCtx` passou a FECHAR o cartão da ÚLTIMA secção** (`Corredor::fecha_a_ultima`,
  só em tema moderno e dentro de cartões), e as fixas re-tematizam-se DEPOIS disso. O
  `end_section_cards` pinta só cartões FECHADOS ⇒ a última secção ficava **sem cartão e sem o tema
  que o artista lhe deu** (achado do agente da Física). ⚠️ A mensagem do `da18c1224` dizia o
  contrário (*«o cartão da última fica aberto para o `end_section_cards`»*) — ela estava errada, e
  a altura de abertura do Painter mostra-o (§8.4).

### 8.3 Quem é o quê, por painel

| Painel | Arrastam-se | Fixas | Blocos / fora |
|---|---|---|---|
| Inspector | as de sempre | Nome/Visibilidade | — |
| Vector | todas as do corpo | — | — |
| Painter (Brush) | as 8 da aparência | Máscara · o MEIO da tinta | topo, Digital, Mixing; a rampa do Grain é ANINHADA (fora do livro) |
| Widget Gallery | as 11 (`SECCOES` ≡ `SECTION_IDS`, gate) | — | — |
| Physics | as 9 (`SECTIONS` + as 4 pintadas à mão, `debug_assert` a prendê-las) | — | — |
| Grid Snap | Kind · Target · Display · Inspect | — | Snap (bloco) |
| Sculpt3d | Brush · Symmetry · Topology · Shading · Scene · Bake | **Tool** (decide que linhas as outras têm) | — |
| Wet tuning | as 6 (Paint · Water · Physics · Tools · Paper · Experimental) | — | — |
| Audio editor | as 8 | — | — |
| Audio mixer | os 5 efeitos (EQ · Reverb · Delay · Comp · Ducking) | — | as fitas de canal; Play Test/loudness/Limiter |

- ⛔ **`Authored` fica FORA de propósito:** as secções dele são a árvore que o ARTISTA autorou; a
  ordem mora nela, e uma segunda ordem por cima seria duas respostas à mesma pergunta.
- ⛔ **O `paint_card_params` do Motion graph não é corpo de painel** — são cartões de NÓ.
- ⚠️ **Sculpt3d:** os pintores perguntavam o tema ao host (sempre o do painel) ⇒ `plano::tema(ctx)`
  é a única porta (thread-local posto enquanto a secção pinta), com censo a recusar `host.theme()`
  nos pintores de secção. Trocar as assinaturas eram ~100 sítios.
- Cartões adoptados agora (não tinham): Physics · Grid Snap · Sculpt3d · Wet tuning · Audio mixer.

### 8.4 Gates que mudaram, com a conta

- `architecture_the_foundation_modules_form_a_dag`: `widget → interaction` **44 → 43** (as duas
  importações do `segmented.rs` fundiram-se).
- `ph2d-widget-sync`: `section_plan` em `PUB_MODULE_OVERRIDE` (o bloco gerado do `widget/mod.rs`).
- `architecture_widget_showcase_coverage`: `section_plan` no `WIDGET_OPT_OUT` (é LEI, não widget).
- `architecture_panel_loc_cap`: `sculpt3d::populate` (`208`) e `audio_mixer::populate` (`201`) curados
  por CORTE (`register_sections` · `populate_sections`), nunca por isenção.
- `a_altura_de_abertura_de_um_painel_so_encolhe` — ⚠️ **nenhuma secção nasceu aberta**; a conta de
  cada número está escrita ao lado dele no ficheiro:
  - `painter_layers` `1 596 → 1 605`: o cartão da última secção passou a existir (§8.2).
  - `sculpt3d` `2 186 → 2 196` · `physics` `1 281 → 1 317` · `audio_mixer` `1 207 → 1 237`: os
    painéis passaram a cartões; o corredor (`12`) e o fecho do último substituem os vãos à mão.
  - ⛔⛔ `inspector` lia `822` contra `831` — **e era um DEFEITO do `750c7b52a`, não um ganho.**
    Bissectado numa worktree à parte (`a684a2591` = `831`, `750c7b52a` = `822`) e localizado por uma
    sonda que imprime o controlo mais fundo a cada passo: tudo até ao bloco do topo (os botões do
    objecto) igual, e do campo do Nome para baixo **`−9 px`**. O plano antigo fechava SEMPRE antes da
    1.ª secção, e isso punha o bloco do topo num cartão próprio; o `Corredor::default()` salta esse
    fecho e o bloco caía DENTRO do cartão da 1.ª secção. ⚠️ A minha 1.ª hipótese (a regra «só fecha se
    pintou») foi **refutada por mutação** antes desta — com ela desligada o Inspector continuava em
    `822`. Cura: `Corredor::com_conteudo_acima()`, que reproduz o comportamento antigo nos DOIS temas
    (no clássico o plano antigo também desenhava o separador ali). O número fica `831`.

### 8.5 Prova

- Testes das crates: Painter `208` (+1 ignorado) · Physics `29` · Grid Snap `35` · Galeria `10` ·
  Sculpt3d `139` (+2) · Wet tuning `17` · Audio editor `96` · Audio mixer `37` · `architecture_`
  `97/97` · `ph2d-panel-registry-init` `132` no âmbito de WORKSPACE (⚠️ com `-p` ele reprova por
  ambiente: `painter_layers`, `flip`, `flip_frames` e `wet_tuning` só entram pela unificação de
  features).
- Mutação: Painter `2/2` · Galeria `2/2` · Physics+Grid `4/4` · Sculpt3d+Wet `6/6` · Audio `4/5` por
  crate — a 5.ª (só o `register(.., Plain)` da pega) **sobrevive e é NOMEADA**: nenhum gesto a
  observa (o arrasto acha a pega pelo livro do quadro e o hover pelo `HitIndex`); fica pela
  paridade com Inspector/Vector/Painter.
- Clippy `--all-targets -D warnings` zero nas crates tocadas · `fmt` limpo.

## §9 — Adenda (2026-10-01): os dois abertos desta frente fecharam, e um terceiro achado pelo caminho

Ordem do dono: *«smoke ok. siga»* — fecharam-se os dois ⏳ deste handoff.

### 9.1 *Load Custom Theme* sem nada gravado fica APAGADA (o ⏳ da §6.2)

- Porta única `WidgetStore::menu_row_is_unavailable(id)` (`tema_custom.rs`) — hoje só a linha *Load* com
  `custom == None`. ⚠️ Um custom VAZIO é gravação, logo a linha acende com ele.
- **Dois leitores**: o pintor (`context_menu_overlay.rs` — texto `Text3`, sem realce de hover) e o
  `click_belongs_to_the_open_menu` (braço `SectionOutline`: o Down numa linha indisponível NÃO é «clique
  fora», o menu fica aberto). O clique a seguir chega ao verbo, que já recusa sem gravação.
- ⛔ Uma 3.ª guarda no despacho do clique foi escrita, medida por mutação (**sobreviveu**) e APAGADA: o
  verbo já recusa e o clique nunca fecha o menu VIVO (`consume_last_context_menu` só toca a fotografia).
- Gates: `load_esta_indisponivel_ate_haver_o_que_por` · `load_sem_gravacao_nao_age_nem_fecha_o_menu` ·
  `the_down_on_an_unavailable_row_keeps_the_menu_open` (com o controlo gravado) ·
  `the_painter_reads_the_unavailable_door` (censo de texto, prosa cortada). Mutação **3 de 3**.

### 9.2 As notas da Galeria e do Inspector têm caixas PRÓPRIAS (o ⏳ da §1, item 4)

- `ids::notes`: `NoteIds { slot, title, body, grip }` · `INSP_NOTES` (as tabelas de sempre, nomes
  `insp_note_*` intocados) · `GAL_NOTES` **derivadas** por XOR com `SAL_DA_GALERIA` (o idioma da
  `grip_de`, bijecção, sem tabela à mão) · `NOTE_HOSTS` (a única lista de quem pinta notas) ·
  `note_ids(panel)` · `is_note_text`. `note_index_of`/`note_of_grip` procuram nos dois hosts.
- Consumidores: `permute_note_texts(panel, order)` (⚠️ **assinatura mudou** — ganhou o `panel`),
  `populate_notes` regista os dois hosts, o `CreateNote` escreve nas caixas do painel do pedido,
  `paint_one_note(.., caixas: &NoteIds, slot)` (⚠️ **assinatura pública mudou**, a Galeria passa
  `GAL_NOTES` e o Inspector `INSP_NOTES`), `lugar_da_queda` e o fantasma lêem as ranhuras do painel.
- ⚠️ O `seed` do arrasto ficou como estava (`note_of_grip` + `panel_at`): com ids distintos por painel a
  versão «a pega tem de ser do painel» é **inobservável** e foi revertida.

### 9.3 ⛔ *Create Note* era um botão MUDO em ~15 painéis (achado pelo caminho)

- O botão direito decidia por uma lista de **EXCLUSÃO** (hierarquia, ferramentas de imagem, camadas,
  timeline, grade, assets) e todo outro painel oferecia *Create Note* — mas **só o Inspector e a Galeria
  pintam notas**. No Vector, na Física, no Áudio… a nota nascia no store e nunca aparecia.
- Hoje a pergunta é `note_ids(panel).is_some()` — a MESMA tabela das ranhuras. ⚠️ Por construção isto
  também deixa de oferecer o menu sobre o selector de cor, o overlay de áudio e janelas flutuantes (o
  comentário da guarda do selector foi corrigido: o fallback continua a FECHAR o menu e a consumir).

### 9.4 Prova

- Gates novos: `os_noventa_e_seis_ids_das_notas_sao_distintos` · `as_notas_de_um_painel_nao_tocam_no_texto_do_outro`
  · `permutar_num_painel_sem_notas_nao_toca_em_nada` · `na_galeria_a_queda_conta_as_ranhuras_dela` ·
  `create_note_opens_only_on_a_panel_that_paints_notes` (controlo Inspector/Galeria; Vector e Física
  mudos) · e o `gallery_create_note_targets_gal_panel` estendido (título na caixa da Galeria; pintura com
  as ranhuras dela, nos DOIS pintores — o da nota sem secção só se vê rolando ao fundo pela altura publicada).
- Mutação **8 de 8** (sal a zero · os dois pintores da Galeria com `INSP_NOTES` · *Create Note* em todo
  painel · `GAL` a apontar `INSP` · permutação, queda e criação a ignorar o painel).
- `nextest-impacted` **19 138/19 138** · clippy `--all-targets -D warnings` zero nas 3 crates · `fmt` limpo.
- ⚠️ **Superfície de colisão**: `permute_note_texts` e `paint_one_note` mudaram de assinatura (quem os
  chamar noutra linha não compila — falha ALTA, barata).
