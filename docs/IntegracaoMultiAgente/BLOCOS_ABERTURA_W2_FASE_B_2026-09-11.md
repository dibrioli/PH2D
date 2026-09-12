# W2 — FASE B: os cinco blocos de reabertura

> **A Fase A fechou e integrou em 2026-09-11** (seis linhas, −61 704 LOC da shell). Isto são os
> blocos da **Fase B** — o corte de verdade, agora que o substrato (`ph2d-app-host` +
> `ph2d-app-registry-init` + `ph2d-app-sync`) está no `main`.
>
> ⭐ **O que a Fase B vale, medido em 11/09 depois da integração:** **214 097 LOC** — **49 %** da
> shell — ainda pertencem às seis famílias. A piloto (`field3d`) já fez as duas metades e ficou com
> **784**, o que prova que o método vai até ao fim.
>
> | família | pasta própria | `render_loop/` | **total ainda na shell** |
> |---|---:|---:|---:|
> | motion | 267 f / 60 782 | 143 f / 39 058 | **100 446** |
> | vec | ⚠️ 121 f soltos, **sem pasta** | 6 f / 2 215 | **35 671** |
> | sculpt3d | 111 f / 31 811 | 1 f / 91 | **31 902** |
> | physics | 39 f / 9 973 | 43 f / 13 642 | **23 615** |
> | flip | 65 f / 17 931 | 15 f / 3 748 | **21 679** |
> | *field3d (piloto, feita)* | — | **0** | *784* |
>
> ⭐⭐ **O `render_loop` NÃO é «metade-shell por natureza»** — a piloto tirou de lá **tudo**. O
> handoff da física supôs o contrário, e a medição desmente-o: são **58 754 LOC** das cinco famílias
> lá dentro, o maior bloco isolado da Fase B. O método é o do HOWTO §4: *o laço fica na shell (é ele
> que garante a ordem) e os corpos saem* — a shell passa a chamar `ph2d_app_<fam>::<mod>::<fn>`
> pelo nome, no ponto certo.

---

## §0 — A ORDEM, e porquê ela não é «as cinco de uma vez»

⛔⛔ **A `physics` abre PRIMEIRO e sozinha, como batedora.** Não é cautela: é a pergunta do
[HOWTO §5.3](HOWTO_partir_uma_familia_da_shell.md), que está **em aberto** e cuja resposta é uma só
para as cinco — *o trait `AppHost` tem cinco métodos e cobriu a piloto inteira; a `physics` toca
**126** membros de `App` em 22 `impl App`*. Se cinco linhas baterem nessa parede ao mesmo tempo,
cada uma inventa a extensão dela, e isso é **colisão de mesmo-símbolo em foundational** — o único
caso que a DIRETRIZ §1.5.5 manda parar e reportar.

⇒ a `physics` corre até bater (ou não) na parede e **reporta**; o integrador decide o substrato
**uma vez**; as outras quatro abrem a seguir, já com a resposta. Se ela não bater, abrem na mesma.

---

## §1 — REGRAS DA FASE B que valem para as CINCO

Estas entram em **todos** os blocos. Não são repetição: são o que a Fase A e a integração pagaram.

1. ⛔⛔ **NÃO toque no `TETO_LOC`** do gate `the_shell_only_shrinks`
   (`crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs`). Ele é **um número
   que soma entre linhas**, e a lei do `CLAUDE.md` §5.0 é que ele se **CONTA, nunca se escolhe** —
   com cinco linhas a escrever cinco números, o merge resolve para *um* deles e **nenhum está
   certo**, em silêncio. Quem o conta é o integrador, no fim, com a árvore junta. ⚠️ O gate vai
   **reprovar na sua worktree** assim que você tirar código (metade da obsolescência: *«a catraca já
   não descreve a árvore»*) — **isso é esperado e é o seu marcador de progresso**; escreva o número
   medido no handoff, não no ficheiro.
2. ✅ **RESPONDIDO PELA BATEDORA (11/09): as 5 portas do `AppHost` CHEGAM — zero sextos métodos, e
   nenhum é pedido.** A `line/app-physics` correu a família que mais toca a `App` (126 membros, 22
   `impl App`) e não precisou de estender o substrato.
   ⭐⭐ **E o achado útil não é o «sim» — é o que PARECIA precisar da `App` e não precisava:**
   - **três folhas residentes na shell, partilhadas entre famílias** (`inspector_ordering`,
     `preview_drive`, `name_unique` — 11 / 38 / 14 consumidores de famílias diferentes). São
     **puras**; ficaram na shell por PARTILHA, não por acoplamento. ⇒ se a sua família tropeçar
     numa destas, **não é porta e não é sua**: reporte, é linha própria;
   - **os três gestos de corpo** (`body_fk`/`body_grab`/`body_pose`, que eram `impl App`) queriam
     **três tipos** que a `App` por acaso segurava — viraram funções livres.
   ⇒ ⭐ **Antes de pedir porta, escreva o que a função PRECISA em tipos.** Se a resposta é *«três
   coisas que a `App` segura»*, não é porta — **é assinatura**.
   ⛔ **Ainda assim: se depois disto precisar mesmo de um SEXTO método — PARE e reporte com a lista.** O HOWTO §5.3 e a
   §1.5 dizem-no: *se a sua família precisa de um método por campo da `App` que hoje toca, ela não
   precisa de um trait maior — precisa de tirar o campo da `App`.* Estender o substrato é decisão do
   integrador, nunca de cinco linhas em paralelo.
3. ⭐ **O FIM DA SUA LINHA É GATEADO, e é uma frase só:** a sua família sai da catraca
   `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` (em `ph2d-app-registry-init/src/lib.rs`) porque o
   `const FAMILY` da sua crate passou a declarar os roteadores que ela **de facto lê**. O gate
   `every_registered_family_declares_a_reachable_router` reprova nos dois sentidos, então não há
   como dizer que acabou sem ter acabado. ⚠️ **Tire só a SUA linha da lista** — as outras quatro
   ainda lá estarão.
4. ⚠️ **`max_level` CONTA-SE no roteador, nunca se escreve de memória** (`CLAUDE.md` §5.0). Um
   roteador que é interruptor (`var_os(..).is_some()`) tem `max_level: 1`; um que responde por uma
   faixa tem o maior nível que o `match` de facto responde.
5. ⛔ **Não edite a árvore de outra linha.** Quando um módulo seu é consumido por `<outra>_*`, deixe
   um **alias de uma linha** na shell com a data de validade escrita (HOWTO §4). Um alias com prazo
   é dívida nomeada; um sem prazo é uma camada.
6. **A PROVA, sempre:** `cargo nextest list --workspace --cargo-profile ci-test` **antes** de mover
   (na base) e depois, comparados por `python3 scripts/nextest-list-diff.py antes.txt depois.txt`.
   `ONLY-A` vazio ou a linha não fechou.
7. ⚠️ **Leia o [HOWTO](HOWTO_partir_uma_familia_da_shell.md) INTEIRO antes de mover o primeiro
   ficheiro** — a §2 tem **11** armadilhas medidas e **quatro têm modo de falha MUDO** (uma feature
   não viaja com o código · `#[cfg(test)]` é invisível do outro lado da crate · um censo que varre
   por prefixo passa a varrer zero e fica verde · uma fronteira põe um elo novo na corrente e ele não
   tem gate).

---

## §2 — O BLOCO: `line/app-physics` (a BATEDORA — abre primeiro, sozinha)

```
═══════════════════════════════════════════════════════════════════
REABERTURA DE LINHA — Modo L · W2 FASE B      (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-physics.  A linha JÁ EXISTE: a Fase A
dela fechou e FOI INTEGRADA ao main em 11/09. Isto é a FASE B — o corte.

⭐ VOCÊ É A BATEDORA. Abre sozinha, e a razão está no §0 do
   docs/IntegracaoMultiAgente/BLOCOS_ABERTURA_W2_FASE_B_2026-09-11.md:
   a sua família toca 126 membros de `App` em 22 `impl App`, e a pergunta
   do HOWTO §5.3 (o trait de host chega?) tem de ser respondida UMA vez,
   por você, antes de as outras quatro abrirem.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-physics
   pwd && git branch --show-current     # DEVE dizer line/app-physics
2. git rebase main                      # o main andou MUITO em 11/09
      → conflito em Cargo.lock ou em registry-init: NUNCA na mão —
        `git checkout main -- Cargo.lock` + `cargo metadata` + `git add`.
      → conflito em código FORA dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-physics      # warm-up; o 1º build é frio
4. LEIA INTEIRO, nesta ordem:
   a) docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md
      — é a LEI desta wave. A §2 tem 11 armadilhas medidas, 4 MUDAS.
   b) docs/Physics/handoffs/HANDOFF_INTEGRACAO_line_app-physics_FASE_A_2026-09-11.md
      — o §14 é o SEU plano (escrito por você mesma) e o §6 é a lista
        dos 7 prendedores. ⚠️ Um deles já está resolvido: o gesto do
        PONTEIRO é coberto pelas 5 portas do `AppHost` (pointer, mods,
        pointer_over_chrome, modal_takes_the_pointer,
        note_authored_change). E o §6 supõe que os 64 ficheiros de
        render_loop são «metade-shell por natureza» — ISSO ESTÁ
        REFUTADO: a piloto tirou de lá TUDO (0 ficheiros), e são
        43 f / 13 642 LOC do seu total.
   c) §1 do BLOCOS_ABERTURA_W2_FASE_B_2026-09-11.md (as 7 regras da
      Fase B) — em especial a nº 1 (NÃO toque no TETO_LOC) e a nº 2.
5. Reporte "Fase B da physics pronta para começar" e SIGA (não pare).

A TAREFA — o alvo, medido em 11/09:
  · shells/desktop/src/physics/        39 ficheiros /  9 973 LOC
  · shells/desktop/src/render_loop/…   43 ficheiros / 13 642 LOC
  ────────────────────────────────────────────────────────────────
    total a sair                       82 ficheiros / 23 615 LOC

  O método é o do HOWTO. Para o render_loop, o da §4 (e é o da piloto):
  o LAÇO fica na shell — é ele que garante a ordem dos 48 símbolos —
  e o que sai são os CORPOS; a shell passa a chamar
  `ph2d_app_physics::<mod>::<fn>` no ponto certo. ⛔ NÃO abstraia o laço.

  O FIM DA LINHA (gateado, não é opinião):
  · o `PH2D_PHYSICS_SMOKE` passa a ser lido DENTRO da crate;
  · o `const FAMILY` da ph2d-app-physics declara esse roteador, com o
    `max_level` CONTADO no `match` (hoje é `routers: &[]`);
  · "physics" SAI da catraca FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL
    (só a sua linha; as outras quatro ficam);
  · `cargo test -p ph2d-app-registry-init` verde.

  ⚠️ E O QUE VOCÊ TEM DE RESPONDER PARA AS OUTRAS QUATRO:
  as 5 portas do `AppHost` chegaram? Se em algum ponto você precisar de
  uma SEXTA, **PARE e reporte com a lista** — não invente. É a regra 2.

REGRAS PERMANENTES (valem até o fim):
A. Tudo acontece DENTRO de Worktrees/line-app-physics/. O mesmo path
   relativo existe na raiz: editar lá é editar a árvore ERRADA. `pwd`
   na dúvida.
C. Commits locais frequentes, `git commit --no-verify`. NUNCA push,
   NUNCA --force, NUNCA `git add -A`.
E. Fechar = gate batched 1× (nextest-impacted + clippy --all-targets +
   auditoria ≥2 lentes) e PARE. Você NÃO integra e NÃO roda o
   foundational-integrate.sh — isso é do integrador, por ordem do Enio.
H. HANDOFF de integração obrigatório (DIRETRIZ §1.5.9), com: a prova do
   nextest-list-diff (ONLY-A vazio), o LOC medido da shell depois do
   seu corte (para o integrador contar o TETO_LOC), e a resposta sobre
   o trait de host.
I. DEIXE O SMOKE COMPILADO na sua worktree, 2 corridas, a 2ª colada no
   handoff (Finished em segundos, ZERO Compiling):
   cargo build -p ph2d-host-desktop --profile smoke
═══════════════════════════════════════════════════════════════════
```

---

## §3 — O BLOCO: `line/app-motion` (a maior — 100 446 LOC)

```
═══════════════════════════════════════════════════════════════════
REABERTURA DE LINHA — Modo L · W2 FASE B      (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-motion.  A linha JÁ EXISTE: a Fase A
dela fechou e FOI INTEGRADA ao main em 11/09. Isto é a FASE B — o corte.

⭐ VOCÊ É A MAIOR DAS CINCO, e por uma razão que é elogio à Fase A: ela
   tirou só 491 LOC e tocou 436 ficheiros porque gastou o dia a
   DESPRENDER a família da `App` (10 `impl crate::App` viraram funções
   livres, 4 campos soltos viraram uma struct, ~130 `motion_*.rs`
   viraram a pasta `src/motion/`). O corte é agora, e é mecânico.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-motion
   pwd && git branch --show-current      # DEVE dizer line/app-motion
2. git rebase main                       # o main andou MUITO em 11/09
      → Cargo.lock / registry-init: NUNCA na mão (regenere).
      → código fora dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-motion        # warm-up
4. LEIA INTEIRO: (a) o HOWTO_partir_uma_familia_da_shell.md — a LEI
   desta wave, §2 com 11 armadilhas, 4 MUDAS; (b) o seu handoff
   docs/Motion Nodes/handoffs/HANDOFF_INTEGRACAO_line_app_motion_FASE_A_2026-09-11.md;
   (c) o §1 do BLOCOS_ABERTURA_W2_FASE_B_2026-09-11.md (as 7 regras).
5. Reporte "Fase B da motion pronta" e SIGA.

A TAREFA — o alvo, medido em 11/09:
  · shells/desktop/src/motion/        267 ficheiros / 60 782 LOC
  · shells/desktop/src/render_loop/…  143 ficheiros / 39 058 LOC
  ────────────────────────────────────────────────────────────────
    total a sair                      410 ficheiros / 100 446 LOC

  ⚠️ É o MAIOR bloco de `render_loop` das cinco (39 058 LOC). O método é
  o do HOWTO §4 e é o da piloto: o LAÇO fica na shell (é ele que garante
  a ordem dos 48 símbolos) e saem os CORPOS — a shell chama
  `ph2d_app_motion::<mod>::<fn>` no ponto certo. ⛔ NÃO abstraia o laço.

  ⚠️ Vá por FATIAS que compilam, e commite cada uma. 410 ficheiros num
  commit é um passo que não se bissecta — e a §1 do HOWTO manda o
  contrário por escrito.

  O FIM DA LINHA (gateado):
  · os roteadores que hoje a shell lê pela sua família passam para
    dentro da crate. São 12 — CONTE-OS você, não copie desta lista:
    PH2D_GPU_COOK_DEMO · PH2D_MOTION_OBJ_SMOKE · PH2D_MOTION_NODE_PATH_SMOKE
    · PH2D_MOTION_DELAY_SMOKE · PH2D_MOTION_FX_SMOKE · PH2D_PATH_SMOKE
    · PH2D_SHAPE_SMOKE · PH2D_AUTOFIX_SMOKE · (+ PH2D_GPU_COOK,
    PH2D_LADO, PH2D_LAYOUT_LEVEL, PH2D_DROPS_SCAN_MAX, que são
    DIAGNÓSTICO e NÃO entram no `FAMILY` — a forma é `PH2D_*_SMOKE`);
  · o `const FAMILY` da ph2d-app-motion declara-os, com cada
    `max_level` CONTADO no `match` do roteador (hoje é `routers: &[]`);
  · "motion" SAI da catraca FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL
    (só a sua linha);
  · `cargo test -p ph2d-app-registry-init` verde.

REGRAS PERMANENTES: A (tudo dentro da SUA worktree; `pwd` na dúvida) ·
C (commits locais, --no-verify; NUNCA push/--force/`git add -A`) ·
E (fechar = gate batched 1×, e PARE: você não integra) ·
H (handoff obrigatório, com a prova do nextest-list-diff, o LOC medido
   da shell depois do corte, e os roteadores declarados) ·
I (smoke compilado na sua worktree, 2 corridas, a 2ª no handoff:
   cargo build -p ph2d-host-desktop --profile smoke).
⛔ NÃO toque no TETO_LOC do the_shell_only_shrinks (regra 1 do §1).
⛔ Precisa de um 6º método no AppHost? PARE e reporte (regra 2).
═══════════════════════════════════════════════════════════════════
```

---

## §4 — O BLOCO: `line/app-sculpt3d` (e ela desbloqueia a piloto)

```
═══════════════════════════════════════════════════════════════════
REABERTURA DE LINHA — Modo L · W2 FASE B      (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-sculpt3d.  A linha JÁ EXISTE: a Fase A
dela fechou e FOI INTEGRADA ao main em 11/09. Isto é a FASE B — o corte.

⭐ E VOCÊ DESBLOQUEIA A PILOTO: os CINCO alias que a `line/app-host`
   deixou na shell (field3d_views, _navball, _layout, _view_menu,
   _gizmo — 15 linhas cada) existem SÓ porque `sculpt3d_*` os consome.
   Quando a sua linha fechar, eles SOMEM e os chamadores passam a
   escrever `ph2d_viewport3d::…`. Está escrito no HOWTO §5.1 — é dívida
   com o seu nome nela.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-sculpt3d
   pwd && git branch --show-current     # DEVE dizer line/app-sculpt3d
2. git rebase main                      # o main andou MUITO em 11/09
      → Cargo.lock / registry-init: NUNCA na mão (regenere).
      → código fora dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-sculpt3d     # warm-up
4. LEIA INTEIRO: (a) o HOWTO — a LEI, §2 com 11 armadilhas, 4 MUDAS, e
   a §5.1 que é sua; (b) o seu handoff
   docs/3D/handoffs/HANDOFF_INTEGRACAO_line_app_sculpt3d_FASE_A_2026-09-11.md;
   (c) o §1 do BLOCOS_ABERTURA_W2_FASE_B_2026-09-11.md (as 7 regras).
5. Reporte "Fase B da sculpt3d pronta" e SIGA.

A TAREFA — o alvo, medido em 11/09:
  · shells/desktop/src/sculpt3d/      111 ficheiros / 31 811 LOC
  · shells/desktop/src/render_loop/…    1 ficheiro  /     91 LOC
  ────────────────────────────────────────────────────────────────
    total a sair                      112 ficheiros / 31 902 LOC
  + os 5 alias de `field3d_*` na shell, APAGADOS (HOWTO §5.1)

  ⭐ A sua Fase A já fez o trabalho de ENDEREÇO (os ~180 `sculpt3d_*.rs`
  viraram a pasta `src/sculpt3d/`) e o seu `render_loop` são 91 linhas.
  É a família com o corte mais limpo das cinco.

  O FIM DA LINHA (gateado):
  · o PH2D_SCULPT3D_SMOKE passa a ser lido DENTRO da crate (hoje está
    em shells/desktop/src/sculpt3d/scenes_*.rs);
  · ⚠️ a sua família lê ~32 `PH2D_*` na shell e a ESMAGADORA MAIORIA é
    DIAGNÓSTICO de retopologia (PH2D_RETOPO_*, PH2D_DUMP*, PH2D_BENCH_*,
    PH2D_TIP_ALIGN…). Só o que tem a forma `PH2D_*_SMOKE` entra no
    `FAMILY` — CONTE, não copie;
  · o `const FAMILY` declara o(s) roteador(es) com `max_level` CONTADO
    no `match` (hoje é `routers: &[]`);
  · "sculpt3d" SAI da catraca FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL;
  · `cargo test -p ph2d-app-registry-init` verde.

REGRAS PERMANENTES: A (tudo dentro da SUA worktree; `pwd` na dúvida) ·
C (commits locais, --no-verify; NUNCA push/--force/`git add -A`) ·
E (fechar = gate batched 1×, e PARE: você não integra) ·
H (handoff obrigatório, com a prova do nextest-list-diff, o LOC medido
   da shell depois do corte, e a confirmação de que os 5 alias sumiram) ·
I (smoke compilado na sua worktree, 2 corridas, a 2ª no handoff:
   cargo build -p ph2d-host-desktop --profile smoke).
⛔ NÃO toque no TETO_LOC do the_shell_only_shrinks (regra 1 do §1).
⛔ Precisa de um 6º método no AppHost? PARE e reporte (regra 2).
═══════════════════════════════════════════════════════════════════
```

---

## §5 — O BLOCO: `line/app-vec` (a única que ainda não tem PASTA)

```
═══════════════════════════════════════════════════════════════════
REABERTURA DE LINHA — Modo L · W2 FASE B      (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-vec.  A linha JÁ EXISTE: a Fase A dela
fechou e FOI INTEGRADA ao main em 11/09. Isto é a FASE B — o corte.

⚠️ VOCÊ COMEÇA UM PASSO ATRÁS DAS OUTRAS, e sabê-lo poupa-lhe o dia: as
   outras quatro agruparam a família numa PASTA na Fase A; a sua não —
   os seus 121 ficheiros `vec_*.rs` continuam SOLTOS no topo de
   shells/desktop/src/. O seu passo 1 é o agrupamento
   (src/vec_x.rs → src/vec/x.rs), que é o que torna o corte mecânico.

⚠️ E a razão de a sua Fase A ter levado só 8 ficheiros está MEDIDA no
   `lib.rs` da sua crate: `#[path]` é aresta DURA nos dois sentidos —
   `vec_gizmo_view.rs` DECLARA `#[path = "vec_gizmo_pick.rs"]`, e esse
   toca `App`, logo o pai não podia sair. Um `mod` declarado por
   `#[path]` é parte da árvore de módulos do pai, não uma referência
   que se re-aponta. LEIA aquele doc-comment antes de planear.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec
   pwd && git branch --show-current      # DEVE dizer line/app-vec
2. git rebase main                       # o main andou MUITO em 11/09
      → Cargo.lock / registry-init: NUNCA na mão (regenere).
      → código fora dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-vec           # warm-up
4. LEIA INTEIRO: (a) o HOWTO — a LEI, §2 com 11 armadilhas, 4 MUDAS;
   (b) o seu handoff
   docs/Vector Module/handoffs/HANDOFF_INTEGRACAO_line_app_vec_FASE_A_2026-09-11.md;
   (c) o doc-comment de crates/ph2d-app-vec/src/lib.rs (a tabela das 4
   réguas e o `#[path]`); (d) o §1 do BLOCOS_ABERTURA_W2_FASE_B (7 regras).
5. Reporte "Fase B da vec pronta" e SIGA.

A TAREFA — o alvo, medido em 11/09:
  · shells/desktop/src/vec_*.rs       121 ficheiros / ~33 456 LOC
  · shells/desktop/src/render_loop/…    6 ficheiros /   2 215 LOC
  ────────────────────────────────────────────────────────────────
    total a sair                      127 ficheiros /  35 671 LOC

  Passo 1: AGRUPAR em src/vec/ (o que as outras fizeram na Fase A).
  Passo 2: o corte, pelo HOWTO. Para o render_loop, a §4: o LAÇO fica
  na shell e saem os CORPOS (`ph2d_app_vec::<mod>::<fn>`).

  O FIM DA LINHA (gateado):
  · os roteadores que a shell lê pela sua família passam para dentro da
    crate — PH2D_VEC_BONE_SMOKE · PH2D_VEC_STACK_SMOKE ·
    PH2D_VEC_APPEARANCE_SMOKE · PH2D_VEC_FADE_SMOKE (+ PH2D_BUILD_SMOKE
    e PH2D_UI_MOTION_SMOKE, se forem seus — CONTE, não copie; e
    PH2D_BLEND_LOG / PH2D_TEXT_LOG são DIAGNÓSTICO e não entram);
  · o `const FAMILY` declara-os com `max_level` CONTADO (hoje é
    `routers: &[]`, e o doc-comment dele explica porquê — actualize-o);
  · "vec" SAI da catraca FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL;
  · `cargo test -p ph2d-app-registry-init` verde.

REGRAS PERMANENTES: A (tudo dentro da SUA worktree; `pwd` na dúvida) ·
C (commits locais, --no-verify; NUNCA push/--force/`git add -A`) ·
E (fechar = gate batched 1×, e PARE: você não integra) ·
H (handoff obrigatório, com a prova do nextest-list-diff e o LOC medido
   da shell depois do corte) ·
I (smoke compilado na sua worktree, 2 corridas, a 2ª no handoff:
   cargo build -p ph2d-host-desktop --profile smoke).
⛔ NÃO toque no TETO_LOC do the_shell_only_shrinks (regra 1 do §1).
⛔ Precisa de um 6º método no AppHost? PARE e reporte (regra 2).
═══════════════════════════════════════════════════════════════════
```

---

## §6 — O BLOCO: `line/app-flip` (a única que já declara roteadores)

```
═══════════════════════════════════════════════════════════════════
REABERTURA DE LINHA — Modo L · W2 FASE B      (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é o agente da linha  line/app-flip.  A linha JÁ EXISTE: a Fase A dela
fechou e FOI INTEGRADA ao main em 11/09. Isto é a FASE B — o corte.

⭐ VOCÊ É A ÚNICA DAS CINCO QUE JÁ DECLARA ROTEADORES: o `const FAMILY`
   da ph2d-app-flip tem 15, e por isso a `flip` NÃO está na catraca
   FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL. A dívida nomeada no
   doc-comment dele são TRÊS que ficaram na shell — e fechá-la é parte
   desta wave: a lista passa a 18.

SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip
   pwd && git branch --show-current      # DEVE dizer line/app-flip
2. git rebase main                       # o main andou MUITO em 11/09
      → Cargo.lock / registry-init: NUNCA na mão (regenere).
      → código fora dos seus ficheiros: PARE e reporte.
3. cargo check -p ph2d-app-flip          # warm-up
4. LEIA INTEIRO: (a) o HOWTO — a LEI, §2 com 11 armadilhas, 4 MUDAS;
   (b) o seu handoff
   docs/Flip/handoffs/HANDOFF_INTERMEDIO_line_app-flip_FASE_A_2026-09-11.md;
   (c) o doc-comment do `const FAMILY` em crates/ph2d-app-flip/src/lib.rs
   (a dívida dos três); (d) o §1 do BLOCOS_ABERTURA_W2_FASE_B (7 regras).
5. Reporte "Fase B da flip pronta" e SIGA.

A TAREFA — o alvo, medido em 11/09:
  · shells/desktop/src/flip/           65 ficheiros / 17 931 LOC
  · shells/desktop/src/render_loop/…   15 ficheiros /  3 748 LOC
  ────────────────────────────────────────────────────────────────
    total a sair                       80 ficheiros / 21 679 LOC

  O método é o do HOWTO. Para o render_loop, a §4: o LAÇO fica na shell
  (garante a ordem) e saem os CORPOS — `ph2d_app_flip::<mod>::<fn>`.

  O FIM DA LINHA (gateado):
  · os TRÊS roteadores que ainda vivem na shell passam para a crate:
    PH2D_FLIP_HARDNESS_SMOKE (o mestre) · PH2D_FLIP_PRESSURE_SMOKE ·
    PH2D_FLIP_RESAMPLE_SMOKE  (PH2D_FLIP_FILL_DEBUG e
    PH2D_FLIP_SELECT_DEBUG são DIAGNÓSTICO e NÃO entram no `FAMILY`);
  · o `const FAMILY` passa de 15 para 18 roteadores, com cada
    `max_level` CONTADO — os 15 que lá estão são interruptores
    (`var_os(..).is_some()` ⇒ 1); CONFIRA se estes três também são, ou
    se algum responde por uma faixa;
  · ⚠️ APAGUE do doc-comment do FAMILY a nota «⏳ o que ainda NÃO está
    aqui» quando ela deixar de descrever a árvore — uma dívida cumprida
    e não apagada lê-se como dívida aberta para sempre;
  · `cargo test -p ph2d-app-registry-init` verde
    (`no_two_families_claim_the_same_router` é o que morde aqui).

REGRAS PERMANENTES: A (tudo dentro da SUA worktree; `pwd` na dúvida) ·
C (commits locais, --no-verify; NUNCA push/--force/`git add -A`) ·
E (fechar = gate batched 1×, e PARE: você não integra) ·
H (handoff obrigatório, com a prova do nextest-list-diff e o LOC medido
   da shell depois do corte) ·
I (smoke compilado na sua worktree, 2 corridas, a 2ª no handoff:
   cargo build -p ph2d-host-desktop --profile smoke).
⛔ NÃO toque no TETO_LOC do the_shell_only_shrinks (regra 1 do §1).
⛔ Precisa de um 6º método no AppHost? PARE e reporte (regra 2).
═══════════════════════════════════════════════════════════════════
```

---

## §7 — Notas para o integrador (o dia da fusão)

1. ⭐ **O `TETO_LOC` conta-se UMA vez, no fim, com a árvore junta.** Nenhuma linha lhe toca (§1,
   regra 1). Depois da última fusão: mede-se `shells/desktop/**/*.rs`, escreve-se
   `medido + FOLGA_DE_COMPOSICAO`, e o gate volta a verde. ⚠️ O gate **vai estar vermelho** em cada
   worktree durante toda a wave — é a metade da obsolescência a funcionar, não um defeito.
2. **A catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` encolhe uma linha por família.** Cinco
   remoções da mesma lista: fundem limpo por serem linhas distintas, mas **confira a lista final
   contra as famílias registadas** — o censo de obsolescência do gate já o faz nos dois sentidos.
3. **A ordem de fusão mede-se** (`collision-surface.sh` em cada worktree, imediatamente antes). Na
   Fase A o critério que funcionou foi *churn de costura decrescente*: quem reescreve mais território
   partilhado entra primeiro, quem menos reescreve rebaseia por último.
4. ⚠️ **O `render_loop/mod.rs` é a costura desta wave** (14 058 linhas, e as cinco tiram corpos de
   lá). Espere conflitos e resolva-os por **união** — na Fase A os 8 conflitos de código foram todos
   duas linhas a melhorar o mesmo sítio por razões diferentes.
5. **Um gerador novo entra no passo 2 do `foundational-integrate.sh` no MESMO commit** (DIRETRIZ
   §6.7 item 9) — a Fase A pagou isso.
