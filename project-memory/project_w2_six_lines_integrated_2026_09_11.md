---
name: project_w2_six_lines_integrated_2026_09_11
description: A W2 (partir a shell) foi integrada em QUATRO rondas até 2026-09-12 por ordem do Enio -- a shell caiu de 526 809 para 186 647 linhas (−65%), sete das nove famílias chegaram ao padrão de chegada, nada foi enviado; e as lições de régua que cada ronda pagou
metadata:
  type: project
---
Estado em 2026-09-11: **as seis linhas da W2 estão no `main`** (65 commits sobre `8fa4f115b`),
sem push — o ship é ordem à parte (CLAUDE.md §0.7).

**Ordem usada** (L0 primeiro por ser o substrato; as cinco por churn de costura DECRESCENTE, para
que quem menos reescreve território partilhado rebaseie por último): `app-host` → `app-vec` →
`app-flip` → `app-physics` → `app-sculpt3d` → `app-motion`.

**Resultado:** `shells/desktop` de **526.809 → 465.105 LOC** (−61.704, −11,7 %) e de 2.023 → 1.801
ficheiros `.rs`; crates novas `ph2d-app-{host,registry-init,field3d,vec,flip,physics,sculpt3d,motion}`
+ `ph2d-viewport3d` + a ferramenta `ph2d-app-sync`.

⭐⭐ **QUATRO defeitos de substrato que a integração pagou, e a forma é a mesma nos quatro: o
portão que os apanharia não corre no dia em que o defeito nasce.**

1. **`ph2d-tool-sync` abria `tests/<x>.rs`** — a W1 (10/09) mudou tudo para `tests/it/<x>.rs`. É o
   único sítio do repo que abre um ficheiro de teste por caminho FIXO (os outros varrem o
   directório) e nada o invoca no laço interno: ele só corre no passo 2 do
   `foundational-integrate.sh` ⇒ o defeito nasceu num dia e apareceu na primeira integração.
2. **O gate de integração não conhecia o TERCEIRO gerador.** A L0 criou o `ph2d-app-sync` +
   `ph2d-app-registry-init` e o passo 2 do script só corria `tool`/`node` ⇒ toda família reprovava
   no passo 3 por staleness. Curado: o script corre os três e testa os três.
3. ⛔⛔ **A invariante da L0 — «`crates/ph2d-app-*` ⇒ família, e família declara ≥1 roteador de
   smoke» — é forte demais para a FASE A.** Medido: das cinco famílias só a `flip` lê as próprias
   `PH2D_*_SMOKE` dentro da crate (15); `vec`, `motion`, `physics` e `sculpt3d` extraíram código e
   **não** o roteador (o `match` de cenas toca a `App`, que é o que a Fase B deve). ⇒ catraca
   `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` no registo, **com censo de obsolescência nas DUAS
   metades** (entrada cuja família já declara roteador reprova; entrada que não é família nenhuma
   reprova), as duas provadas por mutação. ⚠️ Aceitar `routers: &[]` em silêncio seria pior que o
   bloqueio: *«ainda não saiu»* e *«alguém esqueceu»* leem-se **igual** numa lista vazia — a forma
   do `id órfão` contra o `controlo morto`, cuja cura é oposta.
4. ⛔⛔ **O gerador do registo e o `rustfmt` eram um PING-PONG, e o gate escondia-o.** O `rewrite()`
   cortava no TEXTO do marcador de fecho e não no início da LINHA dele; como o bloco vive dentro de
   uma função, o `fmt` indenta e o sync desindentava ⇒ `cargo fmt --check` do `ship.sh` reprovaria
   para sempre. ⚠️ **O gate de staleness já compensava isto a jusante** (compara com a indentação
   do fecho aparada), então o defeito era invisível a todo portão da linha — *uma compensação a
   jusante esconde o defeito a montante, e quem lê o gate conclui que o assunto está tratado*.

**Conflitos de rebase:** `Cargo.lock` em 4 das 6 (a receita §1.5.5: `git checkout main -- Cargo.lock`
+ `cargo metadata` + `git add`, nunca à mão) e **8 de código**, todos resolvidos por UNIÃO — duas
linhas a melhorar o mesmo sítio por razões diferentes (um doc-link que a L0 cortou por atravessar a
fronteira da crate e a `sculpt3d` por o módulo virar pasta privada; um gate a que a L0 deu uma porta
por linha e a `sculpt3d` um nome explícito; campos que a `flip` já retirara da `App`).
⚠️ **Um filtro por PALAVRA ao resolver deixa órfã a primeira linha de um doc-comment de duas** —
aconteceu com o `flip_entities` e só se vê olhando o resultado.

⭐⭐⭐ **E a regra ficou com INSTRUMENTO, a pedido do dono** (*«não seria necessário atualizar os docs
canónicos para que nenhum outro agente volte a trazer problemas de performance na compilação?»*):
**`the_shell_only_shrinks`** em `crates/ph2d-editor-core/tests/it/`. ⛔⛔ **O buraco que ele fecha é
estrutural:** todo tecto de LOC deste repo é **por FICHEIRO** (workspace 700 · painel · widget ·
tool-runtime · `file_loc_caps` da shell) e **nenhum mede a unidade que o compilador constrói** — 465 k
linhas em 1 801 ficheiros de ~258 passam em todos eles com folga, e é a CRATE que decide o relógio.
É a forma do `CLAUDE.md` §5.0 um nível acima: lá quem soma entre linhas sem ninguém a contar é o tecto
por-ficheiro; aqui é a crate. Catraca com as **duas** metades + controlo positivo, as três provadas por
mutação. ⚠️ E a ferramenta que já existia para isto (`ph2d-loc-trend`) **não é chamada pelo `ship.sh`
nem pelo CI** — o padrão que o `CLAUDE.md` §2 nomeia: *ponteiro não é adoção*.

⏳ **FASE B, em curso (11/09).** A `physics` correu como **batedora** e respondeu a pergunta que
prendia as outras quatro: ⭐ **as 5 portas do `AppHost` CHEGAM — zero sextos métodos.** O achado
útil não é o «sim»: o bloqueador **nunca foi a `App`** — três folhas ficaram na shell por
**PARTILHA** entre famílias (`inspector_ordering`/`preview_drive`/`name_unique`, 11/38/14
consumidores) e os três gestos de corpo queriam **tipos** que a `App` por acaso segurava. ⇒ *antes
de pedir porta, escreva o que a função precisa em tipos; se a resposta é «três coisas que a `App`
segura», não é porta — é assinatura.* Integrada: shell **465 105 → 451 084** (−14 021), `render_loop`
da física de 43 ficheiros para **0**, e `"physics"` fora da catraca. ⚠️ **Duas reconciliações do
integrador, ambas da mesma espécie — a prosa a envelhecer atrás do código:** o doc-comment do
`FAMILY` dizia *«`routers: &[]`»* uma linha acima do código que declara o roteador (*uma dívida
cumprida e não apagada lê-se como dívida aberta para sempre*), e o `PH2D_PHYSICS_SMOKE` ficou com
**dois leitores** (a crate e o `init.rs`). ⛔ E a decisão de desenho foi do integrador, não da linha:
`ph2d-app-physics` **pode** depender de `ph2d-timeline` (sem ciclo; as crates de família já dependem
de 5–11 crates-motor irmãs) ⇒ o roteador sai INTEIRO, nunca partido.

✅ **FASE B — o grosso FEITO em 12/09.** Integradas `sculpt3d` (−32 071, saiu quase inteira: de
31 902 para **501** na shell), `vec` (−4 975), `flip` (−2 838) e depois a **`line/shell-folhas`**
(−13 209). Shell **526 809 → 398 037 em dois dias: −128 772, −24,4 %.**

⭐⭐ **A `line/shell-folhas` nasceu de TRÊS linhas independentes a darem o mesmo diagnóstico** —
motion (*«13 pedaços partilhados prendem os 26 bloqueadores»*), flip (*«duas funções de outra
família prendem 84 % de mim»*) e vec (*«o meu bloqueador não é a `App`, é o `name_unique`»*). ⛔ Ela
**não** abriu no dia em que foi pedida: as peças maiores eram território da `line/app-vec`, que
estava a mexer nelas naquele momento. *Uma linha de folhas partilhadas colide com quem está dentro
das famílias — ela abre quando ninguém está.* Lei do bloco: **uma folha por ASSUNTO, nunca um saco**
(uma crate «das coisas partilhadas» resolve o compilador e cria a segunda shell) — saíram sete,
todas nomeáveis sem «comum»/«utils».

⛔⛔ **E o briefing que EU escrevi tinha dois números errados, os dois pela armadilha que eu próprio
documentara no dia anterior** (HOWTO §2.12): dei `transport` como 7 consumidores — tem **um**, e as
outras seis eram a palavra *«transporte»* em PROSA dentro de comentários; e dei `audio` como 569
LOC — são **8 544**, porque contei o ficheiro de topo e não a árvore de 37 por baixo dele. *Aplicar
mal a própria régua um dia depois de a escrever é o modo de falha normal dela, não a excepção.*

⚠️ **E a `flip` reprovou no gate da ÁRVORE COMBINADA** — o único sítio onde podia: o portão de fecho
de uma linha corre o nextest **impactado**, e `shells/desktop/tests/it/` não entra no filtro. ⇒ toda
linha desta wave passa a correr `cargo test -p ph2d-host-desktop --test it` à parte.

⭐ **Espécie nova no HOWTO (§2.13), paga por DUAS linhas no mesmo dia e independentemente:** *a
agulha que nomeia a VISIBILIDADE*. Publicar a API de uma folha obriga `pub(crate) fn` a virar
`pub fn`, e um gate ancorado no modificador reprova sem que a lei mude uma linha. *Uma agulha ancora
na LEI, nunca em quem pode chamá-la.*

**Actualização 2026-09-12 — as rondas seguintes, todas integradas:** Fase B (`physics`) ·
2.ª volta (`flip`, `vec`) + `line/shell-folhas` · Fase C (`motion` −111 397, `vec`, `physics`) ·
Fase D (`components`, `vec`, `painter` — a última linha NOVA). Shell **526 809 → 186 647** (2 023 →
917 ficheiros); o portão de testes impactados caiu de 51 s para 27 s na mesma máquina. Estado
completo, as leis e os blocos de reabertura em `docs/archive/integracao-jornadas/ESTADO_W2_2026-09-12.md`.

**Aberto (12/09):** o **envio** — o `ship.sh` ficou **CI-clean** localmente (12/12, 22 689 testes) depois de
curar 25 avisos, 79 dependências mortas, 2 ficheiros fora do build e um gate que lia uma dependência
morta; faltam a ORDEM do dono e a CI nos três SO. O objectivo da obra foi MEDIDO e cumprido (a shell
`bin (check-test)` 36,2 → 5,9 s; o gate de fecho 92,9 → 45,5 s); o que sobra grande na
shell já **não é família** (o laço `render_loop/mod.rs`, o `input_dispatch`, `project`/`undo`), mais
o **roteador partilhado `build_smoke`** (64 módulos, três famílias) que prende ~18 mil linhas do
Vetor e não tem dono.

⚠️ **E uma lei desta memória está REFUTADA desde 12/09:** a regra *«o `nextest-impacted` não alcança
`shells/desktop/tests/it/`»*, que o integrador escreveu sem medir e espalhou por três blocos de
reabertura — ele alcança (`rdeps(<família>)` → 793 testes). Ver o ESTADO §4 lei 4.

**Actualização 2026-09-13 — a 2.ª jornada da auditoria de arquitectura, integrada por ordem do dono
(*«integre»*):** duas linhas paralelas sobre o mesmo `main`, com uma CERCA de ficheiros entre elas.
`line/render-loop` (A9 + o quadro: `run_render_frame` 13 685 → 984 linhas, 125 fases; `App` 245 → 187
campos) primeiro — preserva os 145 hashes que o handoff dela cita — e `line/editor-core` (A5b + A10:
1 737 ids para as crates donas, módulos da fundação em DAG) replicada por cima: UM conflito
(`vector_bridge.rs`). ⚠️ **O tecto da shell SUBIU** (190 629 → 196 990) por autorização explícita do
dono para caber a divisão do quadro. A integração desceu os 207 ids que a cerca prendia, matou a fachada
`screens::hero::ids` e as três cópias de slug, e pagou curas de gates que nomeavam ENDEREÇOS (três de
família a lerem o `mod.rs`, o verificador de docs cego aos gates da shell, cinco agulhas de id).
O smoke do dono devolveu uma queixa (a arte do Flip sem contorno de realce, PRÉ-EXISTENTE) e a cura
dela achou o `ph2d-app-flip` a não compilar SOZINHO (75 erros escondidos pela unificação de features
da workspace) ⇒ as duas curadas e o `ship.sh` ganhou o passo `check-standalone-optional.sh`.
**ENVIADO em 13/09 por ordem do dono** (*«envia»*): `aceaa439a`, 412 commits. A run 34757814212
reprovou nos três sistemas antes de compilar (o `spike.yml` pedia o shim `ph2d-editor`, apagado a 12/09);
cura `4b170862b` + portão `check-workflow-packages.sh`; a run 34758869553 ficou VERDE inteira.
A seguir o dono abriu três linhas para fechar a refatoração
(`docs/archive/integracao-jornadas/BLOCOS_ABERTURA_REFATORACAO_FINAL_2026-09-13.md`).

**Actualização 2026-09-13 (tarde) — a REFATORAÇÃO FINAL integrada LOCALMENTE** (*«os 3 agentes
finalizaram»*): `input-dispatch → render-bodies → loc-caps` rebaseadas em `integ/refatoracao-final`
(134 commits) e o `main` do primário avançado por fast-forward para `530f66659` (depois `be42390df`,
com a medição de compilação). ✅ **ENVIADO** em 13/09 15:40 por ordem do dono (`4b170862b..be42390df`),
CI run 34775366240 **verde** (lint · 3 OS · replay C9). Números e achados no ESTADO W2 §6 e no
BLOCOS §6-bis: `input_dispatch.rs` 7 115 → 527, `run_render_frame` 984 → 71, listas de LOC das
crates/painéis a zero, shell 196 003 (tecto 196 990 intocado), suíte 22 735 (`ONLY-A 0`), `ship.sh`
14/14. Três achados de integração: o Mergiraf largou duas deleções numa lista e disse «Solved»
(prova commit a commit); dois `dead_code` só na árvore combinada (curados no commit em que nascem);
e o ship reprovou um gate de OUTRA crate cujo contador passava por um memo POR THREAD (curado numa
pool de 1 thread, [[feedback_a_counter_behind_a_per_thread_memo_depends_on_the_scheduler]]). A prova de
movimento ficou versionada em `scripts/moved-proof.py`. ✅ **Smoke do dono OK** (13/09). ⭐ E a
velocidade de compilação foi MEDIDA a pedido dele (antes `0bfee712e` × depois `530f66659`, edição real,
45/45 recompilaram a shell): `check` incremental `0,95 → 0,88 s`, sem incremental `2,33 = 2,33 s`, smoke
`5,00 → 4,72 s`, testes `ci-test` `10,65 → 10,2 s` — **não piorou, 3–8 % melhor**; tabela no ESTADO W2 §6,
commit `be42390df`, e o `main` local avançou para ele.

**Actualização 2026-09-13 (noite) — linhas limpas e pasta de processo arquivada** (*«é preciso limpar a
pasta… deixando apenas os docs úteis»*): **20 worktrees `line-*` removidas** (os 21 ramos `line/*` FICAM);
o que não se regenera foi preservado ANTES em `Worktrees/_arquivo-linhas-2026-09-13/` (73 MB: as três
`.cauda-*` das linhas da refatoração final, o `ph2d_project.postcard` + `spikes/field-spike/out` da
`line-3DModeling`, dois `.claude/settings.local.json`), e as 5 013 `referencias` da `line-UIUX` foram para
`docs/UI_New_and_Simple/referencias/` do primário (gitignored). Disco `555 → 340 GB`. Ficam as worktrees
`line-editor-core` (sessão integradora) e `integ-arquivo-docs`. Os 19 registos datados saíram de
`docs/IntegracaoMultiAgente/` para `docs/archive/integracao-jornadas/` (gate
`the_live_process_folder_holds_no_dated_record`) e os docs de abrir/assumir/fechar/integrar linha foram
conferidos contra o código — `f065b17bb` (scripts) + `1d43da737` (docs). ✅ **ENVIADO** em 13/09 por ordem do dono (*«envie»*,
`be42390df..1d43da737`): `ship.sh` verde (22 736/22 736) e CI run 34778901059 verde (3 SO + C9).
⚠️ Antes de reabrir, os `+12` do `git cherry` em `line/loc-caps`/`line/render-bodies` foram conferidos um a um:
todos têm gémeo no `main` pelo título, 10 com patch idêntico e 2 com a adaptação da integração ⇒ nada perdido.

**Reabertura (13/09, noite):** o dono reabriu **6 linhas de módulo** — `motion-value`, `sculpt3d`, `3DModeling`,
`Vector` (a janela que ele chama «Bones»), `UIUX`, `components` —, recriadas pelo integrador em `Worktrees/line-*`
por **fast-forward** do `main` enviado (os 6 ramos eram ancestrais, `cherry +0`), `target/` com `chattr +C`, e os
restauros do arquivo devolvidos (postcard + `spikes/field-spike/out` do 3DModeling; `.claude/settings.local.json` do
sculpt3d). As janelas já estavam abertas com contexto antigo ⇒ o prompt de cada uma manda executar o
`MODELO_TROCA_DE_AGENTE_NA_LINHA` + um adendo (a memória da conversa é mais velha que a árvore; onde a família mora
hoje; o handoff da W2 dela; seguir com o item em curso em vez de esperar tarefa).

**Why:** o Enio ordenou *«siga. integre.»* em 11/09, depois de as seis linhas fecharem.

**How to apply:** quem integrar uma família da W2 lê a catraca do `ph2d-app-registry-init` antes de
propor que uma família «esqueceu» de se registar; quem criar um gerador novo põe-no no passo 2 do
`foundational-integrate.sh` no MESMO commit, senão o gate dele vira bloqueio de integração.
[[project_dev_speed_audit_2026_09_10_w0_applied]]
