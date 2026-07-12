# 47 — **Influência** (o que o nó afeta) e o **postage stamp** (o que o nó faz) — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F3** (parte 2)
**Status:** implementado, testado (2 mutantes provados — um deles achou um bug meu), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum

---

## 1. Influência — *"se eu mexer aqui, o que se move?"*

É a pergunta que se faz antes de ousar editar um grafo grande, e a mais difícil de responder no olho: o fio que
você precisa seguir é exatamente o que some atrás de três cards. Selecionou um nó → **o que o alimenta** (ancestrais)
e **o que ele muda** (descendentes) ficam de pé; o resto **recua** (mesmo véu do inerte — uma região do canvas some
como UMA região, não como cards apagados amarrados por fios acesos). **Esc** solta a seleção e o canvas volta.

**Seleção vazia não escurece NADA.** Um canvas que fica cinza no instante em que você clica no vazio pune o gesto
mais comum que existe.

### O bug que a guarda pegou

Minha 1ª BFS andava **nos dois sentidos a partir de cada nó alcançado**. Isso é o **componente conexo**, não a
influência: ela subia até o Output e **descia de volta por todo galho vizinho** que também alimenta o Output —
acendendo um grafo que a seleção não pode tocar. Agora são **dois passeios DIRIGIDOS** (ancestrais só sobem,
descendentes só descem) e a união com a seleção. Guarda:
`selecting_a_node_lights_what_feeds_it_and_what_it_changes` (o galho paralelo tem de ficar apagado).

**E o fio pertence à influência sse AS DUAS pontas pertencem.** Um fio saindo de um ancestral para um nó qualquer
carrega dado que a seleção nunca vê.

### O que NÃO fiz (e por quê)

O plano F3 dizia "influence (BFS por **AttrAccess**)" — influência por-atributo (*"este nó mexe em `P`, não em
`tint`"*). **`AttrAccess` não existe no `NodeManifest`, e o `NodeManifest` é contrato CONGELADO (§6).** A influência
é **estrutural** (arestas). A versão fina vale um ADR algum dia; não vale **quebrar o freeze** hoje.

## 2. Postage stamp — *"onde a espiral vira grade?"*

Cada card ganha uma janelinha com **um espalhamento dos próprios pontos** que o nó emite. É o **thumbnail do Nuke**
— *"those little pictures… show what each node passes onto the next node in the tree"* — e responde o que fio nenhum
responde, por melhor que seja desenhado.

- **Custo limitado por construção:** `PREVIEW_POINTS = 96` por card ⇒ o preço é função do **número de CARDS**, nunca
  do tamanho dos streams. (A Foundry precisa mandar **desligar/congelar** os thumbnails num script pesado, porque os
  dela renderizam a imagem real. Um scatter de 96 pontos não tem esse precipício — por isso o nosso pode nascer
  **ligado**.)
- **A subamostra é POR PASSO (stride), não `take(96)`** — mutante provado: os 96 primeiros pontos de uma espiral de
  5 000 são o primeiro oitavo de uma volta; o card mostraria uma **vírgula** e a chamaria de espiral.
- **Escala UNIFORME e y-para-cima:** o stamp é uma janela para o canvas. Esticar o conteúdo para preencher a caixa
  mostraria um círculo como elipse — mentindo exatamente sobre o que ele existe para mostrar.
- **Nó de VALOR não ganha stamp** (o número dele já é o stamp). Caixa vazia = promessa de desenho que não vem.
- A altura do stamp entra em **`geom::card_h`**, não no `paint` — senão o card seria desenhado mais alto do que é
  clicável (uma faixa morta na borda de baixo).

## 3. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| painel | `flow::influence_set` · `flow::edge_in_influence` · `geom::PREVIEW_H` / `geom::preview_rect` · `paint::draw_preview` |
| `GraphNodeView` | **`preview: Option<Vec<[f32; 2]>>`** (subamostra das posições, em unidades de mundo) |
| shell | `readout::preview_of` + `PREVIEW_POINTS = 96` |
| semântica | `draw_wire(.., bright)` — o antigo `live` agora é "vivo **E** (com seleção) dentro da influência" |

## 4. A lição

**Uma BFS "nos dois sentidos" não é a união de duas BFS dirigidas.** A primeira é o componente conexo e responde
*"quem está no mesmo emaranhado?"*; a segunda responde *"quem eu afeto?"*. As duas passam num teste que só pergunta
*"o ancestral acendeu?"*. O que separa é o **galho paralelo** — e ele só entra no teste se você escrever o teste
para a pergunta, não para o código.
