---
name: reference-topic-control-design-hazards
description: "Família: como um CONTROLO mente — morto vs ausente, faixa derivada do que ele escreve, rótulo que promete o que o modelo não dá, variante removida do meio de um enum"
metadata:
  type: reference
---

⚠️ **Índice de família, 2 saltos.** Saíram do `MEMORY.md` porque ele passou o limite de leitura e
as últimas linhas desapareciam em silêncio. Cada entrada continua a ser um ficheiro próprio.

*Pergunta-mãe: **o que este controlo promete, e o que ele de facto faz ao modelo?***

- [O desenho pedido pode já ser LEI noutra metade do app — procure antes de desenhar](feedback_the_design_being_asked_for_may_already_be_law_in_another_half_of_the_app.md)
- [MORTO ≠ AUSENTE: procure o controlo que JÁ faz a pergunta antes de construir](feedback_a_dead_control_and_an_absent_one_read_the_same_and_building_is_the_wrong_cure.md)
- [Knob morto: 2 espécies que gate nenhum apanha — siga o TERCEIRO passo; e a 3.ª LEITURA: consumidor GENÉRICO lê-se como consumidor NENHUM](feedback_a_dead_knob_has_two_species_no_probe_catches.md) · [id ÓRFÃO ≠ knob MORTO: curas opostas](feedback_an_orphan_id_and_a_dead_knob_read_the_same_and_their_cures_are_opposite.md) · [smoke que ensina o CONTRÁRIO é pior que ausente](feedback_a_smoke_scene_that_teaches_the_opposite_is_worse_than_no_scene.md)
- [«Difícil de ajustar» = bug de DESIGN](feedback_ergonomics_verdict_is_a_design_bug.md)
- [Faixa tirada do objecto que o botão substitui = não idempotente](feedback_a_knob_whose_range_is_derived_from_the_object_it_rewrites_is_not_idempotent.md)
- [Knob por-passo é ALVO](feedback_a_knob_consumed_as_a_per_step_rate_is_a_target_not_a_rate.md) · [remédio novo = contagem dupla](feedback_a_new_remedy_makes_the_old_one_double_counting.md) · [param inerte: grepe o consumidor](feedback_a_parameter_that_changes_nothing_is_discarded_downstream.md)
- [Campo colapsado MANDA](feedback_a_collapsed_field_does_not_go_neutral_it_takes_over.md) · [rótulo promete o modelo](feedback_a_label_must_promise_what_the_model_delivers.md) · [affordance se re-deriva](feedback_inherited_affordance_must_be_rederived.md)
- [⛔ Numa lista aplicada por varredura, NÃO nomear é comandar DESLIGADO](feedback_not_naming_a_thing_in_an_absolute_list_is_commanding_it_off.md)
- [Tirar variante do MEIO de um enum serializado reescreve ficheiros gravados, sem erro](feedback_removing_a_middle_variant_from_a_serialized_enum_silently_rewrites_saved_files.md)
- [bool onde havia ID apaga a próxima](feedback_publishing_a_bool_where_the_source_had_an_id_throws_away_the_next_feature.md) · [não-idempotente: autoria ≠ depósito](feedback_a_nonidempotent_target_excludes_nothing_split_authoring_from_deposit.md)
- [[feedback_when_the_composition_already_expresses_the_geometry_ask_who_owns_the_numbers]] — «a composição já exprime isto?» tem duas metades: a GEOMETRIA e a AUTORIA. Se o dono dos números os reescreve por quadro, a linha de painel nasce MORTA — e é isso que decide se a forma nova existe
- [[feedback_a_palette_derived_from_a_registry_offers_the_implementations_vocabulary]] — ⛔ paleta «tudo o que está registado» oferece TIPOS onde o artista escolhe INTENÇÕES (30 de 32 na física, 27 deles rows que a secção já anexa); a régua é *anexar isto sozinho muda alguma coisa?*, e o que a produziu foi o **helper por-família cujo caminho de menor esforço era uma das respostas**
- ⛔⛔⛔ **Duas ordens do dono podem CONTRARIAR-SE, e a spec tem de escolher com a medição à vista.**
  Medido 2026-09-15 (`line/UIUX`): *«Label acima do campo numérico! Muito ruim!»* põe o nome ao lado e
  entrega ao controlo **metade** da linha; *«não permita que a caixa seja redimensionada para menor
  que isso»* põe um piso de `72 px` por caixa. Numa row de `X`/`Y` à largura de omissão a coluna do
  controlo mede `128` e dois campos ao piso pedem `148` — **as duas não cabem**. ⇒ a saída não é
  encolher a coluna do nome (a granularidade dela é a SECÇÃO; por-linha devolve a coluna
  esfarrapada), é o **controlo REFLUIR**: o que não cabe ao piso desce, dentro da coluna do controlo.
  ⚠️ **E o app já lá tinha chegado, à mão, numa row só** — o comentário de um `Rect2Editor` dizia por
  escrito *«the Inspector column is too narrow for four number inputs in one row»* enquanto as outras
  dezoito rows não a conheciam. *Uma lei escrita num sítio é uma nota; só uma PORTA é uma lei.*
- ⛔⛔ **Um rótulo que descreve DUAS caixas («A / B») só funciona enquanto elas estiverem lado a
  lado.** Medido 2026-09-15: três rows da §Animation empacotavam duas propriedades num nome
  (`Frame ms / Repeat (0 = forever)`); com o nome POR CIMA o mapeamento era esquerda→direita, e ao
  pôr o nome AO LADO com as caixas a refluírem **desaparece** — um nome que descreve duas caixas
  EMPILHADAS não diz qual é qual. ⇒ *mudar a disposição de uma linha FORÇA o corte de uma linha que
  fazia duas perguntas*, e isso é ganho, não custo.
