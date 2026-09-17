---
name: reference-topic-integration-discipline
description: Integração de linhas paralelas (Modo L) — ordem medida · estágios do índice · checkout --ours · mesmo símbolo · números que somam · marcadores · latentes do ship (12)
metadata: 
  node_type: memory
  type: reference
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-08-23T00:59:19.635Z
---

- [[feedback_integration_order_comes_from_measured_overlap]] — a ordem de integração se MEDE (sobreposição par-a-par); quando o Enio fixa a ordem, siga-a e meça cada linha contra o main DE AGORA
- [[project_integrator_ship_catches_latents_budget_iterations]] — o ship do integrador drena latentes: 2-4 iterações é a mediana (22/08: tofu do motion-value · BloomParams do Sprite · pill MODEL · uma flake)
- [[feedback_clean_text_merge_can_be_semantically_broken]] — merge limpo pode estar quebrado: `check --workspace` (22/08: campos novos em `BloomParams` vs inicializador de outra linha)
- [[feedback_resolve_conflicts_from_index_stages_not_markers]] — resolva pelos ESTÁGIOS `:1` base `:2` ours `:3` theirs, nunca pelos marcadores
- [[feedback_checkout_ours_discards_the_hunks_git_already_merged]] — `checkout --ours` DESCARTA os hunks limpos de theirs que o git já fundiu; resolva só a região marcada ou reaplique todos
- [[feedback_two_lines_curing_the_same_report_is_a_same_symbol_collision]] — duas linhas curando o MESMO report do Enio = mesmo símbolo; fórmulas iguais ⇒ uma lei que passa as DUAS suítes; diferentes ⇒ Enio
- [[feedback_a_shared_list_is_merged_against_todays_main]] — lista compartilhada funde contra a main de HOJE: só ADICIONE; remover é integração
- [[feedback_sweep_conflict_markers_every_commit]] — varra marcadores (inclusive `|||||||`) em CADA commit rebaseado
- [[feedback_foundational_editable_design_for_isolation]] — foundational editável = crie isolado (módulo irmão, append-only); anote ids
- [[feedback_numbers_that_sum_across_lines_count_dont_pick]] — números que SOMAM (PROJECT_SCHEMA, registros, ADR) se CONTAM: 84+2+3 = 89, e cada degrau renumerado diz de que número nasceu
- [[feedback_collision_surface_reads_the_fork_point_not_the_tip_of_main]] — ⚠️ a sonda lê o MERGE-BASE, não o main de agora: da 2ª linha em diante ela imprime `base: 95` para um main já em 96, e a colisão de mesmo-literal passa MUDA. Leia o valor no main à mão, ou rode a sonda DEPOIS do rebase (24/08: 96 vs 96, o certo era 97)
- [[project_integration_prefork_lines_ship_drift]] — integrar linhas pré-cutover = drift
- [[feedback_bash_cwd_resets_and_slips_to_the_primary]] — a cwd do Bash VOLTA ao primário: todo comando de worktree começa com o `cd` dela (22/08: diagnostiquei «os arquivos da linha sumiram» no repo errado)
- [[feedback_seven_of_eight_integration_failures_are_properties_of_the_sum]] — ⭐⭐⭐ **o custo de uma rodada não é o tamanho das linhas, é a PRIMEIRA vez que cada família de falha aparece** (medido 17/09, seis linhas, ~9 h): **uma** linha comeu `5h36` e as outras cinco `~1h30`; as **duas últimas** — uma com 92 commits e sete conflitos — custaram **26 min juntas**, porque quando chegaram as famílias já eram conhecidas e havia dois scripts escritos a meio do caminho. ⛔ **`7` das `8` falhas eram propriedades da SOMA e nenhuma linha as vê sozinha por construção**: o censo de texto (HR-15) é escrito pela linha `X` e o literal que o acorda pela linha `Y` (**5**), e o tecto de LOC soma entre linhas sem ninguém a contar (**2**). ⛔⛔ **E o CI não os corre** (o job de teste é um `-p` de ~25 pacotes; eles vivem em `tests/it/`) ⇒ o ÚNICO sítio onde são descobertos era o portão do integrador, **em série**. ⇒ a cura é a LINHA correr `bash scripts/censos-da-arvore-combinada.sh` depois do `git rebase main` que o protocolo já manda (DIRETRIZ §1.5.9 item **5-bis**, `/pd-linha-fechar` passo 2-bis): converte N descobertas em SÉRIE em N descobertas em PARALELO, e **quem cura o literal é quem sabe se ele chega ao ecrã**. Anatomia medida: `docs/archive/integracao-jornadas/ANATOMIA_DE_UMA_RODADA_2026-09-17.md`
- [[feedback_size_the_cut_by_the_ratchet_not_by_generosity]] — ⛔ **tecto de LOC vermelho: corte pelo excesso que a catraca IMPRIME.** Em 17/09 ela pedia `711` linhas e o corte levou `4 470` (6×); o excesso arrastou um gate que lia um ficheiro por caminho em RUNTIME e quatro ficheiros de teste que o `git mv` deixou para trás — **uma corrida inteira de portão a mais**. ⚠️ `cargo check --all-targets` é **CEGO aos testes de uma DEPENDÊNCIA**: quem os apanha é `cargo nextest list`
