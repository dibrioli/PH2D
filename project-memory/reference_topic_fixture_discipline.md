---
name: reference-topic-fixture-discipline
description: Disciplina de fixture — o gate só prova o que ela contém
metadata: 
  node_type: memory
  type: reference
  originSessionId: ac1a9702-6b56-4e69-aa92-f36f1c65684e
  modified: 2026-07-25T05:20:23.038Z
---

- [[feedback_a_gate_only_proves_what_its_fixture_contains]] — e meça o DOCUMENTO
- [[feedback_zero_valued_fixture_is_a_gate_that_cannot_fail]] — o neutro é o ponto fixo que esconde
- [[feedback_gate_the_edges_of_the_domain]] — DC/Nyquist, 1ª/última coluna, 0 e 1
- [[feedback_a_fixtures_setup_order_can_mask_an_order_dependent_bug]] — smoke/teste na ordem CONVENIENTE esconde bug da ordem do PRODUTO
- [[feedback_changing_a_fixture_invalidates_the_mutation_proof]] — encolher para matar flake tira os dentes do gate em silêncio
- [[feedback_two_quantities_that_should_differ_can_coincide_by_fixture_phase]] — max≠last verde-sobre-nada; ache fixture onde diferem por FÍSICA
- [[feedback_identical_fixtures_hide_the_tiebreak_you_meant_to_test]] — fixture com os dois lados IDÊNTICOS não arma empate (0.0 exato vs 1e-16): o ruído decide certo por acidente
- [[feedback_a_library_doc_can_use_a_word_in_another_sense_and_the_easy_fixture_hides_it]] — «absolute coordinates» = comandos, não espaço; a fixtura ÓBVIA concorda com as duas leituras — só o caso ANINHADO as separa
