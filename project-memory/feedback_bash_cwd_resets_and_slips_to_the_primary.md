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
