# Diretriz de implementação Multi-Agente — Enio

**Versão:** 3.0 — 2026-05-12
**Audiência:** Enio (coordenador da operação).

## Seu papel

Você é o único decisor humano. Não escreve código, não roda comandos
git, não pensa em worktrees, branches, slugs ou paths. Você só:

1. Descreve features que quer.
2. Aprova ou redireciona o que volta.
3. Confere o CI visualmente quando o PR sai.

O resto, o agente faz — incluindo criar worktree, descobrir slug,
preparar ambiente. Se ele tiver dúvida sobre escopo ou direção, ele
te pergunta.

## Fluxo padrão pra desenvolver uma feature

### Passo 1 — Abra Claude Code no diretório principal do projeto

Path: `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva` (ou o que
estiver).

### Passo 2 — Cole o doc do Implementador + diga o que quer

Na primeira mensagem dessa sessão Claude Code, cole o conteúdo
inteiro de [`02-Implementador.md`](02-Implementador.md), e logo
abaixo:

```
Quero implementar: <descrição da feature em 1-5 linhas>
```

Pode ser bem informal — "Quero uma Tool de Background Removal com 4
algoritmos" ou só "quero uma Tool nova, vou descrever". Se faltar
detalhe, o agente te pergunta.

### Passo 3 — O agente faz setup E implementa, tudo na mesma sessão

Vendo que está no diretório principal (§2.1 da diretriz dele), o
agente:
- Pergunta o nome/escopo da feature se você ainda não descreveu.
- Se tiver dúvidas sobre arquitetura ou abordagem, te apresenta
  opções concretas. Você decide.
- Deriva o slug (kebab-case curto, ex: `bgremoval`).
- Cria worktree + branch (`git worktree add ...`).
- Entra na worktree via `cd .claude/worktrees/agent-<slug>` (o Bash
  do Claude Code persiste cwd entre comandos).
- **Continua trabalhando ali na mesma sessão**: lê SKILL, implementa
  a feature completa, roda testes, commita local na branch.
- Reporta "feature pronta na worktree, aguardando integração".

Você não abre nova sessão. Você não cola briefing duas vezes.

### Passo 4 — Implementador reporta "pronto"

Quando ele reportar "pronto pra integração", a feature está
esperando na worktree dele. Você pode rodar outras features em
paralelo (volta ao Passo 1 noutra janela Claude Code no diretório
principal, descrevendo outra feature — cada uma vai virar sua
worktree isolada).

### Passo 5 — Integração local (quando vários estão prontos)

Quando uma ou mais features estão prontas E nenhum Implementador
está mais ativo:
- Abre Claude Code no diretório principal.
- Cola [`03-Integrador.md`](03-Integrador.md) + diga "Integra as
  branches `feature/X` e `feature/Y` na `main`."
- Agente confere `ls .claude/worktrees/`, faz merge + wiring +
  atualizações de docs.
- Reporta "integração local pronta".

### Passo 6 — Push e PR (quando você decide enviar pro GitHub)

- Abre Claude Code no diretório principal (ou na branch integrada).
- Cola [`04-Agente-PRCI.md`](04-Agente-PRCI.md) + diga "Manda PR
  da branch `<nome>` pra main."
- Recebe link do PR + link da run de CI. Confere visualmente no
  GitHub.

## Os 3 tipos de agente

| Papel | Doc que você cola | Pode pushar? |
|---|---|---|
| Implementador (1-N em paralelo, cada um na sua worktree) | [`02-Implementador.md`](02-Implementador.md) | ❌ |
| Integrador (1 por vez, sem Implementador ativo) | [`03-Integrador.md`](03-Integrador.md) | ❌ |
| Agente PRCI (1 por vez, após integração) | [`04-Agente-PRCI.md`](04-Agente-PRCI.md) | ✅ |

## Regras de ouro

- **Nunca rode mais que 1 Integrador OU 1 PRCI por vez.**
- **Nunca rode Integrador enquanto há Implementador ativo.**
  `ls .claude/worktrees/` é a fonte de verdade.
- **Implementadores não se comunicam entre si** — comunicam só com você.
- **Você não roda comandos git.** Os agentes rodam.
- **Você não nomeia paths nem branches.** Os agentes nomeiam.

## Quando algo dá errado

- **Implementador sinaliza que precisa tocar a blacklist** (§7 da
  diretriz dele) → ele te explica o porquê e propõe alternativas.
  Você decide: ajustar escopo, virar tarefa de Integrador, abrir ADR.
- **Implementador identifica hook arquitetural faltando** (ex:
  canvas não envia pointer pra Tool) → ele te diz e propõe entregar
  só painel + API; Integrador projeta a amarração depois.
- **Integrador encontra conflitos não-óbvios** → ele te explica.
  Pode pedir Implementador a corrigir feature na worktree dele.
- **PRCI reporta CI vermelho** → você lê o link da run, decide se
  vale pedir investigação. Cola novo briefing pra agente diagnóstico
  se quiser.

## Hierarquia de docs

- Este doc (`01-Enio.md`) — seu manual de bolso.
- `02-Implementador.md`, `03-Integrador.md`, `04-Agente-PRCI.md` —
  você cola um deles ao iniciar cada agente. Não precisa ler na
  íntegra; basta saber qual usar.
- `docs/PARALLEL_AGENTS.md` — política referenciada pelos 3.
- `SKILL_Stack_PH2D_Definitiva.md` + `CLAUDE.md` — leitura obrigatória
  dos agentes; você só consulta se quiser entender uma dúvida deles.
