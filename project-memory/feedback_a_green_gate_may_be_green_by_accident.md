---
name: feedback-a-green-gate-may-be-green-by-accident
description: "Gate verde de 1ª pode estar verde por acidente do fixture — só a mutação separa \"guarda a propriedade\" de \"não pode falhar\""
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08ec2d6a-35b1-4812-a467-769e905c0028
---

Um gate que nasce verde é suspeito, e a prova de mutação é o que separa **"ele guarda a
propriedade"** de **"o fixture não consegue violá-la"**. No §4.B do Flip, 3 fixtures em ~20
gates estavam verdes por acidente, e **cada um foi exposto por uma mutação sobrevivente ou por
um vermelho que era do fixture, não do motor**:

1. Um cortador **horizontal** para cortar a aresta **horizontal** de um quadrado: colinear
   **não cruza**, então o corte nunca existiu (vermelho — mas do fixture, não do código).
2. O gate do "corte em `λ=0`" passava porque a MESMA linha colinear injetava, pela ponta, um
   2º corte que eu não pedi: o teste media outra coisa e concordava.
3. O gate "o miolo do preenchimento ignora os cortes" usava uma região `hide_stroke` — que
   **por construção não entra na lista de cortadores** e portanto **não tem corte**. A
   mutação sobreviveu: não havia corte a ignorar.

**Why:** os três teriam ido para o `main` como gates decorativos, e o defeito que eles diziam
prender apareceria no smoke do Enio.

**How to apply:** para todo gate, rode a mutação que ele afirma pegar e exija VERMELHO (sobre
um verde já visto). Quando a mutação **sobrevive**, o suspeito nº 1 não é o gate — é o fixture
não conter o que o nome dele promete. Dois corolários que valeram achado: (a) quando a
mutação sobrevive, cheque se o sítio que você mutou **existe** — eu afirmei num doc uma
mutação num sítio que a minha própria representação tinha eliminado, e a afirmação era mentira
no código; (b) quando duas mutações matam o mesmo par de gates, ache a mutação que mata um e
**poupa** o outro — senão são um gate só com dois nomes. Ver
[[reference_topic_mutation_proofs]], [[reference_topic_fixture_discipline]] e
[[feedback_identical_fixtures_hide_the_tiebreak_you_meant_to_test]].
