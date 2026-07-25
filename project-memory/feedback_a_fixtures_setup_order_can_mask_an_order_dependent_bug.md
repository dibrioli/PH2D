---
name: feedback-a-fixtures-setup-order-can-mask-an-order-dependent-bug
description: A ordem de SETUP de um fixture/smoke esconde bugs dependentes de ordem — espelhe a ordem do PRODUTO
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ac1a9702-6b56-4e69-aa92-f36f1c65684e
  modified: 2026-07-25T05:20:46.558Z
---

Um smoke/teste que arma o estado na ordem **conveniente** pode passar sobre um bug que só aparece na ordem do **PRODUTO**.

Caso real (line/Vector, envelope, 2026-07-24): "a gaiola não aparece, mas o efeito se aplica" (Enio). O smoke `=27` criava o envelope e **só então** selecionava a forma (tudo num frame) → o pen mudava → `sync_selection` promovia filho→container **por acidente** → a gaiola aparecia; verde. Mas o artista faz o **oposto**: seleciona a forma e **depois** clica Envelope, em frames separados. Enveloparr re-parenteia o filho **sem tocar o pen**, então a promoção não rerodava (nem o pen mudou, nem o conjunto vetorial do gizmo) e o gizmo ficava no FILHO — sem gaiola, com alças de nó. O `create()` até DEVOLVIA o container "para a seleção/gizmo", mas o render_loop descartava o valor.

**Why:** o bug era de ORDEM (reparent-sem-pen-change), e o fixture escolhera a ordem que dispara a promoção por outro caminho. Igual à família [[reference_topic_fixture_discipline]] (o fixture só prova o que contém) — aqui o que faltava conter era a **sequência**.

**How to apply:** quando o report descreve um GESTO ("acrescentei X", "cliquei Y depois de Z"), o gate/smoke tem de reproduzir a MESMA ordem, em frames separados se o produto os separa. Antes de confiar num verde, pergunte: *meu setup chega ao estado pelo mesmo caminho que o usuário, ou por um atalho?* O fix aqui foi mutação-provado (`enveloping_a_selected_shape_promotes_the_gizmo_to_the_container`) E o smoke passou à ordem do produto (`=27`: seleciona frame 4, envelopa frame 5). Ver também [[feedback_derived_coordinate_seed_must_match_sample]] (a mesma doença no eixo do tempo).
