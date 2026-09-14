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
- ⚠️ **Uma cláusula REDUNDANTE na fixtura não é testada por ela** (2026-09-16, `ph2d-field` coarsen): «as pontas de um arco nunca saem» sobreviveu à mutação em todas as fixturas de quina — lá as pontas viram `90°` e o orçamento de giro mantinha-as de qualquer forma. Só um arco RASO (`6°`, pontas a `3°`) põe o caso em que a cláusula é quem decide. *Antes de dar uma guarda por gateada, construa a fixtura onde, sem ela, a resposta muda.*
- [[feedback_gate_the_edges_of_the_domain]] — DC/Nyquist, 1ª/última coluna, 0 e 1
- [[feedback_a_fixtures_setup_order_can_mask_an_order_dependent_bug]] — smoke/teste na ordem CONVENIENTE esconde bug da ordem do PRODUTO
- [[feedback_changing_a_fixture_invalidates_the_mutation_proof]] — encolher para matar flake tira os dentes do gate em silêncio
- [[feedback_two_quantities_that_should_differ_can_coincide_by_fixture_phase]] — max≠last verde-sobre-nada; ache fixture onde diferem por FÍSICA
- [[feedback_identical_fixtures_hide_the_tiebreak_you_meant_to_test]] — fixture com os dois lados IDÊNTICOS não arma empate (0.0 exato vs 1e-16): o ruído decide certo por acidente
- [[feedback_a_library_doc_can_use_a_word_in_another_sense_and_the_easy_fixture_hides_it]] — «absolute coordinates» = comandos, não espaço; a fixtura ÓBVIA concorda com as duas leituras — só o caso ANINHADO as separa
- ⛔⛔ (Tags W3a, 13/09) **as TRÊS mutações sobreviventes de uma volta foram a MESMA espécie: a fixtura não continha o fenómeno** — e as três eram fixturas *naturais*, escritas sem esforço: criar as tags por ordem alfabética faz a ordem dos IDS coincidir com a da ÁRVORE (a mutação que as troca fica verde) · clicar no CENTRO de um rect acerta no alvo tanto se o registo for o `×` como se for a pílula inteira (o gesto não distingue; o discriminador é a GEOMETRIA do rect) · e um `y = f(…)` sobre uma secção cujo snapshot é `None` é um **no-op**, logo deitar o `y` fora não empilha nada. ⇒ *escreva a fixtura pelo que a mutação tem de mover, não pelo que é natural escrever*, e ponha no gate a metade **«a fixtura produz o fenómeno»** (aqui: `assert_ne!(ordem_por_id, ordem_da_arvore)`)
- ⭐⭐ (mesma volta) **um gate fraco esconde um ARGUMENTO invertido tão bem como esconde um defeito** — o doc ao lado do código dizia *«assim os chips não saltam ao renomear»* e a verdade é o contrário (com ordem de árvore eles saltam, de propósito; é a ordem dos ids que os deixaria parados). O código estava certo, a razão escrita ao lado não, e **só a mutação sobrevivente mandou relê-la**
- ⛔⛔ **Uma junta de OFFSET ZERO num esqueleto importado envenena tudo abaixo dela** (BVH da CMU, 14/09): `LowerBack`, `Neck` e `LeftShoulder` estão no MESMO ponto do pai — são ajudantes de rotação. Mapear o nosso marcador da RAIZ no `LowerBack` deu um par de comprimento zero, e um marcador degenerado dá um ângulo qualquer: o erro de representação lia **43 cm em TODOS os onze clipes, sempre no mesmo osso**. *Uniformidade suspeita entre fixturas independentes é assinatura de defeito do importador, não do dado.*
- ⭐ **Um portão de «cabe no nosso modelo?» tem de julgar por GRUPO, não por média** (mesmo dia): num salto capturado, as PERNAS e o TRONCO são planos (6 a 11 cm fora do plano) e os BRAÇOS não são (50 cm — a pessoa cruza os braços). Uma média única recusava o clipe inteiro e deitava fora pernas perfeitamente úteis — e o trabalho da vez era justamente a perna.
- ⚠️ **Diferença em centímetros entre corpos de proporções diferentes não é erro de representação** — é proporção. A canela do sujeito é 54 cm e a nossa 40; dali só vêm os ângulos. Um número em cm nessa comparação mede o corpo, não a lei.
- ⛔ **Todo gate e todo smoke no ponto de OMISSÃO de um knob** (16/09, `Resolution` do modelador): o defeito só existia com o botão acima de `1`, e o corpus inteiro estava no `1`. É a lei «um corpus no neutro de um knob não testa esse knob» (Motion, 31/08) noutro módulo — varra o knob no gate.
- [[feedback_choosing_faces_does_not_drop_positions_and_the_orphans_hijack_the_cursor]] — recortar por FACES deixa 721 órfãos; o «vértice mais próximo» aterra na metade deitada fora e a queixa aponta para o verbo
