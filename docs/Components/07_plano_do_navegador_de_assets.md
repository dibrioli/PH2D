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
[§6 Fora, com motivo](#6) · [§7 A auditoria antes do smoke](#7) · [§8 Riscos MEDIDOS](#8)

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
| D2 | **Layout dividido na vertical** por omissão (catálogos em cima, grade em baixo) + botão para **só-grade** | `DisplayMode` do Godot, reduzido a dois: `HSPLIT` só faz sentido com largura que não temos |
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
