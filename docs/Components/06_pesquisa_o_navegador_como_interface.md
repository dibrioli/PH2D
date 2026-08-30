# 06 — O navegador como INTERFACE: Godot, Blender e Unreal (2026-08-30)

> **Por que este doc existe.** A [`03_pesquisa_sistema_de_assets.md`](03_pesquisa_sistema_de_assets.md)
> respondeu *o que um asset É* — identidade, catálogos, marcar-vs-exportar, dependências — e é forte.
> Ela **não** respondeu *como o painel se comporta*. Medido em 30/08: o que lá está sobre **Godot** é a
> migração do `uid://` (identidade), **não** o dock FileSystem; e o que lá está sobre **Blender** é o
> **Asset Browser**, não o **File Browser**. O plano §F7 declara o mockup existente como
> *«inspiração, não spec»*. ⇒ **não havia spec de UI nenhuma**, e é isso que este doc traz.
>
> Pedido do Enio, 2026-08-30: *«queremos o padrão ouro … vamos buscar o verdadeiro estado da arte»*.

## 📍 Índice
- [§0 Higiene de licença](#0) · [§0-bis O muro do 403 CAIU](#0bis)
- [§1.1 Godot FileSystem dock](#11) · [§1.2 Blender File Browser](#12) ·
  [§1.3 Blender Asset Browser](#13) · [§1.4 Unreal Content Browser](#14) · [§1.5 A tabela](#15)
- [§2 As oito leis que a comparação destila](#2)
- [§3 O que a CASA já tem — medido](#3)
- [§4 O que isto põe na mesa (sem escolher)](#4) · [§5 Fontes](#5)

<a id="0"></a>
## §0 — Higiene de licença

| Alvo | Licença | O que foi lido | Marca |
|---|---|---|---|
| **Godot** | **MIT** (permissiva) | ⭐ o **cabeçalho do dock** (`editor/docks/filesystem_dock.h`) | `[SRC]` |
| **Blender** | GPL (motor) · manual **CC-BY-SA** | ⭐ **só o MANUAL**, pelo fonte `.rst` — ⛔ **nenhuma linha de código** | `[DOC]` |
| **Unreal** | EULA | só documentação pública | `[DOC]` |
| A casa | — | `crates/` + `shells/` do PH2D | `[MED]` |

⚠️ **`[SRC]` é uma marca NOVA e ela é mais forte que `[DOC]`**: o cabeçalho de um dock declara os modos
que ele **de facto tem**, enquanto um manual descreve os que alguém escolheu documentar. Ela só pode
existir para alvo **permissivo** — para o Blender continua valendo `⛔ nenhum código lido`.

<a id="0bis"></a>
## §0-bis — ⭐⭐ O muro do 403 CAIU, e isso re-qualifica a pesquisa anterior

A `03` marcou **toda** afirmação sobre o Blender como `[DOC-indireto]`, com a razão escrita:
*«`docs.blender.org` recusa fetch automatizado (403)»*. **Confirmado — ele ainda recusa.**

⭐ **Mas o manual é versionado, e o fonte dele responde:**
```
https://projects.blender.org/blender/blender-manual/raw/branch/main/manual/editors/file_browser.rst
https://projects.blender.org/blender/blender-manual/raw/branch/main/manual/editors/asset_browser.rst
```
⇒ as afirmações sobre o Blender **sobem de `[DOC-indireto]` para `[DOC]`**, e passam a ter o texto
exacto em vez de um resumo de busca. ⚠️ *Um obstáculo declarado numa pesquisa é uma nota que
envelhece como qualquer outra — a `03` disse a verdade em 21/08 e ela deixou de valer.*

<a id="11"></a>
## §1.1 — Godot: o dock FileSystem **[SRC]**

⚠️ **Ele é INDOCUMENTADO como UI.** Os `godot-docs` têm `inspector_dock.rst` e **não têm**
`filesystem_dock.rst`; a página de classe só expõe `navigate_to_path()` e sinais. ⇒ a única fonte
honesta é o cabeçalho, e ele é generoso:

```
enum DisplayMode          { DISPLAY_MODE_TREE_ONLY, DISPLAY_MODE_VSPLIT, DISPLAY_MODE_HSPLIT }
enum FileListDisplayMode  { FILE_LIST_DISPLAY_THUMBNAILS, FILE_LIST_DISPLAY_LIST }
enum Overwrite            { OVERWRITE_UNDECIDED, OVERWRITE_REPLACE, OVERWRITE_RENAME }
```

- ⭐ **TRÊS layouts, não dois:** só-árvore · dividido na **vertical** · dividido na **horizontal**. O
  dock é estreito, e a divisão horizontal é o que o torna utilizável quando ele está largo.
- ⭐⭐ **DUAS buscas e DOIS sorts, um por metade** (`tree_search_box` + `file_list_search_box`,
  `tree_button_sort` + `file_list_button_sort`). *Filtrar a árvore e filtrar a lista são perguntas
  diferentes* — e um painel com uma busca só obriga a escolher qual delas ele responde.
- **Navegação de browser:** `button_hist_prev` / `button_hist_next` + `current_path_line_edit`
  (o caminho é **editável**, não uma etiqueta).
- **`thumbnail_size_slider`** — um **slider** contínuo, não uma lista de tamanhos.
- ⭐ **Favoritos vivem DENTRO da árvore** (`favorites_item` ao lado de `resources_item`), e não num
  painel próprio. Custa zero superfície nova.
- **O menu de contexto é o mapa do que um asset É:** `DEPENDENCIES` e `OWNERS` (o grafo **nos dois
  sentidos**), `COPY_UID` (a identidade opaca é copiável à mão), `INSTANTIATE` e `INHERIT`,
  `SHOW_IN_FILESYSTEM`, `REIMPORT`, `NEW_*`.
- ⚠️ **`Overwrite` é uma política de QUEDA declarada**: largar sobre um nome que já existe pergunta
  *substituir* ou *renomear*, e `UNDECIDED` é o estado em que ninguém respondeu ainda.
- **`EXTRA_FOCUS_PATH` / `EXTRA_FOCUS_FILTER`** — o teclado tem alvos nomeados.

<a id="12"></a>
## §1.2 — Blender: o **File Browser** **[DOC]**

Quatro regiões, e a repartição é a lição:

| Região | O que carrega |
|---|---|
| **Main** | a lista/grade, com tooltip por item e preview para imagem, vídeo, fonte, blend e data-block |
| **Directory** | navegação + criação: `Previous Folder` (Backspace / Alt-←) · `Next Folder` (Shift-Backspace / Alt-→) · `Parent Directory` (**P** / Alt-↑) · `Refresh` (**R**) · `Create New Directory` (Ctrl-Shift-N) · campo de caminho com **auto-completar (Tab)** · busca (**Ctrl-F**) com **curinga** (`bl*er`) |
| **Quick Access** (esquerda) | `Bookmarks` · `System` · `Volumes` · `Recent` (com *limpar*) |
| **Execution** | `File Name` com **incrementar/decrementar**, `Cancel` (Esc), `Confirm` (Return) |

- **Display Mode:** `Vertical List` · `Horizontal List` · `Thumbnails` + `Thumbnail Size`.
- **Sort By:** `Name` · `Extension` · `Modified Date` · `Size`.
- ⭐⭐ **`Recursion Levels`: `None` · `Blend File` · `One/Two/Three Levels`** — *o navegador pode
  mostrar o conteúdo de N níveis abaixo numa vista só*. É a resposta ao problema real de *«sei o nome
  e não sei em que pasta está»*, sem inventar busca global.
- **Filtro:** um funil que **liga/desliga** + as categorias ao lado; `H` mostra escondidos.
- **Selecção:** LMB · Ctrl-LMB alterna · Shift-LMB intervalo · **arrastar faz caixa de selecção** ·
  setas navegam e **Shift mantém a selecção**.
- **Contexto:** `Delete` (Del/X), `Rename` (**F2**), e as externas (`Open`, `Edit`, `Properties`).

<a id="13"></a>
## §1.3 — Blender: o **Asset Browser** **[DOC]**

O parente mais próximo do PH2D (o asset **nasce dentro** da ferramenta), e agora com o texto exacto:

- **Quatro regiões:** biblioteca+catálogos à esquerda (**T**) · grade ao centro · **metadados à
  direita (N)** · cabeçalho.
- **Selector de biblioteca:** `All Libraries` · `Current File` · `Essentials` · as do utilizador.
  ⚠️ **Metadado só é editável em `Current File`**.
- ⭐ **Árvore de catálogos:** duplo-clique **renomeia**; **arrastar um catálogo para dentro de outro
  reparenteia**; escolher um mostra ele **e os filhos**.
- **Sort By:** `Name` · **`Asset Catalog`** (agrupa por catálogo, alfabético dentro).
- **Metadados:** `Name` · `Description` · `Author` · `License` · `Copyright` · `Source` (só-leitura) ·
  **`Tags`** · **`Preview Image`** (carregar / gerar / capturar).
- ⭐⭐ **A queda decide pelo TIPO, e não por um alvo genérico:** objeto→viewport instancia ·
  material→**sobre o objeto** · geometry nodes→**adiciona modificador** · coleção→instancia.
  *Não existe «largar um asset»; existe largar ISTO ALI.*
- **Import Method** como propriedade do gesto: `Link` · `Append` · `Pack` · *Follow Preferences*.

<a id="14"></a>
## §1.4 — Unreal: o **Content Browser** **[DOC]**

O mais maduro, e o que a maturidade acrescenta:

- **Navigation Bar:** botões de asset (`Add`, `Import`, `Save All`) · **histórico ← →** ·
  **breadcrumb** clicável.
- **Sources panel:** árvore de pastas + **`Favorites`** como painel próprio + busca de **pastas** com
  ⭐ **exclusão por prefixo `-`**.
- **Collections:** agrupamentos **transversais à pasta**, com contagem de assets.
- **Filters + Search:** filtros ligáveis, e ⭐ **`Save Search`** + **`Previous Searches`** — *a busca
  é um objeto, não um estado transitório*.
- **View style:** `Tiles` · `List` · `Columns`.

<a id="15"></a>
## §1.5 — A tabela de uma olhada

| | Godot | Blender File | Blender Asset | Unreal |
|---|---|---|---|---|
| Layout | 3 (só-árvore/V/H) | 4 regiões fixas | 4 regiões | Sources + Asset View |
| Vistas | Thumbs / List | V-List / H-List / Thumbs | H-List / Thumbs | Tiles / List / **Columns** |
| Tamanho da miniatura | **slider** | `Thumbnail Size` | `Preview Size` | opção |
| Busca | ⭐ **duas** (árvore + lista) | uma, **curinga** | uma (nome+tags) | uma + **salvável**, `-` exclui |
| Ordenação | árvore e lista, separadas | Name/Ext/Date/Size | Name / **Catálogo** | por coluna |
| Taxonomia | pastas | pastas | ⭐ **catálogos UUID** | pastas + **Collections** |
| Favoritos | **dentro da árvore** | `Bookmarks` | — | **painel próprio** |
| Histórico | ← → + caminho editável | ← → ↑ + Tab-complete | — | ← → + breadcrumb |
| Queda | política `Overwrite` | — | ⭐ **por tipo de alvo** | arrastar para o nível |
| Recursão | — | ⭐ **N níveis** | catálogo + filhos | — |
| Grafo | ⭐ `Dependencies`/`Owners` | — | — | Reference Viewer |

<a id="2"></a>
## §2 — As oito leis que a comparação destila **[INF]**

1. ⭐⭐ **A árvore e a grade são DUAS perguntas, e cada uma quer a sua busca e a sua ordenação.**
   Só o Godot as tem separadas — e ele é o único cujo dock é estreito por omissão. *Um painel com uma
   busca só obriga a decidir qual metade ela filtra, e o utilizador nunca adivinha a escolha.*
2. ⭐⭐ **A queda é resolvida pelo TIPO DO ALVO, nunca por um verbo genérico** (Blender §1.3). *«Largar
   um asset» não é uma operação; «largar uma textura NUM campo» e «largar um componente NO canvas»
   são duas.*
3. **O caminho é EDITÁVEL** (Godot, Blender). Uma etiqueta de caminho é um beco quando o utilizador
   sabe para onde quer ir.
4. ⭐ **O que é raro tem de ser alcançável sem navegar:** favoritos/bookmarks nos três. E há **duas**
   formas — dentro da árvore (Godot, barato) ou painel próprio (Unreal, visível). *A escolha é
   quanto espaço a raridade merece.*
5. ⭐⭐ **A taxonomia que não é a pasta é o que separa um navegador de ficheiros de um navegador de
   assets** — catálogos UUID (Blender) e Collections (Unreal). A `03 §2.4` já tinha escolhido o
   modelo do Blender **pelo sistema**; aqui ele reaparece **pela UI**, e é o mesmo.
6. **O grafo de dependências é um item de MENU DE CONTEXTO, nos dois sentidos** (`Dependencies` /
   `Owners`, Godot). *A `03 §2.5` chamou-lhe pré-requisito de exportar e apagar; a UI dele é uma
   linha de menu, não um painel.*
7. ⚠️ **A colisão de nome na queda é uma PERGUNTA, não uma política silenciosa** (`OVERWRITE_UNDECIDED`).
8. ⭐ **A busca pode ser um objeto** (`Save Search`, `Previous Searches` — Unreal). Barato, e é o que
   transforma um filtro numa vista.

<a id="3"></a>
## §3 — O que a CASA já tem — medido em 30/08 **[MED]**

| Peça | Estado |
|---|---|
| **Widget de árvore** | ✅ **existe** — `crates/ph2d-editor-core/src/widget/tree_view.rs` |
| Busca de painel | ✅ existe (`ph2d-panel-hierarchy/src/search.rs`) |
| Menu de contexto | ✅ `widget/context_menu.rs` |
| Slider, tabs, pill group, list_item, card | ✅ |
| **Grade de miniaturas** | ⛔ **NÃO existe** — nada de `grid`/`tile`/`thumbnail` no catálogo |
| **Arrasto ENTRE painéis** | ⛔⛔ **NÃO existe.** Zero `DragPayload`/`DragAsset`/`begin_drag` em `ph2d-editor-core` |
| Arrasto **dentro** de um painel | ✅ o reparent da Hierarquia — `WidgetEvent::HierReparent { dragged, new_parent, before, after }`, resolvido pelo dispatcher da própria Hierarquia |
| Queda no **canvas** | ⚠️ existe **só a do SISTEMA** (`winit::DroppedFile`), e ela **já projeta o ponto para o mundo** (cursor cacheado em `CursorMoved` — `app_state.rs`) |
| Preview renderizado | ✅ `render_texture_preview` + `GameRt` (offscreen com tonemap) — a `03 §2.3` já o tinha medido |

⇒ **As duas peças genuinamente novas da F7 são a GRADE e o ARRASTO QUE ATRAVESSA PAINEL.** ⚠️ O plano
resume a segunda como *«`DragAsset(AssetRef)` no action bus»*, e isso subestima: hoje **nenhum**
gesto do app começa num painel e termina noutro sítio. ⭐ O que já está pago é o lado da **queda no
canvas** — a projeção para coordenadas de mundo existe e tem dono.

<a id="4"></a>
## §4 — O que isto põe na mesa para o PH2D (sem escolher) **[INF]**

1. **O dock estreito do Godot é o nosso caso** (o painel do PH2D vive numa coluna), ⇒ os três layouts
   e as **duas buscas** são a referência mais próxima — não o Content Browser, que assume largura.
2. **Catálogos por UUID já estão decididos** (ADR-0165 / `03 §2.4`). A UI deles é a **árvore de
   catálogos** do Blender: renomear por duplo-clique, reparentear por arrasto, e o filho conta.
3. ⚠️ **A queda por tipo de alvo (lei 2) decide a forma do payload**: ele tem de dizer **o que é**
   para o alvo poder recusar. Um payload opaco obriga cada alvo a adivinhar.
4. ⚠️ **`Recursion Levels` é a feature barata que ninguém copia** e que resolve *«sei o nome, não sei
   a pasta»* sem busca global — e o nosso índice (F6) já responde a isso por construção.
5. **O grafo (F6 `deps`) tem UI de uma linha de menu**, não de um painel — e isso muda o preço dele.
6. ⛔ **O que fica FORA por decisão já registada** (ADR-0165): similaridade por ML · colheita
   automática de órfãos · escopos partilhados. **Cor dominante entra** (histograma OKLab).

<a id="5"></a>
## §5 — Fontes

- **Godot** `[SRC]` — [`editor/docks/filesystem_dock.h`](https://raw.githubusercontent.com/godotengine/godot/master/editor/docks/filesystem_dock.h) ·
  [`class_filesystemdock`](https://docs.godotengine.org/en/stable/classes/class_filesystemdock.html) ·
  [`godot-docs/tutorials/editor/`](https://github.com/godotengine/godot-docs/tree/master/tutorials/editor) (a ausência de `filesystem_dock.rst` é o achado)
- **Blender** `[DOC]` — [`manual/editors/file_browser.rst`](https://projects.blender.org/blender/blender-manual/raw/branch/main/manual/editors/file_browser.rst) ·
  [`manual/editors/asset_browser.rst`](https://projects.blender.org/blender/blender-manual/raw/branch/main/manual/editors/asset_browser.rst)
  (⚠️ `docs.blender.org` continua a devolver **403**; o fonte do manual não)
- **Unreal** `[DOC]` — [Content Browser Interface](https://dev.epicgames.com/documentation/en-us/unreal-engine/content-browser-interface-in-unreal-engine) ·
  [Content Browser](https://dev.epicgames.com/documentation/en-us/unreal-engine/content-browser-in-unreal-engine)
- **A casa** `[MED]` — varredura de `crates/ph2d-editor-core/src/widget/`, `crates/ph2d-panel-hierarchy/src/`,
  `shells/desktop/src/` em 2026-08-30.
