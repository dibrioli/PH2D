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
- ⛔⛔⛔ **UMA MUTAÇÃO QUE NÃO PARSEIA LÊ-SE EXACTAMENTE COMO UMA LEI FRACA** (Teste Cascadeur, 19/09): o `de` da mutação casava só o `if (...)` e o comentário `// MUTAÇÃO` **engolia o corpo** do `if`; o ficheiro deixou de ser JavaScript, a suíte morreu antes da 1.ª linha, não houve `OK` nem `FALHOU` nenhum — e o relatório escreveu *«nenhum portão reprovou»*, que é letra por letra o que ele escreve sobre uma lei que ninguém gateia. *Um zero de «não correu» e um zero de «correu e passou tudo» são o mesmo byte.* ⇒ toda corrida mutada compara a **POPULAÇÃO de portões** com a do controlo e acusa com o erro do node ao lado (controlo positivo: `A SUÍTE NÃO CORREU … SyntaxError`). ⚠️ É a metade OPOSTA da lição já registada no arnês de gesto, onde *uma mutação que não compila lê-se como SANGRAR* — o mesmo defeito lê-se ao contrário conforme o arnês, e por isso os dois precisam do mesmo controlo.
- ⛔⛔ **Um portão que mede se algo foi ESCRITO não pode partilhar o sujeito com quem escreveu antes dele** (mesma corrida): o portão *«a lei não escreve na entrada»* fotografava o array que os portões de cima já tinham usado — com a lei a escrever na entrada, aquele array vinha **já achatado**, e uma lei que não tem nada para tirar não escreve nada. ⇒ o sujeito constrói-se **do zero**, por uma função, dentro do próprio portão.
- ⛔⛔⛔ **PARA GATEAR UMA PROPRIEDADE DE FASE É PRECISO UMA FIXTURA QUE TENHA FASE** (Teste Cascadeur, 19/09): a mutação que torna uma média móvel **causal** (olhar só para trás) SOBREVIVEU — e com razão, porque uma média causal ainda cobre um período inteiro e ainda anula a oscilação. O que a fase zero compra é a **HORA**: o que é guardado atrasa-se meia janela. Sobre uma **rampa** isso é invisível (uma recta atrasada é a mesma recta com outro offset) ⇒ a fixtura ganhou uma **corcova** e o portão passou a medir em que quadro o que ficou atinge o pico (`q21` contra `q22`). *Uma fixtura monótona não pode reprovar um filtro que só erra no tempo.*
- ⛔⛔ **«Um corpus no ponto neutro de um osso não testa esse osso» cobrou-se DUAS vezes na mesma wave** (mesma corrida): primeiro no antebraço, depois na COLUNA — e a segunda foi **duas horas depois de eu registar a primeira**. A cura é a mesma e tem de ser feita para TODA a população que a lei podia alcançar, não só para a que a mutação daquele dia nomeou: a fixtura passou a mexer em ossos de fora do escopo, **com um controlo a provar que ela os mexe**, senão o `0,0000 cm` do portão é vácuo.

## ⛔⛔ UMA MUTAÇÃO QUE O CORPUS NÃO DISCRIMINA LÊ-SE COMO UMA LEI QUE NÃO EXISTE (2026-09-19)

A cerca *«um tuplo reconhece-se pelo `(` que o abre»* foi testada com duas linhas de `match` lado a
lado — e ali ela é **inerte**: depois de emparelhar por `=>` o percurso segue sem guardar o valor,
logo o valor nunca chega a ser candidato a chave. A mutação que a apagava **SOBREVIVEU**.

Quem a discrimina é um **ARRAY** (`&["Paint", "Erase"]`): ali há uma vírgula entre dois literais e o
que abre é um `[`. Com a fixtura certa, sangra.

**How to apply:** antes de escrever a mutação, pergunte **por que caminho** o corpus faz a lei
correr. Se a fixtura não contém o fenómeno, a sobrevivência não diz nada sobre a lei — diz sobre a
fixtura. Irmã de [[feedback_a_corpus_at_full_strength_cannot_test_the_strength_curve]].

## ⛔⛔⛔ UM ACESSÓRIO DERIVADO DA TABELA É INDISTINGUÍVEL DA PORTA CERTA (2026-09-19)

Escrevi um gate de costura: *«o rótulo do chip é igual ao que o `tr` devolve para a chave»*. Ele
**passou com o defeito vivo**, e a mutação que devolvia o pintor ao acessório inglês **sobreviveu** —
porque `label()` **é** `tr_em(Ingles, chave)` e o processo de teste corre em inglês: os dois lados
dão a MESMA string. *Nenhum teste de IGUALDADE separa os dois; eles só divergem numa segunda língua.*

⛔ E não há saída por `tr_em(Teste, …)`: quem escolhe a língua do `tr` é um `OnceLock` do PROCESSO,
e escrever-lhe torna a suíte mais um membro da família de flakes de fan-out.

⇒ **o instrumento tem de ser TEXTUAL** (quem CHAMA o acessório), e o teste vácuo foi **apagado**:
*um teste vácuo é pior do que nenhum — ele lê-se como cobertura.*
- ⛔⛔ [Restaurar com `git checkout` numa árvore SUJA apaga a FATIA, não a mutação](feedback_a_mutation_restore_by_git_checkout_deletes_the_wave.md) — 3 ficheiros perdidos; quem o disse foi o controlo do filtro.
- ⛔⛔⛔ **O arnês tem de perguntar se o teste está VERDE antes de mutar — senão ele certifica
  qualquer coisa.** Medido 2026-09-19 (`line/Vector`): um gate meu estava vermelho (a régua esperava
  a fracção ao longo da CORDA e o código devolve o parâmetro da CURVA — numa quina os pontos de
  controlo colapsam e a posição avança com `3t² − 2t³`, o *smoothstep*, logo `0,3` de corda são
  `0,375` de parâmetro), e **todas** as mutações sobre ele imprimiram *«SANGRA»*. *Um teste já
  vermelho não discrimina nada.* ⇒ o arnês corre o filtro **antes** e recusa (`exit 5`) se já houver
  `test result: FAILED`. ⚠️ É a QUARTA cegueira deste arnês, ao lado das três que ele já cobria (o
  filtro que casa zero e imprime `ok` · a mutação que não compila e se lê como sangrar · o `mv` que
  devolve mtime antigo).
- ⭐⭐ **E a cura da régua foi trocar o número medido pelo PRODUTO:** em vez de afirmar o parâmetro,
  afirmar **onde o ponto nasceu** — a pergunta do artista, que não depende da parameterização e mata
  na mesma o valor cravado. *Uma régua escrita sobre uma grandeza interna herda a convenção dela.*

- ⛔⛔ [Guarda de ramo inalcançável e cerca a jusante do estrago: as duas mutações sobrevivem, e a cura é APAGAR](feedback_a_line_the_mutation_cannot_kill_is_not_law.md)
