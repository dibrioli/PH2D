# Diretriz de Implementação — PH2D

**Versão:** 8.2 — 2026-07-07 (**integração + ship = ordem EXPLÍCITA do Enio, nunca autônomos**: no Modo L a linha fecha o módulo, escreve o **handoff de integração** (§1.5.9) e PARA; o Enio junta os handoffs e abre **um agente integrador dedicado** que resolve todos os conflitos (§1.5.3–1.5.4). Reforçado: **foundational é editável, mas ao CRIAR arquivo foundational projete-o para isolamento** (§1.5.2.1). Só edições pontuais — o mecanismo (`foundational-integrate.sh` + `--ff-only` + Mergiraf) é o mesmo. Baseline: 8.1 — 2026-07-05 — **foundational deixou de ser serial no Modo L** — [ADR-0107](../architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md): qualquer linha toca foundational sob 3 camadas — pontos de extensão append-only, Mergiraf (merge sintático) no resíduo textual, e o **gate da árvore combinada** `scripts/foundational-integrate.sh` no resíduo semântico; só contrato congelado (§4) e mesmo-símbolo de tipo-núcleo seguem seriais. v8.0 — 2026-07-05: **o modo de operação virou função do hardware** — [ADR-0106](../architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md); `bash scripts/hw-profile.sh` decide: tier `workstation` (Linux 128 GB) = **Modo L**, linhas paralelas por `git worktree` sem Coordenador de plantão, §1.5; tier `constrained` (Mac mini 8 GiB, sessões de smoke/hotfix) = **Modo C**, o modelo v7.1 de 1 Coordenador único + N Implementadores em shared tree, §1.1–1.4 + §7. Baseline anterior: 7.1 — 2026-05-28, papéis consolidados).
**Audiência:** **toda LLM que entra no projeto.** Este doc é **referência** — **NÃO
leia inteiro**; use o roteador leia-por-tarefa em [`CLAUDE.md §1`](../../CLAUDE.md) e
leia só a(s) seção(ões) que sua tarefa exige. Obrigatório p/ todos: §0 (sanity), §1
(papéis), §2 (triagem), §6 (velocidade), §7 (git). O resto é por-tarefa.

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
cargo check --workspace 2>&1 | tail -5    # baseline compila?
```

Algo divergente (HEAD inesperado, working dirty, build quebrado) → **pare e reporte ao Enio.**

**Leitura mínima:**
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18 (Hard Rules) e §1 (arquitetura).
- [`CLAUDE.md`](../../CLAUDE.md) (CI, push, batching).
- Memória persistente (versionada no repo): [`project-memory/MEMORY.md`](../../project-memory/MEMORY.md) (symlink de `~/.claude/projects/<key>/memory`).

---

## 1. Papéis + infra multi-agente

> **Todo o §1 (papéis abaixo) + §1.1–1.4 descrevem o Modo C** (tier `constrained` — Mac mini
> 8 GiB): o "Coordenador (único)" só existe no Modo C. No tier `workstation` vale o **Modo L
> (§1.5)** — SEM Coordenador de plantão; a infra anti-colisão (slots CoW, arbitragem de posse,
> índice compartilhado) é substituída por isolamento físico de `git worktree`.

**Coordenador (único).** Um só por jornada. Absorve o que antes eram Coord-A (foundational) e Coord-B (baldes). Autoridade **exclusiva** sobre: contratos congelados, arch-gates, foundational crates (`ph2d-render`, `ph2d-editor-core`, `ph2d-host`, `ph2d-tokens`, …), codegen tools, `shells/*` plumbing compartilhado, scaffolds de painel/widget/chrome, ADRs, `CLAUDE.md`/DIRETRIZ, `.github/workflows/`. É o **único** que toca arquivo foundational/compartilhado — isso serializa a superfície de colisão (causa-raiz dos incidentes que motivaram o modelo). Mexe nos 2 contratos congelados (§4) só via amendment ADR, nunca cap-bust ad-hoc. Responsabilidades do modelo multi-implementador:
- (a) escrever um **sub-handoff focado por implementador** (estado + pasta exclusiva + task + anti-colisão);
- (b) manter o **mapa de posse** em SESSION_ACTIVE (§1.1) — quem é dono de quê;
- (c) **arbitrar colisões** e **sequenciar dependências** entre implementadores (ex.: liberar `ph2d-render` ao módulo B só quando o módulo A soltar);
- (d) **ship-de-jornada** (ship.sh + commit + push + babysit CI — §8), incluindo limpar fmt-drift e ship-blockers cross-session no fim.

Não implementa feature de módulo — **coordena**.

**Implementador (sempre vários).** Sessão isolada, **uma por módulo físicamente disjunto** (uma crate-pasta ou um cluster de crates do mesmo módulo). Caminho **(A)**: cria pasta + roda sync + testa, sem Coordenador. Caminho **(D)**: edita dentro de pasta de módulo existente. Caminho **(B)**: recebe pasta já scaffoldada pelo Coordenador, edita **só** dentro dela. A não-colisão é garantida pela arquitetura física (glob `workspace.members` + codegen splice em marcadores) **somada** à regra de posse exclusiva arbitrada pelo Coordenador. **Precisou de QUALQUER coisa fora da sua pasta** (foundational, shell plumbing, contrato congelado, outro módulo)? **PARA e reporta ao Coordenador** — não edita, e **nunca renegocia direto com outro implementador**.

**Enio.** Humano que orquestra: abre sessões Claude Code, cola mensagens entre elas, roda smoke visual quando Coord pede. **Não decide nada operacional.**

### 1.1 Protocolo SESSION_ACTIVE (mapa de posse mantido pelo Coordenador)

[`docs/SESSION_ACTIVE.md`](../SESSION_ACTIVE.md) é o post-it vivo da orquestração. **Só o Coordenador escreve;** os implementadores **leem antes de cada burst** e não editam. O Coordenador mantém ali:

1. O **mapa de posse**: qual implementador é dono de qual pasta/módulo (escrita exclusiva) + seu slot.
2. Os **pontos compartilhados** e como estão resolvidos (ex.: crate X é escrita do Impl-N, leitura dos demais).
3. Os **itens que o Coordenador segura** (ship-blockers, foundational, sequenciamento de dependências).
4. **Pre-existing failures cross-session** a NÃO fixar (com owner identificado).

Implementador que precise tocar pasta fora da sua: **PARA e reporta ao Coordenador** — nunca renegocia direto com outro implementador. O Coordenador limpa os itens concluídos ao encerrar a jornada.

### 1.2 Isolamento físico — `scripts/slot-env.sh`

Cada sessão roda `source scripts/slot-env.sh <slot-id>` no início para isolar `CARGO_TARGET_DIR` por slot. Sem isso, dois agentes paralelos serializam no lock de `target/`. Slot IDs: `coord` + um por implementador nomeado pelo módulo (`impl-sprite`, `impl-painter`, `impl-vector`, …).

**RAM 8 GiB → máximo realista = 2-3 slots cargo-ativos simultâneos.** Com N implementadores, isso NÃO autoriza N cargos simultâneos: o Coordenador **escalona quem compila quando** (lê SESSION_ACTIVE). 4º cargo ativo causa swap thrashing.

*(Modo L: slots dispensáveis — cada worktree já tem `target/` próprio; vide §1.5.1.)*

### 1.3 Anti-colisão git — `scripts/git-stage-guard.sh`

Pre-commit roda o guard que **rejeita stage fora da pasta declarada** (env `PH2D_SLOT_FOLDER`). Coords legítimos exportam `COORD_OVERRIDE=1` na sessão pra bypass. Padroniza a disciplina §7 sem depender de memória humana.

### 1.4 As 3 obrigações do Implementador (sempre)

1. **ISOLAMENTO.** Edita **só** dentro da pasta exclusiva. Precisa algo fora? **Reporta** — não edita.
2. **UI canônica.** Toda cor/espaço/raio/tipografia/stroke passa por tokens. Zero hex, zero `f32` literal de UI (§5).
3. **Codificação rápida.** `cargo check -p <crate>` no editing burst. Sem `--workspace` em loop (§6).

Pra violar uma? **Pare e reporte.** Quase certo o Coord não fez scaffold direito.

### 1.5 Modo L — linhas paralelas por `git worktree` (tier `workstation`)

> Ativa quando `scripts/hw-profile.sh` = `workstation`. **N linhas de desenvolvimento = N
> worktrees + N branches** (`line/<módulo>`), cada uma numa sessão Claude Code própria;
> `main` vira só ponto de integração. O worktree elimina a colisão de **git** de raiz
> (índice, HEAD, working tree e `target/` próprios por linha — a classe inteira de
> incidentes que o §7 legisla vira impossibilidade física); a **pasta disjunta**, que já
> era regra, elimina o conflito de **merge** nos drop-crates. Juntos = integração fast-forward
> na prática, **sem Coordenador de plantão** — mas **a integração e o ship são disparados por
> ordem EXPLÍCITA do Enio** (nunca autônomos): cada linha fecha o módulo, escreve o **handoff de
> integração** (§1.5.9) e PARA; o Enio junta os handoffs e abre **um agente integrador dedicado**
> que resolve TODOS os conflitos (§1.5.3). As 3 obrigações do §1.4 valem dentro de cada linha,
> **com uma emenda (ADR-0107): a obrigação 1 (ISOLAMENTO) NÃO proíbe mais foundational no Modo L**
> — foundational é editável por qualquer linha sob o protocolo testado (1.5.2.1 + 1.5.3), **mas
> a própria foundation tem arquitetura de isolamento de propósito: ao CRIAR arquivo foundational
> novo, projete-o para várias linhas o estenderem sem colidir** (§1.5.2.1, último parágrafo);
> permanecem-fora-de-limite só os contratos congelados (§4) e mesmo-símbolo de tipo-núcleo.
> Triagem §2, receitas §3, contratos §4, UI §5 e a DIRETIVA_IMPLEMENTACAO idem.
> **Proibido no tier `constrained`** — N worktrees × `target/` não cabem em 8 GiB (vide 1.5.6).

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
2. Commits locais frequentes (`--no-verify` de dia, fast mode §8.1). **NUNCA push.**
   **NUNCA integre nem faça ship por conta própria** — quem funde as linhas é o **agente
   integrador dedicado**, e só por ordem EXPLÍCITA do Enio (1.5.3–1.5.4). Você fecha a linha,
   entrega o handoff (§1.5.9) e espera.
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

**Um comando faz tudo:** de dentro da worktree da linha, com o gate batched do módulo verde:

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
| Ship + push + babysit CI (§8) | **SÓ por ordem EXPLÍCITA do Enio** (nunca autônomo — nem "o último apaga a luz"): quando o Enio manda "ship/push", o **agente integrador** (ou uma sessão-ship dedicada que ele abre) roda `./scripts/ship.sh` + push + babysit, 1× por jornada sobre o main integrado. Uma linha/agente que integra ou pusha sem ordem explícita **violou o protocolo** ([[feedback_ship_only_enio_end_of_all_lines]] + [[feedback_integration_only_enio_command_end_of_all_lines]]) |

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
| SESSION_ACTIVE / `CLAUDE.md §5` / trackers | Só na integração, no primário, uma linha por vez; cada linha edita só o SEU `HANDOFF_*` |
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

**Agente NOVO numa linha que já existe** (troca de janela de contexto, ou retomada depois de a linha ter integrado): o bloco é OUTRO — [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](MODELO_TROCA_DE_AGENTE_NA_LINHA.md). A worktree já existe (não se cria), e o risco específico dessa troca é o agente trabalhar no **primário** em vez da linha: toda janela abre na raiz, que está em `main`, e **o mesmo path relativo existe nas duas árvores** — editar `crates/…` da raiz compila e commita sem erro nenhum, e só aparece na integração. Por isso aquele bloco começa por `cd` + `pwd` + `git branch --show-current` **antes de ler qualquer arquivo**, e traz o procedimento de resgate para quando já escreveram no main.

Você é `workstation`, sem bloco colado, e `git branch --show-current` devolve `main`?
Você é uma sessão do **primário** (setup/integração/ship) — **não code em `main` no
Modo L**; pergunte ao Enio qual é a sua linha.

#### 1.5.9 Handoff de integração (cada linha entrega; o Enio passa ao integrador)

Antes de integrar, o **Enio pede a cada linha um handoff de integração** — o documento que
passa ao **agente integrador** os pontos que evitam conflito/regressão. A linha **NÃO integra
nem faz ship**; entrega este handoff e espera. Conteúdo mínimo (curto, factual):

1. **Identidade:** branch `line/<módulo>`, HEAD, base do fork (merge-base com main), nº de commits.
2. **Foundational/compartilhado tocado + por quê** — todo arquivo fora da sua pasta de módulo
   (ex.: `editor-core`, `ph2d-core`, `shells/*`, `tokens`), aditivo ou não.
3. **Símbolos que podem COLIDIR com outra linha** — ids/consts/variants/tokens novos com seus
   valores literais (ex.: `NodeId(832)`, variant de enum, entrada em lista ordenada, chave de
   token). É o que o integrador grepa pra detectar mesmo-símbolo (§1.5.5).
4. **Contratos congelados encostados** (§4) — deve ser **nenhum**; se sim, exige ADR (pare e reporte).
5. **O que só o `ship.sh` pega** (o gate de integração NÃO roda): fmt/typos pré-fork, deps novas
   p/ machete, clippy latente, RUSTSEC ([[project_integration_prefork_lines_ship_drift]]).
6. **Ordem/dependências** entre commits, se houver, e **o que smoke-testar** (o que NÃO foi smokado).

Modelo de resumo no fim da linha: *"Linha `<módulo>` pronta (HEAD `<sha>`, N commits). Handoff
de integração: <itens 2–6>. Aguardo ordem de integração."*

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
| **Avaliador novo (Wave-neck)** — Shader/Som/Gameplay | **(C)** durante neck → (A) depois | Trabalho "tipo W2" serial; abre fan-out só após o neck. Tracker em [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md). |
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

Sub-trait com 5 métodos (`set_source` / `current_preview` / `take_pending_commit` / `run_full` / `deactivate`), congelado em ADR-0041. **3 tools de produção implementam** (BgRemoval, Color Equalization, Upscale). Padding e Equalize Sizes são exceção documentada (geométrico-only / multi-sprite-required).

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
| target/ em tmpfs | não | não | **sim** (`scripts/target-on-tmpfs.sh`) |
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

#### B. Alavancas (deep-research 2026-05-29, verificada 3-votos; run `wf_8d23212a-39e`)

Separadas por confiança. Linux-benchmarks **não transferem direto** pro Apple Silicon
8 GiB → as marcadas "pilotar" exigem medição local antes de virar mandato.

**🥇 Tier 1 — diagnóstico via LSP (MEDIDO 2026-05-29 — leia o veredito):**
- **Diagnóstico via LSP, não saída crua.** O agente NÃO deve ler output bruto do cargo
  e adivinhar tipos (desperdício de token + erro).
- **⛔ FULL rust-analyzer / MCP type-query = BLOQUEADO por RAM nesta máquina.** Spike de
  medição: 8 GiB físicos, **swap 5187/6144 MiB usados, ~89 MiB livres** com editores +
  1 agente e os rust-analyzers dos editores **dormentes (3 MiB)**. Um RA *indexando* o
  workspace (~30 crates wgpu/vello/bevy) custa ~1.5–4 GB → **não cabe nem ×1, quanto mais
  ×3**. Só viável num Mac de 32 GB (dispensado). NÃO adotar rust-analyzer-as-oracle / MCP
  de tipo aqui.
- **✅ Caminho viável nesta máquina = `scripts/cargo-check-narrow.sh` ON-DEMAND.** O
  agente checa quando quer, recebe só os erros (corta tokens), **zero processo residente**.
  É o Tier-1 prático no teto de 8 GiB.
- **⚠️ `bacon-ls` (backend cargo) — pesar, não adotar cego:** dá diagnósticos via LSP sem
  o índice do RA, MAS roda `cargo check`/clippy **continuamente** em background; com ≤3
  agentes = 3 loops de check contínuos = pressão constante numa máquina já em swap. Pode
  ser PIOR que o check on-demand. Só vale se medido folgado.
  *(MCP de terceiros rust-mcp/cursor-rust-tools = EXPERIMENTAL hobby E type-query = RAM-blocked.)*

**🥈 Tier 2 — build/test loop (PILOTADO 2026-05-29: já capturado, ver status):**
- **Linker rápido = ✅ JÁ ATIVO.** `~/.cargo/config.toml` global usa
  `-fuse-ld=/opt/homebrew/bin/ld64.lld` (lld para Mach-O) — corta ~30-50% do link
  incremental. `mold` é **incompatível com macOS** (ELF-only, erro fatal) — não usar.
- **Redução de debug-info = ✅ JÁ NO GATE.** `[profile.ci-test] debug = false`, e o gate
  (`nextest-impacted.sh` + `ship.sh`) roda `--cargo-profile ci-test` → debug-info já
  cortado onde importa. O `[profile.dev] debug = true` só afeta `cargo check` (irrelevante —
  não linka) e builds ad-hoc (que evitamos).
- **`prefer-dynamic` (dynamic-linking) = ❌ NÃO adotar.** Só ajuda LINK (gate infrequente),
  não o inner loop (check). Com lld + debug=false já ativos, o link deixou de ser dominante
  → ganho marginal. Custo: mudar RUSTFLAGS invalida a **base CoW warm** (rebuild completo) +
  quirks de prefer-dynamic no macOS. Net-negativo nesta máquina. (O ~5× do `bevy_dylib` é da
  feature whole-Bevy, que não usamos — só `bevy_ecs` standalone.)

**🥉 Tier 3 — pilotar + medir (ganho real M-series incerto):**
- **`cargo-hakari`** (workspace-hack): mata a **cascata de recompile por feature-unification**
  ("check ganha mais que build"; até 100× em comando isolado, ~1.7× cumulativo em Linux).
  Custo: crate central novo (acoplamento leve) + entrada em `cargo-machete` ignore. Medir
  ganho no nosso slot CoW antes de adotar.

**🚫 NÃO fazer (achados contrários verificados):**
- **Cranelift:** irrelevante ao inner loop (check já não faz codegen); no macOS unwinding
  de panic **não-suportado** (força `-Cpanic=abort`) + `std::arch` SIMD parcial = ruim p/
  wgpu/vello/rapier. Experimental.
- **`mold`:** incompatível com macOS.
- **`-Zthreads`** (frontend paralelo): não-provado em RAM-bound; aumenta pico de memória.

#### C. Anti-padrão que matou a velocidade (não repita)

Mandar cada implementador rodar `cargo test` + `clippy --all-targets` + **spawnar 2
auditores POR TASK** = tempestade de builds redundantes. Auditoria é **por módulo
fechado**, não por micro-task (vide §6.6.A.2).

---

## 7. Anti-colisão git (Modo C — shared tree)

> **Modo L:** esta seção inteira descreve o problema que o worktree elimina — cada linha tem
> índice/HEAD/tree próprios. No Modo L valem só as tabelas 1.5.5 (conflitos de merge) e
> 1.5.6 (proibições). No Mac (Modo C), esta seção vale INTEIRA.

`git commit` é serializado pelo índice global do git. Duas sessões com arquivos staged ao mesmo tempo: uma roda commit e agarra os arquivos da outra junto.

### 7.1 Protocolo atômico stage→commit

```bash
# 1) Antes de stage: confira working tree
git status
#    Há M/?? que não são seus? PARE. Outro agente em vôo.

# 2) Stage só os seus. NUNCA -A / -a / git add .
git add <arquivos-específicos>

# 3) Antes de commit: confere índice
git status --cached
#    Arquivo que não estagiou? Vazamento.
#    git restore --staged <não-meus>

# 4) Commit. Hook tiered roda automaticamente.
git commit -m "<descrição em inglês, imperativo, <70 char>"
```

Stage→commit é **operação contínua**. Não pause entre os dois passos.

### 7.2 Proibições

- **Nunca** `git push --force` em main
- **Nunca** `--no-verify` (se hook falha, fix root cause)
- **Nunca** `git commit --amend` (sempre novo commit)
- **Nunca** `git config` mudando settings do repo
- **Nunca** `git restore --staged --worktree` em path fora da sua pasta sem coordenar

### 7.3 Sintomas de colisão

| Sintoma | Recuperação |
|---|---|
| `fatal: cannot lock ref 'HEAD'` no commit | Outra sessão commitou no meio. `git status` → diagnose |
| `git status` mostra M que você não tocou | Outro agente paralelo. NÃO comite. Reporte |
| `git log -1` mostra mensagem fundida (2 títulos) | Colisão. Se NÃO pushado: `git reset --soft HEAD~1` + split + recommit |
| Hook trigga T2 quando esperava T1 | `git status --cached` — vazamento de outro agente |

### 7.4 Armadilhas conhecidas

**Typos engine bloqueia palavras pt-BR ambíguas.** `erros` (typo de `errors`), `usso` (typo de `use`), `nao` sem acento (typo de `not`). Solução: prefira sinônimos ou use acento; se necessária, adicione exceção em `.typos.toml` `[default.extend-words]` **com justificativa no commit**, não esconda com `--no-verify`.

**Cargo lock entre sessões.** Se rodar `cargo` enquanto outra sessão Claude Code paralela está rodando, a 2ª **espera silenciosamente** pelo lock. Não é crash, só lentidão. Use `slot-env.sh` pra isolar (§1.2).

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

| Resultado | Resposta |
|---|---|
| Success 9/9 | Reporta link + sha bom ao Enio. Ciclo fechado |
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
✓ CI verde 9/9 em <duração>. sha bom novo: <sha>.
Ciclo fechado. Disponível para próxima ordem.
```

---

## 9. Quando algo dá errado

> As linhas com **Coord/Implementador** abaixo são **Modo C** (shared tree). No **Modo L** cada linha isola por worktree — não há Coordenador; vide §1.5.2/§1.5.5.

| Sintoma | Resposta |
|---|---|
| Não sabe o que fazer | Releia §0 + §1 + pergunte ao Enio |
| Arquivo que não tocou em `git status` | §7.3 (colisão) — **Modo C only** (Modo L isola por worktree) |
| Hook falha em fmt/clippy/test | Fix root cause; nunca `--no-verify` |
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
- **Tracker vivo do fan-out de nodes:** [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md)
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

## 12. Quando esta diretriz fica obsoleta

Se a arquitetura mudar materialmente (3º papel surge, fluxo invertido vira lateral, contrato 3 surge), atualize **in-place** e bump versão. **Não fragmente em múltiplos docs** — lição dos 4 docs antigos que dessincronizaram é que doc único é mais fácil de manter.

LLM lendo isto depois de mudança arquitetural maior e diretriz contradiz código: **confie no código**, reporte ao Enio com diagnose, atualize quando autorizado.
