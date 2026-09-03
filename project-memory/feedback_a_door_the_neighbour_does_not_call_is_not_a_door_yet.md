---
name: feedback_a_door_the_neighbour_does_not_call_is_not_a_door_yet
description: Escrever a porta única não fecha a pergunta — a mesma pergunta foi respondida com a metade errada QUATRO vezes, a última a quinze linhas da porta. Só um CENSO a torna porta.
metadata:
  type: feedback
---

Uma peça do PH2D carrega a textura de duas formas, e elas não são meio-a-meio: **átlas** (o caminho
de toda imagem importada e de toda tela nova — a esmagadora maioria) e **`SpritePixels`** (o
carimbo, a minoria). Um sítio que pergunta só pela minoria **compila, corre e responde vazio sobre
o caminho normal**: sem erro, sem aviso.

Aconteceu **quatro vezes**, sempre fechado por um report do Enio: a cor dominante de um prefab · a
peça-cara · os utilizadores de uma imagem · as dependências de um prefab. Depois da 1.ª escrevi a
porta (`texture_of`, que sabe as duas formas) com um doc-comment a prometer que *«uma terceira
forma amanhã entra aqui e não volta a partir o cartão»*.

**A 4.ª ocorrência aconteceu QUINZE LINHAS abaixo dessa porta, no mesmo ficheiro, com o vizinho já
a chamá-la.**

**Why:** escrever a porta resolve os chamadores que existem naquele dia. Ela não tem forma de
alcançar o código que se escreve depois — e a forma errada continua a compilar, porque ela é uma
das duas respostas válidas. ⇒ *uma porta que o vizinho não chama ainda não é uma porta — é uma
função com um bom doc-comment.*
⚠️ E o gate de valor **já tinha a fixtura certa** (montava exactamente a cena do report): ele media
a cor e o retrato, e não media a lista. *Uma fixtura que já tem o fenómeno não protege as perguntas
que ninguém lhe faz.*

**How to apply:** ao curar a 2.ª ocorrência de uma pergunta com duas formas, escreva no MESMO
commit o **censo** que recusa a leitura crua fora da porta, com a lista de ficheiros do assunto — e
prove-o com a mutação que repõe a linha antiga (ele tem de nomear o número da linha). ⚠️ A régua
olha **leituras**, não a palavra: um `use`, ou uma consulta que peça as DUAS formas para as dar à
porta, são inocentes, e um censo que os acusa é abandonado na primeira semana.
E ao acrescentar uma pergunta nova sobre uma fixtura antiga, pergunte o que ela **ainda não** mede.

Irmão de [[feedback_a_registry_cannot_tell_a_missing_feature_from_a_typo_ask_the_tree]] e de
[[feedback_a_tool_is_adopted_only_when_a_written_step_names_it]] — a mesma lei, um nível abaixo:
*ponteiro não é adopção, e porta escrita não é porta usada.*
