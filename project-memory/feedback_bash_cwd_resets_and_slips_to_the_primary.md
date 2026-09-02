---
name: feedback-bash-cwd-resets-and-slips-to-the-primary
description: "Modo L: a cwd do Bash volta ao repo primário entre turnos, e um `cd primário && ...` a move para o resto da sessão — prefixe TODO comando com o cd da worktree"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4d33a1a5-4cde-45a3-94d1-ae7125cd56bf
  modified: 2026-08-05T23:09:33.805Z
---

No Modo L, a cwd da ferramenta Bash **não é estável**: ela volta ao **repo primário** (`~/Documentos/Projetos/PH2D`, que está em `main`) entre turnos do usuário, e qualquer comando composto que termine com `cd <primário> && ...` (ex.: conferir `git status` do primário) a **deixa lá para todos os comandos seguintes daquele turno**.

**Why:** o mesmo path relativo existe nas DUAS árvores, então o comando errado **não falha** — ele lê/edita a árvore errada em silêncio. É a armadilha nº 1 do Modo L ([[project_multiagent_modo_l_2026_07_05]]), e ela morde a LLM, não só o Enio (o [[feedback_run_command_include_cd]] cobre o outro lado: comandos entregues ao Enio).

Como ela se revela (se revelar): `cargo test --test <alvo>` responde **"no test target named ..."** para um arquivo que você acabou de criar — porque o alvo existe na worktree e você está no primário.

**How to apply:** prefixe **todo** comando com `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-<módulo> && ...`, e nunca termine um comando composto com um `cd` para outro lugar — rode o que for do primário com `git -C <primário>` em vez de `cd`. Um `pwd` no começo de qualquer bloco que vá EDITAR é barato e é a única confirmação que vale.

⚠️ **Refinamento (2026-07-29, 2ª e 3ª escorregadas na mesma sessão): todo script de edição in-place usa path ABSOLUTO.** O `cd` protege o cargo, mas um heredoc `python3`/`sed -i` com path relativo resolve contra a cwd escorregada e **edita a árvore errada sem erro** — foi assim que duas linhas de `mod` foram parar no `main`. Com path absoluto a disciplina do `cd` deixa de ser load-bearing para a CORREÇÃO (só para a velocidade do build). E a reversão de um acidente desses é **remoção cirúrgica** das linhas inseridas (python com path absoluto), **nunca `git checkout`** ([[feedback_mutation_undo_with_cp_never_git_checkout]]) — a árvore primária costuma ter trabalho alheio não-commitado (a `project-memory/`, por exemplo).

O sintoma que denuncia: `cargo` reclama de `failed to create directory .../PH2D/target` — ele está tentando construir no primário.

⚠️ **Refinamento (2026-08-07): pior que a busca vazia é a busca que RESPONDE — com o valor do `main`.** Numa mesma sessão o `grep` de um `const` leu a árvore primária e devolveu `spring_damping: 0.5` onde a linha diz `1.0`; ao lado de um doc-comment (da linha, lido pela ferramenta Read, que usa path absoluto e portanto acerta) que dizia *"1,00 é o que shipa"*, isso montou uma **contradição inteiramente fabricada** — e eu a anunciei ao Enio como *"o doc está mentindo"* antes de a sonda imprimir o número real. **Read/Write usam path absoluto e sempre acertam; `grep`/`sed`/`cargo` num Bash escorregado sempre erram** ⇒ misturar as duas fontes num mesmo raciocínio produz um conflito que não existe em árvore nenhuma. Quando um número lido por grep contradiz um doc lido pelo Read, **a primeira hipótese é a CWD**, não o doc; e o desempate é `grep` com path absoluto nas DUAS árvores, lado a lado.

⚠️ **Refinamento (2026-08-05): o tell mais perigoso é uma BUSCA VAZIA, não um erro.** Os dois sintomas acima *falham alto*. O que morde de verdade é abrir a sessão seguinte (a cwd já voltou ao primário) e ler: `sed: arquivo inexistente` para um arquivo que o handoff diz existir, e depois `grep` de um símbolo da sua própria linha voltando **vazio**. Isso não se lê como *"árvore errada"* — lê-se como *"a feature não está aqui"*, e a resposta natural é contornar. **Um arquivo que o seu handoff afirma existir e não existe é sinal de CWD, nunca fato sobre o código** — é o [[feedback_a_negative_search_needs_a_positive_control]] aplicado à árvore: o controle positivo é `pwd && git branch --show-current`, e ele custa nada. Faça-o **no primeiro comando de todo turno**, não só antes de editar.

⚠️ **Refinamento (2026-08-23, 4ª escorregada registada): o `cat >> <path relativo>` é o irmão do `sed -i` e ele APENDE na árvore errada.** A sessão corria há dezenas de comandos com a cwd certa; a slip aconteceu no meio, e um `cat >> crates/.../lib_tests.rs <<'EOF'` escreveu 48 linhas de um gate novo no **`main`**. ⭐ **O tell foi o compilador**, e ele foi honesto: `no function or associated item named 'from_directions' found` — para uma função que a ferramenta `Edit` (path absoluto ⇒ árvore certa) tinha acabado de criar. *Um símbolo que você acabou de escrever e que o compilador não vê é sinal de CWD, exactamente como a busca vazia.*
⭐ **A reversão foi limpa e vale como receita:** `git -C <primário> diff --stat <ficheiro>` confirmou que o único delta era o meu (48 inserções, 1 linha de contexto removida), `tail -48` extraiu o bloco para `/tmp`, `git checkout -- <ficheiro>` no primário (⚠️ só porque o diff provou que **nada mais alheio** estava naquele ficheiro — a regra do [[feedback_mutation_undo_with_cp_never_git_checkout]] continua a valer para ficheiros com trabalho alheio), e o bloco foi apendido na worktree. **Confira o `git status` do primário no fim de todo turno que tenha usado path relativo em escrita.**

⚠️ **Refinamento (2026-08-23): a escorregada é barata de reverter; o ALARME FALSO que ela
provoca é que custa.** Depois de um commit de docs cair no `main` (revertido por
`reset --soft` + `restore --staged` + `checkout --`, **nunca `--hard`** — a primária tinha
`project-memory/` alheia não-commitada), fui verificar se o gate de fecho tinha medido a
árvore certa: procurei os testes NOVOS na saída da corrida, achei **zero**, e concluí que
17.923 testes tinham corrido contra o `main`. **Errado** — o comando terminava em
`| tail -20`, então o arquivo tinha vinte linhas e a busca era vazia **por construção**.

**Why:** a suspeita de CWD é a hipótese CERTA e por isso ela chega com força total — e nesse
estado uma busca vazia lê-se como confirmação em vez de como o que é. *O [[feedback_a_negative_search_needs_a_positive_control]] vale contra um LOG truncado exactamente como vale contra uma árvore errada*, e o pipe que trunca é meu.

**How to apply:**
1. ⚠️ **Todo comando longo lançado em background começa por `cd <worktree> && pwd && git branch --show-current &&`** — o gate passa a AUTO-VERIFICAR a árvore que mediu, e nenhuma arqueologia posterior é precisa. Custa três palavras.
2. ⛔ **Nunca conclua «rodou na árvore errada» a partir de um `grep` sobre saída PIPADA.** Ou se guarda a saída inteira, ou a evidência é outra: aqui, duas corridas darem o **mesmo** `17.923 / 1 ✗` já provava que mediram a mesma árvore (se uma fosse o `main`, faltariam os ~30 testes da linha), e um `cargo nextest list` resolveu em um comando.
3. A reversão na primária é **cirúrgica e nesta ordem**: `reset --soft HEAD~1` → `restore --staged <arquivo>` → `checkout -- <arquivo>`. O `--hard` apagaria o trabalho alheio que a primária quase sempre tem.

⚠️ **Refinamento (2026-08-25, 5.ª e 6.ª escorregadas — as duas com `python3` de path RELATIVO, a regra que o refinamento de 2026-07-29 já escrevia):** o tell de hoje **não** foi *"failed to create directory"* e sim **`File exists (os error 17)`** sobre o `target/debug` do primário — porque **outra worktree estava a construir no mesmo `target` naquele instante**. *A mesma causa dá mensagens diferentes conforme o que a outra linha está a fazer;* o que não muda é o **caminho** na mensagem: se ele diz `/PH2D/target` e não `/PH2D/Worktrees/…/target`, é CWD, seja qual for o erro. ⭐ E a reversão foi limpa nas duas vezes pela mesma receita: `git status --porcelain` no primário (só os MEUS ficheiros), `git diff --stat` + `git diff | grep '^+'` (**zero adições alheias**, e uma delas era pura remoção — 58 linhas, 0 inserções), então `git checkout --`. ⛔ Continua a valer que isto só é seguro **depois** de o diff provar que nada alheio está naquele ficheiro.
---

## ⭐⭐⭐ Adenda 2026-08-26 — a escorregadela é SILENCIOSA para EDIÇÕES, e só o `cargo` a denuncia

A meio de um bloco, três ficheiros de uma linha em Modo L foram editados **na árvore
primária**. O que os escreveu foi um `python3` com caminhos RELATIVOS: ele resolveu-os a
partir da cwd escorregada, **escreveu com êxito** e imprimiu «3 edições aplicadas».

⛔ **Nada no caminho da edição pode falhar.** O ficheiro existe nas duas árvores, o
conteúdo casa nas duas, e o `assert` de contagem — que é a rede desta casa contra o
`replace` que não casa — **passa**, porque ele mede o texto, não o sítio.

⭐ **Quem denunciou foi o `cargo`**, e por acidente: `failed to create directory
/…/PH2D/target/debug`. Isto é sorte de configuração, não uma rede: num dia em que aquele
`target/` exista e seja gravável, a corrida compila a árvore ERRADA e passa.

**How to apply:**
1. ⭐⭐ **Todo comando que ESCREVE começa com o `cd` da worktree** — não só os que correm
   cargo. A regra escrita cobria «comandos»; o que morde é a EDIÇÃO, porque ela é a única
   que não tem sintoma.
2. ⭐ **A verificação é `git status --porcelain` nas DUAS árvores**, e ela custa uma
   chamada: a primária tem de mostrar só o que já lá estava. Depois de qualquer bloco de
   edição por script, vale o preço.
3. ⚠️ **A recuperação tem uma ordem:** copie primeiro os ficheiros da árvore errada para a
   certa, **confirme que o diff na errada é só seu** (`git diff` + procurar as suas marcas),
   e só então `git checkout -- <os caminhos exactos>`. Nunca um `checkout` de árvore
   inteira: a primária tinha ficheiros modificados de OUTRA sessão, e um `--` sem caminhos
   tê-los-ia levado.
4. ⛔ **Não confie no `pwd` de uma chamada anterior.** A cwd persiste entre chamadas *até
   deixar de persistir*, e a escorregadela não avisa.
5. ⭐⭐⭐ **Todo script que EDITA começa com um `assert` da própria árvore.** É a única rede que
   não depende de eu me lembrar:
   ```python
   assert os.getcwd().endswith("Worktrees/line-<módulo>"), os.getcwd()
   ```
   e, quando o ficheiro tem uma marca da linha, um segundo `assert` sobre o CONTEÚDO — foi
   ele que apanhou a segunda escorregadela.

## ⛔⛔ E a receita de recuperação acima está INCOMPLETA — ela destruiu trabalho

Na segunda ocorrência (mesma jornada, 2026-08-26) eu segui os meus próprios passos e
**revertí um commit da minha linha**. O passo «copie da árvore errada para a certa» só é
seguro quando o ficheiro é **idêntico** nas duas — e o primário está em `main`, que **não
tem** os commits da linha. O ficheiro que copiei era `main` + a minha edição nova, e ao
pousá-lo na worktree apaguei **36 linhas** de uma célula que a mesma linha tinha fechado
dias antes. Os testes disseram-no na hora (`o picker oferece o canal Velocity X`), mas só
porque havia gate; sem ele, a regressão viajava no commit.

⭐ **A receita CERTA, e a ordem importa:**
1. `git -C <primário> diff --stat -- <caminhos>` — confirme que o diff lá é só o seu.
2. **NÃO copie.** Guarde o diff (`git -C <primário> diff -- <caminhos> > /tmp/x.patch`) ou,
   melhor, **descarte-o e reaplique a edição na árvore certa** — o script que a fez ainda
   está no seu contexto e correr outra vez custa nada.
3. `git -C <primário> checkout -- <os caminhos EXACTOS>`.
4. Na worktree, `git checkout -- <caminho>` **antes** de reaplicar, se já lá tiver posto a
   cópia envenenada.

⚠️ *A pergunta que resolve isto numa linha: «este ficheiro é o mesmo nas duas árvores?» Se a
linha alguma vez lhe tocou, a resposta é não, e copiar é um `revert` silencioso.*


## ⭐⭐⭐ Adenda 2026-08-27 (7.ª) — a guarda que FUNCIONA é um assert sobre o CONTEÚDO da sua linha

Mesma escorregadela, mesmo tell (`File exists (os error 17)` sobre `/PH2D/target/debug`), mesma
receita de recuperação — que desta vez correu limpa: `git status --porcelain` no primário (só os
meus dois ficheiros), `git diff` para provar que o conteúdo era meu, `git checkout -- <os dois
caminhos exactos>`, e **reaplicar na worktree**, nunca copiar ([[feedback_the_memory_symlink_points_at_the_primary_tree_not_your_worktree]]).

⭐ **O que mudou o resultado foi a guarda que passei a pôr no topo de todo script de edição:**

```python
W='/home/enio/.../Worktrees/line-<módulo>'
src=open(W+'/caminho/do/ficheiro.rs').read()
assert 'refinement: bool' in src, 'ARVORE ERRADA: falta a W89'
```

**Why:** o `assert os.getcwd().endswith(...)` desta memória protege contra a cwd, e o path absoluto
protege contra o `cd` — mas **nenhum dos dois protege contra o path absoluto ERRADO** (a árvore
primária escrita à mão num script). Um assert sobre uma marca que **só o commit da sua linha tem**
prova a árvore pelo que ela É, não por onde eu acho que ela está — e é a única guarda que não
depende de eu me lembrar de qual das duas raízes é a minha.

**How to apply:** escolha a marca no ficheiro que vai editar (um símbolo que a sua linha criou), e
ponha o assert **antes** de qualquer `replace`. Ele custa uma linha e falha alto.

⛔⛔⛔ **Refinamento (2026-08-30, 5ª escorregada registada) — e a lição JÁ ESTAVA ESCRITA aqui
desde 29/07.** Um `python3` heredoc com caminhos **relativos** reescreveu **quatro ficheiros de
produto** no `main`. ⭐ **Só foi apanhado por sorte:** um deles era um `Cargo.toml` que passou a
referir uma crate inexistente no primário, e o `cargo` recusou a workspace **com o caminho do
primário no erro**. ⚠️ **Fossem só `.rs`, teria compilado** — e o `main` ficava com quatro
alterações que ninguém pediu.

⇒ ⭐ **A regra do path absoluto não basta se ela não estiver no script.** O que falta é uma
**asserção no topo de todo heredoc de escrita**: `W = Path("<worktree absoluta>"); assert (W /
".git").exists()`, e **todo** `read_text`/`write_text` a partir de `W`. *Um `cd` no início do
comando não protege um script que corre depois de outro comando ter mudado a cwd* — e nesta
sessão o `cd` estava lá, no comando anterior.


⚠️⚠️ **Refinamento (2026-08-30, ~10ª escorregada numa sessão só): o tell MAIS RÁPIDO de todos é `git log` a mostrar commits que você não reconhece.** Numa auditoria da própria linha, um `grep` numa árvore escorregada devolveu **10 leituras directas ao `WidgetStore`** num ficheiro cuja cura (a W9 daquela manhã) as tinha apagado; o gate que as proíbe respondeu *"no test target"*, e o `ls` do ficheiro dele deu **inexistente**. Três sinais a apontar todos para a mesma conclusão errada — *"o gate que eu shipei está vazio e a cura foi revertida"* — e cheguei a correr `git show --stat` do commit para caçar quem a tinha apagado.
⭐ **O que fechou o caso em UM comando foi `git log --oneline -3`:** ele imprimiu commits de `docs(registo)` e `fix(ci)` que não eram meus — os do `main`. *Uma busca vazia mente, um `ls` inexistente mente, um `no test target` mente; o topo do log não.*
⇒ Ao investigar um defeito **na sua própria linha**, o primeiro comando não é o `grep` — é `pwd && git branch --show-current && git log --oneline -1`. E há um invariante que resolve sozinho: **se o defeito é «a minha cura de hoje desapareceu», a hipótese nº 1 é a CWD, nunca a reversão** — trabalho commitado não evapora, e a árvore primária *nunca* teve a cura.


## ⛔⛔⛔ Adenda 2026-08-31 (8.ª) — a escorregadela numa corrida de LEITURA devolve **VERDE**

Todos os tells desta memória **falham alto**: erro de build, busca vazia, símbolo que o compilador
não vê, `File exists` sobre o `target/`. Eles existem porque a árvore errada não tem o meu trabalho.

⛔ **Um `cargo test` de um gate que reprovou na MINHA árvore não falha nada no primário: ele passa.**
Foi o que aconteceu — dois gates vermelhos na worktree, e ao re-corrê-los para ler a mensagem a cwd
já tinha escorregado. Os dois disseram `test result: ok`, e a leitura natural é *«afinal era ruído do
fan-out»* ⇒ eu quase os riscei da lista. O que os denunciou foi **uma linha de `Compiling`**:

```
Compiling ph2d-editor-core v0.0.0 (/home/enio/Documentos/Projetos/PH2D/crates/ph2d-editor-core)
```

— sem `Worktrees/line-<módulo>` no caminho.

**Why:** o repo compila nas duas árvores. Uma corrida de leitura no primário mede **`main`**, e
`main` está verde por construção. *Um verde da árvore errada é indistinguível de um verde meu*, e é
o único sintoma desta família que não tem tell próprio.

**How to apply:**
1. ⭐⭐ **Um verde inesperado é a MESMA suspeita que uma busca vazia.** Se um gate reprovou e passa a
   passar sem eu ter mudado a causa, a 1.ª hipótese é a cwd — não a flake de recurso.
2. ⭐ **Leia a linha `Compiling`/`Running`** do cargo: ela imprime o caminho da crate e é a
   confirmação mais barata que existe (não custa uma chamada extra, já está na saída).
3. ⚠️ Um `--test <alvo>` que casa em ambas as árvores **não** dá o tell «no test target named …»
   desta memória — esse só aparece para um alvo que só existe na worktree.


## ⛔⛔⛔ Adenda 2026-08-31 (9.ª, **minutos depois da 8.ª**) — o tell foi o CONTEÚDO não bater

Na mesma sessão em que escrevi a adenda acima, escorreguei outra vez e um `cat >> <path relativo>`
apendou **124 linhas de gate** no ficheiro do **primário**. O `cargo test` a seguir reprovou com
`this function takes 8 arguments but 9 were supplied` — sobre uma função cuja assinatura eu tinha
lido (na worktree) com 9.

⭐ **O que denunciou não foi o caminho: foi o TEXTO.** Ao imprimir a assinatura, o ficheiro dizia
`"Made a component"` e `"This is already a component"`, e eu sabia que a minha linha diz
`"Made a prefab"`. *Um ficheiro que não diz o que eu sei que ele diz é a árvore errada, antes de ser
um bug.*

⚠️ **E o erro do compilador era um FALSO diagnóstico perfeitamente plausível** — «assinatura
diferente» lê-se como *«o meu teste está errado»*, e a reacção natural é ir mudar o teste. Duas
árvores com a mesma função em versões diferentes produzem erros de tipos que **fazem sentido** e
apontam para o sítio errado.

**How to apply (o que muda depois da 9.ª):**
1. ⛔⛔ **`cat >`/`cat >>` são ferramentas de EDIÇÃO** e obedecem à mesma regra dos scripts: caminho
   ABSOLUTO + `assert` de uma marca da minha linha. A adenda da 4.ª já o dizia; eu fi-lo à mesma
   porque estava a encadear comandos depressa. *A regra não falhou — eu é que a apliquei só aos
   `python3`.*
2. ⭐ **Ao ler fonte para decidir alguma coisa, confirme uma FRASE que só a sua linha tem.** É o
   mesmo assert do ponto 1, feito com os olhos, e custa zero.
3. ⚠️ Um erro de tipos que contradiz o que você acabou de ler é sinal de CWD — **antes** de ser um
   sinal sobre o código.

⚠️ **2026-08-31 — a 3.ª forma, e a mais silenciosa: `cat >> ficheiro` com caminho
RELATIVO.** Um `cargo check` acusou por acaso (os caminhos que ele imprime eram
`/PH2D/crates/...` em vez de `/PH2D/Worktrees/line-.../crates/...`), e o `cat >>`
**CRIOU** o ficheiro na árvore errada em vez de falhar — 64 linhas de doc num ficheiro
novo e não rastreado do primário, invisíveis ao `git status` da worktree. *Um `>>` nunca
avisa: se o caminho não existe, ele nasce.*
⇒ **Todo comando que ESCREVE (`cat >>`, `tee`, redirecção) leva o caminho ABSOLUTO da
worktree**, mesmo quando o anterior correu bem: a cwd volta ao primário entre chamadas.


## ⛔⛔⛔ Adenda 2026-08-31 (10.ª) — o `cd` de um comando em SEGUNDO PLANO não persiste, e o harness diz-o

O mecanismo desta vez não foi a escorregadela lenta das outras nove: foi **causal e imediato**. Um
comando lançado com `run_in_background: true` que começa por `cd <worktree>` devolve, no próprio
resultado:

> *Session cwd remains /home/enio/Documentos/Projetos/PH2D; directory changes made by the
> backgrounded command do not apply to subsequent commands.*

⇒ o comando de fundo corre certo, **e o turno seguinte corre no primário**. Eu li aquela linha, não
a processei, e corri o portão de fecho — `cargo test` + `cargo clippy` sobre cinco crates — na
árvore errada. **Deu verde**, que é o sintoma da 8.ª adenda: um verde da árvore errada é
indistinguível do meu.

⭐ **O que denunciou foi um GREP negativo com controlo:** o smoke deixou de imprimir, e
`grep -c "level == 80" <router>` devolveu `0` para uma cena que eu tinha **comitado**. Um símbolo
que está num commit meu e que o `grep` não vê é a árvore errada — nunca um facto sobre o código
([[feedback_a_negative_search_needs_a_positive_control]]).

**How to apply:**
1. ⭐⭐⭐ **Todo comando de fundo que precisa da worktree leva o `cd` DENTRO dele** — e o **seguinte**
   também, porque a sessão não herda nada. Não é «lembrar do `cd` uma vez por bloco»: é **por
   chamada**.
2. ⭐ **A linha «Session cwd remains …» é um AVISO, não ruído.** Ela aparece exactamente quando a
   próxima chamada vai correr no sítio errado.
3. ⚠️ Os scripts de edição com caminho ABSOLUTO sobreviveram intactos (as 8 curas da auditoria
   aterraram todas na worktree) — foi só a **verificação** que se perdeu. *A disciplina do caminho
   absoluto protege a escrita; nada protege a leitura senão o `cd` por chamada.*

---

## ⛔⛔⛔ Adenda 2026-08-31 (11.ª) — o tell foi um `ls` que fez o BINÁRIO ANDAR PARA TRÁS no tempo

Uma build de release lançada em **background com `cd` explícito para a worktree** correu certa (o
log dela nomeia `/…/Worktrees/line-UIUX/shells/desktop`) e saiu `exit 0`. A cwd escorregou
**depois**, entre turnos. Então:

```
ls -la target/release/ph2d-host-desktop      # 20:11, 64 707 088 bytes   ← a WORKTREE
… (a cwd volta ao primário) …
ls -la target/release/ph2d-host-desktop      # 30/08 15:33, 64 600 784   ← o PRIMÁRIO
```

⛔ **O mesmo comando, duas respostas, e a segunda é ANTERIOR à primeira.** Não é um erro nem uma
busca vazia: é um facto plausível sobre um ficheiro que existe. A leitura natural — *«a build disse
`Finished` e o binário não mudou; o cargo não relinkou»* — é **uma teoria sobre o cargo**, e eu
comecei a persegui-la: fui ver `target/release/deps/` e depois corri um `cargo build --release` de
**3 min 08 s no PRIMÁRIO**, que é a escorregadela a pagar-se a si própria.

⭐ **O que resolveu em UM comando foi o `ls` com path ABSOLUTO:** a worktree tinha o binário de
`20:56`, exactamente a hora em que o background terminou. Nada estava partido.

> ⚠️ *Um relógio que anda para trás entre dois comandos idênticos é sinal de CWD, nunca um facto
> sobre a ferramenta que os produziu.* Junta-se à família: a busca vazia · a busca que responde com
> o valor do `main` · o símbolo que o compilador não vê · o `File exists` do `target`.

**How to apply:** ⚠️ **toda inspecção de artefacto de build (`ls`/`stat`/`file` sobre `target/`) usa
path ABSOLUTO** — é uma leitura, logo não falha alto, e o `target/` é o ponto exacto onde as duas
árvores têm o mesmo nome relativo com conteúdos diferentes. E antes de teorizar sobre o cargo,
`pwd`.

## 12.ª e 13.ª (2026-09-01) — e a 13.ª tem um modo de falha NOVO

⛔⛔ **Um filtro de saída transforma «árvore errada» em VERDE VAZIO.** A corrida foi
`cargo test -p ... --test <nome> | grep -E "^test |test result|..."`. Na árvore primária aqueles
alvos não existem, o cargo devolve `error: no test target named ...` — e **nenhuma das linhas do
erro casa com o filtro**. O ficheiro de saída fica com zero bytes e o pipe devolve `0`.

⇒ *Um `grep` que só deixa passar sucesso não distingue «passou» de «nunca correu»*
([[feedback_pipe_masks_script_exit_code]], [[feedback_an_automatic_tools_exit_code_says_nothing_about_what_it_produced]]).

**How to apply:**
- ⭐ **Ponha o `cd` absoluto DENTRO de cada comando**, mesmo quando o anterior já o tinha: o harness
  avisa (`Session cwd remains ...`) só quando o comando vai para segundo plano.
- ⭐⭐ **Todo filtro de saída tem de deixar passar `error`** (`grep -E "...|^error|no test target"`),
  senão ele esconde exactamente a linha que diz que a corrida não aconteceu.
- ⚠️ E a prova de que uma corrida foi na árvore CERTA é o nome de um teste que **só lá existe**
  aparecer na saída.

---

⭐⭐ **12.ª (2026-09-01) — e o gatilho é ESTRUTURAL, não distracção: ESCREVER A MEMÓRIA obriga
a ir ao primário.** O símlink `~/.claude/projects/<key>/memory` aponta para o
`project-memory/` da árvore **primária**, e o índice `MEMORY.md` só lá existe ⇒ indexar uma
memória nova pede `cd /…/PH2D`. O comando **seguinte** — um `python3` que editava um gate —
herdou essa cwd e escreveu **no primário**, sobre um ficheiro que a minha worktree também tem
no mesmo caminho relativo. Não houve erro: o `python3` achou o padrão, o `assert` de contagem
passou, e só o aviso do harness (*«changed on disk»*, a apontar `/PH2D/shells/...`) o revelou.

⇒ **Depois de todo `cd` ao primário — e escrever memória é sempre um —, o comando seguinte
recomeça com o `cd` absoluto da worktree.** E a cura, quando acontece, é
`git checkout -- <o ficheiro>` no primário (⛔ **nunca** `git reset --hard`), seguida do mesmo
`python3` com o `cd` certo. Ver [[feedback_the_memory_symlink_points_at_the_primary_tree_not_your_worktree]].

## ⚠️ 2026-09-03 — a 14.ª, e o que a torna INVISÍVEL

O `cd /…/PH2D && python3 …` que escreve uma memória **persiste**, e as chamadas seguintes correm no
primário. Isso passou despercebido durante quatro comandos porque **os ficheiros existem nas DUAS
árvores com o mesmo conteúdo** — ler `toggle.rs` no primário deu exactamente o mesmo que na
worktree, e a leitura foi usada para planear.

⛔ **O que denuncia a fuga é ler um ficheiro que ACABASTE de editar** — ele volta ao HEAD, e a
conclusão natural (*"alguma coisa reverteu o meu trabalho"*) é falsa e cara: quase gastei uma
sessão a caçar um `git checkout` que nunca existiu.

⇒ **Nunca `cd` isolado.** Toda chamada começa com `cd <caminho ABSOLUTO da minha árvore> &&`, ou usa
caminhos absolutos em `python3`/`grep`. E antes de acreditar que uma edição desapareceu:
`pwd && git branch --show-current`.

## ⛔⛔ 2026-09-03, a 15.ª — e desta vez EDITEI O `main`

A cwd voltou ao primário **sozinha**, sem nenhum `cd` meu no meio: entre duas chamadas, um
`cd <worktree> && …` deixou de valer. Todas as edições seguintes usavam **caminhos relativos**
(`python3` com `crates/…`), e foram parar ao **`main`** — oito ficheiros, um bloco inteiro de
feature. O `cargo check` que corri a seguir **passou**, porque a árvore errada também compila.

⭐ **O que denunciou foi um `cargo test` a dizer «no test target»:** a `Write` tinha usado caminho
ABSOLUTO e pôs o ficheiro de teste na worktree, enquanto o `cargo` corria no primário. *A
inconsistência entre uma ferramenta que usa caminho absoluto e um script que usa relativo é o
sintoma mais barato desta falha — e só aparece por acaso.*

**A cura, e é mecânica:**
- ⛔ **Nenhum comando com caminho relativo.** Ou `cd <ABS> && …` **na mesma chamada**, ou caminhos
  absolutos dentro do `python3`/`grep`. Não confie em que o `cd` da chamada anterior sobreviveu.
- ⭐ **Recuperar é barato se se apanhar cedo:** `git diff -- crates > /tmp/x.patch` no primário,
  `git checkout -- crates`, e `git apply -3` na worktree (o Mergiraf resolve o resíduo).
- ⚠️ **Mas o patch pode aterrar no sítio errado sem conflito:** a worktree tinha o ficheiro de ids
  **partido em dois** e o símbolo novo caiu na metade errada — compilava, e ficava a três écrãs dos
  irmãos. *Depois de um `apply -3`, confira ONDE cada símbolo novo aterrou, não só que compila.*

## ⛔⛔⛔ 2026-09-03, a 16.ª — e a RECUPERAÇÃO fez mais estrago que a escorregadela

Mesma causa (um `cd` ao primário para indexar memória, cinco `python3` seguintes com caminho
relativo). O que é novo é o que veio a seguir: eu **copiei os ficheiros do primário para a
worktree** — e a worktree estava **49 commits à frente**. Cada `cp` pousou a versão do `main` por
cima de trabalho da linha, e o `cargo` acusou com erros que não faziam sentido nenhum
(`cannot find TETO_DIGITAVEL`, `MAX_CHOICES`, `model3d_choice_button` — símbolos que a linha tinha
criado e que o `main` não tem).

⛔ **A adenda de 2026-08-26 desta mesma memória já proíbe isto, por extenso:** *«NÃO copie. Guarde o
diff ou, melhor, descarte-o e reaplique a edição na árvore certa — o script que a fez ainda está no
seu contexto e correr outra vez custa nada.»* Eu li-a, escrevi-a, e fiz o contrário sete dias depois.

⭐ **Recuperou-se sem perda porque os 6 ficheiros estavam LIMPOS na worktree** (`git status` não os
mostrava): `git checkout -- <os 6 caminhos>` devolveu-os ao HEAD da linha, e reaplicar os cinco
scripts com caminho absoluto custou uma chamada. *Se algum deles tivesse edição não-commitada da
linha, ela tinha morrido sem deixar rasto.*

**How to apply — a ordem que evita as duas metades do estrago:**
1. ⭐⭐⭐ **A pergunta ANTES de qualquer `cp` entre árvores: «a minha linha alguma vez tocou neste
   ficheiro?»** Se sim — e com 49 commits a resposta é quase sempre sim —, copiar é um `revert`
   silencioso. **Reaplique, nunca copie.**
2. ⭐ `git status --short` na worktree **antes** de restaurar: só um ficheiro limpo pode levar
   `git checkout --` sem perder nada.
3. ⚠️ E o tell desta variante é **um erro de símbolo em cascata**: vários símbolos que a linha criou
   a desaparecerem de uma vez é *o `main` a chegar por cima*, nunca um refactor mal feito.

## 12.ª e 13.ª da `line/components` (2026-09-01) — a escorregadela apanhou um comando que ESCREVE
As dez primeiras foram leituras — e a lição delas era *«uma leitura na árvore errada devolve
VERDE»*. Estas duas foram piores em espécie:
- **12.ª:** `cargo test` de dois censos na primária devolveu **verde** sobre uma feature que só
  existe na worktree. Eu li e reportei o verde. As falhas do portão eram reais.
- **13.ª:** um `cargo fmt --all` + `git add` + `git commit` correram na primária. O `fmt` **escreve**
  — só não deixou estrago porque a primária já estava formatada, e o commit não fez nada porque não
  havia o que comitar. *A ausência de dano foi sorte, não desenho.*
⇒ **A regra endurece: TODO comando começa com `cd <worktree> && pwd &&`** — não só os de escrita. E
o `pwd` na primeira linha é o instrumento: sem ele, um verde e um verde-na-árvore-errada são o mesmo
byte. ⚠️ Confirmar que a primária ficou intacta (`git -C <primária> status --short -- crates shells`)
faz parte de reparar a escorregadela, e não é opcional.
