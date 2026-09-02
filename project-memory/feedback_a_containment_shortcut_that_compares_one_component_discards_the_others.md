---
name: feedback-a-containment-shortcut-that-compares-one-component-discards-the-others
description: "Atalho «uma contém a outra, fica a maior» num valor COMPOSTO devolve o vencedor inteiro e deita fora as componentes que não foram comparadas"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-01T21:44:58.055Z
---

Quando um valor passa a ter **duas componentes** (uma esfera *e* uma caixa, um custo *e* um prazo,
um limite *e* uma unidade), todo atalho de contenção escrito para a componente antiga fica errado:
`if a contém b { return a }` devolve `a` **inteiro**, e as componentes que o teste não olhou vão
com ele.

Caso medido (PH2D, `line/3DModeling`, 2026-09-01, report do Enio com foto — *«os 3 cilindros
cruzados viraram isso»*): o `Ball::merge` comparava só as **esferas**. Três cilindros cruzados têm o
**mesmo centro** e o **mesmo raio** com caixas em eixos diferentes ⇒ o atalho disparava à primeira e
a união ficava com a caixa de **um** cilindro (`0,18 × 0,18 × 0,60` em vez de `0,60³`). Duas das três
pontas caíam fora do recorte: **754** de `2 576` pixels do interior com a normal a `172,7°` do
oráculo. Curado (o atalho escolhe **só** centro e raio; a caixa é sempre computada das duas): **0**
de `5 118` — e a contagem de pixels **DOBRA**, que é a prova de que metade da peça estava a ser
cortada.

**How to apply:**
- ⭐ Ao acrescentar uma componente a um valor, **grepe os `return` antecipados** de tudo o que o
  combina. Cada um é um sítio onde a componente nova evapora, e nenhum deles falha a compilar.
- ⭐⭐ A forma da cura é sempre a mesma: **o atalho escolhe o REPRESENTANTE, e as componentes são
  recombinadas fora dele** — nunca `return o_vencedor_inteiro`.
- ⚠️ **A degenerescência que o revela é a IGUALDADE** (mesmo centro, mesmo raio): é aí que o atalho
  dispara com as outras componentes a discordar. Uma fixtura com valores distintos passa
  ([[feedback_a_corpus_sitting_at_a_knobs_neutral_point_does_not_test_that_knob]]).
- ⛔⛔ **Uma resposta pode estar errada há semanas e só doer no dia em que alguém a lê**: enquanto o
  consumidor lia o raio (que estava certo), a caixa errada não cortava nada. *Ligar um leitor novo a
  um campo antigo é auditar esse campo, não usá-lo.*
- ⚠️ **O gate que devia apanhar tinha um ponto cego ESTRUTURAL**, não de valores: as 60 fixturas dele
  eram sempre **um nó folha** com pilha de modificadores, e nunca uma **combinação de filhos**.
  ⇒ ao escrever um gate de bordo, pergunte *que FORMAS DE ÁRVORE ele nunca constrói?*
