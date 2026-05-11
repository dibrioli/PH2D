# ADR-0025: Modelo de GameObject — ECS-composition (Unity-style) sobre bevy_ecs

**Status:** Proposed
**Data:** 2026-05-10
**Decisor:** Enio Oliveira Dias Brito (pending)
**Implementador:** Claude Opus 4.7 (1M context)
**Depende de:** [ADR-0003 (bevy_ecs 0.18)](0003-ecs-choice.md), [ADR-0021 (Sim ↔ Present)](0021-simulation-presentation-boundary.md), [ADR-0019 (Luau ratificado)](0019-spike-scripting-output.md)

## Contexto

O SKILL e os ADRs anteriores fixaram o **substrato** (`bevy_ecs 0.18` + dois worlds com extract one-way) e a **linguagem de gameplay** (Luau strict via mlua), mas não definiram a **fachada de autoria** — o que o usuário do editor e o scripter Luau veem como "uma coisa no mundo". Sem essa decisão, três caminhos disputam:

1. **Godot-style** — `Node` polimórfico, herança, código atrás de subclasse, scene tree é a identidade.
2. **Unity-style** — `GameObject` como container plano de Components, composição sem herança, scripts são Components (MonoBehaviour).
3. **Bevy puro** — `Entity` é u64 opaco e não há "objeto"; tudo é componente + system.

A pergunta importa porque é a partir dela que se decide: vocabulário do editor (Hierarchy panel já existe em `screens::hero`), formato de Prefab/Scene no asset pipeline, semântica de "anexar script", semântica de "parent/child", e a forma como `ph2d-bindgen` expõe APIs para Luau e MCP (HR-10 paridade).

## Decisão

**ECS-composition Unity-style.** Entity é o "objeto"; comportamento é composição de Components; hierarquia é o relationship `ChildOf` built-in do bevy_ecs 0.18; script de gameplay é um Component (`LuauScript`); Prefab e Scene são Assets blake3-addressed (HR-6).

### Vocabulário canônico

| Conceito | Tipo Rust | Onde vive | Notas |
|---|---|---|---|
| **Object** (UI), **Entity** (API) | `bevy_ecs::Entity` (re-exportado em `ph2d_ecs::Entity`) | SimWorld | u64 opaco; satisfaz HR-8 diretamente |
| **Component** | tipo com `#[derive(Component)]` + `impl SimComponent` ou `PresentComponent` | SimWorld ou PresentWorld | ADR-0021 separa por trait |
| **Name** | `Name(String)` (SimComponent) | SimWorld | usado por editor + Luau `ph2d.find_by_name` |
| **Transform** | `Transform { pos: WorldPos, rot: f32, scale: Vec2 }` (SimComponent) | SimWorld | hierarquia: `Transform` é local ao pai |
| **GlobalTransform** | `GlobalTransform(Mat3)` (PresentComponent) | PresentWorld | derivado por `propagate_transforms` no extract phase |
| **Hierarquia (parent/child)** | `bevy_ecs::ChildOf` (built-in 0.18) | SimWorld | despawn em cascata grátis; observar via `On<Add, ChildOf>` |
| **Tag** | `Tag(Smol<32>)` (SimComponent) | SimWorld | string curta, pooled; uso por queries `With<Tag>` + match |
| **Script de gameplay** | `LuauScript { bytecode: AssetId, lateral_key: u64 }` (SimComponent) | SimWorld | estado por instância via `state_table(entity)` (HR-16) |
| **Prefab** | `Asset::Prefab(PrefabDoc)` em `ph2d-asset` | AssetDb | bundle de Components serializado postcard + blake3 |
| **Scene** | `Asset::Scene(SceneDoc)` em `ph2d-asset` | AssetDb | lista de Prefab refs + spawn transforms + relações ChildOf |
| **Messaging** | `ph2d.message_send(target, msg, payload)` | ScriptHost::MessageBus | já canônico per ADR-0019; hash interning, FIFO same-sender→same-target |

### Hierarquia

`bevy_ecs 0.18` tem `ChildOf` como relationship first-class. Despawn em cascata, hooks `On<Add, ChildOf>` e `On<Remove, ChildOf>`, e queries via `Query<&Children>` funcionam por construção. **Não criar wrapper `Node` próprio.**

Transform propagation:

```rust
// Em ph2d-render, sistema de extract.
fn propagate_transforms(
    sim_query: Query<(Entity, &Transform, Option<&ChildOf>)>,  // SimWorld read-only
    mut present: ResMut<TransformGraph>,                       // PresentWorld scratch
) { /* topological walk a partir das roots, multiply matrices */ }
```

Roots (entities sem `ChildOf`) recebem `GlobalTransform = Transform`. Filhos compõem `parent.global * child.local`. Computado dentro do `extract!` per ADR-0021 — sim fica imutável.

### Script de gameplay = Component, não classe

Unity tem `MonoBehaviour` herdado de `Object` com `Start()`/`Update()`. PH2D não tem herança e o script é Luau, não Rust. O equivalente:

```rust
#[derive(Component)]
pub struct LuauScript {
    pub bytecode: AssetId,    // blake3 do bytecode pré-compilado (HR-6)
    pub lateral_key: u64,     // chave em state_table(entity) (HR-16)
}
impl SimComponent for LuauScript {}
```

- **Bytecode é asset** — HR-6 content-addressed, deduplica entre instâncias.
- **Estado por instância é lateral storage** — HR-16 enforce POD-like, serializável, determinístico.
- **`Update()` é cooperativo via coroutines** — script chama `ph2d.wait(dt)`; scheduler resume no próximo tick (já implementado em `ph2d-script::Scheduler`).
- **Acoplamento ECS** — script só lê/escreve via `WriteQueue`/`ReadSnapshot` (HR-8); nunca acessa `Entity` direto, nunca recebe handle Rust.

Script pode receber a `Entity` à qual está vinculado como argumento implícito (`self` em Luau idiomático), exposto como u64 opaco.

### Prefab e Scene

```rust
#[non_exhaustive]
pub enum Asset {
    ImageRgba8 { /* existing */ },
    Prefab(PrefabDoc),
    Scene(SceneDoc),
}

pub struct PrefabDoc {
    pub version: u32,                    // HR-14
    pub components: Vec<ComponentBlob>,  // postcard-encoded por type_id
    pub children: Vec<PrefabRef>,        // sub-prefabs com transform local
}

pub struct SceneDoc {
    pub version: u32,                    // HR-14
    pub instances: Vec<PrefabInstance>,  // { prefab: AssetId, transform, overrides }
    pub relations: Vec<ChildOfPair>,     // restored ao spawn
}
```

- **Identidade = blake3 do conteúdo cooked** (HR-6).
- **Versionamento** = HR-14 (campo `version: u32` no head, migração N→N+1 obrigatória).
- **Overrides** = mesmo padrão Unity Prefab Variants (component override por path).
- **Spawn** = função em `ph2d-ecs::scene::spawn_prefab(world: &mut SimWorld, prefab: AssetId, at: Transform) -> Entity`.

### O que NÃO é parte desta decisão

- **`taffy` ou layout 2D para sprites em runtime** — fora de escopo; layout é responsabilidade do editor UI (§11.9), não da scene runtime.
- **Animação** — virá em ADR separado (provavelmente uma `AnimationPlayer` Component + curves como Asset).
- **Networking de scenes** — `ph2d-net` decide o seu próprio formato de snapshot (já citado em §11.8).
- **Naming visível do conceito em UI** — escolha de chamar "Object", "Node" ou "Entity" no editor é UX, decide-se em PR separado (recomendação: "Object" no editor PT-BR, `Entity` na API).

## Consequências

### Aceitas

- **Editor Hierarchy panel renderiza walk em `ChildOf`** — match direto com a estrutura existente em `screens::hero` (Hierarchy region).
- **LLM-friendliness alta** — todo "código de gameplay" é Component + system, formato dominante no training data de bevy/unity.
- **HR-8 satisfeito por construção** — script recebe `Entity` (u64) e usa `ph2d.set(entity, field, value)`. Sem proxy table com `__index` escondendo handles.
- **Save/replay funciona** — todo SimComponent deriva Saveable (HR-14); Scene round-trip = re-spawn de Prefabs + restore de ChildOf.
- **Determinismo (HR-5) intacto** — sem vtable dispatch, sem ordem de iteração dependente de hash (ADR-0022); ChildOf walk é topologicamente estável.
- **Hot reload reset+restore (M7) continua funcionando** — World é a fonte canônica, lateral storage POD-like.

### Negadas

- **Sem `trait Node` polimórfico.** Tentativas de "Sprite extends Node2D extends Node" são bloqueadas em review.
- **Sem `Box<dyn Behavior>` por entity.** Quebraria HR-3 (alloc), ofuscaria queries.
- **Sem componente "children" custom paralelo a `ChildOf`.** Divergência inevitável.
- **Sem hierarquia em PresentWorld.** Hierarquia é sim-side; PresentWorld recebe `GlobalTransform` já achatado.
- **Sem multi-script-por-entity como design primário.** Uma entity pode ter múltiplos LuauScript components apenas via componente diferente (`LuauScriptAi`, `LuauScriptInput`) — não como `Vec<LuauScript>`. Encoraja decomposição em components de Rust quando o caso pede.

### Neutras

- **Naming "GameObject" não aparece no código Rust** — usamos `Entity` (já o tipo). "Object" só na UI/docs em pt-BR.
- **Prefab variants/overrides** — design Unity é razoável; alternativa "scene composition" estilo Godot fica como possível ADR futuro se complexidade crescer.

## Alternativas consideradas

### Godot-style (Node tree polimórfico) — rejeitada

**Pró:** familiar para devs de Godot; tree-as-truth dá ergonomia de scene composition; signals built-in.
**Contra:**
- Conflito com substrate ECS já decidido (ADR-0003).
- Vtable + heap-per-Node viola HR-3 em hot path.
- Ordem de filhos depende de coleção que precisa ser determinística manualmente (HR-5 mais custoso).
- LLM gen quality menor (training data de inheritance-em-Rust é mínimo; ECS-em-Rust é abundante).
- Padrão "anexa script à subclasse" não tem análogo limpo com Luau no host Rust.

### Bevy puro (Entity como u64 sem fachada) — rejeitada

**Pró:** minimal; tudo coerente com substrate.
**Contra:**
- Editor precisa de uma representação de "thing in the world"; Hierarchy panel não pode mostrar `Entity(34298)` cru.
- Sem Prefab first-class, content creation no editor é dolorosa.
- Luau scripter espera endereçar "objetos por nome" (`ph2d.find_by_name("Player")`); a fachada Unity dá esse afford naturalmente.

### Híbrido: SubScene como tipo do bevy_scene — rejeitada

**Pró:** seria pegar emprestado `bevy_scene` upstream.
**Contra:** depende de `bevy_reflect` completo e `bevy_app` (que não usamos); o crate `ph2d-asset` blake3-addressed é mais alinhado com HR-6 do que o `DynamicScene` do bevy.

## Pré-requisitos (auditoria 2026-05-10)

Para começar a implementar este modelo, precisamos das peças abaixo. Lista cruzada com estado real do código:

| # | Item | Estado | Onde | Tamanho estimado |
|---|---|---|---|---|
| 1 | `SimWorld`/`PresentWorld` + `extract!` | ✅ | `ph2d-ecs` (M4) | — |
| 2 | `SimComponent`/`PresentComponent` markers | ✅ | `ph2d-ecs` (M4) | — |
| 3 | `WorldPos` como tipo de posição | ✅ (mas é newtype, não Component) | `ph2d-core::math` | precisa virar Component **ou** ser embutido em `Transform` |
| 4 | `Transform` Component (pos+rot+scale) | ❌ | a criar em `ph2d-core` ou `ph2d-ecs::transform` | ~50 LOC |
| 5 | `GlobalTransform` PresentComponent | ❌ | a criar em `ph2d-render` ou `ph2d-ecs::transform` | ~30 LOC |
| 6 | `propagate_transforms` system | ❌ | a criar (ph2d-render extract phase) | ~100 LOC |
| 7 | `ChildOf` (built-in bevy 0.18) | ✅ (disponível, não exercitado) | re-export em `ph2d_ecs` | adicionar 1 linha de re-export |
| 8 | `Name` Component | ❌ | a criar em `ph2d-ecs::name` | ~20 LOC |
| 9 | `Saveable` trait + derive | ❌ (HR-14 menciona, não implementado) | a criar em `ph2d-core::save` ou crate proc-macro | ~200 LOC (derive) |
| 10 | `LuauScript` Component + bind a lateral storage | ❌ | a criar em `ph2d-script::component` | ~150 LOC |
| 11 | `Asset::Prefab` / `Asset::Scene` variants | ❌ | extender `ph2d-asset::asset` | ~80 LOC + cooker |
| 12 | `spawn_prefab` / `spawn_scene` helpers | ❌ | a criar em `ph2d-ecs::scene` | ~120 LOC |
| 13 | Editor Inspector: edição de Components | 🟡 (UI existe; binding ECS não) | `ph2d-editor::screens::hero` | wiring novo |
| 14 | Editor Hierarchy: walk em `ChildOf` | 🟡 (UI existe com fixture; binding não) | `ph2d-editor::screens::hero` | wiring novo |
| 15 | `ph2d-bindgen` schema para Component | 🟡 (existe, não testou Component) | `tools/ph2d-bindgen` | extender + HR-10 |
| 16 | `ph2d.find_by_name`/`ph2d.spawn` Luau bindings | ❌ | extender `ph2d-script::host` | ~50 LOC |

**Crítico (bloqueia tudo):** itens 4, 5, 6, 8 — `Transform`/`GlobalTransform`/`Name` + propagation. Sem isso não tem "scene" sequer mínima.

**Importante (bloqueia Luau gameplay):** itens 10, 16 — `LuauScript` + bindings de spawn/find.

**Importante (bloqueia content creation):** itens 11, 12, 13, 14 — Prefab/Scene + editor wiring.

**Diferível:** item 9 (`Saveable`) — pode ser feito quando ph2d-save sair de stub (M13+); por enquanto serialização via `serde + postcard` direto em components individuais funciona como ponte. **Mas:** decidir formato definitivo antes que muitos components proliferem sem versioning, senão temos save corruption depois.

## Próximos passos (proposta de marcos)

Subdividir em três marcos pequenos sobre o terreno atual:

### M14.1 — Transform & Hierarchy (pré-requisito mínimo, ~2-3 dias)

1. Criar `ph2d-ecs::transform` com `Transform` (SimComponent) e `GlobalTransform` (PresentComponent).
2. Criar `ph2d-ecs::name` com `Name(String)` (SimComponent).
3. Re-export `bevy_ecs::hierarchy::ChildOf` em `ph2d_ecs::ChildOf`.
4. Criar `propagate_transforms` system em `ph2d-render::extract` (topological walk).
5. Adaptar `Sprite` em `ph2d-render` para extrair de `Transform`, não de `WorldPos` direto (ou tornar `WorldPos` deprecated/alias).
6. Teste: hierarquia de 3 níveis, `GlobalTransform` correto, despawn de pai cascateia.
7. Teste de determinismo: 2 runs do mesmo seed em Linux+Mac produzem mesmo state hash (HR-5).

### M14.2 — LuauScript Component (~3-5 dias)

1. Definir `LuauScript { bytecode: AssetId, lateral_key: u64 }` em `ph2d-script::component`.
2. Estender `ScriptHost` para resolver bytecode via `AssetDb`.
3. Implementar `state_table(entity)` em `lateral_storage` com restrições HR-16 (POD-like, pairs_sorted).
4. Bindings Luau: `ph2d.spawn(prefab_id)`, `ph2d.despawn(entity)`, `ph2d.find_by_name(name)`, `ph2d.attach_script(entity, script_id)`.
5. Hot reload: bytecode reload → re-run script's `init()` em entities afetadas (reset+restore por entity, não global).
6. Teste: 100 entities com mesmo script, mutate via script, snapshot determinístico.

### M14.3 — Prefab & Scene Assets (~4-7 dias)

1. Estender `Asset` enum com `Prefab(PrefabDoc)` + `Scene(SceneDoc)`.
2. Cooker em `tools/asset-cooker` para formato source (JSON/RON dev) → cooked (postcard + blake3).
3. `spawn_prefab` / `spawn_scene` em `ph2d-ecs::scene` (não em `bevy_scene`).
4. Editor wiring: Hierarchy panel mostra `ChildOf` real; Inspector edita Components reais.
5. Save format: snapshot da Scene = lista de Entity → Components serializados; versionado per HR-14.
6. Teste: cook prefab → spawn → snapshot → restore = bit-identical.

## Não retroceder

- "GameObject = Entity + Components" é decisão final para v0.1.
- Tentativa de reintroduzir herança/polimorfismo exige novo ADR superseding este.
- Adicionar nova categoria de "Object" no editor (ex: "Resource" estilo Godot) exige ADR avaliando se cabe como Asset (HR-6) primeiro.

## Referências

- [ADR-0003 — bevy_ecs 0.18](0003-ecs-choice.md) (substrato)
- [ADR-0019 — Luau ratificado](0019-spike-scripting-output.md) (linguagem)
- [ADR-0021 — Simulation ↔ Presentation](0021-simulation-presentation-boundary.md) (worlds)
- [ADR-0022 — No HashMap em simulation](0022-no-hashmap-in-simulation.md) (determinismo de iteração)
- SKILL §11.7 (Scripting), §14 (padrão "Adicionar component"), HR-3, HR-5, HR-6, HR-8, HR-14, HR-16
- Defold prefab + script-component pattern (referência conceitual)
- Bevy `bevy_hierarchy` (`ChildOf` 0.18) — usado direto, sem wrapper
