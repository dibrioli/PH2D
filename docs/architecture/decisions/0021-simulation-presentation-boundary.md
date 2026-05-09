# ADR-0021: Fronteira simulation ↔ presentation (SubWorld pattern)

**Status:** Accepted
**Data:** 2026-05-08
**Decisor:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Origem:** auditoria pré-plano (sugestão LLM1; padrão derivado do SubApp de Bevy).

## Contexto

§12.2 do SKILL separa game thread / render thread / audio thread por **execução**, mas não por **tipos**. Nada na arquitetura impede um sistema de presentation (render) de ler um component de simulation (Position simulada). Isso é problema para:

- **HR-5 (determinismo):** se render lê `Velocity` simulada e write back algum cache, lockstep replay diverge.
- **HR-7 (editor é a engine):** editor é um pipeline de presentation que NÃO pode mutar estado simulado durante observação; `extract` phase deve ser one-way.
- **Concorrência futura:** se game e render rodam paralelos (game thread escreve, render thread lê — atomic snapshot), sem fronteira de tipos cada query é race.

Bevy formalizou esse problema com `SubApp`/`SubWorld` em 2023: o `RenderApp` é um SubApp com `World` próprio, recebe dados do main World via `Extract` schedule explícito, nunca toca o main após `Extract` finish.

PH2D vai precisar do mesmo padrão. Sem ele, o primeiro PR que mistura `Query<&mut Position>` em sistema de render cria débito difícil de remover.

## Decisão

PH2D adota separação por tipos via **2 worlds distintos** + **extract phase explícita**:

- `SimWorld` — `bevy_ecs::World` que contém o estado simulado canônico (Position, Velocity, Health, FSM state, etc.). Componentes que entram aqui derivam `Reflect` e `Saveable`.
- `PresentWorld` — `bevy_ecs::World` separado que contém estado de presentation (RenderInstance, AnimationFrame, ParticleBatch, EditorWidget, etc.). Componentes daqui NÃO precisam derivar `Saveable` (não vai a snapshot).

**Extract phase:**
- Cada frame, `extract_systems` correm com acesso `Query<&...>` (read-only) em `SimWorld` + `Query<&mut ...>` em `PresentWorld`. É a única ponte permitida.
- Após `extract` completar, `SimWorld` fica imutável durante o resto do frame (até próximo `pre_tick`). `PresentWorld` pode ser mutado livremente por sistemas de render/UI.

**Enforcement:**
- Trait `SimComponent` (marker auto-derivada por `#[derive(SimComponent)]`) marca componentes de simulação. Trait `PresentComponent` marca os de presentation.
- `SimWorld::insert<C: SimComponent>(...)` e `PresentWorld::insert<C: PresentComponent>(...)` — compile-time error se trocar.
- `extract!(sim => present, |q_sim, q_present| { ... })` macro garante que dentro do bloco a única forma de tocar `SimWorld` é via `Query<&...>` (immutable).

**Fluxo do frame:**

```
1. shell event_pump (input) → SimWorld input queue
2. SimWorld::tick (deterministic, fixed timestep): apply input + run physics + run scripts
3. extract_phase (ponte one-way): SimWorld → PresentWorld
4. PresentWorld::update (interpolation, animation, particles, editor)
5. ph2d-render: traverse PresentWorld + emit GPU commands
6. ph2d-gpu: present
```

**Subsistemas e seus worlds:**

| Subsistema | Vive em | Pode ler do outro? |
|---|---|---|
| ph2d-physics | SimWorld | Não (escreve direto) |
| ph2d-script | SimWorld | Não (sandbox HR-8) |
| ph2d-net | SimWorld (input apply + state diff) | Não |
| ph2d-render | PresentWorld | Sim, READ-ONLY via `extract!` |
| ph2d-vector | PresentWorld | Sim, READ-ONLY via `extract!` |
| ph2d-text | PresentWorld | Sim, READ-ONLY via `extract!` |
| ph2d-light | PresentWorld | Sim, READ-ONLY via `extract!` |
| ph2d-editor | PresentWorld + edição de SimWorld via Commands queue | Sim, mas mutações em SimWorld só via Commands (apply pre_tick) |
| ph2d-audio | Lê snapshot SimWorld (cópia) na audio thread | Sim, snapshot (não query direta) |
| ph2d-mcp | Lê SimWorld (read-only); muta via Commands queue | Sim, mas writes só via Commands |

## Consequências

**Aceitas:**
- 2 worlds em vez de 1 — overhead de memória ~10% (componentes de presentation duplicam alguns dados de sim como derivative).
- `extract!` macro vira o ponto onde 95% dos PRs novos vão tocar — vale escrever bem.
- Compile-time errors visíveis para LLM gerando código (HR friendly): "este component não pode ir em PresentWorld" é mensagem clara.

**Negadas:**
- Não vamos ter 3+ worlds (audio/scripting próprios). Audio recebe snapshot pulled do SimWorld; scripting roda dentro do SimWorld.
- Não vamos permitir que PresentWorld escreva em SimWorld diretamente. Editor faz via Commands explícitas (queue aplicada antes do próximo `tick`).

## Alternativas consideradas

- **1 World único + convenção:** descartado — convenção sem enforcement quebra no primeiro PR descuidado.
- **Run-time check via `change_detection`:** descartado — pega o erro tarde (em runtime, não no compile).
- **3 Worlds (SimWorld + PresentWorld + EditorWorld):** descartado — editor edita SimWorld por design (HR-7 "editor é a engine"); separar EditorWorld duplica o problema.

## Próximos passos

1. Implementar `SimWorld`, `PresentWorld`, traits `SimComponent`/`PresentComponent`, e macro `extract!` em `ph2d-ecs` (M4 do plano operacional).
2. Adicionar lint custom `ph2d-clippy::wrong-world-component` que pega `world.insert::<PresentComponent>()` em SimWorld context (ou vice-versa).
3. Atualizar SKILL §12.2 com link para este ADR e bloco "Two-world model".
4. Quando `ph2d-render` for populado (M5), garantir que **toda** query lá vive em `PresentWorld`.
