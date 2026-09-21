---
name: reference_topic_architecture_north_perf_lessons
description: Arquitectura, norte e performance — as lições dobradas do índice (leis de desenho pagas com medição), verbatim, uma linha por memória
metadata:
  type: reference
---

# Arquitetura / norte / perf — lições dobradas do índice (2026-09-10)

> Cada linha é uma memória com o gancho ORIGINAL do índice; abra o ficheiro para o mecanismo.
> Dobradas aqui porque o `MEMORY.md` passou o teto suportado (32 KB > 24 KB): ficam a dois saltos.

- [Conteúdo de asset é PARTILHADO; per-objeto é QUAL asset](feedback_the_content_of_an_asset_is_shared_only_which_asset_is_per_object.md)
- ⭐ [Porta que o VIZINHO não chama ainda não é porta — a mesma pergunta errada 4×, a última a 15 linhas dela; só o CENSO a fecha](feedback_a_door_the_neighbour_does_not_call_is_not_a_door_yet.md)
- [«O mais recente possível» ≠ «o mais recente»: conte os TETOS](feedback_the_newest_possible_is_not_the_newest_count_the_ceilings_first.md) · [duas cópias podem ser o MECANISMO, não resíduo](feedback_two_copies_of_a_dependency_can_be_the_mechanism_not_the_residue.md)
- [Dois motores, um estado](feedback_two_engines_one_state_is_worse_than_a_slow_engine.md) · [contrato congelado escolhe a arquitetura](feedback_frozen_contract_can_pick_the_architecture.md)
- [Tipo em N sítios → componente opcional](feedback_widely_constructed_type_favors_optional_component_over_appended_field.md) · [a representação apaga o caso especial](feedback_the_representation_can_delete_the_special_case.md)
- ⭐ [Antes de acrescentar um CAMPO, veja se a AUSÊNCIA já é o estado — e se ela já tem leitores](feedback_before_adding_a_field_ask_whether_the_absence_is_already_the_state.md)
- ⭐ [Cópia profunda que leva TODO componente leva o ELO: duas entidades com a mesma identidade = sósia que não se move](feedback_a_deep_copy_that_copies_every_component_also_copies_the_identity_link.md)
- [Invariante na DERIVAÇÃO](feedback_enforce_the_invariant_at_the_derivation_not_at_each_gesture.md) · [marca de evento é canal próprio](feedback_a_transient_event_marker_is_its_own_channel.md)
- [A recusa que responde é a do knob VIZINHO — grepe a MÉTRICA](feedback_the_measured_refusal_you_need_is_in_the_neighbouring_knob.md)
- [Rejeição cuja explicação descreve outra obra = PRÉ-REQUISITO](feedback_a_rejection_whose_explanation_describes_another_work_is_a_prerequisite.md)
- [Sonda depois do passo que ARRUMA mede a arrumação](feedback_a_ruler_placed_after_the_tidying_step_measures_the_tidying.md)
- [Sonda no ramo do FRACASSO de A não vê os acertos de A](feedback_a_probe_in_the_failure_branch_cannot_see_the_other_sides_successes.md)
- ⭐ [Defeito ESCONDIDO atrás de outro: o gate faz `continue` sobre entrada inválida e fica cego — curar o 1.º descega o instrumento, e o que aparece NÃO é regressão](feedback_a_defect_can_hide_behind_another_defect_and_blind_the_very_gate_that_would_find_it.md)
- [Correlação sem contra-exemplos pode descrever DISPONIBILIDADE, não correcção](feedback_a_correlation_with_zero_counterexamples_may_describe_another_question.md)

## Descidas do índice em 2026-09-17 — a rodada de seis linhas pôs o `MEMORY.md` a **224 linhas / 36,5 KB** contra o tecto dele (140 / 17 KB), e o carregador cortou **66 linhas em silêncio**. Estas entradas descem VERBATIM; o ponteiro para esta família continua no índice.

- ⛔⛔ [Uma vista NOVA entra ao LADO da que os consumidores já lêem, nunca no lugar dela — 24 leitores tratavam `contours()` como a figura e estavam certos; 2 gates velhos apanharam-no](feedback_a_new_view_cannot_replace_the_one_consumers_read.md)
- ⛔ [Sonda que arma o módulo por env var mede OUTRO programa que o pill (5 reports) — e a do arco cronometrava a PLACA com o dono no modo MODEL, que traça na CPU (12 196 facetas contra 0)](feedback_a_probe_that_arms_a_module_by_env_var_measures_another_program_than_the_pill.md)

- ⛔⛔⛔ [Comutar duas leis por um LIMIAR não dá um salto: dá CHATTER — ajuste a DIFERENÇA, que é zero onde nada há a corrigir](feedback_a_boolean_over_a_continuous_quantity_is_a_step.md)

- ⭐⭐⭐ **Quando um MECANISMO está completo e o produto não o usa, o que falta é uma POLÍTICA — e o sítio onde ela mora mede-se em gates partidos** (2026-09-21, o Inspector): a dobra de secção tinha tudo (chevron, clique, animação, recorte) e **nenhuma secção nascia dobrada**, logo o painel desenhava `2,5`–`6,5` ecrãs contra uma dobra de `880 px`. ⚠️ Posta no `Panel::populate` do painel, a política reprovou **342** gates da crate dele; movida para a porta de arranque do EDITOR, **4**. ⇒ *o painel declara o que PODE mostrar; quem compõe o editor declara como ele ABRE* — e o painel nem conhece a altura da janela. ⛔ E o valor de fábrica não foi escolhido: o orçamento da dobra dá para UMA secção, e medidas uma a uma só a `Transform` cabe (`849 px`; a `Render` no lugar dela dá `1 000`).
