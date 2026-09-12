---
name: project_w2_six_lines_integrated_2026_09_11
description: As SEIS linhas da W2 (partir a shell) foram integradas ao main em 2026-09-11 por ordem do Enio -- L0 primeiro, depois vec/flip/physics/sculpt3d/motion; a shell perdeu 61.704 LOC e 222 ficheiros, e a integração pagou QUATRO defeitos de substrato que nenhum portão de linha via
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

**Aberto:** a **Fase B** das cinco famílias (o corte pelo `HOWTO_partir_uma_familia_da_shell.md`),
que é onde os roteadores saem e a catraca volta a vazia; `ship.sh` + CI **não correram** (ordem do
Enio); o smoke é dele e não foi corrido.

**Why:** o Enio ordenou *«siga. integre.»* em 11/09, depois de as seis linhas fecharem.

**How to apply:** quem integrar uma família da W2 lê a catraca do `ph2d-app-registry-init` antes de
propor que uma família «esqueceu» de se registar; quem criar um gerador novo põe-no no passo 2 do
`foundational-integrate.sh` no MESMO commit, senão o gate dele vira bloqueio de integração.
[[project_dev_speed_audit_2026_09_10_w0_applied]]
