# TrelloClone — pesquisa e plano (W0, 2026-10-27)

> **Pedido do dono (27/10):** *«uma versão completa do Trello com todas as features profissionais
> possíveis, com a maravilhosa UI/UX e usabilidade do Trello. Pode viver como um quadro dentro do
> canvas da Board, mas pode ser aberto em aba exclusiva. O mesmo Kanban pode ser aberto a qualquer
> momento das duas formas, nos dois lugares.»* — e, a meio da pesquisa: *«vamos investir em bela
> aparência também»*.
>
> Linha: `line/TrelloClone` · worktree `Worktrees/line-TrelloClone/` · base `c3662cd8e4`.
> Este doc é o PLANO (o pensamento do módulo). Registos de sessão vão em `handoffs/`.

---

## 1 — O que o Trello é hoje (inventário da pesquisa)

Fonte: produto de referência (trello.com, out/2026) + documentação Atlassian + notas de lançamento
2024–2025. Lista **fechada** — é o conjunto de aceitação (DIRETIVA §5: alvo irrefutável não é DoD).

### 1.1 Quadro (a vista principal)
- Listas na horizontal com rolagem lateral; cartões na vertical com rolagem por lista.
- **Arrastar e soltar** de cartões (entre listas, com espaço reservado que se abre, o cartão
  **inclinado** enquanto voa, rolagem automática nas bordas) e de listas inteiras.
- Adicionar cartão no fim ou no topo da lista, adicionar lista, edição do título no próprio lugar.
- **Cores de lista** e **listas recolhidas** (recolher é da VISTA de cada pessoa, não do documento;
  atalho `\`) — lançamento de 2024.
- Menu da lista: copiar, mover (para outro quadro), mover todos os cartões, ordenar (data de criação,
  nome, data de entrega), arquivar a lista, arquivar todos os cartões, seguir, limite de cartões.
- Frente do cartão: **capa** (cor ou imagem, metade ou inteira), etiquetas (chips, mostrar texto
  ou só cor), título, distintivos (entrega com cor de estado, descrição, comentários, anexos,
  checklist `3/5`, membros em avatares, votos, localização, campos personalizados na frente).
- **Edição rápida** (`E`): o cartão abre-se no lugar com um menu lateral.
- Fundo do quadro: cores, gradientes, fotografias; estrela (favorito); título editável.
- **Arquivo** (cartões e listas arquivados, restaurar, apagar de vez).

### 1.2 Verso do cartão (o modal)
- Título, lista onde está, **seguir**.
- **Descrição** com editor rico (WYSIWYG com atalhos markdown; guarda-se markdown).
- **Etiquetas**: 30 cores (6 matizes × sutil/normal/forte + cinza…), com ou sem nome, criar/editar.
- **Membros**, **Datas** (início, entrega com hora, lembrete antes, marcar como concluída),
  **Checklists** (vários; itens com responsável e data própria; esconder marcados; converter item
  em cartão; copiar de outro cartão), **Anexos** (ficheiros, links com pré-visualização, cartões e
  quadros ligados; definir como capa), **Capa**, **Campos personalizados** (texto, número, data,
  caixa, lista suspensa; mostrar na frente), **Localização**.
- **Atividade**: comentários com `@menção`, reacções com emoji, editar/apagar, histórico de ações
  intercalado (mostrar/esconder detalhes).
- Ações: mover, copiar, **espelhar** (o mesmo cartão em vários quadros — 2025), criar modelo,
  arquivar, apagar, partilhar (link), votar.

### 1.3 Quadro como um todo
- **Filtro** (palavra, membros, etiquetas, datas: atrasado/hoje/semana/sem data, completos) e
  **pesquisa** global (`/`); destaque do que casa.
- **Modelos** de cartão e de quadro; copiar quadro; **separadores** de cartão (hack usual).
- **Atividade do quadro**; menu do quadro (sobre, fundo, etiquetas, campos, arquivo, automação,
  definições: quem comenta/adiciona, capas ligadas, votos).
- **Envelhecimento** de cartões (Power-Up clássico: o cartão parado desbota).

### 1.4 Vistas (Premium)
**Quadro · Tabela · Calendário · Linha do tempo · Painel (gráficos) · Mapa**, e as vistas de
Espaço de trabalho (tabela/calendário que cruzam vários quadros).

### 1.5 Automação (Butler)
Cinco tipos: **regras** (gatilho + condições + N ações), **botões de cartão** (no verso),
**botões de quadro** (no topo), **comandos de calendário** (intervalo fixo) e **comandos de data
de entrega** (relativos ao prazo). Sem código; sugestões de automação a partir do que se repete.

### 1.6 Pessoal (2025)
**Inbox** (captura rápida de tarefas; vindas de e-mail/Slack/Teams no Trello) e **Planner**
(calendário onde se arrasta cartões para blocos de tempo; liga a Google/Microsoft); barra de
navegação Inbox · Planner · Quadros. **Os meus cartões**, notificações, lembretes.

### 1.7 Atalhos de teclado (o idioma do Trello)
`←↑→↓`/`j k` navegar · `Enter` abrir · `Esc` fechar · `N` novo cartão abaixo · `E` edição rápida ·
`T` editar título · `D` datas · `L` etiquetas (`1`–`9` alternam etiqueta) · `M` membros ·
`Space` atribuir-me · `C` arquivar · `,` `.` mover para lista vizinha (`<` `>` para o fundo) ·
`S` seguir · `V` votar · `F` filtro · `Q` os meus cartões · `X` limpar filtros · `W` barra lateral ·
`\` recolher lista · `/` pesquisar · `?` ajuda · `@` `#` `^` completar membro/etiqueta/posição ·
`Ctrl+C`/`Ctrl+V` copiar/colar cartão · `Ctrl+Enter` guardar · `Z` desfazer.

---

## 2 — O que o PH2D já tem (medido no código, 27/10)

| Peça | Onde | Serve para |
|---|---|---|
| Contrato de aba de família `DocumentSurface` | `ph2d-editor-core/src/doc_surface.rs:170` | a aba exclusiva (precedente: `CodeDocs`, `ph2d-app-code/src/docs.rs:42`, instalado em `shells/desktop/src/init.rs:374`) |
| Quadro: `ElementKind` (Shape, Connector, Ink, Frame, Image, Embed) | `ph2d-board-model/src/element.rs:63` | o Kanban no canvas = **variante nova, no fim** (postcard posicional); molde = `Embed` (ADR-0185) |
| Ordem fraccionária `FracKey` | `ph2d-board-model/src/frac.rs:22` | ordem de listas e cartões sem renumerar (reordenar = 1 escrita) |
| Réplica por campo, carimbos Lamport, fusão comutativa | `ph2d-board-sync/src/replica.rs` (ADR-0181) | colaboração ao vivo; LAN (`ph2d-board-net`) e internet QUIC+DHT (`ph2d-board-internet`, ADR-0187) |
| Equipa, menções, votos, privado | `team.rs:111` · `facilitate.rs` · `fac.rs` · ADR-0182/0183 | membros, `@menção`, votos |
| Ficheiro por conteúdo | ADR-0184 | anexos e capas de imagem |
| `RichText` (marcas por trecho) | `ph2d-board-model/src/rich.rs` | base do editor da descrição (não é markdown) |
| Blob com versão própria no `.ph2dproj` | precedentes: Timeline, scripts `PH2DSCRIPTS`, tags | o documento Kanban sem subir `PROJECT_SCHEMA` (193) por cada mudança interna |
| Desfazer por diff `ProjectState` | `shells/desktop/src/undo.rs:42` (`capture :134`, `restore :207`, `parts_that_differ :247`) | campo `kanbans: Arc<…>` (como o Flip) |
| Widgets | `ph2d-editor-core/src/widget/` | existem: avatar, checkbox, combobox, context_menu, drag_ghost, dropdown, modal, popover, scroll_area, tag, text_area, text_input, tooltip, command_palette, card, color_picker, progress_bar |
| Notificações | `ph2d-editor-core/src/toast.rs` | só avisos dentro do app |
| Roteador de smoke do Quadro | `ph2d-app-board/src/smoke.rs:110` (`max_level: 29`) | — o Kanban terá roteador PRÓPRIO (`PH2D_KANBAN_SMOKE`) |

**Ausências confirmadas:** nenhum código de kanban/cartão/checklist no repo; **sem widget de
calendário, seletor de data, nem editor markdown**; **sem crate de datas** no `Cargo.lock` (só
`time` 0.3 transitivo); i18n só tem `Ingles` + `Teste` (a interface é inglesa, como o resto do PH2D).
**Não verificado e por medir na W1:** se duas vistas podem pintar o mesmo documento hoje (nada o
impede, nada o garante — o desenho abaixo torna-o uma garantia).

---

## 3 — A decisão central: UM documento, DUAS janelas

> ADR a escrever na W1: **ADR-0195 — «O Kanban é um documento próprio do projeto; o Quadro
> referencia-o e a aba abre-o»** — escrito na W1 como [0195](../architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md) (provisório: 0192–0194 estão em linhas por integrar).

### 3.1 O desenho
- O Kanban é um **documento de primeira classe** (`KanbanId`), guardado num **conjunto próprio**
  (`KanbanSet`) no projeto — blob com versão própria, irmão do `boards`, **nunca dentro** do blob
  do Quadro.
- No canvas do Quadro, o Kanban é **um elemento que REFERENCIA** o documento:
  `ElementKind::Kanban { kanban: KanbanId, vista: KanbanViewState }` — só o id e o estado de vista
  (zoom interno, rolagem, vista escolhida). É a mesma lei que o dono deu para imagens em 26/10
  (*«imagem = asset do projeto por referência»*).
- A **aba exclusiva** é um `DocumentSurface` da família nova `ph2d-app-kanban`, cuja aba aponta
  para o mesmo `KanbanId`.
- **As duas janelas leem e escrevem o MESMO recurso** — não há cópia, não há sincronização entre
  elas; uma edição na aba aparece no canvas no mesmo quadro de imagem, e vice-versa. Duas abas do
  mesmo Kanban, ou o mesmo Kanban em dois quadros do canvas, são o mesmo caso.
- O que é **de cada vista** (listas recolhidas, filtro activo, rolagem, vista Tabela/Calendário…)
  vive no **estado da vista**, nunca no documento — senão recolher uma lista na aba recolhia-a no
  canvas e na máquina do colega (é exactamente a regra do Trello: recolher é pessoal).

### 3.2 Gestos que ligam os dois lugares
- No canvas: botão **«Open in tab»** na barra do elemento e duplo clique no cabeçalho.
- Na aba: **«Show on board»** (leva ao quadro onde ele está; se estiver em vários, escolher) e
  **«Place on board»** (põe uma referência nova no quadro activo).
- Criar: pelo menu **Add** do Quadro (cria o documento + a referência) ou por uma aba nova
  (documento sem referência — vive só em aba até alguém o pôr num quadro).
- Apagar a referência no canvas **não apaga** o documento (como apagar um sprite não apaga a
  imagem); apagar o documento pergunta e lista onde ele aparece.

### 3.3 No canvas, o Kanban é VIVO, com nível de detalhe
- Zoom alto: o Kanban inteiro, **interactivo** (arrastar cartões, editar, abrir o verso) — o mesmo
  código de desenho da aba, numa transformação de canvas.
- Zoom médio: títulos de listas, cartões só com título e capa, sem distintivos.
- Zoom baixo: blocos de cor por lista com contagem — legível, barato.
- Tamanho do elemento = a moldura; rolagem própria dentro dela.

### 3.4 Colaboração
- O documento Kanban usa **a mesma réplica por campo** do Quadro (ADR-0181): cada cartão/lista/
  item é um registo, cada propriedade um campo com carimbo Lamport; ordem por `FracKey`, então
  dois colegas a arrastar cartões diferentes fundem sem conflito.
- Uma sessão ao vivo de um Quadro **leva os Kanbans que ele referencia**; uma aba de Kanban pode
  ser partilhada **sozinha** (link com chave, como o Quadro). Mesmo transporte (LAN + QUIC/DHT).
- Membros e menções vêm da **Equipa** já existente; votos reutilizam o modelo do Quadro.
- Consequência medida: sobe `PROTOCOL` (11 → 12) e entra uma variante no fim de `ElementKind`
  (conferir na W1 se a escada do `FORMAT_VERSION = 17` pede degrau).

### 3.5 Alternativas recusadas (e porquê)
| Alternativa | Por que não |
|---|---|
| O Kanban como **elementos soltos** do Quadro (lista = frame, cartão = nota) | não existe sem o canvas; abrir em aba duplicaria; vistas Tabela/Calendário impossíveis; ordem e arrasto do Trello não cabem em geometria livre |
| Blob do Kanban **dentro** do elemento do Quadro | dois quadros com o mesmo Kanban = duas cópias que divergem; contraria o «mesmo Kanban nos dois lugares» |
| Documento separado **fora** do projeto (ficheiro próprio) | quebra o projeto como unidade (desfazer, salvar, partilhar). Fica como exportação/importação |
| CRDT externo (automerge/yrs) | o repo já tem réplica por campo provada em LAN e internet; duas leis de fusão = duas classes de defeito |

---

## 4 — Crates (drop-crates, ADR-0075)

| Crate | Conteúdo |
|---|---|
| `ph2d-kanban-model` | `KanbanSet`, `Kanban`, `List`, `Card`, `Label`, `Checklist`, `CheckItem`, `Comment`, `Attachment`, `CustomFieldDef/Value`, `Automation`; `KanbanOp`; ordem `FracKey`; bytes com versão; zero UI |
| `ph2d-kanban-dates` | datas, prazos, recorrência, «atrasado/hoje/esta semana», fusos — sobre **`jiff`** (a escolher por `stack-audit` na W1) |
| `ph2d-kanban-md` | descrição: markdown ↔ `RichText`, sobre **`pulldown-cmark`** (licença a confirmar por artefacto) |
| `ph2d-kanban-filter` | filtro, pesquisa, ordenação, os predicados das automações (puros, testáveis) |
| `ph2d-kanban-auto` | o Butler: regras, botões, comandos agendados; executor determinístico |
| `ph2d-kanban-render` | o desenho (Vello): quadro, listas, cartões, verso, vistas; LOD do canvas |
| `ph2d-kanban-edit` | gestos: arrasto com física de mola, edição no lugar, teclado |
| `ph2d-kanban-sync` | os registos do Kanban na réplica do Quadro |
| `ph2d-kanban-import` | importar o JSON de exportação do Trello; exportar CSV/markdown/JSON |
| `ph2d-app-kanban` | a família: `DocumentSurface` (a aba), cenas de smoke, ligação ao canvas |

Foundational a tocar (aditivo, append-only): `ph2d-board-model` (variante + campo), `ph2d-board-render`
e `ph2d-board-edit` (o elemento delega no `ph2d-kanban-render/edit`), `ph2d-editor-core` (widgets
novos: **date picker, calendário, editor rico**), `ph2d-tokens` (tokens do Kanban), `ph2d-i18n`
(ficheiro irmão `kanban*.rs`), `shells/desktop` (um `install_surface`, o blob no projecto, o campo
do desfazer — **só composição**, catraca `the_shell_only_shrinks`).

---

## 5 — Bela aparência (frente própria, pedido do dono 27/10)

A usabilidade do Trello é metade aparência: **leveza, cor, movimento**. Não se entrega um pixel
sem a maquete aprovada.

1. **Maquete primeiro (W1b):** uma página de maquete (claro e escuro) com o quadro, a frente do
   cartão em todas as variantes (capa metade/inteira, etiquetas, distintivos), o verso, o filtro e
   o Kanban dentro do canvas em três zooms. **O dono aprova antes da W2.** A maquete vira a régua
   das fotos dos smokes.
2. **Tokens próprios, append-only** em `ph2d-tokens`: a paleta das **30 cores de etiqueta**
   (sutil/normal/forte — com contraste medido nos dois temas), cores de lista, superfícies do
   cartão, sombras (repouso / hover / em voo), raios, espaçamentos, tipografia do título e dos
   distintivos. Zero hex no código (HR-15).
3. **Movimento:** o espaço reservado abre com mola; o cartão em voo inclina ~3–4° e ganha sombra;
   soltar assenta com mola; a lista recolhida desliza; check de item com micro-animação; confete
   opcional ao concluir (o Trello tem). Tudo com duração em token e desligável (acessibilidade).
4. **Fundos de quadro:** cores, gradientes e fotografias (das imagens do projecto, por referência);
   a lista translúcida sobre a fotografia, legível — contraste medido sobre a superfície real
   (memória: *a cura de contraste carrega a superfície contra a qual foi calibrada*).
5. **Capas** de imagem com recorte bonito, cor dominante extraída para a capa de cor.
6. **Tema claro e escuro** desde o primeiro dia; densidade confortável/compacta.
7. **Prova:** cada onda fecha com fotografia (`fotografa_cena.sh`) lado a lado com a maquete.

---

## 6 — Ondas (cada uma = uma janela, smoke do dono no fim)

| Onda | Entrega (o que o dono consegue FAZER) | Notas técnicas |
|---|---|---|
| **W1** | — (fundação) Modelo, guardar no projeto, desfazer, ADR-0195 | `ph2d-kanban-model`, blob no `.ph2dproj`, desfazer por Kanban (não `ProjectState`, ADR-0195 §5), testes de ida-e-volta; decisão `jiff` / `pulldown-cmark` por `stack-audit` + licença |
| **W1b** | Ver e aprovar a **maquete** | §5.1; tokens do Kanban |
| **W2** | Abrir um Kanban **numa aba**, criar listas e cartões, arrastar com a sensação do Trello, cores e recolher listas, arquivo, atalhos de navegação | `DocumentSurface`, render, arrasto com mola, rolagem automática, `PH2D_KANBAN_SMOKE=1..` |
| **W3** | O **verso do cartão** completo: descrição rica, etiquetas, membros, datas, checklists, anexos, capa, campos, comentários com menção, atividade | widgets novos: seletor de data + calendário + editor rico (no `editor-core`, com showcase) |
| **W4** | O **mesmo Kanban no canvas** do Quadro, vivo, com zoom; «Open in tab» / «Show on board» / «Place on board» | `ElementKind::Kanban`, LOD, interacção no canvas, exportação Excalidraw como imagem |
| **W5** | **Ao vivo com colegas** (LAN e internet), presença no cartão aberto | registos na réplica, `PROTOCOL` 12, sessão leva os Kanbans referenciados, partilha da aba sozinha |
| **W6** | Filtro, pesquisa, modelos, **cartões espelho**, copiar/mover entre Kanbans, envelhecimento, edição rápida (`E`), todos os atalhos | `ph2d-kanban-filter` |
| **W7** | Vistas **Tabela, Calendário, Linha do tempo, Painel** (sem Mapa — §8) | calendário arrastável; Painel com gráficos pela skill de dataviz |
| **W8** | **Automação** (regras, botões de cartão e de quadro, comandos agendados e por prazo) + scripts Luau a mexer no Kanban | `ph2d-kanban-auto`; API `ph2d.kanban.*` no editor de scripts |
| **W9** | **Calendário do projecto** (§8.1), **Inbox, Planner, Os meus cartões**, notificações e lembretes | eventos próprios no `ph2d-kanban-model`; centro de notificações (hoje só há avisos) |
| **W10** | **Importar do Trello** (o JSON que o trello.com exporta) e exportar CSV/markdown | o JSON real do dono vira fixture e gate |
| **W11** | Os «Power-Ups» do PH2D: ligar um cartão a objectos da cena, scripts, macros, trechos da timeline; **nota adesiva ↔ cartão** | o que o Trello não pode ter: o cartão que abre o objecto |
| **Fecho** | Tutorial em PDF + handoff de integração | — |

Ordem: W1→W2→W3 em série (cada uma usa a anterior); W4 e W6 podem trocar; W5 depois da W4.
Pedir integração a cada 1–2 ondas (DIRETRIZ §1.5.2 item 5).

---

## 7 — Oráculos e réguas

- **O produto de referência do dono é o Trello** (memória: *o produto do dono decide a lei*): as
  capturas e os comportamentos dele medem-se — o tamanho do cartão, os tempos das animações, a
  semântica do filtro, a ordem após arrastar, o que «atrasado» significa à meia-noite.
- **Oráculos que se CORREM** (nenhum instalado hoje — `pacman -Qi` 27/10): **Wekan** (MIT) e
  **Kanboard** (MIT) por contentor, sobre entradas nossas, para leis mensuráveis (recorrência de
  datas, filtros, exportação). Triagem de licença por artefacto antes de instalar.
- **Importação do Trello:** um JSON real exportado pelo dono = fixture com cabeçalho; ida-e-volta
  é gate.
- **Gates próprios desde a W1:** ida-e-volta dos bytes · fusão comutativa (dois colegas, ordem
  qualquer, mesmo resultado) · ordem `FracKey` sem renumeração · as duas janelas mostram o mesmo
  estado depois de cada operação (aba ↔ canvas) · seam tests `ph2d-ui-testkit` em cada controlo.
- **Teto (CLAUDE.md §0.0):** quantos cartões cabem a 60 fps medimos na W2 — o limite escreve-se com
  a tabela ao lado; a meta é milhares de cartões por Kanban.

---

## 8 — Decisões de PRODUTO (do dono, 27/10)

1. ✅ **O Kanban mora dentro do projecto** (como os Quadros). Inbox, Planner e «Os meus cartões»
   atravessam os Kanbans **do projecto**, não projectos diferentes.
2. ✅ **Calendários externos (Google/Microsoft): talvez no futuro — fora desta linha.** No lugar
   deles, o dono aceitou a proposta de um **Calendário do projecto, nosso, interno** (§8.1).
3. ✅ **Sem vista Mapa** (dono 27/10: *«não precisa de mapa»*). O campo «Localização» do cartão
   fica, em texto.
4. ✅ **IA no fim** (como no editor de scripts).
5. ✅ **Ordem das ondas: a do plano** (§6) — decisão técnica delegada.

### 8.1 O Calendário do projecto (substitui o Google/Microsoft)
- Uma **aba Calendário** (mês · semana · dia · agenda) que junta **todas as datas do projecto**:
  início e entrega dos cartões de **todos** os Kanbans, itens de checklist com data, lembretes —
  e **eventos próprios** que não são cartões (reunião, marco, entrega ao cliente, feriado).
- Arrastar um cartão no calendário **muda a data dele** (é o mesmo documento, como aba ↔ canvas).
- O **Planner** é este calendário com uma coluna lateral (Inbox + cartões sem data) para arrastar
  para blocos de tempo.
- Cores por Kanban/etiqueta, filtro por pessoa; vive no projecto e viaja na sessão ao vivo.
- Porta para o futuro: exportar `.ics` (sem login) e, se o dono quiser, a ligação Google/Microsoft
  por cima do mesmo modelo.
- Entra na **W9** (com Inbox, Planner e notificações); a vista Calendário de UM Kanban (W7) é o
  mesmo widget filtrado.

---

## 9 — Fontes da pesquisa
- [Trello: expansão 2025 (Inbox, Planner, espelhos)](https://www.computerworld.com/article/3608748/trello-now-10-offers-up-an-ambitious-platform-overhaul.html)
- [Cartões espelho](https://trello.substack.com/p/mirror-mirror-on-the-wall-trellos)
- [Planner no plano Standard (2025)](https://pricingsaas.com/companies/trello/diffs/2025Q1)
- [Butler: regras, botões, comandos](https://www.atlassian.com/blog/trello/butler-power-up-trello-automation) · [guia de automação](https://trello.com/guide/automate-anything)
- [Cores de lista e listas recolhidas (2024)](https://community.atlassian.com/forums/Trello-articles/New-brighten-up-your-Trello-boards-with-list-colors-and/ba-p/2683618)
