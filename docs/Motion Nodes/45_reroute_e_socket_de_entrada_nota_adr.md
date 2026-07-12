# 45 — O ponto no fio é um **NÓ** (reroute) · e o socket de ENTRADA vira um plugue — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F2**
**Status:** implementado, testado (2 mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum
**SUPERSEDE o doc 44** (waypoints como decoração — **deletados**)

---

## 1. A pergunta do Enio que condenou o desenho

> *"k não preserva o ponto. o fio é todo cortado. e como se conecta esse ponto no meio do fio? ou não se
> conecta?"*

O waypoint do doc 44 era **decoração**: dobrava o fio e **não conectava em nada**. Tecnicamente correto — e
**uma mentira**. Ele **parecia** uma junção. Era um afordance que mentia sobre a própria capacidade, e o Enio o
pegou em **uma pergunta**.

E olha onde a indústria parou: o **Reroute do Blender** e o **Dot do Nuke** são **NÓS**. Nenhum dos dois tem
"dobra decorativa". Não é coincidência — é porque só sendo um nó as três perguntas têm resposta óbvia:

| Pergunta | Decoração (doc 44) | **Nó** (doc 45) |
|---|---|---|
| *Conecta?* | **não** — e parece que sim | tem input e output. **Puxa um fio dele.** |
| *O que o K faz?* | mata o fio inteiro **e o ponto** | corta **o fio que você cruzou**. O nó fica |
| *Delete / undo / seleção / Ctrl+D?* | tudo caso especial | **de graça** — é um nó |

**Reversão completa:** os waypoints (modelo, formato, painel, shell, 3 intents) foram **DELETADOS**. O
`[backdrop]` do documento voltou a ter um só cidadão.

## 2. `util.reroute` — pass-through, e por isso seguro

`eval` emite a entrada, intacta. `Effect::Pure` — memoiza como qualquer nó, e o preço de ser **real** é **um
hop**, não um recálculo.

**A guarda que importa:** `splicing_a_reroute_does_not_move_a_single_pixel` — o oráculo é o **render**. Você
arruma o canvas numa cena viva e **nada se move**. É o que faz o gesto seguro de usar sem pensar.

### Três tipos, e o artista NUNCA escolhe

`NodeManifest.inputs/outputs` são `&'static` (contrato congelado), então um tipo de nó carrega **UMA** porta —
não existe reroute genérico sem descongelar o contrato, e **descongelar o contrato pra economizar dois `const`
seria um péssimo negócio**. A biblioteca inteira (80 nós) usa **exatamente 3 tipos de porta**, então há
exatamente 3 reroutes: `util.reroute` (stream) · `util.reroute_value` · `util.reroute_pulse`.

**O gesto é *duplo-clique num fio*, e o shell escolhe pelo FIO.** Um fio sabe o que carrega. Não há menu, não há
escolha errada a fazer, e os três tipos são detalhe de implementação que o artista não encontra. (Estão no
add-menu também, pra quem quiser posicionar antes de ligar.)

**Um crate, três nós.** O codegen só chama `register()` por crate — nada exige 1:1. Três crates de quarenta
linhas idênticas seriam três lugares pro mesmo bug se esconder.

## 3. E o socket de ENTRADA finalmente faz alguma coisa

> *"não dá pra puxar fio do socket de entrada. implemente"*

Até agora um input era um **ponto pintado**: o `interact.rs` tinha literalmente `// Input sockets (the
reverse-drag of an occupied input) land later`. Isso é **meio editor** — um grafo se lê nos dois sentidos, e às
vezes você sabe o que um nó **precisa** e vai caçar quem fornece.

Um input tem **dois** gestos, e qual você recebe depende de já haver fio ali:

- **Input VAZIO → desenha pra TRÁS.** Um fantasma segue o cursor caçando um output. Solta nele e a conexão é
  feita — a mesma que seria feita no outro sentido. (O fantasma corre **do cursor PARA o input**: o fio já se lê
  como vai se ler quando existir.)
- **Input OCUPADO → PEGA A PONTA DO FIO.** Ele solta o socket e segue o cursor, **ainda preso à fonte**. Daí em
  diante é um desenho de fio comum — toda a maquinaria do `DrawWire` (fantasma, destaque do alvo, prévia de
  compatibilidade) funciona sem mudar uma linha.

### As três decisões que fazem isso não morder

1. **Uma ponta agarrada MOVE; não copia.** O drop emite **`MoveWireEnd`** (um intent, **um** passo de undo:
   desliga + liga), não um `Disconnect` seguido de `Connect`. *Mutante provado:* sem o unplug, arrastar um fio o
   **duplicaria**.
2. **Uma recusa NÃO destrói o fio.** Tentou levar o fio pra onde ele não pode ir? A resposta é **"não"** — não
   *"e o fio que você tinha também sumiu"*. Tudo num clone de teste; a recusa devolve o original e toasta.
3. **O fio arrancado NÃO é desenhado.** Um fio ainda plugado no socket de onde você o está visivelmente
   puxando é um fio que **não se moveu** — o artista arrastaria um fantasma sem saber se o gesto pegou. E os
   hit-rects vão junto: não se alt-clica um fio que se está segurando.

Soltar no vazio **desliga**. Soltar de volta no mesmo lugar **não faz nada** — e não empilha undo pra um gesto
que não mudou nada.

## 4. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| crate nova | **`ph2d-node-util-reroute`** — 3 nós: `util.reroute` · `util.reroute_value` · `util.reroute_pulse` (**80** crates-nó) |
| `GraphIntent` | **`SpliceReroute`** · **`MoveWireEnd`** (os 3 intents de waypoint **removidos**) |
| `Interaction` | `DrawWire.detached` · **`DrawWireBack`** (`DragWaypoint` removido) |
| painel | `interact_socket.rs` (novo) · `geom::hit_output_socket` · `paint::detached_edge` (`route.rs`/`interact_waypoint.rs` **removidos**) |
| shell | `motion_bridge_rewire.rs` (novo) · `motion_bridge_waypoints.rs` **removido** |
| `ph2d-motion-doc` | `Waypoints` + o registro `w` **REMOVIDOS** — o formato voltou ao que era |

## 5. A lição

**Um afordance que parece uma coisa e não é, é um bug — mesmo quando o código está certo.** O waypoint fazia
exatamente o que eu projetei e passava em todos os testes que eu escrevi. Nenhum deles podia pegar o problema,
porque o problema era **o que o artista esperava dele**, e disso só o smoke sabe.
