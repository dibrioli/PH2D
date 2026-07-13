# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (2026-07-13)

> **Para o agente integrador.** A linha está **fechada e parada**. Não integrei, não pushei.
> Gate em lote verde (fmt · clippy `-D warnings` · 3 caps de LOC · typos · machete · suíte).

---

## 1. Coordenadas

| | |
|---|---|
| **Branch** | `line/motion-value` |
| **HEAD** | `787e69e0` (+ este handoff) |
| **Base** | `4cd8ef13` (= `main` no início da jornada) |
| **Commits** | 20 |
| **O que entregou** | **FILA 1 — subgrafos** + **FILA 1.b** (fio novo entra em grupo fechado) + **FILA 2 — params dirigidos por fio** + **busca no add-menu** |
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

## 9. Fila restante (o próximo da linha escolhe)

1. ~~**FILA 2 — promoção param → socket**~~ — **FECHADA** (doc 58; virou *param dirigido por
   fio*, sem estado de promoção).
2. **FILA 3 — FX passes no compositor HDR** (glow/bloom/blur/vignette/levels/hue-shift).
   Reuso obrigatório do compositor GPU do Painter (`ph2d-painter-effects`).
3. **FILA 4 — os 4 nós que faltam** (`motion-delay`, `motion-buoyancy`,
   `motion-distribute-poisson`, `motion-path`).
4. **FILA 5 — W4.T4** (dock da timeline no `motion_timeline_slot`) — coordene com o Enio, encosta
   na linha `anim`.

### Gaps conhecidos DESTA fatia (nomeados, não escondidos)

- **Um backdrop cujos nós foram agrupados fica órfão** na raiz (emoldurando nada). Ele não é
  apagado nem carregado junto — decisão consciente (um backdrop não possui nada, ele desenha em
  volta), mas dá pra questionar.
- **Reuso/instanciação** (o datablock do Blender / Gizmo / HDA) está **fora de escopo por
  desenho** (ADR §7) — exige o contrato congelado.
- **O menu do card não oferece input já alimentado por DENTRO** (doc 57 §6.1): trocar esse fio
  de fora, sem ver o que ele alimenta, é armadilha — entre no grupo e arraste a ponta. É uma
  decisão, não um esquecimento; se o Enio quiser o contrário, é uma linha no `hidden_ports`.
