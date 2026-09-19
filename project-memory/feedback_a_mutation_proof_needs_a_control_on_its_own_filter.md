---
name: feedback_a_mutation_proof_needs_a_control_on_its_own_filter
description: "Uma prova de mutação com filtro de teste que casa ZERO testes imprime \"SOBREVIVEU\" — conte quantos correram antes de acreditar"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-27T23:48:23.502Z
---

Numa prova de mutação, o filtro (`cargo test -p X <filtro>`) é parte do instrumento. Se ele
casar **zero** testes, o cargo sai `0` e o script imprime **"SOBREVIVEU"** — indistinguível de
um gate genuinamente vazio, e na direção que faz desistir da cura.

Aconteceu em 2026-08-27 (auditoria do Motion): o filtro `cook_lazy` casava zero, porque o
caminho do módulo é `cook::lazy::tests`. Duas mutações reportaram "sobreviveu"; com o filtro
certo as duas morreram.

⚠️⚠️ **E há uma segunda forma, PIOR, porque o controle «quantos correram?» passa nela:** o filtro
casa **o teste VIZINHO**. Em 2026-09-05 (`line/components`) o filtro `every_menu_row` casou
`every_menu_row_reaches_a_handler` — 1 teste, verde — enquanto o gate que a mutação atacava era
`every_painted_menu_row_is_registered_and_therefore_clickable`, noutro ficheiro. Contar deu `1` e o
veredito foi *"SOBREVIVEU"*. ⇒ o controle honesto não é *«correu alguma coisa?»* mas **«correu o
gate que eu nomeei no comentário da mutação?»** — dois ficheiros de gate cujos nomes partilham um
prefixo são a armadilha, e eles partilham prefixo **de propósito** (são a mesma família).
⚠️⚠️ **Aconteceu DUAS vezes na mesma jornada, e a segunda foi DEPOIS de esta nota ser escrita:** `hit_indexed_ids_are_registered` é *estruturalmente* cego a registos guiados por tabela, e o gate que via a mutação era o irmão `table_driven_chips_are_registered_too`. *Um filtro que casa 1 teste passa no controle «quantos correram?» e mente na mesma.*

**Why:** a prova de mutação afirma *"este gate reprova quando o produto quebra"*. Ela tem duas
metades — a mutação chegou ao binário, e o gate correu. Um filtro errado mata a segunda em
silêncio, e o mesmo vale para um `--test` com nome errado ou um crate sem o alvo.

**How to apply:** antes de acreditar num "SOBREVIVEU", faça o arnês **contar quantos testes
correram** com aquele filtro e abortar se for zero (ou menos que o esperado). É o mesmo controle
positivo que se exige de qualquer outra sonda — ver [[reference_topic_mutation_proofs]] e
[[feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate]]. E prefira
`bash scripts/cargo-test-narrow.sh <crate>` sem filtro quando a suíte for barata: o filtro só
compra tempo, e este é o preço dele.

⛔ O irmão: `| head`/`| tail` num comando cujo veredito importa destrói o exit code
([[feedback_pipe_masks_script_exit_code]]) — na mesma sessão o portão de fecho imprimiu
`error: test run failed` e reportou sucesso.

⚠️⚠️ **E uma TERCEIRA forma, na direcção contrária: o resumo do script lê igual para «filtro
vazio» e para «não compila».** Medido em 2026-09-06 (`line/components`): o
`cargo-test-narrow.sh` imprimiu `✗ — 0 falharam · 0 passaram · 0 ignorados` sobre um ficheiro de
teste com um erro de assinatura, que é **a mesma linha** que ele imprime quando o filtro não casa
nada. O script tem exit codes distintos (`1` teste vermelho · `2` não compila) e **a linha de
resumo não os distingue** — foram precisas duas leituras e um `cargo test` cru para ver o erro.
⇒ num controlo de filtro, leia o **exit code**, nunca só a contagem: `0 passaram` com `exit 2`
não é um gate vazio, é uma árvore partida.

⚠️⚠️⚠️ **E em 2026-09-19 o MESMO arnês mentiu QUATRO vezes numa corrida, cada uma na direcção que
faz desistir — as quatro em `line/UIUX`, a armar a varredura de elisões:**

1. **`error: test failed` casa `^error:`.** A cerca de *«a mutação compila?»* era um `grep` por
   `^error(\[|:)`, e o cargo imprime `error: test failed, to rerun pass …` quando um teste fica
   VERMELHO — que é exactamente o que uma mutação a sangrar produz. **As seis mutações leram-se
   como *«NÃO COMPILA»*.** ⇒ a cerca é `error[E<n>]` ou `could not compile`, nunca `^error:`.
2. **`running 1 test` é SINGULAR.** O contador de *«quantos testes correram?»* casava
   `running <n> tests`, e **quatro** das seis mutações atacavam um gate só ⇒ o contador leu vazio e
   imprimiu *«O FILTRO CASOU 0 TESTES»* sobre mutações que sangravam à primeira. *Um arnês que só
   conhece o plural é cego a toda mutação de um gate.*
3. **Um caminho de BACKUP relativo, resolvido depois do `cd`.** `BAK=$(dirname "$0")/bak` com o
   `mkdir` antes do `cd` para a raiz: as cópias de backup e de restauro passaram a apontar para uma
   pasta que não existe e **falharam em silêncio** — as seis mutações **ficaram na árvore**, e a
   corrida seguinte leu-as como *«a mutação não entrou»*. ⇒ caminho **absoluto**, o backup conferido
   com `-s` depois de escrito, e o restauro **conferido por `sha256sum`** contra o hash de antes.
4. **A suíte que corria em FUNDO mediu a árvore MUTADA e saiu VERDE.** Ela foi lançada antes das
   mutações e terminou no meio delas ⇒ o `93 passed` não afirmava nada. É
   [[feedback_a_harness_that_writes_to_the_real_tree_cannot_be_parallel]] outra vez, e a cura é a
   mesma: *nada corre em paralelo com um arnês que escreve na árvore de verdade.*

⭐ **A leitura das quatro é uma só:** cada uma faz uma mutação **honesta** ler-se como um defeito do
instrumento (*não compila* · *filtro vazio* · *não entrou*), e as três primeiras só apareceram
porque as seis mutações sangravam de verdade — *um arnês estreado contra mutações que MORREM
esconde os próprios furos, porque «✗» lê-se como trabalho a fazer no produto*.
