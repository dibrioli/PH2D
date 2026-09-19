---
name: reference-topic-mutation-proofs
description: "Provas de mutação — as regras do placar"
metadata:
  node_type: memory
  type: reference
---

- [[feedback_mutate_the_code_not_just_the_test]] — verde na mutação = gate frouxo ou comentário errado
- [[feedback_mutation_red_only_counts_on_a_seen_green_gate]] — vermelho nos 2 mundos prova nada; ciclo verde→red→verde
- [[feedback_check_the_oracle_is_achievable_before_writing_the_gate]] — o prescrito pode ser impossível
- [[feedback_an_optimization_needs_a_gate_that_proves_it_fires]] — o fallback silencia o bug
- [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] — explique por que é inofensiva ALI
- [[feedback_a_restored_file_keeps_its_old_mtime_and_cargo_reuses_the_mutant]] — `touch` depois de todo restore
- [[feedback_a_mutation_harness_needs_a_positive_control_that_a_test_ran]] — exija `running 1 test`; filtro que não casa corre ZERO e sai 0
- [[feedback_a_mutation_anchor_must_be_the_whole_expression]] — `tr("x")` dentro de `ph2d_i18n::tr("x")` gera mutante que não compila, e o `cargo-test-narrow.sh` sai 1 (não 2) com `0/0`: âncora = expressão inteira; conta só com o NOME do teste a reprovar
- [[feedback_nextest_error_line_makes_a_mutation_script_read_killed_as_uncompiled]] — o nextest imprime `error: test run failed` num teste VERMELHO: `^error:` leu 4 mortas como «não compilou»; compilação é `error[E…]`/`could not compile`
- ⛔ (Tags W3a, 13/09) **o alias de ficheiro do CORPO colide com a variável do ARNÊS**: os aliases das mutações chamavam-se `T`/`I`/`S`/`P`, e o `S` do arnês é a pasta dos LOGS ⇒ ele escreveu `…/sections/tags.rs/1-controlo.log`, e **os 18 controlos saíram «inválidos» de uma vez**. Falha alto, mas o sintoma (*«o controlo não corre»*) aponta para a árvore, não para o script — perdi duas corridas inteiras a procurar no sítio errado. ⇒ **aliases de DUAS letras**, e o arnês diz o porquê ao lado do `S=`
- ⛔⛔ (mesma volta) **um `assert` que aborta o python seguido de `;` deixa a corrida seguinte ler o ficheiro POR CORRIGIR** — e ela imprime um veredito com cara de medição. É a irmã do `str.replace()` mudo ([[feedback_python_replace_silent_noop_after_fmt]]): ali o no-op é silencioso, aqui ele é ALTO e o `;` cala-o. ⇒ encadeie a correcção e a corrida com **`&&`**, e imprima a VERIFICAÇÃO (o `grep` do símbolo novo) antes de correr
- (W148, 13/09) **um gate de IMAGEM não prende uma política de serviço de cache**: servir a fita fora da região certa sobreviveu a três deles, porque o corte guarda arestas com folga generosa e a imagem sai igual *quase sempre* — a propriedade gateia-se no `get` (`docs/3DModeling/12` §12.9.1)
- [[feedback_a_mutation_that_deletes_a_cap_allocates_what_the_cap_prevented]] — fixtura de tecto = `TECTO + ε`; com `u32::MAX` o mutante alocou 27 GB
- ⚠️ **UMA lei com DOIS guardas devolve «SOBREVIVEU» sobre produto CORRECTO** — mutar um só deixa o outro a tapar o buraco (3× em dois dias na `line/components`: o `Spawned` no reconcile+query, o dedup+`is_ok` do dreno, o `MasterRoot` na escrita+leitura). ⇒ o arnês precisa de uma variante de **duas agulhas**, e só UM dos guardas costuma ser observável sozinho
- ⚠️ **Uma agulha que não muda a ORDEM não prova um gate de ordem** — envolver a chamada num bloco é um no-op; o que sangra é fazer o marco aparecer **duas vezes** (um marco sem posição)
- ⛔ **`--lib` não casa teste nenhum numa SHELL** (ela não tem biblioteca): o arnês lê «CONTROLO inválido» e o filtro certo é `--bins`
- ⛔⛔ **Uma cerca que repete o que a LINGUAGEM já garante lê-se como a cerca que falta** (2026-09-16,
  `line/components`): o dreno convertia `f32 → u32` com `round().clamp(0, u32::MAX)` e **a mutação
  que apagou o `clamp` não matou gate nenhum** — em Rust um `as` de vírgula flutuante para inteiro
  **satura** desde a 1.45 (`-3.0 as u32` é `0`). ⇒ apagou-se a cerca, escreveu-se a razão no doc, e
  a mutação passou a atacar o que NÃO é de graça (o `round`).
- ⛔ **Uma fixtura que parte do valor por OMISSÃO não distingue «recusou» de «escreveu o mesmo»**
  (mesma wave): o gate do índice fora-da-lista começava no `Point` e uma cura falsa que caísse no
  `Point` devolvia `false` na mesma. ⇒ a fixtura parte de um valor que **não** é o do fallback.
## ⛔ Um corpus onde uma DESIGUALDADE nunca aperta não testa a desigualdade (2026-09-16)
O corte espacial do índice usa dois limites conservadores com a FLECHA do arco (minorante e
majorante). Das 8 mutações da cura do arco, **as duas que esqueciam a flecha SOBREVIVERAM** a cinco
gates verdes — nenhuma região do corpus caía onde a flecha decide o corte. **How to apply:** quando
uma lei é uma desigualdade conservadora, construa À MÃO a região onde ela aperta (a conta ao lado:
«cintura» — o arco curva para a caixa e uma recta fecha o `dmax`; «pé» — a corda encosta à caixa e o
arco foge), e só então a mutação morre.
- ⛔⛔⛔ **UM RELATÓRIO DE SOBREVIVÊNCIA QUE ESCONDE OS VERMELHOS MANDA PROCURAR NO SÍTIO ERRADO** (Teste Cascadeur, 19/09): o arnês imprime, quando a mutação é dada por sobrevivente, **só os portões ESPERADOS que ficaram verdes** — e um portão vermelho que ninguém nomeou aparece como `(+1 outro)` **quando ela morre** e **desaparece quando ela sobrevive**, que é exactamente o caso em que ele é a informação. A mutação acusada **morria**, na fixtura da metade SIMÉTRICA da lei; o `esperados` é que nomeava o portão do ramo OPOSTO. ⇒ a sobrevivência imprime também os vermelhos que ninguém nomeou, **e diz em voz alta quando não há nenhum** (*«a linha mutada não é observável»*) — são as duas leituras opostas, e até aqui eram o mesmo texto. ⚠️ E o achado de fundo é o outro: aquele portão não tinha **nenhuma** mutação que o matasse, porque a única que o nomeava não o alcançava. *Um portão que nenhuma mutação mata não está provado — está a ser acreditado.*
