---
name: feedback_a_gate_that_presumes_the_destination_of_an_effect_accuses_the_living
description: Um censo de «controlo morto» que pergunta pelo DESTINO (o barramento) e não pelo EFEITO acusa de morto o controlo vivo com outro destino — e a mensagem dele manda construir a doença.
metadata:
  type: feedback
---

Acrescentei dois itens ao menu de um cartão do navegador de assets (2026-09-02). Eles mudam **a
vista do painel** e, ao contrário dos três vizinhos do mesmo menu, não têm nada a dizer ao mundo —
logo não empurram nada para o barramento.

O censo `every_asset_card_menu_entry_dispatches_something` reprovou os dois, com esta mensagem:

> linhas do menu do cartão que são PINTADAS e não despacham nada: ["Show what it uses", "Show what
> uses it"]. Ligue cada uma no `card_verb_of` … e drene a acção no `asset_card_verbs.rs`.

Os dois estavam vivos e correctos. O gate perguntava *«empurrou para o barramento?»* — o único
destino que existia no dia em que foi escrito — e não *«chegou a um efeito?»*.

**Why:** um censo de controlo morto é caro de escrever e é acreditado. Quando ele erra, **a
mensagem dele é uma instrução**: aqui, seguir à letra teria levado a acção ao shell para o shell a
devolver ao painel — construindo a segunda fonte de verdade sobre a vista, que é exactamente a
doença que o censo existe para prevenir. Um falso positivo num gate deste tipo não custa tempo:
custa arquitectura.

**How to apply:** a pergunta de um censo de alcance é **«alguma coisa mudou?»**, nunca «foi por
aquele cano?». Compare o estado antes/depois **além** de olhar o barramento — e compare-o por
`{:?}` do agregado, não por um campo escolhido à mão, para que um campo de vista novo entre no
oráculo sozinho (a mesma razão de a população vir de uma tabela e não de uma lista escrita).
⚠️ E quando um gate reprovar sobre código que você acredita estar certo, leia a **pergunta** dele
antes de obedecer à **instrução** dele.

Irmão de [[feedback_a_new_feature_can_empty_an_existing_gates_population]] e de
[[reference_topic_gate_discipline]]; a família do controlo morto está em
[[reference_topic_control_design_hazards]].
