# 61 — Nomes no grafo (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-13).
> **Gesto novo:** **F2** renomeia o card, o grupo ou o backdrop selecionado. Contrato congelado
> **intocado**.

## 1. O buraco

Um grafo com **88 tipos de nó** e vinte cards, seis dos quais dizem `Move`, `Move`, `Drive`,
`Drive`, é um grafo que você lê **rastreando fios**. Todo editor de nós que espera segurar um
documento de verdade deixa nomear os cards — Blender (F2), Houdini, Nuke, TouchDesigner, Cavalry —
e todos usam **a mesma tecla**.

Nós não tínhamos — **mas leia a §2 antes de acreditar na força dessa frase**: o backdrop e o grupo
já podiam ser nomeados pelo painel de params. O que não existia por caminho NENHUM era o nome do
**nó**, e o que não existia em lugar nenhum era o **gesto**.

## 2. ⚠️ CORREÇÃO (2026-07-13, mesmo dia) — a §2 original desta nota estava ERRADA

A primeira versão deste documento afirmava que *"dois dos três já tinham onde guardar o nome e nada
no editor conseguia escrever nele"*. **É falso, e o erro é meu.**

Eu greppei o `GraphIntent::SetBackdropTitle`, não achei emissor, e concluí que a feature nunca tinha
sido construída. **Ela existia por OUTRO caminho:** o **painel de params** já tinha, o tempo todo, as
linhas **Title** e **Color** do backdrop (`motion_bridge_backdrops::params_snapshot` +
`apply_param_intent`, num canal de intent *diferente*), e o card de grupo já tinha a linha **Name**
(`motion_bridge_subgraph::params_snapshot`). Provado **rodando o seam** e imprimindo as rows — depois
de eu já ter afirmado o contrário duas vezes por escrito.

O que estava de fato morto eram **os dois `GraphIntent`** (`SetBackdropTitle`/`SetBackdropColor`):
handlers sem emissor, **duplicatas** de um caminho vivo — não a ausência da capacidade.

**O que o F2 então REALMENTE trouxe:**

1. **O label do NÓ — isso sim não existia.** O `Graph` não tinha canal nenhum pra um nome de nó, e o
   painel de params de um nó mostra os *params dele*, não um nome. Um nó **não podia** ser nomeado
   por caminho nenhum. É a metade nova de verdade, e é a que importa: são 88 tipos e vinte cards.
2. **O gesto inline.** Nomear deixa de ser *"selecione, vá olhar em outro painel, ache a linha
   Title"* e passa a ser **F2 sobre a coisa** — que é como todo editor de nós faz, e é a diferença
   entre uma capacidade e uma ferramenta.

Lição registrada em [[feedback_stale_comment_and_dead_code_lie]] (3º caso): **cace a CAPACIDADE, não
o símbolo** — *"quem emite X?"* e *"o usuário consegue fazer X?"* são perguntas diferentes, e só a
segunda importa. E responda-a **executando**, nunca por grep.

## 3. O canal do nó: mais uma volta da mesma manivela

`NodeInstance` tem `id` e `type_name`. O label **não** virou um campo dele — virou um **mapa
paralelo** no `Graph` (`node_labels: BTreeMap<NodeId, String>`), exatamente como o
[text param (doc 32)](32_expression_text_param_channel_nota_adr.md) e as
[fontes de param (doc 58)](58_params_dirigidos_nota_adr.md), e pela mesma razão:

**o `ph2d-nodegraph` é a foundational mais quente do repo, e várias linhas a estendem ao mesmo
tempo.** Um mapa append-only é um merge que nunca conflita; um campo novo na struct toca **todo
sítio de construção** do repositório ([[feedback_foundational_editable_design_for_isolation]]).

**Formato:** record **`t <id> <label…>`**, campo final **free text** (como o `x`) — *"The Sea"* é
**um** nome, não dois. Header **`v4` só quando alguém nomeou alguma coisa**; um grafo que ninguém
renomeou serializa **byte por byte** como sempre. E um nome que você pôs e depois apagou **não deixa
rastro** no arquivo (nome vazio = sem label = volta pro `v1`).

**Um label não é semântico** (nenhum cook o lê) — e mesmo assim fica **acima** do `[layout]`. A
divisão não é *semântico × cosmético*: é **o que o EDITOR decide** (onde o card senta) × **o que o
ARTISTA decide** (como a coisa se chama). Renomear um nó pertence a um diff; empurrar o card não.

## 4. Um gesto, três alvos

`GraphIntent::Rename { target: RenameTarget, name }` — **um** intent, **um** passo de undo, e o
`RenameTarget::{Node, Subgraph, Backdrop}` diz em qual dos três espaços de id o nome pousa.

Não é cerimônia de tipo: **um documento tem rotineiramente um nó 3, um subgrafo 3 E um backdrop 3 ao
mesmo tempo** (os espaços são independentes — é por isso que o card do grupo carrega o
`SUBGRAPH_VIEW_TAG`). Um `u32` cru seria **cara ou coroa** sobre o que você acabou de renomear. O
gate `the_same_id_in_three_spaces_renames_the_right_one` monta exatamente essa cena.

O `SetBackdropTitle` **morreu** — não porque a feature não existia (§2: existia, pelo painel de
params), mas porque, como `GraphIntent`, ele era **um terço de um gesto** e **ninguém o emitia**.

## 5. As decisões pequenas que fazem a caixa não irritar

- **A caixa nasce com o nome que a coisa TEM** (não vazia), e **selecionado por inteiro** — o
  primeiro caractere substitui, mas o nome velho está ali se você só queria apendar. É a convenção do
  Finder, do Explorer, do Blender.
- **A semente sai do MESMO snapshot que a pintura** — o que você edita é a string que você está
  vendo ([[feedback_derived_coordinate_seed_must_match_sample]]).
- **Nome vazio não é nome:** limpar a caixa significa *"chame do que ela é"* (o card volta pro tipo,
  o grupo pro título default), não *"deixe em branco"*.
- **Enter numa caixa que você não editou não é uma edição** — sem isso a fila de undo enche de passos
  que não mudam nada e o artista aperta Ctrl+Z três vezes pra desfazer uma coisa.
- **F2 com nada (ou com muita coisa) selecionado é INERTE.** Renomear é uma pergunta sobre **um**
  nome; apagar muitos é significativo, nomear muitos não é.
- **O teclado VOLTA.** Esse painel já enviou um campo que ficava com o foco depois de fechar — a
  partir dali `A` não abre o menu, digita um "a" num buffer invisível. O foco se assenta **num lugar
  só** (`settle_focus`), qualquer que tenha sido a saída (Enter, Esc, o alvo sumir).

## 6. Os gates

Além dos unitários, **o gate que pinta e digita de verdade**
(`tests/f2_actually_renames_the_thing.rs`) — irmão do `the_add_menu_actually_adds_a_node`, e existe
pelo mesmo motivo: **o último widget que este painel ganhou tinha teste unitário e nenhum teste de
PINTURA, e embarcou uma caixa em que não dava pra clicar** ([[feedback_a_click_is_a_press_that_drifted]]).
Ele pinta o painel no layout REAL (o centro **split** — sem isso o rect do grafo é **zero** e a
pintura retorna antes de desenhar), aperta F2 de verdade e lê o canal de intents.

**4 mutações mataram os gates:** o foco que não volta · F2 renomeando o primeiro de uma
multi-seleção · o card de grupo resolvendo como nó (os espaços de id colapsando) · a caixa abrindo
vazia.

## 7. Superfície (para o integrador)

| Arquivo | O quê | Risco de merge |
|---|---|---|
| `ph2d-nodegraph/src/graph.rs` | campo **`node_labels`** + `set_label`/`label`/`node_labels`; `remove_node` limpa | **Médio** — aditivo, mas é o `Graph` |
| `ph2d-nodegraph/src/graph_tests.rs` | **arquivo novo** — os testes saíram do `graph.rs` (782 LOC, **cap 700**). Split, nunca allowlist | Baixo |
| `ph2d-nodegraph/src/format.rs` | record **`t`** + header **`v4`** (aditivo; ausente = byte-idêntico) | Baixo |
| `ph2d-editor-core/.../types.rs` | **+1 variante no FIM** do `GraphKey`: `Rename` | Baixo (append-only — mas **conte, não escolha**) |
| `ph2d-editor-core/.../keymap.rs` | **`KEY_F2: u32 = 0xF705`** (const nova) | Baixo |
| `ph2d-editor-core/.../key.rs` + `dispatch/mod.rs` + `interaction/mod.rs` | `KEY_F2` nos re-exports + 1 arm no `graph_key_for` | Baixo |
| `ph2d-panel-motion-graph/src/rename.rs` | **arquivo novo** (a caixa) | — |
| `.../snapshot_intent.rs` | **`RenameTarget`** + intent `Rename`; **`SetBackdropTitle` REMOVIDO** (morto) | **Médio** — se outra linha passou a emiti-lo, ela vira `Rename` |
| `.../snapshot.rs` | `display_name` = label > display do tipo | Baixo |
| `.../{state,hits,paint,interact,interact_key,lib}.rs` | a caixa (estado, id, pintura, tecla, eventos) | Baixo |
| `shells/desktop/src/keymap.rs` | `KeyCode::F2` + 1 caso no gate | Baixo |
| `.../motion_bridge_intents.rs` | `pub(super) fn rename` (3 alvos, 1 undo) | Baixo |
| `.../motion_bridge_rename_tests.rs` | **arquivo novo** (seam) | — |
| `shells/desktop/src/motion_demo_strobe.rs` | **os 19 cards do boot ganharam nome** | Baixo |

**Contrato congelado:** verde (`NodeManifest`=8 / `NodeOp`=2 / `OpResolver`=1).

**Aberto:** a **cor** do backdrop segue sem gesto (o `SetBackdropColor` existe, o shell o executa, e
nada o emite — hoje o tom só cicla por id ao nascer). Nomeado, não escondido.
