# ADR-0003: Escolha de ECS — flecs-rs vs bevy_ecs

**Status:** Proposed (rev2 — pending spike output 2026-05-29)
**Data:** 2026-05-08 (rev2 abertura)
**Decisor:** Enio
**Supersedes:** ADR-0003 original (que pinava `bevy_ecs = "0.18"`)

## Contexto

A camada de scripting da PH2D foi reaberta para revisão (ver `docs/spike/2026-05-plan.md`). Como parte dessa revisão, a escolha de ECS foi questionada: a proposta de scripting Luau via mlua veio acompanhada de mudança implícita de `bevy_ecs` para `flecs-rs`.

A justificativa inicial — "C API estável + dynamic systems + sem lifetime fight com runtime de script" — não foi medida. Sugestões posteriores adicionaram (potencialmente): hierarquias nativas, observers para hot reload de assets, prefab inheritance.

A decisão é arquitetural e custa semanas para reverter depois de N meses de código. Não pode ser tomada por intuição.

## Decisão

**Pendente.** ECS canônico será decidido como output do critério C11 do spike (`docs/spike/2026-05-plan.md`), com base em medição. ADR será atualizado para `Status: Accepted` após o spike, com justificativa baseada nos números coletados.

Critérios de medição (resumo — ver C11 no plano do spike):

- LOC para cobrir caso fixture (200 entities, 1 hierarquia, 3 components, 2 systems, 1 observer/hook em delete)
- Compile time clean + incremental
- Binary size delta (`cargo bloat`)
- Contagem de `unsafe` em `ph2d-ecs`
- Qualidade de stack trace em panic forçado
- Qualidade de geração de código por LLM (cross-vendor)

**Default em caso de medições inconclusivas:** `bevy_ecs = "0.18"` (Rust idiomático, comunidade maior, training data abundante para LLM-as-sole-dev).

## Consequências

**Aceitas neste rev2:**
- Atraso de 3 semanas na decisão final.
- SKILL.md §11 mantém referência a `bevy_ecs = "0.18"` mas com nota explicando reabertura.
- ph2d-ecs adapter trait (`Plugin` ↔ `Module`) será escrito durante spike para isolar a escolha.

**A serem aceitas (qualquer ECS vencer):**
- Plugin model canônico em PH2D continua sendo `Plugin::build(&self, app: &mut App)`.
- Componentes ECS são serializáveis para snapshot determinístico (HR-14).
- Componentes têm lifecycle hooks acessíveis (`on_remove` para limpar storage lateral, HR-16).

## Alternativas consideradas

### bevy_ecs 0.18 (incumbente)
- **Pró:** Rust puro, sem `unsafe` FFI; Reflect maduro; scheduler paralelo automático; comunidade Rust gamedev majoritária; lifecycle hooks (`Component::on_remove`, `on_add`) em 0.18; training data abundante.
- **Contra:** API evolui rápido (breaking changes a cada minor); hierarquias requerem plugin externo; observers limitados.

### flecs-rs (binding C)
- **Pró:** core C maduro (~10 anos produção); observers, prefabs, hierarchies first-class; queries arguably mais expressivas; Sander Mertens responsivo; API C estável (não muda toda minor release).
- **Contra:** binding C em Rust = unsafe surface, debugging mais difícil, stack traces piores; comunidade Rust gamedev menor; LLM training data menor; Reflect equivalente é via `flecs::meta` → JSON → postcard, não direto serde.

### Hecs (descartada)
- **Razão:** mantida menos ativamente; sem scheduler paralelo automático.

### Specs (descartada)
- **Razão:** legada; comunidade migrou para bevy_ecs.

## Não retroceder

- Princípio "LLM é o único programador" pesa explicitamente em C11 (sub-critério LLM ergonomia).
- Se medições forem ambíguas: bevy_ecs vence por defaults (Rust idiomático + training data).

## Próximos passos

1. Spike executa C11 na semana 1 (deadline 2026-05-15).
2. Resultado publicado em `docs/spike/2026-05-report.md`.
3. Este ADR atualizado para `Status: Accepted` com tabela de medições anexada.
4. SKILL.md §5 (Stack canônico) atualizado se vencedor for diferente do incumbente.
