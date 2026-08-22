# Diretriz de Implementação — PH2D

**Versão:** 8.3 — 2026-08-18. O **modo de operação é função do hardware** ([ADR-0106](../architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)) e **foundational não é serial no Modo L** ([ADR-0107](../architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)); **integração e ship só por ordem explícita do Enio** (§1.5.3–1.5.4). *O histórico de versões vive no `git log` deste arquivo — não é re-narrado aqui.*

**Audiência:** **toda LLM que entra no projeto.** Este doc é **referência** — **NÃO
leia inteiro**; use o roteador leia-por-tarefa em [`CLAUDE.md §1`](../../CLAUDE.md) e
leia só a(s) seção(ões) que sua tarefa exige.

⚠️ **A leitura obrigatória é função do TIER — `bash scripts/hw-profile.sh` decide, antes de tudo.**
Ler a lista do outro modo é o desperdício que esta linha existe para cortar:

| `hw-profile.sh` diz | Seu modo | Leia (e só) |
|---|---|---|
| `workstation` (Linux 128 GB) | **L** — linhas por worktree | §0 (sanity) · **§1.5** (o protocolo que você de facto executa) · §2 (triagem) · §6 (velocidade) |
| `constrained` (Mac 8 GiB) | **C** — shared tree + Coordenador | §0 · §1 (papéis, resumo) · §2 · §6 · §7 (git) — o detalhe de ambos está no [arquivo de processo](../archive/processo-2026-08-18/DIRETRIZ.md) |

O resto é por-tarefa.

> **Seu primeiro output sempre = TRIAGEM (§2).** Classifique a tarefa do Enio
> e diga **como proceder** antes de codar.

---

## TL;DR

- **Modo por hardware (v8):** rode `bash scripts/hw-profile.sh` PRIMEIRO. `workstation` (Linux 128 GB) = **Modo L (§1.5)** — N linhas paralelas por `git worktree`, sem Coordenador de plantão. **A integração e o ship NÃO são autônomos: acontecem só por ordem EXPLÍCITA do Enio** (§1.5.3–1.5.4). Cada linha fecha o módulo, escreve um **handoff de integração** (§1.5.9) e PARA; o Enio junta os handoffs e abre **um agente integrador dedicado** que resolve TODOS os conflitos e integra as linhas via `--ff-only` + gate testado. `constrained` (Mac mini 8 GiB — smoke/hotfix) = **Modo C** — o modelo abaixo. Triagem (§2), receitas (§3), contratos (§4), UI (§5) e a DIRETIVA valem NOS DOIS modos; o que muda é git + concorrência + quem integra/pusha.
- **Dois papéis (Modo C):** **um Coordenador único** (absorve foundational + scaffolds + ship + arbitragem de posse) + **N Implementadores** (sempre vários, cada um numa pasta/módulo físicamente disjunto).
- **Três caminhos** (descobertos via Triagem §2):
  - **(A) Drop-crate (fan-out, §3.A)** — node ou tool nova. Implementador sozinho. Zero edit central. Paraleliza com outros (A).
  - **(B) Scaffold central (§3.B)** — painel/widget/chrome. O Coordenador faz scaffold + delega.
  - **(C) foundational ou contrato congelado (§3.C)** — Modo C: Coord-only, não paraleliza. Modo L: foundational **não-contrato** paraleliza pela sua linha (gate testado, §1.5/ADR-0107); só contrato congelado + mesmo-símbolo de tipo-núcleo ficam seriais (ADR / reporte ao Enio).
- **Dois contratos congelados (§4)** com arch-gate ativo: nodes (ADR-0039) e tools (ADR-0040+0041). Mexer = (C).
- **Enio é relay mecânico**, não decisor.
- **Norte:** engine cresce por **duas famílias-irmãs** simétricas — `crates/ph2d-node-*` (declarativo, FBP) e `crates/ph2d-tool-*` (imperativo, manipulação direta). Ambas wireadas por codegen (`ph2d-{node,tool}-sync`). Adicionar conteúdo = drop-crate.

---

## 0. Antes de começar (sanity check obrigatório)

Independente do papel, **rode primeiro**:

```bash
bash scripts/hw-profile.sh       # seu tier → seu MODO (L ou C — vide TL;DR)
git log --oneline -5             # confirma HEAD
git branch --show-current        # Modo C: main · Modo L: line/<seu-módulo> (NUNCA main)
git status -sb                   # working tree limpo?
```

⚠️ **`cargo check --workspace` NÃO é passo de abertura por omissão** — §6.3 lista *"validar
baseline no início da sessão se o último commit já está verde"* entre os desperdícios, e um
`--workspace` frio custa minutos. Rode-o **só** quando o baseline está sob suspeita: HEAD
inesperado, working tree suja, rebase acabado de resolver, ou primeira jornada numa worktree nova.

Algo divergente (HEAD inesperado, working dirty, build quebrado) → **pare e reporte ao Enio.**

**Leitura mínima:**
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18 (Hard Rules) e §1 (arquitetura).
- [`CLAUDE.md`](../../CLAUDE.md) (CI, push, batching).
- Memória persistente (versionada no repo): [`project-memory/MEMORY.md`](../../project-memory/MEMORY.md) (symlink de `~/.claude/projects/<key>/memory`).

---

## 1. Papéis (resumo — o detalhe do Modo C está no arquivo)

- **Modo L** (`workstation`) = **§1.5, abaixo.** Não há Coordenador: cada linha é um agente
  autónomo na sua worktree; o operador humano é o Enio ([`GUIA_JORNADA_MODO_L.md`](GUIA_JORNADA_MODO_L.md)).
  ⚠️ **Se o seu tier é `workstation`, pule para §1.5 — o resto deste §1 não descreve a sua máquina.**
- **Modo C** (`constrained`, Mac 8 GiB — smoke/hotfix): **um Coordenador único** (dono exclusivo de
  foundational, contratos congelados, scaffolds, ADRs, `.github/`, e do ship) + **N Implementadores**,
  cada um numa pasta fisicamente disjunta, que **param e reportam ao Coordenador** ao precisar de
  qualquer coisa fora dela — e nunca renegoceiam entre si.
- A infra que só existe no Modo C — mapa de posse em [`SESSION_ACTIVE.md`](../SESSION_ACTIVE.md),
  slots CoW (`scripts/slot-env.sh`), `scripts/git-stage-guard.sh` — e as **3 obrigações do
  Implementador** estão **verbatim** em
  [`docs/archive/processo-2026-08-18/DIRETRIZ.md`](../archive/processo-2026-08-18/DIRETRIZ.md).
  No Modo L o worktree substitui as três por isolamento físico (§1.5.1).
  ⚠️ **A NUMERAÇÃO foi preservada:** onde este doc ainda diz *§1.1* (SESSION_ACTIVE), *§1.2* (slots),
  *§1.3* (stage-guard) ou *§1.4* (as 3 obrigações), é **naquele arquivo** que a seção está, com o
  mesmo número. O mesmo vale para *§7.1–§7.4* (§7 abaixo).
- **Enio.** Dono do produto e único decisor: ele manda integrar, manda shipar e faz o smoke.

### 1.5 Modo L — linhas paralelas por `git worktree` (tier `workstation`)

> Ativa quando `scripts/hw-profile.sh` = `workstation`. **N linhas de desenvolvimento = N
> worktrees + N branches** (`line/<módulo>`), cada uma numa sessão Claude Code própria;
> `main` vira só ponto de integração. O worktree elimina a colisão de **git** de raiz
> (índice, HEAD, working tree e `target/` próprios por linha — a classe inteira de
> incidentes que o §7 legisla vira impossibilidade física); a **pasta disjunta**, que já
> era regra, elimina o conflito de **merge** nos drop-crates. Juntos = integração fast-forward
> na prática, **sem Coordenador de plantão**.
>
> - **Integrar e pushar não é seu:** a regra é o [`CLAUDE.md §0.7`](../../CLAUDE.md) (que você já
>   tem carregado); o **mecanismo** é §1.5.3–1.5.4 e o **entregável** é §1.5.9.
> - **Foundational é editável pela sua linha** (ADR-0107) — a regra e o desenho-para-isolamento
>   estão em **§1.5.2.1**, uma vez só.
> - As 3 obrigações do §1.4 valem dentro de cada linha, com essa emenda. Triagem §2, receitas §3,
>   contratos §4, UI §5 e a DIRETIVA_IMPLEMENTACAO idem.
> - **Proibido no tier `constrained`** — N worktrees × `target/` não cabem em 8 GiB (vide 1.5.6).

#### 1.5.1 Setup de linha (o PRÓPRIO agente faz, guiado pelo modelo)

**Local canônico das worktrees: `Worktrees/line-<módulo>/` DENTRO do repo** (gitignorada
via `/Worktrees/`). Toda janela VSCode/Claude abre **na raiz do repo primário** (sempre a
mesma pasta, uma janela por agente); o agente-de-linha **cria a própria worktree** na 1ª
mensagem, guiado pelo bloco de [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md)
que o Enio cola. Essência:

```bash
git pull --ff-only origin main                             # hotfixes do Mac
mkdir -p Worktrees
git worktree add -b line/<módulo> Worktrees/line-<módulo> main
cd Worktrees/line-<módulo>                                 # TODO o trabalho a partir daqui
```

- **Depois do setup, NENHUM path da raiz é seu:** o mesmo path relativo existe nas duas
  árvores — editar `crates/...` na raiz = editar o checkout primário compartilhado (árvore
  ERRADA). Todo read/edit/git/cargo acontece dentro de `Worktrees/line-<módulo>/`.
- `target/` é por-worktree automaticamente → **slots CoW (§1.2) dispensáveis**; 1º build é
  frio (minutos, esperado). sccache global + mold (§6.0) amortizam.
- Hooks vivem no `.git` comum → pre-commit tiered roda normal em cada worktree.
- Registro de posse = `git worktree list`. **Uma linha por módulo, nunca duas.**

#### 1.5.2 Regras do agente-de-linha

1. **Edite a(s) pasta(s) do seu módulo livremente; foundational, sob o protocolo testado
   (ADR-0107).** Tocar `ph2d-core`/`ph2d-editor-core`/`ph2d-tokens`/`ph2d-host`/… a partir da
   SUA linha **é permitido** — a integração (1.5.3) roda `scripts/foundational-integrate.sh`
   (gate da árvore COMBINADA: `cargo check --workspace` + impacted tests sobre o tip rebaseado),
   e o Mergiraf (1.5.5) funde o resíduo textual. **PARE e reporte ao Enio** só nos **dois casos
   irredutíveis** (decisão de design com um dono, não merge): (a) **contrato congelado** (§4 —
   caps + arch-gate; exige ADR) ou (b) o rebase (1.5.2.3) conflita em **código fora dos arquivos
   do seu módulo** — colisão de *mesmo símbolo* com outra linha viva (1.5.5, última linha), que
   você não resolve na mão. Fora esses dois, foundational não é mais uma fila única (1.5.4).
   **Isolamento AO CRIAR foundational (importante):** a foundation tem arquitetura de isolamento
   *de propósito* — é o que permite várias linhas a estenderem em paralelo sem colidir. Quando
   você **adiciona** algo foundational, projete-o para isolamento: prefira **arquivo/módulo irmão
   novo** a engordar um arquivo compartilhado; exponha um **ponto de extensão append-only** (lista,
   marcador de codegen, `mod` por responsabilidade) onde a próxima linha pluga, em vez de um site
   central que todas editam. Pegue um **id/const/variant único** e some-o a uma lista ordenada
   (ex.: `NodeId(NNN)` no próximo livre, variant de enum, token) — e **anote-o no handoff (§1.5.9)**
   para o integrador detectar colisão. Menos superfície compartilhada = menos conflito de merge.
2. Commits locais frequentes (`--no-verify` de dia, fast mode §8.1) — e **não integre nem pushe**
   ([`CLAUDE.md §0.7`](../../CLAUDE.md)): você fecha, entrega o handoff (§1.5.9) e espera.
3. **`git rebase main` no início de CADA jornada e antes de integrar** (refs compartilhadas
   entre worktrees — sem fetch). Conflito em arquivo GERADO ou `Cargo.lock` → NUNCA resolva
   na mão (tabela 1.5.5). Conflito em código fora da sua pasta → você violou o item 1.
4. Inner loop normal (`cargo check -p`, §6); **gate batched no fechamento do módulo**
   (§6.6.A.2) — verde-de-compilação vale zero, como sempre. Então **escreva o handoff de
   integração (§1.5.9), reporte "linha pronta + handoff" e PARE.** Não rode
   `foundational-integrate.sh` — isso é do integrador, sob ordem do Enio.
5. Linha longa é anti-padrão: peça integração a cada 1–2 jornadas. Base velha = rebase caro.

#### 1.5.3 Integração ao main (agente integrador dedicado, por ordem do Enio; serializada por `--ff-only`)

**Quem integra:** NÃO é a linha. Quando o Enio **decide integrar** (ordem explícita, fim das
implementações paralelas), ele coleta o **handoff de integração** de cada linha (§1.5.9) e abre
**um agente integrador dedicado**. Esse agente resolve TODOS os conflitos entre as linhas e as
funde ao main, **uma de cada vez**, com o mecanismo abaixo (inalterado, ADR-0107). *(Numa jornada
de 1 linha só, o próprio operador pode ser o integrador — mas ainda por decisão do Enio, nunca a
linha se auto-fundindo no meio do trabalho.)*

**ANTES de integrar, leia o MAPA — uma chamada, não mil greps:**

```bash
bash scripts/collision-surface.sh            # a linha contra main — AGORA, de dentro da worktree
```

⚠️ **Rode-o você, em cada worktree, imediatamente antes de fundir aquela linha — inclusive depois
de cada fusão anterior.** A tabela que veio no handoff (§1.5.9 item 3) mede a linha contra o `main`
do dia em que ela FECHOU; entre esse dia e a sua ordem de integração o `main` andou (e cada linha
que você já fundiu nesta sessão o moveu de novo). O handoff diz o que a linha *achava* que estava
tocando — **a evidência é a leitura de agora**, e a diferença entre as duas nomeia quem mexeu no meio.

Ele mede de uma vez o que o integrador redescobre a cada vez: os **schemas nos TRÊS sítios**
(`PROJECT_SCHEMA` + a escada + a tripla), o **registro de componentes e os DOIS espelhos**, o
**contrato congelado** (§4), **ADR** novo e o próximo livre, **`Cargo.lock`** (pacote externo
contra aresta interna), **marcadores de conflito incluindo `|||||||`** e os **tetos de LOC** dos
arquivos que a linha tocou — cada número com o da base ao lado, porque *um número que soma entre
linhas se CONTA, nunca se escolhe*, e a colisão passa **MUDA** quando as duas escrevem o mesmo literal.

> ⚠️ **Por que existe (medido 2026-08-18):** nas 6 maiores sessões de integração o integrador
> gastava **~1.000 `grep`/`sed` por integração** exatamente nessas perguntas (LOC 2.388× ·
> `PROJECT_SCHEMA` 717× · contratos 402× · ADR 360×), num total de 36 mil chamadas de navegação
> e **62 MB puxados para o contexto**. Uma integração arrastava **~8,4 M tokens** para uma janela
> de 1 M — **~8 compactações por construção**, e o integrador re-descobria tudo depois de cada uma.
> Isso é conferência **repetitiva**, não investigação: a mesma lista, toda vez.

**Depois do mapa, um comando faz o gate:** de dentro da worktree da linha, com o gate batched do
módulo verde:

```bash
bash scripts/foundational-integrate.sh
```

Ele executa, em ordem, e aborta com a orientação certa em qualquer falha:
`git rebase main` → re-sync (tool/node) + commit da regen → staleness gate → **gate da árvore
COMBINADA** (`cargo check --workspace` se a linha tocou foundational; senão `-p` nas crates
mudadas) → `nextest-impacted` → `git -C <primário> merge --ff-only`.

**Por que o `check --workspace` é obrigatório para foundational:** o `--ff-only` prova só que
ninguém entrou entre seu rebase e seu merge; **não prova que a árvore combinada compila**.
Duas linhas disjuntas (drop-crate) estão fisicamente isoladas — mas duas linhas em foundational
podem quebrar juntas (A muda uma assinatura, B chama a antiga), e isso só o build da árvore
rebaseada pega. Como `--ff-only` faz o tip da linha **virar** o main, testar o tip rebaseado ==
testar o futuro main (ADR-0107, prova de correção).

**O `--ff-only` É a serialização:** se falhar, outra linha integrou entre seu rebase e seu
merge → **re-rode o script** (rebase de novo sobre a recém-integrada, re-testa, re-tenta) — você
nunca funde uma combinação não-testada. Precondição dura: primário limpo e em main (sujo =
violação do 1.5.1 — pare e reporte). Linha integrada segue viva pra próxima wave do módulo;
morreu de vez → `git worktree remove Worktrees/line-<x> && git branch -d line/<x>`.

*(Fluxo manual equivalente, se precisar depurar passo a passo: os mesmos comandos, na ordem
acima. O script é a fonte única — não duplique a lista aqui.)*

#### 1.5.4 Pra onde foi o Coordenador

| Função (Modo C) | No Modo L |
|---|---|
| Arbitragem de posse / anti-colisão git (§7) | **Extinta** — worktree isola git; pasta disjunta isola merge; `--ff-only` serializa a integração |
| Foundational (não-contrato) | **Não é mais serial** (ADR-0107): qualquer linha toca foundational e integra pelo gate testado (1.5.3). A não-colisão vem de 3 camadas — pontos de extensão append-only onde couber (Camada 0), Mergiraf no resíduo textual (1.5.5), gate da árvore combinada no resíduo semântico (1.5.3). A `line/foundational` dedicada vira **opcional** (útil só p/ um refactor foundational grande e coeso), não mais a única porta |
| Contratos congelados / scaffolds (B) | Seguem **seriais por natureza** (decisão de design com um dono): contrato congelado exige ADR (§4); mesmo-símbolo de tipo-núcleo = reporte (1.5.2.1). Estes NÃO passam pela Camada 0/1 |
| Ship + push + babysit CI (§8) | **Quem** o faz: o **agente integrador** (ou uma sessão-ship dedicada), 1× por jornada sobre o main integrado — `./scripts/ship.sh` + push + babysit. **Quando** ele pode fazê-lo é a regra do [`CLAUDE.md §0.7`](../../CLAUDE.md), não desta tabela |

#### 1.5.5 Conflitos de rebase/merge esperados (os ÚNICOS legítimos)

> **Mergiraf resolve o resíduo textual (ADR-0107, Camada 1).** Com o driver registrado
> (`scripts/mergiraf-setup.sh`, 1× por máquina), dois agentes que adicionam um variant/campo/token
> em **partes diferentes** do mesmo `.rs`/`.toml`/`.json` fundem sozinhos via AST — o que era
> conflito textual vira auto-merge. Ele **não** decide os dois casos abaixo (`Cargo.lock`/gerados
> = regenere; mesmo-símbolo = reporte) nem pega quebra semântica (isso é o gate testado, 1.5.3).

| Arquivo | Regra |
|---|---|
| `Cargo.lock` | NUNCA na mão (fica no merge default do git, sem Mergiraf): `git checkout main -- Cargo.lock` + `cargo check -p <sua-crate>` regenera + `git add Cargo.lock` |
| `ph2d-{tool,node}-registry-init/` (GERADO) | NUNCA na mão: aceite qualquer lado, re-rode o sync; o staleness gate confirma |
| `chrome/mod.rs` blocos GERADOS (`mod` + `dispatch_all`, ADR-0107) | NUNCA na mão: `cargo run -p ph2d-chrome-sync` regenera dos `chrome/*.rs` (ordem = marcador `z=NN`); gate `architecture_chrome_dispatch_in_sync` confirma |
| `ph2d-editor-core/src/icons.rs` (IconId) · `color_tokens!` list (ColorToken) | Mergiraf mantém AMBAS as entradas; se cair na mão, união trivial (dois lados); gates `enum_order_matches_svgs` / testes de `ph2d-tokens` confirmam |
| SESSION_ACTIVE / `CLAUDE.md §5` / trackers | Só na integração, no primário, **uma linha de trabalho por vez**; cada linha edita só o SEU `HANDOFF_*`. ⚠️ E o que se escreve no §5 é **uma linha de TEXTO** (a de `Aberto`, e a de estado se o módulo mudou de natureza) — a narrativa da jornada é do handoff, §1.5.9 item 8 |
| **Mesmo símbolo** fora dos arquivos do seu módulo | Conflito aqui = duas linhas reescrevendo a mesma função/assinatura de núcleo (colisão de *design*, não de texto). Mergiraf não decide, você não resolve na mão → **reporte ao Enio** (1.5.2.1). É o núcleo irredutivelmente serial (ADR-0107) |

#### 1.5.6 Proibições (Modo L)

- Nunca `push` fora do ship de jornada; nunca `--force` em main; nunca `git worktree add`
  dentro de outro worktree.
- Nunca duas linhas na mesma pasta/crate (a triagem de jornada garante).
- **Nunca abrir Modo L no tier `constrained`** — worktrees múltiplos × `target/` próprios
  estouram RAM/disco no Mac 8 GiB; lá o modo é C, sempre.

#### 1.5.7 Interação com o Mac (Modo C itinerante)

Fluxo multi-máquina (GitHub = fonte única; runbook em
[`docs/DevOps/MULTI_MACHINE_SETUP.md`](../DevOps/MULTI_MACHINE_SETUP.md)):

1. Ship no Linux (1.5.4) → push → CI verde.
2. Mac faz `git pull` em main e roda o smoke (`./play.command`).
3. Erro achado no Mac = **hotfix em Modo C, direto em main** (sessão solo ou Coordenador +
   Implementadores; §1.1–1.4 + §6.6 baseline + §7 valem INTEIROS lá) → ship + push do Mac.
4. De volta ao Linux: primário faz `git pull --ff-only origin main`; linhas abertas fazem
   `git rebase main` (1.5.2.3) e seguem.

**Branches `line/*` não viajam pro Mac** — trabalho no Mac é sempre sobre main.

#### 1.5.8 Abertura de linha — modelo pronto (fonte única)

O bloco que o Enio cola na 1ª mensagem de cada sessão-de-linha vive em
[`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md) — **fonte única, não duplique
aqui**. Fluxo em 2 fases: (1) o agente cria a própria worktree em
`Worktrees/line-<módulo>/`, valida, lê §1.5 + DIRETIVA, reporta **"linha pronta"** e
ESPERA; (2) a tarefa vem na mensagem seguinte (pasta exclusiva + o que construir).
Tracker/docs do módulo nascem depois, **dentro da própria worktree**.

**Operador (Enio):** o passo a passo do SEU lado (planejar linhas → abrir cada uma → intervir só nos 2 casos irredutíveis → **coletar o handoff de cada linha e, por ordem sua, abrir o agente integrador + mandar o ship**) está em [`GUIA_JORNADA_MODO_L.md`](GUIA_JORNADA_MODO_L.md).

**Agente NOVO numa linha que já existe** (troca de janela de contexto, ou retomada depois de a linha ter integrado): o bloco é OUTRO — [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](MODELO_TROCA_DE_AGENTE_NA_LINHA.md). A worktree já existe (não se cria). **O modo de falha dessa troca — e o procedimento de resgate — estão escritos uma vez só, no topo daquele doc** (`§Por que este doc existe`); é ele que manda o `cd` + `pwd` + `git branch --show-current` **antes de ler qualquer arquivo**.

Você é `workstation`, sem bloco colado, e `git branch --show-current` devolve `main`?
Você é uma sessão do **primário** (setup/integração/ship) — **não code em `main` no
Modo L**; pergunte ao Enio qual é a sua linha.

#### 1.5.9 Handoff de integração (cada linha entrega; o Enio passa ao integrador)

Antes de integrar, o **Enio pede a cada linha um handoff de integração** — o documento que
passa ao **agente integrador** os pontos que evitam conflito/regressão. É o **entregável** que
fecha a linha ([`CLAUDE.md §0.7`](../../CLAUDE.md)). Conteúdo mínimo (curto, factual):

1. **Identidade:** branch `line/<módulo>`, HEAD, base do fork (merge-base com main), nº de commits.
2. **Foundational/compartilhado tocado + por quê** — todo arquivo fora da sua pasta de módulo
   (ex.: `editor-core`, `ph2d-core`, `shells/*`, `tokens`), aditivo ou não.
3. **Símbolos que podem COLIDIR com outra linha** — ids/consts/variants/tokens novos com seus
   valores literais (ex.: `NodeId(832)`, variant de enum, entrada em lista ordenada, chave de
   token). É o que o integrador grepa pra detectar mesmo-símbolo (§1.5.5).
   ⚠️ **Cole a saída de `bash scripts/collision-surface.sh` aqui, não a escreva de memória.**
   Ela já traz schemas (nos três sítios), registro + os dois espelhos, contrato congelado, ADR,
   `Cargo.lock`, marcadores e LOC — cada um com o valor da base ao lado. *Um handoff que erra
   aqui manda o integrador procurar o conflito no lugar errado*, e esta §5 registra isso
   acontecendo mais de uma vez (a tabela que "envelheceu entre o fechamento e a ordem", o degrau
   contado a menos, o componente dado como pré-existente e que era novo).

   ⚠️ **PRAZO DE VALIDADE — a tabela colada é REFERÊNCIA, nunca EVIDÊNCIA.** Ela mede a sua linha
   **contra o `main` do dia em que você fechou**. Se a linha fecha na segunda e o Enio manda
   integrar na quinta, com duas linhas fundidas no meio, todo número da coluna "base" mudou e o
   handoff descreve um `main` que já não existe — **e ele não reclama**, porque uma tabela colada
   não sabe que envelheceu. Por isso: **o integrador RE-RODA `collision-surface.sh` em cada
   worktree imediatamente antes de fundir** (§1.5.3), e usa a tabela do handoff só para *saber o
   que a linha ACHAVA que estava tocando* — a divergência entre as duas leituras é ela própria um
   achado, e aponta para a linha que integrou no meio.
4. **Contratos congelados encostados** (§4) — deve ser **nenhum**; se sim, exige ADR (pare e reporte).
5. **O que só o `ship.sh` pega** (o gate de integração NÃO roda): fmt/typos pré-fork, deps novas
   p/ machete, clippy latente, RUSTSEC ([[project_integration_prefork_lines_ship_drift]]).
6. **Ordem/dependências** entre commits, se houver, e **o que smoke-testar** (o que NÃO foi smokado).
7. ⚠️ **RECLAME o `incremental/` da sua worktree** — depois do gate batched e do handoff, antes de
   parar: `rm -rf "$(git rev-parse --show-toplevel)"/target/*/incremental`. São **25 GB por
   worktree** (medido 2026-08-16: 54% do target), risco **zero** (o cargo o recria) e **sem ship**.
   Cinco linhas fechando assim tiram ~125 GB do pico. ⚠️ **Reclamar no FIM, nunca desligar no
   COMEÇO:** durante a jornada o `incremental/` do `dev` é o que faz o `cargo check -p` voar; o que
   ele não pode é sobreviver à linha que o criou. Tabela e as outras duas regras:
   [`DIRETIVA_FIM_DE_DIA.md`](DIRETIVA_FIM_DE_DIA.md) §2-bis.
8. ⚠️ **A NARRATIVA da jornada vai no HANDOFF; o `CLAUDE.md §5` recebe UMA LINHA.** O §5 é o
   **roteador de estado** (o que o módulo é, o que está **aberto**, como smokar, onde ler) — o
   *mecanismo* de cada wave é exatamente o que este handoff existe para guardar. Ao integrar,
   edite a linha **Aberto** do módulo e, quando a wave muda o que o módulo *é*, a linha de estado;
   **não acrescente um parágrafo de jornada.**

   > ⚠️ **Por que a regra existe (medido em 2026-08-18):** o §5 crescia por acréscimo e levou o
   > `CLAUDE.md` de **1,7 KB** (2026-05-08) a **917 KB** em 326 commits — 94,6% dele era o §5, com
   > **um único bullet de 155 KB**. O custo não é estético: este arquivo é injetado **por inteiro**
   > em todo agente, todo subagente e toda worktree, **antes da primeira palavra do Enio** — medidos
   > **466 k tokens de contexto inicial, ~47% da janela de 1 M** —, e **a compactação não o alcança**
   > (ele é re-injetado inteiro em toda janela nova, então o custo fixo é pago de novo a cada
   > compactação). A história até 2026-08-18 foi arquivada verbatim em
   > [`docs/archive/estado-2026-08-18/`](../archive/estado-2026-08-18/) e o §5 voltou a **41 KB**.
   > É a mesma lição da parede de 208 handoffs, um nível acima: **acrescentar é barato para quem
   > escreve e caro para todos os que leem, sempre.**

Modelo de resumo no fim da linha: *"Linha `<módulo>` pronta (HEAD `<sha>`, N commits). Handoff
de integração: <itens 2–6>. Aguardo ordem de integração."*

**ONDE ele é escrito — `docs/<Módulo>/handoffs/`, nunca a raiz de `docs/`** (regra de
2026-08-10). O topo da pasta do módulo é o **pensamento** dele (planos, pesquisas, `BUGS_*`);
`handoffs/` é o **registro cronológico de sessão**. Vale para os três tipos: o handoff de
integração, o de continuação/troca-de-agente (§1.5.7) e o briefing de abertura.

> ⚠️ **Por que a regra existe:** a raiz de `docs/` acumulou **208** handoffs soltos em ~2 meses
> — mais do que o resto de `docs/` inteiro —, e o custo não é estético: o roteador do CLAUDE.md
> §1 manda a LLM nova ler a pasta do módulo, e ela abria numa parede onde plano e registro de
> sessão eram indistinguíveis. Os 208 foram arrumados nesta data, com os links reescritos e
> conferidos por um verificador antes/depois (**zero link quebrado novo**). Escrever o próximo
> na raiz reconstrói a parede um arquivo por vez.

---

## 2. TRIAGEM — seu PRIMEIRO output

Quando o Enio descreve uma tarefa, **antes de codar** responda exatamente neste formato:

```
TRIAGEM
- Tarefa: <1 linha do que o Enio pediu>
- Caminho: (A) drop-crate | (B) scaffold | (C) Coord-only
- Toca contrato congelado (nodegraph/expr OU Tool/RasterEditTool/CanvasPaintTool/PanelEvent)?
    <Não | Sim — exige ADR + bump de cap>
- Razão: <1-2 linhas>
- Se grande/ambíguo: <peças isoláveis vs. compartilhadas>
```

### Tabela de decisão

| Tarefa | Caminho | Razão |
|--------|---------|-------|
| **Nó novo** (domínio com avaliador existente) | **(A) §3.A** | Drop-crate `crates/ph2d-node-<dom>-<slug>/` + `cargo run -p ph2d-node-sync`. Wiring gerado. |
| **Tool nova** (any shape) | **(A) §3.A** | Drop-crate `crates/ph2d-tool-<slug>/` + `cargo run -p ph2d-tool-sync`. Sem variant novo em `EditorAction`. |
| **Modificar** nó/tool existente | **(A) §3.D** | A pasta já existe — edite dentro dela. |
| **Painel novo** (`ph2d-panel-<slug>`) | **(B) §3.B.1** | Coord plumba feature flag + `register_all_panels` ANTES. |
| **Widget primitive novo** | **(B) §3.B.2** | Cria o arquivo + `cargo run -p ph2d-widget-sync` (bloco `mod` GERADO); `pub use` + showcase à mão. |
| **Chrome handler novo** | **(B) §3.B.3** | Cria stub `chrome/<slug>.rs` + marcador `z=NN` + `cargo run -p ph2d-chrome-sync` (`mod` + `dispatch_all` GERADOS). |
| **Avaliador novo (Wave-neck)** — Shader/Som/Gameplay | **(C)** durante neck → (A) depois | Trabalho "tipo W2" serial; abre fan-out só após o neck. Tracker **histórico** (arquivado): [`HANDOFF_node_system.md`](../archive/handoffs-2026-06-16/HANDOFF_node_system.md). |
| **Mudar tokens / editor-core (não-contrato) / shells / arch tests** | **(C)** | Foundational. Modo C: não paraleliza (Coord). Modo L: sua linha + gate testado (ADR-0107, vide nota abaixo). |
| **Mudar contrato de nós** (porta, EvalCtx, motor) | **(C) + ADR** | Bump cap em `architecture_contract_surface.rs` + ADR estendendo 0039. |
| **Mudar contrato de tools** (método em `Tool`/`RasterEditTool`, variant em `PanelEvent`) | **(C) + ADR** | Bump cap em `architecture_tool_contract_surface.rs` + amendment de ADR-0040 §7. |

**Heurística de 1 frase:** conteúdo (nó) OU peça que manipula bitmap (tool) = **(A) drop-crate**. Chrome que renderiza tools/nós (painel/widget/chrome) = **(B) Coord scaffold**. Mudar regra do jogo (contrato congelado, foundational) = **(C) Coord-only + ADR**.

**Na dúvida A vs B:** "exige editar QUALQUER arquivo fora de UMA pasta nova?" Sim → (B). Único arquivo fora = wiring **gerado** (`ph2d-{node,tool}-sync`) → ainda **(A)**.

**Modo L (ADR-0107):** a triagem é idêntica; muda o **executor**. (A)/(D)/(B) **e (C) quando é
foundational NÃO-contrato** (tokens / editor-core / shells / arch tests) = **a sua linha** —
integra pelo gate testado (§1.5.3). **Só (C) contrato congelado** (nós/tools, linhas 273-274) =
reporte ao Enio pra sequenciar + ADR (§4); mesmo-símbolo de tipo-núcleo idem (§1.5.2.1). A
`line/foundational` dedicada é opcional (refactor foundational grande e coeso), não a única porta.

**Diff do sync é esperado** — não viola §1.4 ISOLAMENTO. O staleness gate em CI exige a regeneração.

---

## 3. Receitas

### 3.A Fan-out drop-crate (caminho (A)) — node OU tool

Receita simétrica única. Drop a crate, roda o sync, gates fecham. **Sem coordenação, sem edit central.**

#### 3.A.1 Mapa node ↔ tool

| Aspecto | **Node** (declarativo, pull / FBP) | **Tool** (imperativo, push) |
|---|---|---|
| Pasta exclusiva | `crates/ph2d-node-<dom>-<slug>/` | `crates/ph2d-tool-<slug>/` |
| Codegen | `cargo run -p ph2d-node-sync` | `cargo run -p ph2d-tool-sync` |
| Wiring gerado | `register_all_nodes` + deps (1 superfície) | `register_all` + `register_all_tools` + deps + 2 testes (5 superfícies) |
| Gate wiring | `cargo test -p ph2d-node-registry-init` | `cargo test -p ph2d-tool-registry-init` |
| Contrato | `NodeOp` + `NodeManifest` (`ph2d-nodegraph`) | `Tool` + opcional `RasterEditTool` + `ToolManifest` (`ph2d-editor-core` + `ph2d-tool-registry`) |
| 🔒 Cap arch-gate | `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` (ADR-0039) | `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` (ADR-0040+0041+amendments; cap real em `architecture_tool_contract_surface.rs`) |
| Entry points | `pub fn register(reg: &mut NodeRegistry) -> Result<…>` | `pub fn register(reg: &mut Registry)` (manifest) E/OU `pub fn make() -> Box<dyn Tool>` (behavior); 3 sabores §3.A.3 |
| Vocab de canal | portas tipadas + effect + clock + params | `EditorAction::{ActivateTool, OneShotImageOp, ToolPanelEvent(PanelEvent), CancelActiveTool}` (4 genéricos — sem variant per-tool) |
| Templates | `ph2d-node-debug-const/` (Pure trivial) · `-debug-wave/` (Temporal + ph2d-expr + golden) · `-motion-{grid,clone,transform}/` (vertical Stateful-free) | `-make-square/` (sabor 1) · `-move/` (sabor 2, `is_default=true`) · `-padding/` (sabor 3 leve) · `-bgremoval/` (sabor 3 completo) |
| Pegadinhas | `ctx.param("nome")` no eval (nunca `MANIFEST.params[..].default`); `param_as_count(v, max)` p/ alocação capada | `apply_ui_edit` = single-source-of-truth de clamps; ícone exige IconId variant alfabético em `editor-core/src/icons.rs` |

#### 3.A.2 Briefing pronto-pra-colar

Substitua `<family>` por `node`/`tool`, `<slug>` pelo seu, e (se node) `<domínio>`. Apague os blocos da família errada antes de mandar ao agente.

> **Variante 100% paste-ready** (zero placeholder, com algorithm.rs + icon.rs preenchidos): [`examples-fan-out.md`](examples-fan-out.md) instancia esse briefing fim-a-fim para `ph2d-node-shader-blur` e `ph2d-tool-grayscale`. Use a parametrizada abaixo pra flexibilidade; use os exemplos concretos quando o agente é novo e o objetivo é zero-substituição-mental.

```
═══════════════════════════════════════════════════════════════════
BRIEFING — <family>-crate · slug: <slug>  [node]  · domínio: <domínio>
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA:
  [node]  crates/ph2d-node-<domínio>-<slug>/
  [tool]  crates/ph2d-tool-<slug>/
Glob workspace.members cobre — NÃO edite Cargo.toml raiz.

ANTES DE CODAR: leia DIRETRIZ §3.A.1 (mapa) + copie o template do seu
sabor (vide §3.A.3 pra tool).

O QUE VOCÊ FAZ (só dentro da sua pasta):
0. **`src/lib.rs` PRIMEIRO** (mesmo com 1 linha — `#![forbid(unsafe_code)]`).
   Cargo recusa o manifest enquanto crate-novo não tem lib.rs; como o
   workspace usa glob `crates/*`, TODAS as outras sessões paralelas
   ficam bloqueadas com `can't find library X` até esse arquivo existir.
   Regra: lib.rs primeiro, depois Cargo.toml, depois módulos auxiliares.
1. Cargo.toml: deps mínimas.
   [node]  ph2d-nodegraph, ph2d-node-registry, ph2d-expr se usar math
           por-elemento.
   [tool]  ph2d-tool-registry, ph2d-editor-core (Tool / FloatingPanel
           se stateful), ph2d-a11y, ph2d-core, ph2d-vector p/ ícone.
2. src/lib.rs: implemente o contrato.
   [node]  pub const MANIFEST: NodeManifest { id (NodeTypeId::of(
           "<dom>.<slug>")), name, inputs/outputs, effect (Pure|
           Temporal|Stateful), clock, params, lowerings };
           impl NodeOp { manifest(); eval(ctx) — lê params via
           ctx.param("nome"); cape via param_as_count(v, max) se aloca };
           pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError>
   [tool]  Escolha o sabor (§3.A.3) e siga o template.
3. [node] Golden test: source→seu-nó, register, g.validate(&ops), cook,
         asserta saída.
   [tool] Tests: register attaches manifest / make builds / panel layout /
         handle_panel_event clamping.
4. ÍCONE (só tool — node não tem pill).
   [tool] src/icon.rs com BezPath (porte docs/design/icons/<slug>.svg,
         Lucide 24×24, stroke="currentColor"). Adicione IconId variant
         em ph2d-editor-core/src/icons.rs em ORDEM ALFABÉTICA — gate
         enum_order_matches_svgs falha se sair de ordem; NUNCA pule
         via --no-verify (quebra TODOS os ícones).

O QUE VOCÊ NÃO TOCA:
- Qualquer arquivo fora da sua pasta.
- 🔒 Contrato congelado (vide §4). Mudança = serial + ADR (Modo C: Coord-only; Modo L: reporte ao Enio, §1.5.2.1).
  [node]  ph2d-nodegraph, ph2d-expr, ph2d-node-registry,
          ph2d-node-registry-init/ (GERADO).
  [tool]  editor-core/src/tool.rs (Tool/RasterEditTool/PanelEvent),
          action_bus.rs::EditorAction (use os 4 genéricos),
          ph2d-tool-registry, ph2d-tool-registry-init/ (GERADO),
          resto de editor-core (foundational).
- Cargo.toml raiz.

WIRING (sem colisão, sem edit central):
  cargo run -p ph2d-<family>-sync          # regenera tudo
  cargo test -p ph2d-<family>-registry-init  # staleness fecha

VALIDAÇÃO (§6):
  cargo check  -p ph2d-<family>-<slug>
  cargo test   -p ph2d-<family>-<slug>
  cargo clippy -p ... --all-targets -- -D warnings
  cargo fmt -p ...

NOMES (gates ativos):
  [node]  type name canônico = "<dom>.<slug>", único cross-crate
          (colisão pega em RegistryError::Collision).
  [tool]  manifest id = "<slug>" único; label_key = "tool.<slug>.label".

SE PRECISAR ALGO FORA (dep externa, contrato congelado, EditorAction
variant, domínio novo): PARE e reporte ao Enio. Provavelmente não era
fan-out puro — revise triagem §2.

QUANDO TERMINAR, reporte:
  "<Family> <slug> pronto. Commit local: <sha>. cargo test -p
   ph2d-<family>-<slug> e -p ph2d-<family>-registry-init verdes."
═══════════════════════════════════════════════════════════════════
```

#### 3.A.3 Sabores de tool

| Sabor | Expõe | Templates | Quando usar |
|---|---|---|---|
| **(1) One-shot stateless** | `pub fn register` (manifest) | `-make-square/` · `-trim-transparency/` · `-real-size/` · `-rasterize/` | Pill dispara algoritmo puro no Sprite ativo. Sem `impl Tool`. Shell drena via `EditorAction::OneShotImageOp`. |
| **(2) Palette modal** | `pub fn make` (`Box<dyn Tool>`) | `-move/` (`is_default=true`) | Cursor de canvas, sem pill. `impl Tool` + `build_panel`. Sem `ToolManifest`. |
| **(3) Stateful + panel docado** | ambos `register` E `make` | `-padding/` (leve) · `-bgremoval/` (completo) · `-color-equalization/` · `-upscale/` | Pill + panel próprio (`ph2d-panel-<slug>/`) + preview/commit raster. (1)+(2)+opcional `impl RasterEditTool`. |

O `ph2d-tool-sync` é configurado pelas needles `"pub fn register("` (manifest) e `"pub fn make("` (behavior) — sabor (1) só em `register_all`, (2) só em `register_all_tools`, (3) nos dois.

#### 3.A.4 Trait `RasterEditTool` (heads-up importante)

Sub-trait com 5 métodos (`set_source` / `current_preview` / `take_pending_commit` / `run_full` / `deactivate`), congelado em ADR-0041. **4 tools de produção implementam** — BgRemoval, Color Equalization, Upscale e **Painter** (`ph2d-tool-painter/src/tool/trait_impls_raster.rs`, num arquivo próprio por causa do teto de LOC da workspace). Padding e Equalize Sizes são exceção documentada (geométrico-only / multi-sprite-required). *A conta se faz com `git grep -l "impl RasterEditTool for" crates/ph2d-tool-*`, nunca de memória: o gate `architecture_image_tool_kind_contract` lê o mesmo fonte.*

**Padrão pra tool stateful que produz raster:**

1. **No tool crate:** `impl RasterEditTool for <Tool>` com os 5 métodos. Cache via `cached_canvas_preview: Option<(Vec<u8>, u32, u32)>`. **Critical:** `set_source` e `Tool::on_deactivate` DEVEM zerar o cache (audit Wave 10 §A1+A2: pular causa stale-frame).
2. **No shell:** `shells/desktop/src/render_loop/<slug>_bridge.rs` espelhando `bgremoval_preview.rs`. Use os 4 helpers de `ph2d-tool-runtime`: `drive_source_push`, `drive_preview_cache`, `drive_pending_commit`, `drive_deactivate_cleanup`.
3. **Bits tool-specific** (panel snapshot publish, brush ring, tint overlay) seguem via `as_any_mut().downcast_mut::<ConcreteTool>()` — **exceção documentada** (ADR-0040 §3), NÃO code smell.

**Template canônico:** [`shells/desktop/src/render_loop/bgremoval_preview.rs`](../../shells/desktop/src/render_loop/bgremoval_preview.rs).

#### 3.A.5 Garantia formal de não-colisão

Dois agentes adicionando duas features (mesma família ou não) **não tocam nenhum arquivo em comum**: cada um cria sua pasta; `workspace.members` é glob; superfícies centrais são geradas determinísticamente pelo sync entre marcadores codegen, e staleness gates pegam regen-esquecida. O contrato é o único acoplamento — e está congelado pelo arch-gate (§4). **Para tool especificamente**, `editor-core` está proibida de ganhar dep em qualquer `ph2d-tool-*` concreto (`editor_core_has_no_concrete_tool_deps`) — a única edge permitida é `tool-* → editor-core`.

#### 3.A.6 Checklist do revisor

**Comum:**
- [ ] `cargo run -p ph2d-<family>-sync` rodado; staleness verde.
- [ ] arch-gate do contrato congelado verde (sem cap-bust).
- [ ] clippy `--all-targets` + fmt limpos.
- [ ] Sem dep fora do contrato.

**Node:**
- [ ] `MANIFEST` completo (params + lowerings); nome canônico `"<dom>.<slug>"` único.
- [ ] `eval` puro (sem global, sem IO); effect declarado bate; params via `ctx.param`; alocação capada via `param_as_count`.
- [ ] Golden test verde.

**Tool:**
- [ ] `MANIFEST` completo OU `is_default` correto (sabor 2: só o tool default — Move — retorna true).
- [ ] Se stateful: `handle_panel_event` cobre 1:1 os NodeIds; rota tudo via `apply_ui_edit`.
- [ ] Se `impl RasterEditTool`: `as_raster_edit_mut` retorna `Some(self)`; cache zerado em `set_source` + `on_deactivate`.
- [ ] Ícone: SVG em `docs/design/icons/` + IconId alfabético em `icons.rs`.
- [ ] **Painel docado segue Widget Gallery (§5.2)**: `link_slider_number`, `mark_chip_no_stepper`, storage `0..1`, bridge `<slug>_bridge.rs` se altera pixels.

### 3.B Scaffold central (caminho (B)) — Coordenador (Modo C) / a própria linha (Modo L)

Painel/widget exigem alguns plugues centrais; **chrome é totalmente codegenado** (`mod` + `dispatch_all` via `ph2d-chrome-sync`, ADR-0107 — vide §3.B.3). Cria-se pasta/arquivo + plugues/stubs verdes; no Modo C o Coordenador faz e entrega briefing, no Modo L (ADR-0107) a própria linha faz sob o gate testado.

#### 3.B.1 Painel novo (`ph2d-panel-<slug>`)

Coord:
1. Decide `slug`, `DEFAULT_VISIBLE`, feature flag (`panel-<slug>`).
2. Cria `crates/ph2d-panel-<slug>/` com `Cargo.toml` (deps: `ph2d-editor-core`, `ph2d-a11y`, `ph2d-tokens`, `ph2d-text`, `ph2d-vector`, `ph2d-tool-registry`).
3. Cria `src/lib.rs` com stub `impl Panel` (template completo: [`ph2d-panel-inspector`](../../crates/ph2d-panel-inspector/src/lib.rs)). **Notas factuais:** `Panel::paint` tem 2 params (`state`, `ctx`); o host fica em `ctx.host` (campo de `PaintCtx`), não param separado; trait usado pelo host é `PanelHostInternal`; `hash_node_id` vive em `ph2d-tool-registry`.
4. Em [`ph2d-panel-registry-init/Cargo.toml`](../../crates/ph2d-panel-registry-init/Cargo.toml): adiciona feature `panel-<slug> = ["dep:ph2d-panel-<slug>"]` + entrada em `[dependencies]` `{ path = "...", optional = true }` + inclui em `default = [...]`.
5. Em `ph2d-panel-registry-init/src/lib.rs::build_typed_registry`: `#[cfg(feature = "panel-<slug>")] reg.push(ErasedPanel::new::<ph2d_panel_<slug>::Panel>());` (ordem não é alfabética — sem arch-gate, mantém ordem de migração ADR-0029).
6. Atualiza `EXPECTED_TYPED` no `#[cfg(test)] mod tests` (incrementa contador).
7. `cargo check -p ph2d-panel-<slug>` + `cargo test -p ph2d-panel-registry-init` verde.
8. Commit + briefing pro Implementador (§2.B).

Implementador: preenche `paint`, `apply_event`, `populate`, `State`.

#### 3.B.2 Widget primitive novo (em `editor-core/src/widget/`)

Coord:
1. Cria `crates/ph2d-editor-core/src/widget/<slug>.rs` (template: [`button.rs`](../../crates/ph2d-editor-core/src/widget/button.rs)).
2. `cargo run -p ph2d-widget-sync` regenera o bloco `mod` de `widget/mod.rs` (entre os marcadores `ph2d-widget-sync` — NÃO edite à mão); adicione só o `pub use <slug>::{...};` (re-export, à mão, ordem alfabética).
3. Cria seção no showcase em `widget/showcase/` (copia layout de `switches.rs`). Arch test `architecture_widget_showcase_coverage` enforça.
4. `cargo check -p ph2d-editor-core` + 4 arch-tests de widget verdes: `architecture_widget_loc_cap` (≤500 LOC), `architecture_widget_showcase_coverage`, `no_literal_color`, `hr12_widgets_a11y`.

Implementador (Modo C) / a própria linha (Modo L, sem handoff): preenche paint usando **só tokens**, adiciona tests, ajusta showcase.

#### 3.B.3 Chrome handler novo (dispatch GERADO — ADR-0107)

Modo C: Coord. **Modo L: a própria linha** (editor-core é editável sob o gate testado, ADR-0107).
O bloco `mod` **e** a chain `dispatch_all` são GERADOS — zero edit central à mão:
1. Cria `editor-core/src/screens/hero/chrome/<slug>.rs` com stub `pub fn apply(_hero, _event) -> bool { false }`
   e o marcador de prioridade `// ph2d-chrome-sync:z=NN` na 1ª linha (menor = despacha antes, "vence" em id overlap; omita → vai pro fim, `DEFAULT_Z`).
2. `cargo run -p ph2d-chrome-sync` regenera `mod` + `dispatch_all` (ordem = `z=NN`, depois nome). **NUNCA edite os blocos entre marcadores à mão** — gate `architecture_chrome_dispatch_in_sync` confirma.
3. Se precisa NodeIds: `screens/hero/ids.rs` via `hash_node_id`.
4. `cargo check -p ph2d-editor-core` verde.

Implementador (Modo C) / a própria linha (Modo L): preenche o corpo do handler.

### 3.C Foundational + contratos congelados (caminho (C))

Foundational = `ph2d-core`, `ph2d-tokens`, `ph2d-editor-core` (exceto widget/chrome scaffold de B), `ph2d-a11y`, `ph2d-host`, `ph2d-vector`, `ph2d-text`, `ph2d-tool-registry`, `ph2d-{tool,node,panel}-registry-init`, `tools/ph2d-{node,tool}-sync`, `shells/*`, arch tests, **+ os 2 contratos congelados** (§4).

**Modo C:** não paralelizável — o Coordenador faz sozinho. **Modo L (ADR-0107):** foundational NÃO-contrato é editável por **qualquer linha** sob o gate testado (§1.5.2.1 + §1.5.3); só **contrato congelado** e **mesmo-símbolo de tipo-núcleo** ficam seriais (reporte/ADR).

Exemplo (adicionar o token semântico `accent-teal`):
1. Edita `docs/design/tokens.json` em todos 4 temas (o valor OKLCH; Mergiraf une adições JSON disjuntas).
2. Adiciona **uma linha** na lista `color_tokens!` de `crates/ph2d-tokens/src/color.rs`: `AccentTeal => "accent-teal",` (com doc opcional). O enum `ColorToken` **e** `key()` saem dessa lista — não há mais `match` separado a manter (ADR-0107).
3. `cargo test -p ph2d-tokens` (build.rs regenera as tabelas; gates WCAG revalidam).
4. Commit: `feat(tokens): add ColorToken::AccentTeal`.

### 3.D Modificar feature existente

Sem scaffold. Pasta já existe. **Caminho (A) Implementador-só** — Enio abre sessão Implementador e cola:

```
Edite crates/ph2d-<family>-<slug>/src/<arquivo>.rs. Tudo da feature
vive no crate isolado (manifest + tool + algorithm + icon + params +
panel docado em ph2d-panel-<slug>/ quando aplicável). Não toque em nada
fora. Se exigir arquivo central (Cargo.toml raiz, EditorAction,
contrato congelado, foundational): PARE e reporte — quase certo a
tarefa estava mal triada.
```

Pasta canônica por feature:

| Feature | Pasta |
|---|---|
| Tool (algo / ícone / manifest / `impl Tool` / `handle_panel_event`) | `crates/ph2d-tool-<slug>/` |
| Vocab UI de um tool (`<Slug>UiEdit`, `…UiSnapshot`, `…Params`) | `crates/ph2d-tool-<slug>/src/params.rs` |
| Panel docado de um tool | `crates/ph2d-panel-<slug>/` |
| Nó | `crates/ph2d-node-<dom>-<slug>/` |
| Painel genérico (Inspector/Hierarchy/etc.) | `crates/ph2d-panel-<slug>/` |
| Widget primitive | `crates/ph2d-editor-core/src/widget/<slug>.rs` |
| Chrome handler | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` |

**A pasta `crates/ph2d-editor-core/src/tools/` NÃO existe** desde ADR-0040 TG-D (`c4063b7`). Memória/doc antigo apontando lá = stale.

### 3.E Cross-cutting (perf audit, refactor cross-crate, sweep de lint)

Algumas tarefas não cabem em §3.A-D porque tocam múltiplos crates por natureza. **Modo C: o Coordenador autoriza explicitamente a exceção ao ISOLAMENTO** no briefing. **Modo L: a sweep cross-crate é a sua própria linha** — a emenda do §1.5.2.1 já permite tocar foundational; colisão de mesmo-símbolo com outra linha viva → reporte ao Enio (§1.5.5). Briefing (Modo C):

> "Você toca tests em vários crates conforme os achados. Exceção autorizada à regra de uma pasta isolada (DIRETRIZ §1.4). Cada commit ainda fica T1 single-crate sempre que possível."

**Regras:**
1. Cada commit valida-se sozinho (`cargo test -p <crate>` verde antes).
2. Não tocar production code de foundational sem motivo claro — em audit de tests, só `tests/` + `#[cfg(test)]`.
3. Documentar risk surface no relatório final.

---

## 4. Contratos congelados — caps + arch-gates

**Dois contratos paralelos, mesma disciplina.** Mexer = serial + ADR (Modo C: Coordenador-only; **Modo L: PARE e reporte ao Enio**, §1.5.2.1 — não há Coordenador de plantão).

| Contrato | Arquivos | Arch-gate (cap) | ADR | Mudar exige |
|---|---|---|---|---|
| **Sistema de nós** (W2.T4, 2026-05-22) | `crates/ph2d-nodegraph/src/{lib,node,port,effect,attr,cook,graph}.rs` + `crates/ph2d-expr/src/lib.rs` | [`architecture_contract_surface`](../../crates/ph2d-nodegraph/tests/architecture_contract_surface.rs) — `NodeOp ≤ 2` métodos, `OpResolver ≤ 1` método, `NodeManifest ≤ 8` campos | [ADR-0039](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md) | Bump cap + ADR estendendo 0039 + (se `ph2d-expr`) re-provar paridade CPU↔WGSL |
| **Sistema de tools** (TG-E + ADR-0041, 2026-05-22) | `crates/ph2d-editor-core/src/tool.rs` (`Tool`, `RasterEditTool`, `CanvasPaintTool`, `PanelEvent`) + canal genérico em `crates/ph2d-editor-core/src/action_bus.rs` (`EditorAction::{ActivateTool, OneShotImageOp, ToolPanelEvent, CancelActiveTool}`) | [`architecture_tool_contract_surface`](../../crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs) — `Tool ≤ 12` métodos, `RasterEditTool ≤ 5` métodos, `CanvasPaintTool ≤ 1` método, `PanelEvent ≤ 4` variants | [ADR-0040](../architecture/decisions/0040-tool-as-isolated-feature-crate.md) + [ADR-0041](../architecture/decisions/0041-rasteredit-rename-and-deactivate.md) | Bump cap + amendment de ADR-0040 §7 |

**O que NÃO mexe nesses contratos** (vide §3.A, sem Coord):

- Nó novo num domínio com avaliador — `ph2d-node-<dom>-<slug>/` + sync.
- Tool nova (any shape) — `ph2d-tool-<slug>/` + sync.
- NodeId novo num panel docado — só edita o crate do tool/panel.
- Campo novo num `<Slug>UiEdit` — vive em `ph2d-tool-<slug>/src/params.rs`.

---

## 5. UI canônica — única fonte de verdade

Tudo de UI passa por **tokens**. Sem exceção.

```
docs/design/tokens.json    (designer edita; 4 temas; OKLCH p/ cores)
        │  (build.rs em ph2d-tokens regenera)
        ▼
crates/ph2d-tokens/src/    (5 enums: ColorToken, Spacing, Radius, TypeToken, StrokeToken)
        │
        ▼
let bg = ColorToken::Bg2.resolve(theme);
let pad = Spacing::Lg.px();
```

### 5.1 Gates ativos

Violação = build vermelho. Não há "vou abrir exceção".

| Gate | O que barra |
|---|---|
| [`no_literal_color`](../../crates/ph2d-editor-core/tests/no_literal_color.rs) | hex `0xRRGGBB`, `Color::rgba8(...)`, `Color::WHITE` em widget/screens |
| `no_magic_numeric` | `f32`/`f64` literais em UI fora do allowlist (`0.0`, `±0.5`, `±1.0`, `±2.0`) |
| [`hr12_widgets_a11y`](../../crates/ph2d-editor-core/tests/hr12_widgets_a11y.rs) | widget que não emite `Node` AccessKit |
| [`architecture_widget_loc_cap`](../../crates/ph2d-editor-core/tests/architecture_widget_loc_cap.rs) | widget primitive > 500 LOC |
| [`architecture_widget_showcase_coverage`](../../crates/ph2d-editor-core/tests/architecture_widget_showcase_coverage.rs) | widget que não aparece no Widget Gallery (nem em opt-out) |
| [`architecture_no_chip_without_steppers`](../../crates/ph2d-editor-core/tests/architecture_no_chip_without_steppers.rs) | chip pill sem `link_slider_number`/`mark_chip_no_stepper` (phantom stepper) |
| [`architecture_panel_wiring_parity`](../../crates/ph2d-editor-core/tests/architecture_panel_wiring_parity.rs) | id hit-indexado no paint sem registro em `populate.rs` (não-focável → clique morto) |
| [`architecture_workspace_file_loc_cap`](../../crates/ph2d-editor-core/tests/architecture_workspace_file_loc_cap.rs) | arquivo `crates/*/src/**` > 700 LOC ([ADR-0105](../architecture/decisions/0105-file-loc-cap-600-to-700.md); fora dos caps de painel/widget/runtime) |
| [`architecture_docs_reference_live_gates`](../../crates/ph2d-editor-core/tests/architecture_docs_reference_live_gates.rs) | doc instrucional cita gate `architecture_*` inexistente |
| `mockup_tokens_exist` | `var(--X)` em mockup HTML não resolve em tokens.json |
| `architecture_register_all_alphabetical` | `register_all*` / Cargo deps fora de ordem |
| `staleness` (tool + node) | sync esquecido |
| [`architecture_cycle_prevention`](../../crates/ph2d-editor-core/tests/architecture_cycle_prevention.rs) | `editor-core` ⊥ `panel-*`/`ph2d-editor`; `editor-core` ⊥ `tool-*` (exceto `ph2d-tool-registry`); `panel-*` ⊥ outro `panel-*` |
| 🔒 `architecture_tool_contract_surface` | caps Tool/RasterEditTool/PanelEvent (§4) |
| 🔒 `architecture_contract_surface` (nodegraph) | caps NodeOp/OpResolver/NodeManifest (§4) |
| `tool_manifest_design_sync` | `docs/design/tools/<slug>.toml` divergente do MANIFEST |
| [`no_tofu_glyphs`](../../crates/ph2d-editor-core/tests/no_tofu_glyphs.rs) | glifos fora da fonte Inter bundled (setas, ⌘, ↵, ✕, ▸ etc.) viram tofu |

**Exceção declarada legítima:** comentário `// LITERAL-COLOR-OK: <razão>` ou `// LITERAL-PX-OK: <razão>` na mesma linha. Coord valida na revisão.

### 5.2 Widget Gallery é a fonte de verdade

[`ph2d-panel-widget-gallery`](../../crates/ph2d-panel-widget-gallery/) (showcase em [`editor-core/src/widget/showcase/`](../../crates/ph2d-editor-core/src/widget/showcase/) + seed em [`pre_populate.rs`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs)) é a **única fonte de verdade da UI**. Todo painel novo **DEVE** usar EXATAMENTE o mesmo padrão de cada widget que aparece no Gallery. Sem "minha variação compacta".

#### Regras herdadas do Gallery (cada uma já queimou ≥1×)

1. **Slider + chip pareados → SEMPRE `store.link_slider_number(slider_id, chip_id)`.** Engata mirror bidirecional automático + clamp `0..1`. Sem o link, painel escreve mirror manual que dessincroniza. Chip e slider compartilham espaço `0..1`; unidade natural ("2.00 clip", "+0.30 brightness") via `display_override` no `paint_slider_with_chip_layout` (paint-only). Veja [`pre_populate.rs:212-231`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs).
2. **Tempo real no canvas — todo slider que altera pixels publica preview por frame.** Tool expõe `take_params_dirty()` + `current_preview()` (ou implementa `RasterEditTool` §3.A.4); bridge em [`render_loop/<tool>_bridge.rs`](../../shells/desktop/src/render_loop/) (espelho de [`bgremoval_preview.rs`](../../shells/desktop/src/render_loop/bgremoval_preview.rs)) refaz cache `Arc<Vec<u8>>` quando dirty, pinta com `vector_scene.draw_image_rgba`, zera em Apply/deactivate. Sem isso = canvas congela = UX inaceitável.
3. **`paint_number_chip` (pill, sem setinhas) ≠ `paint_number_input_with_buffer` (boxed).** Dispatch carve coluna 16-22 px lado direito de TODO `NumberInput` como hit-zone de stepper. Pra chip pill = zona invisível: click direito arma `number_stepper_hold` → incrementa a cada 30ms com cursor parado. **Sempre chame `store.mark_chip_no_stepper(chip_id)` no populate.**
4. **Chip drag = incremental delta**, não absolute-from-Down. Dispatch usa `step_dx = event - last` + `advance_number_input_drag_anchor` por Move. Modelo absoluto pregava valor no bound até cursor voltar até `start_x` — bug invisível.

### 5.3 Anti-padrões UI que já queimaram (NÃO repita)

Bases de conhecimento: [`docs/UI_Bugs/README.md`](../UI_Bugs/README.md) e [`docs/Image Tools Bugs/README.md`](../Image%20Tools%20Bugs/README.md). **Leia antes de tocar em painter/dispatch/tool.**

1. **Estado de MODO e estado DERIVADO não podem viver desacoplados.** Toggle/modo (ex: `image_edit.mode_on`) que governa o que aparece (tool, painel, preview): quem desliga o modo é responsável por **desligar tudo** que ele expõe. Lugar certo = **reconciliação por frame** sobre estado derivado, **não** guard pontual no click.
2. **Enumere TODOS os caminhos de ativação.** Feature costuma ter >1 via de ligar (pill TopBar, tool palette, atalho, bus action). Gatear só uma deixa o bug vivo. Grep TODOS os `set_active`/push de action OU centralize.
3. **Hit-test e paint do MESMO widget têm que ser gateados pela MESMA condição.** Hit-test rodando onde o widget NÃO é pintado = zona de clique invisível. Sempre condicione paint E hit juntos.
4. **Pertencimento é data-driven, não lista de ids hardcoded.** "É image tool?" = está no cluster `"image_tools"` do manifest, resolvido por UM helper (`is_image_edit_tool`) — não `id == "x" || id == "y"` espalhado.
5. **Diagnostique medindo, não chutando.** Bug de UI/input com repro: instrumente (env-gate `PH2D_UIDBG`) e capture estado real antes de propor fix. Reverta a instrumentação no fim.

#### Checklist antes de mergear painel novo (Coord)

- [ ] Cada slider+chip tem `store.link_slider_number(slider, chip)` no populate.
- [ ] Cada chip pill tem `store.mark_chip_no_stepper(chip)` no populate.
- [ ] Storage chip + slider no MESMO espaço (`0..1`); unidade natural só em paint via `display_override`.
- [ ] Se altera pixels: existe `render_loop/<tool>_bridge.rs` espelhando `bgremoval_preview.rs`, refresh em `take_params_dirty()` + overlay via `draw_image_rgba`.
- [ ] `apply_event` é forwarder thin (sem mirror manual slider↔chip).

Faltou → **bounce pro Implementador antes de mergear.** Não "vou abrir exceção".

---

## 6. Codificação rápida

### 6.0 Perfil de máquina — RODE ANTES DE LER O RESTO

> PH2D é desenvolvido em máquinas de potência muito diferente (Mac fraco · Windows fraco ·
> **desktop Linux 128 GB**). A estratégia de velocidade **NÃO é fixa** — é função do hardware.
> **As regras de §6.6 abaixo são o baseline `constrained` (o Mac de 8 GiB).** Seu tier pode
> sobrescrevê-las. Descubra o seu:
>
> ```bash
> bash scripts/hw-profile.sh          # imprime tier + knobs (fonte da verdade)
> ```

| knob | `constrained` (Mac 8 GiB) | `standard` (Windows, a medir) | `workstation` (desktop 128 GB) |
|---|---|---|---|
| cargos paralelos (build) | ≤3 | ~cores/4 | ~cores/6 (ex.: 5) |
| cargos paralelos (check) | ≤3 | ~cores/2 | ~cores/3 (ex.: 10) |
| **rust-analyzer** | ❌ off (RAM-blocked) — `cargo-check-narrow` on-demand | on | **full (RA-as-oracle)** |
| CoW slots (`slot-seed.sh`) | **obrigatório** | opcional | opcional (com 128 GB, dispensável) |
| nextest jobs | ~4 | ~cores/2 | full (= cores) |
| linker | `ld64.lld` (Mach-O) | `rust-lld` | **`mold`** (global, nunca no repo) |
| target/ em tmpfs | não | não | **não — RETIRADO 2026-08-22, medido:** 33 GB de target viraram 30 GB de zram (RAM) e swap a 100%. Disco + `chattr +C`, via `scripts/target-to-disk.sh` ([runbook](../DevOps/BTRFS_METADATA_E_SWAP.md) §2) |
| saúde do disco (btrfs) | — | — | **`bash scripts/btrfs-health.sh`** antes da jornada e no fim de dia — «disco cheio» com 500 GB livres é metadata sem espaço para crescer, e `df` não a vê |
| sccache | não | pilotar | **sim** (global, transparente — não fere paridade-CI) |

- **`constrained`** → siga §6.6 **ao pé da letra** (é o seu tier).
- **`workstation`** → §6.6 fica **sobrescrita** nos pontos acima: use RA como oráculo (não leia saída
  crua do cargo), rode muitos cargos, esqueça o teto de ≤3 e os slots. Setup da máquina em
  [`docs/DevOps/MULTI_MACHINE_SETUP.md`](../DevOps/MULTI_MACHINE_SETUP.md) §3.2. Racional: [ADR-0104](../architecture/decisions/0104-hardware-tiered-speed-strategy.md).
- **`standard`** (Windows) → knobs medidos, ainda **a pilotar** — Linux-benchmarks não transferem
  direto (§6.6.B). Meça antes de virar mandato.
- **O tier também define o MODO de operação** (git + papéis): `workstation` = Modo L (§1.5),
  `constrained` = Modo C (§1.1–1.4 + §7). Vide TL;DR.
- **`target-cpu=native` fica OFF mesmo no `workstation`**: diverge do cache do CI (fere a paridade
  do `ship.sh`) e dá **zero** ganho no inner loop (`cargo check` não faz codegen). Opt-in só pra
  builds de run isolados.

---

**Princípio:** não duplique o pre-commit hook durante editing burst. **Hook ≠ CI** em 2 pontos:

1. **clippy `--all-targets`:** o tier **T2 workspace** roda `cargo clippy --workspace -- -D warnings` **SEM `--all-targets`** (cortado no perf audit 2026-05-19 por velocidade). CI roda completo: `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`.
2. **arch-gates:** mudança estrutural (widget novo, campo serializado) dispara arch-tests que só aparecem após ~3min de compile do hook. Rode o gate do crate antes (§6.1).

**Regra: antes do PUSH, rode `./scripts/ship.sh`** — paridade-CI completa (§8). E `git commit` SEMPRE em background (hook estoura timeout 2min em foreground).

### 6.1 Tabela de validação

| Situação | Comando | Tempo |
|---|---|---|
| Editou 1 arquivo, quer ver se compila | `cargo check -p <crate>` | 3-15s |
| Editou crate, quer rodar testes | `cargo test -p <crate>` | 5-30s |
| Quer rodar UM teste | `cargo test -p <crate> -- <pattern>` | 1-5s |
| Editou foundational, quer ver downstream | `cargo check --workspace` | 30-60s warm |
| Vai commitar (T2 hook vai rodar) | **nada** — deixa o hook validar | 0s |
| **Antes do push (obrigatório)** | `./scripts/ship.sh` | 3-8min warm |
| Só mudou `.md` | **nada** — hook é T0 (skip) | 0s |

### 6.2 LOC threshold (não interrompa o editing burst)

| LOC editados | Comando OK |
|---|---|
| 0-400 | nada, continue |
| 400-1200 | `cargo check -p <crate>` opcional |
| 1200+ ou módulo inteiro | `cargo check -p <crate>` — sane stop |
| Antes do commit | nada — hook valida |

**Não rode `cargo test` durante editing burst.** Só no hook ou em diagnóstico de falha específica.

### 6.3 O que NÃO fazer

- ❌ `cargo test --workspace` depois de cada edit
- ❌ `cargo clippy --workspace --all-targets` a cada COMMIT (SIM antes do PUSH via ship.sh)
- ❌ Re-rodar testes que já passaram pra "confirmar"
- ❌ Validar baseline no início da sessão se último commit já está verde
- ❌ `cargo build` antes de `cargo test` (test já compila)
- ❌ Re-`Read` arquivo que acabou de editar

### 6.4 Pre-commit hook tiered

| Tier | Ativa quando | Tempo |
|---|---|---|
| **T0** | só docs / `.md` / scripts | ~5s |
| **T1** | arquivos de UM crate isolado | ~30s |
| **T2 escopado** | multi-crate **sem** foundational/Cargo.toml/shells | ~30s-3min |
| **T2 workspace** | `Cargo.toml/lock`, foundational, `shells/desktop/` | ~5-15min |

Acidentalmente trigou T2 workspace numa pasta isolada? Provavelmente staged junto com algo de outro agente — confira `git status --cached`.

**Cortes A+B (2026-05-19):** hook NÃO roda `cargo test --doc --workspace` nem `clippy --all-targets`. Esses ficam pro CI. Implicações:
- Doctest novo só verificado em CI. Quem cria valida manual com `cargo test --doc -p <crate>`.
- Benches/examples só clippados em CI.

### 6.5 Como NÃO escrever test slow

**❌ NÃO faça:**
- `TextSystem::new()` — enumera fontes do sistema (25-77s × site). Use `TextSystem::without_system_fonts()`.
- Alloc gigante pra exercitar limit-check (`RgbaImage::new(16384, 16384)` = 1 GiB). Use dimensão 1 px acima do limite (8193×1 = 32 KiB).
- GPU init repetido por test. Use `OnceLock<Option<GpuContext>>` lazy module-level.
- Font shaping real quando só precisa shape de palavra fixa.

**✅ Faça:**
- Setup caro em `OnceLock` lazy, compartilhado entre tests do mesmo binário.
- Input minimal: 1 caso simples + 1 caso edge.
- IO real → `#[ignore]` + `cargo test -- --ignored` no CI separado.

---

### 6.6 Stack de velocidade multi-agente — "agents flying" (2026-05-29)

> **⚠️ Esta seção é o baseline do tier `constrained` (Mac 8 GiB).** Se `scripts/hw-profile.sh`
> (§6.0) disser `workstation`, os tetos abaixo (≤3 cargos, RA-blocked, slots obrigatórios)
> estão **sobrescritos** — leia a tabela de §6.0. O núcleo A (o loop, batching, "audit ≠ compilar")
> vale em TODO tier; só os limites de RAM/concorrência mudam.

Teto de 8 GiB RAM **aceito** (tier `constrained`); concorrência **≤3 agentes**. O gargalo NÃO é
paralelismo — é (a) build/teste **redundante** e (b) tokens queimados em saída
crua do compilador + adivinhação de tipos. Otimiza-se a **velocidade de iteração
por-agente**. Norte: [ADR-0075](../architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)
(monorepo Rust + ECS-decoupling + build-speed; não plugins runtime).

#### A. O loop (núcleo — sempre vale)

1. **Inner loop = SÓ `cargo check -p <crate>`** (ou `scripts/cargo-check-narrow.sh`
   p/ cortar payload de erro). ZERO test/clippy/auditor **por task**.
2. **Validação pesada BATCHED 1× no fechamento do módulo** (não por task): auditoria
   ≥2 lentes rotacionadas + `nextest` + `clippy --all-targets` + smoke, sobre o diff
   ACUMULADO. Padrão-ouro é preservado **no gate**, não repetido a cada commit.
3. **Slot warm por CoW:** `bash scripts/slot-seed.sh <slot>` (clone APFS, ~1s, 0 bytes)
   → **prefixe cada cargo** com o `CARGO_TARGET_DIR` impresso (Bash-tool não persiste
   env). **Não use o `target/` default.** Coordenador rebuilda `target-slots/base`
   (SOZINHO, RAM-heavy) só quando `Cargo.lock`/toolchain muda.
4. **Teste de módulo:** `scripts/nextest-impacted.sh` (só `rdeps()` do que mudou +
   força o golden de determinismo). Gate final = `./scripts/ship.sh`.
5. **Concorrência:** **≤3 `cargo` simultâneos** — Coordenador escalona via
   SESSION_ACTIVE (§1.1). CoW barateia criar slots, NÃO levanta esse teto.

#### B. Alavancas — *arquivada*

A pesquisa de alavancas de build de **2026-05-29** foi **verbatim** para
[`docs/archive/processo-2026-08-18/DIRETRIZ.md`](../archive/processo-2026-08-18/DIRETRIZ.md).
⚠️ **Ela é um veredito sobre um Mac**: o que ela apurou foi que o rust-analyzer full não cabe em
8 GiB, que o linker é o `ld64.lld` do `/opt/homebrew`, e que **"`mold` é incompatível"** — três
frases que, lidas numa workstation Linux com mold e 128 GiB, dizem o **oposto** do que vale aqui.
Os vereditos ainda vivos (Cranelift ❌, `-Zthreads` ❌, `prefer-dynamic` ❌, hakari = medir antes)
estão condensados no [`CLAUDE.md §2`](../../CLAUDE.md) e em §6.0. **Não a leia para decidir stack
nesta máquina** — leia-a só para não re-fazer a pesquisa.

#### C. Anti-padrão que matou a velocidade (não repita)

Mandar cada implementador rodar `cargo test` + `clippy --all-targets` + **spawnar 2
auditores POR TASK** = tempestade de builds redundantes. Auditoria é **por módulo
fechado**, não por micro-task (vide §6.6.A.2).

---

## 7. Git — o que sobrou aqui

**No Modo L a colisão de git não existe:** índice, HEAD, working tree e `target/` são próprios de
cada worktree, e o que resta são os **conflitos de merge (§1.5.5)** e as **proibições (§1.5.6)** —
é lá que se olha, não aqui. O protocolo `stage→commit` do Modo C (nunca `git add -A`, `git status`
antes de estagiar, os sintomas de colisão e a recuperação de cada um) foi **verbatim** para
[`docs/archive/processo-2026-08-18/DIRETRIZ.md`](../archive/processo-2026-08-18/DIRETRIZ.md), e
vale INTEIRO quando a jornada é no Mac — **com a numeração intacta**: *§7.1* (stage→commit),
*§7.2* (proibições), *§7.3* (sintomas de colisão) e *§7.4* (armadilhas) estão lá, com esses números.

Uma armadilha sobrevive aqui porque **morde nos dois modos** — o gate de `typos` roda no
`ship.sh` e no CI, escreva você de que árvore escrever:

**Typos engine bloqueia palavras pt-BR ambíguas.** `erros` (typo de `errors`), `usso` (typo de `use`), `nao` sem acento (typo de `not`). Solução: prefira sinônimos ou use acento; se necessária, adicione exceção em `.typos.toml` `[default.extend-words]` **com justificativa no commit** — nunca esconda um typo real atrás do `--no-verify` do fast mode (§8.1). ⚠️ **A allowlist é uma lista que SOMA entre linhas:** uma chave duplicada mata o gate **no parse**, e ele para de escanear sem falhar — apende ao fim da seção e confira se outra linha tocou o mesmo bloco.
---

## 8. Ship + Push + CI (Modo C: Coordenador absorve PRCI · Modo L: SÓ por ordem explícita do Enio, via o agente integrador — §1.5.4)

### 8.1 Fast mode (dia) vs Ship (fim do dia)

**Princípio: separe "implementar" de "entregar".** Validação completa + CI rodam **1× por jornada**, não 1× por commit.

**De dia — fast mode:**
- Checkpoints com `git commit --no-verify` → instantâneo, pula hook. Salva trabalho, permite reverter.
- `cargo check -p <crate>` só quando quiser confirmar. Sem `--workspace`/test em loop.
- **ZERO push, ZERO CI durante o dia.**

**Fim do dia — ship (Enio dispara: "commit"/"push"/"ship"/"fim do dia"):**
O Coordenador entra em **modo observa-e-corrige** e tem a OBRIGAÇÃO de entregar verde:

1. **`./scripts/ship.sh`** — job de lint+test do CI inteira, local, de uma vez (fmt, clippy `--all-targets --features ph2d-spike/bevy_ecs`, `cargo machete`, `cargo deny`, `cargo audit`, `nextest --workspace`, `typos`). Paridade EXATA com `spike.yml`.
2. Pra CADA `✗`: diagnostica + corrige + re-roda. **NÃO pusha enquanto não estiver 100% verde.**
3. Organiza os checkpoints `--no-verify` do dia em commits limpos (squash se preciso).
4. Push (§8.3) → babysit do CI (§8.4) até verde; em vermelho, fix + re-push até verde (escalona após 3 falhas do MESMO job).
5. Reporta link da run verde ao Enio.

### 8.2 Smoke local — antes do push

```bash
./play.command
```

Smoke é do **Enio**, sob comando do Coord. Coord escreve checklist concreta:

> "Enio, rode `./play.command` e verifica:
> 1. App abre sem panic.
> 2. Tool X aparece na TopBar Image Tools com ícone correto.
> 3. Clique → ação esperada.
> 4. Tools/Actions pré-existentes continuam funcionando.
> 5. Sem regressão visual em Hierarchy / Inspector / Widget Gallery."

### 8.3 Push (Coordenador faz)

Batching: **push UMA vez por jornada**. CI matrix (linux + macOS + windows + replay hash + bench) demora ~30min.

```bash
./scripts/ship.sh    # paridade-CI completa (§8.1)
# Só pusha se ✓
git push origin main
```

### 8.4 Babysit CI

```bash
gh run list --workflow=spike.yml --limit=1 --json databaseId,url
```

Polling **15min** (`gh run watch <id>` ou Monitor com `sleep 900`).

⚠️ **Não espere por um NÚMERO de checks — o `spike.yml` define 5 jobs e quantos deles CORREM
depende do evento e da ref.** Medido no run `32158997562` (push a `main`, `success`): **9 jobs
listados — 8 `success` + 1 `skipped`**, porque `bench` tem `if: github.event_name ==
'pull_request'` e `test`/`determinism` são matrizes de 3 OS. Num push a `feat/**`, `test` também
é pulado e sobra só o `lint`. Um agente esperando *"9/9 success"* espera para sempre: o critério é
**nenhum job em `failure`**, e `skipped` é resultado legítimo.

```bash
gh run view <id> --json jobs -q '.jobs[] | "\(.conclusion)\t\(.name)"'
```

| Resultado | Resposta |
|---|---|
| **Todos os jobs verdes** (nenhum `failure`) | Reporta link + sha bom ao Enio. Ciclo fechado |
| Falha de código | `gh run view --log-failed`, fix local, commit, push, re-watch |
| Falha de infra (cache/network/rustup flaky) | `gh run rerun --failed` + re-watch |
| 3 falhas consecutivas do mesmo job | Escala pro Enio com diagnose |

**Regra de ouro:** fora do babysit, ninguém polla CI. Push, link, próxima tarefa.

### 8.5 Comunicação pós-push

```
✓ Wave <N> pushed. CI run: https://github.com/dibrioli/PH2D/actions/runs/<id>
Entrei em babysit. Reporto quando concluir.
```

E ao terminar:

```
✓ CI verde (todos os jobs, nenhum failure) em <duração>. sha bom novo: <sha>.
Ciclo fechado. Disponível para próxima ordem.
```

---

## 9. Quando algo dá errado

> As linhas com **Coord/Implementador** abaixo são **Modo C** (shared tree). No **Modo L** cada linha isola por worktree — não há Coordenador; vide §1.5.2/§1.5.5.

| Sintoma | Resposta |
|---|---|
| Não sabe o que fazer | Releia §0 + §1 + pergunte ao Enio |
| Arquivo que não tocou em `git status` | §7.3 (colisão) — **Modo C only** (Modo L isola por worktree) |
| Hook falha em fmt/clippy/test | Fix root cause. O `--no-verify` do fast mode (§8.1) salva o **checkpoint**; ele não conserta o **ship** |
| Hook trigga T2 quando esperava T1 | `git status --cached` — vazamento |
| Smoke quebrou em `./play.command` | Implementador (Modo C) / a linha (Modo L) diagnostica + fix local |
| CI failure cíclico (3× mesmo job) | Escala pro Enio (Modo C: Coord; Modo L: quem shippa) |
| Descobre bug fora da pasta | **Modo C:** reporta ao Enio, Coord faz. **Modo L:** a linha corrige (foundational não-contrato, gate testado) OU reporta se contrato-congelado/mesmo-símbolo (§1.5.2.1) |
| Editar shared enquanto outra linha trabalha | **Modo C:** anuncie via Enio, espere estado estável. **Modo L:** N/A — worktrees isolam; só o merge serializa (§1.5.3) |
| Dúvida arquitetural | Opções pro Enio com recomendação + tradeoff |
| Memória diz X mas código diz Y | Confie no código. Atualize memória depois |

---

## 10. Cheat-sheet

### 10.1 Hard Rules CI-gated

| HR | Conteúdo | Gate |
|---|---|---|
| HR-3 | Zero-alloc no dispatcher hot-path | `interaction_dispatch_no_alloc` |
| HR-5 | Determinism cross-platform | CI replay-hash matrix (3 OS) |
| HR-12 | A11y obrigatória | `hr12_widgets_a11y` |
| HR-13 | Memory budget declarado | manifest `memory_budget` |
| HR-15 | Zero hex + zero hardcoded UI string | `no_literal_color` + `hr15_no_hardcoded_ui_strings` |
| HR-18 | `shells/<plat>/src/` ≤ 600 LOC | `file_loc_caps` |
| (Wave 9) | Widget primitive ≤ 500 LOC | `architecture_widget_loc_cap` |
| (Wave 9) | Widget aparece no showcase | `architecture_widget_showcase_coverage` |

Completo em [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18.

#### ⚠️ Os caps de LOC são CINCO, e confundi-los é erro recorrente

Não existe "o cap de 600" nem "o cap de 700": **cada um cobre uma árvore diferente**, e o que vale
para o seu arquivo é o do gate cuja árvore o contém. Duas auditorias já "corrigiram" um pelo outro.
Os números abaixo saem do fonte do gate — se divergirem dele, **o fonte está certo**:

| Árvore coberta | Cap | Gate (a fonte do número) |
|---|---:|---|
| `crates/**` (workspace inteira) | **700** | `ph2d-editor-core/tests/architecture_workspace_file_loc_cap.rs` (`FILE_LOC_CAP`) — ADR-0105 subiu 600→700; existentes acima ficam **congelados** numa allowlist (podem encolher, nunca crescer) |
| `shells/<plat>/src/**` | **600** | `shells/desktop/tests/file_loc_caps.rs` (`FILE_LOC_CAP`) — HR-18 |
| `ph2d-panel-*/src/**` | **600** arquivo · **200** função | `ph2d-editor-core/tests/architecture_panel_loc_cap.rs` (`PANEL_FILE_LOC_CAP` / `PANEL_FN_LOC_CAP`) |
| `ph2d-editor-core/src/widget/**` | **500** | `ph2d-editor-core/tests/architecture_widget_loc_cap.rs` (`WIDGET_LOC_CAP`) |
| `ph2d-tool-runtime/src/**` | **650** | `ph2d-tool-runtime/tests/architecture_runtime_loc_cap.rs` (`CAP`) |

⚠️ **Cap de FUNÇÃO e cap de ARQUIVO são grandezas diferentes:** extrair um corpo grande para uma
função irmã **no mesmo arquivo** cura o de função e estoura o de arquivo. Corte para o **arquivo
irmão** ([[feedback_a_fn_cap_and_a_file_cap_measure_different_things]]), e rode `cargo fmt` **antes**
de medir — a re-quebra de linhas do fmt reexpande o arquivo depois do corte.

### 10.2 Caminhos canônicos

| O que | Onde |
|---|---|
| **Node crate** (fan-out (A)) | `crates/ph2d-node-<dom>-<slug>/` |
| **Tool crate** (fan-out (A)) | `crates/ph2d-tool-<slug>/` |
| Painel (caminho (B)) | `crates/ph2d-panel-<slug>/` |
| Widget primitive (caminho (B)) | `crates/ph2d-editor-core/src/widget/<slug>.rs` |
| Chrome handler (caminho (B)) | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` |
| Vocab UI de um tool | `crates/ph2d-tool-<slug>/src/params.rs` |
| 🔒 Contrato de nós | `crates/ph2d-nodegraph/` + `crates/ph2d-expr/` |
| 🔒 Contrato de tools | `crates/ph2d-editor-core/src/tool.rs` + `action_bus.rs` |
| **Tool registry (GERADO)** | `crates/ph2d-tool-registry-init/` |
| **Node registry (GERADO)** | `crates/ph2d-node-registry-init/` |
| Panel registry (manual) | `crates/ph2d-panel-registry-init/src/lib.rs` |
| Codegens | `tools/ph2d-{node,tool,panel,chrome,widget}-sync/` |
| Widget showcase | `crates/ph2d-editor-core/src/widget/showcase/` |
| Tokens source | `docs/design/tokens.json` → build.rs gera `crates/ph2d-tokens/src/` |
| Tool design TOML | `docs/design/tools/<slug>.toml` |
| Icon SVG | `docs/design/icons/<slug>.svg` |
| Shell init | `shells/desktop/src/init.rs` |
| Arch tests editor | `crates/ph2d-editor-core/tests/` |
| Arch tests contrato tool 🔒 | `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` |
| Arch tests contrato nodegraph 🔒 | `crates/ph2d-nodegraph/tests/architecture_contract_surface.rs` |

**Removido em ADR-0040 TG-D (`c4063b7`):** `crates/ph2d-editor-core/src/tools/` foi **deletado**. Foundation ⊥ tools gateado. Memória/doc apontando lá = stale.

### 10.3 Comandos mais usados

```bash
# Implementador — durante edição
cargo check -p ph2d-<family>-<slug>
cargo test  -p ph2d-<family>-<slug>
cargo test  -p ph2d-<family>-<slug> -- some_pattern

# Drop-crate fan-out — Implementador roda após criar a pasta
cargo run  -p ph2d-<family>-sync          # regenera wiring
cargo test -p ph2d-<family>-registry-init # staleness fecha

# Coordenador — antes do push (paridade-CI completa, obrigatório)
./scripts/ship.sh

# Coordenador — push + babysit
git push origin main
gh run list --workflow=spike.yml --limit=1
gh run watch <id> --exit-status
```

---

## 11. Referências canônicas

- **Stack + Hard Rules + "Adicionar uma tool":** [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md)
- **Operacional dia-a-dia + CI:** [`CLAUDE.md`](../../CLAUDE.md)
- **Exemplos fan-out 100% paste-ready:** [`examples-fan-out.md`](examples-fan-out.md)
- **Tracker do fan-out de nodes — HISTÓRICO, não estado atual:** [`HANDOFF_node_system.md`](../archive/handoffs-2026-06-16/HANDOFF_node_system.md) (o estado vivo dos módulos é o `CLAUDE.md §5`)
- **Plano de nodes (W1+W2 fechados, W3+ aberto):** [`docs/plans/2026-05-node-waves.md`](../plans/2026-05-node-waves.md)
- **Plano Wave 11 carry-overs:** [`docs/plans/2026-05-wave-11-carry-overs.md`](../plans/2026-05-wave-11-carry-overs.md)

**ADRs estruturais (leitura indispensável):**

- [ADR-0027 Convention-by-discovery](../architecture/decisions/0027-convention-by-discovery.md)
- [ADR-0029 Trait-driven panel host](../architecture/decisions/0029-trait-driven-panel-host.md)
- [ADR-0030 Multi-domain node engine](../architecture/decisions/0030-multi-domain-node-engine.md)
- [ADR-0031 Node E tool como unidade de feature](../architecture/decisions/0031-node-and-tool-as-feature-unit.md)
- [ADR-0032 `ph2d-nodegraph` substrato](../architecture/decisions/0032-nodegraph-substrate.md)
- [ADR-0033 `ph2d-expr` shared compute](../architecture/decisions/0033-shared-compute-expr.md)
- 🔒 [ADR-0039 Nodegraph contract FREEZE](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)
- 🔒 [ADR-0040 Tool as isolated feature crate](../architecture/decisions/0040-tool-as-isolated-feature-crate.md)
- 🔒 [ADR-0041 RasterEdit rename + deactivate](../architecture/decisions/0041-rasteredit-rename-and-deactivate.md)
- [ADR-0042 Wave 10 closure](../architecture/decisions/0042-wave-10-closure.md)
- [ADR-0104 Estratégia de velocidade por hardware](../architecture/decisions/0104-hardware-tiered-speed-strategy.md)
- 🔒 [ADR-0106 Linhas paralelas por `git worktree` (Modo L)](../architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)
- 🔒 [ADR-0107 Foundational concorrente — gate testado + Mergiraf](../architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)
- [GUIA_JORNADA_MODO_L (companheiro do operador ao §1.5)](GUIA_JORNADA_MODO_L.md)

**Memória LLM (versionada no repo):** [`project-memory/MEMORY.md`](../../project-memory/MEMORY.md) (auto-loaded via symlink `~/.claude/projects/<key>/memory` → `project-memory/`)

**Histórico anterior** (v6.0..v6.10): `git log docs/IntegracaoMultiAgente/DIRETRIZ.md`. Arquivados pre-v6.0: `docs/archive/multi-agente-pre-v6.0/`.

---

## 12. Quem mantém estes docs

**Doc único é falso: a regra é UMA porta por regra + ponteiro.** *(Esta seção mandava o contrário
— "não fragmente em múltiplos docs" — enquanto a própria pasta tinha nove; o que dessincroniza não
é ter vários arquivos, é a **mesma regra escrita em dois deles**, porque a próxima emenda acerta um
e deixa o outro mentindo em silêncio. A cura é escrever a regra **uma vez** e apontar de todo lugar
que precisa dela.)*

**Quem atualiza:** ⚠️ **a linha que muda um mecanismo do processo atualiza o doc de processo no
MESMO commit** — se o mecanismo mudou e o doc não, a próxima linha executa o mecanismo antigo, e
descobre na integração. E o **handoff de integração (§1.5.9) lista o que mudou aqui**, para o
integrador não fundir uma regra nova por cima de outra regra nova sem reparar.

**Diretriz contradiz o código:** **confie no código**, reporte ao Enio com diagnose, corrija o doc
quando autorizado.
