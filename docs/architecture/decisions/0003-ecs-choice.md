# ADR-0003: Escolha de ECS — bevy_ecs 0.18

**Status:** Accepted (rev2)
**Data:** 2026-05-08 (rev2 abertura), 2026-05-08 (rev2 decisão)
**Decisor:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Supersedes:** ADR-0003 original (rev1) — pinning original `bevy_ecs = "0.18"` foi reaberto em 2026-05-08 e ratificado em 2026-05-08 com base em medição.

## Contexto

A camada de scripting da PH2D foi reaberta para revisão (ver `docs/spike/2026-05-plan.md`). A escolha de ECS foi questionada: a proposta de scripting Luau via mlua veio acompanhada de mudança implícita de `bevy_ecs` para `flecs-rs`, justificada por "C API estável + dynamic systems + sem lifetime fight com runtime de script".

Como a decisão é arquitetural e custa semanas para reverter, o critério C11 do spike foi definido para forçar medição direta antes de cristalizar.

## Decisão

**ECS canônico = `bevy_ecs = "0.18"`**, conforme proposto originalmente. flecs-rs descartado para uso canônico.

## Justificativa baseada em medição (C11)

Fixture mínima (200 entities, 1 hierarquia, 3 components, 2 systems, 1 observer/hook em delete) implementada em ambos backends:
- `tests/spike/src/bin/c11_bevy.rs` (84 LOC, `bevy_ecs = "0.18.1"`)
- `tests/spike/src/bin/c11_flecs.rs` (78 LOC, `flecs_ecs = "0.2.2"`)

Medições no Mac M-series (sessão de 2026-05-08, profile release com `lto=thin`, `codegen-units=1`, `strip=symbols`):

| Métrica | bevy_ecs 0.18 | flecs_ecs 0.2.2 | Threshold C11 (flecs aceitável) | Resultado |
|---|---|---|---|---|
| LOC fixture | 84 | 78 (-7%) | flecs ≤ +10% | flecs aceitável |
| Compile time clean (release) | 27.31s | 20.18s (-26%) | flecs ≤ +30% | **flecs ganha** |
| Binary size (release stripped) | 881 KB | 1450 KB (**+64%**) | flecs ≤ +15% | **flecs ESTOURA** |
| `unsafe` blocks na fixture | 0 | 0 | n/a | empate |
| Observer dispara em fixture (corrigido S2) | **5/5** | 5/5 (+ 195 on drop) | "igualmente clara" | empate |

**Interpretação dos resultados:**

flecs_ecs ganha em DX (observer behavior previsível) e em compile time. Mas estoura threshold de binary size em **+64%** quando o threshold é **+15%**. Per critério C11 do plano (`docs/spike/2026-05-plan.md` L137): "flecs aceitável se TODOS os thresholds passam". Como binary size falha, flecs não é aceitável por critério estrito.

Per HANDOFF L173-180: "flecs precisa vencer claramente em hierarquias/observers/queries para justificar o custo do binding C. Se ambíguas: bevy_ecs vence". Resultado é misto, não vitória clara para flecs → bevy_ecs por default.

## Medições secundárias (LLM friction)

LLM-as-sole-developer (Claude 4.7+) experimentou ambos APIs durante implementação da fixture. Observações qualitativas:

**bevy_ecs 0.18 — friction alta inicial, corrigida em S2:**
- API mudou em 0.18: `Trigger<OnRemove, T>` → `On<Remove, T>`, `iter_entities()` removido. Sem release notes claras na crates.io page; descoberta via grep no source.
- "Bug" inicial (observer 0/200) **não era do bevy** — fixture C11 original usava `world.query::<Entity>()` para escolher entities a despawn, e a query retorna entities INTERNAS criadas implicitamente por bevy ao usar `ChildOf` (ex: storage do `Children` component). Despawn dessas não dispara `On<Remove, Health>` — comportamento correto, porque elas nunca tiveram Health. Fix aplicado em S2: armazenar IDs explicitamente no spawn loop e despawnar esses. Resultado: 5/5 fires, exato.
- Tempo total para fazer fixture funcionar + descobrir o "bug": ~45 min de iteração com compilador + grep no source. LLM friction real é a desinformação na crates.io page sobre nomes de tipos atualizados (Trigger→On, etc), não o lifecycle observer behavior.

**flecs_ecs 0.2.2 — friction baixa:**
- Builder fluent (`world.observer::<flecs::OnRemove, &Health>().each_iter(...)`) intuitivo.
- API estável (binding sobre flecs C, que tem ~10 anos de produção).
- Observer disparou 200/200 vezes (5 destructs explícitos + 195 implícitos no World drop).
- Tempo para fixture funcionar: ~10 min, primeira tentativa quase compilou (pequeno ajuste em `run_iter` → `each` no query final).

**Net LLM-friction:** flecs vence por margem material. Mas isso sozinho não basta — bevy mantém a vantagem em Rust idiomático puro (zero unsafe FFI, stack traces 100% Rust, training data muito mais abundante).

**Critério C11 LLM gen quality (3 prompts × 2 modelos):** não executado formalmente nesta sessão. Subjetivamente Claude 4.7 favoreceu flecs marginalmente. Cross-vendor (Gemini 3.1+ Pro) deferido para C8/C15/C16 da Semana 3.

## Consequências

**Aceitas:**
- `crates/ph2d-ecs` será populado com wrapper sobre `bevy_ecs = "0.18"`.
- `Plugin` model canônico em PH2D segue o `bevy_ecs::plugin::Plugin`.
- Componentes ECS serão `#[derive(Component)]` do bevy_ecs.
- Lifecycle hooks via `Component::on_add` / `on_remove` (não via observers — vide bug não-resolvido abaixo).
- Migração de schedule/system definitions já é Rust idiomático familiar para LLM.

**Riscos aceitos:**
- ~~bevy_ecs 0.18 lifecycle observers têm comportamento não-óbvio~~ **RESOLVIDO em S2**: investigação isolou que o "bug" era do meu fixture C11 (despawn de entities sem Health via query). Bevy observers funcionam corretamente. Vide `tests/spike/src/bin/bevy_observer_debug.rs` (probe T1..T11).
- bevy_ecs muda API entre minors. **Mitigação:** pin estrito de versão em `Cargo.toml`, upgrade deliberado em ADR separado.
- Hierarquias usam `ChildOf` relacionamento built-in em 0.18 (mais ergonômico que versões anteriores), mas se precisarmos de relacionamentos custom (estilo flecs), implementar via componentes ad-hoc.

**Negadas:**
- Não vamos manter trait abstraída `EcsBackend` para permitir swap futuro. Custo de manter abstração > benefício de flexibilidade. Se precisarmos trocar de ECS no futuro, fazemos refactor focado.

## Pontos não-medidos (a revisar)

- **Stack trace quality em panic forçado:** não medido formalmente. Bevy_ecs é Rust puro (panic em sistema mostra stack 100% Rust); flecs panic geralmente atravessa FFI C↔Rust e perde frames. Vantagem qualitativa para bevy. Não muda decisão.
- **LLM gen quality cross-vendor (3 prompts × 2 modelos):** deferido para C8/C15/C16 (Semana 3). Não bloqueia decisão.
- **Hot reload lifecycle hooks (HR-16 storage lateral cleanup):** validado em S2 (C4 PASS). Reset+restore via postcard+blake3 é determinístico em 100/100 ciclos. Observer não é necessário para HR-16 (snapshot-based, não event-based).

## Alternativas consideradas

### bevy_ecs 0.18 (escolhida)
- **Pró:** Rust puro sem `unsafe` FFI; scheduler paralelo automático; comunidade Rust gamedev majoritária; lifecycle hooks via `Component::on_remove` em 0.18; training data abundante para LLM (treinado até janeiro 2026); Reflect maduro; binary size mais enxuto.
- **Contra:** API evolui rápido (breaking changes em cada minor); observers de lifecycle têm comportamento não-óbvio em fixtures complexas (vide bug Semana 1, pendente investigação).

### flecs_ecs 0.2.2 (descartada por C11 binary size)
- **Pró:** core C maduro (~10 anos); observers funcionam consistentemente; hierarquias e relations first-class; queries expressivas; LOC menor; compile time menor.
- **Contra:** binding C → unsafe surface; binary size +64% vs bevy (estoura threshold +15%); training data Rust gamedev menor; stack traces atravessam FFI; Reflect equivalente via JSON.

### Hecs (descartada)
- Mantida menos ativamente; sem scheduler paralelo automático.

### Specs (descartada)
- Legada; comunidade migrou para bevy_ecs.

## Não retroceder

- bevy_ecs é decisão final para v0.1 da PH2D. Trocar exige novo ADR com justificativa.
- Se observer issue de bevy 0.18 (vide riscos aceitos) provar ser bloqueio sério para hot reload na Semana 2, abrir ADR de revisão (não rebaixar silenciosamente).

## Próximos passos

1. Atualizar SKILL §5 — `bevy_ecs = "0.18"` permanece (já está). Adicionar nota de "Decisão ratificada por ADR-0003-rev2".
2. Popular `crates/ph2d-ecs/src/lib.rs` com wrapper minimal (Plugin trait, World re-export, schedule helpers).
3. Remover `tests/spike/src/bin/c11_flecs.rs` e dep `flecs_ecs` do `tests/spike/Cargo.toml` ao final do spike (preservar até ADR-0019 final).
4. Investigar observer bug de bevy 0.18 na Semana 2 (parte de C4 hot reload setup).
