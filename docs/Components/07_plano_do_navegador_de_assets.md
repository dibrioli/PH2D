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
- ⏳ **`Dependencies` / `Owners` no menu de contexto** (D9): o índice **responde** aos dois sentidos
  com gate, e falta o menu que os mostra.
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
