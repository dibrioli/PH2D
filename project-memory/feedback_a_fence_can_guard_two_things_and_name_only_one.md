---
name: feedback-a-fence-can-guard-two-things-and-name-only-one
description: Apertar uma cerca para a grandeza que o NOME dela diz derruba o que ela protegia sem nomear — pergunte o que mais falha se ela ceder
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-01T19:14:02.718Z
---

Uma cerca escrita sobre um majorante **folgado** costuma estar a proteger mais do que o doc dela
diz. Apertá-la para a grandeza exacta que o nome promete é «corrigir» — e derruba a protecção que
ninguém escreveu.

Caso medido (PH2D, `line/3DModeling`, 2026-09-01): a `BEND_FOLD_MARGIN` diz que impede o **colapso
radial** (o lado de dentro da dobra a cair no centro do arco), e para isso o alcance certo é a
extensão em `X`. Ela lia o **raio da esfera**. Ao trocar para a caixa, numa barra
`0,10 × 0,10 × 0,80` o tecto de curvatura saltou de `1,11` para `9,0`: a uma volta pedida a peça
sai **sem saturar**, com `ρ = 0,159` sobre um eixo de `1,6` ⇒ **1,6 voltas**, atravessando-se a si
própria. O CONTROLE do gate de imagem caiu de milhares para **898** pixels de interior.

⇒ a esfera cobria o **enrolamento** por acidente, porque numa barra ela é a ALTURA — e o
enrolamento não estava escrito em lado nenhum.

**How to apply:**
- ⭐ Antes de apertar uma cerca, pergunte: *«que outros modos de falha ficam do lado errado se ela
  ceder?»* — e procure-os na **geometria/física**, não no doc dela (o doc só tem o que alguém
  escreveu).
- ⚠️ O sinal é o CONTROLE de um gate a cair, não a asserção: *o produto não ficou errado, ficou
  outro* ([[feedback_a_cure_that_moves_the_defect_names_it]]).
- ⭐⭐ A cura certa é **partir os dois leitores**: um número para o que a cerca de facto mede (aqui o
  recorte, que quer a caixa) e outro para o que ela protegia (aqui a curvatura, que fica na esfera
  até alguém escrever a cerca do enrolamento). *Duas perguntas com o mesmo nome é como uma delas
  desaparece.*
- ⇒ e deixe **nomeada** a cerca em falta, senão o item «tecto morto do slider» volta a ser lido como
  afinação ([[feedback_documented_decision_chesterton_fence]]).
