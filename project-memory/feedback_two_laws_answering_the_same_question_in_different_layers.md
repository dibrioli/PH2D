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
