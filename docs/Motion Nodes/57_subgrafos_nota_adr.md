# 57 — Subgrafos: nesting é uma DOBRA DA VISTA, sobre um grafo que continua PLANO

> **Status:** implementado (linha `line/motion-value`, 2026-07-13). Aceita as consequências
> do contrato congelado ([ADR-0039](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md))
> em vez de pedir para quebrá-lo.
>
> **O buraco que fecha:** com 86 tipos de nó na biblioteca e a neve gastando 19 cards, um
> grafo real vira uma parede. Todo editor de nós sério resolve isso com nesting; nós tínhamos
> **zero**.

---

## 1. A decisão, numa frase

**O `Graph` nunca aninha.** O documento ganha um mapa de pertencimento (nó → subgrafo) e o
**editor DOBRA**: no nível de cima, um grupo de nós desenha como **um card**; duplo-clique
entra; um breadcrumb clicável sai. O cook não sabe que subgrafos existem.

## 2. Por que — e é o contrato congelado que decide, não o gosto

O desenho óbvio (um nó `motion.subgraph` cujo `eval` cozinha um sub-grafo) é **impossível
aqui**, e as três razões são todas o ADR-0039:

| Obstáculo | Onde | Consequência |
|---|---|---|
| `NodeManifest.inputs: &'static [PortSpec]` | `ph2d-nodegraph/src/node.rs:101` | As portas de um subgrafo são **dinâmicas** (são o que cruza a fronteira dele). Uma lista `&'static` não pode ser por-instância. Um 9º campo estouraria o cap `NodeManifest = 8` **e quebraria as ~120 crates-nó de uma vez** (todas escrevem o literal em contexto const). |
| `NodeOp::eval(&self, ctx: &mut EvalCtx)` | `node.rs:141` | O `EvalCtx` não dá `Cook` nem `OpResolver`. Um nó **literalmente não consegue** avaliar um sub-grafo. `NodeOp = 2`, sem folga. |
| `Cook` memoiza em `(NodeId, ScopeKey)` | `cook.rs:334` | Um `Graph` aninhado teria `next_id` próprio → dois nós em dois ninhos compartilhariam `NodeId` e **aliasariam no memo**. |

Então o grafo fica plano e a **vista** dobra. Isso não é um consolo: é o que compra o guard
mais forte que essa feature poderia ter (§5).

**Precedente exato na indústria — a Unreal:** um *collapsed graph* é declaradamente
organizacional (*"Collections of nodes in the graph can be collapsed into sub-graphs **for
organizational purposes**"*), e a API do compilador colapsa os tunnels na compilação
(`FKismetCompilerContext::ExpandTunnelsAndMacros` — *"Expands any macro instances and
**collapses any tunnels**"*). A Epic é explícita também sobre o que isso **não** é: *"Unlike
macros, a set of collapsed nodes is **not shared** … If you copy the collapsed node, it
duplicates the internal graph. … more intended to 'tidy up' a graph … rather than any sort of
sharing or reuse."* — que é exatamente o nosso escopo.
*(Fontes: [Collapsing Graphs](https://dev.epicgames.com/documentation/en-us/unreal-engine/collapsing-graphs-in-unreal-engine) · [API FKismetCompilerContext](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Editor/KismetCompiler/FKismetCompilerContext). O "custo zero de runtime" é **inferência** a partir do compilador — a doc não afirma isso, e eu não afirmo.)*

## 3. A fronteira é DERIVADA, nunca declarada

Blender, Nuke, Houdini e Unreal todos **derivam a interface das arestas que cruzam, no momento
do collapse** — Blender: *"**Group Input and Group Output nodes will be created to represent
connections to unselected nodes outside the group.**"*; Unreal: *"**These pins are automatically
generated when the nodes are collapsed.**"*; Houdini: *"it will be **properly rewired**"*. E aí
os quatro **materializam** essa interface em nós-proxy dentro (Group Input/Output · `Input1..N` ·
indirect inputs · tunnel nodes), que passam a ser a fonte da verdade e podem ser editados.

**Nós não podemos materializar** — um proxy é um nó, e um nó precisa de manifest estático (§2).
Logo a nossa interface é **derivada CONTINUAMENTE**: puxar um fio através da costura faz o card
crescer um socket; cortá-lo tira o socket. Determinístico (ordenado por `(node, port)`), e uma
porta-fonte que alimenta dois alvos de fora é **UM** socket, não dois.

**O preço, dito na cara:** os inputs de um card colapsado estão **sempre ocupados** (eles *são*
os fios que cruzam). Então um fio NOVO para dentro de um grupo se autora **de dentro** — e é por
isso que existe o ghost.

### 3.1 Ghosts — o *indirect input* da Houdini

Dentro de um grupo, os nós **de fora** que tocam a fronteira desenham como cards **read-only,
vélados, sem hit de corpo** (não se arrasta, não se apaga) mas **com os sockets vivos**. É
literalmente a definição da Houdini: *"a node-like item that appears inside subnets and
**corresponds to the node wired into the subnet**"* — só que com o **nome do nó real**, porque
nós podemos nos dar a esse luxo.

Sem eles, um membro alimentado de fora desenharia um **socket de entrada vazio** — e isso é a
única coisa que um socket não pode fazer (é a lição do doc 45: *afordância que parece junção e
não é*).

## 4. Ids: o card tem espaço próprio, e a tag é explícita

Ids de nó e ids de subgrafo são **contadores independentes** — um documento com o nó 3 e o
subgrafo 3 é o caso comum, não o de canto. Então o **view id** de um card é `sid | 0x8000_0000`
(`SUBGRAPH_VIEW_TAG`). O **shell cunha a tag e o shell a decodifica** (`subgraph::target` /
`resolve_port`); o painel só pergunta *isto é um card?*.

**A regra que impede a mentira:** `card_ports` é **UMA** função, e ela é lida por **dois**
consumidores — a dobra (que **desenha** o socket) e o decode do intent (que **fia** o socket).
Uma segunda derivação é exatamente como um socket passa a significar uma porta diferente da que
desenhou ([[feedback_derived_coordinate_seed_must_match_sample]]).

## 5. O guard que sustenta o desenho inteiro

```
grouping_never_changes_the_cook
```
Cozinha a neve 40 ticks **plana** → agrupa a cadeia de idade pelo caminho REAL do intent →
cozinha 40 ticks de novo → **os bytes do buffer de instâncias são idênticos**.

É a afirmação inteira do desenho numa asserção, e ela é **falsificável — e foi falsificada de
propósito** (3 mutações, todas vermelhas):

| Mutação no código de produção | Gate que morreu |
|---|---|
| `group()` remove um nó do grafo | `grouping_never_changes_the_cook` |
| `group()` chama `mark_dirty()` | `grouping_never_re_cooks` |
| `card_ports` conta também as arestas internas | `a_card_exposes_exactly_the_crossing_edges` + `at_the_root_the_card_stands_in_for_its_contents` |

Se algum dia alguém "melhorar" isto transformando o subgrafo num nó de verdade, o gate fica
vermelho — e **essa mentira seria invisível em todos os outros gates**, porque todos os outros
são sobre o editor.

## 6. As três armadilhas que o desenho tinha que desarmar

1. **Um nó criado DENTRO de um grupo tem que nascer membro dele.** Senão ele cai na raiz e
   **some no ato de ser criado** — o artista adiciona um nó e nada aparece. Há 4 caminhos que
   cunham nó (add-menu, smart-connect, reroute, Ctrl+D); em vez de 4 chamadas para esquecer,
   todos passam por **UM funil** (`motion_bridge::reconcile`, que já era obrigatório para o
   `pre`-plumbing) e a adoção mora lá.
2. **O nível pode DEIXAR DE EXISTIR debaixo dos seus pés** (um Ctrl+Z que desfaz o agrupamento
   em que você está dentro). Por isso o `level` mora no **shell** (`MotionState.level`), não no
   painel: só o shell vê o undo. `clamp_level` roda a cada frame e devolve a raiz.
3. **Mover o card move o conteúdo** — em toda profundidade. Um card que se moveu sozinho faria
   o duplo-clique cair numa tela vazia a uma tela de distância do card que você acabou de
   arrastar.

## 7. O que ficou de FORA, deliberadamente

**Subgrafo REUTILIZÁVEL** (a mesma definição instanciada em vários lugares): o *datablock* do
Blender, o Gizmo do Nuke, o HDA da Houdini. Os três chegam lá por um **ato explícito de
PROMOVER** um grupo que já existe — e o Blender, que é o outlier (o grupo já nasce datablock),
paga o preço correspondente: precisa **proibir recursão** explicitamente e precisa de um
**editor de interface** desde o dia 1.

Para nós, reuso exige um segundo espaço de ids e um ponto de entrada real no avaliador — ou
seja, **o contrato congelado**. Fica para um ADR, se e quando o Enio quiser.

## 8. Superfície (para o integrador)

- **Doc:** `MotionDoc.{subgraphs, members, backdrop_members}` + seção **`[subgraph]`**
  (records `g` / `m` / `bm`), **append-only e ausente quando não há subgrafo** → um documento
  antigo serializa **byte por byte** como sempre serializou. `PROJECT_SCHEMA` **não sobe** (o
  grafo entra no projeto como `String`).
- **Foundational tocado:** `GraphKey::{Group, Ungroup}` (append no fim do enum) + `KEY_KEY_G` +
  o `graph_key_for` ganhou o parâmetro `alt`. **`Ctrl+G` / `Ctrl+Alt+G`** são os acordes que
  Blender **e** Nuke usam para exatamente estes dois verbos.
- **Contrato congelado: intacto.** `architecture_contract_surface` = 3 verdes (8/2/1).
- **Zero `GraphHitKind` novo:** o breadcrumb anda no canal `Chrome { id }` (ordinais a partir de
  `CHROME_CRUMB_BASE = 100`), que o editor-core nunca interpreta.
