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
- ⛔⛔ [E restaurar com `git checkout --` APAGA o gate novo ainda não commitado — a corrida seguinte fica verde por não haver régua](feedback_a_mutation_restore_by_git_checkout_deletes_the_new_gate.md)
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
- ⛔⛔ **Um arnês que verifica o próprio RESTAURO contra o `HEAD` mede se a LINHA está commitada** (2026-09-19): o `git diff --quiet -- <ficheiros>` no fim do script gritou *«A ARVORE FICOU SUJA»* sobre um restauro perfeito — a linha tinha trabalho por commitar, logo o `HEAD` difere **por construção**. ⇒ compare-se com o BACKUP (`cmp -s "$f" "$TMP/$(basename "$f").orig"`), que é a única coisa que o arnês de facto prometeu repor.
- ⛔ **Um CONTROLO que não percorre o mesmo `match` da metade positiva não prova a selectividade dela** (mesma wave): o gate «só os símbolos de rig se penduram» usava a `Circle` como controlo, e ela é uma das OITO formas tratadas **por nome**, com braço próprio que nunca chega ao braço mutado ⇒ trocar o `_ => box_` por `_ => rig_box` **sobreviveu**. A 2.ª forma do controlo (`Cross`) passa pela rota genérica e mata-a. *Um controlo ilibar a normalização e um controlo ilibar o despacho são duas afirmações diferentes.*

---

## Um arnês de mutação mente de TRÊS maneiras, e as três leem-se como «sobreviveu» (2026-09-20)

Numa wave só, o mesmo arnês deu **cinco** falsos sobreviventes sobre produto correcto:

1. **A mutação não ENTROU.** Os blocos de `python3` que a aplicavam foram gerados por outro
   `python3`, e um `\n` dentro de uma string foi expandido cedo demais ⇒ `SyntaxError`, o ficheiro
   ficou **intacto**, e o teste passou (como devia). *Uma mutação que não aplica é indistinguível
   de uma que sobrevive.*
2. **A cerca é do COMPILADOR.** Uma lei guardada por `const _: () = assert!(…)` reprova antes de
   qualquer teste correr, e um arnês que só sabe ler `cargo test` conta isso como «zero testes».
3. **A AGULHA envelheceu com a ferramenta.** A busca pela mensagem do rustc procurava
   `evaluation of constant value failed` e o rustc de hoje escreve `evaluation panicked:
   assertion failed: <a nossa expressão>`.

**Why:** um arnês de mutação é um instrumento como outro qualquer, e o modo de falha dele é
**silencioso e a favor** — ele imprime o veredito que o autor quer ver. As três formas partilham a
causa: *nada no arnês verifica o PRÓPRIO arnês.*

**How to apply:** o arnês precisa de três controlos, um por forma —
`cmp -s` do ficheiro contra o backup (**recusa se não mudou**) · um caminho para leis cuja cerca é
o compilador · e uma agulha que seja **texto NOSSO** (a expressão da asserção), nunca a prosa de
uma ferramenta. ⚠️ E some-se a isto o controlo que esta casa já tinha: **contar quantos testes de
facto correram**, porque um filtro que casa zero imprime `ok`. Ver
[[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]].

## ⛔ `running 1 test` é SINGULAR — o controlo do filtro tem de contar os dois (2026-09-18)
O arnês de mutação da `line/sculpt3d` ganhou um **controlo sobre o próprio filtro** (reprova se ele
casar menos de N testes), depois de um filtro vazio imprimir `ok` e se ler como *sobreviveu*. Mas o
extractor era `grep -oE 'running [0-9]+ tests'` e **o cargo escreve `running 1 test`, no singular**
⇒ com uma corrida de um teste só ele lê `0` e acusa *«ARNÊS QUEBRADO»* sobre uma mutação que
**sangrava**. **Why:** é a 4.ª forma conhecida de um arnês de mutação mentir, ao lado do filtro que
casa zero, da mutação que não compila e do `| tail` que destrói o exit code. **How to apply:** o
padrão é `running ([0-9]+) tests?`; e ⭐ esta falha é do lado **SEGURO** — ela acusa-se a si mesma em
vez de se ler como sobrevivência, que é como todo controlo de arnês devia falhar. Ver
[[reference_topic_measurement_discipline]].

## ⭐⭐⭐⭐ Uma mutação SOBREVIVENTE às vezes diz que a EXPLICAÇÃO está trocada, não que falta um gate (2026-09-20)
A `line/sculpt3d` construiu uma cerca de forma nova para destravar um knob que o portão do produto
recusava, mediu `9 → 0` lascas e ia shipar a história *«a cerca nova é mais esperta e destrava»*. A
prova de mutação incluía *«a cerca desaparece do caminho do produto»* — e ela **ficou VERDE**, o que
não podia acontecer se a cerca fosse o que fazia o portão passar. Medido a sério pela porta do
produto: **sem cerca nenhuma também dá zero**, e quem dava `4` lascas era a cerca **ANTIGA**. ⇒ a
cerca antiga era a **FÁBRICA** do defeito que existia para impedir. **Why:** o reflexo ao ver um
sobrevivente é *«falta-me um gate»*, e a leitura mais valiosa é *«o meu modelo do porquê está ao
contrário»* — ali o `9 → 0` vinha de uma sonda com arnês escrito à mão e a porta do produto dizia
outra coisa. **How to apply:** antes de escrever o gate que mata o sobrevivente, pergunte se ele é
**refutação da causa**: corra a tabela nas TRÊS configurações (com a peça nova · com a antiga · com
NENHUMA) pela porta do produto. A terceira coluna é a que quase nunca se mede e foi a que decidiu.
Ver [[reference_topic_measurement_discipline]].

---

## ⛔⛔ O arnês pode depender de uma ferramenta que a MÁQUINA não tem (`bc`), e aí ele aborta TUDO

`line/sculpt3d`, 2026-09-20 (a caixa de cor). O arnês contava *«quantos testes
de facto correram»* — o controlo positivo que esta família prescreve — somando
as linhas `test result:` com `… | paste -sd+ | bc`. **Esta máquina não tem
`bc`**: a soma vinha **vazia**, o `${corridos:-0}` lia `0`, e as **nove**
mutações saíram `ABORTA: o filtro correu ZERO testes` — sobre um filtro que
corria `4` e `6` testes.

**Why:** o modo de falha foi o **conservador** — ele abortou alto em vez de ler
*«sobreviveu»* —, e é isso que torna o incidente barato em vez de caro. Mas o
diagnóstico custou uma corrida inteira, porque o sintoma (*«o filtro não casa»*)
aponta para o **filtro**, e a causa estava no **somatório**. É a mesma forma do
alias `S` colidir com a pasta de logs, um bloco acima: *o arnês acusa a árvore
quando o defeito é dele.*

**How to apply:** um arnês de mutação só pode depender de `sh`, `awk`, `grep`,
`python3` e `cargo` — tudo o resto confirma-se antes (`which`), ou não se usa.
E quando **todas** as células abortam com a mesma mensagem, a hipótese nº 1 é
**o arnês**, nunca o código: uma corrida que acusa nove de nove não está a medir
nove coisas, está a medir uma.

⚠️ E na mesma jornada ele mentiu uma **segunda** vez, também para o lado seguro:
o `cargo fmt` juntou uma chamada de três linhas numa só **depois** de a prova ter
corrido, e a agulha multi-linha passou a casar **zero** vezes — irmã de
[[feedback_python_replace_silent_noop_after_fmt]], e do porquê de
[[feedback_a_restored_file_keeps_its_old_mtime_and_cargo_reuses_the_mutant]].
⇒ *toda prova de mutação re-corre depois de um `fmt`*, e a contagem da agulha
faz-se em Python: **`grep -cF` conta LINHAS, não ocorrências.**

---

## ⛔⛔ Uma mutação sobrevivente sobre uma HEURÍSTICA não pede um gate — pede a medição no CORPUS (2026-09-20, `line/sculpt3d`, o corte do atlas)

Escrevi dez linhas de heurística (*«antes de abrir peça nova, oferece a face a uma peça
vizinha»*) e a mutação que as apagava deixou **os 25 gates verdes**. A leitura fácil é
*«falta um gate»*; a certa é **medir no produto** — e a medição disse que ela **troca de
sinal entre peças**:

| corrida | com a heurística | sem ela |
|---|---|---|
| peça A | `227` | `226` |
| peça A (remalhada) | `92` | `93` |
| peça B | `122` | `129` |
| peça B (remalhada) | `89` | `88` |

⇒ *uma heurística que ganha numa peça e perde noutra não é uma alavanca; é ruído com dez
linhas de código.* Foi **apagada**, com a tabela ao lado para quem a quiser reconstruir
saber o que compra. ⚠️ **E a lição de método é o denominador:** com UMA peça eu teria lido
`227 → 226` e concluído *«é inerte, apaga»*, ou `122 → 129` e concluído *«vale 5 %,
fica»* — **as duas leituras erradas, da mesma medição feita numa amostra de um**.

⭐ A regra que fica: quando uma mutação sobrevive, pergunte primeiro *isto é uma LEI ou
uma HEURÍSTICA?* Uma lei sem gate escreve-se o gate; uma heurística sem gate mede-se no
corpus inteiro e ou ganha sempre, ou sai. Ver
[[feedback_a_line_a_mutation_cannot_kill_is_a_comment_with_code_syntax]].

---

## 30 — Um ramo DEFENSIVO sem chamador lê-se, numa mutação, exactamente como um ramo MORTO (2026-09-20)

`ph2d-mesh-colors`: trocar o discriminante `4 if face[3] == TRI => 3` por `=> 4`
**sobreviveu aos 18 gates da crate**. A causa não era uma fixtura em falta — era
que **nada no produto percorre aquele ramo**: a `ph2d_mesh::Face::verts()`
devolve `&self.0[..vert_count()]` e **corta o sentinela antes de sair**.

⚠️ **A pergunta que a mutação faz não é *«falta um gate?»*, é *«quem chama isto?»***
— e as duas respostas têm curas OPOSTAS: um ramo morto apaga-se, um ramo
defensivo gateia-se. Aqui ele ficou, com o mecanismo escrito: a porta aceita
`&[u32]` cru, o array de uma `Face` desta casa é `[u32; 4]` com `u32::MAX` no
4.º slot, e um chamador que passe `&face.0[..]` em vez de `face.verts()` lê um
triângulo como quad com um canto `u32::MAX` — `index out of bounds` ou endereços
trocados **em silêncio**.

## 31 — Uma PERMUTAÇÃO é invisível a uma régua que CONTA (2026-09-20)

Na mesma crate, inverter o `t` do lado `d→a` de um quad (`lado - j` por `j`)
sobreviveu a **19** gates, incluindo a bijecção — que é o gate mais forte da
crate e cujo doc promete provar a fronteira partilhada. ⭐ **Inverter o `t` de
uma aresta é uma permutação do bloco dela, logo a contagem de índices distintos
fica IGUAL AO BIT.** O que a apanha é a igualdade por **ponto FÍSICO**: duas
faces que se tocam têm de devolver o mesmo índice para o mesmo sítio.

⚠️⚠️ E o gate que já media isso tinha a fixtura errada de espécie: dois
**TRIÂNGULOS**. O único quad do corpus estava **sozinho**, onde não há vizinho
com quem discordar. ⇒ *o TAMANHO de uma fixtura deriva-se da pergunta*: os
quatro lados de um quad têm de ser partilhados pelo menos uma vez, senão o ramo
que não é cruzado fica sem régua — numa fita de dois quads o gémeo `c→d` ficava
de fora, e a mutação escrita depois prova-o. A fixtura é uma grelha `2×2`.

Ver [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] ·
[[feedback_a_surviving_mutation_can_mean_the_code_is_redundant]] ·
[[reference_topic_measurement_discipline]].

## ⛔⛔ Um ABORTO mudo de arnês de mutação lê-se exactamente como uma mutação que não entrou (2026-09-20)
A 1.ª corrida do arnês da tinta fina deu `9 de 15` com **cinco** abortos *«zero testes correram»* —
e um deles era o **CONTROLO** (uma linha em branco, que não pode abortar). Corridas à mão provaram
que as cinco **SANGRAVAM**: a `M6` isolada dá `rc = 101`, `257` testes contados e o teste certo
vermelho. **Why:** as três mentiras que este repo já tinha registado (âncora que casa zero · mutação
que não compila · contador que lê só o `passed`) foram todas curadas por uma GUARDA que aborta — e
a guarda passou a ser o novo sítio onde a informação se perde: *um instrumento que se declara
inconclusivo sem dizer porquê não é mais honesto que um que mente, e o número final é o mesmo nas
duas leituras*. **How to apply:** todo ramo de ABORTO imprime a EVIDÊNCIA (as últimas linhas da
corrida), e quem lê um relatório com abortos re-corre pelo menos um deles à mão antes de acreditar
no placar.
Ver [[reference_topic_gate_discipline]].

## ⛔⛔⛔ E a EVIDÊNCIA que esse aborto passou a imprimir acusou o BINÁRIO DE TESTE, não a mutação (2026-09-20)
Com o ramo de aborto a falar, as **cinco** mensagens eram a mesma: `signal: 11, SIGSEGV` no
processo do `--lib` da crate. *O arnês reportava o próprio acidente* — um crash mata o processo
inteiro e leva o `test result:` com ele, logo a contagem lê `0` e o caso cai no ramo de aborto.
**Why:** um arnês que corre a suíte num processo ÚNICO herda todo defeito de processo daquela
crate, e esse defeito converte-se em «inconclusivo» em TODAS as mutações — não numa. **How to
apply:** o `corrida()` de um arnês de mutação usa `cargo nextest run` (**um processo por teste**),
e a população é o `N tests run` do `Summary`; ⚠️ **nunca** o `running N tests` do libtest, que
CONTA os `#[ignore]`. Depois da troca: `296 tests run: 296 passed`, zero abortos, e o placar passou
de `8 de 16` para **`15 de 16`** com a 16.ª a ser o CONTROLO.
Ver [[reference_topic_gate_discipline]] · [[reference_topic_measurement_discipline]].

## A prova de mutação HERDA a cegueira da corrida que ela usa (2026-09-21, `line/sculpt3d`)

O `corrida()` de um arnês roda `nextest run -p … --lib` — **sem `--ignored`**. Logo toda lei cuja
única prova de comportamento viva num gate `#[ignore]` (nesta casa: **tudo o que precisa de um
`wgpu::Device`**) é lida pelo arnês como **lei sem régua**, e a mutação dela **SOBREVIVE**.
Medido: a cura da tinta fina saiu com `20 de 25` a sangrar, e as **quatro** sobreviventes reais
(M19–M22) eram exactamente as quatro metades escritas nesse dia — os três consumidores da porta
`o_gesto_muda_a_topologia` e a metade da tinta EMPRESTADA. A prova de comportamento delas existia,
no `tinta_no_produto_tests.rs`, e é `#[ignore]` + placa; **o CI também nunca a corre** ⇒ uma
regressão na cura seria silenciosa em todo o sítio onde alguém a fosse procurar.
**Why:** o arnês não estava errado — ele disse a verdade, e a verdade é que a lei não tinha régua
ALCANÇÁVEL. *Um sobrevivente não é sempre um gate fraco: às vezes é um gate que ninguém corre.*
**How to apply:** ao ler um relatório de mutação, pergunte de cada sobrevivente **se o gate dela é
`#[ignore]`** antes de a chamar gate frouxo. Se for, e se o consumidor não for construtível sem
placa (`Sculpt3dScene::new` pede um device), a cura é o **censo de TEXTO do ELO**, com as duas
metades obrigatórias: cortar a PROSA antes de medir (um doc-comment que explica a cura contém o
nome da porta) e o CONTROLO inverso — cada agulha tem de estar **ausente** da metade comentada.
⚠️ E **dois** gates quando a mutação *substitui* em vez de apagar: um vê a ausência da agulha certa,
o outro a presença da errada. Depois disso (com a 5.ª mutacao, a do gesto que erra a peca): `25 de 26`, com o unico sobrevivente a ser o CONTROLO.
Ver [[feedback_a_bins_run_never_reaches_the_gates_that_live_in_tests]] · [[reference_topic_gate_discipline]].

## O 4.º controlo: a corrida LIMPA tem de estar VERDE (2026-09-21)

Os arneses desta casa controlavam-se em tres pontos (a ancora casa uma vez · a mutacao compila · N > 0
testes correram) e faltava o mais barato: **`verde=$(corrida | populacao)` deita fora o codigo de
saida**, logo com a arvore ja' VERMELHA antes de mutar **toda** mutacao le-se como `SANGRA` e o placar
sai perfeito. **Why:** o erro e' para o lado que ninguem investiga — *um placar cheio nao faz ninguem
olhar duas vezes*. **How to apply:** `limpa=$(corrida); rc=$?` e abortar com `rc != 0`, imprimindo as
linhas de falha. ⚠️ E **a ordem no ficheiro importa**: uma mutacao acrescentada DEPOIS do `echo` do
sumario corre e imprime o veredito dela **abaixo do total**, que fica por contar (`24 de 25` com a 25.ª
a sangrar por baixo) — *um total impresso antes do ultimo caso mede outra populacao*.

## Copiar a arvore com o arnes A CORRER congela a mutacao viva (2026-09-21)

O arnes fotografa `$APP`/`$PAN`, aplica **uma** mutacao de cada vez e restaura (inclusive num
`trap EXIT`). Copiar essas arvores enquanto ele corre captura o estado **MUTADO**, e parar a tarefa e
restaurar dessa copia **congela a mutacao no produto**. Medido: a copia de emergencia gravou a `M1`
(os dois bracos do `match` do `garante` identicos), o passe de compilacao fechou **VERDE** — os dois
compilam — e quem a apanhou foi a suite: `883` testes, `1` vermelho em `0,004 s`, com a mensagem do
gate. **Why:** a mutacao viva e', por construcao, uma edicao que COMPILA; o laco interno e' cego a ela
e ela e' indistinguivel de um defeito proprio. **How to apply:** nao editar nem copiar `$APP`/`$PAN`
com o arnes a correr (pare-o e deixe o `trap` restaurar) · depois de um restauro de emergencia corra a
SUITE antes de acreditar na arvore · uma reprovada deterministica e isolada logo a seguir e' a
assinatura: procure a ancora dela no `muta_*.sh` **antes** de suspeitar do seu codigo, porque ali o
nome da mutacao diz a cura · e nao deixe uma tarefa de GPU em fila enquanto ele corre (ela pode ganhar
a placa a meio de uma mutacao e construir a arvore mutada).
⚠️ Mordeu DUAS vezes no mesmo dia: a segunda comeu uma edicao de cabecalho feita durante a corrida.
- ⛔⛔⛔ (tinta fina, 21/09) **UM ARNÊS QUE CRESCE ATÉ AO PRAZO DA FATIA É MORTO A MEIO, E ISSO É UMA MUTAÇÃO CONGELADA À ESPERA.** Medido: `35` mutações × `~50 s` de `nextest` = **~30 min**, que é o prazo do `ph2d-run.sh` — a corrida foi morta na **M25** com a árvore inteira, e o `trap restore` só salva se o `bash` receber o sinal. **Why:** o placar não é o único produto de um arnês; o **estado da árvore** também é, e um arnês morto a meio não diz qual dos dois deixou. **How to apply:** dê-lhe um **filtro** (`MUTA_FILTRO=<regex>` contra o nome) e corra por fatias · o sumário tem de **DIZER que é parcial**, porque *um placar parcial lido como completo é a forma mais barata de um arnês mentir* · e a seguir a um kill **confira a árvore antes de acreditar nela** (`grep` pelos substitutos das mutações + `git diff --stat` das crates que o arnês faz backup), nunca por inferência a partir da última linha impressa.
- ⛔⛔ (mesma volta) **UMA FIXTURA QUE SOBRESCREVE O CAMPO QUE ESTÁ A TESTAR DEIXA A MUTAÇÃO SOBREVIVER.** O gate montava a identidade por `IdDoPlano { amostras: len + 64, ..IdDoPlano::de(&t) }` — a mutação punha `amostras: 0` na `de`, e o `..` do gate reescrevia-o ⇒ a desigualdade continuava a valer **pelo campo que a fixtura pôs**, e o gate passava. **Why:** um `..Default`/`..de(x)` com um campo sobrescrito é exactamente «não meço este campo». **How to apply:** quando um gate existe para um campo, a fixtura tem de o deixar vir da PORTA e mover **outra** grandeza. ⭐ E a cura verdadeira foi melhor que o gate: o campo respondia a uma **segunda pergunta** (*«os meus índices cabem?»*) enfiada numa igualdade de identidade — partido em duas cercas, cada uma ficou gateável, e a redundante desapareceu.
- ⛔⛔⛔ (tinta fina, 21/09) **DUAS CERCAS EM SEQUÊNCIA: a fixtura que faz a PRIMEIRA disparar torna a SEGUNDA inobservável — e a cura é a FIXTURA, não o código.** A rede da cerca do plano deu `5 de 8` com dois sobreviventes que não podiam sobreviver, um deles a própria reprodução do pânico do dono. Causa: a fixtura eram quads triangulados, e ali a recusa dispara em `f = 0` pela cerca dos **CANTOS** (um triângulo onde o plano espera um quad) ⇒ apagar a cerca da **CONTAGEM** não mudava um bit, e o `out` nunca chegava a ter nada dentro para a asserção do *«não deixa registo meio escrito»* morder. **Why:** é a irmã do [[feedback_a_fence_that_never_bites_hides_the_one_that_does]] com o papel trocado — lá a cerca a mais mascara as outras, aqui é a **ordem do laço** mais uma fixtura que satisfaz as duas condições ao mesmo tempo. Uma fixtura que dispara DUAS cercas mede UMA. **How to apply:** uma fixtura por cerca, cada uma a violar **exactamente uma** das condições (aqui: mais faces com os mesmos cantos · menos faces · mesma contagem com outros cantos) · e o caso REAL do report fica com gate próprio cujo doc DIZ qual das cercas ali dispara, senão a que sobra fica sem régua. ⭐ E as três fixturas isoladas são todas reais: a das contagens é o `refine_for_dab` (triângulos em triângulos) e a dos cantos é a triangulação do pen-down.
- ⭐⭐ (mesma volta) **UMA MUTAÇÃO QUE SOBREVIVE POR FALTA DE CORPUS É PARA CONSTRUIR, NÃO PARA NOMEAR.** A metade dos VÉRTICES da cerca do device sobreviveu porque o veredito face-a-face já recusava toda a fixtura do gate; nomeá-la seria honesto e mais fraco. O corpus que faltava é uma malha com as MESMAS faces e um vértice **ÓRFÃO** a mais (`Mesh::from_parts` aceita-o), que é a única forma de pôr as duas réguas a discordar — com ele, `8 de 9` sangram. **How to apply:** antes de escrever «NOMEADA», pergunte que ENTRADA separaria as duas réguas; se ela for construtível, o nome é a saída cara.
- ⛔⛔⛔ (tinta fina, 21/09) **UMA AGULHA QUE É UM FRAGMENTO MEDE O FRAGMENTO, NÃO A CHAMADA.** O censo de elo do atalho do upload perguntava por `matches!(line.job, SlotJob::Full)` e esse texto aparece **duas vezes** no mesmo ficheiro (a outra é a condição que decide se vale a pena subir o plano) ⇒ a mutação que troca o ARGUMENTO por `false` deixava a outra ocorrência a satisfazer o censo, e **SOBREVIVEU** com o gate verde. **Why:** é a mesma lei que este repo já escreve para as âncoras de mutação (*«âncora = expressão inteira»*), do lado do CENSO — e ali ela morde mais, porque um censo é escrito precisamente onde não há comportamento para medir. **How to apply:** `grep -c` a agulha no ficheiro antes de a aceitar; se não for `1`, a agulha é a chamada INTEIRA (todas as linhas dos argumentos), e o gate imprime-a na mensagem de falha.
- ⭐⭐⭐ (mesma volta) **DEPOIS DE `cargo fmt`, AS ÂNCORAS TÊM DE SER RECONFERIDAS — e o instrumento tem de ser BARATO, senão ninguém o corre.** O `fmt` reescreve a indentação de uma âncora, ela passa a casar ZERO, e **isso lê-se exactamente como uma mutação que sobreviveu** — ao preço de uma corrida inteira para descobrir. ⇒ `MUTA_SO_ANCORAS=1` nos dois arneses desta família: salta a corrida limpa e o `corrida()`, conta cada âncora e sai `1` se alguma não casar exactamente uma vez (`43 de 43` e `9 de 9` em segundos). ⚠️ **E o sumário dele DIZ que zero testes correram** — *um pré-voo que imprimisse «N de N sangram» seria um instrumento a descrever-se mal, que é o defeito que o arnês inteiro existe para não ter.*


---

## A POPULAÇÃO DE UM ARNÊS é de quem OBSERVA a mutação, nunca de quem a CONTÉM

Um arnês que muta a crate `A` e corre `nextest -p B -p C` devolve **SOBREVIVEU** para toda mutação
em `A`: a crate mutada nem chega a ser compilada, e o placar é fabricado sem um único sinal.
**Medido** em 2026-09-22 (`line/sculpt3d`, o aviso da saída da tinta fina): a `S6` mutava
`ph2d-app-field3d` com a população em `-p ph2d-mesh -p ph2d-app-sculpt3d` ⇒ `5 de 7` com um
sobrevivente que não podia sangrar. ⚠️ **O cabeçalho do próprio arnês já escrevia a lei que ele
violava** (*«uma corrida só de uma delas leria VERDE sobre a mutação da outra»*), escrita para DUAS
crates por quem depois mutou uma TERCEIRA. **How to apply:** a pergunta não é *«que crate eu
mutei?»*, é ***«que TESTE vê isto?»***. ⭐ A cura barata quase nunca é acrescentar a crate à
população (isso paga uma suíte inteira em **cada** mutação): é pôr o **elo de texto** no censo que
já corre — um `include_str!` relativo alcança uma crate irmã sem dependência nenhuma. ⚠️ E o
sobrevivente fabricado **esconde a pergunta real**: *aquele lado está gateado de todo?* Ali não
estava. Ver [[reference_topic_gate_discipline]].

## ⛔⛔ Uma fixtura recusada por DOIS motivos não afirma NENHUM dos dois (2026-09-20, `line/components`)

- ⛔⛔ Uma fixtura recusada por DOIS motivos não afirma NENHUM dos dois (2026-09-20, `line/components`)

A lei nova dizia *«uma tabela do Luau só é um valor se ela se DECLARAR (`__kind`)»*, e o gate tinha
três fixturas: uma **lista** `{1,0,0}`, uma com marca **errada**, e **meia** (`__kind` certo e um
campo a menos). A mutação que assume a marca em falta (*«se não tiver, é um `vec2`»*) **SOBREVIVEU**
— porque cada uma daquelas tabelas também era recusada por **não ter os campos**, logo nenhuma
media a marca. **Why:** três casos que parecem cobrir a lei podem estar todos a cair num segundo
`return None` a jusante dela; *uma população onde toda fixtura falha duas vezes lê-se como cobertura
e é vácuo*. **How to apply:** para cada cerca, construa a fixtura que **só** ela recusa — aqui um
registo com `x` e `y` **e sem marca** —, e a prova de mutação é quem diz se ela existe. Ver
[[reference_topic_gate_discipline]] e [[reference_topic_measurement_discipline]].

- ⛔⛔ Um `| head` mata o arnês de mutação por SIGPIPE e deixa o PRODUTO MUTADO na árvore (2026-09-22, `line/components`)

O arnês guarda o ficheiro, muta, corre o filtro e **restaura**. Canalizado por `head` para ler só o
princípio da saída, ele morre com **SIGPIPE entre o `muta` e o `restaura`** — e a mutação fica no
código. A corrida seguinte leu `⛔ ANCORA: … aparece 0 vezes`, que é o arnês a ser honesto; mas uma
que não tocasse naquela âncora teria corrido a suíte **sobre código mutado** e chamado ao resultado
verde. **Why:** o repo já tem escrito que um `| head` destrói o *exit code*; isto é pior — ele
destrói o **estado da árvore**, e a janela em que o defeito é invisível é exactamente a de quem
muda de assunto a seguir. *Um arnês que restaura no caminho feliz não restaura; ele restaura quando
nada corre mal.* **How to apply:** `trap ao_sair EXIT INT TERM PIPE` com a última mutação guardada
numa variável, e a saída do arnês vai para um FICHEIRO (`> mut.txt`) em vez de um pipe. Ver
[[reference_topic_ship_ci_integration_lessons]].

- ⛔⛔ Um CRASE dentro de `"..."` no bash abre substituição de comando, e o erro sai 50 linhas abaixo (2026-09-22, `line/components`)

Renomear uma prova para `bloco "a forma deixa de ser byte-identica a` lei da W1"` — o `` a` `` é o
«à» que este repo escreve sem acento em shell — **desfez a citação de todo o resto do ficheiro**: o
crase abre `` `…` `` mesmo dentro de aspas duplas, engole tudo até ao crase seguinte, e o que o bash
acusou foi `erro de sintaxe próximo ao token inesperado '('` **numa linha 50 abaixo**, num bloco que
eu não tinha tocado. **Why:** a mensagem aponta para onde a gramática por fim quebra, nunca para
onde a citação abriu; e num ficheiro cheio de português sem acentos o `` a` `` parece inofensivo.
⚠️ E o arnês **não estava partido**: ele nem chegou a correr, logo a corrida anterior — que reportou
sobreviventes reais — continuava a ser a verdade. **How to apply:** `bash -n <script>` antes de o
correr (custa nada e diz a linha), e nada de crases em prosa dentro de um script shell — nem em
comentários, porque uma linha de continuação (`\`) leva o comentário para dentro do comando.

- ⛔⛔ Um passo IDEMPOTENTE no fim de uma cadeia apaga toda mutação de ORDEM que não o REMOVA de lá (2026-09-22, `line/components`)

A paralaxe compõe `confinar → derivar → envolver`, e o envolvimento reduz o deslocamento módulo um
ladrilho. Duas mutações de ordem — antecipar o envolvimento — **sobreviveram**, e não por a régua
ser fraca: o envolvimento do fim **re-envolve**, e o resultado cai na mesma janela. *A mutação
duplicava o passo em vez de o mover.* **Why:** um passo idempotente (um clamp, um `normalize`, um
`fract`, um `dedup`) no fim de uma cadeia torna invisível toda reordenação a montante dele — e a
mutação parece certa a quem a lê, porque o código mutado *é* diferente. **How to apply:** quando o
passo final for idempotente, o `old` da mutação tem de **conter esse passo** para que o `new` o
possa tirar de lá; e a suspeita levanta-se sozinha ao ver a lista da cadeia acabar num redutor. Ver
[[reference_topic_measurement_discipline]].
## ⛔⛔⛔ Um arnês que faz backup por EDIÇÃO e não por FICHEIRO **corrompe a árvore** (2026-09-21)
A `line/3DModeling` estendeu o arnês de mutação para aceitar **várias edições numa mutação só** (a
que representa *«a lei mudou-se para outro ramo»* precisa de duas). Com as duas edições no **mesmo
ficheiro**, a sequência foi: copiar `f → f.bak`, escrever a 1.ª · **copiar `f → f.bak` outra vez**,
escrever a 2.ª · restaurar `f.bak → f` (⇒ a árvore ficou com a **1.ª mutação aplicada**) · restaurar
outra vez (⇒ `FileNotFoundError`). O shader ficou no repositório com `materia = CLAY;` e o gate da
wave verde ao lado dele. **Why:** o modo de falha é o pior de todos — *ele não devolve o original, ele
devolve a versão JÁ MUTADA*, e a segunda restauração estoura DEPOIS do estrago, logo o `traceback`
aponta para o sintoma e não para a causa. *Um arnês que corrompe a árvore é pior que um que mente:
ele deixa o defeito lá.* **How to apply:** o backup é indexado pelo **caminho** (`dict`), nunca pela
lista de edições; e depois de qualquer corrida de mutação com edições múltiplas, **conte no
ficheiro** o que devia lá estar (`grep -c`) antes de acreditar na tabela. Ver
[[reference_topic_measurement_discipline]].

## ⛔⛔ Uma mutação que deixa a chamada VIVA lê-se exactamente como «sobreviveu» (2026-09-21)
Na mesma corrida, a mutação *«a shell deixa de sincronizar»* comentava só a **primeira linha** da
chamada e reescrevia-a logo abaixo — o produto ficou **idêntico**, e o gate de fiação (que lê a
fonte com os comentários já retirados) devolveu `SOBREVIVEU (0/1)`. Eu quase escrevi *«o gate de
fiação não mede nada»* sobre um gate correcto. **Why:** num gate de TEXTO a mutação tem de apagar o
texto, e comentar uma linha de uma chamada multi-linha **não apaga a chamada**. **How to apply:**
para mutar uma chamada de N linhas, a agulha é o **BLOCO INTEIRO** lido do ficheiro em runtime
(`src[i..j]`), nunca uma linha escrita à mão. Ver [[reference_topic_gate_discipline]].

- ⛔⛔ **Uma mutação cuja consequência É o DISPOSITIVO não tem valor de retorno que a denuncie — o
  gate dela é um CENSO DE CHAMADORES.** Medido 2026-09-21: repor um `clear_albedo_source()` num
  braço `Err` (a **porta das traseiras**) sobreviveu a toda régua escrita de dentro da função, que
  devolve `()` e cujo efeito só se vê na textura ligada à placa. ⇒ o censo afirma **duas** metades —
  *uma só* linha de produto chama a porta de reposição, **e** ela é a do braço que a lei declara.
  Com o censo, a mutação sangra com o endereço. *Quando o efeito não é observável do lado de cá,
  gateie a POPULAÇÃO de quem o pode causar.*
- ⛔⛔ **UMA MUTAÇÃO QUE *AUMENTA* A GRANDEZA SOB TESTE NÃO ALCANÇA A PROPRIEDADE — e apertar a
régua para a matar mede OUTRA COISA.** Medido 2026-09-21: para provar que o controlo de vácuo de um
gate dispara, pus a fixtura toda **PRETA** no `base`; ela **sobreviveu** duas redacções seguidas do
controlo. Medida, a excursão da fixtura preta é **`246`** contra **`188`** da boa — *mais*
contrastada, porque um `base_color` preto tira a **COR** e não a **FORMA** (o destaque especular não
sai do albedo). **Why:** uma mutação só afirma alguma coisa se empurrar a grandeza para o lado do
vácuo; empurrando-a para o lado bom, «sobreviveu» é o veredito **correcto** e persegui-la leva a
apertar uma barra até ela medir o ruído de outra propriedade. **How to apply:** antes de curar uma
mutação sobrevivente do ARNÊS, **meça a grandeza nos dois lados**; se a mutação a faz subir,
NOMEIE-a no gate com o número em vez de a matar, e escreva uma irmã que a faça descer (ali: a
cobertura a `0` e a fixtura sem relevo nenhum, as duas a lerem `0` e a sangrar).
