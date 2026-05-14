# Diretriz Multi-Agente — Enio (modelo Coordenador + Periféricos)

**Versão:** 4.0 — 2026-05-13
**Modelo:** sem branches feature/, sem worktrees, tudo local em main,
push pro GitHub só no final.

## Topologia

```
Você (Enio) — relay humano
 │
 ├─ Coordenador  (sessão Claude Code #1 — dedicada, sempre ativa)
 │   • Mantém STATE.md como fonte de verdade
 │   • Único que toca arquivos compartilhados
 │   • Atribui slots, valida pastas, integra features
 │
 └─ Agentes Periféricos (até 4 sessões Claude Code paralelas)
     • Cada um numa sessão separada, MESMO path do projeto
     • Cada um trabalha em pasta(s) exclusiva(s)
     • Comunicam via você ↔ Coordenador
```

Todas as sessões abertas no mesmo diretório principal do projeto.
Sem `git worktree add`. Sem `git checkout -b`. Sem `git push`. Tudo
local em main até o ciclo terminar.

## Seu papel

Você é o **relay humano**: copia/cola mensagens entre Coordenador e
Agentes Periféricos. Não roda código, não roda git, não toma
decisões técnicas. Você decide:
- Quando iniciar nova feature.
- Quando parar e mandar pro GitHub.
- Resolver impasses (quando Coordenador apresenta opções).

## Fluxo padrão

### Setup inicial (uma vez por sessão de trabalho)

1. Abra **Sessão #1** Claude Code no path principal do projeto.
2. Cole o conteúdo inteiro de [`02-Coordenador.md`](02-Coordenador.md)
   + "Você é o Coordenador. Inicialize a operação."
3. Coordenador lê tudo, inicializa STATE.md, reporta pronto.

### Para cada feature nova

1. Diga ao Coordenador: "Quero feature X. Atribua slot."
2. Coordenador prepara briefing personalizado (cola de
   [`03-Agente-Periferico.md`](03-Agente-Periferico.md) + escopo +
   slot). Atualiza STATE.md.
3. Você abre **nova Sessão Claude Code** (mesmo path), cola o briefing
   que o Coordenador te deu.
4. Agente Periférico lê briefing + SKILL + STATE.md. **Decide pasta(s)
   exclusiva(s)** baseado em natureza da feature + arquitetura do app.
5. Agente comunica a você: "Vou trabalhar em <pastas>. Faz sentido?"
6. Você cola essa mensagem na sessão do Coordenador.
7. Coordenador valida (pasta livre? bate com arquitetura?), atualiza
   STATE.md, te devolve "aprovado" ou "use Y em vez de X".
8. Você cola a resposta de volta na sessão do Agente.
9. Agente começa a codificar SÓ na pasta aprovada.

### Durante o trabalho do Agente

Casos típicos onde o Agente para e pede ajuda (via você):

- **Precisa adicionar dep externa** (Cargo.toml fora da pasta dele).
- **Precisa de variant nova em IconId.**
- **Precisa de wiring inicial pra testar visualmente.**
- **Detectou bug em código fora da pasta exclusiva.**

Em qualquer caso:
1. Agente reporta a você com justificativa.
2. Você cola pra Coordenador.
3. Coordenador atende (faz a mudança ele mesmo se autorizado, ou
   pergunta a você se exige ADR).
4. Você cola resposta pra Agente.
5. Agente prossegue.

### Quando Agente termina

1. Agente reporta: "Feature pronta. APIs públicas: A, B, C. Wiring
   pendente: D, E."
2. Você cola pra Coordenador.
3. Coordenador adiciona à fila no STATE.md.
4. Quando vez do Agente chega: Coordenador integra (toca arquivos
   compartilhados), valida, comita, atualiza STATE.md.
5. Coordenador reporta: "Integrado. Slot livre."

### Final do ciclo — passa pro GitHub

Quando todas as features estão integradas e estáveis em main local,
decida: "manda PR pro GitHub".

- Coordenador pode assumir esse papel (lê [`04-Agente-PRCI.md`](04-Agente-PRCI.md))
  ou você abre sessão dedicada nova com esse doc.
- Push da main local + abertura de PR + link da run de CI.
- Daqui pra frente é o fluxo padrão GitHub.

## Tabela de papéis × documento

| Papel | Doc que você cola | Pode pushar? |
|---|---|---|
| Coordenador (Sessão #1, sempre ativa) | [`02-Coordenador.md`](02-Coordenador.md) | só no final |
| Agente Periférico (Sessões #2-5, 1-4 paralelos) | [`03-Agente-Periferico.md`](03-Agente-Periferico.md) | ❌ |
| Agente PRCI (só ativado no final) | [`04-Agente-PRCI.md`](04-Agente-PRCI.md) | ✅ |

## Regras de ouro

- **STATE.md é a fonte de verdade.** Confia mais nele do que em
  sua memória.
- **Apenas Coordenador escreve em arquivos compartilhados.**
- **Apenas Coordenador escreve em STATE.md.**
- **Cada Agente Periférico escreve só na pasta exclusiva dele.**
- **Sem branches.** Sem `git checkout -b`. Tudo em main local.
- **Sem push** até toda fila local estar resolvida.
- **Máximo 4 Agentes Periféricos simultâneos** (limite do STATE.md).
- **Um único Coordenador ativo por vez.** Se você se pegar
  pedindo a duas sessões diferentes pra "atuar como Coordenador"
  em paralelo, PARE — colisão de commits garantida (vide
  `feedback_parallel_agent_collision.md` na memória LLM). Uma
  sessão Coordenador, várias sessões Periféricas.
- **Commits são serializados, não paralelos.** Stage + commit
  são operação atômica. Se uma sessão acabou de `git add`,
  nenhuma outra deve `git commit` até a primeira terminar o
  ciclo dela (incluindo a janela do pre-commit hook, ~30s a
  ~5min dependendo do tier). Em caso de dúvida, sinalize via
  você (relay) antes de iniciar um commit pesado.

## Quando algo dá errado

- **2 agentes querem mesma pasta:** Coordenador propõe ajuste de slug
  (`painter` → `painter-v2`).
- **Agente tenta tocar arquivo fora da pasta:** o briefing dele pede
  pra parar e reportar; se chegou a editar, Coordenador reverte.
- **Build quebra em main local:** Coordenador detecta no `cargo check`
  após integração; se grave, reverte ao "sha conhecido bom" no STATE.md.
- **Coordenador "morre" (você fecha a sessão):** STATE.md persiste no
  disco; abra nova sessão de Coordenador, ele lê STATE.md e continua.

## Hierarquia de docs neste fluxo

- **Este doc** (`01-Enio.md`) — seu manual.
- **`02-Coordenador.md`** — você cola na Sessão #1 (Coordenador).
- **`03-Agente-Periferico.md`** — Coordenador cola na sessão de cada
  Agente (após adicionar escopo + slot).
- **`04-Agente-PRCI.md`** — só ativado no final.
- **`STATE.md`** — fonte de verdade do estado da operação **atual**
  (Coordenador mantém; criado a partir de `STATE.md.template` no
  setup; resetado pra template ao final do ciclo).
- **`STATE.md.template`** — backup imutável do formato de estado.
  Não é editado durante operação. Coordenador copia sobre `STATE.md`
  ao inicializar nova operação e ao resetar no fim do ciclo.
- **`SKILL_Stack_PH2D_Definitiva.md`** + **`CLAUDE.md`** — leitura
  obrigatória de Coordenador e Agentes; você só consulta se quiser
  entender uma dúvida arquitetural.

## Cheat sheet — seu modo de agir

**Início do dia / nova sessão de trabalho:**
1. Sessão #1 Claude Code → cola `02-Coordenador.md` → "Você é o Coordenador. Inicialize."

**Para cada feature nova:**
2. Na Sessão #1 → "Quero feature X. Atribua slot." → recebe briefing.
3. Nova sessão Claude Code (mesmo path) → cola o briefing.
4. Agente propõe pasta → você relay pro Coordenador → Coordenador aprova/ajusta → você relay de volta.
5. Agente pede algo fora (dep, ícone, etc.) → relay pro Coordenador → ele faz → relay de volta.
6. Agente reporta "pronto" → relay pro Coordenador → ele enfileira e integra.

**Final do ciclo:**
7. Tudo integrado → diz pro Coordenador "manda pro GitHub" → recebe link do PR + run de CI.
8. Confere CI visualmente no GitHub.
9. Coordenador reseta STATE.md pra próxima operação.

**O que você NUNCA faz:**
- Roda git, cargo ou qualquer comando.
- Edita arquivos do projeto direto.
- Decide slug, pasta, ou wiring.
- Push pro GitHub manualmente (Coordenador/PRCI faz).

**O que você SEMPRE faz:**
- Copia/cola mensagens entre sessões.
- Decide quando começar feature, quando integrar, quando pushar.
- Resolve impasses quando Coordenador apresenta opções.
