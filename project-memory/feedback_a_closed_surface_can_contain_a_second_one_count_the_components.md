---
name: feedback_a_closed_surface_can_contain_a_second_one_count_the_components
description: "Uma almofada (duas faces coincidentes, costas com costas) passa em χ, bordo e não-manifold — só a CONTAGEM DE COMPONENTES a apanha"
metadata:
  type: feedback
---

Medido 2026-08-28: o artista fotografou uma **face solta a flutuar** sobre uma ponta. O
ficheiro exportado tinha `23 630` quads, **`0` arestas de bordo, `0` não-manifold** — e
**dois componentes ligados**, de `23 628` e de `2`. A ilha era `[68,69,70,71]` e
`[71,70,69,68]`: *o mesmo quadrado emitido duas vezes, um virado ao contrário.*

⛔ **Nenhuma régua de superfície a via**, e não por descuido: uma almofada é uma superfície
fechada legítima. `χ` conta os dois lados dela e dá `2`; o bordo é zero; o não-manifold é
zero; a contagem de quads **sobe**. *O que ela não é, é parte da peça* — e essa é uma
pergunta sobre **conectividade**, não sobre a superfície.

**Why:** todas as réguas de topologia desta família (`χ`, bordo, não-manifold, valência)
medem a malha **como um objecto só**. Um segundo objecto dentro dela é invisível a todas, e
o único sintoma é visual.

⛔⛔ **2.ª OCORRÊNCIA, 2026-08-30 — e a memória existia.** O artista voltou a fotografar
um pedaço solto, agora de `22` faces, ao carregar no botão uma **segunda** vez. A sonda de
componentes tinha sido construída em 28/08 e **imprimia**; o que decidia entre tentativas
continuava a ler só `bordo + não-manifold`, que dá **zero** nas duas peças. ⇒ *uma régua que
só IMPRIME não é uma régua que DECIDE* — a lição da 1.ª ocorrência foi escrita, a sonda foi
construída, e a chave da escolha nunca a consultou. ⚠️ **E há uma segunda metade:** uma escada
de candidatas ordena entre si e **nunca compara com o que o utilizador já tinha na mão** —
quando todas partem a peça, a melhor delas ainda é uma peça partida. A cura é um **veto
absoluto depois da escada**, relativo à entrada (`saiu > entrou`), nunca a um `1` absoluto.

**How to apply:** onde uma malha é **produzida** (extracção, remesh, booleana), conte os
**componentes ligados por aresta** e diga o tamanho de cada um — uma linha de log barata que
nomeia exactamente o defeito que de outra forma só chega por foto. ⭐ E quando o produtor é
um mapa, o par coincidente tem uma causa nomeável (uma **dobra**: a mesma região percorrida
nos dois sentidos) e a chave que o apanha é o **ciclo sem sentido** — rodar para o menor nó e
ficar com o menor entre o anel e o seu inverso; ⚠️ caem **os dois** lados, porque uma
almofada não tem lado certo. Relacionado:
[[feedback_a_gate_on_the_mark_i_chose_is_green_when_the_marks_premise_is_false]] ·
[[feedback_counting_the_work_done_is_not_counting_the_work_delivered]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]]


## O mesmo cego, um nível acima: contar ITENS não vê itens EMPILHADOS — conte os SÍTIOS

**L-System, 2026-08-30.** Enio: *"elas aparecem em cada segmento"*. Eu contei as marcas de
folha (62 a `g = 5`), expliquei-as pela acumulação, e construí uma lei de tamanho para as
esconder. ⛔ **Nunca contei quantos SÍTIOS distintos elas ocupavam: 30.** Empilhamento de
`2,07×` — folhas idênticas umas sobre as outras, que é uma parte da fealdade que ele via.

A causa era a gramática (o `J` vinha depois da sub-árvore, e ao sair dela a tartaruga está de
volta ao ponto de partida, onde as marcas de todas as gerações que a envolvem também caem).
Uma linha de gramática curou-a: `62/30` → `31/31`.

**Why:** uma contagem responde *«quantos?»* e a pergunta era *«quantos LUGARES?»*. É o mesmo
cego da almofada (duas faces coincidentes), do `edge_max` global (cego a um quad de
`0,02 × 0,30`) e do balde vazio lido como perfeito — a régua olha o agregado e a doença é
local.

**How to apply:** quando um report diz *«há coisas a mais»* ou *«fica um borrão»*, meça
**contagem ÷ posições distintas** antes de explicar a contagem. Se a razão não é `1,00`, o
defeito é duplicação, e nenhuma lei de tamanho, cor ou ordenação o cura.
