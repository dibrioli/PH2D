# Handoff de integração — `line/UIUX`, 2026-09-30 — OS CARTÕES DE SECÇÃO, A PEGA E O TEMA POR SECÇÃO

> Leitor: o agente integrador e a próxima janela desta linha. Estado vivo: `CLAUDE.md` §5.
> Base: merge-base `912a9652e` (= `main`, que não andou). Continua o handoff da ROLAGEM
> (`HANDOFF_INTEGRACAO_line_UIUX_2026-09-29_A_ROLAGEM.md`), que segue válido para o que descreve.

## §0 — A ordem do dono (2026-09-29, com o *Properties Editor* do Blender ao lado)

1. *«cada seção é um card muito bem definido, há um pequeno padding entre cards de seções assim
   como um pequeno padding interno no card … Nós por outro lado, ao abrir as seções não temos
   padding entre cards de seções e tudo fica mal definido.»*
2. *«o blender tem no topo de cada seção um ícone de 10 pontos que serve para arrastar e
   reorganizar as seções. Vamos criar isso.»*
3. *«Nós temos em painéis como inspector um círculo cuja única função é dar cor ao círculo. Vamos
   retirar isso … Com o botão direito do mouse sobre o título da seção poderemos escolher o theme
   da seção entre os themes disponíveis para o app.»*
4. *«Temos uma linha separadora nos títulos das seções. Vamos retirá-la.»*

## §1 — O que mudou, por item

### 1. Os cartões encostavam — o vão tinha EXACTAMENTE a soma das duas folgas

Medido na foto (`fotografa_cena.sh`, `PH2D_SCRIPT_SMOKE=1`): painel `#131313`, cartão `#1F1F1F`,
e **nenhuma faixa de painel entre dois cartões**. `section_gap_px()` valia `8 = Xs·2` e cada cartão
recua `card_pad = Xs = 4` para fora do conteúdo nos dois lados ⇒ o fundo de um e o topo do seguinte
coincidiam. Hoje:

- `ph2d_tokens::card_pad_px()` (4) e `card_gap_px()` (4) são portas, e
  `section_gap_px() = card_pad·2 + card_gap` (**12**). Gate `two_cards_never_touch_the_panel_shows_between_them`.
- **Dentro de um cartão o cabeçalho dobrado não pinta a placa `Bg3`** (`section_cards::inside_cards()`):
  o cartão JÁ É a placa. Fora de cartões (clássico, painéis sem livro) a placa fica — é o único sinal
  de dobra ali.
- **Fechar sem nada pintado desde o último fecho é um NO-OP** (`close_at`): o Inspector passou a fechar
  ANTES de cada secção por um corredor só, e as molduras antigas ainda fecham dentro de si; sem a
  guarda, cada fecho duplicado somava um vão. Gate `closing_twice_in_a_row_is_one_close`.
- O Inspector alinha o início do livro com o início do conteúdo
  (`skip_section_header(body_top_y + Xs)`): sem isto a folga do topo fechava como um cartão vazio.

⚠️ **A folga interior horizontal ficou em 4 px de propósito:** o painel deixa 6 px entre a borda e o
cartão; subir a folga comia essa faixa. Se o dono quiser mais folga interior, a alavanca é a
margem do corpo do painel, não o `card_pad`.

### 2. A linha azul do título saiu

`regua_do_titulo`, `SECTION_RULE_PX` e o gate dela foram apagados (a regra era do dono de 2026-09-24;
a de hoje revoga-a).

### 3. O círculo saiu, e o lugar dele é a PEGA

- `SectionHeader::color` → `SectionHeader::grip: Option<bool>` (o `bool` acende-a). `color_circle_hit_rect`
  → `grip_hit_rect`. Módulo novo `widget/section_header/grip.rs` (re-exportado como
  `widget::section_grip`): **10 pontos, 2 × 5**, passo `Spacing::Xs`, ponto `Spacing::Xxs`; o alvo é a
  banda da direita do cabeçalho, da altura dele; `grip_a11y` anuncia-a como botão.
- `SectionHeader::reserve_right(px)` reserva espaço para um controlo do chamador à direita do título
  (o repor da Transform e das secções do Pincel) — antes o `Xl4` do círculo fazia isso por acaso.
- **Os ids:** a 2.ª coluna de `ids::LIVE_SECTIONS` era o ponto de cor e passou a ser a pega —
  `INSP_LIVE_*_COLOR` → `INSP_LIVE_*_GRIP`, `LIVE_SECTION_COLOR_IDS` → `LIVE_SECTION_GRIP_IDS`, e o hash
  muda de nome (`insp_live_*_grip`). Leituras novas `ids::grip_of` / `ids::section_of_grip`.
- **A pega regista-se no `begin_section`** do Inspector (a porta por onde as ~40 molduras passam), DEPOIS
  do cabeçalho — o hit-index resolve o último primeiro. Todas as 41 secções móveis têm pega, incluindo
  as 17 que nunca tiveram ponto de cor. O **Nome** e a **Visibilidade** (as duas fileiras sem
  cabeçalho) não se arrastam (`interaction::SECCOES_FIXAS`).
- **O círculo saiu também da galeria de widgets e das 12 secções do Painter** (os 11 ids
  `INSP_SECTION_*_COLOR`, a `SECTION_COLOR_IDS`, os 12 `PAINTER_*_SECTION_COLOR` e o braço de clique do
  Inspector `section_color_click` — apagados, não órfãos).
- Dica de hover em toda pega (`chrome.section.grip_hint`): ela ensina os dois gestos novos.

### 4. O arrasto — o Inspector passou a ser uma LISTA de secções

- **`crate::plano::Plano`** (ph2d-panel-inspector): os grupos EMPURRAM as secções (`push_*`) e o plano
  pinta-as pela ordem `interaction::ordena_seccoes(natural, autorada)`. A disposição é REAL — cada
  secção pinta-se no sítio onde fica, logo hit-index, popovers e notas registam-se lá (nenhuma
  translação do desenho a corrigir depois). O plano fecha o cartão ANTES de cada secção, resolve o
  tema dela e desenha a marca da queda.
  - `plano::emoldurada` é a moldura de sempre (`begin_section` → corpo → `finish_section`) para as
    secções que viviam em grupos (núcleo, sprite, partilhadas, física, âncoras, vida).
  - As secções opcionais com moldura própria (`paint_optional_*`) entram inteiras, por `plano.push`.
  - `Top20` passou a `Copy` (só referências e `usize`).
- **A lei de ordem** (`interaction::state::section_prefs_ops`, pura, com testes): uma secção que o
  artista nunca moveu nasce LOGO A SEGUIR à vizinha que a precede na ordem natural — nunca no fim
  (o defeito que pôs as Tags em 38.º lugar, 2026-09-21).
- **O gesto** (`interaction::dispatch::section_drag`): Down primário na pega semeia o arrasto — **antes
  da porta da focabilidade**, como o ponto de curva (atrás dela a pega dependia de um `register(Plain)`
  no `pre_populate` do ecrã principal, e o teste do gesto real apanhou-a morta no anfitrião de testes).
  Move avança; Up de um arrasto ACTIVO (limiar das abas) resolve a queda com os cabeçalhos que o
  **hit-index do quadro anterior** registou e grava `store.set_section_order`. Uma lei, dois leitores:
  `alvo_da_queda` é a mesma no despacho e na marca que o plano desenha.
- Botão direito sobre a pega abre o menu da SECÇÃO (a pega fica por cima do título).
- Gates: `a_pega_reordena_as_seccoes` (gesto real no `MockPanelHost`: clique parado não reordena nem
  dobra; arrastar a Sprite Sheet para cima do Render põe-na no topo das três), os do despacho e os da
  lei.

### 5. O tema por secção

- `WidgetStore::{section_theme, set_section_theme}`; o menu do título (`ContextMenuKind::SectionOutline`)
  passou a ter **«App Theme» + os quatro temas da família do app** + o contorno de marcador que já
  tinha. Tabelas em `screens/hero/menu_tables_section.rs` (irmão do `menu_tables.rs` pelo tecto de 700;
  o gate `every_menu_row_reaches_a_handler` lê os três ficheiros). Ids próprios
  `CTX_MENU_SECTION_THEME_*` — os do seletor do topo mudam o tema do APP.
- O clique é tratado por `theme_menu::apply_section_theme_click`, chamado no `HeroScreen::apply_event`
  antes da galeria (a galeria é `widget` e não pode ler `screens` — DAG).
- A marca de estado do menu acende o tema da secção sobre a qual ele abriu.
- **A secção recebe o tema como o `theme` dela** (o plano passa-o a cada tarefa), e o cartão dela é
  repintado nesse tema por `section_cards::retheme(y0, y1, tema)` — todo cartão cujo meio cai na faixa
  da secção, subsecções incluídas.

### 6. Persistência

`~/.ph2d/sections.txt` (ao lado do `layout.txt`): `order=<ids>` e `theme.<id>=<Theme::id>`. O formato
é do editor (`section_prefs_text` / `restore_section_prefs_text`, com as duas chaves isentas no censo
HR-15 com o mecanismo); a shell só lê no arranque e grava no quadro quando `take_section_prefs_dirty`.

## §1-bis — Portão

- `nextest-impacted` **19 066** testes: verde depois de duas catracas ESCRITAS com o motivo
  (`quantas_entradas_tem_cada_painel`: carga de comandos do Inspector `81 → 78` — os pontos saíram;
  altura de abertura `822 → 831` — os vãos novos, não uma secção a nascer aberta).
- Clippy `-D warnings` a zero nos crates tocados · `fmt` limpo · censos da árvore COMBINADA **127/127**.
- **Mutação 8 de 8 a sangrar:** o vão (`section_gap_px` sem o `card_gap`), o fecho vazio, o `retheme`,
  o plano a ignorar a ordem, o plano a ignorar o tema, a pega registada com o id do cabeçalho, a lei da
  queda invertida e o clique do menu a não escrever o tema. ⚠️ O arnês mentiu uma vez: `grep -cF` conta
  LINHAS e abortou duas âncoras de várias linhas; a contagem passou a ser por ocorrência.

## §2 — Superfície de colisão (para o integrador)

- **Renomeação de ids foundational** (`ph2d-editor-core/src/ids/*`): 43 `INSP_LIVE_*_COLOR` →
  `_GRIP`. Uma linha que ainda use o nome antigo **não compila** — a cura é o nome novo, ou apagar o uso
  do ponto de cor (ele não existe mais).
- **`SectionHeader`** perdeu `color` e ganhou `grip` e `reserve_right` (struct com campos públicos: um
  literal noutra linha precisa dos dois campos novos).
- **`section_gap_px()` mudou de 8 para 12** — todo painel com cartões fica 4 px mais alto por secção. Uma
  fixtura noutra linha que dependa da altura absoluta de um corpo com cartões pode mover-se (aqui
  movi `action_verb_is_a_dropdown` de 16 para 13 acções, e o `seam_player` passou a ver a barra de
  rolagem do Inspector).
- **Inspector:** as funções `paint_core_sections`, `paint_sprite_sections`, `paint_shared_sections`,
  `paint_physics_sections`, `paint_anchor_section`, `paint_vida_sections`, `paint_optional_sections` e
  `paint_familia_*` passaram a `push_*` (empurram para o `Plano`). Uma secção NOVA de outra linha que
  chame a forma antiga não compila; a cura é `plano.push(ID, move |c, t, y| …)` no grupo da família.
- `PROJECT_SCHEMA`, registos, contratos §6: **intocados**. Zero ADR.

## §3 — Aberto

- **Só o Inspector se reordena.** Os outros painéis com secções (Vector, Pincel, Áudio…) pintam o
  cabeçalho sem pega — de propósito: uma pega que nenhum despacho lê seria um controlo morto. Levar a
  reordenação a um painel é empurrar as secções dele para um plano como o do Inspector.
- **O tema por secção só pinta secções do Inspector** (é o único que passa o tema por secção); o menu
  abre em qualquer cabeçalho vivo, mas fora do Inspector a escolha não tem leitor — mesma razão.
- A folga interior horizontal (ver §1.1).
