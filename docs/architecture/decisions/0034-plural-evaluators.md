# ADR-0034 — Avaliadores plurais por modelo-de-avaliação + lowering plural por nó

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto).
**Depende de:** ADR-0030, ADR-0032, ADR-0021 (Sim/Present).

## 1. Contexto

A linha de corte do ADR-0030 deixa a **avaliação** plural. A teoria (Conal Elliott, push-pull FRP) diz que push e pull não competem — especializam-se por natureza do sinal (events=push, behaviors=pull). E o domínio de áudio tem natureza distinta: stream consumido em blocos por relógio fixo (synchronous dataflow, Lustre/FAUST), incompatível com cook lazy demand-driven.

## 2. Decisão

Um **avaliador por modelo-de-avaliação** (não por "tema"), cada um um crate fino sobre backend existente:

| Domínio | Modelo de avaliação | Lowering | Determinismo | Crate |
|---------|---------------------|----------|--------------|-------|
| **Gameplay/logic** | **push** (eventos mutam estado, ordem total estável) | Luau/bytecode (CPU) | **HR-5** (escreve SimWorld) | `ph2d-script` (✅ existe) |
| **Motion** | **pull** no playhead `t` | Vello / sprite instancing / SDF | isento | `ph2d-eval-motion` |
| **Shader** | **pull** por-pixel/frame | **WGSL** (naga/`ph2d-gpu`) | isento | `ph2d-eval-shader` |
| **Sound** | **synchronous dataflow** (relógio fixo, escalonamento estático) | grafo DSP (`ph2d-audio`) | isento | `ph2d-eval-audio` |

**Lowering plural por nó:** um nó declara N lowerings; o avaliador do domínio escolhe. (Conserta o vazamento "nó que roda só em Rust nunca vai pra GPU".)

**Benefício de borda:** o escalonamento estático do áudio (relógio conhecido em compile-time) **dá o frame budget de pior caso de graça** (HR-4).

## 3. Consequências

**Aceitas:**
- Cada domínio usa o modelo de avaliação correto pra sua natureza de sinal; o erro de usar **um** modelo pro grafo inteiro (pull-quase-universal do Houdini; push-via-exec-wires do Blueprint) é evitado.
- A membrana (ADR-0030) é o único ponto onde push e pull se tocam, e só numa direção.

**Riscos:**
- 4 avaliadores divergirem → o substrato único (ADR-0032) + lowering-como-spec mantêm o contrato comum; só o agendamento/lowering difere.

## 4. Alternativas consideradas

- **Um avaliador único pro grafo todo:** rejeitado — não há agendador que sirva pull-lazy e synchronous-dataflow de áudio sem latência/glitch.
- **Tudo reativo/push (FRP puro):** rejeitado pro core — push tem problema de sync sinal↔controle; para fixed-step 60Hz determinístico, pull-com-cook-por-tick é correto. Reativo só na camada de input/eventos de UI.
