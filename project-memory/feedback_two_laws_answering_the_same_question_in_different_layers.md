---
name: two-laws-answering-the-same-question-in-different-layers
description: Duas leis que respondem à mesma pergunta em camadas diferentes — a de cima apaga a de baixo, e o que se vê não é um erro: é um portão que deixa de afirmar
metadata:
  type: feedback
---

Quando uma lei nova responde a uma pergunta que uma lei antiga já respondia **noutra camada**, a que
corre primeiro decide e a outra fica inerte. Nada falha. O que muda é que **os portões da lei de
baixo passam a estar verdes por vácuo** — eles continuam a correr, continuam a imprimir números, e
já não afirmam nada.

**Medido (Teste Cascadeur, 2026-09-19).** A lei do pé apoiado (`fixarApoios`) supõe que a animação
TEM um pé apoiado. Uma caminhada de ciclo no lugar não tem, e eu escrevi a pergunta — *«existe nesta
animação algum pé que esteja de facto quieto?»* — **dentro** do `fixarApoios`.

Mas há um segundo consumidor: um ajuste que **já tinha decidido**, com uma cerca de velocidade
própria e medida, quais faixas são apoio, e que chama o `fixarApoios` **depois** dessa decisão. Com a
pergunta lá dentro, a minha decisão passava por cima da dele: as faixas que ele tinha deixado de pé
eram precisamente as rápidas, a minha lei via *«esta animação não tem apoio nenhum»* e devolvia sem
fazer nada. ⇒ **duas mutações que matavam portões daquela cerca passaram a SOBREVIVER**, e a suíte
ficou toda verde.

⭐ **Quem o disse foi a prova de mutação, e mais nada o teria dito.** A suíte passava, o produto
estava certo para o caso novo, e os portões antigos continuavam a imprimir os números de sempre.

⇒ **A cura é de ENDEREÇO, não de lógica:** a pergunta mudou-se para a **entrada** do sistema (ao lado
do interruptor que já decidia se a lei corre), e o segundo consumidor voltou a ser dono da decisão
dele. Zero mudança na lei.

**Como aplicar.** Antes de escrever uma cerca dentro de uma função, conte os **chamadores** e
pergunte de cada um: *este já respondeu a esta pergunta à maneira dele?* Se sim, a cerca não pode
viver lá dentro — ela vive em quem chama. ⚠️ E o sinal de que se errou não é uma reprovação: é uma
mutação que deixa de sangrar. Uma wave que acrescenta uma cerca a uma lei partilhada **corre a prova
de mutação inteira**, nunca só a do pedaço novo.

Irmãs: [[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]] e
[[feedback_a_gate_can_record_a_loading_defect_as_a_law]] — as três são a mesma família: *um portão
pode deixar de afirmar sem nunca ficar vermelho*.

---

### ⭐⭐ A SEGUNDA FORMA, no mesmo dia: a lei que se ALARGA apaga a prova da lei que ela SUBSOME

A primeira metade desta memória é sobre ENDEREÇO (uma cerca escrita dentro de uma função passa por
cima de quem a chama). A segunda é sobre ALCANCE, e o sinal é o mesmo.

**Medido (Teste Cascadeur, 2026-09-19).** A cerca do chão do solver valia para os pontos de
CONTACTO, e a mão estava lá com a espessura dela como altura. Ela passou a valer para o **corpo
inteiro**. A mão continua a parar no chão — agora por outra via —, logo a mutação que a defendia (a
que a tirava da lista de contactos) **deixou de sangrar**, e o portão dela ficou verde sem afirmar
nada.

⇒ **quando uma lei passa a cobrir o que outra cobria, a mutação da antiga tem de ser RE-APONTADA à
linha nova.** Aqui ficaram duas mutações na MESMA linha, cada uma a tirar uma coisa diferente da
população (uma tira a mão, a outra tira a cauda) e cada uma a matar o seu portão.

⚠️ E há um resíduo que se lê como limpeza: a lista antiga fica **MORTA** — continua a ser escrita,
ninguém a lê, e nenhuma sonda de «quem lê isto?» a acusa. Apagá-la parte quem a lia **nos TESTES**,
e aí a pergunta certa não é «como reponho o campo» mas *«o que é que aquele portão devia estar a
medir agora?»* — no caso, o corpo inteiro, que é mais forte do que o que ele media.
