---
name: a-restored-snapshot-resurrects-its-id-counter
description: Preview que restaura cena + re-insere por frame re-cunha o MESMO id — e herda estado de entidade velho (settle/transform) em silêncio; declare o preview como GESTO
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 51a8018f-e2ee-4a90-b494-25aafe003cd8
  modified: 2026-07-20T21:28:33.606Z
---

Um preview vivo que faz `scene.clone_from(&pre)` + re-insere o resultado a cada frame restaura **também o contador de ids** da cena — o resultado renasce com o MESMO id todo frame, o `sync` mantém a entidade do frame 1, e qualquer sistema por-entidade que reagiu à "geometria nova" (o `settle_origins`, que assentou: geometria→local, Transform→centro) deixa estado que o frame 2 **herda sobre geometria diferente** (mundo × centro = pose dobrada — o "pula pro canto direito" do Offset vivo, `9c0446df`, 2026-07-20).

**Why:** id igual ≠ mesma coisa: o conteúdo é re-derivado por frame, mas o estado da ENTIDADE (assentamento, ordem, nome) é write-once e sobrevive ao restore. E a fonte restaurada tem o problema espelho: a entidade dela morreu no churn, então "voltar ao original" (d≈0 no meio do drag) respawna na identidade e desenha na ORIGEM.

**How to apply:** preview que reescreve geometria por frame é um **GESTO** — entra na mesma lista que a caneta (`drawing` do `settle_origins`), e assenta UMA vez, no release. Fonte não-consumida tem 3 destinos distintos (zona morta pré-churn = cena intocada · d≈0 pós-churn = cópia assada em MUNDO · aniquilada = some do frame). Gate de unidade espelha o frame (sync+settle) e **não vê a render_loop** — o sítio real precisa de arch-gate próprio (MUT2: 16 unit-gates verdes com o chain removido). Ver [[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]] e [[feedback_harness_reproduces_mechanism_not_context]].
