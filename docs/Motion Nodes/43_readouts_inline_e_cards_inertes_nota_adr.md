# 43 — Editor F2: **readouts inline** + **cards inertes** — o grafo diz o que está fazendo — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F2** (cauda)
**Status:** implementado, testado (2 mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1 provado depois)
**Foundational tocado:** `ph2d-nodegraph::cook` (**1 método inerente, aditivo**) · `ph2d-tokens` (1 token novo)

---

## 1. O problema

Temos **79 nós** e um editor que não conta **nada** sobre o que eles estão fazendo. O probe (doc 37) responde
por **um** nó de cada vez — você aponta e ele fala. Mas as duas perguntas que um artista faz o tempo todo são:

1. *"o que este nó está produzindo?"* — em **todos** os cards, ao mesmo tempo;
2. *"por que a tela não mudou?"* — que quase sempre tem a mesma resposta: **aquele ramo não está ligado em
   nada**.

## 2. `Cook::peek` — e por que o readout é DE GRAÇA

O cozimento do frame **já avaliou** todo nó que alimenta um sink; os resultados estão no memo do pump. Então um
readout é uma **consulta**, nunca um cozimento:

```rust
pub fn peek(&self, node: NodeId) -> Option<&[CookValue]>  // lê o memo, não cozinha nada
```

Método **inerente** ao `Cook`, **aditivo** — exatamente o padrão já ratificado (`checkpoint`/`restore`
nasceram assim). `NodeOp`/`OpResolver`/`NodeManifest` **intactos** (8/2/1).

**A alternativa tentadora — `cook()` em cada card — é *correta* e mesmo assim ERRADA:** ela avaliaria nós que o
render nunca precisou, **uma vez por card por frame**. Um documento de 79 nós pagaria isso **todo frame, pra
sempre**, só pra que alguns cards mostrassem um número. *Mutante provado.*

## 3. **Branco é a leitura mais útil que existe**

Um nó que o cook nunca puxou **não tem entrada no memo** e fica **sem readout nenhum**. Isso não é um buraco —
**é o diagnóstico**:

> *nada, rio abaixo, consome este card.*

Uma cadeia que o artista esqueceu de ligar no Output; um ramo que a faca orfanou; um nó recém-solto do menu —
**todos brancos**, e o branco diz exatamente por que a tela não mudou. É o alarme *"unit-green ≠ vivo"* deste
módulo, **tornado visível**.

E porque um número **ausente** é algo que você precisa *reparar* que não está lá, o card inerte é **velado**
(token novo `graph-inert`: o bg do próprio painel a 62 % — o card **recua pro canvas** em vez de virar uma cor
nova que precisa ser aprendida). **Véu, não repintura:** o card mantém a cor de categoria, o título, os sockets,
e continua perfeitamente clicável. **O anel de seleção é desenhado POR CIMA** — um nó morto selecionado tem que
continuar parecendo selecionado.

Assim a resposta à pergunta 2 é lida **de relance, no canvas inteiro**, em vez de deduzida card a card.

## 4. O que o número É

Espelha o probe **exatamente** (doc 37), pra que os dois nunca se contradigam: stream de VALOR lê o escalar
(coluna `v`); qualquer outro lê **quantas instâncias** carrega. E uma stream **vazia** diz `empty` — não `0
inst`: um nó que produziu **nada** não é o mesmo que um nó que produziu zero coisas, e dizer isso em palavras
economiza uma investigação.

## 5. A geometria mora no `geom`, não no `paint`

O card com readout é **uma linha mais alto** — e essa linha entra no **`card_h`**, que é a fonte única de
verdade do **hit-test E do paint**. Se a linha entrasse só onde o card é *desenhado*, ele ganharia uma **faixa
morta** na borda de baixo: um lugar que **parece** o nó e **não responde** ao mouse. Guarda:
`a_readout_grows_the_card_and_its_hit_rect_together`.

## 6. As guardas — 2 mutantes provados VERMELHOS

| # | Mutante | Guarda |
|---|---|---|
| 1 | `cook()` em vez de `peek()` | `a_cooked_node_reads_out_and_an_orphan_stays_blank` → o órfão passa a reportar **`Some("9 inst")`**: o diagnóstico some **e** o editor paga uma 2ª avaliação do grafo por frame |
| 2 | (implícita, provada pelo #1) a linha do readout só no paint | `a_readout_grows_the_card_and_its_hit_rect_together` |

## 7. A demo

O documento de boot ganhou **um `motion.grid` ligado em NADA**, de propósito — idêntico aos que as cenas usam,
e fazendo exatamente nada. Ele fica **velado e sem número**, ao lado de vizinhos que mostram os seus. **É a
feature inteira num card.**

## 8. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| `ph2d-nodegraph::cook` | **`Cook::peek(node) -> Option<&[CookValue]>`** (inerente, aditivo; root lane só) |
| `ph2d-tokens` | **`ColorToken::GraphInert`** (`graph-inert`, nos 3 temas que carregam a família `graph-*`) |
| `GraphNodeView` | campo **`readout: Option<String>`** |
| shell | `render_loop/motion_bridge_readout.rs` (módulo irmão novo) — `stamp()` |

## 9. Aberto (F2)

**Waypoints** nos fios — e a decisão já está tomada: eles são **decoração pura** (mudam como o fio é
*desenhado*, nunca o que o grafo computa), então vivem no **`MotionDoc`**, como os backdrops — **não** na
`Edge`. O gate `is_dirty` que os backdrops trouxeram (doc 35) já serve pra provar que arrastar um waypoint não
re-cozinha nada.
