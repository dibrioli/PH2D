---
name: topic-repro-discipline
description: "Família: reprodução e diagnóstico — harness, medição antes de causa, controles positivos"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-07-21T01:04:08.371Z
---

# Disciplina de reprodução/diagnóstico (índice de família — detalhe em cada arquivo)

- [[feedback_a_gesture_report_needs_a_fixture_containing_the_gesture]] — report de GESTO pede fixture com o gesto; chamar a mutação direto pula a costura input→bus→drain
- [[feedback_harness_reproduces_mechanism_not_context]] — harness reproduz mecanismo, não contexto; instrumente o app real
- [[feedback_a_windowed_drive_races_the_real_cursor]] — drive janelado disputa com o cursor FÍSICO/WM; re-afirme a posição por frame; anomalia só-em-alguns-runs = ambiente
- [[feedback_nonreproduction_is_not_proof_of_fix]] — não-reprodução ≠ correção; cheque o `git diff`
- [[feedback_first_case_rescued_by_side_effect_test_repetition]] — 1º caso salvo por efeito colateral; teste a REPETIÇÃO
- [[feedback_try_to_build_the_harness_before_declaring_it_impossible]] — CONSTRUA o harness antes de desistir ("o App exige janela" era falso)
- [[feedback_a_negative_search_needs_a_positive_control]] — busca negativa pede controle positivo; grep vazio mente
- [[feedback_remeasure_a_documented_residual_before_curing_it]] — re-meça resíduo anotado; a causa E o número podem estar errados
- [[feedback_measure_perf_symptom_scale]] — meça a ESCALA antes da causa (frame 4-16ms vs ⅓s muda a classe)
- [[feedback_a_frontier_is_not_a_census]] — fronteira não é censo; o custo é da POSIÇÃO; meça antes de construir sobre um "N" herdado
- [[feedback_a_conservative_verdict_must_separate_unchanged_from_unmeasurable]] — colapsá-los mata a otimização com todos os gates VERDES
- [[feedback_a_cited_number_whose_probe_lost_its_caller_stops_being_reproducible]] — devolva a chamada e confira o valor, nunca silencie o lint
- [[feedback_a_persistent_default_bug_lives_in_a_reset_path_not_the_create_path]] — enumere toda porta que reconstrói o estado (new/default/purge/load)
- [[feedback_an_approximation_inside_a_fixed_point_walks_it_does_not_merely_err]] — tabela num laço de realimentação: meça deriva sob iteração, não erro de chamada única
- [[feedback_probes_that_measure_parallelism_must_run_alone]] — concorrentes disputam o pool e medem uma à outra; o controle interno é o detector
- [[feedback_a_component_missing_its_contract_suspect_the_caller_first]] — trocar o componente esconde a causa e costuma trazer um 2º defeito
- [[feedback_a_per_pass_gain_becomes_a_product_gain_only_through_the_cadence]] — 1,56× virou 1,10×; e razão não se transporta entre cenas
- [[feedback_a_rule_copied_to_a_second_site_may_lose_its_premise]] — ablacione um braço por vez; o outro sítio pode ser o controle positivo
- [[feedback_a_flattening_curve_may_need_more_points]] — curva de 4 pontos que achata pode ser uma de 6 que nao; nao declare 2o mecanismo cedo
- [[feedback_a_defect_count_without_provenance_names_the_wrong_phase]] — N defeitos sem PROVENIENCIA culpa a fase errada; a decomposicao e' o plano de trabalho
- [[feedback_two_meanings_behind_one_primitive_only_the_parameter_name_guards]] — `VecPathId` e bits de entidade sao ambos `u64`: o compilador nao separa, e um gate escrito da ASSINATURA fica verde sobre o panico
- [[feedback_reproduce_with_the_real_constructors_and_look_at_the_image]] — foto com desenho: reproduza com os construtores reais, grave o PNG, LEIA-o antes de nomear a causa
- **O SÍTIO onde um defeito ATERRA não é o que o PROVOCA** (report do dono, 18/09 — *«o desenho não
  encolhe ao abrir a timeline»*). A medição dizia que um botão do HUD ficava clicável em `y 728..848`,
  que é **onde a faixa da timeline é desenhada**; eu escrevi *«com a timeline aberta»* em quatro
  páginas, dois gates e os passos de smoke. O único escritor de um `CenterSplit ≠ None` é o
  `if motion_active` do `ph2d_app_motion::motion_bridge_surfaces` — a **ferramenta Motion**. ⚠️ O
  custo não é a palavra: *um doc que nomeia o gatilho errado manda o próximo reproduzir onde o
  defeito não pode acontecer, e ele conclui que o defeito não existe.* ⇒ antes de escrever o gatilho
  num doc, **meça quem ESCREVE a grandeza** (`grep` no escritor), nunca onde o sintoma aparece.

---

## ⛔⛔ O `~/.ph2d/` é PARTILHADO por todas as worktrees (2026-09-18)

Uma foto de smoke apanhou o `~/.ph2d/layout.txt` com o espaço de trabalho errado, e eu atribuí-o —
**sem medir** — às minhas próprias corridas. Falso, e a medição tem três passos:

1. a cena irmã aprovada não o muda, e a minha também não;
2. a suíte da shell corrida com `HOME` num directório temporário **não escreve em lado nenhum**, e o
   ficheiro do dono mudou **na mesma** durante essa corrida;
3. `pgrep -af ph2d-host-desktop` + `ls -l /proc/<pid>/cwd` → um app a correr de **outra worktree**.

⇒ *O ficheiro de preferências vive fora do repositório e é UM só para todas as árvores* — quem corre
o app por último ganha, e uma linha vê o ambiente mudar sem nada no seu diff.

⚠️ **E eu «repus» o valor duas vezes antes de medir**, o que era disputá-lo com um processo VIVO de
outra linha. ⛔ A régua é **não lhe tocar**; quem precisa de arrumação determinística pô-la na
própria cena (`panel_visibility`, a selecção, o espaço de trabalho).

⭐ *Um ambiente partilhado desmente-se com `/proc`, não com raciocínio sobre o próprio diff.*
