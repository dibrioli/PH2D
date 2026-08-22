# Auditoria — o modelo de objeto, de propriedade e de asset que o PH2D TEM hoje (Fase A)

> **O que este doc é:** o levantamento **factual** do código, em 2026-08-21, para o briefing de
> *"modelo de objetos composáveis + Asset Browser"*. Toda afirmação aqui aponta arquivo (e linha,
> quando ajuda). **Não há recomendação nenhuma** — a decisão é da Fase D, e é do Enio.
>
> **O que este doc NÃO é:** não é pesquisa externa (Fase B), não é escolha de arquitetura, e não
> resolve ambiguidade: onde o código não responde, este doc escreve *"não determinado"* e diz por quê.
>
> **Método:** leitura direta de `crates/ph2d-ecs`, `-asset`, `-render`, `-nodegraph`, `-script`,
> `-panel-inspector`, `-panel-motion-params`, `-panel-hierarchy`, `shells/desktop/src/{project*,undo,
> init,render_loop/*,vec_component*,vec_variants}.rs`, ADR-0025/0037/0074/0021/0075, mais **um spike
> descartável** de medição (§7) que foi rodado e **apagado** — a árvore está limpa.
>
> **Irmão:** [`00_levantamento_componentes.md`](00_levantamento_componentes.md) responde *"que
> componentes de GAMEPLAY faltam"*. Este responde *"que máquina existe para carregar qualquer
> componente, e o que ela não faz"*. Os dois convergem no mesmo item nº 1 (§4).

---

## 📍 Índice — salte, não leia

| § | assunto |
|---:|---|
| **§0** | Veredito sobre os 5 sintomas e as 7 premissas do briefing |
| **§1** | A1 — Modelo de dados: ECS, registro, identidade, **e a relação Entidade ↔ nó de DAG** |
| **§2** | A2 — A Sprite, campo a campo, medida |
| **§3** | A3 — Hierarquia, cena, serialização |
| **§4** | A4 — Propriedades e Inspector (**e o inspector derivado que já existe**) |
| **§5** | A5 — Assets e I/O |
| **§6** | A6 — Restrições transversais: ADRs, undo, threading |
| **§7** | As medições (spike descartável) |
| **§8** | Ambiguidades que o código NÃO resolve |

---

## §0 — Veredito sobre as premissas do briefing

### §0.1 — Os 5 sintomas relatados

| # | Sintoma relatado | Veredito | Evidência |
|---|---|---|---|
| 1 | *"Não existe objeto básico vazio"* | ⚠️ **REFUTADO em parte** | Existe e é criável: `Hierarchy → botão direito na linha → "Add child"` faz `spawn((Transform::IDENTITY, Name::new(nome), ChildOf(pai)))` — [`render_loop/hierarchy.rs:245-252`](../../shells/desktop/src/render_loop/hierarchy.rs). É literalmente o GameObject vazio do Unity. **O que não existe é criá-lo na RAIZ**: o único gesto exige um pai. E o botão "Add" do cabeçalho do painel (`HIERARCHY_ADD`, tooltip *"Add entity"*) é **pintado, registrado no hit-index, e não tem handler nenhum** — ver §0.3 |
| 2 | *"Não existe mecanismo de Add Component"* | ⚠️ **REFINADO — o transporte existe, falta a PONTA** | `EditorCommand::{SetComponent, RemoveComponent, Spawn, Despawn, Reparent}` existe, é type-erased por `ComponentTypeId`, e está **wired** ([`scene/commands.rs:31-59`](../../crates/ph2d-ecs/src/scene/commands.rs); drenado em [`render_loop/inspector_commits.rs`](../../shells/desktop/src/render_loop/inspector_commits.rs) em ~10 sítios). `RemoveComponent` **já é usado em produção** para destacar componentes opcionais de ordenação ([`inspector_ordering.rs:163`](../../shells/desktop/src/render_loop/inspector_ordering.rs)). Faltam **duas** coisas, e são pequenas e precisas: (a) o `ComponentRegistry` **não tem construtor de default** (§1.2) — não há como um UI genérico produzir os bytes de um componente que ele não conhece; (b) não há UI de catálogo |
| 3 | *"A Sprite é monolítica"* | ⚠️ **REFINADO — ela é grande, mas já foi fatiada por ADR, e o corte está CONGELADO** | 20 campos, **200 bytes medidos** (§7). Mas [ADR-0074](../architecture/decisions/0074-sprite-component-boundary.md) já decidiu a regra dos 3 lugares, congelou `Sprite` em **20 campos** e o teto de componentes opcionais em **32**, e a fatiação já aconteceu: sorting/masking/sampling/blend/visibilidade são **19 componentes ECS opcionais separados** ([`scene/registry.rs:230-246`](../../crates/ph2d-ecs/src/scene/registry.rs)). O problema restante não é "onde a propriedade mora" — é que **o Inspector pinta as 12 seções à mão** (§4) |
| 4 | *"Não existem objetos compostos reutilizáveis"* | ❌ **REFUTADO — existe um, completo, e é do modelo do Figma** | `VecComponentMain` (mestre) + `VecInstance { main, overrides }` + `OverrideSlot` + **variants** — [`vec_component.rs`](../../crates/ph2d-ecs/src/vec_component.rs), [`vec_variants.rs`](../../shells/desktop/src/vec_variants.rs), verbos em [`vec_component_edit.rs`](../../shells/desktop/src/vec_component_edit.rs). Tem Create/Place/Detach/Reset/Update Main/Swap Main/visibilidade por peça/variant, geometria **derivada por frame** (não copiada), overrides **esparsos e canonicamente ordenados**. ⚠️ **Escopo: caminhos VETORIAIS apenas** — não alcança entidades/sprites. Detalhe em §3.4 |
| 5 | *"Não existe FileSystem/Asset Browser"* | ✅ **CONFIRMADO, e é ainda mais vazio do que parece** | `grep -ri "asset_browser\|AssetBrowser\|asset_library"` sobre `crates/` + `shells/` = **zero linhas**. O chip "Assets" da topbar (`TOPBAR_RIGHT_ASSETS`, tooltip *"Asset library"*) só imprime o próprio nome em `stdout` ([`topbar/mod.rs:189-196`](../../crates/ph2d-editor-core/src/screens/hero/topbar/mod.rs)). Existe **mockup** (`docs/design/screens/05-asset-browser.html`, 22 KB) e **nada de código**. Detalhe em §5 |

### §0.2 — As 7 premissas de contexto do briefing §1

| Premissa | Veredito | Evidência |
|---|---|---|
| *"ECS estilo Bevy"* | 🔧 **REFINE** | Não é *"inspirado em"*: é **`bevy_ecs = "0.18"` standalone**, sem o resto do Bevy ([`ph2d-ecs/Cargo.toml`](../../crates/ph2d-ecs/Cargo.toml), [ADR-0003](../architecture/decisions/0003-ecs-choice.md)). Storage = archetype (o do bevy). `ChildOf`/`Children` são os relacionamentos nativos do 0.18 |
| *"Tudo é nó de DAG desde o dia zero — pintura, vetor, shader nodes, motion, sculpt"* | ❌ **REFUTADO, e isto muda o desenho** | `ph2d-nodegraph` **não depende de `bevy_ecs`** e não menciona `Entity` uma única vez; `NodeId` é um `u32` próprio ([`graph.rs:35`](../../crates/ph2d-nodegraph/src/graph.rs)). Os consumidores do DAG são **9 crates + 125 `ph2d-node-*`**, e todos são do domínio **Motion/expr/tokens**: `eval-motion`, `expr`, `gpu-cook`, `motion-diagnose`, `motion-doc`, `panel-motion-graph`, `token-math`, `tokens-dtcg`. **Painter, Vector, Flip e Sculpt3D não usam o DAG** — e no Vector isso é decisão escrita: os Live Path Effects são *"a per-path stack, **not** a node graph"* ([ADR-0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md)). **Não existe "shader node" no repo** |
| *"IDs estáveis para objetos em todo o sistema"* | ❌ **REFUTADO — é a premissa mais cara a corrigir** | O doc-comment de [`name.rs:48-89`](../../crates/ph2d-ecs/src/name.rs) diz textualmente: *"**O ECS não tem id estável.** `Entity::to_bits()` é um id de ALOCAÇÃO"*. O que existe é `stable_name_id(&str)` = FNV-1a do `Name` — e o mesmo comentário avisa: *"**Renomear um objeto muda o id dele**, e portanto desliga o que apontava para ele"*, com a nota de que a cura seria *"um `StableId` de verdade atribuído no spawn"* — que **não existe**. Detalhe em §1.3 |
| *"Formato de save estilo Rive Core, com type keys"* | 🔧 **REFINE — metade é verdade, e a outra metade é o oposto** | As **type keys existem**: `ComponentTypeId = blake3(nome_canônico)[0..8]` ([`registry.rs:52`](../../crates/ph2d-ecs/src/scene/registry.rs)), e `WorldSnapshot`/`PrefabDoc` são listas de `(type_id, bytes)`. Mas o **arquivo de projeto** é postcard **posicional** com uma versão monolítica `PROJECT_SCHEMA` (hoje **84**), e o load **RECUSA** qualquer arquivo cuja versão não seja exatamente essa ([`project_load.rs:46-50`](../../shells/desktop/src/project_load.rs)) — o oposto exato da propriedade do Rive Core (pular chave desconhecida). Ver §3.3 |
| *"Sistema de propriedades tipado unificado"* | ❌ **REFUTADO como "unificado"; existe UM, e é de nós** | Não há reflexão, nem derive de painel, nem schema por componente ECS. O `ComponentRegistry` só sabe (de)serializar bytes opacos. O que existe é o vocabulário do **Motion**: `ParamSpec { name, default: f32 }` no manifesto do nó ([`node.rs:52-55`](../../crates/ph2d-nodegraph/src/node.rs)) + o `ParamRow` de **12 variantes** que o painel de params pinta (§4.3). Isso é um sistema de propriedades tipado real — **para nós, não para componentes** |
| *"Vello para o chrome; wgpu bespoke para o canvas"* | ✅ **CONFIRMA** | SKILL §11.9 + `ph2d-editor-core/src/paint.rs`; o canvas é `ph2d-render`/`ph2d-paint-gpu`/`ph2d-flip-render`/`ph2d-mesh-render` |
| *"A engine cria os próprios assets; não há round-trip externo"* | ✅ **CONFIRMA — e o cooker está mais vazio do que o SKILL diz** | `tools/asset-cooker/src/` tem **`lib.rs`, `main.rs` e `texture/`** — só isso. A tabela de importadores do SKILL §11.10 (Aseprite, Tiled, Spine, Lottie, glTF…) é **aspiração, não estado** |

### §0.3 — Três achados que ninguém pediu e que mudam a conta

1. **`HIERARCHY_ADD` é um botão morto.** Ele é registrado ([`panel-hierarchy/populate.rs:21,56`](../../crates/ph2d-panel-hierarchy/src/populate.rs)), pintado e hit-indexado ([`paint.rs:128-133`](../../crates/ph2d-panel-hierarchy/src/paint.rs)), tem tooltip *"Add entity"* — e **nenhum `Click(HIERARCHY_ADD)` é tratado em lugar nenhum** do repo (varredura sobre `crates/` + `shells/`: as únicas ocorrências são a definição do id, o tooltip, dois comentários e dois testes de colisão de id). O único caminho vivo para criar entidade é o menu de contexto de uma linha.
2. **`spawn_prefab`/`spawn_scene`/`PrefabDoc`/`SceneDoc` têm ZERO consumidores de produção.** A varredura só encontra as próprias definições, os testes da crate, e `tests/end_to_end_m14.rs`. O `AssetDb` nunca recebe um `Asset::Prefab` fora de teste. ⚠️ Isto **corrige** a leitura de *"`PrefabDoc`+`spawn_prefab` prontos, ZERO UI"* do [inventário](pesquisa/inventario_ph2d.md): eles estão prontos e **nunca foram exercitados pelo caminho real de save**, que é outro (`ProjectFile`, §3.3).
3. **`extract_component_snapshot` também tem ZERO consumidores.** A função que produz, para uma entidade, a lista completa `(nome_canônico, type_id, bytes)` de todo componente registrado presente — o substrato exato de um Inspector genérico — existe desde M14.3c ([`scene/snapshot.rs:245-274`](../../crates/ph2d-ecs/src/scene/snapshot.rs)), é testada, e o único lugar que a menciona é o `pub use` do `mod.rs`.

---

## §1 — A1: Modelo de dados

### §1.1 — ECS, storage, custo

- **Qual ECS:** `bevy_ecs = "0.18"` standalone ([`crates/ph2d-ecs/Cargo.toml`](../../crates/ph2d-ecs/Cargo.toml)), ratificado por [ADR-0003](../architecture/decisions/0003-ecs-choice.md). Storage = **archetype** (o padrão do bevy). Não é arena própria, não é `hecs`/`flecs`/`shipyard`.
- **Dois mundos:** `SimWorld` e `PresentWorld` são newtypes opacos sobre `bevy_ecs::World`, com as traits-marcador `SimComponent`/`PresentComponent` e ponte **one-way** pela macro `extract!` ([ADR-0021](../architecture/decisions/0021-simulation-presentation-boundary.md), [`sim.rs`](../../crates/ph2d-ecs/src/sim.rs), [`present.rs`](../../crates/ph2d-ecs/src/present.rs)). Todo componente de autoria vive no `SimWorld`.
- **Custo de add/remove em runtime — MEDIDO** (§7): `insert` de `Sprite` = **70–91 ns/op**, `remove` = **31–33 ns/op**, em release, sobre mundos de 1 k e 10 k entidades. É o custo de um archetype move e **não é o gargalo de nada** nesta escala. ⚠️ O gargalo real está noutro sítio: o **undo captura o mundo inteiro por frame** (§6.2 e §7).

### §1.2 — O `ComponentRegistry` — e o buraco exato

[`crates/ph2d-ecs/src/scene/registry.rs`](../../crates/ph2d-ecs/src/scene/registry.rs). Registro **manual** (não `inventory` — o motivo está no cabeçalho do arquivo: wasm32 + grep-ability + HR-17).

```rust
pub struct ComponentTypeEntry {           // registry.rs:102-108
    pub canonical_name: &'static str,     // "ph2d::ecs::Transform"
    pub type_id: ComponentTypeId,         // blake3(nome)[0..8]
    pub insert_from_bytes: InsertFromBytesFn,
    pub serialize: SerializeFn,
    pub remove: RemoveFn,
}
```

- **`ComponentTypeId = blake3(canonical_name)[0..8]`**, deliberadamente **não** `std::any::TypeId` (instável entre rustc).
- **`BTreeMap` por id e por nome** → iteração ordenada (HR-5).
- **Registro dá persistência E undo de graça:** `world_to_snapshot` itera o registro; o snapshot É a unidade do undo E o corpo do save.
- ⚠️ **Componente não registrado é DESCARTADO EM SILÊNCIO** pelo snapshot — o próprio arquivo lista os que já se perderam assim (`Locked`, `GroupedChildren`, `VecPathRef`).
- ⛔ **O buraco:** a vtable tem `insert_from_bytes`, `serialize` e `remove` — e **nenhum `default_bytes` / `insert_default`**. Um UI genérico de *"Add Component"* recebe do catálogo um `type_id` e **não tem como produzir um valor inicial** sem conhecer o tipo Rust em tempo de compilação. É o elo que falta, e é ~1 fn pointer por tipo.

**Contagem registrada hoje = 91**, somando quatro chamadas, todas no boot ([`init.rs:485-497`](../../shells/desktop/src/init.rs)):

| `register_*` | # | Arquivo | Gate de contagem |
|---|---:|---|---|
| `register_ecs_components` | **57** | `ph2d-ecs/src/scene/registry.rs:222` | `register_ecs_components_populates_registry` (assert 57) |
| `register_render_components` | **1** | `ph2d-render/src/registry.rs:14` | assert 58 (57+1) |
| `register_script_components` | **1** | `ph2d-script/src/registry.rs:14` | assert 58 (57+1) |
| `register_physics_components` | **32** | `ph2d-physics-ecs/src/lib.rs:146` | `registers_every_physics_component` (assert 32) |

⚠️ **`register_script_components` NÃO é chamado no boot.** O `init.rs` chama três dos quatro — ECS, render, physics. Consequência: **`LuauScript` não entra no `WorldSnapshot`**, logo não é salvo nem desfeito. Não determinado se é intencional (o script não tem UI nenhuma, §4.4) ou esquecimento; registrado aqui como fato.

### §1.3 — Identidade: três esquemas, nenhum é um id estável de objeto

| Esquema | O que é | Onde | Sobrevive a… |
|---|---|---|---|
| `Entity::to_bits()` | id de **alocação** do bevy | runtime, handles opacos (HR-8) | nada — o restore do undo **respawna tudo** com bits novos ([`undo.rs:109-117`](../../shells/desktop/src/undo.rs)) |
| `stable_name_id(&Name)` | FNV-1a do nome, `0` reservado | [`name.rs:80-89`](../../crates/ph2d-ecs/src/name.rs) | undo ✅, load ✅, **renomear ❌** |
| índice de linha no snapshot | posição no `Vec<EntitySnapshotRow>` | [`save.rs:32-39`](../../crates/ph2d-ecs/src/scene/save.rs), [ADR-0037](../architecture/decisions/0037-stable-entity-wire-id-scenedoc.md) | um round-trip ✅, edição estrutural ❌ |

Consumidores reais do `stable_name_id` — **duas famílias** (correção de 2026-08-21, noite: `VecLabel.host` é um `VecPathId` cru, [`vec_label.rs:35`](../../crates/ph2d-ecs/src/vec_label.rs), não um hash de nome): **timeline** (`WireId` em [`binding.rs:25`](../../crates/ph2d-timeline/src/binding.rs), `timeline_persist.rs:38`, `frame_solve.rs:139/218/251`, `persist.rs:28/54/88`) e **física** (`PhysicsJoint.body_a/b` em [`joint.rs:117-121`](../../crates/ph2d-physics-ecs/src/joint.rs), `PulleyWheel.rope/.body` em `components/rope.rs:99/146`, `bridge/joints.rs:152/315`, `bridge/rope.rs:130`, `joint_group.rs:139`, + 12 sítios de inspector na shell). ~26 linhas de produção em 13 arquivos; 431 contando testes/smokes/c9. A unicidade do nome é **imposta pelo editor**, não pelo tipo ([`shells/desktop/src/name_unique.rs`](../../shells/desktop/src/name_unique.rs)); o `Name` em si documenta *"names are **not** required to be unique"*.

⚠️ **Duas coisas guardam bits de entidade dentro dos bytes de um componente**, que é o padrão que o próprio repo declara venenoso:
- `LuauScript.lateral_key = entity.to_bits() XOR hash(bytecode)` ([`ph2d-script/src/component.rs:49-62`](../../crates/ph2d-script/src/component.rs)). Mitigado hoje porque `LuauScript` não está no snapshot (§1.2) e porque `derive_lateral_key` **não é chamado em lugar nenhum** fora do doc-comment.

Nenhum remapeamento de id acontece na carga: o `snapshot_to_world` spawna do zero e reconstrói `ChildOf` por índice ([`save.rs:201-227`](../../crates/ph2d-ecs/src/scene/save.rs)); as pontes por identidade (`VecPathRef`, `FlipObjectRef`) são reconstruídas por varredura (`rebuild_map`).

### §1.4 — ⭐ Entidade ECS × nó do DAG — a pergunta mais importante da auditoria

**Resposta: são duas hierarquias PARALELAS e DISJUNTAS, sem ponte de tipo. Não há acoplamento nenhum a desfazer — e não há ponte nenhuma a herdar.**

A prova é de dependências, não de leitura:

- `ph2d-nodegraph/Cargo.toml` tem **uma** dependência: `rayon`. **Não** depende de `bevy_ecs`, **não** depende de `ph2d-ecs`.
- `grep -rn "bevy_ecs\|Entity" crates/ph2d-nodegraph/src/` → **zero linhas**.
- `NodeId(pub u32)` ([`graph.rs:35`](../../crates/ph2d-nodegraph/src/graph.rs)) é um índice do grafo, gerado e persistido pelo **formato textual** do `ph2d-motion-doc` — outro espaço de nomes, outro arquivo (`ProjectFile.motion`, que é **texto**), outro undo (`MotionHistory`).
- Quem depende de `ph2d-ecs`: **21 crates**. Quem depende de `ph2d-nodegraph`: **134** (125 delas são `ph2d-node-*`). **A interseção é `shells/desktop` e mais nada.**

A ligação entre os dois universos é feita **só na shell**, por bake explícito: `motion_object_bake.rs`, `motion_flip_bake.rs`, `sculpt3d_bake.rs`. É uma cópia de resultado, não uma identidade compartilhada.

**Corolário para o desenho:** *"componente ECS, nó do DAG e item do inspetor são a mesma abstração?"* — hoje são **três** abstrações com **três** espaços de id, **três** undos e **três** formatos de persistência. Qualquer proposta que os unifique está propondo uma **fusão**, não uma limpeza; e qualquer proposta que os mantenha separados está descrevendo o que já existe.

| | Espaço de id | Persistência | Undo | Inspector |
|---|---|---|---|---|
| Entidade | `Entity` (alocação) + `Name` | `WorldSnapshot` (postcard, type keys) dentro de `ProjectState` | fila global por diff | `ph2d-panel-inspector` (artesanal) |
| Nó do DAG | `NodeId(u32)` | `ProjectFile.motion` (**texto**, diffável por linha) | `MotionHistory` próprio | `ph2d-panel-motion-params` (**derivado**) |
| Caminho vetorial | `VecPathId` | `VecScene` dentro de `ProjectState` | fila global (mesma) | `ph2d-panel-vector` (artesanal) |

---

## §2 — A2: A Sprite

**Arquivo:** [`crates/ph2d-render/src/sprite/component.rs`](../../crates/ph2d-render/src/sprite/component.rs) (449 LOC).
**Tamanho medido:** `size_of::<Sprite>() = 200 bytes` (§7). `SpriteSource` sozinho = 36 B (o variant maior carrega um `LogicalTextureId([u8;32])`).

### §2.1 — Os 20 campos, agrupados por afinidade

| Grupo | Campos | Bytes | Fração de objetos que usaria | Poderia ser componente à parte? |
|---|---|---:|---|---|
| **Schema** | `version` | 4 | 100 % | não (é o marcador HR-14) |
| **Fonte** | `source` | 36 | 100 % | não (é a razão de a Sprite existir) |
| **Geometria intrínseca** | `size` | 8 | 100 % | não |
| **Pivô / origem** | `anchor`, `centered`, `offset` | 8+1+8 | ~15 % usa não-default | ⚠️ candidato: 17 B com default benigno, mas §2.5 do ADR-0074 diz *"TODO sprite tem origem"* |
| **Cor** | `tint`, `self_tint`, `per_corner_tint`, `tint_fill`, `opacity` | 16+16+**64**+1+4 = **101 B** | `per_corner_tint` (64 B) é o gradiente de 4 cantos — uso raro | ⚠️ **o maior candidato isolado**: `per_corner_tint` sozinho é **32 % dos 200 B** e é identidade em quase todo sprite |
| **Espelhamento** | `flip_x`, `flip_y` | 2 | ~20 % | marcadores plausíveis |
| **Folha inline** | `hframes`, `vframes`, `frame` | 12 | só sprites animados | ⚠️ candidato claro (é o embrião do `AnimatedSprite` do §3 do levantamento) |
| **Região** | `region_enabled`, `region_rect`, `region_filter_clip` | 1+16+1 | uso raro fora de atlas | ⚠️ candidato |
| **Runtime** | `premultiplied` (`#[serde(skip)]`) | 1 | interno | é dica de render, não estado |

⚠️ **As frações da coluna 4 são ESTIMATIVAS, não medição** — o repo não tem corpus de projetos salvos para contar. Estão aqui porque o briefing pediu; qualquer decisão que dependa delas precisa medi-las antes.

### §2.2 — O que já foi decidido sobre fatiar a Sprite

[ADR-0074 — *Sprite struct vs Component ECS*](../architecture/decisions/0074-sprite-component-boundary.md), **Accepted**, com spec normativa em [`docs/Sprite_projeto/02_components_ortogonais.md`](../Sprite_projeto/02_components_ortogonais.md). Ele já traz:

- a **regra dos 3 lugares** (campo do struct · componente ECS anexável · derivado por sistema/nó);
- o **teste decisivo**: *"ausência ≠ default explícito?"* → componente;
- **anti-padrões banidos por nome**, com a fonte de cada um: `image_*` do GameMaker (sopa), mixins do Phaser (state-stuffed), 9-Patch como objeto separado do Construct, `Sprite.material: Option<…>`, `Sprite.z_index`, `Sprite.skew`;
- **caps congelados** (§2.11): `Sprite` = **20 campos FROZEN v4**; componentes opcionais do Sprite Inspector ≤ **32**; bump exige amendment.

**Ou seja: a fatiação da Sprite não é uma pergunta aberta de arquitetura — é uma pergunta de EXECUÇÃO contra uma regra que já existe.** A parte já feita: 19 componentes opcionais de ordenação/máscara/amostragem/blend já saíram do struct e estão registrados.

### §2.3 — Outros tipos monolíticos com o mesmo problema

- ❌ **Camada / grupo / texto / curva / osso** — não existem como tipos monolíticos: grupo é entidade com filhos; texto e curva são a família `Vec*` (31 componentes granulares); osso não existe.
- ⚠️ **`PlatformPlayer`** é o candidato real: uma seção de **19 números + 3 botões** no Inspector, com a contagem gateada por `PLAYER_ROW_COUNT` ([`panel-inspector/src/lib.rs`](../../crates/ph2d-panel-inspector/src/lib.rs)).
- ⚠️ **`PaintedDocument`, `TimelineDoc`, `FlipDoc`, o blob `sculpt`** — não são componentes, são **documentos inteiros** pendurados no `ProjectFile`. É a mesma questão noutra escala (§5.2).

---

## §3 — A3: Hierarquia, cena, serialização

### §3.1 — Parentesco e transform

- Parentesco = `bevy_ecs::hierarchy::{ChildOf, Children}` **nativos do 0.18**, re-exportados por `ph2d_ecs`. Decisão explícita de [ADR-0025](../architecture/decisions/0025-gameobject-model.md): *"**Não criar wrapper `Node` próprio**"*. Despawn cascateia de graça.
- Propagação: `propagate_transforms` ([`transform.rs:526`](../../crates/ph2d-ecs/src/transform.rs)) faz um walk DFS iterativo do `SimWorld` e escreve `GlobalTransform` no `PresentWorld` (`propagate_transforms_into_present`, linha 599). Zero-alloc gateado por [`propagate_no_alloc.rs`](../../crates/ph2d-ecs/tests/propagate_no_alloc.rs) (HR-3).
- Ordem das raízes: `RootOrder(u32)` explícito, desempate por `to_bits()` ([`snapshot.rs:161-177`](../../crates/ph2d-ecs/src/scene/snapshot.rs)). *"Não se escolhe um desempate melhor; não se tem empate."*

### §3.2 — Os dois formatos de cena que coexistem

| | `PrefabDoc`/`SceneDoc` | `WorldSnapshot` |
|---|---|---|
| Arquivo | [`ph2d-asset/src/{prefab,scene}.rs`](../../crates/ph2d-asset/src/prefab.rs) | [`ph2d-ecs/src/scene/save.rs`](../../crates/ph2d-ecs/src/scene/save.rs) |
| Para quê | conteúdo autorado, referências por `AssetId`, overrides estilo Unity | captura total de estado (save + rollback) |
| Identidade da entidade | índice em `instances[]` | índice em `entities[]` |
| Overrides | **sim** — `PrefabRef.overrides: Vec<ComponentBlob>`, aplicados DEPOIS dos do filho ([`spawn.rs:97-99`](../../crates/ph2d-ecs/src/scene/spawn.rs)) | não |
| Aninhamento | **sim** — `PrefabDoc.children: Vec<PrefabRef>`, recursivo | n/a |
| Versão | `PrefabDoc::VERSION` / `SceneDoc::VERSION` própria | `WorldSnapshot::VERSION = 1` |
| **Usado em produção** | ❌ **não** (§0.3) | ✅ sim — é a espinha do undo E do save |

⚠️ O modelo de override do `PrefabDoc` é **estrutural, granularidade de componente inteiro** ("substitui o blob do tipo X neste filho"), não por propriedade nomeada. Ele **não** distingue "o artista mudou este campo" de "este componente veio inteiro do override".

### §3.3 — O arquivo de projeto real

[`shells/desktop/src/project.rs`](../../shells/desktop/src/project.rs) — o formato é `(PROJECT_SCHEMA, ProjectFile)` em postcard.

`ProjectState` (a unidade do undo, 5 campos): `world: WorldSnapshot` · `vec: VecScene` · `flip: FlipDoc` · `guides: GuideSet` · `ui_states: StateSets`.

`ProjectFile` (11 campos): `state` · `assets` (pixels dos sprites) · `painted` (documentos do Painter) · `motion` (**texto**) · `timeline` (postcard) · `physics` · `tokens` · `settings` · `sculpt` (`Vec<u8>` opaco) · `baked_forms` · `player_tape`.

⚠️ **Cada campo fora do `ProjectState` está fora por uma razão escrita: undo próprio.** O padrão do repo é *"um Ctrl+Z do canvas não deve rebobinar a gravidade/o grafo/a animação"*.

⚠️ **`PROJECT_SCHEMA = 84`** ([`project_schema.rs:515`](../../shells/desktop/src/project_schema.rs)) com a escada de 84 degraus documentada ao lado. **Postcard é posicional** ⇒ acrescentar um campo em qualquer lugar bumpar o número, e o load **recusa** tudo que não bata exatamente:

```rust
if ver != PROJECT_SCHEMA {                                  // project_load.rs:46
    eprintln!("[proj] schema {ver} != {PROJECT_SCHEMA} — recusado");
```

**Não existe uma única função de migração de projeto no repo.** HR-14 pede migração N→N+1; a prática é hard-break. Isto é o fato mais determinante para qualquer plano que altere a forma de um componente já salvo.

### §3.4 — ⭐ Instanciamento, referência e reuso que JÁ existem

**a) Componentes vetoriais (o modelo do Figma, completo)** — [`crates/ph2d-ecs/src/vec_component.rs`](../../crates/ph2d-ecs/src/vec_component.rs):

```rust
pub struct VecComponentMain;                      // marcador: presença é o booleano
pub enum OverrideSlot { Fill([u8;4]), Hidden }    // vocabulário FECHADO, 2 espécies
pub struct InstanceOverride { sub: u64, slot: OverrideSlot }   // `sub` endereça a peça NO MESTRE
pub struct VecInstance { main: u64, overrides: Vec<InstanceOverride> }
```

Propriedades que o código já garante, e que valem mais que a lista de features:

1. **A instância não é cópia:** a geometria é derivada por frame a partir da sub-árvore do mestre e assada na pose da instância. *"Editar o mestre propaga"* é verdade **por construção**, não por um passe que alguém esquece.
2. **O override endereça a peça NO MESTRE** (`sub` é o `VecPathId` do mestre), *"o que faz o override sobreviver a editar o mestre: a peça continua a mesma peça mesmo que a geometria dela mude por inteiro"*.
3. **Overrides esparsos** — só o que difere viaja; `reset()` limpa.
4. **A ordem da lista é LEI**, mantida por `VecInstance::set` via `binary_search`, porque o `canonicalize` do undo compara **bytes**: duas instâncias logicamente iguais em ordens diferentes virariam um passo de undo espúrio por frame.
5. **Variants existem e são DERIVADOS, não declarados** ([`vec_variants.rs`](../../shells/desktop/src/vec_variants.rs)): o conjunto de variants é *"os mestres irmãos que pendem do mesmo pai"*, e os EIXOS saem do `Name` (`Size=Small, State=Idle`) — **zero schema novo**. Trocar de variant é o `Swap Main` restrito aos irmãos. Combinação inalcançável **não é oferecida** como chip.
6. **Verbos com UI:** Create · Place · Detach · Reset · Update Main · Swap Main · visibilidade por peça (cap `MAX_INSTANCE_PIECES = 16`) · Variant.

⚠️ **Limites, para não superestimar:** vive no espaço de `VecPathId`, não de `Entity`; o vocabulário de override tem **duas** espécies (`Fill`, `Hidden`); o cap de peças visíveis na UI é 16; e o comentário do `OverrideSlot` registra que o `PropPath` genérico que o plano previa **não foi construído** — *"quando a W4b chegar, ela estende ESTA lista ou a absorve; o que não pode acontecer é nascerem duas"*.

**b) Outras formas de reuso existentes:** `Mask2D`/`ClipChildren` (referência hierárquica), `VecTextPath`/`VecPatternPath`/`VecLabel` (vínculo a um caminho-guia por `u64`), `VecBlend`/`VecMorph`/`VecEnvelope` (derivação de fontes), `PaintedDoc`/`BakedForm`/`FlipObjectRef`/`VecPathRef` (as 4 pontes de identidade documento↔entidade), `motion.clone` (instanciamento no DAG).

---

## §4 — A4: Propriedades e Inspector

### §4.1 — Como o Inspector decide o que pintar: **hard-coded, do começo ao fim**

[`crates/ph2d-panel-inspector`](../../crates/ph2d-panel-inspector/), **11.894 LOC**, 24 arquivos + 23 `sections/`.

- **Não há reflexão, não há derive, não há schema.** O host publica snapshots tipados por **thread-local setters, um por assunto**: `set_current_inspector_{sprite, transform, physics, player, joint, wheel, ordering, sampling, visibility, blend, name, visibility_section}` + `set_current_display_unit` ([`lib.rs:97-106`](../../crates/ph2d-panel-inspector/src/lib.rs)).
- **`populate()` é uma lista literal de 16 funções** ([`populate.rs:18-34`](../../crates/ph2d-panel-inspector/src/populate.rs)).
- **`paint()` é uma cascata de 9 `live_section!`** com ordem e separadores escritos à mão ([`paint.rs`](../../crates/ph2d-panel-inspector/src/paint.rs)), mais as seções de física/joint/wheel/player.
- O commit de uma edição volta por `EditorCommand::SetComponent` com o blob postcard montado à mão por tipo ([`inspector_commits.rs`](../../shells/desktop/src/render_loop/inspector_commits.rs)).

### §4.2 — Quanto do registro o Inspector alcança — MEDIDO

**Método (reproduzível):** extrair os 91 nomes canônicos dos quatro `register_*`, e para cada um procurar o identificador do tipo em `crates/ph2d-panel-inspector/src/`, `crates/ph2d-editor-core/src/screens/hero/` e os 6 `shells/desktop/src/render_loop/inspector_*.rs`.

| | # |
|---|---:|
| Tipos registrados | **91** |
| Mencionados em algum ponto do caminho do Inspector (**limite SUPERIOR** — uma menção pode ser só comentário) | **55** |
| **Certamente ausentes** | **36** |

Os 36 ausentes: **31 da família `Vec*`** (`VecShape`, `VecBlend`, `VecMorph`, `VecEnvelope`, `VecOffset`, `VecSymmetry`, `VecCutPath`, `VecBoolGroup`, `VecFrame`, `VecFilter`, `VecContour`, `VecLabel`, `VecTextPath`, `VecPatternPath`, `VecPatternRotation`, `VecStrokeProfile`, `VecConnector`, `VecBindings`, `VecAnchors`, `VecResizeBox`, `VecLayout`, `VecLayoutItem`, `VecLayoutSize`, `VecLayoutAbsolute`, `VecWidget`, `VecWidgetBind`, `VecWidgetValue`, `VecWidgetIcon`, `VecComponentMain`, `VecInstance`, `VecPathRef`) + `PaintedDoc`, `BakedForm`, `FlipObjectRef`, `LuauScript`, `RopeStops`.

⚠️ **A leitura correta disto não é "faltam 36 seções".** As 31 `Vec*` são editadas por **outro painel artesanal** (`ph2d-panel-vector`), e 4 das 5 restantes são pontes de identidade que **não devem** ser editáveis. O fato é outro e é estrutural: **não existe UM inspector; existem N painéis artesanais, um por família**, e cada componente novo escolhe a qual deles pagar uma seção escrita à mão.

### §4.3 — ⭐ E existe um inspector DERIVADO, funcionando, para ~180 tipos

[`crates/ph2d-panel-motion-params`](../../crates/ph2d-panel-motion-params/) pinta *"uma row canônica **label + slider + chip numérico** por param"* a partir de um `ParamsSnapshot`, usando um **pool fixo de widgets posicionais** (`param_slider_id(slot)`), e devolve a edição como `MotionParamIntent::SetParam { node, param, value }`.

O vocabulário do snapshot ([`snapshot.rs`](../../crates/ph2d-panel-motion-params/src/snapshot.rs)) é um **`enum ParamRow` de 12 variantes**: `Scalar` · `Color` · `Toggle` · `Enum` · `Angle` · `Seed` · `Text` · `Curve` · `Palette` · `Gradient` · `Channels` · `Source`. Mais:

- **`sections: Vec<(String, usize)>`** — agrupamento com cabeçalho, derivado por uma ponte que ordena uma vez;
- **`modified: BTreeSet<String>`** — quais params o artista mexeu, e ⚠️ é **presença de chave, nunca `valor != default`**, porque o grafo guarda overrides esparsos (é a mesma lei do `VecInstance`);
- rolagem do corpo, faixa dupla (slider macio × caixa dura), reset por row.

⚠️ A **fonte** desses rows é magra: o manifesto do nó só declara `ParamSpec { name: &'static str, default: f32 }` ([`node.rs:52-55`](../../crates/ph2d-nodegraph/src/node.rs)) — **todo param é `f32`**; a riqueza dos 12 tipos de row é produzida pela **ponte** (`ph2d-eval-motion` + `shaper_dispatch`), com tabelas paralelas de unidade e de teto. Ou seja: o padrão *"UI derivada de um snapshot tipado"* está provado no repo; o que **não** existe é o schema tipado do lado do dado.

### §4.4 — Disclosure progressivo, e o que não existe

- **Existe:** seções com cabeçalho e fold (`SectionFold`, `SectionHeader` com count chip), `TreeView`, `Tabs`, `Combobox` filtrado, rolagem por painel, `ContextMenu`, notas ancoradas por seção (`split_notes`), e o `modified` do Motion.
- **Não existe:** busca dentro do Inspector, favoritos, "advanced section", nem catálogo de tipos.
- **Existe uma busca**, mas de outro assunto: `HIER_SEARCH` na Hierarquia ([`panel-hierarchy/src/search.rs`](../../crates/ph2d-panel-hierarchy/src/search.rs)) e a `command_palette` da shell ([`command_palette_input.rs`](../../shells/desktop/src/command_palette_input.rs)).
- **Script sem UI:** `LuauScript` existe como componente, o host resolve bytecode e `StateTable`, e **não há UI nenhuma** — nem anexar, nem ver campos, nem editar. (Mockup `docs/design/screens/10-script-editor.html` existe.)

---

## §5 — A5: Assets e I/O

### §5.1 — O que é "asset" hoje

[`crates/ph2d-asset`](../../crates/ph2d-asset/) — **2.392 LOC**. `AssetDb` = `RwLock<{ by_id: BTreeMap<AssetId, Arc<Asset>>, by_path: BTreeMap<PathBuf, AssetId> }>`, `AssetId = blake3(bytes)` (HR-6).

```rust
#[non_exhaustive]
pub enum Asset {                       // asset.rs:18-46
    ImageRgba8 { width, height, pixels: Arc<[u8]> },
    Prefab(Arc<PrefabDoc>),
    Scene(Arc<SceneDoc>),
    TextureKtx2 { tier, blob },
}
```

**Quatro variantes; uma exercitada.** `ImageRgba8` é a única usada em produção (import de imagem, atlas, "New Canvas"); `TextureKtx2` só no `ktx2_smoke`; `Prefab`/`Scene` em nenhum lugar (§0.3).

- **Referência:** por `AssetId` (hash), com `by_path` como metadado — *"paths são índice, o hash é identidade"* (HR-6). Renomear não invalida.
- **Cache:** o `by_id` é o cache; `Arc<Asset>` permite swap sem invalidar leitores.
- **Hot reload:** `AssetWatcher` + `ReloadEvent` existem em [`watcher.rs`](../../crates/ph2d-asset/src/watcher.rs) e têm **ZERO consumidores** fora da própria crate.
- **Thumbnail / preview de asset:** ❌ não existe. Existe `render_texture_preview` para a textura **procedural** do pincel (`ph2d-tool-painter`), que prova que a engine sabe gerar preview — mas não há preview de asset.
- **Grafo de dependências entre arquivos:** ❌ não existe. O único mapa que se parece com um é `LogicalTextureMap` (`LogicalTextureId → BTreeMap<TierIndex, AssetId>`), que resolve variantes de tier, não dependências.
- **`ph2d-save`:** stub de **4 linhas**.

### §5.2 — Onde as saídas dos módulos criativos vão parar hoje

| Saída | Onde é salva | Formato | Sobrevive a fechar o app? |
|---|---|---|---|
| Geometria vetorial | `ProjectState.vec` (`VecScene`) | postcard, dentro do `ProjectState` | ✅ |
| Documento do Painter (camadas + pixels + relevo) | `ProjectFile.painted` | postcard, por `PaintedDoc` estável | ✅ |
| Animação quadro-a-quadro | `ProjectState.flip` (`FlipDoc`) | postcard | ✅ |
| Grafo de Motion Nodes | `ProjectFile.motion` | **texto** (diffável por linha) | ✅ |
| Timeline | `ProjectFile.timeline` | postcard, versão própria | ✅ |
| Escultura 3D | `ProjectFile.sculpt` | `Vec<u8>` **opaco**, versão própria dentro | ✅ |
| Canais assados (base/form/rig) | `ProjectFile.baked_forms` | postcard | ✅ |
| Tokens de cor autorados | `ProjectFile.tokens` | esparso, por **chave** (nunca índice) | ✅ |
| Paletas do color picker | **`~/.ph2d/palettes.txt`** | texto, escopo do UTILIZADOR | ✅ (fora do projeto) |
| Preferências (`motion_character`, `reduced_motion`) | **`~/.ph2d/prefs.txt`** | texto, escopo do UTILIZADOR | ✅ (fora do projeto) |
| **Rack de áudio / `EditClip` (42 efeitos, 23 presets)** | ⛔ **em lugar nenhum** | — | ❌ |
| **`BrushSettings` do Painter** | ⛔ **em lugar nenhum** | — | ❌ |
| **Malha do sculpt como asset reusável** | só dentro do blob `sculpt` do projeto | — | ❌ como asset |

⚠️ **Nenhuma dessas saídas é um `Asset`.** Nenhuma tem id de conteúdo, nenhuma é referenciável por outro projeto, nenhuma aparece num browser. O padrão real do repo é: **cada módulo ganha um campo próprio no `ProjectFile`**, com undo próprio e versão própria — e é isso que faz `PROJECT_SCHEMA` estar em 84.

⚠️ **Existe um embrião de "promover a asset"**, e é ad-hoc: o menu de contexto da Hierarquia tem *"Use as brush texture / brush shape / paper / granulation"* (`EditorAction::HierUseAs*`, [`render_loop/mod.rs:3550`](../../shells/desktop/src/render_loop/mod.rs)) — uma sprite da cena vira recurso de pincel por **cópia one-way**, sem identidade e sem volta.

### §5.3 — Escopo, busca, portabilidade

- **Escopos existentes:** documento (`ProjectFile`) e utilizador (`~/.ph2d/*.txt`). ❌ Não há biblioteca de projeto, nem compartilhada, nem instalada. Não há resolução nem sombreamento entre escopos.
- **Busca de asset:** ❌ inexistente.
- **Import:** um botão "Import…" e drag-and-drop de imagem ([`image_import.rs`](../../shells/desktop/src/image_import.rs)); formatos via os 16 crates `ph2d-imageio-*`.
- **Export:** áudio (Ogg/Opus) e imagem; ❌ não há export de asset com dependências.
- **Arrastar asset entre projetos:** ❌ nem existe o conceito.

---

## §6 — A6: Restrições transversais

### §6.1 — ADRs que tocam este território

| ADR | Uma linha | Status |
|---|---|---|
| [0003](../architecture/decisions/0003-ecs-choice.md) | `bevy_ecs 0.18` standalone é o ECS | Accepted |
| [0021](../architecture/decisions/0021-simulation-presentation-boundary.md) | `SimWorld`/`PresentWorld` separados por TIPO, extract one-way | Accepted |
| [0022](../architecture/decisions/0022-no-hashmap-in-simulation.md) | `HashMap` banido em crates de simulação (ordem de iteração) | Accepted |
| **[0025](../architecture/decisions/0025-gameobject-model.md)** | **GameObject = Entity + Components, estilo Unity; hierarquia = `ChildOf`; script é componente; Prefab/Scene são Assets blake3** — e lista 16 pré-requisitos, dos quais os itens **13/14 (wiring do Inspector e da Hierarquia com componentes reais)** são exatamente o assunto deste briefing | Accepted |
| [0025-amendment-1](../architecture/decisions/0025-amendment-1.md) | skew 2D no `Transform` | Accepted |
| [0029](../architecture/decisions/0029-trait-driven-panel-host.md) | painel = crate tipada com `Panel` + `PanelHostInternal` | Accepted |
| [0037](../architecture/decisions/0037-stable-entity-wire-id-scenedoc.md) | id de entidade no save é **índice**, nunca `to_bits`; type key = `blake3(nome)[..8]` | Accepted (nota: já satisfeito) |
| [0069](../architecture/decisions/0069-sprite-inspector-v2.md)/[0070](../architecture/decisions/0070-sprite-schema-v4.md) | Sprite Inspector v2; schema v4 do `Sprite` | Accepted |
| **[0074](../architecture/decisions/0074-sprite-component-boundary.md)** | **regra dos 3 lugares** (campo × componente × derivado); `Sprite` congelado em 20 campos; ≤32 componentes opcionais | Accepted |
| [0075](../architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) | desacoplar por ECS; feature nova = drop-crate; plugin em runtime **rejeitado** | Accepted |
| [0076](../architecture/decisions/0076-vector-as-scene-object.md) / [0110](../architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md) / [0111](../architecture/decisions/0111-vector-shapes-have-transforms-and-use-the-sprite-gizmo.md) | todo path vetorial é entidade ECS numa hierarquia só, com `Transform` e o gizmo da sprite | Accepted |
| [0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) | componentes de física são **config**, nunca estado vivo do solver — *"o undo ordena por bytes"* | Accepted |
| [0132](../architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) | LPE do vetor é pilha por-path, **não** grafo de nós | Accepted |
| [0148](../architecture/decisions/0148-vector-live-width-profile-is-an-ecs-component-and-one-baker-serves-preview-and-apply.md) | perfil de largura vivo é componente ECS, um baker serve preview e apply | Accepted |
| [0153](../architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md) | auto layout publica **onde as coisas ficam**; nunca escreve `Transform` (senão cada resize vira passo de undo) | Accepted |

### §6.2 — Undo: como uma mudança ESTRUTURAL se encaixa

[`shells/desktop/src/undo.rs`](../../shells/desktop/src/undo.rs). É **snapshot-based por DIFF num ponto só**, não command-based:

1. `post_frame_undo()` roda **1×/frame**, depois do render;
2. se houve input e nenhum gesto está em curso, `capture_project()` compara o estado atual com o baseline;
3. qualquer diferença de bytes vira um passo; `UNDO_CAP = 256`;
4. `restore()` **despawna toda entidade com `Transform`** e re-spawna do snapshot, depois reconstrói as pontes por varredura.

**Consequência para "Add Component":** adicionar ou remover um componente registrado **já é desfazível, sem uma linha de código** — ele muda os bytes do snapshot e o diff o pega. Isto é uma propriedade rara e vale nomear: no PH2D a mudança estrutural é **mais** barata de desfazer que num sistema por comandos.

**Três invariantes que qualquer desenho novo tem de honrar:**

- ⚠️ **`canonicalize()` ordena as linhas por CONTEÚDO** (bytes serializados), nunca por `to_bits()` ([`undo.rs:152-180`](../../shells/desktop/src/undo.rs)) — foi isso que fazia todo frame virar um passo espúrio. Logo: **qualquer lista dentro de um componente tem de ter ordem canônica** (é a razão de `VecInstance::set` usar `binary_search`).
- ⚠️ **Bits de entidade dentro dos bytes de um componente envenenam o próprio undo** — dois estados logicamente iguais comparam diferente. É por isso que a referência durável é o nome.
- ⚠️ **Serialização não-determinística ⇒ um passo de undo por frame.**

### §6.3 — Threading: o que pode ser mutado de onde

- Threads canônicas (SKILL §12.2): game (main) · render · áudio (`cpal`) · IO (`rayon`). **`tokio` proibido no core.**
- `SimWorld` é mutado só pela game thread; `PresentWorld` recebe por `extract!` one-way. `Sim`/`PresentComponent` fazem a fronteira valer **em tempo de compilação**.
- O `EditorCommandQueue` é `Arc<Mutex<Vec<EditorCommand>>>` com `cap = 4096` e backpressure explícita — o padrão pelo qual **qualquer** mutação do editor entra no `SimWorld`.
- ⛔ HR-5: GPU compute é proibido em qualquer cálculo cujo output entre no `SimWorld`.

---

## §7 — As medições

**Spike descartável**, escrito em `crates/ph2d-ecs/tests/`, rodado em `--release`, e **apagado depois** (a árvore só tem `docs/Components/` não-rastreado). Máquina: workstation Linux, 32 cores, 123 GiB, `load` baixo.

⚠️ **Leitura de relógio desta workstation não vale acima de `load ~5`** (lei do `CLAUDE.md` §5.0). Os números abaixo foram colhidos com a máquina calma e reproduziram entre duas corridas.

### §7.1 — Tamanhos

| Tipo | `size_of` |
|---|---:|
| `ph2d_render::Sprite` | **200 B** |
| `SpriteSource` | 36 B |
| `Transform` | 28 B |
| `Name` | 24 B |

### §7.2 — Custo de adicionar/remover componente (archetype move)

| n de entidades | `insert::<Sprite>` | `remove::<Sprite>` |
|---:|---:|---:|
| 1.000 | 91,4 ns/op | 31,8 ns/op |
| 10.000 | 71,4 ns/op | 33,4 ns/op |

**Conclusão:** o custo de composição em runtime **não é um limite de nada** nesta ordem de grandeza. 10.000 inserções custam ~0,7 ms.

### §7.3 — ⚠️ O custo que MANDA: a captura do undo, por frame

`world_to_snapshot` + `canonicalize` (a réplica verbatim da função privada do shell), sobre entidades com `Transform + Name + Sprite`:

| n | `world_to_snapshot` | `canonicalize` | **total da captura** | snapshot postcard |
|---:|---:|---:|---:|---|
| 100 | 0,052 ms | 0,020 ms | **0,072 ms** | 21,9 KB (218 B/ent.) |
| 1.000 | 0,483 ms | 0,208 ms | **0,691 ms** | 219,9 KB (219 B/ent.) |
| 10.000 | 4,808 ms | 2,083 ms | **6,892 ms** | 2,21 MB (220 B/ent.) |

> ⚠️ **CORREÇÃO (2026-08-21, noite — [medição de 2ª rodada](pesquisa/instancias_2026-08-21/medicao_captura_incremental.md)):
> a tabela acima mediu o PISO, não o teto.** O `canonicalize` constrói a chave (`Vec<u8>` de ~230 B)
> **dentro do comparador** do `sort_by`, e o sort do Rust é adaptativo: sobre entrada **já ordenada**
> (o mundo logo após um restore) faz ~n comparações — é o regime que o spike desta seção apanhou; sobre
> entrada em **ordem de criação** (qualquer cena construída ou reordenada nesta sessão) faz ~n·log n ⇒
> ~266 k alocações a 10 k. Os dois regimes, a 10.000 entidades: **6,27 ms** (pós-restore) e
> **23,8 ms** (edição real) — `canonicalize` sozinho = 1,18 vs **18,7 ms**. O `PartialEq` do baseline
> custa 0,076 ms (desprezível). Leia o parágrafo seguinte com **38 % a 143 %** no lugar de 41 %.

**Isto roda uma vez por frame em que houve input**, e depois ainda há o `clone()` de `VecScene`, `FlipDoc`, `GuideSet` e `StateSets`.

**A 10.000 entidades a captura sozinha consome de 38 % a 143 % de um frame de 16,6 ms** — antes dos clones. Qualquer desenho de prefab/instanciamento que **multiplique a contagem de entidades** paga aqui primeiro, e paga por frame. Isto é um fato de escala, não uma opinião de arquitetura, e o `UNDO_CAP = 256` significa que a pilha pode chegar a ~565 MB de snapshots a 10 k entidades. ⭐ **E está resolvido por medição no doc 04 v2 §1.1**: captura incremental por change ticks = **0,27 ms** no mesmo mundo.

⚠️ Nota metodológica: o crescimento da tabela acima é ~**linear** porque a entrada era canônica; no regime de edição o `sort` domina (n·log n com alocação por comparação). Não determinado qual ordem de spawn este spike usou.

---

## §8 — Ambiguidades que o código NÃO resolve

Escritas como ambiguidades de propósito. Nenhuma é resolvida aqui.

1. **`register_script_components` não é chamado no boot** (§1.2). Não determinado se é decisão (o script não tem UI) ou omissão. Consequência hoje: `LuauScript` não é salvo nem desfeito. **Confira antes de construir sobre isso.**
2. **`HIERARCHY_ADD` sem handler** (§0.3). Não determinado se o gesto foi deliberadamente movido para o menu de contexto ou se o handler se perdeu numa migração de painel (o comentário em `screens/hero/tests.rs:185` fala da mudança de dono na ADR-0029 Fase C.2).
3. **`PrefabDoc`/`SceneDoc`/`spawn_prefab` sem consumidor**: não determinado se o `ProjectFile` os torna redundantes ou se são o caminho pretendido que nunca foi ligado. Os dois formatos **respondem à mesma pergunta de formas incompatíveis** (§3.2), e nada no repo declara qual vence.
4. **`extract_component_snapshot` sem consumidor**: o substrato do inspector genérico existe e nunca foi usado. Não determinado se por decisão (o Inspector artesanal chegou primeiro) ou por esquecimento.
5. **Ausência de `default_bytes` no registro** (§1.2): não determinado se foi decisão (nem todo componente tem default sensato) ou lacuna. `Transform`, `Name` e a maioria dos `Vec*` derivam `Default`; `Sprite` **não** (precisa de uma `source`).
6. **`OverrideSlot` com 2 variantes e o `PropPath` genérico não-construído** (§3.4): o comentário diz que a W4b *"estende ESTA lista ou a absorve"*. Não determinado quando, nem se o modelo de override do `VecInstance` deve ser o do sistema inteiro ou só do vetor.
7. **Migração de `PROJECT_SCHEMA`**: HR-14 exige `migrate_vN_to_vN+1`; o repo tem **zero** e recusa arquivos antigos. Não determinado se existe alguma promessa de compatibilidade (não há projeto publicado), ou se a recusa é a política declarada.
8. **`Tag(Smol<32>)` está no vocabulário canônico da ADR-0025 §"Vocabulário"** e **não existe no código**. Não determinado se foi diferido ou abandonado; o [levantamento](00_levantamento_componentes.md) §5 o lista como **P0** sob outro nome (`Tags` hierárquicas).
9. **Fração de sprites que usa cada grupo de campos** (§2.1): estimada, não medida. Não há corpus de projetos salvos para contar.
10. **Escopo do `VecInstance` fora do vetor**: nada no código diz se `VecComponentMain`/`VecInstance` foram desenhados para generalizar a entidades ou se o `VecPathId` é intrínseco ao desenho.
11. **(acrescentado 2026-08-21, noite)** **O PH2D nunca avança o change tick do `bevy_ecs`** — zero chamadas a `clear_trackers`/`increment_change_tick`/`check_change_ticks` e zero `Changed<>`/`Added<>` em `crates/` + `shells/`; não há `Schedule`. Todo componente tem `added == changed == Tick(1)`. A change detection existe na memória sem custo e sem informação. Não determinado se foi decisão.
12. **(acrescentado)** **A ordem de irmãos não é dado persistido** — nenhum componente a guarda (só `RootOrder` para raízes); `world_to_snapshot` lê a ordem de `Children`, `canonicalize` reordena por conteúdo, `snapshot_to_world` reinsere `ChildOf` na ordem das linhas ⇒ reordenar filhos **não é desfazível** e **não sobrevive a um restore**. Pré-existente (classe BUGS #15). ⚠️ A nota de `children_order.rs:5-8` ("o bevy não tem *põe no índice k*") envelheceu: o 0.18.1 tem `insert_related::<R>(index, …)`.
13. **(acrescentado)** **A instância vetorial não renderiza aninhamento** — `cook_one` lê `src.cooked()`, nunca `self.live` ([`instance_live.rs:149-152`](../../shells/desktop/src/instance_live.rs)): uma instância dentro do mestre aparece como o retângulo-suporte na cópia. Só `main == at` é recusado; profundidade capada em 64 sem erro.

---

## Apêndice — Comandos de verificação (para quem quiser refazer a conta)

```bash
cd /home/enio/Documentos/Projetos/PH2D

# Os 91 tipos registrados
grep -ho 'reg.register::<[^>]*>("\([^"]*\)")' \
  crates/ph2d-ecs/src/scene/registry.rs crates/ph2d-render/src/registry.rs \
  crates/ph2d-script/src/registry.rs crates/ph2d-physics-ecs/src/lib.rs | wc -l

# O DAG não conhece o ECS
grep -rn "bevy_ecs\|Entity" --include="*.rs" crates/ph2d-nodegraph/src/ | wc -l   # → 0

# Asset browser
grep -rni "asset_browser\|AssetBrowser\|asset_library" --include="*.rs" crates/ shells/ | wc -l  # → 0

# Prefab em produção
grep -rn "spawn_prefab\|spawn_scene" --include="*.rs" crates/ shells/ | grep -v tests | grep -v "scene/spawn.rs"
```
