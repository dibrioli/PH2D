# Plano vivo — objetos composáveis, instâncias vivas, undo incremental, assets (fases F0–F8)

> **Executor:** a linha `line/components` (Modo L, worktree própria). **Governança:**
> [ADR-0164](../architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md)
> (F0–F5) e [ADR-0165](../architecture/decisions/0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md)
> (F6–F7). **A forma longa de cada decisão** é o [doc 04 v2](04_decisao_arquitetura.md); **a
> evidência** (medições, refutações, `file:line`) é
> [`pesquisa/instancias_2026-08-21/`](pesquisa/instancias_2026-08-21/). Este plano não re-explica o
> porquê — ele diz **o quê, onde, e como saber que ficou pronto**.
>
> **É um PLANO VIVO:** ao fechar uma fase, a linha marca ✅ na tabela do §0 com o link do handoff, e
> corrige aqui o que a implementação provou errado (com a razão — nunca apague a linha original,
> risque). ⚠️ Regra do §5 do CLAUDE.md: narrativa vai para handoff/arquivo, não para cá.

---

## §0 — Placar (a linha atualiza; UMA linha por fase)

| Fase | O quê (uma frase) | Estado |
|---|---|---|
| F0 | O descritor de componente + `insert_default` — o inspector aprende a derivar | ⬜ |
| F1 | `StableId` + `SiblingOrder` + snapshot v2 + **a 1ª migração** + corte da Sprite | ⬜ |
| F2 | O undo vira incremental (protocolo das 6 condições) | ⬜ |
| F3 | Add Component com cascata visível + objeto vazio na raiz — **walking skeleton** | ⬜ |
| F4 | Núcleo de instância: Duplicar/Criar componente/Instanciar/sync/Destacar + física | ⬜ |
| F5 | Aninhamento + variantes + Overrides sem alvo | ⬜ |
| F6 | O índice de assets (`ph2d-asset-index`) — sem UI | ⬜ |
| F7 | O painel Asset Browser + o arrasto único | ⬜ |
| F8 | Restore incremental + `VecScene`/`FlipDoc` versionados | ⬜ |

**Ordem é lei:** F1 antes de F2 (a chave do cache é o `StableId`); **F2 antes de F4** (materializar
sem undo incremental multiplica um custo que já estoura o quadro); F4 antes de F6 (mestres são o
conteúdo do browser). Cada fase é **mergeável isoladamente** — a engine nunca fica quebrada entre
fases.

## §0.1 — Leis herdadas que TODA fase honra (e onde está cada uma)

1. **Inner loop = `bash scripts/cargo-check-narrow.sh <crate>`**; teste/clippy/auditoria 1× no
   fechamento (CLAUDE.md §2). Edições pela ferramenta `Edit`, nunca `python3`/`sed`.
2. **Componente novo = registrar nos contadores** — os asserts de contagem do registro **somam**
   entre linhas, nunca se escolhe um lado (registry 57 · render/script 58 · física 32; CLAUDE.md §5.0).
3. **Zero bits de `Entity` dentro de bytes de componente**; toda lista interna em **ordem canônica**;
   serialização determinística (BTreeMap, HR-5, ADR-0022).
4. **UI canônica**: zero hex, zero `f32` literal, tudo tokens/i18n (HR-15); painel novo COPIA a
   Widget Gallery (SKILL §14); slider+chip → `link_slider_number`.
5. **Gates de relógio nunca no gate batched** — a família de flakes sob fan-out (CLAUDE.md §5.0):
   benches ficam `#[ignore]`, rodados com a máquina calma (`load ≤ 5`), com a barra escrita.
6. **Cena de smoke se CONTA lendo o roteador**, e o comando ao Enio vai inteiro, com `cd` da
   worktree (CLAUDE.md §0.8/§5.0).
7. **Mudou forma de dado persistido ⇒ degrau no `PROJECT_SCHEMA`** com a escada + a tripla em
   `project_schema_tests.rs` — três sítios, nunca um.
8. **Ids/consts novos vão ANOTADOS no handoff** (colisão entre linhas passa muda — CLAUDE.md §5.0).

## §0.2 — Números e nomes NOVOS desta linha (anote aqui + no handoff; colisão!)

| O quê | Valor | Quem soma |
|---|---|---|
| Crates novas | `ph2d-component-desc` (F0) · `ph2d-asset-index` (F6) | workspace glob |
| Componentes novos no registro | `StableId` · `SiblingOrder` (F1) · `MasterRoot` · `MasterPiece` · `InstanceOf` · `ObjectInstance` (F4) | contador do `ph2d-ecs` (57 → 59 → 63) **+ os espelhos 58 das suítes render/script** |
| `WorldSnapshot::VERSION` | 1 → **2** (F1) | `save.rs` |
| `PROJECT_SCHEMA` | 84 → **85** (F1; conte contra o main do dia — pode ter subido) | escada + tripla |
| Envs de smoke | `PH2D_INSTANCE_SMOKE=<n>` (F4+) · `PH2D_ASSET_BROWSER_SMOKE` (F7) | roteador de cenas próprio |
| Campo novo no `ProjectFile` | `stable_id_counter` (F1 — FORA do `ProjectState`, undo não rebobina) | conta no degrau 85 |

---

## §F0 — O descritor de componente

**Objetivo:** um componente descreve os próprios campos UMA vez, e disso derivam inspector,
override, remap de referências e política de propagação.

**Pronto quando (observável):** a seção *Ordering* do Inspector é **pintada pelo descritor** (não
mais artesanal) e fica **visualmente idêntica** à atual; `insert_default` existe na vtable e um
teste insere `SortingLayer` por `type_id` sem conhecer o tipo.

**Toca:**
- **Nova** `crates/ph2d-component-desc/` — crate-FOLHA (⛔ sem `bevy_ecs`, sem `ph2d-nodegraph`;
  o precedente é `ph2d-warp-style`): `ComponentDesc { fields: &[FieldDesc] }`,
  `FieldDesc { field_id: u16 /*append-only, estilo tag protobuf*/, name, kind: FieldKind,
  policy: Propagation, is_ref: Option<RefKind> }` + `FieldKind` espelhando o vocabulário do
  `ParamRow` (Scalar·Color·Toggle·Enum·Angle·Seed·Text·…).
- `ph2d-ecs/src/scene/registry.rs` — `ComponentTypeEntry` ganha `insert_default: Option<fn>`,
  `desc: Option<&'static ComponentDesc>`, `component_id: ComponentId` (preenchido no boot;
  pré-requisito do scan da F2) e `patch_field`/`read_field` por `field_id`.
- `ph2d-panel-inspector` — UMA seção piloto derivada (Ordering), atrás do mesmo paint.

**Testes:** gate de **snapshot da tabela de `field_id`** por tipo descrito (mudar/reordenar id =
vermelho; apender = ok) — é o `FormerlySerializedAs` de graça; prova de mutação: trocar dois
`field_id` ⇒ o gate mata. Round-trip `patch_field`→`read_field` para cada `FieldKind`.

**Não fazer:** descrever os 91 tipos já — F0 descreve `Transform`, `Name`, `Sprite` + os de
ordering (o resto entra por demanda nas fases seguintes; a tabela cresce append-only).

---

## §F1 — `StableId` + `SiblingOrder` + snapshot v2 + a PRIMEIRA migração + corte da Sprite

**Objetivo:** toda entidade editável ganha identidade estável; a ordem de irmãos vira dado; o
snapshot passa a ordenar por id (o `canonicalize` morre); e a Sprite perde 3 grupos para
componentes opcionais.

**Pronto quando:** (1) round-trip de um corpus de projetos **v84 → v85 → save** é byte-equivalente
no re-save (gate com fixtures geradas ANTES de mexer — o precedente é
`generate_transform_v1_fixtures.rs`); (2) **reordenar irmãos na Hierarquia é desfazível** e
sobrevive a Ctrl+Z de outra ação (o Enio vê); (3) renomear um objeto **não** desliga binding de
timeline nem joint (gate novo; hoje desliga); (4) gate `no_two_rows_share_a_stable_id` verde com
prova de mutação (duplicar blob ⇒ vermelho).

**A ordem interna desta fase (não isolável — checkpoints obrigatórios):**
1. `StableId(u64)` componente + hook `on_add` de `Transform` + contador `stable_id_counter` no
   `ProjectFile` (⚠️ **fora** do `ProjectState`; ids nunca reusados, `0` reservado).
2. `SiblingOrder(u32)` (gêmeo do `RootOrder`; a nota envelhecida de `children_order.rs:5-8` cai —
   o 0.18.1 tem `insert_related::<R>(index, …)`).
3. `WorldSnapshot` **v2**: linhas ordenadas por `StableId`, `parent: Option<StableId>`; `restore()`
   despawna `With<StableId>`; `canonicalize` **removido** (mover o resíduo útil para `ph2d-ecs` —
   o bench da F2 precisa dele lá).
4. Migração `v84→v85` em `project_load.rs` (a primeira do repo): atribui `StableId`/`SiblingOrder`
   na ordem canônica antiga (determinística), semeia o contador.
5. **As duas famílias de `stable_name_id`, na MESMA wave** (o `name.rs:73-79` proíbe metade):
   timeline (`WireId` = `StableId`; `timeline_persist.rs:38`, `frame_solve.rs:139/218/251`,
   `persist.rs:28/54/88`) e física (`PhysicsJoint.body_a/b`, `PulleyWheel.rope/.body`,
   `bridge/joints.rs:152/315`, `bridge/rope.rs:130`, `joint_group.rs:139` + 12 sítios de inspector
   + fixtures do `physics_ecs_c9`). `stable_name_id` fica **deprecado com nota**, não removido.
6. Corte da Sprite (doc 04 C4): `per_corner_tint` → `SpriteCornerTint` · folha inline →
   `SpriteSheet` · região → `SpriteRegion` — **ausência = default benigno**, criar sprite continua
   1 gesto. ⚠️ Os três somam nos contadores (regra §0.1.2) e o cap do ADR-0074 (≤32 opcionais)
   recebe +3.

**Testes:** determinismo — `state_hash` idêntico em duas capturas do mesmo estado e através de
restore; mutação: remover o remap de `StableId` na cópia de blobs ⇒ gate de unicidade mata.
⚠️ O `deterministic_hash` do c9 muda de valor com o snapshot v2 — re-capturar o golden é parte da
fase, com a matriz 3-OS verde no fechamento.

---

## §F2 — O undo incremental

**Objetivo:** a captura custa o tamanho da edição, não do mundo; a pilha guarda deltas.

**Pronto quando:** bench `#[ignore]` em `crates/ph2d-ecs` (máquina calma) imprime e cumpre:
**≤ 0,3 ms** @10 k parado · **≤ 1,0 ms** @10 % sujo · baseline de referência na tabela do bench
(medido 2026-08-21: 0,269 / 0,953 ms; hoje: 23,8 ms). E os três gates de correção passam com prova
de mutação.

**O protocolo é o do doc 04 §2.7 — as SEIS condições são lei** (cada uma nasceu de uma refutação;
[refutacao_2](pesquisa/instancias_2026-08-21/refutacao_2_captura_incremental.md)):
cache `BTreeMap<StableId, Arc<Row>>` com `ArchetypeId` por linha · sujo = tick de registrado **ou
`ChildOf`** > tick da última captura **ou** archetype ≠ cacheado · tick é pré-filtro, **bytes são a
verdade** · spawn por ausência, despawn por carimbo · **`clear_trackers()` 1× por CAPTURA, nunca em
quadro de retorno precoce** · restore reinicializa o cache e dá o próprio clear.

**Toca:** `shells/desktop/src/undo.rs` (+ irmãos novos pelo teto de LOC) · `ph2d-ecs` (o scan por
archetype: interseção `registro ∩ archetype.components()` — o scan ingênuo 91×entidade custa 23×
mais, medido) · `SimWorld` expõe `clear_trackers`/ticks.

**Testes (cada um com a mutação que o mata):**
- **Ponto fixo**: `capture → frame → capture` sem input ⇒ zero passo (mata: `clear_trackers` no
  lugar errado).
- **Remoção**: `remove::<VecFilter>` (o Detach do FX) ⇒ **1 passo registrado** (mata: esquecer
  archetype/`removed_with_id` — hoje o tick não vê remoção, medido C″).
- **Gesto segurado**: arrasto de n quadros ⇒ 1 passo com TODAS as mutações (mata: clear por quadro).
- **Equivalência**: captura incremental ≡ rebuild completo v2, byte a byte, sob spawn+despawn+
  reparent+remove no mesmo quadro.
- Escrita não-rastreada: lint estrutural proibindo `bypass_change_detection`/`get_mut_by_id`/
  `as_unsafe_world_cell*` nas crates de sim (hoje zero usos — que continue).

**Instrumentação:** `PH2D_UNDO_LOG=1` passa a imprimir `linhas sujas / re-serializadas / delta B`
por captura.

**Fora desta fase (F8):** restore incremental; `VecScene`/`FlipDoc` versionados — o clone deles
continua O(doc) e o log deve dizê-lo.

---

## §F3 — Add Component + objeto vazio — ⭐ walking skeleton

**Objetivo:** o artista cria um objeto vazio, adiciona componentes por um diálogo com busca, e vê
ANTES de aplicar o que vem junto.

**Pronto quando (o smoke que o Enio roda):** criar objeto vazio na raiz (o botão **Add** da
Hierarquia — hoje morto, `§0.3 do doc 01`) → *Add Component* → buscar "Rigid" → o diálogo mostra
**"RigidBody — traz junto: Collider"** → aplicar → salvar → reabrir → Ctrl+Z remove o componente.
Tudo num fluxo, sem tocar código.

**Toca:** handler do `HIERARCHY_ADD` (spawn `Transform+Name+StableId+RootOrder`) · diálogo Add
Component (busca fuzzy sobre o registro via descritor; ⚠️ **a cascata é MOSTRADA antes de aplicar**
— é a correção da crítica medida ao Bevy, doc 02 §1.4) · `#[require]`-equivalente por descritor
(`requires: &[type_id]`) · `EditorCommand::Spawn`/`SetComponent` já existem — é wiring.

**Testes:** seam completo (pintado/populado/clicado/**sequência** — as 4 perguntas); censo "todo
tipo com descritor aparece na busca"; a11y (HR-12).

---

## §F4 — O núcleo de instância

**Objetivo:** Duplicar (independente) · Criar componente (mestre na biblioteca + instância no
lugar) · Instanciar (peças reais ligadas) · override por campo capturado por diff · sync vivo ·
Destacar/Redefinir/Aplicar ao mestre — **com física de verdade**.

**Pronto quando (os três smokes-gate):**
1. **Ragdoll**: mestre com 2 corpos + junta → *Instanciar* 3× → play → **as três caem e cada junta
   prende os corpos DELA** (gate `the_instance_joint_binds_the_instances_own_bodies`; hoje
   prenderia os do mestre).
2. **Propagação viva**: editar a cor/forma de uma peça do mestre em *Editar mestre* → as 3
   instâncias mudam **no mesmo quadro**, exceto o campo overridado numa delas.
3. **Ponto fixo sob física**: Ctrl+Z sobre instância com peça dinâmica **mid-play** = um passo, sem
   passo espúrio no quadro seguinte (o cenário do refutador 1).

**As condições da ponte são LEI** ([refutacao_1](pesquisa/instancias_2026-08-21/refutacao_1_sync_determinismo.md)):
- `Without<MasterPiece>` nas **5** `QueryState`s (`bridge.rs:84-127`) e na resolução de nomes da
  timeline; **lane nova no `physics_ecs_c9`** com mestre+instância (hoje o hash 3-OS é verde por
  vacuidade).
- A ponte **exporta** `pose_owner(entity)`; sync e captura de override consultam — campo de peça
  `Solver|Player` é *Do runtime* (nem sincroniza nem vira override).
- Config de física propagada **aplica no Reset** (`bodies.rs:169`) — v1 declara; levantar depois.
- Sync escreve com `set_if_neq`, ordem topológica, **recusa de ciclo no gesto** (mensagem, não cap).
- Refs (`is_ref` do descritor) **remapeadas em toda propagação**: `body_a/b`, `.rope/.body`,
  `VecTextPath/VecPatternPath/VecEnvelope.path`, `VecLabel.host`.

**Componentes novos:** `MasterRoot` · `MasterPiece` (mantido pelo sync) · `InstanceOf` ·
`ObjectInstance { overrides: BTreeMap<OverrideKey, Bytes> }` — chave `(path[], peça, type_id,
field_id)`, ids no **escopo do mestre** (a lei do `VecInstance.sub`).

**O `VecInstance` é subsumido AQUI** (doc 04 §2.9): peças viram entidades; `InstanceLive` morre;
verbos/UI ficam; documentos antigos materializam no load (degrau — conte o `PROJECT_SCHEMA` do dia).
⚠️ Re-medir a lei do `vec_instance_follow.rs` sob o sync antes de a portar.

**Duplicar profundo:** nasce sobre `extract_component_snapshot` + `insert_from_bytes` (zero
consumidores hoje) com remap de `StableId` e refs — substitui a cópia rasa de
`render_loop/hierarchy.rs:171-238`.

**Smoke:** `PH2D_INSTANCE_SMOKE=<n>` — roteador próprio; cena 1 = ragdoll auto-play
([feedback: exemplo pronto pra smoke](../../project-memory/feedback_ready_to_smoke_example.md)).

---

## §F5 — Aninhamento + variantes + Overrides sem alvo

**Objetivo:** instância dentro de instância a qualquer profundidade; variantes irmã e derivada;
o que quebra no mestre vira lista visível, nunca perda muda.

**Pronto quando:** (1) instância³ (três níveis) propaga uma edição do mestre mais fundo até a cena
num quadro, na ordem topológica; (2) trocar variant↔base **preserva** os overrides (re-key
determinístico — a operação que nenhuma referência tem, doc 04 §2.6); (3) apagar uma peça no mestre
põe o override na seção **"Overrides sem alvo"** do Inspector — e o **undo da peça no mestre faz o
override voltar a pegar** (a propriedade que o id compra); (4) *Aplicar ao mestre interno* apaga o
override do mesmo campo nos níveis intermediários (senão é no-op visível — regra Unity).

**Trocar para mestre NÃO aparentado:** só por gesto, com os 3 modos (`Nenhum` default · `Por nome`
só com nomes únicos · `Por hierarquia`) + relatório. ⛔ Nunca automático (HR-5).

**Testes:** a tabela de operações do doc 04 §2.6 vira uma suíte — uma linha, um gate; ciclo
indireto (B contém instância de V que é-a B) é recusado com mensagem; prova de mutação no re-key.

---

## §F6 — O índice de assets (`ph2d-asset-index`) — ADR-0165

**Objetivo:** responder *"que assets existem, de que tipo, em que catálogo, quem usa quem, com que
preview"* **sem carregar nenhum**.

**Pronto quando:** teste headless popula 10 k assets e `query`/`deps`/`preview` respondem; bench
`#[ignore]` com a barra escrita ao lado da medição; mestres da F4 aparecem no índice ao *Criar
componente*; catálogo renomeado não desliga nenhum asset (UUID).

**Toca:** nova `crates/ph2d-asset-index/` (folha; `LogicalId` generalizando o `LogicalTextureId`;
catálogos `{uuid, caminho, nome}` em texto; deps; preview assíncrono com cache limitado, gerado
pelo `GameRt` **fora do frame**) · `ph2d-asset` (metadado de cabeçalho). ⚠️ O índice lê cabeçalho,
nunca corpo; atualiza pelos change ticks — nunca por varredura disparada por movimentação (a
patologia medida no Godot).

**Fora (ADR-0165):** similaridade por ML · coleta automática de órfãos (relatório, não colheita) ·
escopos partilhados. Cor dominante ENTRA (histograma OKLab, uma passagem).

---

## §F7 — O painel Asset Browser + o arrasto único

**Objetivo:** o painel que mostra o índice; um payload de arrasto serve canvas, timeline, grafo e
inspector.

**Pronto quando (smoke):** `PH2D_ASSET_BROWSER_SMOKE` — abrir o painel (chip **Assets** da topbar,
hoje só imprime no stdout) → buscar por nome/tag/catálogo → **arrastar um mestre para o canvas ⇒
Instanciar** → arrastar uma textura para um campo `AssetRef` do inspector ⇒ preenche. Preview
aparece sem travar a UI (assíncrono).

**Toca:** nova `ph2d-panel-asset-browser` (COPIA a Widget Gallery; registro de painel = os 5 sítios
da [memória](../../project-memory/reference_topic_panel_registration.md)) · `DragAsset(AssetRef)` no
action bus · `AssetRef<Kind>` como variante do `ParamRow` (F0 preparou o vocabulário).
Referência visual: `docs/design/screens/05-asset-browser.html` (o mockup de 2026-05 — **é
inspiração, não spec**; a spec é o índice da F6).

---

## §F8 — Restore incremental + docs fora do ECS versionados

**Objetivo:** o Ctrl+Z aplica um diff, não reconstrói o mundo; `VecScene`/`FlipDoc` param de ser
clonados/comparados inteiros por captura.

**Pronto quando:** bench: Ctrl+Z **≤ 1 ms** @10 k (diff por `StableId`, aplicar k linhas — o passo
1 do Blender); os 11 `forget()` de memos em `apply_project` viram invalidação dirigida; captura de
`VecScene` O(paths mudados) por contador de versão ou `Arc` por path.

---

## §9 — Decisões de produto embutidas (aprovadas com a v2; o smoke é onde o Enio as sente)

1. **O mestre vive na biblioteca; o canvas mostra sempre instâncias.** *Criar componente* deixa uma
   instância no lugar. Alternativa nomeada se o smoke recusar: modo "mestre fixado no canvas" =
   *Editar mestre* permanente. (Muda a UX atual do vetor.)
2. **Ordem de irmãos é dado overridável por instância** (Unity sim, Figma não — escolhemos sim).
3. **Nome da peça é campo overridável** como outro qualquer; o da RAIZ da instância é *Local*.
4. **Duplicar = independente; Instanciar = vinculado** — vocabulário do Enio, fixado.

## §10 — O que este plano NÃO faz (com o degrau nomeado)

| Fica para depois | Degrau |
|---|---|
| Clipe de timeline do mestre propagar às instâncias | `SequencePlayer` (TOP-20 #19 do [doc 00](00_levantamento_componentes.md)) |
| Config de física do mestre pegar mid-play | re-descrever corpo preservando velocidade (item na ponte) |
| Um ficheiro por asset (Git) | wave própria + ADR próprio; o índice da F6 nasce compatível |
| Thumbnails animados | depois dos estáticos; só pasta visível |
| Bibliotecas partilhadas/instaladas | lado a lado, nunca fundidas (ADR-0165 §5) |
