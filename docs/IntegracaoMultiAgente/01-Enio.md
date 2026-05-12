# Diretriz de implementação Multi-Agente — Enio

**Versão:** 1.0 — 2026-05-12
**Audiência:** Enio (coordenador da operação).

## Seu papel

Você é o único decisor humano e coordenador. Não escreve código,
não comita. Direciona, confere, decide quando integrar e quando
pushar. Toda comunicação entre agentes passa por você — agentes
nunca se comunicam direto.

## Os 3 tipos de agente

| Papel | Doc que você cola no primeiro turno | Pode pushar? |
|---|---|---|
| Implementador | `02-Implementador.md` + escopo + path da worktree | ❌ |
| Integrador | `03-Integrador.md` + lista de branches prontas + destino | ❌ |
| Agente PRCI | `04-Agente-PRCI.md` + branch integrada | ✅ |

## Fluxo padrão

1. **Defina escopo** da feature (ex: "Tool Painter no editor",
   "popular ph2d-audio com mixer básico").
2. **Crie worktree** local para o Implementador:
   ```
   git worktree add .claude/worktrees/agent-<id> -b feature/<nome> <base>
   ```
   `<base>` é normalmente `main` ou a branch ativa do marco corrente.
3. **Inicie agente Implementador**: cole o conteúdo de
   `02-Implementador.md` + abaixo dele, no mesmo turno, os 3 campos
   que ele precisa pra começar (sem esses, ele vai te perguntar):
   - **ESCOPO** (2-5 linhas): descrição da feature **COMPLETA**
     (sem fatiar em MVP — o Implementador entrega a ilha inteira
     de uma vez).
   - **WORKTREE**: path absoluto, ex:
     `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/.claude/worktrees/agent-<id>`.
   - **BRANCH**: `feature/<nome-curto>`.

   Se o Implementador te perguntar "feature inteira ou MVP?" ou
   "qual worktree?", o doc deixou claro que ele não devia perguntar
   isso — você esqueceu de informar. Apenas informe e prossiga.

   Aguarde relatório "pronto pra integração".
4. **Pode rodar múltiplos Implementadores em paralelo**, cada um
   em sua worktree, desde que estejam em features ISOLADAS (vide
   blacklist em `docs/PARALLEL_AGENTS.md`).
5. **Quando uma ou mais features estiverem prontas**, confirme
   que nenhum Implementador está ativo:
   ```
   ls .claude/worktrees/
   ```
6. **Inicie agente Integrador**: cole `03-Integrador.md` + lista
   de branches a integrar + branch destino. Aguarde "integração
   local pronta".
7. **Decida**: pushar agora, ou acumular mais integrações antes
   de pushar?
8. Se for pushar, **inicie agente PRCI**: cole `04-Agente-PRCI.md`
   + nome da branch integrada. Receba link do PR + link da run de CI.
9. **Confira CI visualmente** no GitHub. Se vermelho, decida se
   investiga agora ou pausa.

## Regras de ouro

- Nunca rode mais que **1 Integrador OU 1 PRCI** por vez.
- Nunca rode Integrador enquanto há Implementador ativo.
- Implementadores **não se comunicam entre si** — comunicam só com você.
- `ls .claude/worktrees/` é a fonte de verdade sobre quem está ativo.

## Quando algo dá errado

- **Implementador sinaliza que precisa tocar a blacklist** → pause-o.
  Decide: ajusta escopo da feature pra não precisar, vira tarefa
  de Integrador, ou ramifica decisão pra ADR nova.
- **Integrador encontra conflitos não-óbvios** → pode pedir
  Implementador a corrigir a feature na worktree dele e retomar
  integração depois.
- **PRCI reporta CI vermelho** → leia o link da run, decide se
  vale investigar agora (cola briefing pedindo investigação) ou
  pausar pra próxima janela.

## Hierarquia de docs neste fluxo

- Este doc (`01-Enio.md`) — seu manual.
- `02-Implementador.md`, `03-Integrador.md`, `04-Agente-PRCI.md` —
  você cola um deles ao iniciar cada agente.
- `docs/PARALLEL_AGENTS.md` — política referência citada pelos 3.
- `SKILL_Stack_PH2D_Definitiva.md` + `CLAUDE.md` — leitura obrigatória
  dos agentes; você só consulta se surge dúvida arquitetural.
