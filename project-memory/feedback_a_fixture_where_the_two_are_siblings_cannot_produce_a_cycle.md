---
name: feedback_a_fixture_where_the_two_are_siblings_cannot_produce_a_cycle
description: "Mutação sobreviveu duas vezes seguidas no mesmo bloco: uma porque a linha era redundante, outra porque a RAZÃO escrita ao lado dela estava errada"
metadata:
  type: feedback
---

Duas mutações sobreviveram no mesmo bloco de código em 2026-09-06 (`line/components`, o passe que
reparenta as peças de uma cópia), e **nenhuma das duas era um gate em falta**. As duas leituras são
diferentes e as duas valem:

**1. A linha era REDUNDANTE.** Uma guarda explícita (*«a raiz da cópia nunca é reparentada»*) não
mudava nada: quem já recusava era um `None => continue` três linhas abaixo, porque o pai da raiz do
mestre **nunca** está no mapa que o bloco consulta — ele fica *acima* do mestre, e o mapa só tem
peças de *dentro* dele. ⇒ a linha fica como **cerca legível** (é o sítio onde alguém leria a lei) e a
redundância passa a estar **escrita**. *Uma linha que mutação nenhuma mata ou é dívida ou é cerca — e
a diferença é dizê-lo.*

**2. A PROPRIEDADE era real e a RAZÃO escrita ao lado dela era falsa.** Eu tinha documentado que *«a
pré-ordem do mestre é o que impede um ciclo»*. Invertida a travessia, **nada reprovou** — porque os
alvos são calculados **antes** de qualquer escrita, logo as atribuições são independentes e o ciclo,
quando existe, vive **entre dois `insert`** que nenhuma travessia observa. A ordem não era
load-bearing; a forma *recolher-e-depois-aplicar* é que era. ⇒ o comentário foi reescrito a dizer
qual é a propriedade que de facto compra a segurança, e **o que a reabriria**.

⚠️⚠️ **E a fixtura da segunda também estava errada, na direcção que absolve:** para um ciclo se fechar,
o alvo do movimento tem de ser **descendente** de quem se move. A minha punha os dois como
**irmãos** — a operação era segura por construção, e o gate media um caso que a linha sob teste não
precisava de defender. *Uma fixtura que não produz o fenómeno absolve a linha que o impede.*

**Why:** o custo de não olhar é escrever no repo uma explicação errada ao lado de código certo. A
próxima pessoa acredita nela, protege a coisa errada (a ordem) e destrói a certa (a colecta) sem que
nenhum teste fale.

**How to apply:** quando uma mutação sobrevive, faça **três** perguntas, não duas: *falta um gate?* ·
*a linha é redundante?* · **e a fixtura chega a produzir o fenómeno?** E ao escrever *«X é o que
impede Y»*, mute o X: se nada reprovar, o que impede Y é outra coisa. Ver
[[feedback_a_surviving_mutation_can_mean_the_code_is_redundant]] e
[[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]].
