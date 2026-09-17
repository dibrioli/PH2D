---
name: feedback_the_artists_key_outranks_the_automatic_correction
description: Uma correção automática que não sabe distinguir a pose que o ARTISTA fez da deriva que ela existe para curar desfaz o trabalho dele — e o discriminador é quem PÔS, nunca um limiar
metadata:
  type: feedback
---

Toda correção automática que atua sobre uma animação precisa saber **o que ali é intenção e o que é resíduo**. Sem isso ela desfaz o trabalho do artista, e a queixa que chega é *«não consigo posicionar como eu desejo»*.

Medido no testbed do Cascadeur (14/09, report do Enio com foto). A trava «pés apoiados não escorregam» ancora o pé onde ele **tocou** o chão. O artista vai a um quadro no meio do apoio e abre as pernas 65 cm; a trava puxa o pé de volta para a âncora e a abertura vira **0 cm**. Reproduzido nos dois exemplos, em vários quadros.

⛔ **A cura NÃO é um limiar, e a medição fecha essa porta:** a deriva que a trava existe para curar chega a **15,9 cm** no mortal (o artista roda o pé entre chaves e a ponta arrasta) e um pé posto de propósito anda **45 cm**. Não há vale limpo entre as duas — e pela velocidade também não (o pé real a rolar faz 4,7 cm num quadro; o arrasto deliberado, 6,4).

⭐ **O que separa é quem PÔS.** Uma chave criada pelo artista é marcada (`minha: true`) e re-ancora a trava; uma chave que veio com o exemplo, não. Zero constantes novas, zero calibração.

**How to apply:** (1) quem cria a intenção **marca-a na hora** — depois não há como inferi-la sem adivinhar; (2) a marca viaja com o DADO, não com a chamada: aqui a animação amostrada leva as próprias chaves, senão cada instrumento teria de se lembrar de as passar e os que esquecessem mediriam outro programa que o app; ⚠️ **e uma propriedade não atravessa uma transformação sozinha** — a passagem do atraso dos braços devolve um array novo e a marca sumia ali, com o defeito a voltar inteiro; (3) ⛔ **não deixe o consumidor responder à mesma pergunta por outra via**: o app continuou a passar *todas* as chaves por cima da marca, a trava deixou de funcionar e o pé voltou a escorregar 5,1 cm — quem apanhou foi o teste de **gesto real**, não a suíte. Ver [[reference_topic_authored_state_and_clocks]] e [[project_teste_cascadeur_2d_bones_testbed]].
