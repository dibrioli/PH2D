---
name: feedback-a-new-feature-can-empty-an-existing-gates-population
description: "Uma feature nova pode ESVAZIAR a população que um gate antigo varre — ele fica verde a medir nada, e o único aviso possível é um controlo de população escrito ANTES"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T01:36:44.535Z
---

Um gate que varre uma população (*«nenhum painel publica rect sobre a área de desenho»*) depende de
essa população existir no arnês dele. **Uma feature nova pode encolhê-la a zero sem tocar numa linha
do gate** — e ele passa, verde, a medir nada.

**Medido na `line/UIUX`, 2026-08-30 (entrega 21, as ABAS):** o gate da D1 pintava um quadro com
*«tudo aberto»* e lia os rects publicados. Ao introduzir abas, **doze dos treze** ocupantes da coluna
da direita deixam de pintar por construção — a varredura foi de **18 medidos para 2**. O gate
continuaria verde para sempre.

⭐ **Quem o disse foi um `assert!(measured >= 10)`** escrito na entrega ANTERIOR, por outra razão
(evitar que um registry não instalado passasse sobre zero painéis). *O controlo de população é a
única coisa que avisa, e ele tem de existir antes da feature que o vai acordar.*

**Cura:** o gate passou a medir **um sujeito de cada vez** (abrir só ele, pintar, ler) — populações
de tamanho `1` que a feature não consegue esvaziar.

**Why:** um gate verde que você não sabe derrubar não é um gate, e aqui nem uma mutação no código
medido o derrubaria — a população é que desapareceu.

**How to apply:** todo gate que varre uma população escreve, no mesmo commit, o **piso** dela
(`measured >= N`, `saw_a_bar >= N`) com a razão ao lado. E ao construir uma feature que muda **quem
aparece**, corra os gates de varredura e leia o número que eles medem, não só o veredito.

Relacionadas: [[feedback_absence_gate_needs_a_presence_sibling]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_a_sampled_fixture_proves_what_it_sampled_gate_the_property_where_it_is_defined]] ·
[[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]]
