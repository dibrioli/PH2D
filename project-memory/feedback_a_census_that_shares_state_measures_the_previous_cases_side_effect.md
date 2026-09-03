---
name: feedback_a_census_that_shares_state_measures_the_previous_cases_side_effect
description: O meu censo de controlos acusou 14 vivos de mortos — ele reutilizava um painel, e o 1.º clique fechava-o. E os 3 acusados que sobraram eram pontos cegos do ORÁCULO, cada um descoberto por uma acusação falsa.
metadata:
  type: feedback
---

Construí um censo que clica em **tudo o que um painel pintou** e exige que cada clique chegue a um
efeito (2026-09-02). Ele acusou **catorze** controlos de mortos, todos vivos.

A causa: eu percorria a lista com **um** painel. O primeiro id era o botão de FECHAR — e a partir
daí todos os outros batiam na guarda *«só existe com o painel aberto»* e saíam `Ignored`.
*Um censo que partilha estado entre casos mede o efeito colateral do caso anterior.* ⇒ instância
fresca por caso, sempre.

Com isso corrigido sobraram três acusados, e **nenhum era defeito** — os três eram pontos cegos do
oráculo, e cada um só apareceu porque a acusação falsa me obrigou a ir ver:

| acusado | porque estava vivo | o que faltava ao oráculo |
|---|---|---|
| o botão de fechar | o efeito dele vive no **HOST**, não no estado nem no barramento | **três** canais, não dois |
| um campo de texto · um slider | respondem ao FOCO e a `ValueChanged`, não a `Click` | excepção **nomeada com o gesto real** de cada um |
| o chip já escolhido | clicar no que já está escolhido é um no-op **correcto** | a excepção tem de ser **derivada do estado** |

⚠️ E o terceiro trouxe uma armadilha de fixtura: **uma fixtura no estado de omissão não consegue
medir o controlo que devolve ao estado de omissão** — ela tem de semear deliberadamente o
não-omissão.

**Why:** um censo de «controlo morto» é acreditado, e a mensagem dele é lida como instrução. Um
falso positivo manda alguém procurar um defeito que não existe — ou pior, «curar» código correcto.
E o oráculo de um censo destes é sempre uma **enumeração dos sítios onde um efeito pode aterrar**:
o que falta na enumeração é exactamente o que só se descobre por uma acusação falsa.

**How to apply:** instância fresca por caso. Enumere os canais de efeito **todos** (estado do
widget · barramento · host) antes de acreditar num «morto». Toda excepção é nomeada com o gesto
real que aquele controlo tem, e a que depende do estado é **derivada** dele, nunca uma lista de
índices — uma lista dispensa o controlo errado no dia em que a omissão muda, em silêncio.
E quando o censo acusar em massa, suspeite dele antes do código.

Irmão de [[feedback_a_gate_that_presumes_the_destination_of_an_effect_accuses_the_living]] (a mesma
doença, um nível acima) e de [[reference_topic_gate_discipline]].
