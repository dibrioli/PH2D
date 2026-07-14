# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (2026-07-13)

> **Para o agente integrador.** A linha está **fechada e parada**. Não integrei, não pushei.
> Gate em lote verde (fmt · clippy `-D warnings` · 3 caps de LOC · typos · machete · suíte).

---

## 1. Coordenadas

| | |
|---|---|
| **Branch** | `line/motion-value` |
| **HEAD** | `12e4e598` (+ este handoff) |
| **Base** | `4cd8ef13` (= `main` no início da jornada) |
| **Commits** | 24 |
| **O que entregou** | **FILA 1 — subgrafos** + **FILA 1.b** (fio novo entra em grupo fechado) + **FILA 2 — params dirigidos por fio** + **busca no add-menu** + **FILA 4 (metade) — Poisson-disc + Bóia** + **NOMES no grafo (F2)** |
| **Notas-ADR** | [`57_subgrafos`](Motion%20Nodes/57_subgrafos_nota_adr.md) · [`58_params_dirigidos`](Motion%20Nodes/58_params_dirigidos_nota_adr.md) |

> **`CLAUDE.md` §5 NÃO foi tocado de propósito** — é a maior superfície de colisão do repo, e
> toda linha aberta encosta nela. A entrada de **Motion Nodes** precisa ganhar uma frase sobre
> subgrafos na integração (sugestão: *"**Subgrafos** ([doc 57](docs/Motion%20Nodes/57_subgrafos_nota_adr.md), landou 2026-07-13): nesting é uma **dobra da VISTA** — o `Graph` continua PLANO e o cook é **byte-idêntico** com/sem grupo (o contrato congelado forçou isso: `NodeManifest.inputs` é `&'static`, então porta dinâmica é impossível). Ctrl+G / Ctrl+Alt+G / duplo-clique entra / breadcrumb sai. Interface do card = as arestas que **cruzam** (derivada, não declarada — não temos nó-proxy); dentro, os vizinhos de fora são **ghosts** read-only. Reuso (datablock/Gizmo/HDA) está fora por desenho."*).

```
621f93f4 fix(motion): dois no-ops silenciosos que a auditoria propria achou (doc 57)
0d9aca5c chore(typos): allowlist pt-BR — frase/organizacional/HDA (doc 57)
deaecd86 docs(motion): nota-ADR 57 — subgrafos sao uma dobra da vista
13f816d1 feat(motion): subgrafos — nesting como DOBRA da vista, sobre um grafo que segue PLANO
```

## 2. A decisão de arquitetura, em três linhas (leia antes de qualquer conflito)

**O `Graph` NUNCA aninha.** O documento ganha pertencimento (nó → subgrafo) e o **editor
dobra**. O contrato congelado é que forçou isso: `NodeManifest.inputs` é `&'static` (portas
dinâmicas são impossíveis) e `NodeOp::eval` não recebe `Cook`/`OpResolver` (um nó não consegue
cozinhar um sub-grafo). É o *collapsed graph* da Unreal.

**Consequência que te interessa:** o cook é **byte-idêntico** com e sem grupo. Gate:
`grouping_never_changes_the_cook`. Se ele ficar vermelho depois do teu merge, **alguém quebrou
o desenho, não o teste**.

## 3. ⚠️ Foundational tocado (as superfícies de colisão)

| Arquivo | O que mudou | Risco de colisão |
|---|---|---|
| `ph2d-editor-core/src/interaction/types.rs` | **+2 variantes no fim** do `enum GraphKey`: `Group`, `Ungroup` | **Baixo** (append-only). Se outra linha também apendou, a união é trivial — mas **conte, não escolha** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). |
| `.../dispatch/keymap.rs` | **`KEY_KEY_G: u32 = 0x47`** (const nova) | Baixo |
| `.../dispatch/key.rs` | `graph_key_for` ganhou o parâmetro **`alt`** (assinatura mudou; 1 caller) | **Médio** — se outra linha mexeu em `dispatch_key`, o merge textual pode passar e o `check` pegar. |
| `.../dispatch/mod.rs` | `KEY_KEY_G` **e `graph_key_for`** no `pub use` | Baixo |
| `.../dispatch/key.rs` | **`graph_key_for` virou `pub`** e ganhou `KEY_SPACE => TogglePlay` — é agora **o único mapa** dos verbos do grafo, e o shell é seu segundo leitor | **Médio** |
| `shells/desktop/src/keymap.rs` | `KeyCode::KeyG` + um `mod tests` novo | Baixo |
| `shells/desktop/src/input_handlers.rs` | **7 arms bespoke removidas** (D/K/P/F/Delete/A/Space) e substituídas por **um portão único**; o arm do `G` (grid) ganhou `if !cmd_chord` | **ALTO se outra linha mexeu no `handle_editor_key`** — o match encolheu bastante |
| **`crates/ph2d-editor-core/tests/hr12_widgets_a11y.rs`** | **+1 entrada** na `PANEL_A11Y_DELEGATE_OK` (`paint_menu.rs`) | **ALTO se outra linha também adicionou** — mesma armadilha da allowlist: **funda a união, não escolha um lado** |
| `.../dispatch/tests/graph.rs` | `key_cmd` virou wrapper de `key_chord(kc, cmd, alt)`; +1 caso e +1 teste | Baixo |
| **`.typos.toml`** | **+3 palavras** (`frase`, `organizacional`, `HDA`) na `[default.extend-words]` | **ALTO se outra linha também adicionou** — chave duplicada **mata o TOML no parse** e o typos nem escaneia ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse.md]]). **Funda a união, sem duplicar chave.** |

| **`crates/ph2d-nodegraph/src/graph.rs`** | **campo novo `param_sources`** no `Graph` + 4 acessores; **`would_cycle` e `remove_node` estendidos** (doc 58) | **MÉDIO** — o `Graph` é a foundational mais quente do repo. As mudanças são **aditivas** (campo novo, fns novas), mas as DUAS fns existentes ganharam corpo: se outra linha as tocou, resolva **pelos estágios do índice**, e confira que o walk do ciclo continua vendo os dois tipos de dependência. |
| **`crates/ph2d-nodegraph/src/cook.rs`** | `EvalCtx.driven` (campo novo) · `param()` resolve **fio > override > default** · `cook_node` resolve as fontes na mesma recursão · +1 campo no `Fingerprint` | **MÉDIO** — mesmo raciocínio. **Um merge que perca o campo do fingerprint compila e devolve número velho pra sempre** (o gate `re_pointing_a_param_to_another_port_of_the_same_driver_recomputes` é quem pega). |
| `crates/ph2d-nodegraph/src/format.rs` | header **`v3`** + record **`d`** (aditivo, ausente quando não há param dirigido) | Baixo |
| `crates/ph2d-nodegraph/src/attr.rs` | **`VALUE_COLUMN`** (a coluna `"v"`, que era const privado em ~30 crates de nó) | Baixo |
| `crates/ph2d-editor-core/src/widget/mod.rs` | **`format_number`** no `pub use` (já existia; só não era exportado) | Baixo |
| `crates/ph2d-panel-motion-params/src/snapshot.rs` | **`ScalarRow.driven`** (campo novo — toda construção precisa dele) | Médio |
| **`.../interaction/dispatch/key.rs`** + `state/{mod,store_core,store_hierarchy}.rs` | **`cancel_on_escape` virou FLAG por widget** (`mark_cancel_on_escape`) — a lista hardcoded `id == HIER_RENAME_INPUT \|\| id == TIMELINE_MARKER_RENAME_INPUT` MORREU | **MÉDIO** — os dois ids agora se marcam (em `screens/hero.rs` e `ph2d-panel-timeline/src/marker_rename.rs`). Se outra linha adicionou um 3º id àquela expressão, **ele tem que virar um `mark_cancel_on_escape` também** ou o Esc dele para de cancelar (silenciosamente). |
| `crates/ph2d-editor-core/src/widget/mod.rs` | `format_number` exportado | Baixo |
| `crates/ph2d-ui-testkit/src/lib.rs` + `Cargo.toml` | **`paint_with_layout`** + **`dispatch_pointer_event`** (+ deps `ph2d-host`, `bumpalo`) | Baixo (aditivo) — mas **leia o §7.5**: sem eles, nenhum painel do split podia ter gate de paint |

**Contrato congelado: INTACTO.** `architecture_contract_surface` = 3 verdes (8/2/1).
`IconId` **não** mudou (o `IconId::Group` e o `docs/design/icons/group.svg` já existiam).
**Zero `GraphHitKind` novo** — o breadcrumb anda no canal `Chrome { id }` que já existia.

## 4. Ids / consts novos (para o integrador detectar colisão)

- `ph2d_panel_motion_graph::SUBGRAPH_VIEW_TAG = 0x8000_0000` — bit que marca um view id como
  **card** (os espaços de id de nó e de subgrafo são independentes).
- `paint_chrome::CHROME_GROUP = 6` (7º chip da toolbar) e **`CHROME_CRUMB_BASE = 100`** (crumb
  `i` = `100 + i`). Se outra linha adicionou chip na mesma toolbar, **o ordinal 6 colide** —
  renumere o dela ou o meu; os dois são locais ao painel.
- Records novos do formato textual: **`g` / `m` / `bm`**, na seção nova **`[subgraph]`**.

## 5. Crates / arquivos novos

**Nenhuma crate nova.** 8 arquivos novos, todos módulos irmãos (LOC cap):

```
crates/ph2d-motion-doc/src/subgraph{,_tests}.rs          modelo + travessias + formato
crates/ph2d-panel-motion-graph/src/interact_subgraph{,_tests}.rs   gestos
crates/ph2d-panel-motion-graph/src/interact_key.rs       (split: apply_key saiu de interact.rs)
crates/ph2d-panel-motion-graph/src/snapshot_intent.rs    (split: GraphIntent saiu de snapshot.rs)
crates/ph2d-panel-motion-graph/src/paint_breadcrumb.rs   breadcrumb
crates/ph2d-panel-motion-graph/src/paint_menu.rs         (split: draw_add_menu saiu de paint.rs)
shells/desktop/src/render_loop/motion_bridge_fold.rs     A DOBRA (a vista)
shells/desktop/src/render_loop/motion_bridge_subgraph{,_tests}.rs  os verbos
shells/desktop/src/render_loop/motion_bridge_intents.rs  (split: apply_graph_intents saiu do bridge)
```

**4 splits foram forçados pelo cap de LOC** (`interact.rs`, `snapshot.rs`, `paint.rs`,
`motion_bridge.rs` estouraram). Se outra linha editou esses 4 arquivos, **o merge textual vai
doer** — as funções que ela mexeu podem ter MUDADO DE ARQUIVO. Confira por símbolo, não por
linha.

## 6. Mudanças de assinatura (o `check --workspace` pega; o merge-tree não)

| Símbolo | Antes | Agora |
|---|---|---|
| `hits::push_card_hit` | `(hits, node: u32, body, canvas)` | `(hits, n: &GraphNodeView, body, canvas)` |
| `paint_chrome::draw_split_chrome` | 7 args soltos | `(…, state: ChromeState)` (struct nova) |
| `interact::apply_key` | `(state, k, rect)` | `(state, k, rect, snap)` — e **mudou de arquivo** |
| `edit::duplicate` | `-> Vec<u32>` | `-> BTreeMap<NodeId, NodeId>` (o mapa origem→cópia) |
| `motion_demo_strobe::build` | `-> Option<Vec<NodeId>>` | `-> Option<Demo>` (struct nova) |
| `plumbing::reconcile_after` | (inalterada) | mas os **7 call-sites** agora passam por `motion_bridge::reconcile` |
| `GraphNodeView` | — | **+campo `kind: NodeViewKind`** (toda construção precisa dele) |
| `GraphViewSnapshot` | — | **+campos `level` e `breadcrumb`** |
| `state::AddMenu` | struct c/ `connect_from` | **`Menu` + `MenuBody::{Library, CardPorts}`** (renomeado — o nome mentia) |
| `geom::add_menu_*` | 7 fns | **`geom::menu_*`** (mesmo rename) |
| `paint_menu::draw_menu` | `(ctx, menu, canvas, theme)` | `(ctx, menu, snap, canvas, theme)` |
| `interact_menu::{scroll,grab,drag}_menu*` | sem `snap` | **+`snap`** (a contagem de linhas vem de UMA fonte) |
| `snapshot::menu_catalog` | usada pelo hit | **+`menu_rows`** — a fonte ÚNICA (paint E hit) |

## 7. O que só o `ship.sh` pega (conte com 2–4 iterações)

Rodei local: **fmt (pinado 1.95) · clippy `-D warnings` · typos · machete · os 3 caps de LOC ·
`architecture_contract_surface` · a suíte das 4 crates tocadas** — todos verdes.
**Não** rodei: `cargo deny`, `cargo audit`, nextest com `--cargo-profile ci-test`, matrix
macOS/Windows. Não adicionei dependência nenhuma, então `deny`/`audit` não deveriam mexer.

## 7.1 Correções do 1º smoke do Enio (2026-07-13, commit `76e1584c`)

O Enio achou 4 coisas, e as duas primeiras eram a mesma **dívida estrutural**:

1. **`Ctrl+G` desativava o GRID** em vez de agrupar. O shell mantinha uma **segunda lista** dos
   verbos do grafo, escrita à mão (7 arms), e o grafo cresceu um verbo que essa lista nunca
   ouviu falar — o chord caiu no `G` global. O mesmo arquivo **já documentava esse bug** de
   2026-07-12 (*"Ctrl+D não duplica"*) e o tinha consertado **adicionando mais uma arm**. A
   oitava arm seria a mesma dívida pela terceira vez.
   **Fix estrutural:** existe **UM mapa** (`graph_key_for`, agora `pub`) e o shell é seu único
   outro leitor. Portão único: cursor sobre o grafo ⇒ o grafo é dono da tecla, e **consome**.
   Gate novo (`every_graph_verb_is_reachable_through_the_shells_normalizer`, mutation-tested)
   teria pego isso antes do smoke.
2. **Um CHORD nunca pode acertar um atalho de letra solta** — o arm do grid virou
   `KeyCode::KeyG if !cmd_chord`. Isso valia pro **app inteiro**, não só pro grafo.
3. **O chip de agrupar** agora é **um botão com os dois verbos**, e veste o ícone do verbo que
   vai de fato executar (card selecionado ⇒ `IconId::Ungroup`).
4. **Os ghosts são ARRASTÁVEIS.** A teoria de que mover um ghost mexeria numa tela que você não
   está vendo estava errada — um nó tem **uma** posição, e quem olha pro ghost está olhando pro
   nó. Apagá-lo de lá continua **refusado, com toast**.

## 7.2 FILA 1.b — o fio novo entra no grupo fechado (commit `6207bd68`)

O gap que eu mesmo tinha nomeado no §9 desta folha. Fechado na mesma sessão (§0.6 — gaps
in-scope fecham agora). **Detalhe do desenho: [doc 57 §6.1](Motion%20Nodes/57_subgrafos_nota_adr.md).**

Soltar um fio no **corpo** de um card abre um menu com as **portas escondidas** lá dentro
(as que nenhum fio alcança), filtrado pelo que esse fio pode alimentar. Na escolha sai um
`Connect` **ordinário** pra porta real — e **o card ganha o socket por derivação**. Vale nos 3
gestos (fio pra frente · fio pra trás · ponta arrastada → `MoveWireEnd`).

**⚠️ Bug de produção corrigido junto (F2 smart-connect, pré-existente):** o `draw_menu`
desenhava `current_catalog()` (86 tipos) enquanto o clique resolvia contra a lista **filtrada**
— o artista lia uma linha e apertava outra, e a altura/scrollbar do popup eram de uma lista que
ninguém via. **Se outra linha encostou no add-menu, é aqui que o merge dói** (o popup foi
reescrito pra ter UMA fonte de linhas).

## 7.3 FILA 2 — params dirigidos por fio (`cbc62a5b` motor + `76551888` editor)

**Detalhe: [doc 58](Motion%20Nodes/58_params_dirigidos_nota_adr.md).** O plano tinha deferido
isto **duas vezes** dizendo *"exige porta dinâmica no modelo"* — e porta dinâmica é de fato
impossível (`NodeManifest.inputs` é `&'static`). Mas **a porta nunca foi o requisito**: o
requisito é uma **ARESTA que o manifesto não conhece**, e aresta é estado de DOCUMENTO — que é
onde o canal de text param já mora. Mesmo truque, segundo uso.

- `Graph.param_sources` + `drive_param`/`undrive_param`. O cook resolve na mesma recursão dos
  inputs; `EvalCtx::param` = **fio > override > default**. **Os 86 tipos de nó ficaram
  dirigíveis sem uma linha de mudança em nenhum deles** (todos leem param por esse funil).
- **Não existe estado "promovido"**: o fio É a promoção (drop no corpo do nó → menu dos params
  → o socket aparece porque o fio existe). Mesma lei e mesma máquina do doc 57 §6.1.
- **Um param dirigido vira read-only no painel** e mostra o número vivo que o fio põe.

**⚠️ Para o integrador:** este é o único pedaço da linha que mexe em `ph2d-nodegraph` (a
foundational mais quente). Tudo é aditivo, mas `would_cycle`, `remove_node`, `cook_node` e
`Fingerprint` ganharam corpo — veja a tabela do §3.

## 7.4 Busca no add-menu (`cc42f47c`, doc 59)

Smoke do Enio: *"não encontrei value.lfo"* — 86 tipos numa lista plana. (E o nome que EU dei
era o canônico; o menu só mostrava o rótulo `LFO`. Isso é parte do bug.)

O menu abre com um campo que **já tem o teclado**: aperta `A` e digita. Casa nos **dois nomes**
(rótulo + canônico → o **domínio** vira query: `value` lista os `value.*`), é **subsequência**
(`mr` → Map Range) e é **ranqueada** — filtrar sem ranquear devolve a mesma lista que ele já
tinha. Enter escolhe o primeiro, Esc fecha.

**Dívida morta de brinde:** `cancel_on_escape` era uma lista de ids hardcoded dentro do
`dispatch_key`. Virou flag por widget. Ver §3.

## 7.5 O clique que deslizava (`787e69e0`) — e o gate que faltava no repo INTEIRO

Smoke do Enio: *"não consigo inserir nenhum nó ao clicar no menu"*.

**O dispatcher chama de `End` (arrasto), não de `Click`, qualquer press-release com QUALQUER
movimento entre eles** — e mão humana sempre mexe. Um pixel de deslize e a linha que o artista
apertou virava arrasto: o menu se dispensava e nada era adicionado. **Não é regressão da
busca** — o bug existia desde que o menu existe.

**E todo teste desta crate mandava Down e Up na MESMA coordenada**, a única coisa que uma mão
de verdade nunca faz. Verde em tudo, inusável na mão. Lição:
[[feedback_a_click_is_a_press_that_drifted]].

**Fix:** com menu aberto, o ponteiro pertence ao MENU — onde o botão SOBE é o que vale (sobre a
linha, escolhe; fora, dispensa). O foco do campo se assenta num lugar só (`settle_focus`): abriu
pega o teclado, fechou devolve (senão todo atalho do editor digitaria num buffer invisível).

**Testkit (aditivo, e destrava gates futuros):** `paint_with_layout` — o `for_viewport` monta o
centro **sem split**, então o painel do grafo tinha rect **zero** e a pintura retornava antes de
desenhar: **por isso ele nunca teve um gate de paint**. Mais `dispatch_pointer_event` (caminho
`_with_text`, o do shell). O gate novo (`tests/the_add_menu_actually_adds_a_node.rs`) **pinta**,
lê o hit index que a **pintura** registrou, e despacha `PointerEvent` de verdade.

## 7.7 Nomes no grafo — F2 (`5b80e36d`, doc 61)

**O gesto que faltava.** 88 tipos de nó, vinte cards, seis dizendo `Move`/`Drive`: o grafo se lia
rastreando fios. Agora **F2** nomeia o **card**, o **grupo** ou o **backdrop** selecionado.

- ⚠️ **CORREÇÃO (mesmo dia, doc 61 §2):** eu afirmei que o rename do backdrop/grupo "nunca existiu".
  **Falso.** O **painel de params** já tinha as linhas **Title**/**Color** do backdrop e a linha
  **Name** do grupo — por um canal de intent *diferente*. Os `GraphIntent::SetBackdropTitle/Color`
  eram **duplicatas mortas**, não a ausência da feature. O que o F2 realmente traz: **o label do NÓ**
  (isso sim não existia em canal nenhum — 88 tipos, vinte cards, e um nó não podia ser nomeado) **+ o
  gesto inline** (nomear deixa de ser "vá olhar noutro painel"). Lição em
  [[feedback_stale_comment_and_dead_code_lie]] (3º caso): **cace a CAPACIDADE, não o símbolo**, e
  responda **executando**.
- **Foundational (`ph2d-nodegraph`):** campo **`node_labels`** no `Graph` — **mapa paralelo**, não
  campo no `NodeInstance` (append-only = merge que não conflita; campo novo tocaria todo sítio de
  construção do repo). Record **`t <id> <label…>`** + header **`v4` só quando alguém nomeou**
  (ausente = byte-idêntico). É a **3ª volta da mesma manivela** (text-param doc 32, param dirigido
  doc 58).
- **`GraphKey::Rename`** (append no fim do enum) + **`KEY_F2 = 0xF705`** + `KeyCode::F2` no shell.
- **`SetBackdropTitle` REMOVIDO** → virou `Rename { target: RenameTarget, name }` (um gesto, três
  alvos, um undo). ⚠️ **Se outra linha passou a emitir `SetBackdropTitle`, ela vira `Rename`.**
- **`graph.rs` estourou o cap de 700 LOC** do workspace → **split** em `graph_tests.rs` (nunca
  allowlist).
- **Demo:** os **19 cards do boot ganharam nome** — o grafo da neve virou uma frase.
- **Gate que PINTA e DIGITA** (`f2_actually_renames_the_thing`), irmão do gate do add-menu. 4
  mutações mortas.
- **Aberto (nomeado):** a **cor** do backdrop segue sem gesto (`SetBackdropColor` vivo e sem
  emissor; o tom só cicla por id ao nascer).

## 8. Smoke (o Enio)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
```
Abra o Motion. **A neve tem que cair exatamente como antes** — e no meio da cadeia agora há um
card só, **"Age & Fade"**, com uma pilha desenhada atrás dele e o rótulo "6 nodes".

- **Duplo-clique nele** → você entra. O breadcrumb "Root / Age & Fade" aparece no canto
  superior esquerdo; os vizinhos de fora (o collide e o falloff) estão lá, **velados** — são
  ghosts, não dá pra arrastar nem apagar, mas dá pra puxar fio deles.
- **Clique em "Root"** → você volta.
- **Selecione uns nós e Ctrl+G** (ou o 7º chip da toolbar) → viram um card. **Ctrl+Alt+G**
  desfaz. Selecione o card → o painel de params mostra o nome, editável.
- **Ctrl+Z** desfaz o agrupamento (e se você estiver DENTRO do grupo quando desfizer, o editor
  te devolve pra raiz em vez de te deixar numa sala que não existe mais).
- **Desagrupar deixa o conteúdo SELECIONADO** (smoke #2 do Enio) — a mão do artista não esvazia.
- **Puxe um fio de um nó de fora e SOLTE EM CIMA do card** → abre "Connect Inside Group" com as
  portas livres lá dentro; escolha uma e **o card ganha um socket novo** (ninguém declarou
  interface — o fio *é* a interface). Funciona também puxando pra trás de um input vazio.
- **BUSCA:** aperte `A` e **digite** — o campo já está focado. `lfo` acha o **LFO** (o nome que
  o menu mostra); `value.lfo` acha também (o nome que a doc fala); `value` lista o domínio
  inteiro. **Enter** pega o primeiro.
- **FILA 2:** adicione um **LFO** (menu `A` → digite `lfo`), puxe o fio da saída dele e **solte no CORPO
  de qualquer nó** → abre "Drive Parameter" com os params daquele nó. Escolha um (ex.: a
  Strength de um `force.wind`) → **o nó ganha um socket novo**, o fio pousa nele, e o param
  **oscila**. No painel de params aquela linha vira **read-only mostrando o número vivo**.
  Corte o fio (faca `K`, ou arraste a ponta pra fora) → **o socket some** e o knob volta.

- **FILA 4 (doc 60):** o boot já abre com ela — **a neve cai no MAR**. Olhe: os flocos caem,
  **atravessam** a superfície, **batem no leito** e a água os traz de volta pra **boiar e balançar**
  na marola enquanto derretem. Os sítios de nascimento agora são Poisson (a fileira é irregular, não
  um pente). Para brincar: selecione o card **Buoyancy** e mexa em `Level` / `Wave Amplitude` /
  `Density` (menor que a gravidade = o floco AFUNDA — é uma pedra, não um bug); ou puxe um
  **`value.lfo`** pro corpo dele e dirija o `Level` (a maré). O card **Poisson Disk** tem `Radius`:
  aumente e nascem menos sítios, mais separados.

- **Doc 61 (nomes):** o boot já abre com os cards **nomeados** ("The Sea", "Birth Sites", "The Kill
  Disc"…). Clique num card e aperte **F2** → a caixa abre **sobre o título**, já com o nome dentro,
  selecionado; digite e **Enter**. **Esc** mantém o antigo. Funciona igual no **card do grupo**
  ("Age & Fade") e no **backdrop** (clique na faixa do topo dele primeiro). Apague o nome e dê Enter
  → o card volta a se chamar do que ele é. **Ctrl+Z** desfaz o rename num passo só. E depois de
  fechar a caixa, **`A` volta a abrir o menu** (o teclado volta pro grafo).

## 9. Fila restante (o próximo da linha escolhe)

1. ~~**FILA 2 — promoção param → socket**~~ — **FECHADA** (doc 58; virou *param dirigido por
   fio*, sem estado de promoção).
2. ~~**FILA 4 — Poisson + Bóia**~~ — **FECHADA a metade** (doc 60): `motion.distribute_poisson`
   (Bridson 2007) + `force.buoyancy` (Arquimedes + onda viajante). Ver §7.6.
2.b ~~**Nomes no grafo**~~ — **FECHADO** (doc 61, F2). Ver §7.7.
3. **FILA 3 — FX passes no compositor HDR** (glow/bloom/blur/vignette/levels/hue-shift).
   Reuso obrigatório do compositor GPU do Painter (`ph2d-painter-effects`). ⚠️ **Cross-module: PARE
   e reporte ao Enio antes de começar** — é decisão de arquitetura, não fan-out.
4. **FILA 4, a outra metade** — `motion.delay` (fan-out normal; a família *History* do plano §1.3.
   Note antes de codar: `motion.time_remap` já atrasa uma sub-árvore **Pure** de graça, então o
   `delay` só se justifica pra atrasar o que **não é função de t** — uma simulação) ·
   **`motion.path` = PARADO, precisa de DECISÃO do Enio** (o plano dizia *"integra vector.*"*, mas o
   sistema vetorial de nós foi RETIRADO por ADR-0108 e a geometria mora em `ph2d-vec-scene`, que o
   **cook não alcança**: um nó só recebe params/inputs/playhead. Um nó que lê o documento vetorial
   exige um **canal novo shell→cook** — arquitetura, não fan-out).
5. **FILA 5 — W4.T4** (dock da timeline no `motion_timeline_slot`) — coordene com o Enio, encosta
   na linha `anim`.

## 7.6 FILA 4 (metade) — Poisson + Bóia (`12e4e598`, doc 60)

**2 drop-crates novas, ZERO foundational tocado.**

| Nó | O que é | Referência |
|---|---|---|
| `motion.distribute_poisson` | **o raio é o knob, a contagem é a resposta** — preenche um retângulo com pontos que nunca ficam a menos de `radius` um do outro | **Bridson 2007** (`O(N)`, grade de aceleração). NÃO é um 2º `motion.scatter`: aquele é Mitchell best-candidate, `O(N²K)`, e nomeia a **contagem** |
| `force.buoyancy` | **Arquimedes + uma onda viajante** — acumula em `accel` como toda força; o empuxo é normal à SUPERFÍCIE (então o flutuante cavalga a marola, não bombeia no lugar) | **`BuoyancyEffector2D` da Unity** (level/density/drag), com a superfície promovida a onda |

- **HR-5:** a direção do dardo do Bridson é **rejeitada da bola unitária** (o paper diz "ângulo
  aleatório" = `sin`/`cos`, proibido); a onda é o seno parabólico (leaf copiado do `force.wind`).
- **Teto sem `count`:** um nó sem param de contagem não tem `param_as_count` — quem vira o vetor de
  alocação é o **raio** (0 → grade infinita, e o cast `f32 as usize` **satura**, não entra em
  pânico). O teto é a **grade** (uma célula de Bridson guarda ≤1 ponto, então limitar células limita
  memória E contagem).
- **A demo mudou: a neve cai NO MAR.** Os 2 nós entraram **dentro** do grafo da chuva (Enio: *"deixe
  só o grafo da chuva"*), não numa 2ª cena. O mar é **raso de propósito** — o floco atravessa a
  superfície, **bate no leito** (`sim.collide` segue portante) e a água o traz de volta. Medido:
  queda 1,45 s → mergulho até exatamente o leito → **1,3 s boiando**; população estável ~73.
- **Bug pré-existente que a demo expôs:** a faixa de nascimento era **mais larga que a região viva**
  do disco de kill na altura dela — os sítios das pontas **nasciam mortos**. Corrigido (3,2 de
  largura; meia-largura viva a `y=2,6` é 1,92).
- **`ph2d-node-registry-init` REGENERADO** (`cargo run -p ph2d-node-sync`) — **88** crates-nó (era
  86). É o conflito de merge esperado no rebase: **regenere, nunca resolva à mão.**
- **Shell:** `motion_demo_strobe.rs` + `motion_state_tests.rs`. **5 constantes da demo viraram
  `pub(crate)`** (`SEA_LEVEL`/`SEA_DRAFT`/`SEA_WAVE_AMP`/`SNOW_FLOOR_Y`/`RAIN_Y`) — o gate do chão
  tinha um **número mágico duplicando a constante** (`-2.0 + 2.4` à mão) e, quando o leito se moveu,
  seguiu apontando pra água vazia **verde**. Agora ele lê a constante.
- **Contrato congelado:** intocado (`architecture_contract_surface` 8/2/1 verde).
- **3 mutações mataram os gates** (empuxo reto pra cima · submersão binária · varredura de vizinhos
  5×5→4×4).

### Gaps conhecidos DESTA fatia (nomeados, não escondidos)

- **Um backdrop cujos nós foram agrupados fica órfão** na raiz (emoldurando nada). Ele não é
  apagado nem carregado junto — decisão consciente (um backdrop não possui nada, ele desenha em
  volta), mas dá pra questionar.
- **Reuso/instanciação** (o datablock do Blender / Gizmo / HDA) está **fora de escopo por
  desenho** (ADR §7) — exige o contrato congelado.
- **O menu do card não oferece input já alimentado por DENTRO** (doc 57 §6.1): trocar esse fio
  de fora, sem ver o que ele alimenta, é armadilha — entre no grupo e arraste a ponta. É uma
  decisão, não um esquecimento; se o Enio quiser o contrário, é uma linha no `hidden_ports`.
