# Briefings — W2: partir a shell (2026-09-11)

> **Ordem do Enio (10/09, noite): «Amanhã partiremos o shell.»** Este doc é a fonte única dos briefings
> das linhas dessa obra. A base medida está na
> [auditoria de velocidade](../DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md) §4-C2 (o
> mecanismo e o censo W2.0) e na DIRETRIZ §6.7 item 5. Cada linha abre com o bloco do
> [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md) (`/pd-linha-abrir`) e recebe a tarefa daqui.

## §0 — Por que em linhas, e por que NÃO todas de uma vez

**O problema medido.** `shells/desktop` é UMA crate de **493 k linhas** (307 k de produto + 186 k de testes no
`src/`), que dobrou em quatro semanas e é a última unidade de todo build grande (34–45 s sozinha no fim do
gate de fecho; 16,8 s de front-end monotarefa a frio). Dentro dela vivem **famílias inteiras de módulos** que
só ali estão por inércia — 425 ficheiros de cenas de smoke (95 k LOC) e a ponte de cada módulo com a `App`
(`app_state.rs`, **~401 campos públicos**).

**Por que linhas.** As famílias são disjuntas em ficheiros (`motion_*`, `physics_*`, `sculpt3d_*`, `vec_*`,
`flip_*`, `field3d_*`) e cada uma é dias de trabalho: são paralelizáveis, e o Modo L existe para isso.

**Por que as seis NÃO fazem o CORTE de uma vez.** Todas tocam as MESMAS costuras partilhadas: a lista de
`mod` no `main.rs`, os campos de `App` em `app_state.rs`, o `render_loop/mod.rs` (812 KB), o
`input_dispatch.rs` (355 KB, toca 273 membros de `App`), o `Cargo.toml` da shell e os roteadores de smoke.
Seis linhas a mexer nisso ao mesmo tempo **sem um ponto de extensão** dão colisões de mesmo-símbolo na
integração — exactamente o caso em que a DIRETRIZ §1.5.5 manda a linha PARAR. ⇒ **Uma linha constrói o
substrato** (a interface que uma família usa para falar com a shell, e o registo *append-only* por onde ela
se liga — CLAUDE.md §0.2: *«ao CRIAR foundational novo, projete-o para isolamento»*), prova-o com **uma
família piloto** de ponta a ponta, e escreve o **molde**. **As outras cinco abrem no mesmo dia** (§1) e fazem
primeiro o que não precisa do substrato — que é a maior parte dos dias de cada família —, tocando as
costuras só por ADIÇÃO/REMOÇÃO nas regiões da sua família; o **corte** final é depois de o substrato
integrar, pelo molde.

**Janela aberta, medido em 11/09 de manhã:** as oito linhas vivas estão integradas e limpas (`ahead=0`,
zero ficheiros da shell pendentes). Mover ficheiros da shell hoje não colide com ninguém.

## §1 — A ordem: as SEIS abrem hoje, em duas FASES (decisão do Enio, 11/09)

> A 1.ª redacção deste doc abria L0 sozinha e as cinco famílias só depois de ela integrar. O Enio perguntou
> *«não podemos abrir todas em paralelo?»* — e pode-se, porque **a maior parte do trabalho de uma família é
> dentro dos ficheiros dela** e não precisa do substrato: apagar cenas não citadas, desfazer o `impl App`,
> agrupar a família numa pasta, preparar a crate. O que NÃO pode acontecer é cinco linhas inventarem cinco
> interfaces com a shell. Daí as duas fases.

| linha | módulo | abre | **Fase A** (desde já, sem o substrato) | **Fase B** (depois de L0 integrar + `git rebase main`) |
|---|---|---|---|---|
| **L0** `line/app-host` | substrato + piloto `field3d` + HOWTO | hoje | §4 inteiro | — (é ela que integra primeiro) |
| L1 `line/app-motion` | motion | hoje | §5-A | §5-B: o corte pelo HOWTO |
| L2 `line/app-physics` | physics | hoje | §5-A | §5-B |
| L3 `line/app-sculpt3d` | sculpt3d | hoje | §5-A | §5-B |
| L4 `line/app-vec` | vec | hoje | §5-A | §5-B |
| L5 `line/app-flip` | flip | hoje | §5-A | §5-B |
| (3.ª rodada) | `fx` · `instance` · `painter` · `inspector` · `ui` · `project` | depois | absorvidos pela família dona, ou uma linha curta | |

**Ordem de integração:** L0 **primeiro** (é o substrato); as cinco depois, na ordem que o `collision-surface.sh`
medir. ⛔ **Nunca duas linhas na mesma família.** ⛔ Uma família que na Fase A precise de algo que só o
substrato pode dar **PÁRA nessa parte e avança nas outras** — não desenha a porta (regra B do MODELO: nunca
negocie com outra linha; a porta é da L0 e uma extensão dela é decisão do integrador).

## §2 — O censo por família (medido em `shells/desktop/src`, 11/09)

| família | ficheiros | LOC produto | ficheiros de smoke | LOC de smoke | LOC de teste | `impl App` | membros de `self.` tocados | roteadores |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| motion | 232 | 56 460 | 141 | 31 887 | 43 892 | 0 | 59 | `PH2D_MOTION_*`, `PH2D_GPU_COOK_DEMO`, `PH2D_AUTOFIX_SMOKE`, `PH2D_LENS_SMOKE`, … |
| sculpt3d | 83 | 23 078 | 17 | 2 503 | 8 260 | 6 | 180 | `PH2D_SCULPT3D_SMOKE` |
| physics | 94 | 22 783 | 80 | 18 608 | 15 479 | 22 | 126 | `PH2D_PHYSICS_SMOKE` |
| vec | 68 | 17 527 | 4 | 922 | 17 933 | 3 | 89 | `PH2D_BUILD_SMOKE`, `PH2D_VEC_*_SMOKE` |
| flip | 61 | 16 674 | 19 | 3 842 | 10 103 | 0 | 75 | `PH2D_FLIP_*_SMOKE` |
| **field3d (piloto)** | 53 | 15 207 | 17 | 6 558 | 16 990 | **1** | 33 | `PH2D_FIELD_SMOKE`, `PH2D_MODEL3D_SMOKE` |
| (fica) `render_loop/mod.rs` + `input_dispatch.rs` + `app_state.rs` | 3 | 14 153 | — | — | — | — | 326 | — |

Leitura: o **piloto é a família mais desacoplada** (1 `impl App`, 33 membros, e as 104 referências `crate::`
dos ficheiros `field3d_*` são quase todas para outros `field3d_*`). A **física** é a mais acoplada (22
`impl App`, 126 membros) e a **motion** a maior (232 ficheiros, mas 0 `impl App` — funções soltas).

## §3 — A prova comum (toda linha entrega estas cinco, com o número no handoff)

1. **Nenhum teste se perde.** `cargo nextest list --workspace --cargo-profile ci-test` ANTES (na base, antes de
   mover) e DEPOIS, comparados por `python3 scripts/nextest-list-diff.py antes.txt depois.txt` — `ONLY-A`
   tem de ser **0**; os `MOVED` (mesmo nome, pacote novo) são o esperado; os `ONLY-B` listam-se no handoff.
2. **Nenhuma cena de smoke se perde sem ser DE PROPÓSITO.** O conjunto de roteadores e níveis
   (`grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE' shells/desktop/src crates/ph2d-app-*/src | sort -u`) é igual antes e
   depois, e os gates `no_two_*_scenes_claim_the_same_level` continuam a correr (mudam de crate, não morrem).
   As cenas que **nenhum doc cita pelo número** (`grep -rhoE 'PH2D_<ROTEADOR>=[0-9]+' docs CLAUDE.md
   project-memory .claude`, fora de `docs/archive`) **apagam-se** — resposta do Enio em 10/09 a *«descartar
   depois de testar?»* — com a lista das apagadas no handoff.
3. **A shell encolheu, medido:** LOC de `shells/desktop/src` antes/depois, e a unidade `ph2d-host-desktop
   bin (check-test)` num `cargo check -p ph2d-host-desktop --tests --timings -j 32` **a frio, num target
   novo**, antes/depois (a auditoria mediu 16,8 s a frio, sem contenção).
4. **O gate de fecho da linha** (DIRETRIZ §1.5.9): `nextest-impacted` + clippy `--all-targets` + tetos de LOC
   + doc-index + typos. ⚠️ As armadilhas que a W1 pagou e que uma extracção repete: `include_str!` com
   caminho relativo muda de sítio quando o ficheiro se move; gates que varrem `shells/desktop/src` por
   nome de família; o `file_loc_caps.rs` lista caminhos; **588 citações de caminho** nos docs foram
   reapontadas por script na W1 — faça o mesmo (fora de `docs/archive`, logs `.txt` intocados).
5. **O smoke do Enio**, na worktree, com `--profile smoke`, sobre as MESMAS cenas de antes (mesmos números):
   o comportamento tem de ser idêntico — a extracção não muda produto.

## §4 — L0 `line/app-host` — o substrato + o piloto + o molde

**Objectivo.** Construir a porta por onde uma família sai da shell, tirar a família `field3d` por essa porta,
e escrever o HOWTO que L1–L5 seguem à letra.

**Entregáveis, por ordem:**

1. **`crates/ph2d-app-host`** — a INTERFACE (só traits/tipos, zero lógica): o que uma família precisa da
   shell e não pode obter de uma crate de módulo. O censo diz o que cobrir: as cenas de smoke usam **seis
   módulos internos da shell** (`instance_docs`, `image_import`, `instantiate`, `render_loop`, `vec_entities`,
   `audio`) e **22 ficheiros escrevem `impl App`** (no piloto, um: `field3d_input.rs`). ⚠️ **ADR-0075 primeiro:**
   estado de família vira **recurso/componente do ECS** e comunicação vira **evento/recurso** — o trait de
   host é o **fallback** para o que é genuinamente da shell (janela, gfx, painéis, captura de undo), não a
   primeira ferramenta. Uma família **não** deve precisar de um método por campo de `App` que hoje toca: se
   precisa, o campo é dela e sai da `App` com ela.
2. **`crates/ph2d-app-registry-init`** — o ponto de extensão *append-only*, pelo **precedente da casa**:
   `ph2d-panel-registry-init` (um `register_all_*()` chamado UMA vez em `init.rs`, bloco gerado por
   `cargo run -p ph2d-panel-sync`, com gate de *staleness*). ⚠️ Desenhe-o para que **cinco linhas o estendam
   sem colidir**: o bloco é gerado, não escrito à mão, e a lista sai de uma varredura (`crates/ph2d-app-*`)
   — a mesma razão por que o `Cargo.toml` da raiz tem membros por glob. `inventory`/`linkme` **não** estão
   no `Cargo.lock`; um pacote novo passa pelo `stack-audit.sh --tetos` e é decisão a registar.
3. **O piloto: `crates/ph2d-app-field3d`** — os 53 ficheiros de produto de `field3d_*` (15 207 LOC), as
   17 cenas de smoke (6 558) e os 16 990 LOC de testes deles, **com a prova do §3**. O que fica na shell:
   `main.rs`/`init.rs`, `App`, o esqueleto do laço de quadro, o `input_dispatch` (roteador). Um ficheiro da
   família que precise de `App` por dentro é o sinal de que falta um recurso/evento — resolva-o no substrato,
   não com um `pub` a mais na shell.
4. **`docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md`** — o molde para L1–L5: o que se
   move e para onde · como a família se regista · como se resolve cada um dos seis módulos internos e um
   `impl App` · os comandos da prova (§3) · as armadilhas que o piloto encontrou, com o número. ⚠️ Este doc
   é a metade mais valiosa da linha: *uma extracção que só o piloto sabe fazer não é um molde*.
5. **Handoff de integração** (DIRETRIZ §1.5.9) + UMA linha no CLAUDE.md §5 (a do módulo 3D Modeling) +
   uma linha no §5.0 se nascer lei nova.

**Regras específicas:**
- ⛔ **Não é uma feature `smokes` por `#[cfg]`** (recusa medida na auditoria §9): fora do `default` o gate e
  o CI deixariam de compilar as cenas em silêncio; no `default` ninguém compila sem ela. O que muda o
  tecto é a **crate**.
- ⛔ **Nada de `pub` a mais na shell para «facilitar»**: a shell é um `bin`; uma crate de família **não
  depende dela** — depende de `ph2d-app-host` e das crates do módulo. Se a família precisa de algo que só a
  shell tem, isso é um método do trait de host ou um recurso do ECS.
- **Testes:** os unitários movem-se com o código; os de integração da crate nova nascem em `tests/it/`
  (W1: um binário por crate). Os gates que vivem em `shells/desktop/src/field3d_*_tests.rs` movem-se ou são
  re-apontados — nenhum some (prova §3.1).
- **Medir antes de generalizar:** o piloto sozinho é UMA jornada. Se o substrato pedir mais (o piloto
  descobre que precisa de um mecanismo que as outras cinco famílias também vão precisar), é aqui que ele
  se constrói — não em cinco cópias.

**Smoke do Enio (na worktree):**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-host && env PH2D_FIELD_SMOKE=11 cargo run -p ph2d-host-desktop --profile smoke
```
(o pill **MODEL** e as cenas `=26..=32` do roteador do módulo — as mesmas de antes; o `field3d_smoke_scenes.rs`
diz quais existem).

## §5 — L1–L5 — uma família cada, pelo HOWTO

**Objectivo (o mesmo para as cinco, com os parâmetros da tabela do §2):** tirar a família `<FAM>` da shell
para `crates/ph2d-app-<FAM>`, em duas fases, com a prova do §3 em CADA fase (a lista de testes e os
roteadores são a rede: nada se perde sem ser de propósito) e o handoff no fim.

**§5-A — Fase A, desde já (só ficheiros da família; zero interface nova com a shell):**
1. **Censo de citações e poda:** cada cena de smoke do roteador da família que **nenhum doc cita pelo
   número** apaga-se (lista no handoff, com o roteador e o nível). ⚠️ Uma cena citada como *pendente de
   smoke* (a `=114` do Motion) fica.
2. **Desfazer o acoplamento, dentro da família:** cada `impl App` da família vira função livre sobre
   `&mut World`/recursos/APIs das crates do módulo; os campos de `App` que só a família lê **saem de `App`
   para UM estado da família** (um recurso do ECS, ou uma única struct `<Fam>State` num único campo de
   `App` — ADR-0075) — o censo do §2 dá o número de partida (`impl App` · membros) e o handoff dá o de
   chegada. ⛔ O que precisa da shell e não é da família (janela, gfx, painéis, captura de undo) **fica
   como está** e é NOMEADO no handoff — é a lista que a L0 tem de cobrir.
3. **Agrupar:** `shells/desktop/src/<fam>_*.rs` → `shells/desktop/src/<fam>/` (um `mod <fam>;` no
   `main.rs` no lugar de N linhas; os testes vão junto). O corte da Fase B passa a ser mover UMA pasta.
4. **A crate nasce:** `crates/ph2d-app-<FAM>` com `Cargo.toml` + o que **já** só depende de crates do
   módulo e do ECS (construtores de cena puros, leis) — a shell chama-os; a crate **não** depende da shell.
5. Prova do §3 (as cinco) sobre a Fase A e um **handoff intermédio** com os números: LOC movidas, `impl
   App` de N para M, membros de N para M, cenas apagadas, e a lista do que só o substrato resolve.

**§5-B — Fase B, depois de L0 integrar:** `git rebase main`, ler o `HOWTO_partir_uma_familia_da_shell.md`
inteiro, e fazer o corte: o resto da pasta vai para a crate, a família regista-se em `ph2d-app-registry-init`,
o que é genuinamente da shell passa pelo trait de `ph2d-app-host`. Prova do §3 outra vez, handoff final.
⛔ Se o HOWTO não cobrir algo de que a família precisa, **PARE e reporte** com a lista — a extensão do
substrato é decisão do integrador, não de cinco linhas em paralelo.

| linha | família | o que a torna diferente |
|---|---|---|
| L1 `line/app-motion` | motion (232 ficheiros, 56 k + 44 k de testes, 141 cenas) | a MAIOR: 0 `impl App`, mas 59 membros e os roteadores mais numerosos (`motion_state_demo_router.rs` conta as cenas — CLAUDE.md §5.0). ⚠️ O ciclo 5 do Motion tem a cena `=114` **não smokada** (§5): não a apague por não ser citada — ela está citada como pendente |
| L2 `line/app-physics` | physics (94, 22,8 k, 80 cenas) | a MAIS ACOPLADA: **22 `impl App`**, 126 membros. É a linha que mais vai pedir ao substrato; se o pedido for grande, PARE e reporte com a lista — é decisão do integrador estender L0 |
| L3 `line/app-sculpt3d` | sculpt3d (83, 23 k, 17 cenas) | 6 `impl App` e **180 membros** — a navegação orbital *mora na shell de propósito* (§5: «nunca numa `Tool`»); o que fica na shell tem de ficar pela mesma razão, escrita |
| L4 `line/app-vec` | vec (68, 17,5 k, 4 cenas) | 3 `impl App`, 89 membros; o `vec_entities` é um dos seis módulos internos que as cenas de OUTRAS famílias usam — sai com esta linha só se o substrato de L0 já o expõe |
| L5 `line/app-flip` | flip (61, 16,7 k, 19 cenas) | 0 `impl App`, 75 membros; o `FlipDoc` partilhado (F8 dos Componentes) é a ponte a não partir |

**Smoke do Enio (na worktree da linha), o molde:**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-<FAM> && env PH2D_<ROTEADOR>=<n> cargo run -p ph2d-host-desktop --profile smoke
```

## §6 — Notas para o integrador (quando o Enio mandar)

- **Ordem:** L0 sozinha; depois L1–L5 na ordem que o `collision-surface.sh` medir (ele conta schemas e
  registos — ⚠️ **não conta** o registo novo `ph2d-app-registry-init`; conte-o à mão: cada linha acrescenta a
  sua família, o bloco é gerado, e o gate de *staleness* diz se alguém escreveu à mão).
- **Costuras que ainda conflitam textualmente** (todas por REMOÇÃO/ADIÇÃO em regiões distintas — o Mergiraf
  funde, o olho confere): a lista de `mod` no `main.rs`; campos de `App` em `app_state.rs`; deps no
  `Cargo.toml` da shell; o `file_loc_caps.rs`. ⛔ **Nenhum contador partilhado se move** (`PROJECT_SCHEMA`,
  registos do `ph2d-ecs`): mover código não muda serialização — se uma linha o subiu, ela fez mais do que
  a tarefa.
- **A prova da integração é a mesma do §3, sobre a árvore combinada:** `nextest-list-diff.py` contra a lista
  da base de L0, `ONLY-A = 0`.

## §7 — Os comandos de abertura (uma linha cada; o `/pd-linha-abrir` renderiza o bloco do MODELO)

**As seis abrem hoje**, uma janela nova por linha (MODELO §«Como usar»). O agente responde «Linha pronta.
Aguardo a tarefa.» — a tarefa já vai no comando; responder «siga» basta.

```
/pd-linha-abrir app-host "W2 L0: o substrato para partir a shell (crates ph2d-app-host + ph2d-app-registry-init) + o PILOTO field3d (53 ficheiros, 1 impl App) + o HOWTO_partir_uma_familia_da_shell.md que as outras cinco linhas seguem. Prova: nextest-list-diff ONLY-A=0, roteadores iguais, LOC da shell e unidade bin(check-test) antes/depois." "docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md §0-§4 (INTEIRO antes de tocar num ficheiro); docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md §4-C2; DIRETRIZ §6.7"
```

As cinco famílias (mesmo dia; Fase A até a L0 integrar, Fase B depois do rebase):

```
/pd-linha-abrir app-motion "W2 L1: tirar a família motion da shell para crates/ph2d-app-motion. FASE A desde já (só ficheiros motion_*: poda das cenas não citadas — a =114 fica —, impl App/membros de App para um MotionState, agrupar em src/motion/, nascer a crate com o que só depende das crates do módulo); FASE B depois de L0 integrar (rebase + o corte pelo HOWTO). 232 ficheiros, 141 cenas, 0 impl App, 59 membros. Prova do §3 nas duas fases." "docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md §0-§3 e §5 (INTEIRO antes de tocar num ficheiro); docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md §4-C2"
/pd-linha-abrir app-physics "W2 L2: tirar a família physics da shell para crates/ph2d-app-physics. FASE A desde já (só ficheiros physics_*: poda das cenas não citadas, os 22 impl App e 126 membros de App para um PhysicsState — a mais acoplada: o que só o substrato resolve NOMEIA-SE no handoff, não se inventa —, agrupar em src/physics/, nascer a crate); FASE B depois de L0 integrar (rebase + corte pelo HOWTO). Prova do §3 nas duas fases." "docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md §0-§3 e §5 (INTEIRO antes de tocar num ficheiro); docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md §4-C2"
/pd-linha-abrir app-sculpt3d "W2 L3: tirar a família sculpt3d da shell para crates/ph2d-app-sculpt3d. FASE A desde já (só ficheiros sculpt3d_*: poda das cenas não citadas, 6 impl App e 180 membros para um Sculpt3dState — a navegação orbital FICA na shell por decisão escrita no §5 do CLAUDE.md —, agrupar em src/sculpt3d/, nascer a crate); FASE B depois de L0 integrar (rebase + corte pelo HOWTO). Prova do §3 nas duas fases." "docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md §0-§3 e §5 (INTEIRO antes de tocar num ficheiro); docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md §4-C2"
/pd-linha-abrir app-vec "W2 L4: tirar a família vec da shell para crates/ph2d-app-vec. FASE A desde já (só ficheiros vec_*: poda das cenas não citadas, 3 impl App e 89 membros para um VecState, agrupar em src/vec/, nascer a crate; vec_entities é usado por cenas de OUTRAS famílias — fica na shell até o substrato o expor); FASE B depois de L0 integrar (rebase + corte pelo HOWTO). Prova do §3 nas duas fases." "docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md §0-§3 e §5 (INTEIRO antes de tocar num ficheiro); docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md §4-C2"
/pd-linha-abrir app-flip "W2 L5: tirar a família flip da shell para crates/ph2d-app-flip. FASE A desde já (só ficheiros flip_*: poda das cenas não citadas, 75 membros de App para um FlipState, agrupar em src/flip/, nascer a crate; o FlipDoc partilhado é a ponte a não partir); FASE B depois de L0 integrar (rebase + corte pelo HOWTO). 61 ficheiros, 19 cenas, 0 impl App. Prova do §3 nas duas fases." "docs/IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md §0-§3 e §5 (INTEIRO antes de tocar num ficheiro); docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md §4-C2"
```
