---
name: feedback-a-declaration-with-a-default-is-decoration-until-something-reads-it
description: Constante declarativa COM default é herdada por todos e nunca confrontada — 20 de 21 herdaram e 3 mentiam; o default morre quando um consumidor real aparece
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T01:37:02.020Z
---

Uma constante associada que **descreve** algo (*onde este painel fica*, *que tipo de objeto isto
aceita*) e que nasce **com default** para não obrigar N crates a mudar no mesmo commit fica
**decoração**: ninguém a escreve, ninguém a lê, nada a confronta com a realidade.

**Medido na `line/UIUX`, 2026-08-30:** `Panel::DEFAULT_SLOT` nasceu com default `RightTop`.
**20 dos 21** painéis registados herdaram-no, e **três mentiam** — a `hierarchy` publica a coluna da
**esquerda**, `timeline` e `flip_frames` a faixa de **baixo**, e o `motion_graph` é o **centro**. O
custo só apareceu quando as abas passaram a **derivar dela** quem divide qual coluna: até aí, o valor
errado não fazia nada.

⚠️ **E a regra escrita à volta dela também estava errada por omissão:** o gate dizia *«nenhum painel
declara o CENTER»*, e o painel do grafo **é** o centro (ele parte a região em duas irmãs). *Uma regra
escrita quando ninguém declarava nada descreve o vazio, não o modelo.*

**Why:** o default existe para o commit não ser grande, e o preço é que a declaração nunca é medida.
Ela parece dados e é ruído até alguém a consumir — e nesse dia ela está errada em silêncio.

**How to apply:** ao criar a constante, escreva no **mesmo commit** o gate que a confronta com o
comportamento observável (*o encaixe declarado contém o rect publicado*). Se não houver consumidor
ainda, **não lhe dê default**: forçar N crates a escrever uma linha é mais barato que N declarações
por medir. E quando o consumidor chegar, **mate o default** — foi o que curou este caso.

Relacionadas: [[feedback_a_dead_knob_has_two_species_no_probe_catches]] ·
[[feedback_a_promise_that_justifies_a_decision_must_have_a_reader]] ·
[[feedback_a_new_feature_can_empty_an_existing_gates_population]] ·
[[feedback_a_hand_written_list_beside_a_predicate_is_two_answers]]
