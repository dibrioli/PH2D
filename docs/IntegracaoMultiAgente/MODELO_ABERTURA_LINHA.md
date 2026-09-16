# MODELO — Abertura de Linha Paralela (Modo L)

> **Fonte única** do bloco que o Enio cola na 1ª mensagem de cada sessão-de-linha
> (DIRETRIZ §1.5.8). O agente cria a própria worktree e prepara tudo; a tarefa vem depois.

## Como usar (Enio — 4 passos, sempre iguais)

1. Abra uma **janela nova** do VSCode/Claude **na pasta do repo** (`~/Documentos/Projetos/PH2D`
   — sempre a mesma; uma janela por agente).
2. Copie o bloco abaixo inteiro e escreva o nome do módulo **UMA vez só**, na 1ª linha
   (`Sua linha: line/…` — 1 palavra, kebab-case curto: ex. `grayscale`, `painter`,
   `vector`, `foundational`). O resto do bloco se refere a ele como **"o novo módulo"**
   (`$MODULO` nos comandos) — não precisa trocar mais nada.
3. Cole como **1ª mensagem** da sessão. O agente faz o setup sozinho e responde
   **"Linha pronta. Aguardo a tarefa."**
4. Mande a tarefa na mensagem seguinte (o que construir + em qual pasta `crates/...`).
   Docs/tracker do módulo nascem depois, dentro da própria worktree.

**Nunca** abra duas linhas pro mesmo módulo. Pra fechar uma linha que terminou de vez:
peça ao agente "encerre a linha" — o procedimento, com o que tem de ser guardado ANTES, está em
§"Encerrar uma linha" (o `git worktree remove` apaga ficheiros ignorados em silêncio).

---

## O BLOCO (copie daqui pra baixo; escreva o módulo SÓ na 1ª linha)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/<módulo>

O nome após "line/" acima é o NOVO MÓDULO. Todo o resto deste briefing
deriva dele — nos comandos ele aparece como $MODULO: substitua pelo
nome literal ao executar (env não persiste entre chamadas de shell).
Sua branch:    line/$MODULO
Sua worktree:  Worktrees/line-$MODULO/   (você vai criá-la agora)

FASE 1 — SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh
      → tem que dizer `workstation`. Disse `constrained`? PARE:
        esta máquina opera em Modo C, linhas são proibidas aqui.
2. git status -sb
      → você está na RAIZ do repo primário, branch main. Arquivos
        M/?? alheios podem existir (outros agentes): NÃO toque neles.
3. git pull --ff-only origin main
      → falhou (rede/divergência)? Siga com o main local e reporte.
4. mkdir -p Worktrees
   git worktree add -b line/$MODULO Worktrees/line-$MODULO main
      → a branch do novo módulo já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-$MODULO line/$MODULO
        e em seguida, DENTRO dela, a FASE 1 do
        MODELO_TROCA_DE_AGENTE_NA_LINHA.md (o `git cherry` ANTES do
        rebase: uma linha integrada por rebase tem hashes velhos)
5. cd Worktrees/line-$MODULO
   git branch --show-current        # DEVE imprimir a sua branch
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina (config vai no .git comum). Falhou por
        "mergiraf not found"? NÃO é bloqueio: git faz fallback pro merge
        embutido. Reporte a linha do ✗ e siga (Enio instala depois).
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/STACK_VERSOES.md       → tudo (1 pagina)
        as versoes que voce usa (gateadas contra o Cargo.lock; nao se
        copiam para aqui) e as tres regras que um agente novo erra.
        NUNCA escreva uma versao de memoria: `bash scripts/stack-audit.sh
        --tetos` responde.
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha do novo módulo pronta em Worktrees/line-$MODULO.
   Aguardo a tarefa." — e PARE. A tarefa vem na próxima mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-$MODULO/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar crates/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do novo módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107): a integração roda
   scripts/foundational-integrate.sh (gate da árvore combinada) e o
   Mergiraf funde o resíduo textual. PARE e reporte ao Enio SÓ se: (a)
   for contrato congelado (§4, exige ADR), ou (b) o rebase conflitar em
   código FORA dos seus arquivos (colisão de mesmo-símbolo com outra
   linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO em vez de engordar um compartilhado, ponto de
   extensão append-only, id/const/variant novo = próximo livre + ANOTE
   no handoff (regra H). O desenho completo está numa porta só:
   DIRETRIZ §1.5.2.1, que o passo 8 já te manda ler.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar, com o
   log gravado (`git rebase main 2>&1 | tee target/rebase.log`).
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5). Conflito em código
   fora da sua pasta = você violou a regra B. ⚠️ `Solved` do Mergiraf
   não é prova: numa lista partilhada ele larga a remoção de um lado
   (13/09, 2 de 130) — `git range-diff ORIG_HEAD...HEAD` antes de seguir.
E. Fechamento do módulo = `/pd-linha-fechar` (DIRETRIZ §1.5.9): gate
   batched 1× — `BASE=$(git merge-base main HEAD) bash
   scripts/nextest-impacted.sh` · `CARGO_BUILD_WARNINGS=deny cargo check
   --workspace --all-targets` · clippy --all-targets · `cargo machete` ·
   `bash scripts/check-standalone-optional.sh` · `bash
   scripts/check-workflow-packages.sh` · auditoria ≥2 lentes + DIRETIVA
   §3-§5. Então PARE — NÃO integre nem faça ship por conta própria. Quem funde
   as linhas é um AGENTE INTEGRADOR DEDICADO, e só por ORDEM EXPLÍCITA
   do Enio (DIRETRIZ §1.5.3–1.5.4). Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria. É ordem
   EXPLÍCITA do Enio, feita pelo integrador (DIRETRIZ §1.5.4 + §8).
   Integrar ou pushar sem ordem = violação do protocolo.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar): escreva o
   handoff que o Enio passa ao integrador (DIRETRIZ §1.5.9) — branch/HEAD/
   base; foundational tocado + por quê; ids/consts/variants novos com
   valores (colisão!); contratos congelados encostados (deve ser nenhum);
   o que só o ship.sh pega (fmt pré-fork/deps machete/clippy latente); o
   que smoke-testar. E também: toda entrada de lista/catraca que você
   BAIXOU e todo item PARTILHADO cujos usos apagou (duas linhas que apagam
   usos do mesmo item deixam um `dead_code` que só a árvore combinada
   tem); o delta de linhas da `shells/desktop` (o tecto dela SOMA entre
   linhas); e as premissas deste briefing que a medição derrubou.
   Reporte "linha pronta + handoff" e ESPERE.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados (CLAUDE.md §6)
   são intocáveis nesta linha.
I. DEIXE O SMOKE COMPILADO. O ÚLTIMO passo da linha — depois do commit
   final e da limpeza do incremental — é construir, DENTRO da sua
   worktree, o binário do comando que você vai entregar ao Enio:
   `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile
   smoke` (+ as `--features` de
   cada smoke que as exija; `--release` só para smoke de PERFORMANCE —
   o `smoke` reconstrói em 3 s, o `release` em 161 s, medido 10/09). Nada do seu dia o produz: `cargo check`
   não gera código e o gate roda no perfil `ci-test`, que é outro
   target/ — sem este passo o primeiro gesto dele é esperar o build
   mais caro do repo. Rode 2× e cole a 2ª saída no handoff ("Finished"
   em segundos, ZERO "Compiling"): é a prova. Compile a MESMA linha de
   comando que entrega (pacote, perfil, features e a árvore do `cd`) —
   qualquer diferença é outro build. Detalhe: DIRETRIZ §1.5.9 item 9.
J. RÉGUA E INSTRUMENTOS. (1) A régua da linha é o MERGE-BASE: todo diff,
   contagem e "antes" é contra `git merge-base main HEAD`, nunca contra o
   `main` que anda. (2) Código de família vive em `crates/ph2d-app-<fam>`;
   a shell é composição, com tecto que só desce (`the_shell_only_shrinks`):
   acima dele MOVA, nunca suba o número. Vai mover código da shell?
   `git grep -n 'shells/desktop/src' -- crates tools` ANTES: cada leitor
   por caminho é gate seu. (3) O shell das ferramentas é zsh: `for f in
   $LISTA` itera UMA vez e um glob sem aspas aborta — verificação que
   enumera vai num ficheiro `bash` com arrays e controlo positivo.
   (4) Instrumento que o handoff cita (sonda, prova, extractor) vive
   VERSIONADO (`scripts/` ou `docs/<Módulo>/ferramentas/`), nunca numa
   pasta não rastreada da worktree: ela morre com a worktree. Script mais
   novo que a sua worktree: chame-o pelo caminho absoluto do primário.
K. A MÁQUINA É PARTILHADA — e há um guarda que o obriga. TODO comando
   pesado (cargo test/build/run/nextest/bench/clippy) vai por
   `bash scripts/ph2d-run.sh <cmd>`: ele põe o comando numa fatia que é
   da LINHA — CPU ≤ 50% dos núcleos, RAM ≤ 24G sem swap, prazo 30 min, e
   mata a ÁRVORE INTEIRA no fim. O laço interno (`cargo check`/`fmt`) NÃO
   passa por aqui, de propósito. Se você digitar o comando cru, o guarda
   `.claude/hooks/tecto-de-recursos.sh` recusa e devolve a linha corrigida.
   K1. TOCA NA PLACA? `PH2D_GPU=1 bash scripts/ph2d-run.sh <cmd>` — gates
       de GPU, smoke, sondas de device. A placa é de EXCLUSÃO, não de
       fatia: 50% dela NÃO é exprimível nesta máquina (MIG [N/A], dmem
       não delegado, compute mode só cobre CUDA) e o defeito real é
       SEGURAR, não partilhar — em 14/09 uma sonda pendurada ficou com o
       driver 56 MINUTOS e parou os smokes do dono. Recusa com exit 75 =
       outra linha está na placa: ESPERE, não force.
   K2. NADA DE VIGIA DE FUNDO SEM PRAZO. Um `until …; do sleep 45; done`
       em segundo plano não termina sozinho e o silêncio dele lê-se igual
       a "ainda a trabalhar". Quando o trabalho é um comando de fundo
       desta sessão, o próprio arnês avisa ao terminar — vigia só para
       estado que ele não vê, e sempre com `timeout`.
   K3. ANTES DE ACUSAR UM PROCESSO DE PENDURADO, MEÇA: leia utime+stime
       de /proc/<pid>/stat DUAS vezes. O `ps` mostra a média da VIDA do
       processo — um binário BLOQUEADO a segurar a GPU lê-se ali como 95%.
   K4. VARRA ANTES DE SAIR: `pgrep -af 'ph2d|cargo|rustc'` e
       `fuser -v /dev/dri/*` no fecho da linha. Matar quem lançou NÃO
       mata o teste (ele reparenta-se ao systemd --user).
   Medições, tectos e como subir um: docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md
═══════════════════════════════════════════════════════════════════
```

---

## Encerrar uma linha (quando o módulo morreu de vez, pós-integração)

```bash
cd ~/Documentos/Projetos/PH2D          # raiz (ou git -C ../.. de dentro dela)
W=Worktrees/line-<módulo>
git -C "$W" status --porcelain --ignored --untracked-files=normal   # o que o remove APAGARIA
git cherry main line/<módulo> | grep -c '^+'                        # commits que o main não tem
```

1. **O que não se regenera sai ANTES, verificado** (`cp -a` + `diff -r`): pastas de instrumento
   (`.cauda-*`), projectos gravados, saídas de spike, repositórios de referência (os de
   `docs/UI_New_and_Simple/referencias/` vão para o primário, onde o `.gitignore` os espera).
   ⚠️ `git worktree remove` **apaga ignorados em silêncio** e recusa não-rastreados — o que
   empurra para o `--force`, que apaga os dois. Só `target/`, `__pycache__/` e as fixtures geradas
   idênticas às do primário (`assets/sprites/`) se deitam fora.
2. **Os portões da [DIRETIVA_FIM_DE_DIA](DIRETIVA_FIM_DE_DIA.md) §1**, re-checados logo antes:
   ninguém constrói, executa ou tem `cwd` dentro dela; fonte limpa. Só então
   `git worktree remove --force "$W"`.
3. **O ramo:** `git cherry` sem `+` ⇒ tudo está no `main` ⇒ `git branch -D line/<módulo>` é seguro
   (o `-d` recusa uma linha integrada por REBASE, que ficou com hashes velhos). Com `+` ⇒ deixe o
   ramo e reporte: há commit cujo patch o `main` não tem.

Linha que continua na próxima jornada **não precisa** disso — fica aberta; o agente
seguinte usa o mesmo bloco (o passo 4 tem a rota "linha reaberta").
