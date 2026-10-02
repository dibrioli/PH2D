# Handoff de integração — `line/UIUX`, 2026-10-01 — A LINHA INTEIRA (rolagem · cartões · temas e notas · fonte, peso e tamanho · os campos)

> Leitor: o **agente integrador** (só por ordem do Enio) e a próxima janela desta linha. Estado vivo:
> `CLAUDE.md` §5. Este é o documento de integração da linha; ele **supersede** os três da rodada como
> guia de fusão, e cada um continua válido para o mecanismo que descreve (§1).
>
> **Base:** merge-base `912a9652e` = `main` — **o `main` não andou** desde que a linha nasceu, logo não
> há rebase a fazer e a fusão é `--ff-only` de **25 commits**. Smoke do dono **aprovado** em cada wave.

---

## §0 — O que a linha entrega, numa frase por frente

1. **A ROLAGEM É UMA** — toda lista que rola passa pela porta `panel::scroll_area` (recorte, alturas,
   dono da barra), a roda sem lista, inércia, o D9 e a margem do fim.
2. **OS CARTÕES DE SECÇÃO** — vão entre cartões, a pega de 10 pontos que reordena, o tema por secção pelo
   botão direito; o círculo de cor e a linha do título saíram.
3. **TEMAS, NOTAS E FANTASMA** — quatro temas novos e os oito originais com cor, cores vivas de contorno, o fantasma do cartão
   arrastado, notas com pega/menu em **qualquer** painel, o menu de tema e o arrasto em todo painel de
   secções.
4. **O CONTORNO É O CARTÃO** — a linha do contorno coincide com o cartão da secção (§2.1).
5. **FONTE, PESO E TAMANHO DA INTERFACE** — *Edit ▸ Preferences… ▸ Interface font / Font weight / Font
   size*, e a Inter passa a ser **desenhada** (§2.2, §2.3).
6. **OS CAMPOS OBEDECEM AO TAMANHO** — o piso da caixa de número cresce com o texto (§2.4).

---

## §1 — Mapa: que handoff descreve que commits

| Frente | Commits | Mecanismo em |
|---|---|---|
| Rolagem (W1–W6, D9, margem do fim) | `1ef8766af` … `57c2aa3b3` (6) | [`…2026-09-29_A_ROLAGEM.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-09-29_A_ROLAGEM.md) + spec [`04`](../spec/04_a_rolagem_unica.md) |
| Cartões de secção | `3a1cbce5d` | [`…2026-09-30_OS_CARTOES.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-09-30_OS_CARTOES.md) |
| Temas, notas, fantasma, quinas, Tinted, tema em todo painel, notas em todo painel | `81fa14f97` … `59599e9e7` (11) | [`…2026-09-30_TEMAS_NOTAS_FANTASMA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-09-30_TEMAS_NOTAS_FANTASMA.md) §1–§10 |
| O contorno é o cartão | `afd9d49d8` | **este**, §2.1 |
| Fonte, peso e tamanho | `cab3af30a` | **este**, §2.2 |
| O peso Light | `28824e001` | **este**, §2.3 |
| O piso dos campos | `7d3ceb20c` | **este**, §2.4 |
| Higiene do portão (`typos`) + este handoff | os dois últimos | **este**, §4 |

⚠️ **Uma frase de um handoff anterior ENVELHECEU dentro da própria linha:** o §3 do dos CARTÕES diz
*«só o Inspector se reordena»* — a §8 do de TEMAS levou o menu de tema e o arrasto a **todo painel de
secções** (`3be7d2d65`, `750c7b52a`…). Leia o §3 dos CARTÕES como história.

---

## §2 — O que este handoff descreve pela primeira vez

### §2.1 — O contorno da secção É o cartão dela (`afd9d49d8`)

Report do dono (foto): *«a linha do contorno deve coincidir com o card da seção»*. O contorno fazia a
conta dele (descontava o vão depois da última nota) e o cartão fechava no `y` devolvido, que ainda o
tinha ⇒ as duas caixas separavam-se um `Md`, com a quina a `Radius::Md` contra o raio moderno do cartão.

- `section_cards::card_rect` — **porta única** da caixa, lida pelo cartão e pelo contorno;
  `section_cards::card_radius` — o raio do cartão do corpo aberto (clássico fora dele).
- `notes_chrome::fecha_seccao` devolve o **fundo** da última nota; o cartão fecha ali e o contorno
  traça-o (`caixa_do_contorno(inner_x, inner_w, y_before, fundo)`).
- A excepção do `notes_chrome` no gate da quina **saiu** (o raio passa pela porta).
- Gate `o_contorno_coincide_com_o_cartao_da_seccao` mede a caixa **PINTADA** (hook `cfg(test)`
  `ultimo_contorno`); 2/2 mutações — ⚠️ a 1.ª redacção refazia a conta e a mutação **sobrevivia**.

### §2.2 — Fonte, peso e tamanho da interface (`cab3af30a`)

Ordem do dono: *«pelo menos mais 2 tipos de fonts e 2 tipos de peso … assim como ajuste do tamanho das
fonts. Veja como o blender e a godot fazem»*. Os dois oráculos foram **corridos** (Blender 5.2.2 por
`bpy`, Godot 4.7.2 pelas `EditorSettings`): os dois separam fonte, peso, tamanho e nitidez em controlos
independentes ⇒ três eixos ao lado do `TextRendering`.

- **`ph2d-tokens::text_style`** (novo): `UiFont` (`Inter` · `NotoSans` · `AtkinsonHyperlegible`),
  `UiWeight`, `UiTextSize` (`Small` · `Normal` · `Large`), `UiTextStyle`. **Nenhum número escolhido:**
  os tamanhos são degraus da escala de tipo (`Sm/Base`, `1`, `Md/Base`) e o peso é um degrau da escada
  de pesos. ⛔ **O tamanho pára no `Large` por RECURSO medido:** as linhas têm `ROW_H_PX = 22` e o corpo
  a `15 px` ainda cabe; acima disso é a **escala da interface inteira**, outra obra (§6).
- **`ph2d-text`**: as três fontes vão **dentro** do binário (`crates/ph2d-text/fonts/`, SIL OFL 1.1, a
  licença ao lado de cada uma — ⚠️ **+2,2 MB** ao binário, Noto `2,0 MB` + Atkinson `0,1 MB`). O estilo
  é lido **uma vez** no `TextSystem::layout_inner`, a porta única de todo texto ⇒ medir e pintar
  concordam por construção; a chave da `layout_cache` leva a fonte. Estado no thread-local
  `ACTIVE_TEXT_STYLE` (`ph2d_text::set_active_text_style`), publicado pelo `paint_hero_screen` a partir
  de `HeroScreen.text_style`, gravado pela shell em `~/.ph2d/prefs.txt` (`ui_font`, `ui_font_weight`,
  `ui_text_size`).
- **Centragem:** quem centra pelo tamanho nominal sobe meia diferença — `paint_text::centrado`, aplicado
  em `paint_text_lines` e no texto rodado; o `text_area` mede a linha por `displayed_font_px`.
- **Settings ▸ Interface font / Font weight / Font size:** três submenus de escolha com a marca do valor
  activo, tabela **única** em `screens/hero/text_style_rows.rs` (o clique e a marca lêem-na), despacho em
  `chrome/settings_font.rs` (`mod` **gerado** por `cargo run -p ph2d-chrome-sync`).

⛔⛔ **DEFEITO ACHADO — o app NÃO desenhava a Inter.** A pilha pedia `"InterVariable"` e a família da
fonte chama-se `"Inter Variable"`; com as fontes do sistema carregadas caía no `sans-serif` — nesta
máquina a `NotoSans-Medium.ttf` instalada (`619 976` bytes). O `without_system_fonts` forçava o nome,
logo **os testes mediam a Inter e o produto desenhava a Noto**. Cura: `bundled::register_bundled` força
os nomes nos dois construtores. ⇒ **A fábrica fica Inter** (o design system; **135** construções de teste
calibradas nela); quem quiser a aparência de antes escolhe *Noto Sans*. ⚠️ **Mudança visível de
produto, aprovada no smoke do dono.**

⛔ **E a Inter de verdade expôs uma volta de UM ULP:** três rótulos de caixa MEDIDA contra o texto
faziam `texto + 2·recuo` à mão, e `(t + a) − a` fica abaixo de `t` numa fracção do domínio — a elisão
compara `<=` ⇒ `Window` → `Wind…` (barra de menus), `Flip` → `F…` (abas de tarefa, Large), `Inspector`
→ `Inspect…` (aba de encaixe, Atkinson). Os dois primeiros passam pela porta **`rect_for_label`** (que
já pagava esse ULP desde 19/09) e os recuos escritos à mão (`title_pad_x`, `tab_pad_x`) **saíram**; a aba
de encaixe confere a largura contra o orçamento da própria `face` (`slot_tabs_face::budget_of`).
Gate `nenhum_rotulo_medido_do_chrome_e_cortado` — as 3 superfícies × 3 fontes × 3 pesos × 3 tamanhos ×
3 nitidezes, régua = o que foi **PINTADO**; mutação **3 de 3** (cada cura desfeita sozinha).

⚠️ **Quatro gates re-baseados, cada um com a causa escrita ao lado do número** — eles mediam a fonte
**desta máquina** (a Noto do sistema) através do `TextSystem::new()`:
`ph2d-panel-inspector/…/every_label_this_panel_paints_fits_its_column.rs` (`220.0, 18`) ·
`ph2d-panel-painter-layers/…/cada_nome_deste_painel_cabe_na_coluna_da_seccao.rs` (`220.0, 12`) ·
`ph2d-panel-wet-tuning/…/as_duas_caixas_cabem_na_coluna.rs` (`245.0, 2`) ·
`ph2d-panel-audio-mixer/…/nenhum_nome_de_barra_corta_na_coluna.rs` (`vec!["Return"]`).
⛔ **Uma linha que funda um gate de elisão calibrado com o `TextSystem::new()` ANTES desta fusão mede
outra fonte** — se ele reprovar na árvore combinada, a cura é re-medir com a Inter (o número muda; a lei
não), **nunca** voltar a carregar a Noto.

### §2.3 — O peso Light (`28824e001`)

Pedido do dono: *«uma opção mais delicada que normal»*. O `Light` desce **exactamente** o degrau que o
`Strong` sobe (`Regular − Medium = −100`): o corpo vai de `Medium` a `Regular`, os títulos de `SemiBold`
a `Medium`, e a hierarquia fica igual. ⚠️ `UiWeight::boost` passou de **`u16` a `i16`**. As três fontes
cobrem o degrau (Inter e Noto `100..900`, Atkinson `200..800`). Gate: o Light é `< 0` e simétrico do
Strong ao bit; o gate do §2.2 já varre `UiWeight::ALL`.

### §2.4 — O piso da caixa de número obedece ao tamanho (`7d3ceb20c`)

Pedido do dono: *«veja como os campos de input numérico e dropdowns obedecem às configurações»*.
**Medido antes de mexer** (fotos em tela virtual — o Inspector das partículas e a Galeria — nas 3 fontes
× 3 pesos × 3 tamanhos): fonte e peso **já** chegavam (todo texto passa pelo `layout_inner`); dropdowns,
chips de slider e segmentados liam bem. O que não obedecia era a **caixa**: `MIN_W_PX = 72` fixos, logo no
`Large` uma linha de dois campos (`Position X / Y`) apertava cada um ao piso e a unidade saía inteira
(`-80 px` → `-80`) — a lei tudo-ou-nada da unidade a fazer o que diz, sobre um piso que não acompanhava o
texto.

- **A porta:** `widget::number_input_min_w_px()` (= `number_input::min_w_px`) — a parte do piso que é do
  **texto** cresce na razão do texto (`UiTextSize::scale`); o recuo e a coluna das setas ficam. **No
  tamanho de fábrica a resposta é o `MIN_W_PX` AO BIT** ⇒ nenhum painel se move no caminho de omissão.
  `widget::default_chip_w()` é o mesmo para o chip do slider.
- **46 sítios de produto em 17 crates** passaram a ler a porta (uma palavra cada; lista no §3). Os dois
  `const` (`NUMBER_INPUT_MIN_W_PX`, `DEFAULT_CHIP_W`) **ficam** como valor de FÁBRICA e para contextos
  `const` (o `ph2d-panel-widget-lab::study`); os testes continuam no `const` porque correm no estilo de
  fábrica, onde os dois são iguais ao bit.
- **Resultado:** no `Normal` nada muda; no `Large` a linha de dois campos que já não cabe ao piso **desce
  o 2.º campo para a linha seguinte** (a lei que a `property_box` já tinha para o painel estreito) e as
  unidades saem inteiras.
- Gate `o_piso_da_caixa_obedece_ao_tamanho_do_texto`: fábrica ao bit · `Small < Normal < Large` · a
  folga do `-1234.5` é a mesma fracção do texto em todo tamanho, **medida na fonte real**, nas 3 fontes ×
  3 pesos. Mutação **2 de 2** (piso fixo · escala ao quadrado).
- ⚠️ **Achado e NÃO curado, declarado:** o doc do `MIN_W_PX` promete *«fits 7 digits»* e o `12345.67`
  pede `52,6 px` em Inter Normal contra `44` reservados — **pré-existente** (com a Noto também não cabia:
  `51,3`); o campo recorta à direita. Ver §6.

---

## §3 — Superfície de colisão (o que a fusão pode partir)

Medida pelo `collision-surface.sh` sobre a linha inteira (`22` commits na hora da medição, `398`
ficheiros; os 3 seguintes não mexem em contadores):

| Contador | Linha | Base |
|---|---|---|
| `PROJECT_SCHEMA` (+ tripla) | `176` `(176, 13, 22)` | igual |
| `VEC_SCENE` · `FLIP` · `DOC_VERSION` · `FIELD_DOC` | `22` · `13` · `18` · `23` | iguais |
| Registos `ph2d-ecs` · `-render` · `-script` | `108` · `109` · `109` | iguais |
| Contrato §6 (`node.rs`, `tool.rs`) | intocado | — |
| ADR | nenhum | próximo livre `0176` |
| `Cargo.lock` | nenhum pacote externo novo | — |
| Tectos de LOC dos ficheiros tocados | nenhum acima | — |

**O que falha ALTO numa linha paralela (não compila):**
- dos handoffs anteriores: ids `INSP_LIVE_*_COLOR` → `_GRIP`; `SectionHeader` sem `color`, com `grip` e
  `reserve_right`; `paint_*_sections` do Inspector → `push_*` no `Plano`; `NoteData.section`,
  `WidgetEvent::CreateNote { panel, section }`, `begin/update_section_drag`, `plano::run` a devolver
  `(f32, Option<Fantasma>)`, `PlanoCtx::corre` (+`panel`), `note_ids` sem `Option`,
  `paint_note_editable_*` com `CaixaLida`; apagados `widget::showcase::state::{LAST_SECTION_TOPS_Y, …}`
  e a tabela `scrollbar_panel_for_id` (uma barra registada à mão **fica morta sob o dedo**).
- **deste:** `UiWeight::boost() -> i16`; `menu_bar::title_pad_x` e `layout_tabs::tab_pad_x` **apagados**;
  `HeroScreen` ganhou `text_style` (um literal da struct noutra linha precisa do campo); `Prefs` ganhou
  `text`.

**O que funde LIMPO e muda de comportamento (o perigo mudo):**
- `section_gap_px()` `8 → 12` (cartões) e a margem do fim `row_pitch_px()` (rolagem): uma fixtura de
  outra linha que afirme a altura **absoluta** de um corpo com cartões, ou o `panel_content_h` EXACTO de
  uma lista que transborda, lê o número novo — **é a cura, não regressão**.
- **A fonte desenhada mudou** (§2.2): todo gate de largura de texto calibrado com o `TextSystem::new()`
  noutra linha passa a medir a Inter.
- **Os 17 crates tocados pela porta do piso** (`ph2d-editor-core` · `ph2d-panel-{audio-editor,
  audio-mixer, bgremoval, color-equalization, equalize-sizes, flip, inspector, model3d, padding,
  painter-layers, physics, sculpt3d, upscale, vector, wet-tuning}`) — cada um numa palavra. ⚠️ Uma linha
  que **acrescente** um uso novo de `NUMBER_INPUT_MIN_W_PX`/`DEFAULT_CHIP_W` em produto compila e **não
  obedece** ao tamanho: a cura é a porta (`number_input_min_w_px()` / `default_chip_w()`).
- **Enums append-only:** `Theme` (+4 no fim: `8 → 12`), `HIGHLIGHTER_RGBA` (+4), `ContextMenuKind` (+3:
  `SettingsFontSubmenu`/`WeightSubmenu`/`SizeSubmenu`), ids de menu novos. ⚠️ Uma linha que tenha
  acrescentado ao mesmo enum escreve no MESMO fim — reconte a ordem e o byte do `theme_from_u8`.
- **Catracas:** DAG `widget → interaction` em **43** (a base tinha `49`; outra linha reprova em cima de 43, não de 49).

---

## §4 — Prova (sobre o `HEAD` desta linha)

- `nextest-impacted` **19 161 / 19 161** (`load ~40`). Pelo caminho, dois vermelhos curados: um `→` meu
  dentro de uma mensagem de teste (o censo de *tofu* — a Inter empacotada não tem setas) e o flake
  catalogado `the_cost_of_depth_is_linear_not_explosive` (3/3 verde sozinho).
- `clippy --workspace --all-targets -D warnings`: zero. `cargo fmt --check`: limpo.
- `censos-da-arvore-combinada.sh`: verde, controlo do filtro **12 de 12**.
- `typos` (o scan do projecto que o `ship.sh` corre): **0** — dois achados desta linha curados no fecho
  (`sais` num comentário de `ids/notes.rs`; o teste `o_dark_guarda_o_caracter_do_nome` →
  `o_dark_guarda_a_identidade_do_nome`).
- Fotos em tela virtual (`fotografa_cena.sh`, HOME isolado): as 3 fontes, Light/Normal/Strong,
  Small/Normal/Large; a barra de menus, as abas de tarefa e as de encaixe inteiras; o Inspector e a
  Galeria nos três estilos extremos. ⚠️ A placa foi partilhada com a `line/motion-value` (exclusão +
  prazo, nunca forçada).
- Mutação nesta parte: contorno 2/2 · rótulos 3/3 · piso 2/2 (os dos handoffs anteriores estão neles).

---

## §5 — A UMA linha para o `CLAUDE.md` §5 (o integrador aplica)

Na entrada **UI/UX**, a seguir à linha de 25/09 — ⚠️ **substitui** a linha que o handoff da ROLAGEM §7
propunha:

> ⭐⭐⭐ **A ROLAGEM É UMA, AS SECÇÕES SÃO CARTÕES E O TEXTO TEM FONTE, PESO E TAMANHO** (01/10,
> [handoff do integrador](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-01_A_LINHA.md)):
> toda lista rola pela porta `panel::scroll_area` (⛔ barra registada à mão fica morta sob o dedo);
> secções em cartões com pega, tema e notas em todo painel; *Edit ▸ Preferences…* escolhe fonte
> (Inter · Noto Sans · Atkinson), peso (Light · Normal · Strong) e tamanho (Small · Normal · Large),
> lidos UMA vez no `layout_inner`. ⛔⛔ **Até 01/10 o app desenhava a Noto do SISTEMA e os testes a
> Inter** (nome de família errado) — a fábrica é a Inter; o piso de um campo é a porta
> `number_input_min_w_px()`, nunca o `const`.

---

## §6 — O que fica ABERTO

- **A escala da interface inteira** (o *ui_scale* do Blender / *display_scale* da Godot): o `Large` é o
  maior tamanho de texto que as linhas de `22 px` levam. Decisão do dono quando a pedir.
- **As legendas minúsculas por cima dos botões de ferramenta** (`M…`, `S…`) saem cortadas em **toda**
  variante, a de antes incluída (pré-existente); no `Large` cortam mais.
- **O `MIN_W_PX` não cabe o `12345.67` que o doc dele promete** (§2.4) — pré-existente; o campo recorta
  à direita. Curar é decisão de produto (piso mais largo, ou menos casas decimais).
- **O `ph2d-panel-widget-lab::study`** continua no `const` (contexto `const`; é um laboratório).
- Os abertos dos handoffs anteriores que **não** fecharam: a timeline fora da porta de rolagem (por
  desenho), o painel da escultura como território da `line/sculpt3d`, e o fantasma do arrasto não
  fotografável (XTest ignorado na Xwayland virtual).
