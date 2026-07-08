# Guia — Como tocar uma jornada Modo L (do seu lado, Enio)

> **O que é:** o passo a passo do **operador humano** numa jornada multi-agente no
> desktop Linux (`workstation`). Do abrir-janelas ao ship. **Sem coordenador de plantão** —
> cada linha é autônoma; mas a **integração e o ship são decisão SUA (ordem explícita)**,
> executados por um **agente integrador dedicado** munido do handoff de cada linha (ADR-0106 + ADR-0107).
>
> **Não duplica** os outros docs — aponta pra eles:
> - o **bloco** que você cola: [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md)
> - o **protocolo** que o agente segue: [`DIRETRIZ.md §1.5`](DIRETRIZ.md)
> - o **porquê**: [ADR-0106](../architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md) (linhas) · [ADR-0107](../architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md) (foundational concorrente)

## Pré-requisito (1×)

`bash scripts/hw-profile.sh` tem que dizer **`workstation`**. Se disser `constrained`,
esta máquina é **Modo C** (Coordenador + Implementadores, shared tree) — este guia não
vale; veja DIRETRIZ §1.1–1.4. Modo L é só no desktop 128 GB.

## Papéis: só você + N linhas. Zero coordenador.

| Quem | Faz |
|---|---|
| **Você (operador)** | Decide os módulos/tarefas disjuntos, abre as janelas, cola o bloco, dá a tarefa. Arbitra só os 2 casos irredutíveis (abaixo). **No fim: coleta o handoff de cada linha, abre o agente integrador e manda o ship — tudo por ordem sua.** |
| **Cada linha** (1 janela = 1 agente) | Trabalha na worktree dela, roda `cargo check -p` no loop, fecha o gate batched, **escreve o handoff de integração (DIRETRIZ §1.5.9) e PARA** — reporta "linha pronta + handoff" e espera. NÃO integra nem faz ship sozinha. |
| **Agente integrador** (1 janela, no primário, quando você mandar) | Recebe os handoffs, resolve TODOS os conflitos e funde cada linha via `foundational-integrate.sh` (`--ff-only` + gate testado); o ship só depois, também por ordem sua. |

Não há Coordenador de plantão — a não-colisão é por construção (worktree isola git, pasta
disjunta isola merge, Mergiraf + gate testado cobrem foundational).

## A jornada em 4 passos

### 1. Planeje (triagem sua)
- Liste as tarefas da jornada e **agrupe por módulo/pasta disjunta** — idealmente cada
  linha mexe numa área diferente (`crates/ph2d-tool-x/`, `crates/ph2d-node-y/`, painel z…).
- **Quantas linhas ao mesmo tempo?** É função do hardware — o `hw-profile.sh` sugere; no
  `workstation` várias voam. Comece com 3–4 se estiver calibrando.
- **Foundational agora pode entrar numa linha** (ADR-0107) — não precisa mais de uma fila
  única. Só **contrato congelado** (nodes/tools, §6) você sequencia à parte (exige ADR).
- Confira quem já está aberto: `git worktree list` (ou [`SESSION_ACTIVE.md`](../SESSION_ACTIVE.md)).
  **Nunca 2 linhas no mesmo módulo.**

### 2. Abra cada linha
Para cada linha, **uma janela nova** do Claude **na raiz do repo** (sempre a mesma pasta):
1. Copie o bloco inteiro de [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md).
2. Escreva o módulo **só na 1ª linha** (`Sua linha: line/<módulo>` — 1 palavra kebab-case).
3. Cole como **1ª mensagem**. O agente monta a worktree, roda o setup e responde
   **"Linha pronta. Aguardo a tarefa."**
4. Mande a **tarefa** na mensagem seguinte (o que construir + em qual pasta `crates/…`).

Repita por linha. Cada janela é independente; podem rodar em paralelo.

### 3. Durante a jornada — você quase não intervém
- Cada linha commita local (`--no-verify`, fast mode), fecha o gate batched, **escreve o
  handoff de integração (DIRETRIZ §1.5.9) e reporta "linha pronta + handoff"** — e PARA.
  **Ela NÃO integra sozinha:** a fusão é depois, pelo agente integrador, por ordem sua.
- **Você só age quando um agente te REPORTA** (os 2 casos irredutíveis):

  | Agente reporta | O que você faz |
  |---|---|
  | **Contrato congelado** (cap em `Tool`/`RasterEditTool`/`PanelEvent` ou `NodeOp`/`OpResolver`/`NodeManifest`) | Decide o ADR (amendment). Sequencie: uma linha mexe no contrato + ADR, as outras rebaseiam depois. |
  | **Rebase conflita fora dos arquivos do módulo** (mesmo-símbolo de tipo-núcleo) | Duas linhas reescreveram a mesma função/assinatura. Você decide a ordem/quem cede; não é merge, é design. |

  Fora esses dois, **não faça nada** — a linha resolve (incl. foundational).

### 4. Feche — você dispara a integração e o ship (nunca a linha sozinha)
Quando as linhas reportarem "pronta + handoff" e **você decidir integrar**:
1. **Junte os handoffs** (um por linha, DIRETRIZ §1.5.9) e abra **uma janela de agente
   integrador** no primário (`main`). Cole os handoffs + a ordem de integração.
2. O integrador funde **uma linha de cada vez** via `bash scripts/foundational-integrate.sh`
   (de dentro de cada worktree): `--ff-only` serializa — a 1ª é FF, as demais rebaseiam sobre
   o novo main e o integrador resolve os conflitos (Cargo.lock/gerados = regenera; mesmo-símbolo
   = renumera/decide com base nos handoffs). Ele reporta o **main verde local** e PARA.
3. Cada linha integrada segue viva pra próxima wave (ou "encerra a linha":
   `git worktree remove` + `git branch -d`).
4. **Ship é 1× por jornada, e SÓ quando você mandar** ("ship"/"push"). O integrador (ou uma
   sessão-ship dedicada) roda:
   ```
   ./scripts/ship.sh          # paridade EXATA com o CI (fmt/clippy/deny/audit/nextest/typos)
   git push origin main       # só depois de verde
   ```
   Aí babysit o CI (`gh run watch`) até `success` — protocolo em DIRETRIZ §8.
   **Nenhum agente pusha sem sua ordem explícita.**

## Higiene (as 3 que evitam 90% dos problemas)

1. **Uma janela por linha, sempre aberta na RAIZ** do repo. O agente cria/entra na worktree
   sozinho; todo o trabalho dele acontece dentro de `Worktrees/line-<módulo>/`.
2. **A janela do "primário" (raiz, em `main`) é só pra setup/integração/ship** — não code em
   `main` direto no Modo L.
3. **Uma linha por módulo.** Duas linhas no mesmo módulo = colisão de merge garantida.

## Modo L × Modo C (1 linha)

| | Modo L (este desktop) | Modo C (Mac 8 GB) |
|---|---|---|
| Estrutura | N linhas autônomas, **sem coordenador** | 1 Coordenador + N Implementadores, shared tree |
| Isolamento | `git worktree` por linha | pastas + disciplina git (DIRETRIZ §7) |
| Integração | agente integrador dedicado (`--ff-only` + gate testado), por ordem sua | Coordenador arbitra e comita |
| Quando | dev do dia-a-dia | smoke/hotfix quando o projeto vai ao Mac |

Branches `line/*` **não viajam pro Mac** — trabalho no Mac é sempre sobre `main`.
