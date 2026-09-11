# Os SEIS blocos de abertura — W2: partir a shell (2026-09-11)

> Gerado de um molde só (o [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md) + a tarefa de cada linha,
> dos [briefings](BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md)), para os seis não divergirem.
> **Uma janela nova do VSCode por linha, na pasta do repo; cole o bloco inteiro como 1.ª mensagem.**
> ⚠️ **Desvio deliberado do MODELO:** o passo 9 dele manda o agente parar e esperar a tarefa; aqui a tarefa
> já está decidida e vem no mesmo bloco, para ser UMA colagem por janela.

## `line/app-host` — L0 — o SUBSTRATO + o piloto field3d + o molde

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/app-host

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
   git worktree add -b line/app-host Worktrees/line-app-host main
      → a branch já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-app-host line/app-host
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-app-host
   git branch --show-current        # DEVE imprimir line/app-host
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina. Falhou por "mergiraf not found"?
        NÃO é bloqueio: git faz fallback. Reporte o ✗ e siga.
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/STACK_VERSOES.md       → tudo (1 página)
        Rust 1.98/edition 2024, wgpu 29, vello 0.10, parley 0.11,
        rapier2d 0.35, bevy_ecs 0.19. NUNCA escreva uma versão de
        memória: `bash scripts/stack-audit.sh --tetos` responde.
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6 (§6.7 inclusive)
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha pronta em Worktrees/line-app-host." — e siga direto
   para A TAREFA abaixo: ela já está decidida, não espere mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-app-host/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar shells/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do seu módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107). PARE e reporte ao Enio
   SÓ se: (a) for contrato congelado (CLAUDE.md §6, exige ADR), ou (b)
   o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO, ponto de extensão append-only, id/const/
   variant novo = próximo livre + ANOTE no handoff (regra H).
   DIRETRIZ §1.5.2.1.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5).
E. Fechamento = gate batched (nextest-impacted + clippy --all-targets
   + auditoria ≥2 lentes + DIRETIVA §3-§5). Então PARE — NÃO integre
   nem faça ship. Quem funde é um AGENTE INTEGRADOR DEDICADO, só por
   ORDEM EXPLÍCITA do Enio. Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados são intocáveis.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar, DIRETRIZ
   §1.5.9): branch/HEAD/base; foundational tocado + por quê; ids/
   consts/variants novos com valores; contratos encostados (deve ser
   nenhum); o que só o ship.sh pega; o que smoke-testar; e os números
   da PROVA abaixo. Reporte "linha pronta + handoff" e ESPERE.
I. DEIXE O SMOKE COMPILADO — último passo, depois do commit final e de
   `rm -rf target/*/incremental`:
   `cargo build -p ph2d-host-desktop --profile smoke`
   (⚠️ --profile smoke, NÃO --release: 3 s contra 161 s, medido 10/09;
   --release só para smoke de PERFORMANCE). Rode 2× e cole a 2ª saída
   no handoff ("Finished" em segundos, ZERO "Compiling") — é a prova.
═══════════════════════════════════════════════════════════════════
A TAREFA — W2/L0: o substrato por onde uma família sai da shell

CONTEXTO (leia antes de tocar num ficheiro):
  docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md  → §0 a §4, INTEIRO
  docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md      → §4-C2 (o mecanismo e o censo)
  DIRETRIZ §6.7                                                          → as regras que a auditoria deixou

O PROBLEMA, medido: `shells/desktop` e' UMA crate de 493 k linhas (307 k de produto +
186 k de testes no src/), dobrou em quatro semanas, e e' a ultima unidade de todo build
grande (34-45 s sozinha no fim do gate; 16,8 s de front-end monotarefa a frio). Dentro
dela vivem familias inteiras que so' ali estao por inercia.

VOCE E' A PRIMEIRA DAS SEIS LINHAS e a unica que constroi substrato. As outras cinco
(app-motion, app-physics, app-sculpt3d, app-vec, app-flip) abrem hoje e estao a fazer a
FASE A delas (so' dentro dos ficheiros da propria familia). O CORTE delas espera o seu
molde. => o que voce entrega tem de servir a CINCO familias, nao so' ao seu piloto.

ENTREGAVEIS, por ordem:

1. crates/ph2d-app-host — a INTERFACE (traits/tipos, zero logica): o que uma familia
   precisa da shell e nao pode obter de uma crate de modulo.
   ⚠️ ADR-0075 PRIMEIRO: estado de familia vira recurso/componente do ECS e comunicacao
   vira evento/recurso. O trait de host e' o FALLBACK para o que e' genuinamente da shell
   (janela, gfx, paineis, captura de undo) — nao a primeira ferramenta. Se uma familia
   precisa de um metodo por campo de `App` que hoje toca, o campo e' DELA e sai com ela.
   O censo diz o que cobrir: as cenas de smoke usam SEIS modulos internos da shell
   (instance_docs, image_import, instantiate, render_loop, vec_entities, audio) e 22
   ficheiros escrevem `impl App` (no seu piloto: um, field3d_input.rs).

2. crates/ph2d-app-registry-init — o ponto de extensao APPEND-ONLY, pelo precedente da
   casa: ph2d-panel-registry-init (um register_all_*() chamado UMA vez em init.rs, bloco
   GERADO por um -sync, gate de staleness). ⚠️ Projete-o para CINCO linhas o estenderem
   sem colidir: o bloco e' gerado de uma varredura (crates/ph2d-app-*), nunca escrito a'
   mao. `inventory`/`linkme` NAO estao no Cargo.lock; um pacote novo passa pelo
   `bash scripts/stack-audit.sh --tetos` e e' decisao a registar no handoff.

3. O PILOTO: crates/ph2d-app-field3d — os 53 ficheiros de produto field3d_* (15 207 LOC),
   as 17 cenas de smoke (6 558) e os 16 990 LOC de testes deles. E' a familia mais
   desacoplada (1 impl App, 33 membros de self., e as 104 refs `crate::` sao quase todas
   para outros field3d_*) — por isso e' o piloto. Fica na shell: main.rs/init.rs, `App`,
   o esqueleto do laco, o input_dispatch (roteador).
   ⛔ Nada de `pub` a mais na shell para «facilitar»: a shell e' um bin; uma crate de
   familia NAO depende dela.

4. docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md — o molde que as
   outras cinco seguem: o que se move e para onde · como a familia se regista · como se
   resolve cada um dos seis modulos internos e um `impl App` · os comandos da prova ·
   as armadilhas que o piloto encontrou, COM O NUMERO.
   ⚠️ Este doc e' a metade mais valiosa da linha: uma extraccao que so' o piloto sabe
   fazer nao e' um molde.

5. Handoff de integracao (DIRETRIZ §1.5.9) + UMA linha no CLAUDE.md §5 (modulo 3D
   Modeling) + uma linha no §5.0 se nascer lei nova.

⛔ RECUSA MEDIDA, nao a reconstrua: uma feature `smokes` por #[cfg] (auditoria §9) — fora
do default o gate e o CI deixam de compilar as cenas EM SILENCIO; no default ninguem
compila sem ela. O que muda o tecto e' a CRATE.

A PROVA (entregue os cinco numeros no handoff):
  a) Nenhum teste se perde: `cargo nextest list --workspace --cargo-profile ci-test`
     ANTES (agora, antes de mover) e DEPOIS, comparados por
     `python3 scripts/nextest-list-diff.py antes.txt depois.txt` — ONLY-A tem de ser 0.
  b) Roteadores e niveis de smoke iguais antes/depois; as cenas que NENHUM doc cita pelo
     numero apagam-se (lista no handoff) — ordem do Enio.
  c) A shell encolheu: LOC de shells/desktop/src e a unidade `ph2d-host-desktop
     bin (check-test)` num `cargo check -p ph2d-host-desktop --tests --timings -j 32`
     A FRIO, num target novo, antes/depois.
  d) Gate de fecho (DIRETRIZ §1.5.9). ⚠️ Armadilhas que a W1 pagou e uma extraccao
     repete: include_str! com caminho relativo muda de sitio quando o ficheiro se move;
     gates que varrem shells/desktop/src por nome de familia; file_loc_caps.rs lista
     caminhos; 588 citacoes de caminho nos docs foram reapontadas por script na W1 —
     faca o mesmo (fora de docs/archive; logs .txt intocados).
  e) O smoke do Enio, na sua worktree, com --profile smoke, sobre as MESMAS cenas de
     antes: o comportamento tem de ser identico — a extraccao nao muda produto.

SMOKE A ENTREGAR (deixe-o COMPILADO, regra I):
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-host && env PH2D_FIELD_SMOKE=11 cargo run -p ph2d-host-desktop --profile smoke
  (o pill MODEL e as cenas do field3d_smoke_scenes.rs — as mesmas de antes)
═══════════════════════════════════════════════════════════════════
```

## `line/app-motion` — L1 — a família motion (232 ficheiros, 56 460 LOC de produto)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/app-motion

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
   git worktree add -b line/app-motion Worktrees/line-app-motion main
      → a branch já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-app-motion line/app-motion
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-app-motion
   git branch --show-current        # DEVE imprimir line/app-motion
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina. Falhou por "mergiraf not found"?
        NÃO é bloqueio: git faz fallback. Reporte o ✗ e siga.
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/STACK_VERSOES.md       → tudo (1 página)
        Rust 1.98/edition 2024, wgpu 29, vello 0.10, parley 0.11,
        rapier2d 0.35, bevy_ecs 0.19. NUNCA escreva uma versão de
        memória: `bash scripts/stack-audit.sh --tetos` responde.
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6 (§6.7 inclusive)
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha pronta em Worktrees/line-app-motion." — e siga direto
   para A TAREFA abaixo: ela já está decidida, não espere mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-app-motion/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar shells/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do seu módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107). PARE e reporte ao Enio
   SÓ se: (a) for contrato congelado (CLAUDE.md §6, exige ADR), ou (b)
   o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO, ponto de extensão append-only, id/const/
   variant novo = próximo livre + ANOTE no handoff (regra H).
   DIRETRIZ §1.5.2.1.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5).
E. Fechamento = gate batched (nextest-impacted + clippy --all-targets
   + auditoria ≥2 lentes + DIRETIVA §3-§5). Então PARE — NÃO integre
   nem faça ship. Quem funde é um AGENTE INTEGRADOR DEDICADO, só por
   ORDEM EXPLÍCITA do Enio. Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados são intocáveis.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar, DIRETRIZ
   §1.5.9): branch/HEAD/base; foundational tocado + por quê; ids/
   consts/variants novos com valores; contratos encostados (deve ser
   nenhum); o que só o ship.sh pega; o que smoke-testar; e os números
   da PROVA abaixo. Reporte "linha pronta + handoff" e ESPERE.
I. DEIXE O SMOKE COMPILADO — último passo, depois do commit final e de
   `rm -rf target/*/incremental`:
   `cargo build -p ph2d-host-desktop --profile smoke`
   (⚠️ --profile smoke, NÃO --release: 3 s contra 161 s, medido 10/09;
   --release só para smoke de PERFORMANCE). Rode 2× e cole a 2ª saída
   no handoff ("Finished" em segundos, ZERO "Compiling") — é a prova.
═══════════════════════════════════════════════════════════════════
A TAREFA — W2/L1: a família `motion` sai da shell, em DUAS FASES

CONTEXTO (leia antes de tocar num ficheiro):
  docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md  → §0 a §3 e §5, INTEIRO
  docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md      → §4-C2 (o mecanismo e o censo)
  DIRETRIZ §6.7                                                          → as regras que a auditoria deixou

O PROBLEMA, medido: `shells/desktop` e' UMA crate de 493 k linhas que dobrou em quatro
semanas e e' a ultima unidade de todo build grande (34-45 s sozinha no fim do gate).
A sua familia la' dentro: 232 ficheiros, 56 460 LOC de produto, 141 ficheiros de cena, 0 `impl App`, 59 membros de self..

SEIS LINHAS ABREM HOJE. A line/app-host constroi o SUBSTRATO (crates ph2d-app-host +
ph2d-app-registry-init) e o molde (HOWTO_partir_uma_familia_da_shell.md), com o field3d
como piloto. Por isso o seu trabalho e' em duas fases — e a FASE A comeca JA':

═══ FASE A — desde já (só ficheiros da sua família; ZERO interface nova com a shell) ═══

A1. CENSO E PODA DAS CENAS. Cada cena de smoke do roteador desta familia que NENHUM doc
    cita pelo numero APAGA-SE (ordem do Enio, 10/09: uma cena que ninguem cita sao ~1 k
    linhas pagas por todos em cada build, por ninguem). O censo:
      grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE=[0-9]+' docs CLAUDE.md project-memory .claude \
        --include='*.md' | sort -u          # (fora de docs/archive)
    contra os niveis que o roteador da familia de facto oferece. Lista das apagadas no
    handoff, com roteador e nivel. ⛔ Uma cena citada como PENDENTE de smoke fica.

A2. DESFAZER O ACOPLAMENTO, dentro da familia. Cada `impl App` desta familia vira funcao
    livre sobre &mut World / recursos / APIs das crates do modulo; os campos de `App` que
    SO' esta familia le' saem de `App` para UM estado da familia (MotionState num recurso do
    ECS, ou uma unica struct num unico campo de `App` — ADR-0075). Numeros de partida no
    censo acima; os de chegada vao no handoff.
    ⛔ O que precisa da shell e NAO e' da familia (janela, gfx, paineis, captura de undo)
    FICA como esta' e e' NOMEADO no handoff — e' a lista que a L0 tem de cobrir.

A3. AGRUPAR: shells/desktop/src/motion_*.rs → shells/desktop/src/motion/ (um `mod motion;`
    no main.rs no lugar de N linhas; os testes vao junto). O corte da Fase B passa a ser
    mover UMA pasta.

A4. A CRATE NASCE: crates/ph2d-app-motion com Cargo.toml + o que JA' so' depende de crates
    do modulo e do ECS (construtores de cena puros, leis) — a shell chama-os; a crate NAO
    depende da shell (ela e' um bin).

A5. Prova (abaixo) sobre a Fase A + HANDOFF INTERMEDIO com os numeros: LOC movidas,
    `impl App` de N para M, membros de N para M, cenas apagadas, e a lista do que so' o
    substrato resolve.

═══ FASE B — só depois de a line/app-host INTEGRAR (o Enio avisa) ═══

B1. git rebase main
B2. Ler INTEIRO o docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
B3. O corte: o resto da pasta vai para crates/ph2d-app-motion; a familia regista-se em
    ph2d-app-registry-init; o que e' genuinamente da shell passa pelo trait de
    ph2d-app-host. Prova outra vez + handoff final.
⛔ Se o HOWTO nao cobrir algo de que a familia precisa: PARE e reporte com a lista. A
extensao do substrato e' decisao do integrador, nunca de cinco linhas em paralelo.

═══ A PROVA (em CADA fase; os numeros vao no handoff) ═══
  a) Nenhum teste se perde: `cargo nextest list --workspace --cargo-profile ci-test`
     ANTES (agora, antes de mover) e DEPOIS, comparados por
     `python3 scripts/nextest-list-diff.py antes.txt depois.txt` — ONLY-A tem de ser 0.
     Os MOVED (mesmo nome, pacote novo) sao o esperado; os ONLY-B listam-se.
  b) Roteadores e niveis de smoke iguais antes/depois, menos os apagados de proposito em
     A1; os gates no_two_*_scenes_claim_the_same_level continuam a correr.
  c) A shell encolheu: LOC de shells/desktop/src e a unidade `ph2d-host-desktop
     bin (check-test)` num `cargo check -p ph2d-host-desktop --tests --timings -j 32`
     A FRIO, num target novo, antes/depois.
  d) Gate de fecho (DIRETRIZ §1.5.9). ⚠️ Armadilhas que a W1 pagou e uma extraccao
     repete: include_str! com caminho relativo muda de sitio quando o ficheiro se move;
     gates que varrem shells/desktop/src por nome de familia; file_loc_caps.rs lista
     caminhos; 588 citacoes de caminho nos docs foram reapontadas por script na W1 —
     faca o mesmo (fora de docs/archive; logs .txt intocados).
  e) O smoke do Enio, na sua worktree, com --profile smoke, sobre as MESMAS cenas de
     antes: comportamento identico — a extraccao NAO muda produto.

⚠️ ESPECIFICO DESTA FAMILIA: e' a MAIOR (232 ficheiros, 43 892 LOC de teste) e a de mais
roteadores (PH2D_MOTION_*, PH2D_GPU_COOK_DEMO, PH2D_AUTOFIX_SMOKE, PH2D_LENS_SMOKE, …);
quem conta as cenas e' o motion_state_demo_router.rs, nunca uma nota (CLAUDE.md §5.0).
⛔ A cena =114 (o motion.collide dentro de uma simulacao) esta CITADA COMO PENDENTE de
smoke no §5 — ela NAO e' uma cena orfa: fica.

SMOKE A ENTREGAR (deixe-o COMPILADO, regra I) — o molde, com o roteador e o nivel que
esta familia de facto tem:
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-motion && env PH2D_<ROTEADOR>=<n> cargo run -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

## `line/app-physics` — L2 — a família physics (94 ficheiros, 22 783 LOC de produto)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/app-physics

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
   git worktree add -b line/app-physics Worktrees/line-app-physics main
      → a branch já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-app-physics line/app-physics
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-app-physics
   git branch --show-current        # DEVE imprimir line/app-physics
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina. Falhou por "mergiraf not found"?
        NÃO é bloqueio: git faz fallback. Reporte o ✗ e siga.
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/STACK_VERSOES.md       → tudo (1 página)
        Rust 1.98/edition 2024, wgpu 29, vello 0.10, parley 0.11,
        rapier2d 0.35, bevy_ecs 0.19. NUNCA escreva uma versão de
        memória: `bash scripts/stack-audit.sh --tetos` responde.
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6 (§6.7 inclusive)
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha pronta em Worktrees/line-app-physics." — e siga direto
   para A TAREFA abaixo: ela já está decidida, não espere mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-app-physics/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar shells/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do seu módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107). PARE e reporte ao Enio
   SÓ se: (a) for contrato congelado (CLAUDE.md §6, exige ADR), ou (b)
   o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO, ponto de extensão append-only, id/const/
   variant novo = próximo livre + ANOTE no handoff (regra H).
   DIRETRIZ §1.5.2.1.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5).
E. Fechamento = gate batched (nextest-impacted + clippy --all-targets
   + auditoria ≥2 lentes + DIRETIVA §3-§5). Então PARE — NÃO integre
   nem faça ship. Quem funde é um AGENTE INTEGRADOR DEDICADO, só por
   ORDEM EXPLÍCITA do Enio. Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados são intocáveis.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar, DIRETRIZ
   §1.5.9): branch/HEAD/base; foundational tocado + por quê; ids/
   consts/variants novos com valores; contratos encostados (deve ser
   nenhum); o que só o ship.sh pega; o que smoke-testar; e os números
   da PROVA abaixo. Reporte "linha pronta + handoff" e ESPERE.
I. DEIXE O SMOKE COMPILADO — último passo, depois do commit final e de
   `rm -rf target/*/incremental`:
   `cargo build -p ph2d-host-desktop --profile smoke`
   (⚠️ --profile smoke, NÃO --release: 3 s contra 161 s, medido 10/09;
   --release só para smoke de PERFORMANCE). Rode 2× e cole a 2ª saída
   no handoff ("Finished" em segundos, ZERO "Compiling") — é a prova.
═══════════════════════════════════════════════════════════════════
A TAREFA — W2/L2: a família `physics` sai da shell, em DUAS FASES

CONTEXTO (leia antes de tocar num ficheiro):
  docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md  → §0 a §3 e §5, INTEIRO
  docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md      → §4-C2 (o mecanismo e o censo)
  DIRETRIZ §6.7                                                          → as regras que a auditoria deixou

O PROBLEMA, medido: `shells/desktop` e' UMA crate de 493 k linhas que dobrou em quatro
semanas e e' a ultima unidade de todo build grande (34-45 s sozinha no fim do gate).
A sua familia la' dentro: 94 ficheiros, 22 783 LOC de produto, 80 ficheiros de cena, **22** `impl App`, 126 membros de self..

SEIS LINHAS ABREM HOJE. A line/app-host constroi o SUBSTRATO (crates ph2d-app-host +
ph2d-app-registry-init) e o molde (HOWTO_partir_uma_familia_da_shell.md), com o field3d
como piloto. Por isso o seu trabalho e' em duas fases — e a FASE A comeca JA':

═══ FASE A — desde já (só ficheiros da sua família; ZERO interface nova com a shell) ═══

A1. CENSO E PODA DAS CENAS. Cada cena de smoke do roteador desta familia que NENHUM doc
    cita pelo numero APAGA-SE (ordem do Enio, 10/09: uma cena que ninguem cita sao ~1 k
    linhas pagas por todos em cada build, por ninguem). O censo:
      grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE=[0-9]+' docs CLAUDE.md project-memory .claude \
        --include='*.md' | sort -u          # (fora de docs/archive)
    contra os niveis que o roteador da familia de facto oferece. Lista das apagadas no
    handoff, com roteador e nivel. ⛔ Uma cena citada como PENDENTE de smoke fica.

A2. DESFAZER O ACOPLAMENTO, dentro da familia. Cada `impl App` desta familia vira funcao
    livre sobre &mut World / recursos / APIs das crates do modulo; os campos de `App` que
    SO' esta familia le' saem de `App` para UM estado da familia (PhysicsState num recurso do
    ECS, ou uma unica struct num unico campo de `App` — ADR-0075). Numeros de partida no
    censo acima; os de chegada vao no handoff.
    ⛔ O que precisa da shell e NAO e' da familia (janela, gfx, paineis, captura de undo)
    FICA como esta' e e' NOMEADO no handoff — e' a lista que a L0 tem de cobrir.

A3. AGRUPAR: shells/desktop/src/physics_*.rs → shells/desktop/src/physics/ (um `mod physics;`
    no main.rs no lugar de N linhas; os testes vao junto). O corte da Fase B passa a ser
    mover UMA pasta.

A4. A CRATE NASCE: crates/ph2d-app-physics com Cargo.toml + o que JA' so' depende de crates
    do modulo e do ECS (construtores de cena puros, leis) — a shell chama-os; a crate NAO
    depende da shell (ela e' um bin).

A5. Prova (abaixo) sobre a Fase A + HANDOFF INTERMEDIO com os numeros: LOC movidas,
    `impl App` de N para M, membros de N para M, cenas apagadas, e a lista do que so' o
    substrato resolve.

═══ FASE B — só depois de a line/app-host INTEGRAR (o Enio avisa) ═══

B1. git rebase main
B2. Ler INTEIRO o docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
B3. O corte: o resto da pasta vai para crates/ph2d-app-physics; a familia regista-se em
    ph2d-app-registry-init; o que e' genuinamente da shell passa pelo trait de
    ph2d-app-host. Prova outra vez + handoff final.
⛔ Se o HOWTO nao cobrir algo de que a familia precisa: PARE e reporte com a lista. A
extensao do substrato e' decisao do integrador, nunca de cinco linhas em paralelo.

═══ A PROVA (em CADA fase; os numeros vao no handoff) ═══
  a) Nenhum teste se perde: `cargo nextest list --workspace --cargo-profile ci-test`
     ANTES (agora, antes de mover) e DEPOIS, comparados por
     `python3 scripts/nextest-list-diff.py antes.txt depois.txt` — ONLY-A tem de ser 0.
     Os MOVED (mesmo nome, pacote novo) sao o esperado; os ONLY-B listam-se.
  b) Roteadores e niveis de smoke iguais antes/depois, menos os apagados de proposito em
     A1; os gates no_two_*_scenes_claim_the_same_level continuam a correr.
  c) A shell encolheu: LOC de shells/desktop/src e a unidade `ph2d-host-desktop
     bin (check-test)` num `cargo check -p ph2d-host-desktop --tests --timings -j 32`
     A FRIO, num target novo, antes/depois.
  d) Gate de fecho (DIRETRIZ §1.5.9). ⚠️ Armadilhas que a W1 pagou e uma extraccao
     repete: include_str! com caminho relativo muda de sitio quando o ficheiro se move;
     gates que varrem shells/desktop/src por nome de familia; file_loc_caps.rs lista
     caminhos; 588 citacoes de caminho nos docs foram reapontadas por script na W1 —
     faca o mesmo (fora de docs/archive; logs .txt intocados).
  e) O smoke do Enio, na sua worktree, com --profile smoke, sobre as MESMAS cenas de
     antes: comportamento identico — a extraccao NAO muda produto.

⚠️ ESPECIFICO DESTA FAMILIA: e' a MAIS ACOPLADA do repo — 22 `impl App` e 126 membros.
E' a linha que mais vai querer pedir ao substrato. ⛔ NAO desenhe a porta: o que so' o
ph2d-app-host pode resolver NOMEIA-SE no handoff intermedio, com a lista, e o Enio/
integrador decide. Regra B: nunca negocie com outra linha.
⚠️ A cena =15 tem as paredes CINEMATICAS de proposito desde 30/08 (§5) — nao a
«arrume»: com paredes estaticas ela ensina o contrario do que diz.

SMOKE A ENTREGAR (deixe-o COMPILADO, regra I) — o molde, com o roteador e o nivel que
esta familia de facto tem:
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-physics && env PH2D_<ROTEADOR>=<n> cargo run -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

## `line/app-sculpt3d` — L3 — a família sculpt3d (83 ficheiros, 23 078 LOC de produto)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/app-sculpt3d

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
   git worktree add -b line/app-sculpt3d Worktrees/line-app-sculpt3d main
      → a branch já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-app-sculpt3d line/app-sculpt3d
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-app-sculpt3d
   git branch --show-current        # DEVE imprimir line/app-sculpt3d
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina. Falhou por "mergiraf not found"?
        NÃO é bloqueio: git faz fallback. Reporte o ✗ e siga.
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/STACK_VERSOES.md       → tudo (1 página)
        Rust 1.98/edition 2024, wgpu 29, vello 0.10, parley 0.11,
        rapier2d 0.35, bevy_ecs 0.19. NUNCA escreva uma versão de
        memória: `bash scripts/stack-audit.sh --tetos` responde.
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6 (§6.7 inclusive)
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha pronta em Worktrees/line-app-sculpt3d." — e siga direto
   para A TAREFA abaixo: ela já está decidida, não espere mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-app-sculpt3d/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar shells/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do seu módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107). PARE e reporte ao Enio
   SÓ se: (a) for contrato congelado (CLAUDE.md §6, exige ADR), ou (b)
   o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO, ponto de extensão append-only, id/const/
   variant novo = próximo livre + ANOTE no handoff (regra H).
   DIRETRIZ §1.5.2.1.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5).
E. Fechamento = gate batched (nextest-impacted + clippy --all-targets
   + auditoria ≥2 lentes + DIRETIVA §3-§5). Então PARE — NÃO integre
   nem faça ship. Quem funde é um AGENTE INTEGRADOR DEDICADO, só por
   ORDEM EXPLÍCITA do Enio. Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados são intocáveis.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar, DIRETRIZ
   §1.5.9): branch/HEAD/base; foundational tocado + por quê; ids/
   consts/variants novos com valores; contratos encostados (deve ser
   nenhum); o que só o ship.sh pega; o que smoke-testar; e os números
   da PROVA abaixo. Reporte "linha pronta + handoff" e ESPERE.
I. DEIXE O SMOKE COMPILADO — último passo, depois do commit final e de
   `rm -rf target/*/incremental`:
   `cargo build -p ph2d-host-desktop --profile smoke`
   (⚠️ --profile smoke, NÃO --release: 3 s contra 161 s, medido 10/09;
   --release só para smoke de PERFORMANCE). Rode 2× e cole a 2ª saída
   no handoff ("Finished" em segundos, ZERO "Compiling") — é a prova.
═══════════════════════════════════════════════════════════════════
A TAREFA — W2/L3: a família `sculpt3d` sai da shell, em DUAS FASES

CONTEXTO (leia antes de tocar num ficheiro):
  docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md  → §0 a §3 e §5, INTEIRO
  docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md      → §4-C2 (o mecanismo e o censo)
  DIRETRIZ §6.7                                                          → as regras que a auditoria deixou

O PROBLEMA, medido: `shells/desktop` e' UMA crate de 493 k linhas que dobrou em quatro
semanas e e' a ultima unidade de todo build grande (34-45 s sozinha no fim do gate).
A sua familia la' dentro: 83 ficheiros, 23 078 LOC de produto, 17 ficheiros de cena, 6 `impl App`, **180** membros de self..

SEIS LINHAS ABREM HOJE. A line/app-host constroi o SUBSTRATO (crates ph2d-app-host +
ph2d-app-registry-init) e o molde (HOWTO_partir_uma_familia_da_shell.md), com o field3d
como piloto. Por isso o seu trabalho e' em duas fases — e a FASE A comeca JA':

═══ FASE A — desde já (só ficheiros da sua família; ZERO interface nova com a shell) ═══

A1. CENSO E PODA DAS CENAS. Cada cena de smoke do roteador desta familia que NENHUM doc
    cita pelo numero APAGA-SE (ordem do Enio, 10/09: uma cena que ninguem cita sao ~1 k
    linhas pagas por todos em cada build, por ninguem). O censo:
      grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE=[0-9]+' docs CLAUDE.md project-memory .claude \
        --include='*.md' | sort -u          # (fora de docs/archive)
    contra os niveis que o roteador da familia de facto oferece. Lista das apagadas no
    handoff, com roteador e nivel. ⛔ Uma cena citada como PENDENTE de smoke fica.

A2. DESFAZER O ACOPLAMENTO, dentro da familia. Cada `impl App` desta familia vira funcao
    livre sobre &mut World / recursos / APIs das crates do modulo; os campos de `App` que
    SO' esta familia le' saem de `App` para UM estado da familia (Sculpt3dState num recurso do
    ECS, ou uma unica struct num unico campo de `App` — ADR-0075). Numeros de partida no
    censo acima; os de chegada vao no handoff.
    ⛔ O que precisa da shell e NAO e' da familia (janela, gfx, paineis, captura de undo)
    FICA como esta' e e' NOMEADO no handoff — e' a lista que a L0 tem de cobrir.

A3. AGRUPAR: shells/desktop/src/sculpt3d_*.rs → shells/desktop/src/sculpt3d/ (um `mod sculpt3d;`
    no main.rs no lugar de N linhas; os testes vao junto). O corte da Fase B passa a ser
    mover UMA pasta.

A4. A CRATE NASCE: crates/ph2d-app-sculpt3d com Cargo.toml + o que JA' so' depende de crates
    do modulo e do ECS (construtores de cena puros, leis) — a shell chama-os; a crate NAO
    depende da shell (ela e' um bin).

A5. Prova (abaixo) sobre a Fase A + HANDOFF INTERMEDIO com os numeros: LOC movidas,
    `impl App` de N para M, membros de N para M, cenas apagadas, e a lista do que so' o
    substrato resolve.

═══ FASE B — só depois de a line/app-host INTEGRAR (o Enio avisa) ═══

B1. git rebase main
B2. Ler INTEIRO o docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
B3. O corte: o resto da pasta vai para crates/ph2d-app-sculpt3d; a familia regista-se em
    ph2d-app-registry-init; o que e' genuinamente da shell passa pelo trait de
    ph2d-app-host. Prova outra vez + handoff final.
⛔ Se o HOWTO nao cobrir algo de que a familia precisa: PARE e reporte com a lista. A
extensao do substrato e' decisao do integrador, nunca de cinco linhas em paralelo.

═══ A PROVA (em CADA fase; os numeros vao no handoff) ═══
  a) Nenhum teste se perde: `cargo nextest list --workspace --cargo-profile ci-test`
     ANTES (agora, antes de mover) e DEPOIS, comparados por
     `python3 scripts/nextest-list-diff.py antes.txt depois.txt` — ONLY-A tem de ser 0.
     Os MOVED (mesmo nome, pacote novo) sao o esperado; os ONLY-B listam-se.
  b) Roteadores e niveis de smoke iguais antes/depois, menos os apagados de proposito em
     A1; os gates no_two_*_scenes_claim_the_same_level continuam a correr.
  c) A shell encolheu: LOC de shells/desktop/src e a unidade `ph2d-host-desktop
     bin (check-test)` num `cargo check -p ph2d-host-desktop --tests --timings -j 32`
     A FRIO, num target novo, antes/depois.
  d) Gate de fecho (DIRETRIZ §1.5.9). ⚠️ Armadilhas que a W1 pagou e uma extraccao
     repete: include_str! com caminho relativo muda de sitio quando o ficheiro se move;
     gates que varrem shells/desktop/src por nome de familia; file_loc_caps.rs lista
     caminhos; 588 citacoes de caminho nos docs foram reapontadas por script na W1 —
     faca o mesmo (fora de docs/archive; logs .txt intocados).
  e) O smoke do Enio, na sua worktree, com --profile smoke, sobre as MESMAS cenas de
     antes: comportamento identico — a extraccao NAO muda produto.

⚠️ ESPECIFICO DESTA FAMILIA: 180 membros de `self.` — o maior numero do censo, com so' 6
`impl App`: a maior parte e' estado que e' DELA e deve sair de `App` na Fase A.
⛔ A navegacao orbital MORA NA SHELL de proposito (§5: «nunca numa Tool» — e' isso que
mantem Tool=12 fora do caminho): o que fica na shell fica pela mesma razao, ESCRITA no
handoff. ⚠️ A `eared_sphere()` e as fixturas de paridade do tecido (86 tracos do oraculo)
nao podem mudar de sujeito — um gate cujo sujeito muda nao afirma nada.

SMOKE A ENTREGAR (deixe-o COMPILADO, regra I) — o molde, com o roteador e o nivel que
esta familia de facto tem:
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-sculpt3d && env PH2D_<ROTEADOR>=<n> cargo run -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

## `line/app-vec` — L4 — a família vec (68 ficheiros, 17 527 LOC de produto)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/app-vec

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
   git worktree add -b line/app-vec Worktrees/line-app-vec main
      → a branch já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-app-vec line/app-vec
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-app-vec
   git branch --show-current        # DEVE imprimir line/app-vec
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina. Falhou por "mergiraf not found"?
        NÃO é bloqueio: git faz fallback. Reporte o ✗ e siga.
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/STACK_VERSOES.md       → tudo (1 página)
        Rust 1.98/edition 2024, wgpu 29, vello 0.10, parley 0.11,
        rapier2d 0.35, bevy_ecs 0.19. NUNCA escreva uma versão de
        memória: `bash scripts/stack-audit.sh --tetos` responde.
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6 (§6.7 inclusive)
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha pronta em Worktrees/line-app-vec." — e siga direto
   para A TAREFA abaixo: ela já está decidida, não espere mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-app-vec/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar shells/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do seu módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107). PARE e reporte ao Enio
   SÓ se: (a) for contrato congelado (CLAUDE.md §6, exige ADR), ou (b)
   o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO, ponto de extensão append-only, id/const/
   variant novo = próximo livre + ANOTE no handoff (regra H).
   DIRETRIZ §1.5.2.1.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5).
E. Fechamento = gate batched (nextest-impacted + clippy --all-targets
   + auditoria ≥2 lentes + DIRETIVA §3-§5). Então PARE — NÃO integre
   nem faça ship. Quem funde é um AGENTE INTEGRADOR DEDICADO, só por
   ORDEM EXPLÍCITA do Enio. Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados são intocáveis.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar, DIRETRIZ
   §1.5.9): branch/HEAD/base; foundational tocado + por quê; ids/
   consts/variants novos com valores; contratos encostados (deve ser
   nenhum); o que só o ship.sh pega; o que smoke-testar; e os números
   da PROVA abaixo. Reporte "linha pronta + handoff" e ESPERE.
I. DEIXE O SMOKE COMPILADO — último passo, depois do commit final e de
   `rm -rf target/*/incremental`:
   `cargo build -p ph2d-host-desktop --profile smoke`
   (⚠️ --profile smoke, NÃO --release: 3 s contra 161 s, medido 10/09;
   --release só para smoke de PERFORMANCE). Rode 2× e cole a 2ª saída
   no handoff ("Finished" em segundos, ZERO "Compiling") — é a prova.
═══════════════════════════════════════════════════════════════════
A TAREFA — W2/L4: a família `vec` sai da shell, em DUAS FASES

CONTEXTO (leia antes de tocar num ficheiro):
  docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md  → §0 a §3 e §5, INTEIRO
  docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md      → §4-C2 (o mecanismo e o censo)
  DIRETRIZ §6.7                                                          → as regras que a auditoria deixou

O PROBLEMA, medido: `shells/desktop` e' UMA crate de 493 k linhas que dobrou em quatro
semanas e e' a ultima unidade de todo build grande (34-45 s sozinha no fim do gate).
A sua familia la' dentro: 68 ficheiros, 17 527 LOC de produto, 4 ficheiros de cena, 3 `impl App`, 89 membros de self..

SEIS LINHAS ABREM HOJE. A line/app-host constroi o SUBSTRATO (crates ph2d-app-host +
ph2d-app-registry-init) e o molde (HOWTO_partir_uma_familia_da_shell.md), com o field3d
como piloto. Por isso o seu trabalho e' em duas fases — e a FASE A comeca JA':

═══ FASE A — desde já (só ficheiros da sua família; ZERO interface nova com a shell) ═══

A1. CENSO E PODA DAS CENAS. Cada cena de smoke do roteador desta familia que NENHUM doc
    cita pelo numero APAGA-SE (ordem do Enio, 10/09: uma cena que ninguem cita sao ~1 k
    linhas pagas por todos em cada build, por ninguem). O censo:
      grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE=[0-9]+' docs CLAUDE.md project-memory .claude \
        --include='*.md' | sort -u          # (fora de docs/archive)
    contra os niveis que o roteador da familia de facto oferece. Lista das apagadas no
    handoff, com roteador e nivel. ⛔ Uma cena citada como PENDENTE de smoke fica.

A2. DESFAZER O ACOPLAMENTO, dentro da familia. Cada `impl App` desta familia vira funcao
    livre sobre &mut World / recursos / APIs das crates do modulo; os campos de `App` que
    SO' esta familia le' saem de `App` para UM estado da familia (VecState num recurso do
    ECS, ou uma unica struct num unico campo de `App` — ADR-0075). Numeros de partida no
    censo acima; os de chegada vao no handoff.
    ⛔ O que precisa da shell e NAO e' da familia (janela, gfx, paineis, captura de undo)
    FICA como esta' e e' NOMEADO no handoff — e' a lista que a L0 tem de cobrir.

A3. AGRUPAR: shells/desktop/src/vec_*.rs → shells/desktop/src/vec/ (um `mod vec;`
    no main.rs no lugar de N linhas; os testes vao junto). O corte da Fase B passa a ser
    mover UMA pasta.

A4. A CRATE NASCE: crates/ph2d-app-vec com Cargo.toml + o que JA' so' depende de crates
    do modulo e do ECS (construtores de cena puros, leis) — a shell chama-os; a crate NAO
    depende da shell (ela e' um bin).

A5. Prova (abaixo) sobre a Fase A + HANDOFF INTERMEDIO com os numeros: LOC movidas,
    `impl App` de N para M, membros de N para M, cenas apagadas, e a lista do que so' o
    substrato resolve.

═══ FASE B — só depois de a line/app-host INTEGRAR (o Enio avisa) ═══

B1. git rebase main
B2. Ler INTEIRO o docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
B3. O corte: o resto da pasta vai para crates/ph2d-app-vec; a familia regista-se em
    ph2d-app-registry-init; o que e' genuinamente da shell passa pelo trait de
    ph2d-app-host. Prova outra vez + handoff final.
⛔ Se o HOWTO nao cobrir algo de que a familia precisa: PARE e reporte com a lista. A
extensao do substrato e' decisao do integrador, nunca de cinco linhas em paralelo.

═══ A PROVA (em CADA fase; os numeros vao no handoff) ═══
  a) Nenhum teste se perde: `cargo nextest list --workspace --cargo-profile ci-test`
     ANTES (agora, antes de mover) e DEPOIS, comparados por
     `python3 scripts/nextest-list-diff.py antes.txt depois.txt` — ONLY-A tem de ser 0.
     Os MOVED (mesmo nome, pacote novo) sao o esperado; os ONLY-B listam-se.
  b) Roteadores e niveis de smoke iguais antes/depois, menos os apagados de proposito em
     A1; os gates no_two_*_scenes_claim_the_same_level continuam a correr.
  c) A shell encolheu: LOC de shells/desktop/src e a unidade `ph2d-host-desktop
     bin (check-test)` num `cargo check -p ph2d-host-desktop --tests --timings -j 32`
     A FRIO, num target novo, antes/depois.
  d) Gate de fecho (DIRETRIZ §1.5.9). ⚠️ Armadilhas que a W1 pagou e uma extraccao
     repete: include_str! com caminho relativo muda de sitio quando o ficheiro se move;
     gates que varrem shells/desktop/src por nome de familia; file_loc_caps.rs lista
     caminhos; 588 citacoes de caminho nos docs foram reapontadas por script na W1 —
     faca o mesmo (fora de docs/archive; logs .txt intocados).
  e) O smoke do Enio, na sua worktree, com --profile smoke, sobre as MESMAS cenas de
     antes: comportamento identico — a extraccao NAO muda produto.

⚠️ ESPECIFICO DESTA FAMILIA: ela tem mais LOC de TESTE (17 933) que de produto, e o
`vec_entities` e' um dos SEIS modulos internos que as cenas de OUTRAS familias usam —
ele FICA na shell ate o substrato de L0 o expor (nao o leve na Fase A).
⚠️ O PH2D_BUILD_SMOKE e' partilhado com o fx e o instance: o censo de cenas orfas desta
familia so' conta os niveis que sao dela.

SMOKE A ENTREGAR (deixe-o COMPILADO, regra I) — o molde, com o roteador e o nivel que
esta familia de facto tem:
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec && env PH2D_<ROTEADOR>=<n> cargo run -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

## `line/app-flip` — L5 — a família flip (61 ficheiros, 16 674 LOC de produto)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/app-flip

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
   git worktree add -b line/app-flip Worktrees/line-app-flip main
      → a branch já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-app-flip line/app-flip
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-app-flip
   git branch --show-current        # DEVE imprimir line/app-flip
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina. Falhou por "mergiraf not found"?
        NÃO é bloqueio: git faz fallback. Reporte o ✗ e siga.
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/STACK_VERSOES.md       → tudo (1 página)
        Rust 1.98/edition 2024, wgpu 29, vello 0.10, parley 0.11,
        rapier2d 0.35, bevy_ecs 0.19. NUNCA escreva uma versão de
        memória: `bash scripts/stack-audit.sh --tetos` responde.
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6 (§6.7 inclusive)
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha pronta em Worktrees/line-app-flip." — e siga direto
   para A TAREFA abaixo: ela já está decidida, não espere mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-app-flip/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar shells/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do seu módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107). PARE e reporte ao Enio
   SÓ se: (a) for contrato congelado (CLAUDE.md §6, exige ADR), ou (b)
   o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO, ponto de extensão append-only, id/const/
   variant novo = próximo livre + ANOTE no handoff (regra H).
   DIRETRIZ §1.5.2.1.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5).
E. Fechamento = gate batched (nextest-impacted + clippy --all-targets
   + auditoria ≥2 lentes + DIRETIVA §3-§5). Então PARE — NÃO integre
   nem faça ship. Quem funde é um AGENTE INTEGRADOR DEDICADO, só por
   ORDEM EXPLÍCITA do Enio. Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados são intocáveis.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar, DIRETRIZ
   §1.5.9): branch/HEAD/base; foundational tocado + por quê; ids/
   consts/variants novos com valores; contratos encostados (deve ser
   nenhum); o que só o ship.sh pega; o que smoke-testar; e os números
   da PROVA abaixo. Reporte "linha pronta + handoff" e ESPERE.
I. DEIXE O SMOKE COMPILADO — último passo, depois do commit final e de
   `rm -rf target/*/incremental`:
   `cargo build -p ph2d-host-desktop --profile smoke`
   (⚠️ --profile smoke, NÃO --release: 3 s contra 161 s, medido 10/09;
   --release só para smoke de PERFORMANCE). Rode 2× e cole a 2ª saída
   no handoff ("Finished" em segundos, ZERO "Compiling") — é a prova.
═══════════════════════════════════════════════════════════════════
A TAREFA — W2/L5: a família `flip` sai da shell, em DUAS FASES

CONTEXTO (leia antes de tocar num ficheiro):
  docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md  → §0 a §3 e §5, INTEIRO
  docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md      → §4-C2 (o mecanismo e o censo)
  DIRETRIZ §6.7                                                          → as regras que a auditoria deixou

O PROBLEMA, medido: `shells/desktop` e' UMA crate de 493 k linhas que dobrou em quatro
semanas e e' a ultima unidade de todo build grande (34-45 s sozinha no fim do gate).
A sua familia la' dentro: 61 ficheiros, 16 674 LOC de produto, 19 ficheiros de cena, 0 `impl App`, 75 membros de self..

SEIS LINHAS ABREM HOJE. A line/app-host constroi o SUBSTRATO (crates ph2d-app-host +
ph2d-app-registry-init) e o molde (HOWTO_partir_uma_familia_da_shell.md), com o field3d
como piloto. Por isso o seu trabalho e' em duas fases — e a FASE A comeca JA':

═══ FASE A — desde já (só ficheiros da sua família; ZERO interface nova com a shell) ═══

A1. CENSO E PODA DAS CENAS. Cada cena de smoke do roteador desta familia que NENHUM doc
    cita pelo numero APAGA-SE (ordem do Enio, 10/09: uma cena que ninguem cita sao ~1 k
    linhas pagas por todos em cada build, por ninguem). O censo:
      grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE=[0-9]+' docs CLAUDE.md project-memory .claude \
        --include='*.md' | sort -u          # (fora de docs/archive)
    contra os niveis que o roteador da familia de facto oferece. Lista das apagadas no
    handoff, com roteador e nivel. ⛔ Uma cena citada como PENDENTE de smoke fica.

A2. DESFAZER O ACOPLAMENTO, dentro da familia. Cada `impl App` desta familia vira funcao
    livre sobre &mut World / recursos / APIs das crates do modulo; os campos de `App` que
    SO' esta familia le' saem de `App` para UM estado da familia (FlipState num recurso do
    ECS, ou uma unica struct num unico campo de `App` — ADR-0075). Numeros de partida no
    censo acima; os de chegada vao no handoff.
    ⛔ O que precisa da shell e NAO e' da familia (janela, gfx, paineis, captura de undo)
    FICA como esta' e e' NOMEADO no handoff — e' a lista que a L0 tem de cobrir.

A3. AGRUPAR: shells/desktop/src/flip_*.rs → shells/desktop/src/flip/ (um `mod flip;`
    no main.rs no lugar de N linhas; os testes vao junto). O corte da Fase B passa a ser
    mover UMA pasta.

A4. A CRATE NASCE: crates/ph2d-app-flip com Cargo.toml + o que JA' so' depende de crates
    do modulo e do ECS (construtores de cena puros, leis) — a shell chama-os; a crate NAO
    depende da shell (ela e' um bin).

A5. Prova (abaixo) sobre a Fase A + HANDOFF INTERMEDIO com os numeros: LOC movidas,
    `impl App` de N para M, membros de N para M, cenas apagadas, e a lista do que so' o
    substrato resolve.

═══ FASE B — só depois de a line/app-host INTEGRAR (o Enio avisa) ═══

B1. git rebase main
B2. Ler INTEIRO o docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
B3. O corte: o resto da pasta vai para crates/ph2d-app-flip; a familia regista-se em
    ph2d-app-registry-init; o que e' genuinamente da shell passa pelo trait de
    ph2d-app-host. Prova outra vez + handoff final.
⛔ Se o HOWTO nao cobrir algo de que a familia precisa: PARE e reporte com a lista. A
extensao do substrato e' decisao do integrador, nunca de cinco linhas em paralelo.

═══ A PROVA (em CADA fase; os numeros vao no handoff) ═══
  a) Nenhum teste se perde: `cargo nextest list --workspace --cargo-profile ci-test`
     ANTES (agora, antes de mover) e DEPOIS, comparados por
     `python3 scripts/nextest-list-diff.py antes.txt depois.txt` — ONLY-A tem de ser 0.
     Os MOVED (mesmo nome, pacote novo) sao o esperado; os ONLY-B listam-se.
  b) Roteadores e niveis de smoke iguais antes/depois, menos os apagados de proposito em
     A1; os gates no_two_*_scenes_claim_the_same_level continuam a correr.
  c) A shell encolheu: LOC de shells/desktop/src e a unidade `ph2d-host-desktop
     bin (check-test)` num `cargo check -p ph2d-host-desktop --tests --timings -j 32`
     A FRIO, num target novo, antes/depois.
  d) Gate de fecho (DIRETRIZ §1.5.9). ⚠️ Armadilhas que a W1 pagou e uma extraccao
     repete: include_str! com caminho relativo muda de sitio quando o ficheiro se move;
     gates que varrem shells/desktop/src por nome de familia; file_loc_caps.rs lista
     caminhos; 588 citacoes de caminho nos docs foram reapontadas por script na W1 —
     faca o mesmo (fora de docs/archive; logs .txt intocados).
  e) O smoke do Enio, na sua worktree, com --profile smoke, sobre as MESMAS cenas de
     antes: comportamento identico — a extraccao NAO muda produto.

⚠️ ESPECIFICO DESTA FAMILIA: 0 `impl App` (funcoes soltas) — a Fase A e' quase toda
mover e agrupar. ⛔ O FlipDoc e' PARTILHADO (a F8 dos Componentes fechou-o em 10/09): e'
a ponte a NAO partir — se a crate nova precisar dele, ele vem de uma crate de modulo,
nunca da shell.

SMOKE A ENTREGAR (deixe-o COMPILADO, regra I) — o molde, com o roteador e o nivel que
esta familia de facto tem:
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_<ROTEADOR>=<n> cargo run -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```
