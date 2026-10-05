# 01 — PESQUISA: Miro e Excalidraw — o que um quadro de planear, organizar, brainstorm e equipa PRECISA

> **Ordem do dono (2026-10-05):** *«Vamos fazer uma pesquisa sobre o MIRO e o excalidraw. Vamos criar um
> canvas similar dentro da nossa engine. Faça uma pesquisa das features mais úteis que podem ser usadas
> aqui para planejar, organizar, fazer BrainStorm, trabalhar em equipe.»*
>
> **Como foi feita:** três pesquisas em paralelo — Miro (web), Excalidraw (web + estado da máquina) e o
> inventário do que o PH2D já tem (código, com as afirmações decisivas re-conferidas à mão, marcadas ✔).
> ⚠️ O help center do Miro, o G2 e o TrustRadius devolvem HTTP 403 a leitura directa: o que se lhes
> atribui vem dos excertos do motor de busca, não da página inteira. Tudo o resto foi lido.
>
> ⚠️ Isto é **o que existe**, não **o que faz** (a lição da `docs/Components/pesquisa/dossie_godot…` §10):
> cada célula que virar wave abre com uma sonda que **corre** o oráculo (§7).

---

## §1 — A resposta: as features que mais valem, por objectivo

Síntese das notas de reviews (SoftwareReviews, 423 reviews: Collaboration 87 · Annotation 85 · Infinite
Canvas 84 · Sticky Notes 84 · … · AI 75 — a IA é a mais baixa), dos casos de uso declarados (TrustRadius:
«idea management» 47 %; brainstorm «importante» para 99 % de 84) e das dicas de facilitadores. A ordem
dentro de cada grupo é a do valor medido/citado, não a do custo.

| objectivo | o que mais vale | Miro | Excalidraw | PH2D hoje |
|---|---|---|---|---|
| **Base (tudo depende disto)** | canvas infinito com zoom/pan fluido | ✅ | ✅ | ✅ ✔ |
| | **sticky note** — `Tab` cria a seguinte, cor, tamanho, texto que cabe | ✅ | ✅ só no master | ⭐ |
| | formas + texto dentro da forma | ✅ | ✅ | ✅ (47 tipos de forma, texto em caminho) |
| | **conector preso** a formas, que segue ao mover, com rótulo | ✅ (sem desvio) | ✅ (cotovelo) | ✅ ✔ **e desvia de paredes** |
| | caneta / desenho livre | ✅ | ✅ perfect-freehand | ✅ (Flip) |
| | imagens (colar, arrastar) | ✅ | ✅ + recorte | ◑ |
| | desfazer/refazer | ✅ | ✅ | ✅ ✔ fila única por diff |
| **Organizar** | **frames** — levam os filhos, têm nome, viram slides | ✅ | ✅ | ◑ (moldura do auto-layout) |
| | agrupar · travar · alinhar/distribuir · smart guides | ✅ | ✅ | ✅/◑ (travar a conferir) |
| | busca por texto + tags/filtro | ✅ (1 tag de cada vez) | ✅ busca | ◑ (`Tags` existe; busca a conferir) |
| | minimapa | ✅ `M` | — | ⭐ |
| | links entre objectos / quadro-índice | ✅ | ✅ | ⭐ |
| **Planear** | **mind map** com layout automático (`Tab` filho, `Enter` irmão) | ✅ | ◑ (fluxograma por `Ctrl+seta`) | ⭐ (o auto-layout Taffy é a peça) |
| | fluxograma: pontos azuis que criam a forma seguinte já ligada | ✅ | ✅ | ⭐ (conector existe; o gesto não) |
| | cartões com campos · kanban · tabela · timeline | ✅ | — | ⭐ |
| | templates | ✅ (centenas) | bibliotecas `.excalidrawlib` | ⭐ |
| **Brainstorm** | **votação** (N votos/pessoa, escondida até ao fim) | ✅ | ⭐ | ⭐ |
| | **timer** partilhado | ✅ | ⭐ | ⭐ |
| | **modo privado** (escrever sem ver, revelar depois) | ✅ | ⭐ | ⭐ |
| | agrupar stickies (por cor, autor, tag, palavra-chave/IA) | ✅ | ⭐ | ⭐ |
| | colar uma folha de cálculo → uma sticky por célula | ✅ | — | ⭐ |
| | auto-arrumar em grelha | ✅ | — | ◑ (alinhar existe) |
| **Equipa** | cursores ao vivo + presença | ✅ | ✅ (sala por link, cifrada) | ⭐ ✔ (zero rede) |
| | comentários + @menções | ✅ | Plus | ⭐ |
| | «trazer todos até mim» / seguir alguém | ✅ | Plus (Spotlight) | ⭐ |
| | histórico de versões | ✅ (1/h, 90 dias, restaura num quadro NOVO) | — | ◑ (undo de sessão; ficheiro) |
| | apresentação por frames | ✅ | Plus | ⭐ |
| | export PNG/SVG/PDF | ✅ | ✅ (SVG/PNG reabrem editáveis) | ◑ (SVG ✅; região/PDF a conferir) |

---

## §2 — Onde os DOIS falham — é aqui que um editor desktop em Rust ganha

Esta é a coluna que decide o desenho, porque é a queixa repetida dos utilizadores de ambos:

| queixa | Miro | Excalidraw | o que nós temos para responder |
|---|---|---|---|
| **quadros grandes ficam lentos** | o próprio Miro manda ficar < **5 000** objectos e esconder cursores; cai a partir de ~1 000 | 10–14 mil objectos → ~30 fps; mexer → ~10 fps (#8136, fechado «not planned»); > 5 mil trava (#7280); **sem índice espacial** (#10063) | Vello na GPU; o Motion mediu 4,19 M objectos a 3,85 ms. ⚠️ **o número do quadro está por medir** (§0.0: mede-se antes de limitar) |
| **sem offline** | pedido de anos; «Reconnecting…» a meio de apresentações | 1 cena local no grátis | o projecto é um ficheiro (`.ph2dproj`) — offline é o estado natural |
| **conector não desvia** de objectos | pedido aberto | o cotovelo desvia (tldraw também) | ✅ ✔ o nosso já desvia de paredes (`connector_walls.rs`) |
| **sem nível de detalhe por zoom** (longe, o miúdo polui) | pedido aberto | — | ⭐ desenho nosso |
| **frames aninhados** · pertença ao frame ambígua | pedido aberto · relatos | — | hierarquia ECS nativa (`ChildOf`) |
| **texto sem negrito/itálico** | ✅ tem | ⭐ não tem | ⭐ não temos (parley sem estilo por trecho) |
| **complexidade** — «cada feature nova deixa mais lento» | queixa forte | é o elogio dele: «9 ferramentas, os ícones dizem o que fazem» | decisão de produto (§8) |
| arrastar para seleccionar move um objecto; linhas criadas sem querer | relatos | ligações indesejadas | tratar no gesto (§4) |

---

## §3 — Miro, em resumo (fontes no §9)

- **Canvas:** infinito; minimapa (`M`); grelha de linhas ou pontos (`G`) com snap, `Ctrl` suspende; smart
  guides azuis de alinhamento e espaçamento igual; «Quick Arrange»; tecto de 100 000 objectos por quadro.
- **Objectos:** sticky (16 cores, S/M/L, até 8 tags); 3 000+ formas («Miro Diagrams»); conectores; texto;
  caneta (`P`); borracha (`E`); cartões com campos; tabela tipada que alterna tabela/grelha/timeline/kanban
  (1 000 × 50); mind map; em 2025: Docs, Slides, Prototypes, Roadmaps, «Synced Copies» entre quadros.
- **Organização:** frames como páginas; agrupar `Ctrl+G`; travar; ordem `PgUp/PgDn`; busca `Ctrl+F` com
  filtro de tag; para quadros enormes o Miro recomenda um **quadro-índice** com links.
- **Brainstorm:** votação anónima (≤ 99 votos/pessoa, resultado só no fim); timer; modo privado; cluster por
  palavra-chave/autor/tag/cor/sentimento; templates; IA (Sidekicks, Flows, «Catch-up» do que mudou).
- **Equipa:** cursores ao vivo; comentários; *Attention management* (Follow · Bring to me · Bring everyone
  to me); Talktrack (passeio gravado com voz); histórico 1/h guardado 90 dias.
- **Apresentação:** frames são slides; ordem arrastável ou «Magic organize» (cima→baixo, esq.→dir.);
  esconder um frame da apresentação.
- **Import/export:** PDF, JPG, SVG, CSV · importa VSDX (Visio/Lucid/draw.io), PDF, PowerPoint.
- **Preço:** grátis = 3 quadros editáveis; Starter 8 USD, Business 20 USD por membro/mês.

**FigJam** (contraste útil): a sticky **cresce na vertical** em vez de encolher a fonte; *cursor chat*
(`/`, mensagem efémera junto ao cursor); stamps; secções que se recolhem; timer e votação existem
(a doc oficial da Figma desmente o comparativo da ideaplan que diz o contrário).

**tldraw**: é um SDK, não uma plataforma de workshop; seta «imprecisa» (aponta ao centro) × «precisa»
(ponto exacto); cotovelos que contornam obstáculos; offline. ⛔ **Licença própria, source-available**
(chave obrigatória em produção, marca d'água no plano hobby; o MIT só sobrevive no `tldraw-v1` arquivado).

---

## §4 — Os comportamentos que fazem a diferença (copiar fielmente, medindo)

1. **Conector — a semântica da âncora** (Miro): preso ao **centro**, a linha acompanha a borda mais próxima e
   nunca atravessa a forma; preso a um **ponto fixo**, fica nesse ponto e pode atravessar. Tipos: recto,
   cotovelo, curvo; rótulo; pontas. Excalidraw: a ligação só acontece ao **arrastar a ponta** (desde 05/2024,
   para matar ligações indesejadas); `Ctrl` evita ligar. tldraw: pairar devagar = ponto exacto.
   Queixas: linhas criadas sem querer; não haver linha «solta» que não se agarre.
2. **Pontos azuis de criação rápida:** com uma forma seleccionada, um clique num lado cria a forma seguinte
   **já ligada**; arrastar liga a uma existente. Excalidraw: `Ctrl+seta` cria o próximo nó, `Alt+seta` navega.
3. **Sticky:**
   - `Tab` a editar cria a seguinte à direita, mesma cor e tamanho, com o foco nela; `Enter` = modo em massa.
   - **Decisão aberta (§8):** Miro **encolhe a fonte** para caber num quadrado fixo (queixa: stickies
     vizinhas com letras de tamanhos muito diferentes) × FigJam **cresce na vertical**.
   - «Pilha» de onde se arrastam stickies em branco; colar folha de cálculo = uma por célula; pega que
     auto-arruma uma selecção desordenada em grelha.
4. **Mind map:** `Enter` irmão (já em edição) · `Tab` filho · setas navegam · `Shift+Enter` quebra linha ·
   `Delete` apaga o ramo; auto-layout ligado por omissão, desligável no pai.
5. **Frame:** move e copia os filhos; escalar o frame escala o conteúdo; nome, cor, proporção; ordem de
   apresentação. Ambiguidade de pertença é queixa ⇒ a pertença tem de ser **visível**.
6. **Atalhos de uma tecla** (convergem nos dois): `V` seleccionar · `H` mão · `N` sticky · `T` texto ·
   `R`/`S` forma · `A`/`L` seta/linha · `P` caneta · `E` borracha · `F` frame · `C` comentário · `M` minimapa ·
   `G` grelha · `Ctrl+D` duplicar · `Ctrl+G` agrupar · `Ctrl+Shift+L` travar · `Ctrl+F` busca.
   ⚠️ No PH2D vários destes já pertencem a outros modos — o quadro precisa de um **modo próprio**
   (seletor *Mode*, spec/06 da UI/UX) para os ter sem colidir.
7. **Estética à mão (Excalidraw)** não é enfeite: *«the wonky style conveys the approximate precision of the
   presented concept»* — equipas usam o «sloppiness» para marcar **rascunho × final**. É rough.js com uma
   `seed` por elemento ⇒ **determinístico**, o que encaixa na nossa lei de determinismo.

---

## §5 — Excalidraw, em resumo

- **Licença: tudo permissivo.** Repo e `@excalidraw/excalidraw` 0.18.1 **MIT**; `roughjs` 4.6.4 **MIT**;
  `perfect-freehand` 1.2.0 **MIT**; fontes Virgil e Excalifont **OFL-1.1** (derivada não pode usar o nome
  «Virgil»); bibliotecas de libraries.excalidraw.com MIT; servidor `excalidraw-room` MIT. ⇒ **porta-se** o
  algoritmo do traço à mão e do desenho livre, com o aviso MIT. As outras fontes do repo (Nunito, Lilita,
  Comic Shanns, Xiaolai…) **não foram triadas** — triar antes de embarcar.
- **Ferramentas:** mão, seleção, retângulo, losango, elipse, seta, linha, desenho livre, texto, imagem
  (com recorte), borracha, frame, laser (`K`), embeddable; no master: **sticky** (`N`), balde (`B`),
  conta-gotas (`I`); `Tab` converte o tipo do elemento.
- **Estilo:** `fillStyle` hachure / cross-hatch / solid / zigzag; `roughness` 0–2; tracejado/pontilhado;
  cantos arredondados; pontas de seta (incl. cardinalidade).
- **Estrutura:** setas ligadas e em cotovelo (com segmentos fixáveis); texto em contentor com quebra;
  agrupar, alinhar, distribuir, travar, espelhar, copiar estilo; links; busca; paleta de comandos;
  modos zen / só leitura / grelha / snap; Mermaid → diagrama editável.
- **Export:** PNG/SVG/JSON/clipboard; com «embed scene» o PNG/SVG **reabre editável**; SVG com subconjunto
  da fonte embutido.
- **Colaboração:** **não é CRDT** — união por `id`, elemento a elemento: ganha a `version` maior, empate
  decide-se pelo `versionNonce` menor; apagar = lápide (`isDeleted`); ordem por índice fraccionário; duas
  edições a campos diferentes do mesmo elemento ⇒ **uma perde-se inteira**. Servidor só retransmite;
  AES-GCM com a chave no `#fragmento` do link (nunca chega ao servidor).
- **Plus (6 USD/mês):** nuvem, comentários, voz, Spotlight, slides, PDF/PPTX, IA (text-to-diagram,
  wireframe-to-code), API + MCP.
- **Formato `.excalidraw`:** `{type, version:2, elements[], appState, files}`; por elemento `id, type, x, y,
  width, height, angle, strokeColor, backgroundColor, fillStyle, strokeWidth, roughness, seed, opacity,
  version, versionNonce, index, isDeleted, groupIds, frameId, boundElements, locked, link`; seta com
  `points, startBinding, endBinding, elbowed`; texto com `containerId, autoResize`.
  ⚠️ O formato do `startBinding` **mudou entre a 0.18.x e o master** ⇒ o oráculo fixa a versão no cabeçalho.

---

## §6 — O que o PH2D já tem (peças para o quadro)

✔ = re-conferido à mão nesta sessão; o resto vem do inventário do explorador e confere-se antes de usar.

| peça | estado | onde |
|---|---|---|
| conector preso, segue a forma, rótulo, pontos de passagem, **desvia de paredes**; alvo por `VecPathId` (sobrevive a undo e save) | ✅ ✔ | `crates/ph2d-ecs/src/vec_connector.rs:72` · `crates/ph2d-app-vec/src/connector_{live,walls,drag,panel}.rs` |
| formas vectoriais, pontas de seta (8), booleanas vivas, auto-layout (Taffy), âncoras, export SVG | ✅ | `crates/ph2d-vec-scene/src/{kind,stroke_style}.rs` · `crates/ph2d-app-vec/src/{bool_live,bindings,align_live}.rs` · `crates/ph2d-vec-svg/` |
| texto no canvas (parley), texto em caminho | ✅ | `crates/ph2d-app-vec/src/text_edit.rs` |
| desenho livre com suavização (Flip) | ✅ | `crates/ph2d-app-flip/` |
| pan/zoom, grelha com LOD de linhas, snap, selecção por rectângulo, gizmo, multi-selecção | ✅ | `crates/ph2d-editor-core/src/grid.rs` · `crates/ph2d-app-vec/src/vec_selection.rs` |
| hierarquia, grupos (`ChildOf`), `Tags` em árvore | ✅ | `crates/ph2d-panel-hierarchy/` · `crates/ph2d-tags/` |
| projecto em ficheiro, undo por diff, `StableId` | ✅ ✔ | `shells/desktop/src/{project,undo}.rs` |
| imagens (16 formatos), clipboard | ✅/◑ | `crates/ph2d-imageio-*` · `crates/ph2d-editor-core/src/interaction/dispatch/clipboard.rs` |
| **importar SVG** | ⭐ ✔ só valida o ficheiro, não cria nada na cena | `crates/ph2d-imageio-svg/src/lib.rs:2-8` |
| **traço à mão (rough)** | ⭐ ✔ nenhum no vectorial | — |
| **rede / CRDT / presença / comentários** | ⭐ ✔ o `crdt.rs` é um esboço de 41 linhas (ADR-0057 prevê LWW+RGA) | `crates/ph2d-vector-doc/src/crdt.rs` |
| sticky, frame-slide, minimapa, votação, timer, mind map, templates, rich text | ⭐ | — |
| travar objecto, busca por nome, export de região/PDF, arrastar ficheiro | ◑ a conferir | — |

**Leitura:** a peça mais difícil de um quadro destes — o **conector preso que desvia** — já existe e é melhor
que a do Miro. O que falta é sobretudo **produto por cima do motor vectorial** (sticky, frame, mind map,
facilitação) e a **camada de equipa** (que é um assunto inteiro, §8).

---

## §7 — Oráculos (§0.9 do `CLAUDE.md`)

| app | licença | corre-se? | para quê |
|---|---|---|---|
| **Excalidraw** 0.18.1 (fixado) | MIT ✅ | **sim**: `@excalidraw/excalidraw` + Playwright/Chromium numa pasta **fora do repo**, chamando `exportToSvg`/`exportToBlob` sobre `.excalidraw` NOSSOS. Não está instalado; o Node 26 está (`~/.local/bin/node`) | (a) caminhos do rough.js por elemento, mesma `seed`; (b) contorno do perfect-freehand; (c) SVG final por geometria dos `d=`, depois por pixel; (d) `restore()`/`convertToExcalidrawElements` para ligação de setas e texto em contentor. ⛔ rejeitar `@moona3k/excalidraw-export` como oráculo do PRODUTO (reimplementa o render) — serve só para o passo do traço |
| **Miro** | proprietário, web, conta | não por script (só a interface) | referência de **comportamento** observado à mão; nunca fixture |
| **tldraw** | source-available, chave obrigatória | triagem de licença antes de qualquer corrida | conectores-cotovelo e binding preciso/impreciso |
| **Obsidian** (instalado em `~/Apps`) | — | o plugin Excalidraw dele é um caminho alternativo, **não verificado** | — |

---

## §8 — Decisões que são do DONO (produto), antes do plano

1. **Equipa: quanto e quando.** Colaboração ao vivo é um assunto inteiro (rede, servidor ou P2P, CRDT, presença,
   permissões). Opções: (a) **quadro local primeiro** — tudo o resto, um ficheiro, e a equipa troca o ficheiro;
   (b) local + **comentários e histórico no ficheiro** (equipa assíncrona, sem rede); (c) ao vivo desde o
   início. O Excalidraw mostra que o ao vivo **mínimo** (união por elemento + servidor que só retransmite) é
   pequeno; o Miro mostra que a facilitação (seguir, votar, timer) é o que as equipas usam.
2. **Sticky: encolher a fonte (Miro) ou crescer na vertical (FigJam)?**
3. **Estética:** traço limpo, traço à mão (Excalidraw), ou os dois com um botão «rascunho ↔ final»?
4. **Simplicidade:** o elogio n.º 1 do Excalidraw é ter poucas ferramentas; a queixa n.º 2 do Miro é ter
   demasiadas. Quantas ferramentas na barra do quadro?
5. **Onde vive:** um **modo próprio** do editor (seletor *Mode*, como Vector/Flip), com os atalhos de uma tecla
   do §4.6 só dentro dele.

---

## §9 — Fontes

**Miro:** help.miro.com — [Sticky notes](https://help.miro.com/hc/en-us/articles/360017572054-Sticky-notes) ·
[Connection lines](https://help.miro.com/hc/en-us/articles/360017730733-Connection-lines) ·
[Mind map](https://help.miro.com/hc/en-us/articles/360017730753-Mind-map) ·
[Frames](https://help.miro.com/hc/en-us/articles/360018261813-Frames) ·
[Voting](https://help.miro.com/hc/en-us/articles/360017572274-Voting) ·
[Attention management](https://help.miro.com/hc/en-us/articles/360013358479-Attention-management) ·
[Board history](https://help.miro.com/hc/en-us/articles/360021668819-Board-history-versions) ·
[Board performance](https://help.miro.com/hc/en-us/articles/360013588560-Board-performance-and-loading-issues) ·
[Export](https://help.miro.com/hc/en-us/articles/360017572754-How-to-export-your-board) ·
[Tables](https://help.miro.com/hc/en-us/articles/22760922335506-Tables) ·
[2025 recap](https://miro.com/blog/2025-recap/) · reviews:
[SoftwareReviews](https://www.softwarereviews.com/products/miro?c_id=295) ·
[G2](https://www.g2.com/products/miro/reviews) · [TrustRadius](https://www.trustradius.com/products/miro/reviews/all) ·
comunidade: [autorouting](https://community.miro.com/ideas/autorouting-of-connectors-9758) ·
[zoom LOD](https://community.miro.com/ideas/hide-and-reveal-elements-at-different-zoom-levels-6064) ·
[offline](https://community.miro.com/ideas/offline-access-26202) ·
[nested frames](https://community.miro.com/ideas/nested-frames-3902) · facilitação:
[facilitator.school](https://www.facilitator.school/blog/miro-sticky-notes-tricks) ·
atalhos: [keyshortcuts](https://keyshortcuts.net/miro-shortcuts), [tutorialtactic](https://tutorialtactic.com/blog/miro-shortcuts/).

**FigJam / tldraw:** [Guide to FigJam](https://help.figma.com/hc/en-us/articles/1500004362321-Guide-to-FigJam) ·
[FigJam voting](https://help.figma.com/hc/en-us/articles/9359912208663-Run-voting-sessions-in-FigJam) ·
[tldraw arrows](https://tldraw.dev/sdk-features/default-shapes) ·
[tldraw LICENSE](https://raw.githubusercontent.com/tldraw/tldraw/main/LICENSE.md) ·
[license-key](https://tldraw.dev/sdk-features/license-key).

**Excalidraw:** [LICENSE](https://github.com/excalidraw/excalidraw/blob/master/LICENSE) ·
[types.ts](https://github.com/excalidraw/excalidraw/blob/master/packages/element/src/types.ts) ·
[HelpDialog.tsx](https://raw.githubusercontent.com/excalidraw/excalidraw/master/packages/excalidraw/components/HelpDialog.tsx) ·
[export.ts](https://raw.githubusercontent.com/excalidraw/excalidraw/master/packages/excalidraw/scene/export.ts) ·
[reconcile.ts](https://github.com/excalidraw/excalidraw/blob/master/packages/excalidraw/data/reconcile.ts) ·
[json-schema](https://docs.excalidraw.com/docs/codebase/json-schema) ·
[export utils](https://docs.excalidraw.com/docs/@excalidraw/excalidraw/api/utils/export) ·
[releases](https://github.com/excalidraw/excalidraw/releases) ·
[P2P blog](https://plus.excalidraw.com/blog/building-excalidraw-p2p-collaboration-feature) ·
[E2EE blog](https://plus.excalidraw.com/blog/end-to-end-encryption) ·
[pricing](https://plus.excalidraw.com/pricing) · [Excalidraw in 2024](https://plus.excalidraw.com/blog/excalidraw-in-2024) ·
issues de desempenho [#8136](https://github.com/excalidraw/excalidraw/issues/8136),
[#7280](https://github.com/excalidraw/excalidraw/issues/7280),
[#10063](https://github.com/excalidraw/excalidraw/issues/10063) ·
HN [29109995](https://news.ycombinator.com/item?id=29109995), [47571376](https://news.ycombinator.com/item?id=47571376) ·
[roughjs LICENSE](https://github.com/rough-stuff/rough/blob/master/LICENSE) ·
[perfect-freehand LICENSE](https://github.com/steveruizok/perfect-freehand/blob/main/LICENSE) ·
[Virgil LICENSE](https://github.com/excalidraw/virgil/blob/main/LICENSE.md) ·
oráculo headless: [excalidraw-brute-export-cli](https://github.com/realazthat/excalidraw-brute-export-cli),
[mcp_excalidraw PR #114](https://github.com/yctimlin/mcp_excalidraw/pull/114).
