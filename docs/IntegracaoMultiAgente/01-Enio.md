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

### Início do dia / nova sessão de trabalho

1. Abra **Sessão #1** Claude Code no path principal do projeto.
2. Cole o conteúdo de [`02-Coordenador.md`](02-Coordenador.md) +
   "Você é o Coordenador. Inicialize a operação."
3. Coordenador deve reportar em ~30s:
   ```
   Coordenador pronto. STATE.md inicializado.
   Slots livres: 4. Aguardando pedidos.
   ```
4. **Se NÃO reporta isso**, peça pra ele diagnosticar:
   - `git status` mostra mudanças não-commitadas (você fechou
     no meio de algo)
   - STATE.md inconsistente com main local

### Para cada feature nova — fluxo padrão

1. **Você → Sessão #1 (Coordenador):**
   > "Quero feature X — `<descrição em 2-3 linhas>`. Atribua slot."
2. **Coordenador → você:** entrega um briefing pra colar na
   nova sessão (ESCOPO + SLOT + cola integral de
   [`03-Agente-Periferico.md`](03-Agente-Periferico.md)).
3. **Você:** abre **nova Sessão Claude Code** (mesmo path,
   NÃO worktree), cola o briefing.
4. **Agente → você:** pasta proposta + justificativa + tipo
   (Tool stateful / Action one-shot / crate stub).
5. **Você → Coordenador:** cola.
6. **Coordenador → você:** "aprovado" ou "use Y em vez".
7. **Você → Agente:** cola resposta.
8. **Agente** trabalha 10min–2h.

### Durante o trabalho — sinais que você relay

| Mensagem do Agente | Sua ação |
|---|---|
| "Preciso de dep externa `<crate>=<versão>`" | relay → Coordenador edita Cargo.toml + comita |
| "Preciso de variant nova em IconId" | relay → Coordenador cria + comita |
| "Detectei bug em arquivo fora da minha pasta" | relay → Coordenador investiga |
| "Outro agente está mexendo na minha pasta exclusiva" | **PARE TUDO.** relay imediato. Violação grave do modelo. |
| "git status mostra arquivos M que não toquei" | possível colisão. Pause o Agente até saber qual outra sessão tem staged |
| "cargo nextest passa, feature pronta" | relay → Coordenador enfileira |

### Sintomas de colisão — alerta vermelho

Se algum agente reportar:

- `fatal: cannot lock ref 'HEAD'` no terminal dele
- `git log` mostrando commit com mensagem fundida (dois títulos
  colados, dois `Co-Authored-By`)
- `git status --cached` mostrando arquivos que ele não estaviou

**Sua ação imediata:** pause TODAS as sessões. Relay pro
Coordenador → ele segue protocolo de recovery em §3.6 do
[`02-Coordenador.md`](02-Coordenador.md). Não autorize novos
commits até ele dar "limpo".

### Sinais de saúde — pergunte 1× por hora

Pra Coordenador (uma vez a cada hora ou após integração pesada):

> "Status de saúde? git status, STATE.md, último cargo check."

Resposta esperada:
- Working tree clean
- STATE.md reflete slots vivos
- `cargo check --workspace` verde no último commit

Sinal vermelho → Coordenador investiga e reverte se grave.

### Quando Agente termina

1. **Agente → você:** relatório (formato em §12 de
   [`03-Agente-Periferico.md`](03-Agente-Periferico.md)).
2. **Você → Coordenador:** cola.
3. **Coordenador:** integra (5–10min — hook T2 ~5min + smoke
   `cargo run`).
4. **Coordenador → você:** "Integrado. Slot <N>: done."
5. Você pode fechar a sessão do Agente ou mantê-la pra
   próxima feature dele.

### Velocidade dos commits por tier

O pre-commit hook é tiered — o tempo varia muito conforme o
escopo da mudança:

| Mudança | Tier | Tempo |
|---|---|---|
| Só docs / scripts / `.md` | T0 | ~5s |
| Só 1 pasta de Agente | T1 | ~30s |
| Cargo.toml / shells/desktop / multi-crate / foundational | T2 | **~3–5min** |
| Bypass `--no-verify` (após validação manual) | — | ~1s |

Coordenador integrando = T2 obrigatório. Agente em pasta isolada
= T1 ou bypass após validação local.

### Final do ciclo — passa pro GitHub (UMA VEZ POR DIA)

CI roda matrix completa (linux + macOS + windows + replay hash +
bench) e demora **~30min**. Por isso esse fluxo é **uma vez por
dia, ao final da jornada**:

1. **Você → Coordenador:** "Manda pro GitHub."
2. Coordenador valida (`cargo test --workspace`, smoke visual)
   e reseta STATE.md.
3. Coordenador assume papel de PRCI (lê
   [`04-Agente-PRCI.md`](04-Agente-PRCI.md)) OU você abre nova
   sessão com esse doc.
4. PRCI faz `git push` + abre PR + entrega URLs:
   - URL do PR
   - URL da run de CI
5. **PRCI entra em modo babysit** (§7 de
   [`04-Agente-PRCI.md`](04-Agente-PRCI.md)): polling de **15min**,
   diagnostica + corrige + re-push se falhar, até CI ficar verde.
6. Você NÃO confere CI visualmente neste fluxo — é PRCI quem
   cuida. Vê os reports dele quando voltar pro app.
7. PRCI fecha o ciclo com 1 de 3 mensagens:
   - **"CI conclui success"** → tudo OK; jornada fechada.
   - **"Falha 3× no mesmo job — escalando"** → você precisa
     decidir (continuar tentando, reverter pra `backup/...`,
     dropar o esforço).
   - **"Você cancelou o babysit"** → você pediu explicitamente
     pra parar; problema fica pra próxima jornada.

**Importante:** **um único PR por dia** (matrix de CI é cara). Se
durante a jornada surge feature urgente que precisaria de push
imediato, prefira:
- Trabalhar localmente até estável + integrar no Coordenador
- Push só ao final da jornada (junto com outras features que
  vieram no dia)

Push fora desse fluxo é exceção rara (hotfix de produção, etc.) —
neste caso, peça explicitamente "push hotfix agora" ao PRCI.

### O que você NUNCA faz

- Roda git, cargo, brew, ou qualquer comando.
- Edita arquivos do projeto direto.
- Decide slug, pasta, ou wiring (Agente propõe, Coordenador valida).
- Push pro GitHub manualmente (Coordenador / PRCI faz).
- **Tem duas sessões ativas como Coordenador ao mesmo tempo.**
  Uma só. Periféricos podem ser 1–4 em paralelo.
- Autoriza Periférico a `git push`.
- Autoriza Periférico a tocar arquivos fora da pasta exclusiva
  dele (sempre relay pro Coordenador fazer).

### O que você SEMPRE faz

- Copia/cola mensagens entre sessões — verbatim, sem editar.
- Decide quando começar feature, quando integrar, quando pushar
  (fim de jornada).
- Para tudo na primeira mensagem de colisão.
- Decide escalação quando PRCI reporta "3 ciclos de falha" no
  babysit (continuar, reverter, dropar).

### Decisões só suas (Coordenador não decide por você)

| Decisão | Quando aparece |
|---|---|
| "Quero feature X" | Início de cada feature |
| "Manda pro GitHub" | Final da jornada (1× por dia) |
| "Aceito este ADR? `<decisão>`" | Mudança arquitetural não-trivial |
| "Reverter ao sha bom?" | Integração quebrou main + fix mínimo não é possível |
| "Slot <N> fecha ou pega próxima feature?" | Após Agente reportar pronto |
| "PRCI escalou — continuar tentando / reverter / dropar?" | Quando CI falha 3× consecutivas no babysit |

### Exemplo concreto — sequência canônica

```
[Você → Sessão#1]   Quero feature "trim transparency". Action one-shot. Atribua slot.

[Sessão#1 → você]   Slot #1 atribuído: slug `trim-transparency`. Briefing:
                    <ESCOPO + cola de 03-Agente-Periferico.md>

[Você]              (abre Sessão#2, cola briefing)

[Sessão#2 → você]   Pasta proposta: crates/ph2d-editor/src/tools/trim_transparency/
                    Tipo: Action one-shot.

[Você → Sessão#1]   (cola)

[Sessão#1 → você]   Aprovado. Slot #1 = working.

[Você → Sessão#2]   Aprovado.

[Sessão#2]          (trabalha 20min)

[Sessão#2 → você]   Preciso de IconId::TrimTransparency, SVG path "M 4 4 L 20 4...".

[Você → Sessão#1]   (cola)

[Sessão#1 → você]   Criado em icons.rs. Comitado.

[Você → Sessão#2]   Feito.

[Sessão#2]          (trabalha mais 30min)

[Sessão#2 → você]   Feature pronta. APIs: pub fn apply(img). Wiring pendente:
                    TopBar cluster, click handler em shells/desktop.

[Você → Sessão#1]   (cola)

[Sessão#1 → você]   Integrando. ~5min (hook T2).

[Sessão#1]          (5min)

[Sessão#1 → você]   Integrado. Slot #1: trim-transparency done. Fila vazia.

[Você → Sessão#1]   (fim de jornada) Manda pro GitHub.

[Sessão#1 → você]   (assume PRCI, push, PR #42 aberto)
                    PR: github.com/.../pull/42
                    CI: github.com/.../runs/12345
                    Entrando em modo babysit (polling 15min).

[Sessão#1]          (polling em background; pode demorar 30-60min)

[Sessão#1 → você]   ✓ CI conclui success em 32min. PR mergeado por
                    fast-forward. Modo babysit fechado. Jornada
                    encerrada.

OU (cenário de falha):

[Sessão#1 → você]   ✗ Job macos falhou (lint typos: edge_thr).
                    Adicionei thr ao .typos.toml, push, nova run.
                    CI run: github.com/.../runs/12346
                    Continuando babysit.

[Sessão#1]          (mais 30min)

[Sessão#1 → você]   ✓ CI conclui success na 2ª tentativa.
                    Jornada encerrada.
```
