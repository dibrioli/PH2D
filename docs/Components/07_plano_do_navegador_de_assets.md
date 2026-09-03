# 07 — Plano da etapa: **o navegador de assets** (F6 + F7)

> Pesquisa do sistema: [`03`](03_pesquisa_sistema_de_assets.md) · da **interface**:
> [`06`](06_pesquisa_o_navegador_como_interface.md) · decisão:
> [ADR-0165](../architecture/decisions/0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md) ·
> fases: [`05 §F6/§F7`](05_plano_de_implementacao.md).
>
> ⚠️ **Regra de trabalho nova** (Enio, 2026-08-30): *«não em micro passos; cada etapa deve ao fim ter
> um smoke; se a etapa tiver qualquer complexidade, deve ser auditada antes de sugerir o smoke.»*

## 📍 Índice
[§0 A régua](#0) · [§1 O corte em DUAS etapas, e por quê](#1) · [§2 O que o navegador mostra](#2) ·
[§3 As decisões de desenho, com a lei que as escolheu](#3) · [§4 Etapa A](#4) · [§5 Etapa B](#5) ·
[§6 Fora, com motivo](#6) · [§7 A auditoria antes do smoke](#7) · [§8 Riscos MEDIDOS](#8) ·
[§9 O que a etapa A entregou](#9)

<a id="0"></a>
## §0 — A régua da etapa

**Uma etapa acaba quando existe uma frase «faça X e veja Y».** Nem antes nem depois.

<a id="1"></a>
## §1 — O corte em DUAS etapas, e por quê

⛔⛔ **A regra nova PROÍBE a F6 como fatia própria.** A F6 é o índice: headless, sem um pixel. O
critério dela no `05` é *«teste headless popula 10 k assets e `query`/`deps`/`preview` respondem»* —
não há frase «faça X e veja Y». ⇒ **F6 e F7 são uma etapa só.**

E essa etapa, inteira, é grande demais para uma auditoria útil. ⇒ ela parte em **duas**, e o corte é
pelo **gesto**, não pelo diff:

| | Etapa | Acaba com o Enio a… |
|---|---|---|
| **A** | **Achar e usar** | abrir *Assets*, ver os componentes dele, **buscar**, e **usar um** sem arrastar |
| **B** | **Arrastar** | **arrastar** um componente para a tela e uma textura para um campo |

⭐ **A ordem é obrigatória e não é conveniência:** a etapa B constrói o **primeiro arrasto que
atravessa painel deste app** ([`06 §3`](06_pesquisa_o_navegador_como_interface.md)), e um mecanismo
novo sobre um painel que ainda não existe não tem onde ser medido. ⚠️ **A etapa A tem de ser útil
sozinha** — senão ela é um micro-passo com outro nome; é por isso que ela inclui o verbo de **usar**
(§4, A7), e não só o de ver.

<a id="2"></a>
## §2 — O que o navegador mostra: **DUAS fontes, e elas não se parecem** `[MED 30/08]`

| Fonte | O que é | Onde vive | Identidade |
|---|---|---|---|
| **Componentes** | uma **subárvore marcada** (`MasterRoot`) | **no MUNDO** (ECS), dentro do `.ph2dproj` | `StableId` |
| **Texturas** | bytes decodificados | **`AssetDb`**, endereçado por **conteúdo** (blake3) | `AssetId` / `LogicalTextureId` |

⭐⭐ **Isto é a razão de o índice existir, e o `05 §F6` não o diz.** Um componente **não é um
ficheiro** — ele é o *«Mark as Asset»* do Blender, aplicado a uma subárvore
([`03 §1.5`](03_pesquisa_sistema_de_assets.md): *«marcar em vez de exportar»*). Uma textura **é**
conteúdo. Perguntar *«que assets existem?»* hoje exige duas travessias diferentes, e nenhum sítio as
junta. ⇒ **o índice é a junção**, e não uma cache de disco.

⚠️ **Consequência que decide a arquitectura:** ⛔ **não há árvore de PASTAS para navegar.** O
`res://` do Godot e o sistema de ficheiros do Blender **não existem aqui**. A árvore da esquerda é a
de **catálogos** (Blender), e essa decisão já estava tomada pelo ADR-0165 — o que a `06` acrescenta é
que ela também é a decisão **de UI**, e não só de armazenamento.

<a id="3"></a>
## §3 — As decisões de desenho, cada uma com a lei que a escolheu

| # | Decisão | A lei ([`06 §2`](06_pesquisa_o_navegador_como_interface.md)) |
|---|---|---|
| D1 | **Duas buscas**: uma filtra a árvore de catálogos, outra a grade | Lei 1 — o dock do Godot é o único com dock **estreito**, como o nosso, e é o único que as separa |
| D2 | ~~**Layout dividido na vertical** (catálogos em cima, grade em baixo)~~ ⇒ **coluna à ESQUERDA**, colapsável | ⛔ **A premissa dissolveu — ver §10** |
| D3 | **Slider** de tamanho de miniatura, não presets | `thumbnail_size_slider` `[SRC]` |
| D4 | **Favoritos DENTRO da árvore**, como uma raiz irmã | Lei 4, ramo barato (Godot) — ⛔ painel próprio (Unreal) custa superfície que um dock estreito não tem |
| D5 | **Catálogo: duplo-clique renomeia · arrastar reparenteia · escolher mostra ele E os filhos** | Blender Asset Browser `[DOC]` |
| D6 | **Ordenação por metade** (árvore: nome · grade: nome/tipo/recente) | `tree_button_sort` + `file_list_button_sort` `[SRC]` |
| D7 | **A queda decide pelo TIPO DO ALVO** — componente→canvas instancia; textura→campo preenche; ⛔ nunca um «largar asset» genérico | Lei 2 — a lei mais forte da pesquisa |
| D8 | **Preview fora do quadro, cache limitada, só o visível** | [`03 §2.3`](03_pesquisa_sistema_de_assets.md), já medida |
| D9 | **`Dependencies` / `Owners` no menu de contexto**, nos dois sentidos — não um painel | Lei 6 |
| D10 | O índice actualiza por **change tick**, ⛔ nunca por varredura ao mover | [`03 §3.8`](03_pesquisa_sistema_de_assets.md) — a patologia medida do Godot |

⏳ **Adiadas com o nome escrito** (não são «esquecidas»): busca salvável (lei 8) · `Recursion Levels`
(lei do `06 §4.4`) · política `Overwrite` na queda (lei 7 — ela só tem sujeito quando existir
importar-para-catálogo).

<a id="4"></a>
## §4 — ETAPA A — **achar e usar**

| Wave | O quê | Porque não é micro-passo |
|---|---|---|
| **A1** | `crates/ph2d-asset-index/` — crate **folha**: `AssetEntry { id, kind, name, catalog, tags, preview }`, `query(filtro) -> Vec<AssetEntry>`, `deps`/`owners`. **Sem UI, sem I/O.** | é o vocabulário de que tudo o resto depende |
| **A2** | **A junção das duas fontes** (§2): mestres do mundo + texturas do `AssetDb`, por **change tick** (D10) | é o *conteúdo* do painel; sem ela A4 mostra uma lista vazia |
| **A3** | **Catálogos por UUID** em ficheiro de texto, com a árvore (D5) + a raiz `Favoritos` (D4) | a taxonomia é o que separa isto de uma lista |
| **A4** | **`crates/ph2d-panel-asset-browser/`** — os 5 sítios de registo + a **grade de miniaturas** (⚠️ **widget NOVO**, não existe no catálogo — `06 §3`) + o split (D2) | o painel |
| **A5** | **As duas buscas** (D1), as duas ordenações (D6), o slider (D3) | sem isto o painel deixa de servir ao passar de ~20 assets |
| **A6** | **Previews fora do quadro** (D8), via `GameRt`; cache limitada; o que não coube desenha um marcador — ⛔ **nunca em silêncio** | é o que o torna legível |
| **A7** | ⭐ **O verbo de USAR sem arrastar**: duplo-clique / menu de contexto → *Instantiate* (+ `Dependencies`/`Owners`, D9) | **é o que torna a etapa A útil sozinha** |

### O smoke da etapa A
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && cargo run -p ph2d-host-desktop --release
```
1. Faça dois ou três componentes (botão direito na linha → **Make Component**).
2. Abra **Assets** na barra de cima.
3. **O que tem de acontecer:** os seus componentes aparecem com **miniatura**, e as imagens que você
   importou aparecem ao lado.
4. Escreva parte de um nome na busca de baixo → a grade filtra. Crie um catálogo, arraste um
   componente para dentro dele, e escolha o catálogo → só ele aparece.
5. **Duplo-clique num componente** → ele entra na cena.
6. **Deu errado se:** a janela travar enquanto as miniaturas aparecem · um componente que você
   acabou de criar não aparecer · a busca de cima filtrar a grade (ela é dos catálogos).

<a id="5"></a>
## §5 — ETAPA B — **arrastar**

| Wave | O quê |
|---|---|
| **B1** | ⭐⭐ **O payload que atravessa painel** — o mecanismo NOVO (`06 §3`). Ele **diz o que é** (D7), para o alvo poder **recusar**; ⛔ um payload opaco obriga cada alvo a adivinhar |
| **B2** | **Queda no canvas** → instancia **no ponto**. ⭐ Reusa a projecção que o `DroppedFile` do sistema já tem (`06 §3`) — ⛔ não escrever uma segunda |
| **B3** | **Queda num campo do Inspector** → preenche (textura) |
| **B4** | **A voz do arrasto**: fantasma sob o cursor · alvo válido realça · **alvo inválido recusa à vista**, nunca em silêncio |

### O smoke da etapa B
1. Arraste um componente da grade **para a tela** → ele nasce **onde você soltou**.
2. Arraste uma imagem da grade **para o campo de textura** de um objeto no painel da direita → ele passa a usá-la.
3. Arraste um componente **para um campo de número** → ele **recusa à vista** (o campo não acende).
4. **Deu errado se:** nascer no meio da tela em vez de onde você soltou · o alvo inválido aceitar · o fantasma ficar preso depois de soltar.

<a id="6"></a>
## §6 — Fora desta etapa, com o motivo

⛔ Similaridade por ML · colheita automática de órfãos · escopos partilhados (ADR-0165, decidido) ·
⛔ **importar ficheiro para dentro de um catálogo** (é a porta do `import_router`, outra etapa — e é
ela que dá sujeito à política `Overwrite`) · ⏳ busca salvável · ⏳ `Recursion Levels` ·
⏳ cor dominante (OKLab) — **entra no modelo em A1**, sem UI, porque acrescentá-la depois mexe no
registo do índice.

<a id="7"></a>
## §7 — A auditoria ANTES de cada smoke `[regra do Enio, 30/08]`

As duas etapas são complexas ⇒ **as duas são auditadas antes de eu sugerir o smoke**. Lentes:

1. **Correcção** — dois sítios que devem concordar sobre um facto e discordam. Alvo nomeado: *o
   índice e o mundo* (um componente apagado ainda no índice) e *o índice e o `AssetDb`*.
2. **Costura de UI** — todo widget é **pintado, populado E clicável**, e a sequência leva a algum
   lugar. ⚠️ Alvo nomeado: a grade é **guiada por tabela de ids**, que é exactamente o ponto cego que
   o `table_driven_chips_are_registered_too` passou a cobrir em 27/08 — **o gate existe agora e tem
   de ver estes chips**.
3. **A terceira pergunta do knob morto** (memória): *o painel escreve onde · quem lê · o leitor
   DECIDE?* Alvo: o filtro de catálogo — ele chega à query ou é descartado a jusante?

<a id="8"></a>
## §8 — Riscos MEDIDOS (não suposições)

1. ⛔⛔ **Nenhum arrasto atravessa painel hoje** (`06 §3`). É o maior desconhecido, e é por isso que
   ele está **sozinho na etapa B** — misturá-lo com o painel faria um vermelho ambíguo.
2. ⚠️ **A grade de miniaturas é widget NOVO** — o catálogo tem `tree_view.rs` e **nada** de grade.
   Ela nasce em `ph2d-editor-core/src/widget/`, não dentro do painel: um segundo painel com grade
   copiaria o desenho.
3. ⚠️ **Preview de componente é RENDER**, não decodificação. O `GameRt` existe e é offscreen; o
   risco é o **relógio**, e a lei D8 é o que o contém. ⇒ **a wave A6 mede antes de escolher a
   cache**, e o número vai no doc ao lado do teto (§0.0 do `CLAUDE.md`).
4. ⚠️ **`vello` 0.10 tem atlas PERSISTENTE**: quem recozinha pixels **tem** de chamar
   `mark_texture_dirty`, senão a imagem **congela**; e o que não cabe **não é desenhado, em
   silêncio** (`STACK_VERSOES.md`). ⇒ uma grade de miniaturas é exactamente a superfície que isto
   morde, e a A6 tem de o gatear.
5. ⚠️ **O painel novo entra em 5 sítios de registo** (memória `reference_topic_panel_registration`),
   e o 6.º — o `populate` — é o que mata sob o dedo.

<a id="9"></a>
## §9 — O que a ETAPA A **entregou**, e o que ficou (2026-08-30)

> ⚠️ Esta secção é o registo da execução, escrito ao fechar. O plano acima **não** foi reescrito —
> comparar os dois é como se vê o que a implementação refutou.

### §9.1 — Entregue

| Wave | Estado | Onde |
|---|---|---|
| **A1** — o vocabulário | ✅ | [`crates/ph2d-asset-index/`](../../crates/ph2d-asset-index/) |
| **A2** — a junção das duas fontes | ✅ | [`asset_index_build.rs`](../../shells/desktop/src/asset_index_build.rs) |
| **A3** — catálogos | ⏳ **não** | ver §9.3 |
| **A4** — o painel + a grade | ✅ | [`crates/ph2d-panel-asset-browser/`](../../crates/ph2d-panel-asset-browser/) |
| **A5** — busca · ordenações · slider | ✅ (uma busca, não duas — §9.3) | idem |
| **A6** — miniaturas a sério | ⏳ **não**, e a cor está no lugar delas | ver §9.3 |
| **A7** — o verbo de USAR | ✅ | `EditorAction::AssetInstantiate` |

### §9.2 — O que a implementação **achou**, e o plano não previa

⛔⛔ **O pill `Assets` da barra de cima JÁ EXISTIA e estava MORTO.** Ele é pintado
(`cluster_painter.rs`), registado (`topbar::populate`), tem tooltip (*«Asset library»*), tem nome de
chip — e **nenhum `apply_event` do repositório ramificava nele**. É a espécie exacta que o
[`CLAUDE.md` §5.0](../../CLAUDE.md) descreve, e a terceira pergunta (*o leitor DECIDE?*) respondia
**não**.

⚠️ **E os IRMÃOS dele continuam mortos:** `TOPBAR_RIGHT_LAYERS` e `TOPBAR_RIGHT_SCRIPT` têm a mesma
forma e **não** foram curados aqui — curar um chip cuja feature não existe seria construir o
consumidor de um widget que não tem o que consumir. Ficam **nomeados**, que é o que faltava.

⚠️ **A cor do cartão é a média ponderada por ALFA, e a primeira versão estava errada.** Uma sprite
recortada é quase toda transparente: a média crua dela é a cor do **nada** (preto), não a do
desenho. Gate: `the_swatch_of_a_cut_out_sprite_is_the_colour_of_the_drawing_not_of_the_hole`.

⚠️ **«Vazio» e «por publicar» desenham mensagens diferentes.** Um índice que ninguém encheu
lê-se como *«não tenho assets»* — é o balde vazio da memória
[`a_bucket_nobody_fills_reads_as_perfect`](../../project-memory/feedback_a_bucket_nobody_fills_reads_as_perfect.md).

⚠️ **A grade recorta nos DOIS canais** (`scene.push_clip` **e** `HitIndex::push_clip`). Só o
primeiro é a doença que o painel do Motion pagou: um cartão rolado para fora do corpo continua
**clicável**, e o artista instancia o que não vê.

⚠️ **`AssetInstantiate` não podia reutilizar `HierInstantiate`, e a razão é o SUJEITO.** Aquela chega
com uma `row` da Hierarquia; uma receita está **escondida** dela por construção, logo não tem `row`.
A variante nova endereça pelo `StableId` e chama a **mesma** `instance_verbs::drain`.

### §9.2-bis — O que a **AUDITORIA antes do smoke** achou (regra do Enio, 30/08)

⚠️ Os quatro passavam por **todos** os 12 gates de costura que a linha já tinha. *A costura que
faltava não era o clique — era o **tipo do registo** e o **espaço dos ids**.*

| # | O defeito | Por que nenhum gate o via |
|---|---|---|
| 1 | ⛔⛔ A faixa de arrasto e a alça registadas como **`Button`** ⇒ **o painel abria e não se movia nem redimensionava** | o despacho de um painel flutuante não passa pelo `Click`: ele lê `InteractiveState::BlenderHit { parent, kind }` no `pointer_down` |
| 2 | ⛔⛔ O polegar da barra **não registado** ⇒ **inagarrável** — e o comentário que eu escrevi dizia que a ausência era a lei | sem entrada no store o `is_focusable` é falso e o `pointer_down` nunca semeia o arrasto |
| 3 | ⛔ As células com **`register`** (que SUBSTITUI) em vez de `register_if_absent` | o `state: Normal` de cada quadro apagaria o `Pressed` do anterior |
| 4 | ⛔⛔ O gémeo de **runtime** do `hash_node_id` com o **primo errado** (`0x1000_0000_01b3`) ⇒ os ids das células caíam noutro espaço e o hit-test **nunca os resolveria** | era a **terceira cópia** da mesma lei no repositório; a cura foi promovê-la a **porta** (`ph2d_tool_registry::hash_node_id_runtime`), e a cópia do `flip.rs` colapsou nela |

⇒ **+3 gates de censo de REGISTO** (o tipo, não o clique), com prova de mutação: repor o `Button` na
faixa de arrasto faz o gate novo sangrar.

### §9.3 — O que ficou, com o motivo

- ⏳ **A3, os catálogos.** É uma etapa própria com o próprio smoke (*criar um catálogo, arrastar um
  componente para dentro, escolher e ver só ele*) — e é a que traz a **segunda busca** (D1), que sem
  a árvore não tem sujeito. O modelo já a espera: `AssetEntry::catalog` existe, e o filtro já é
  honrado pela consulta com gate (`the_catalog_filter_actually_narrows_the_result`).
- ⏳ **A6, as miniaturas.** Um componente precisa de ser **RENDERIZADO** (offscreen, `GameRt`), e o
  `vello` 0.10 tem atlas persistente — quem recozinha pixels tem de marcar a textura suja, senão a
  imagem **congela em silêncio**. É uma wave com medição própria, e o cartão colorido é o que a
  substitui **com informação** até lá.
- ✅ **`Dependencies` / `Owners` no menu de contexto** (D9) — **feito em 2026-09-02**, ver §13.
- ⏳ **Imagens de 16 bits ficam com a cor neutra** — a média delas pede a descodificação inteira, e
  pagá-la por um quadrado de 24 px é o oposto do que a cache existe para fazer. Declarado no `_` do
  `swatch_for`.

<a id="10"></a>
## §10 — ⛔ A decisão **D2 foi revertida, com a medição ao lado** (2026-08-30)

> ⚠️ Esta seção existe porque o §0.0 do `CLAUDE.md` a exige: *«quem move o número que tornava algo
> inalcançável tem de reconferir a nota»*. A D2 escolheu o **split vertical** com um motivo
> explícito — *«`HSPLIT` só faz sentido com largura que não temos»* —, e essa premissa era sobre um
> **dock estreito**. O navegador não nasceu num dock: ele nasceu **flutuante, mais largo que alto e
> redimensionável** (`default_rect` = 420×520, mínimo 300 — [`paint.rs`](../../crates/ph2d-panel-asset-browser/src/paint.rs)).

### A medição

Com `pad = 8`, `gap = 6` e a fórmula que a grade já usa
(`cols = ⌊(inner_w + gap) / (cell + gap)⌋`):

| largura do painel | coluna | `inner_w` | colunas de cartão @ `cell = 84` |
|---|---|---|---|
| **420** (omissão) | 0 | 404 | 4 |
| **420** | **140** | 264 | **3** |
| 300 (piso do `default_rect`) | 140 | 144 | 1 |
| 220 (`PANEL_MIN_W_PX`) | 140 | 64 | ⛔ **1, com o cartão CORTADO** |

⇒ à largura de omissão a coluna custa **uma** coluna de cartões, e não a viabilidade do painel.

### ⛔ E o `cols` tem um defeito que a coluna torna alcançável

`(...).floor().max(1.0)` nunca devolve zero: ele devolve **um**, e um cartão de 84–160 px num vão
de 64 px **não reflui — é cortado pelo recorte**. O defeito já lá estava; a coluna é que o põe ao
alcance de um redimensionamento normal.

⇒ **a largura da coluna é DERIVADA, não escolhida**: ela é o mínimo entre a largura nominal e o que
sobra depois de a grade guardar **um cartão inteiro**, e colapsa a zero quando nem isso cabe.

### O que fica da D2

⭐ O **botão para só-grade** fica, e é ele que colapsa a coluna — ele deixou de ser um *display
mode* e passou a ser o interruptor da coluna. ⛔ O split **vertical** não fica: uma árvore de
catálogos é uma lista **vertical**, e uma faixa horizontal desperdiça a largura e mata a altura
que a lista precisa.

## §11 — ETAPA D — **arrumar**: o que a coluna de catálogos entregou (2026-08-30)

### §11.1 — O que existe agora

| Gesto | Onde | Lei |
|---|---|---|
| Abrir/fechar a coluna | botão *só-grade* no cabeçalho | é **vista** — não atravessa o barramento |
| `+ Catalog` | topo da coluna, **fora do recorte da lista** | nasce dentro do escolhido; o nome é **gerado e único** (`Catalog`, `Catalog 2`, …) |
| Escolher | clique na linha | **vista**; a grade filtra pelo escopo expandido |
| Arrastar um cartão para a linha | queda | `CatalogVerb::Assign` |
| **Renomear** | botão direito → *Rename…* | campo em linha, **o nome inteiro seleccionado** |
| **Apagar** | botão direito → *Delete* | apaga o catálogo **e os descendentes**; ⛔ nunca um asset |

`All` e `Unassigned` são **linhas**, não estados escondidos num chip — e o menu sobre elas
**não faz nada** (gate `the_menu_over_a_fixed_row_does_nothing`): elas não têm nome para mudar
nem gaveta para apagar.

### §11.2 — ⭐⭐⭐ O defeito que só o SMOKE viu: o campo abria a ACRESCENTAR

A 1.ª versão copiou o molde do `marker_rename` da Timeline, que semeia o campo com o cursor **no
fim e sem selecção**. Medido em produto pela cena `=78`:

| Passo | Esperado | Medido |
|---|---|---|
| abrir *Rename…* sobre `Catalog`, escrever `Heroes`, Enter | `Heroes` | **`CatalogHeroes`** |

⚠️ **Nenhum dos 24 gates de costura o via** — eles medem o que o `Submit` **manda**, e a semente é
escrita pelo **pintor**. ⇒ a lei mudou-se para uma função com nome (`catalog_rename::seed_state`)
e ganhou gate; a cura é `selection_anchor: Some(0)` com o cursor no fim, que é o que o Finder, o
Blender e o Unity fazem. *Um campo aberto pelo item «Rename…» já sabe que o artista quer OUTRO
nome; acrescentar continua a uma seta de distância, substituir passa a estar a zero.*

⛔ **E o molde da Timeline NÃO foi «corrigido» de passagem** — lá o campo tem dois modos (rótulo ·
sinal) e a decisão é dele; mexer-lhe daqui seria mudar produto de outra linha sem smoke.

### §11.2-bis — ⛔⛔ E o campo tinha DUAS portas por onde ficar focado e invisível

Um painel tem **duas** maneiras de desaparecer, e a limpeza estava escrita em zero delas:

| Porta | O que o ramo já largava | O que faltava |
|---|---|---|
| a coluna **colapsa** (`col_w == 0`: botão *só-grade*, ou painel estreitado) | a região de rolagem, as linhas pintadas | **o campo** |
| o **painel fecha** (`!panel_visible`) | o rect do painel, as células, a lista pintada | **o campo** |

Em qualquer das duas o campo deixava de ser pintado e registado, **e o `WidgetStore` continuava
com o foco nele** — a partir daí escrever no app não fazia nada em lado nenhum, sem nada na tela a
dizer porquê. ⚠️ *Uma limpeza escrita num sítio só ainda não é uma limpeza.*

⚠️ `catalog_rename::abandon` larga o foco **só se ele for nosso** — pisar o foco de outro widget
seria trocar um defeito por outro, e há gate de controlo sobre isso.

⭐ O gate da segunda porta usa o `paint_hidden` do testkit, que existe precisamente porque *nenhum
gate deste repo alcançava o ramo escondido de um painel*.

### §11.3 — As leis que a etapa pagou

- **A escada de ids lê-se nos dois sentidos, por UMA porta.** `catalog_row_index` estava escrita
  **três** vezes (o `event.rs` do painel, o `catalog_row_pick` do estado, e a 4.ª cópia ia nascer
  no despachante do botão direito) — hoje é `ph2d_editor_core::ids::catalog_row_index`, com gate
  de ida-e-volta. *Uma lei escrita em N sítios ainda não é uma lei — só uma PORTA é.*
- **O sujeito do menu resolve-se pelo CENSO DO QUADRO, nunca pela escada.** O id é posicional; o
  menu abre no `Down` e é despachado num `Click` posterior, e a lista pode ter mudado no meio.
- **Apagar o catálogo ESCOLHIDO devolve a grade a `All`.** Sem isso ela filtraria por uma gaveta
  que já não existe: zero cartões e nada na tela a explicar porquê.
- **O `Rename…` não manda nada** — ele abre o campo. O nome atravessa o barramento no
  `Submit`/`Blur`, e **um nome igual ao actual não levanta acção** (sujaria o projecto por nada).
- **Quem recusa um nome ilegal é o DRENO, e em voz alta** (vazio ou com `/`). Duplicar a regra no
  painel daria duas respostas à mesma pergunta.
- **O corte do `event.rs` foi por RESPONSABILIDADE.** Os seis gestos da taxonomia levaram o
  `apply_event` a 222 LOC contra o tecto de 200 ⇒ módulo irmão `event_catalog.rs`, ⛔ nunca uma
  entrada de tolerância. A guarda do painel fechado fica no chamador, e por isso não se repete.

### §11.4 — ⏳ O que fica aberto nesta etapa

- **A segunda busca (D1)** — a busca da grade fala de assets e **não filtra a coluna**, de
  propósito. São duas perguntas.
- **Mover um catálogo** (arrastar linha para linha) — hoje a hierarquia só se autora pelo `+`
  dentro do escolhido. Renomear com `/` é **recusado** justamente para não esconder um *mover*
  dentro de um *renomear*.
- **O undo da taxonomia** — ela é do **projecto** e não do `ProjectState`, então estes verbos não
  produzem passo de undo; o que eles compram é o projecto marcado como sujo. Dívida declarada no
  `project_catalogs`.

### §11.5 — A auditoria de 2 lentes da etapa D (2026-08-30) — **13 achados, todos fechados**

#### O que ela achou no PRODUTO

| # | Achado | Mecanismo | Cura |
|---|---|---|---|
| **A** | ⛔⛔ o polegar da barra da coluna era **pintado num rect e agarrado noutro, 30 px acima** | o pintor recebia `col`, o `thumb_rect` do chamador recebia `list_rect`; e a fronteira do `is_needed` era perguntada **duas vezes com denominadores diferentes** ⇒ numa janela de 30 px o `register` corria sobre uma barra que ninguém desenhara (**polegar invisível e agarrável**) | `paint_scrollbar` passa a **devolver o polegar que desenhou**; o chamador regista **esse**. Uma banda, uma altura, uma porta |
| **B** | a **terceira** porta do campo órfão | o ramo *«o catálogo desapareceu»* escrevia `renaming = None` à mão e não largava o foco — as outras duas já chamavam o `abandon` | `abandon` nas três |
| **D** | `catalog_row_index` punha **256 `format!` + 256 FNV no caminho de TODO botão direito do app** | o `pointer_down_menus` avalia-a incondicionalmente no ramo `Secondary`, logo canvas/hierarquia/timeline pagavam o MISS. ⚠️ **o gate de zero-alocação era estruturalmente cego**: só despacha `Move` com o botão **primário** | escada assada uma vez (`LazyLock`) — tira o `format!` também do laço que pinta |

⚠️ **A é PRÉ-EXISTENTE** (nasceu na wave A3) e o irmão que serve de controlo está no **mesmo par de
ficheiros** e fazia o certo. *Uma lei escrita duas vezes acerta numa e falha na outra.*

#### O que ela achou nos GATES — três passavam por coincidência

| # | Gate | Porque era verde |
|---|---|---|
| **C** | `abandoning_the_rename_does_not_steal_someone_elses_focus` | a fixtura punha `renaming = None`, e o `.take()` **curto-circuita antes** de a cláusula do foco ser avaliada ⇒ ele controlava um `set_focus(None)` incondicional, não a cláusula que diz controlar |
| **J** | `the_menu_over_a_fixed_row_does_nothing` | o oráculo é «nada aconteceu» ⇒ passava com o censo do quadro **vazio**; faltava o controlo positivo |
| **K** | `renaming_to_the_same_name_dispatches_nothing` | `commit` devolve `None` por **duas** causas (nome igual · catálogo não encontrado) e o gate não as separava ⇒ a cláusula era creditada por uma ausência |

Mais **H** (o elo semente→rótulo sem gate: `seed_state("")` sobrevivia a tudo — *a metade exacta que
o smoke apanhara em produto*), **I** (o Esc é o par `Cancel`+`Blur` e o gate mandava só um) e **M**
(a escada não afirmava que os ids do MENU não são linhas — e a arm da escada vem **antes** da do
menu, logo uma colisão engoliria o `Rename…` em silêncio).

#### ⭐⭐ **E** — o gate de paridade **nem lia o ficheiro**

O `read_paint_sources` só varre ficheiros cujo nome contém `paint` ou que vivem sob `sections/`, e o
módulo chamava-se `catalog_rename.rs` ⇒ o `register` dele era **invisível**. ⛔ **A regra não foi
alargada** — alargá-la arrasta quatro ids de painéis de **outras linhas** (o doc do próprio gate já
os NOMEIA, entre eles o `TIMELINE_CLIP_RENAME_INPUT`, o gémeo exacto deste campo), e essa decisão é
dos donos deles. ⇒ o módulo foi **renomeado para dentro do alcance da regra**
(`paint_catalog_rename.rs`) — e o gate, mal passou a vê-lo, acusou logo o que faltava: o campo não
estava no `populate.rs`, logo **não era focável de nascença**. *Entrar no alcance de um gate é a
forma barata de descobrir o que ele já sabia.*

#### O que ela mediu e **ILIBOU** (não confunda com «não olhou»)

- **O `consume_last_context_menu` antes de conhecer o sujeito é inócuo** — `last_context_menu` é um
  slot único que o `close_context_menu` **sobrescreve**, e os dois ids só nascem desta arm ⇒
  consumi-lo sobre uma linha fixa descarta **o pedido dele próprio**.
- **A taxonomia publicada não está atrasada no instante que importa** — a publicação corre **antes**
  do dreno dos verbos no mesmo quadro, e a semente e o `current` saem da **mesma** leitura.
- **A ordem das arms está certa**: a guarda `panel_visible` vem antes do `other =>` que delega os
  gestos de catálogo. ⚠️ Efeito colateral que vale saber: o `DoubleClick` é apanhado **inteiro** pela
  arm dos cartões, então *«duplo-clique renomeia»* (o gesto do Finder) exigiria abrir aquela arm.
- **O `rename_y` colhido ANTES do `continue` está certo** e é melhor do que o doc prometia — rolar
  para baixo cola o campo ao bordo inferior em vez de o atirar para o topo. **O doc é que foi
  corrigido.**

### §11.6 — ⭐⭐⭐ **A BIBLIOTECA DESFAZ** (Enio, 2026-08-30: *«deveria ter undo/redo no painel inclusive em del»*)

#### A nota que a mantinha fora estava errada no MECANISMO

O §11.4 declarava a dívida assim: *«a taxonomia é do PROJECTO e não do `ProjectState`, então estes
verbos não produzem passo de undo»*, e o `project_catalogs` justificava-a com *«metê-la na captura
faria toda renomeação de gaveta reescrever o snapshot do mundo inteiro»*.

⛔ **Isso é falso desde a F2** — a captura do mundo é **incremental** e custa o tamanho da edição.
O custo real é outro, e agora está medido (`measure_catalog_capture_cost`):

| catálogos | atribuições | bytes | `collect` | % de um quadro de 16,7 ms |
|---|---|---|---|---|
| 4 | 20 | 827 | 8,9 µs | 0,05 % |
| 20 | 200 | 7 502 | 87,8 µs | 0,53 % |
| 50 | 2 000 | 71 132 | 802 µs | **4,8 %** |
| 200 | 10 000 | 358 514 | 4 680 µs | **28 %** |

⇒ o caro era **codificar por quadro**, e a captura corre em todo quadro com input. ⭐ A cura é a
cache por revisão — codifica-se **uma vez por mutação** —, não ficar de fora.
*Uma dívida justificada por um mecanismo que não é o verdadeiro sobrevive a quem a podia pagar.*

#### As DUAS metades, porque «del» tem dois significados neste painel

| Gesto | Onde vivia | Era desfazível? |
|---|---|---|
| catálogo *New / Rename / Delete*, e arrastar para uma gaveta | `CatalogTree`, **ao lado** do `ProjectState` | ❌ |
| *Remove from Library* num **prefab** | dissolve o mestre ⇒ mundo | ✅ já era |
| *Remove from Library* numa **imagem** | `TextureLibrary` (memória de sessão) | ❌ **e era irreversível** |

⛔⛔ **O segundo caso era pior do que «não desfaz»:** a biblioteca é reconstruída do mundo a cada
quadro e uma imagem **sem utilizadores não tem quem a re-lembre**, então esquecê-la era para
sempre. ⇒ hoje `forget` põe uma **lápide** em vez de apagar a entrada: o `build` filtra-a, e
desfazer é tirar a marca — *a entrada está lá para poder voltar.*

⚠️ **E trazer de volta pela porta da frente LEVANTA a lápide**: re-importar a mesma imagem (mesmos
bytes ⇒ mesmo blake3) devolveria, sem isso, um asset **invisível para sempre** e sem gesto nenhum
que o explicasse. *Um «traz isto» explícito ganha a um «tira isto» antigo.*

#### ⚠️ A revisão é chave de CACHE e **nunca** identidade

Uma árvore restaurada nasce com revisão `0` e a original tem `N`. Se a revisão contasse para a
igualdade, **todo undo registaria um passo espúrio** no quadro seguinte e o Ctrl+Z seguinte não iria
a lado nenhum. ⇒ o `PartialEq` do `CatalogTree` é escrito à mão sobre o **conteúdo**, e há gate
(`a_restored_tree_encodes_to_the_same_bytes`). ⛔ E a cache é **invalidada** em todo sítio onde a
árvore é substituída por baixo (undo · `Open Project`) — a colisão de revisão ali é o caso
**normal**, não o raro.

#### `PROJECT_SCHEMA` 104 → 105, e este degrau **não é aditivo**

A taxonomia **saiu** do `ProjectFile` e **entrou** no `ProjectState`. ⚠️ Manter as duas era a
alternativa, e seria a segunda resposta à mesma pergunta com o load a escolher qual acreditar. Um
campo saiu do meio de uma estrutura e outro entrou no meio da outra ⇒ os bytes de um v104 passam a
**significar outra coisa**, e o postcard lê torto e cala-se. É o degrau mais perigoso desta escada
desde o 102.

### §11.7 — A auditoria do undo (2026-08-30) — **4 defeitos, 3 gates coincidentes, todos fechados**

#### No PRODUTO

| # | Achado | Mecanismo | Cura |
|---|---|---|---|
| **A2** | ⛔⛔ o `next_id` **não sobrevivia ao round-trip** ⇒ ids RECICLADOS | `collect` não o gravava e o `restore` derivava `max(id)+1`, que devolve o id de um catálogo **apagado**. ⚠️ Só ficou alcançável quando o undo passou a substituir a árvore a meio da sessão: a escolha da coluna é **vista** e sobrevive ao Ctrl+Z ⇒ passaria a apontar, em silêncio, para um catálogo criado depois | o número **viaja no blob** (`CATALOG_DOC_VERSION` 1→2); o `max` fica como piso |
| **A5** | ⛔⛔ o laço de render **levantava a lápide** | `remember` limpava a marca e corre **por quadro**: tirar a imagem, fechar o painel, re-importar, reabrir ⇒ a lápide caía **sem gesto**, e um Ctrl+Z a repô-la era desfeito no quadro seguinte — *o Ctrl+Z não pegava e queimava um passo* | a decisão sai da escrita e vai para a **leitura**: quem o mundo usa AGORA ganha à lápide, que é a regra que a recusa do verbo já usava |
| **A6** | um load **sem GPU** herdava as lápides do projecto anterior | `apply` fazia duas coisas — escrevia um global **e** devolvia a árvore —, e o valor de retorno prendia a chamada dentro do `if let Some(gfx)` | partida em `apply_forgotten` (global, fora da guarda) e `apply_catalogs` |
| **A7** | o braço `98 =>` do load podia ler **torto** | ele lê os bytes com o tipo **VIVO**; a v104 apendava no fim (falha limpa, *«fim do buffer»*), a v105 pôs um campo **no meio** | o braço **morreu** — um v98 é recusado com o número na frase, a decisão que a `line/Vector` já tomou para o v97 |

#### Nos GATES — três passavam por coincidência

| # | Gate | Porque era verde |
|---|---|---|
| **A1** | *nenhum* | ⛔⛔ **a lei-título — «esquecer MARCA, não apaga» — não tinha gate**: um `forget` que marcasse **e** apagasse sobrevivia à suíte inteira, e o produto voltava a ser irreversível. O gate vizinho passava igual com o `forget` ANTIGO: ele mede o que se **vê**, e a lápide e o `remove` escondem exactamente o mesmo ⇒ a régua nova é a diferença entre `len()` (*«quantas mostro»*) e `stored_len()` (*«quantas posso devolver»*) |
| **A2** | `a_restored_tree_encodes_to_the_same_bytes` | a fixtura **nunca apagava** um catálogo, e sem um id morto o `max(id)+1` calha certo |
| **A4** | `the_cache_re_encodes_on_a_change_and_only_then` | as asserções eram sobre os **bytes de saída** e o `collect` é determinístico ⇒ apagar a guarda deixava-o verde com a cache a codificar por quadro. *A lei que paga o desenho inteiro não tinha instrumento* ⇒ contador `probe_encodes()` |

Mais: o round-trip pelo **ficheiro** punha uma `LibraryDoc` **vazia** (o irmão do
`the_ui_states_travel_in_the_file` nasceu agora, populado), e a invalidação da cache era obrigação
manual sem censo ⇒ gate de árvore `every_site_that_replaces_the_catalog_tree_invalidates_the_cache`.

⚠️ **E o censo obrigou a RENOMEAR um campo:** `LibraryDoc.catalogs` (bytes) colidia com
`AppGfx.catalogs` (a árvore viva), e o gate textual acusava a cache. ⛔ Ensinar-lhe a excepção seria
pedir-lhe que adivinhasse a diferença ⇒ `catalog_bytes`. *Duas coisas diferentes com o mesmo nome
são um gate cego à espera de acontecer.*

#### ⚠️ Uma justificação minha estava **mecanicamente falsa**

O commit dizia que, com a revisão a contar para a igualdade, *«todo undo registaria um passo
espúrio»*. **Não**: nenhum código de produto compara duas árvores — o diff compara os **bytes**, e o
`collect` nunca serializa a revisão. O `PartialEq` à mão continua certo (a revisão não é
identidade), mas quem o lê tem de saber que o diff não passa por ali. *Uma nota que promete o
mecanismo errado manda a próxima LLM procurar o defeito no sítio errado.*

#### O que a auditoria mediu e **ILIBOU**

O passo nasce e é **UM** (o `Click` sai no `Up`, o `held_button` já voltou a `None`, o dreno corre
antes do `post_frame_undo`) · o **save** lê pela mesma porta e grava a taxonomia certa · os dois
sítios que substituem a árvore invalidam · um **v104 é recusado em voz alta**, com o número na
frase · e nenhum `&mut self` do `CatalogTree` muta sem bump.

#### ⏳ Aberto e NOMEADO

- **`count_in` conta as lápides**: a linha do catálogo diz `N` e a grade desenha `N−1` para uma
  imagem removida mas ainda atribuída. Pré-existente (o `forget` antigo também não desatribuía).
- **Residência**: a medição responde ao **relógio** de codificar, não à **memória** de guardar —
  `UNDO_CAP = 256` × o blob inteiro por passo. Nomeado, não medido.
- **A costura `App`**: os gates chamam `capture` e `apply_*` **directamente**; o caminho pelo `App`
  (cache → captura → passo → restauro) só existe no smoke `=78`.

#### ⏳ O que fica NOMEADO e não curado

Quando o dreno **recusa** o nome (vazio ou com `/`), o campo já fechou e **o texto escrito
perde-se** — há toast a explicar, mas o artista tem de reabrir e reescrever. Curá-lo é manter o
campo aberto sobre uma recusa, o que exige o dreno responder ao painel; hoje ele só fala por toast.

<a id="12"></a>
## §12 — ⭐⭐ **O FUNDO DE UM CARTÃO É O FUNDO DO CANVAS** (report do Enio, 2026-09-02)

> *«seria interessante que o fundo do ícone do asset seja da mesma cor do fundo do canvas mesmo
> quando se muda a cor do canvas»*

### §12.1 — A cláusula que manda é a SEGUNDA

Pintar o cartão com a cor que o canvas tem **hoje** satisfaz a primeira metade e falha a segunda em
silêncio — e era exactamente a doença que já lá estava: **três sítios respondiam «de que cor é o
fundo do canvas?» por conta própria**, e cada um estava certo **sozinho**.

| Sítio | O que respondia | O que se via |
|---|---|---|
| `paint_canvas_bg` (editor-core) | `ColorToken::Bg1` | **só no modo fixtura** — em modo vivo este fill é **saltado** |
| o `clear` da camada de sprites (shell) | literal `(0.047, 0.047, 0.055)` | **é este o fundo que o artista vê** |
| o cartão do navegador | a **cor dominante do asset** (wave A2) | o objecto lia-se de uma cor no cartão e de outra na tela |

⇒ A cura é a **porta única** [`canvas_backdrop(theme)`](../../crates/ph2d-editor-core/src/screens/hero/canvas.rs),
e os três passam a lê-la. Re-vestir o token (trocar de tema **ou** autorar `bg-1` no painel de
Tokens) move os três no mesmo quadro.

### §12.2 — ⛔ O que **não** mudou, e porquê

- **A conversão sRGB→linear continua por fazer** no `clear` (o byte divide-se por 255). Está errada
  em teoria e é o que o produto precisa: as bordas anti-aliased do chrome estão calibradas contra o
  fundo legado, e linearizar a sério é a regressão dos *"pixelated borders"* da M14.5 ronda 2 —
  **medida e revertida**. ⭐ A cerca nunca tivera gate; agora tem
  (`the_forge_clear_stays_where_the_chrome_anti_aliasing_was_calibrated`), e ele defende a
  **distância** ao valor legado, não o literal.
- ⛔ **O token `canvas` NÃO foi adoptado**, e ele é um órfão cujo doc-comment diz literalmente esta
  frase (*«viewport background»*). Ele vale `#020202` no Forge — **cinco vezes mais escuro** do que
  o que se vê —, então ligá-lo não seria curar um órfão: seria reabrir aquela regressão. *O token
  que NOMEIA a pergunta e o token que a RESPONDE não eram o mesmo.*
- ⛔ **A cor dominante (A2) não morreu.** Ela deixa de ser fundo e volta ao papel que a justificava:
  a **cara** de um cartão que ainda não tem miniatura. O orçamento de miniaturas é por quadro, logo
  «sem miniatura» é normal e transitório — apagar a cor ali daria uma grade de quadrados iguais.
- ⛔ **Um xadrez é a terceira resposta, recusada.** Ele diz *"aqui há transparência"*, que é
  informação sobre o **ficheiro**; pediu-se ver o objecto **como ele vai aparecer**, que é
  informação sobre a **cena**.

### §12.3 — ⭐⭐⭐ O gate que quase mentiu nos DOIS sentidos

O censo textual de *"o `clear` sai da porta?"* ficou **VERDE com o literal reposto** — porque a nota
histórica que eu escrevi ao lado dele **menciona o nome da porta**. E o censo irmão, *"ninguém
escreve o fundo à mão"*, ficou **VERMELHO sobre a mesma nota**, que cita o valor legado de
propósito.

⇒ *Uma varredura de fonte que não separa prosa de código **acusa a prosa e absolve o código**.* Só a
prova de mutação o mostrou: os dois censos passam a ler apenas linhas de código, e a nota histórica
fica onde tem de ficar — ela é a única coisa que carrega o mecanismo da cerca da M14.5.

### §12.4 — Correcções ao §9.3 desta mesma página

- ✅ **A6, as miniaturas** — **feita** (2026-09-02): o retrato de um componente é composto na CPU a
  partir das miniaturas das peças ([`asset_card_portrait.rs`](../../shells/desktop/src/asset_card_portrait.rs)),
  sem GPU e sem tocar no atlas. A linha do §9.1 que a dá por pendente envelheceu.
- ✅ **Imagens de 16 bits com a cor neutra** — **fechada**: o `swatch_for` passou a ir pela porta
  `image_rgba8`, que cobre as duas variantes.

<a id="13"></a>
## §13 — ⭐⭐ **AS DUAS PERGUNTAS DE RELAÇÃO** (D9, 2026-09-02)

**O que o artista ganha:** botão direito num cartão → **Show what it uses** / **Show what uses it**.
A grade estreita-se para a resposta, com uma faixa por cima a dizer qual é a pergunta e um `✕` para
a largar. É a pergunta que precede *mudar* e *apagar*.

### §13.1 — ⚠️ *Show what uses it* **não é** *Select users*, e a diferença decide um gesto

| Item | Responde | Onde estão |
|---|---|---|
| **Select users** (já existia) | que **objectos da CENA** usam isto | no canvas — e por isso ele **selecciona** |
| **Show what uses it** (novo) | que **receitas da BIBLIOTECA** usam isto | em lado nenhum da cena ⇒ nenhuma selecção as alcança |

⇒ um artista que vai mudar uma textura precisa **das duas**, e elas nunca dão a mesma resposta. Foi
por isso que a metade nova não pôde reusar o verbo que existia.

### §13.2 — Onde cada decisão vive, e porquê

- **A relação é um FILTRO da consulta que já existia** (`Query::related`), e não uma segunda
  consulta. A grade tem **uma travessia** — o que se pinta e o que se arrasta saem da mesma lista —
  e uma `deps()` chamada à parte devolveria outra ordem e outro recorte de catálogo.
- ⛔ **Ela NÃO passa pelo barramento**, ao contrário dos três verbos vizinhos do mesmo menu. A
  fronteira é o que cada um toca: aqueles mudam o **mundo**; estes mudam **o que a grade mostra**,
  que é vista do painel — como o chip de família.
- ⚠️ **Compõe com a busca e o catálogo em vez de os substituir.** Um modo que desligasse controlos
  visíveis deixá-los-ia a mentir no ecrã.
- ⛔⛔ **Âncora que já não existe ⇒ resposta VAZIA, nunca a biblioteca inteira.** O asset pode sair
  da biblioteca entre o clique no menu e o quadro seguinte; um filtro que se desligasse sozinho
  devolveria tudo **por baixo de uma faixa a dizer «o que usa X»** — a resposta errada com a
  etiqueta certa. Gate: `an_anchor_that_left_the_library_answers_nothing_not_everything`.
- ⚠️ **A faixa traz o próprio `✕`**, e não é decoração: os outros dois filtros da grade têm controlo
  permanente no cabeçalho, este nasce de um menu. *Um filtro que só um menu liga tem de trazer o
  próprio interruptor de desligar.*
- ⚠️ **A grade vazia FALA a relação primeiro** (*«This asset uses nothing else»*), antes de *«nothing
  matches this search»* — com o filtro ligado, o vazio **é** a resposta, e é um facto sobre o asset.

### §13.3 — ⭐⭐⭐ O gate que acusou de mortos dois itens VIVOS, e mandava a cura errada

`every_asset_card_menu_entry_dispatches_something` — o censo que garante que nenhuma linha do menu
é pintada e morta — **reprovou** os dois itens novos, com a mensagem *«ligue cada uma no
`card_verb_of` e drene a acção no `asset_card_verbs.rs`»*.

Ele estava a perguntar **«empurrou para o barramento?»**, que era o único destino que existia quando
foi escrito. Os dois itens estão vivos e correctos: eles escrevem no `AssetBrowserState`.

⇒ *Um gate que presume o **destino** de um efeito acusa de morto quem tem outro — e a mensagem dele
manda alguém construir a doença.* A pergunta passou a ser **«alguma coisa mudou?»** (o barramento
**ou** o estado do painel), comparado por `{:?}` para que um campo de vista novo entre no oráculo
sozinho — a mesma razão de a fonte ser a tabela do menu.

### §13.4 — ⛔⛔ E o `✕` estava MORTO SOB O DEDO com cinco gates verdes

O censo da workspace (`hit_indexed_ids_are_registered`) apanhou o que os meus gates não viam: o
botão era **pintado e hit-indexado** pelo `paint_related` e **não tinha `InteractiveState` no
store** ⇒ `is_focusable` falso, o `Down` não arma, e o **`Click` nunca nasce**.

⚠️ **Os cinco gates desta fatia passavam**, e um deles chamava-se *«o `✕` larga o filtro»*: um
`Click` sintético num teste **não passa pelo store**, então ele media o braço do `apply_event` e
nunca a costura que o precede. ⇒ o gate ganhou a segunda metade (`store().get(…)` é um `Button`), e
o registo vive no `populate` — **fixo, ainda que a faixa seja condicional**: registar no paint
reintroduziria o `register` a apagar o `Pressed` do quadro anterior, que é o defeito que os cartões
já pagaram.

### §13.4-bis — E o mesmo `✕` era MUDO para a acessibilidade

O gate da **HR-12** (`every_widget_file_wires_a11y`) acusou o ficheiro novo: o `✕` era um
`paint_icon` cru, e um ícone desenhado à mão não fala à árvore de acessibilidade **nem veste o
hover vivo** — ele nasceria a pintar uma cor dura no meio de vizinhos que deslizam.

⛔ **A lista de dispensa do gate não era a cura.** Ela serve um ficheiro *sem semântica de
utilizador*; isto é um botão. ⇒ ele passa pela primitiva canónica (`paint_icon_button`), que traz as
duas coisas de graça. ⚠️ Note-se que o `✕` do CABEÇALHO deste mesmo painel continua a ser um
`paint_icon` cru e o gate não o vê — o ficheiro dele satisfaz a regra por usar `paint_button`
noutro sítio. *Um gate por FICHEIRO absolve todo controlo cru que partilhe ficheiro com um
canónico* — nomeado, não curado, e fora desta fatia.

### §13.5 — E o gate da grade media o próprio arnês

A 1.ª versão de `the_question_reaches_what_the_grid_actually_paints` montava uma `Query` à mão e
passava-a ao `probe_query` — o que prova que o **índice** filtra (a `index_law` já o provava) e não
que o painel liga o `state.related` à consulta que ele constrói. A régua passou a ser o
`probe_painted_at`: **o que o `paint` de facto desenhou**. *Um gate que fabrica o alcance mede o seu
próprio arnês.*

### §13.6 — ⛔⛔⛔ O report *«layout ruim»* (foto do Enio, 2026-09-02) — **os oito gates passavam**

A faixa nascia **à largura toda do painel**, e a foto mostra o resultado: o rótulo *«What "Canvas"
uses»* por baixo do botão **+ Catalog**, cortado, e a faixa por cima da coluna de catálogos.

**Duas causas, e as duas são de geometria:**

| O que estava | O que é |
|---|---|
| a faixa medida sobre `rect.w` inteiro, pintada **antes** da coluna | ela mede a largura da **grade** (`rect.x + col_w + pad()`), e por isso é pintada **depois** da coluna — a largura da grade só existe quando a coluna diz quanto tomou |
| linha de base em `band.y + row_h − Xs` (o **bordo de baixo**) | `band.y + (row_h − size) · 0,5` — a MESMA centragem do `paint_list_item` que a coluna ao lado já usa |

⚠️ *Uma linha de base não é uma margem*, e *a largura de um controlo é uma afirmação sobre o que ele
manda*: à largura toda, a faixa dizia que filtrava também a coluna — e não filtra.

### §13.7 — ⭐⭐⭐ Porque os oito gates desta fatia não viram nada

Eles perguntavam duas coisas, e as duas eram **verdadeiras** sobre a foto:

1. *«o id está no índice de toque?»* — estava.
2. *«o clique chega ao estado?»* — chegava.

⇒ **um controlo pode estar VIVO, ALCANÇÁVEL e no SÍTIO ERRADO, e são três perguntas.** Este repo
tem instrumentos para as duas primeiras (o `hit_indexed_ids_are_registered`, os `seam_*`) e
**nenhum** para a terceira: nada mede se dois controlos do mesmo painel se pisam.

A terceira entra agora, para esta faixa: `probe_band_rect` publica o rectângulo que o `paint`
desenhou, e `the_band_never_covers_the_catalog_column` cruza-o com o que a coluna registou. A
mutação que repõe a largura inteira — **literalmente a foto** — sangra só esse gate.

⏳ **NOMEADO e não curado:** a lei geral (*«dois controlos irmãos não se sobrepõem»*) não pode ser
um censo cego sobre todos os rects de um painel — uma linha dentro da coluna sobrepõe-se
legitimamente à região dela. Ela precisa da noção de **irmão na mesma faixa de layout**, que este
repo não tem.

### §13.8 — ⛔ E um gate meu foi APAGADO, com o motivo escrito

Escrevi também `the_bands_close_is_not_stacked_under_the_panels_close` — o `✕` da faixa não podia
partilhar coluna de pixels com o `✕` que fecha o painel. **Ele reprovou a cura**, e foi esse o
sinal: a regra era **minha**, não do report. O Enio fotografou um rótulo por baixo de um botão;
*«dois `✕` na mesma coluna»* foi a minha leitura da foto, e é o padrão normal de toda janela com uma
barra dentro. Satisfazê-lo obrigaria a pôr o `✕` num sítio que ninguém procura.

⚠️ *Um gate que reprova a cura de um defeito real está a medir a preferência de quem o escreveu.*

### §13.9 — ⛔⛔⛔ *«não conseguiu listar os ítens»* — a **QUARTA** vez que a mesma pergunta foi respondida com a metade errada

Foto do Enio (2026-09-02): três objectos na cena, cada um de peças com textura, e o
*Show what it uses* a responder **«This asset uses nothing else»**.

A linha era esta:

```rust
.filter_map(|&p| sim.world().get::<SpritePixels>(p).map(|sp| sp.0))
```

Ela pergunta só por `SpritePixels` — **o carimbo, a minoria**. Toda imagem importada e toda tela
nova é uma sprite de **átlas**, logo `deps` nascia **vazio para o caminho normal**. ⚠️ E como o
sentido *Owners* é **derivado por inversão** dessa lista, **uma linha calava os dois sentidos**.

⛔⛔ **A porta estava quinze linhas abaixo, já a ser chamada pelo vizinho** (o retrato), com um
doc-comment a prometer que *«uma terceira forma amanhã entra aqui e não volta a partir o cartão»*.
*Uma porta que o vizinho não chama ainda não é uma porta — é uma função.*

**As quatro ocorrências, todas fechadas por um report:**

| # | o que ficou mudo | como o Enio o leu |
|---|---|---|
| 1 | a cor dominante de um prefab | *«não funcionou»* — cartão cinzento |
| 2 | a peça-cara (`largest_piece_texture`) | idem |
| 3 | os utilizadores de uma imagem (`users_of`) | *«Selected 1 object(s)»* e nada acendia |
| 4 | as **dependências** de um prefab | *«não conseguiu listar os ítens»* |

⇒ a cura desta vez **não é a linha**: é o censo `the_index_asks_the_texture_door`, que recusa
qualquer leitura de `SpritePixels` nos três ficheiros do assunto que não entregue ao `texture_of`.
A mutação que repõe a linha antiga sangra o censo **com o número da linha** e o gate de valor.

⚠️ **E o gate de valor já tinha a fixtura CERTA** — `a_prefab_of_atlas_sprites_gets_a_colour_and_a_portrait`
monta exactamente a cena da foto. Ele media a **cor** e o **retrato** (as duas curadas na ocorrência
1) e não media as `deps`. *Uma fixtura que já tem o fenómeno não protege as perguntas que ninguém
lhe faz.* Ele passa a fazer as quatro, incluindo o sentido inverso — **derivado não é medido**.
