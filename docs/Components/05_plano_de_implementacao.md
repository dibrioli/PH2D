# Plano vivo — objetos composáveis, instâncias vivas, undo incremental, assets (fases F0–F8)

> **Executor:** a linha `line/components` (Modo L, worktree própria). **Governança:**
> [ADR-0164](../architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md)
> (F0–F5), [ADR-0165](../architecture/decisions/0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md)
> (F6–F7) e [ADR-0166](../architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)
> (a composição do Inspector — afina **F0** e **F3**). **A forma longa de cada decisão** é o
> [doc 04 v2](04_decisao_arquitetura.md); **a evidência** (medições, refutações, `file:line`) é
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
| F0 | O descritor de componente (+ `category`/`applies_to`) + `insert_default` — o inspector aprende a derivar | ✅ 2026-08-24 |
| F1 | `StableId` + `SiblingOrder` + snapshot v2 + **a 1ª migração** + corte da Sprite | ✅ 2026-08-25 |
| F2 | O undo vira incremental (protocolo das 6 condições) | ✅ 2026-08-25 |
| F3 | O Inspector passa a mostrar o que o objeto TEM · o `+` e a paleta · objeto vazio na raiz — **walking skeleton** | ✅ 2026-08-25 |
| F4 | Núcleo de instância: Duplicar/Criar componente/Instanciar/sync/Destacar + física | 🟨 F4.1–F4.5 ✅ · F4.6a/b ✅ · **F4.7 ✅ (os 3 smoke-gates)** · **F4.6c bloqueada na F5** (ela apagaria os variants — ver §F4) · + a auditoria de 27/08 e o modo LIGADO |
| F5 | Aninhamento + variantes + Overrides sem alvo | 🟨 **F5.1 ✅** · **F5.3 ✅** (modelo; a secção mostra a CONTAGEM, não quais) · **variantes ✅ 2026-08-27** · **EIXOS de propriedade ✅ 2026-08-30** (a fatia que a F4.6c pedia) · critério 4 (*Apply to inner master*) ⬜ · troca p/ mestre não aparentado ⬜ |
| F6 | O índice de assets (`ph2d-asset-index`) — sem UI | ✅ 2026-08-30 (996 LOC + a taxonomia) |
| F7 | O painel Asset Browser + o arrasto único | ✅ 2026-08-30 — etapas **A–D** do [plano 07](07_plano_do_navegador_de_assets.md); `DragPayload` com as duas famílias |
| F8 | Restore incremental + `VecScene`/`FlipDoc` versionados | ⬜ |

> ⚠️ **Este placar esteve DESACTUALIZADO** (conferido contra o código em 2026-08-30): a F6 e a F7
> diziam ⬜ com a crate e o painel construídos e smokados. *O §5.0 manda auditar a lista antes de
> pegar um item dela — uma linha ⬜ sobre trabalho já pago manda alguém reconstruí-lo.*

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

> ⚠️ **BASE RE-MEDIDA em 2026-08-24, na abertura da linha** — três números que este plano trazia
> tinham envelhecido entre a redação (21/08) e a ordem de implementação (24/08), porque outras linhas
> integraram no meio. ~~`PROJECT_SCHEMA` 84~~ · ~~registro `ph2d-ecs` 57~~ · ~~"o repo nunca chama
> `clear_trackers`, zero ocorrências"~~. Os valores abaixo são os **lidos no código da worktree**,
> com o comando ao lado. *Um número que soma entre linhas conta-se, nunca se escolhe.*

⚠️⚠️ **RE-MEDIDA outra vez em 2026-08-24 (tarde), depois de a F0+F1-parcial INTEGRAR.** A tabela
abaixo é a de DEPOIS; a de antes está no [handoff](handoffs/HANDOFF_INTEGRACAO_line_components_F0_F1parcial_2026-08-24.md) §3.
*Uma tabela de números medidos tem prazo de validade curto num repo com linhas paralelas.*

**A base de hoje (`main` @ `0f5ce8040`), medida:**

| O quê | Valor lido | Onde |
|---|---:|---|
| `PROJECT_SCHEMA` | **97** | [`project_schema.rs:267`](../../shells/desktop/src/project_schema.rs) |
| `WorldSnapshot::VERSION` | **2** ✅ (esta linha) | [`save.rs:85`](../../crates/ph2d-ecs/src/scene/save.rs) |
| Registro `ph2d-ecs` | **70** ✅ | [`registry_tests.rs:147`](../../crates/ph2d-ecs/src/scene/registry_tests.rs) |
| Espelhos render/script | **71** cada ✅ | `ph2d-render:55`, `ph2d-script:62` |
| Registro física | **32** | `ph2d-physics-ecs/src/lib.rs:194` |
| ADRs | último **0167** (próximo livre: **0168**) | `scripts/adr-index.sh` |
| `clear_trackers` no repo | **1** — num TESTE (`field3d_profile_live_tests.rs`) | ⚠️ **este teste não pode quebrar** quando a F2 passar a chamá-lo por captura |

⚠️ **DOIS números desta linha foram RECONTADOS na integração, e o padrão vale para a próxima:**

- **O degrau de schema desta linha ficou `96`** e o da `line/Vector` (input map) **nasceu 96 e foi
  recontado para 97**. ⇒ `PROJECT_SCHEMA` é hoje **97**, e o próximo degrau é **98**.
- **O ADR-0164 desta linha ficou** e o da *extracção quad* **foi renumerado 0164 → 0167** — o número
  estava tomado **duas** vezes e a colisão passou **muda**, exactamente como o §5.0 avisa.

⚠️⚠️ **UM BURACO NOMEADO na escada, e ele é teórico — MEDIDO, não suposto.** O `project_load.rs`
tem braços para `PROJECT_SCHEMA` (97) e `95`; **um ficheiro `v96` seria RECUSADO**. O `v96` só
existiu num `main` intermédio, entre as duas integrações do mesmo dia. ⇒ **Não há ficheiro nenhum
para perder:** `find /home/enio -name '*.ph2dproj'` devolve **zero** — nem v95, nem v96, nem v97.
⛔ Construir o braço `96` agora seria código defensivo para uma versão que **nunca escreveu um
ficheiro**; a decisão de o fazer é do Enio, e o custo de errar é baixo nos dois sentidos.

⚠️⚠️ **E o mesmo zero desmente um SMOKE que este plano e o handoff pediam.** *«Abrir um `.ph2dproj`
gravado ANTES de hoje»* **não é executável** — não existe nenhum. A migração está provada pelo gate
`the_frozen_v95_bytes_still_load`, que **constrói** os bytes v95, e é essa a prova que existe. Quem
quiser o smoke de verdade tem de **fabricar** um v95 (checkout de um commit antigo + gravar).

**O que esta linha ACRESCENTA (é isto que colide):**

| O quê | Valor | Quem soma |
|---|---|---|
| Crates novas | `ph2d-component-desc` (F0) · `ph2d-asset-index` (F6) | workspace glob |
| Componentes novos no registro | `SiblingOrder` (F1) · `SpriteCornerTint` · **`SpriteGrid`** · `SpriteRegion` (F1.6, corte da Sprite) · ✅ `MasterRoot` (F4.1) · ✅ `InstanceOf` (F4.2) · `ObjectInstance` (F4.4) | ✅ `ph2d-ecs` 69 → 70 → 73 → 74 → **75**; espelhos render/script 70 → 71 → 74 → 75 → **76**. ⚠️ O `StableId` **NÃO** entrou (ver F1), e o **`MasterPiece` também não** — ele é DERIVADO (`assign_master_pieces`), e um valor derivado no arquivo envenena o undo. ⚠️ **`SpriteGrid`, não `SpriteSheet`** — ver F1.6 |
| **Campos novos no `ComponentDesc` / `FieldKind` (F4.2)** | ✅ `FieldKind::Ref` (variante **apendada** no fim) · os 5 primeiros campos declarados `is_ref: Object`: `PhysicsJoint.body_a/b` · `PulleyWheel.rope/.body` · `InstanceOf.master` | ⚠️ **é a declaração que FAZ o remap acontecer** — censo de dois lados em `shells/desktop/src/instance_refs.rs`: declarar uma referência sem remapeador **reprova** |
| **Campo novo no `ComponentDesc` (F4.2)** | ✅ `owned_document: bool` + o construtor `D::owned_bridge` — os **quatro** de `catalog/bridges.rs` | ⚠️ **append-only**: os outros quatro construtores passam `false`. É o que faz a cópia profunda **não** levar o id de um documento possuído 1:1 |
| `WorldSnapshot::VERSION` | 1 → **2** (F1) | `save.rs` |
| `PROJECT_SCHEMA` | ✅ **96** (F1, integrado) · ✅ **98** (F1.6, o corte da Sprite) — ⚠️ o `97` é da `line/Vector`; o **próximo** degrau é o **99** | escada + tripla |
| Ids de widget novos | ✅ `INSP_ADD_COMPONENT` (F3, o `+` do cabeçalho do Inspector) · ✅ `CMD_PALETTE_SHOW_ALL` (F3, a caixa da banda da paleta — mora em `widget/command_palette/header.rs`, ao lado dos irmãos `CMD_PALETTE_*`) | `ph2d-editor-core/src/ids/` + o gate `node_id_collisions` |
| **Ações do barramento novas (F3)** | ✅ `EditorAction::HierAddRoot` (o botão `Add` da Hierarquia, **sem payload**) · ✅ `EditorAction::InspectorAddComponentRequested { entity_bits }` | ⚠️ **e uma MORREU: `PlayerFieldEdit::Add`** — o botão que a levantava vivia numa face vazia que a poda apagou |
| **Campo novo no `ComponentDesc` (F3)** | ✅ `requires: &'static [&'static str]` + o construtor `authored_requiring` | ⚠️ **append-only**: os outros três construtores passam `&[]`. Duas entradas em 108 tipos (`RigidBody → Collider`, `PlatformPlayer → RigidBody`) |
| **Campo novo no `PaletteModel` (F3)** | ✅ `toggle: Option<PaletteToggle>` | ⚠️ **obrigatório no literal** — o compilador apontou os **6** sítios de construção, e é isso que se quer |
| **Catracas de LOC descidas (F3)** | `apply_event_impl` 292 → **276** · `command_palette.rs` 500 → **423** (a banda saiu) · `save.rs` 704 → **345** (os testes saíram) | ⚠️ o `save.rs` estava **acima do teto desde a F2** e ninguém tinha corrido o gate |
| **Superfície pública nova (F0, feita)** | `ComponentRegistry::register_default::<T>` · `ComponentTypeEntry::insert_default` · `ComponentTypeEntry::desc` | ⚠️ **`register_inner` é privado** — as duas portas públicas são `register` (sem default) e `register_default` |
| **Sítios de chamada convertidos (F0, feita)** | **109** `reg.register::<T>` → `register_default::<T>`, menos **27** revertidos (sem `Default`) = **82** convertidos | ⚠️ 5 arquivos: `ph2d-ecs/scene/registry.rs` (70, um deles num teste) · `-render` (1) · `-script` (1) · `-physics-ecs` (32) · `-field-ecs` (5). **É a maior superfície de colisão desta linha** — uma linha que acrescente um componente toca o mesmo arquivo |
| **Dependências novas (F0, feita)** | `ph2d-ecs` → `ph2d-component-desc` · `shells/desktop` → idem · `ph2d-panel-inspector` → idem | ⚠️ conta para o `machete` no `ship.sh` |
| **Arquivos de teste novos (F0, feita)** | `shells/desktop/tests/every_registered_component_is_described.rs` (5 censos) · `ph2d-panel-inspector/tests/the_ordering_labels_come_from_the_descriptor.rs` (2) | nomes novos, sem colisão |
| Componentes acrescentados na F0 | **nenhum** — a F0 não move contador | ✅ a **F1** moveu: `ph2d-ecs` 69 → **70** (só o `SiblingOrder`; o `StableId` ficou FORA do registo) → **73** (os três do corte), espelhos 70 → 71 → **74** |
| Envs de smoke | `PH2D_INSTANCE_SMOKE=<n>` (F4+) · `PH2D_ASSET_BROWSER_SMOKE` (F7) | roteador de cenas próprio |
| Campo novo no `ProjectFile` | `stable_id_counter` (F1 — FORA do `ProjectState`, undo não rebobina) | conta no degrau do schema |
| Teto do ADR-0074 | ✅ +3 opcionais no corte da Sprite (o teto é 32) | `architecture_*` do Sprite |
| **`Sprite::VERSION`** | ✅ 4 → **5** (F1.6): 20 campos → **13**. Envelope `SpriteVersioned::V5` (0x02); o `V4` passa a apontar para o espelho **congelado** `SpriteV4` | gate `sprite_struct_field_count_capped` (20 → **13**) + `sprite_schema_version_v4` |
| **ADRs novos (F1.6)** | [`0070-amendment-8`](../architecture/decisions/0070-amendment-8.md) (o corte) · [`0071-amendment-1`](../architecture/decisions/0071-amendment-1.md) (o 4.º canal de tinta muda de casa) | ⚠️ números **contados** contra `decisions/`, não escolhidos |

---

## §F0 — O descritor de componente

**Objetivo:** um componente descreve-se UMA vez, e disso derivam inspector, override, remap de
referências, política de propagação — **e agora também a paleta da F3** (categoria + aplicabilidade,
[ADR-0166](../architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)).

**Pronto quando (observável):** a seção *Ordering* do Inspector é **pintada pelo descritor** (não
mais artesanal) e fica **visualmente idêntica** à atual; `insert_default` existe na vtable e um
teste insere `SortingLayer` por `type_id` sem conhecer o tipo.

**Toca:**
- **Nova** `crates/ph2d-component-desc/` — crate-FOLHA (⛔ sem `bevy_ecs`, sem `ph2d-nodegraph`;
  o precedente é `ph2d-warp-style`): `ComponentDesc { fields: &[FieldDesc], … }`,
  `FieldDesc { field_id: u16 /*append-only, estilo tag protobuf*/, name, kind: FieldKind,
  policy: Propagation, is_ref: Option<RefKind> }` + `FieldKind` espelhando o vocabulário do
  `ParamRow` (Scalar·Color·Toggle·Enum·Angle·Seed·Text·…).
- ⭐ **E as três declarações que o ADR-0166 exige** — no MESMO descritor, porque a paleta e o
  gating de seção têm de ler a mesma fonte:
  - `category: ComponentCategory` — o grupo colorido da paleta. É o `NodeUiCategory` dos
    componentes; ⚠️ **as categorias derivam-se do que os 107 registados JÁ são**, não se inventam
    (Identity/Transform · Ordering · Rendering · Image · Animation · Anchors · Vector · Physics ·
    3D · Scripting). Escreva a tabela **contando os tipos**, e deixe a contagem por categoria ao lado.
  - `attach: Attach` — `Authored { … }` ou **`Machinery`**. As quatro pontes de identidade
    (`VecPathRef` · `PaintedDoc` · `BakedForm` · `FlipObjectRef`) são `Machinery`: nunca oferecidas
    na paleta, nunca uma seção. *A ausência passa a ser declarada, não um esquecimento.*
  - `applies_to: ObjectKinds` — bitset sobre `{Empty, Image, Vector, Flip, Painted, Model3D, …}`.
    ⚠️ **Um tipo de objeto lê-se por PRESENÇA de marcador** (`Sprite` · `VecPathRef` ·
    `FlipObjectRef` · `PaintedDoc` · `FieldObject` · `BakedForm`; nenhum ⇒ vazio) — o `ObjectKinds`
    é **derivado do marcador**, senão vira a segunda fonte de verdade que o ADR-0166 §3 proíbe.
- `ph2d-ecs/src/scene/registry.rs` — `ComponentTypeEntry` ganha `insert_default: Option<fn>`,
  `desc: Option<&'static ComponentDesc>`, `component_id: ComponentId` (preenchido no boot;
  pré-requisito do scan da F2) e `patch_field`/`read_field` por `field_id`.
- `ph2d-panel-inspector` — UMA seção piloto derivada (Ordering), atrás do mesmo paint.

**Testes:** gate de **snapshot da tabela de `field_id`** por tipo descrito (mudar/reordenar id =
vermelho; apender = ok) — é o `FormerlySerializedAs` de graça; prova de mutação: trocar dois
`field_id` ⇒ o gate mata. Round-trip `patch_field`→`read_field` para cada `FieldKind`.
⭐ **E o CENSO da aplicabilidade** (o gate que impede o `applies_to` de apodrecer): todo tipo
descrito declara `attach`; nenhum `Authored` declara `applies_to` **vazio** (seria inalcançável em
todo objeto — um componente que existe e nunca aparece); nenhum `Machinery` tem seção. Prova de
mutação: pôr `SliceNine` como `Machinery` ⇒ o censo da F3 mata (ela some da paleta).

**Não fazer:** descrever os 107 tipos já — F0 descreve `Transform`, `Name`, `Sprite` + os de
ordering (o resto entra por demanda nas fases seguintes; a tabela cresce append-only).
⚠️ **Mas `category`/`attach`/`applies_to` são para TODOS os 107 desde já** — são uma linha por tipo,
e é o que a F3 precisa para a paleta não nascer com buracos. Descrever *campos* é caro; declarar
*em que gaveta o tipo vive* não é.

---

### ✅ F0 — FECHADA em 2026-08-24. O que ela mediu (e o que mudou por causa disso)

⭐ **O `Attach` tem TRÊS estados, não dois — e quem o provou foi o compilador.** A versão
desenhada era `Authored`/`Machinery`. Ao converter os registadores para `register_default`,
**27 dos 109 tipos não implementam `Default`**, e **17 deles estavam marcados `Authored`**: a
paleta oferecê-los-ia e não os conseguiria construir (anexar é inserir o **ponto neutro do
tipo**). Entrou `Attach::Intrinsic` — *dado do artista que chega com o GESTO, nunca oferecido,
mas que **pode** ter seção* (a `Sprite` tem a maior de todas). Gate:
`every_offered_component_can_be_constructed`.

⚠️ **E dentro dos 27 há DUAS espécies, com consequência para a F3:**
1. **Não há neutro que signifique nada** — `VecShape` sem geometria não é uma forma vazia; a
   `Sprite` exige uma `source`; um objeto sem `Name` não é um objeto de nome vazio.
2. ⚠️ **O neutro existe e anexá-lo seria um NO-OP** — a cerca que `MassOverride` e `Dominance`
   documentam: *"absent = the neutral default and the Inspector detaches it at 0 (a project
   file stays free of the no-op)"*. Neles a **presença** carrega o sentido, e o valor de
   anexação teria de vir do **contexto** (a massa que o corpo tem agora).
   ⇒ ⭐ **Nem todas as cinco portas por-seção são redundantes com o `+`** (emenda ao ADR-0166):
   as que **SEMEIAM do valor vivo** fazem algo que a paleta genérica não pode fazer. A F3 tem
   de as distinguir antes de podar.

⭐ **O piloto (§7 Ordering) achou um defeito ao ser ligado:** o rótulo de cada linha vivia em
**dois** sítios — literal no pintor, `FieldDesc::name` no catálogo — e eles **já discordavam em
2 das 10 linhas** (`Sort At Root` × `Sort at Root`; `Y-Sort` × `Enabled`). Hoje o pintor lê o
descritor (`field_label`/`marker_label`), o descritor foi corrigido **para o que o produto
pinta**, e há dois gates com prova de mutação.

**Entregue:** crate-folha `ph2d-component-desc` (vocabulário + catálogo de **108** tipos,
cortado por 7 famílias) · `ComponentRegistry::register_default` + `ComponentTypeEntry::{insert_default, desc}`
· **13 gates**, todos com prova de mutação (6 no catálogo · 5 de censo na shell · 2 no piloto).

⛔ **NÃO entregue, de propósito:** o `component_id: ComponentId` que a linha «Toca» acima
prevê. Ele é pré-requisito do **scan por archetype da F2** e não tem consumidor hoje — armá-lo
agora seria o fio órfão que a DIRETIVA §1 chama de causa nº 1 de feature morta. A F2 acrescenta-o
**com** o scan que o lê.

⚠️ **Duas armadilhas de arnês que esta fase pagou** (as duas com o gate a ficar verde sobre
código errado): um `shutil.copy2` no restore de mutação devolve o **mtime** antigo e o cargo
serve o binário da mutação ([memória](../../project-memory/feedback_a_mutation_restore_that_preserves_mtime_leaves_cargo_stale.md));
e um gate estrutural escrito **por linha** não vê um literal numa invocação **multi-linha** — a
1.ª versão do `the_painter_does_not_hardcode_the_row_labels` ficou verde sobre um rótulo à mão.

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
5. ✅ **As duas famílias de `stable_name_id` — FEITAS** (5a física, 24/08 · 5b timeline, 25/08).
   `stable_name_id` fica **deprecado com nota**, não removido ([`name.rs`](../../crates/ph2d-ecs/src/name.rs)),
   e continua a ser a função certa para **autoria** (`stable_id_for_name`) e **legado**.

   ⚠️⚠️ **DUAS afirmações desta linha foram REFUTADAS por medição antes de se tocar em código**
   (sondas em [`timeline_persist_tests.rs`](../../shells/desktop/src/timeline_persist_tests.rs), o
   mecanismo no cabeçalho de [`timeline_persist.rs`](../../shells/desktop/src/timeline_persist.rs)):

   - ⛔ **«renomear desliga a binding da timeline» era FALSO.** A metade viva do
     `refresh_and_heal_bindings` re-carimbava o `wire_id` a partir do nome CORRENTE a cada frame, então
     o rename era seguido em silêncio — inclusive atravessando o arquivo de projeto. *Duas famílias
     partilhavam a função e **não** partilhavam o defeito:* a junta guardava o hash e nunca o
     refrescava; a binding refrescava-o sempre. ⇒ A **condição de pronto (3) mede a física**, e o que
     a troca cura na timeline é **outra coisa**: dois objetos com o mesmo nome faziam a animação
     **DESAPARECER** (dormente, sem badge nem erro). Gate novo: `two_homonyms_no_longer_hide_the_animation`.
   - ⭐⭐⭐ **E em 2026-08-27 a nota do ROTEADOR ainda dizia o contrário — três dias depois de o
     trabalho estar pago.** O `CLAUDE.md` §5 carregava *«a F1 continua PELA METADE: a física aponta
     por identidade, a timeline ainda não — renomear um objeto animado desliga o binding»*, que está
     errado **nas duas metades** (a 5a fechou em 24/08, a 5b em 25/08). ⚠️ **A causa é a ausência de
     um GATE:** o comportamento certo existia e nada o afirmava, então a frase falsa podia envelhecer
     sem que uma corrida a contradissesse. *Um comentário não é uma prova; um doc de plano também
     não.* ⇒ dois gates novos em [`timeline_persist_tests.rs`](../../shells/desktop/src/timeline_persist_tests.rs):
     `renaming_an_animated_object_does_not_unbind_it` (o rename **atravessa o arquivo**, que é a
     metade que a nota acusava) e `a_stranger_with_the_old_name_does_not_capture_the_animation` (o
     nome sozinho já não basta — a prova de que o substrato de facto mudou). Duas mutações, duas
     mortes, com o controlo do filtro feito só sobre os gates novos.
   - ⚠️ **E as duas fixturas mordiam antes de medir:** dois `SimWorld` novos alocam `StableId` a
     partir do mesmo contador, então o «estranho» nascia com **exactamente** o id do herói e o gate
     reprovava sobre produto correto; e a asserção indexava `bindings()[0]` numa lista que a **purga**
     já tinha esvaziado — *um gate que só sabe ler um dos desfechos certos reprova metade deles*.
   - ⛔ **`frame_solve.rs:139/218/251` NÃO pertence a este passo, e não deve mudar.** Ali o
     `stable_name_id` hasheia **o texto que o autor escreveu** (`Expr::Attr("Ball.x")` — o
     `resolve_link` parte a string no `.`), não uma referência guardada. Trocá-lo por `StableId` poria
     a fórmula a mostrar `Ball` e a ler outro objeto: *uma variável de fórmula é um nome por
     construção*. `persist.rs:28/54/88` também não muda — são genéricas sobre `WireId` (closures), e o
     que mudou foi o **significado** do número, escrito no shell.

   **O que de facto mudou (5b):** `wire_of` devolve o `StableId` · o mapa do `upkeep` leva **duas
   chaves** (identidade + o hash legado, com a identidade a ganhar) · `serialize` passou a `&mut World`
   e garante os ids ele próprio (corre **antes** da captura). ⭐ **Sem degrau de schema:** um documento
   legado sobe de substrato sozinho — heal por hash num frame, re-carimbo com o id no seguinte.
   ⚠️ Os mundos de teste que spawnavam `Name` **sem `Transform`** foram corrigidos: nenhum objeto deste
   app nasce assim, e a fixtura irreal deixava-os sem id.
6. ✅ **Corte da Sprite — FEITO** (2026-08-25). `per_corner_tint` → `SpriteCornerTint` · a folha
   inline → **`SpriteGrid`** · a região → `SpriteRegion`. **Ausência = default benigno**, criar
   sprite continua 1 gesto. Os três somam nos contadores (§0.1.2) e o cap do ADR-0074 recebe +3.
   Decisões em [ADR-0070-amendment-8](../architecture/decisions/0070-amendment-8.md) +
   [ADR-0071-amendment-1](../architecture/decisions/0071-amendment-1.md).

   ⚠️ **DUAS coisas que esta linha dizia saíram diferentes, e a razão fica escrita:**

   - ⛔ ~~`SpriteSheet`~~ → **`SpriteGrid`**. A `ph2d-ecs` **já tem** `SpriteSheetRef` e
     `SpriteSheetFrame`, e as duas significam a folha **hand-packed** — outra coisa. Um terceiro
     nome quase igual para uma ideia distinta é o que se lê ao contrário; *grelha* já é a palavra
     que os docs usam para esta (§11 Animation).
   - ⛔ ~~«corta TRÊS campos»~~ → **sete campos, em três componentes**. A conta original contava
     *grupos*, não campos: a grelha são três (`hframes`/`vframes`/`frame`) e a região outros três.
     A `Sprite` foi de **20 para 13**, não para 17.

   ⭐ **E a releitura dos 20 campos com a pergunta do ADR-0166 CONFIRMOU os três grupos** — com
   dois candidatos considerados e **recusados com razão**: o `self_tint` (o Godot põe `modulate` e
   `self_modulate` sempre; o par com `tint` é a base) e o `tint_fill` (é um **modo** do tint, não
   uma feature — um componente de um bool só é pior ergonomia que o campo).

   ⭐⭐ **A PRESENÇA do `SpriteRegion` é o antigo `region_enabled`**, e isso apaga um estado que
   ninguém conseguia ler (`enabled = false` **com** um rect autorado ao lado). O mesmo movimento
   apaga o `region_filter_clip` de toda sprite **sem** região — um bool que não se aplicava a ela,
   e cujo `serde default` o próprio campo v4 documentava como **errado para Individual**.

   ⚠️ **O degrau de schema (97 → 98) é obrigatório apesar de a FORMA do `ProjectFile` não mudar** —
   os bytes da `Sprite` vivem dentro do `Vec<u8>` opaco de um `ComponentBlob`, que o parse
   atravessa sem olhar. ⛔ **A tripla do §0.1.7 não podia ver isto:** ela mede a forma da `VecScene`
   e do `FlipDoc`, e nenhuma se mexeu. *Um degrau não é só «a estrutura mudou» — é «os bytes
   deixaram de significar o mesmo».* A migração (`project_migrate_sprite`) é uma travessia do
   snapshot, e o **v95 sobe encadeado** (95 → 96 → o corte).

   ⚠️ **MEDIDO ao fazer:** o `load_sprite` — documentado como *"a ÚNICA forma sancionada de ler um
   sprite persistido"* — **não tem chamador de produção**, só os próprios testes. E o
   `SpriteSheetRef` (folha hand-packed) é construído **por cima** da região: todo sprite de folha
   tem de ficar com um `SpriteRegion`, senão amostra a folha INTEIRA.

**Testes:** determinismo — `state_hash` idêntico em duas capturas do mesmo estado e através de
restore; mutação: remover o remap de `StableId` na cópia de blobs ⇒ gate de unicidade mata.
⚠️ **«Re-capturar o golden» era um FANTASMA, medido em 2026-08-25.** O `deterministic_hash` do c9
de facto muda de valor com o snapshot v2 — e isso **não reprova nada**: os três gates C9 do
`spike.yml` **não têm baseline gravado**; cada SO imprime o hash como artefacto e o
`determinism-compare` reprova só se os três **discordarem entre si**. *Um aviso pode estar certo
sobre o mecanismo e errado sobre o que ele causa*
([memória](../../project-memory/feedback_the_c9_hashes_are_compared_across_oses_not_against_a_stored_baseline.md)).
⇒ O que a F4 deve ao c9 continua a ser **a LANE nova** (mestre + instância): hoje ele não tem
instância nenhuma, então a cláusula *«o hash 3-OS não viola»* é verdadeira **por vacuidade do
gate**, e é isso que a lane fecha.

---

## §F2 — O undo incremental

**Objetivo:** a captura custa o tamanho da edição, não do mundo; a pilha guarda deltas.

**Pronto quando:** ✅ **CUMPRIDO** — bench `#[ignore]` em `crates/ph2d-ecs`, `load 2,75`, três
corridas seguidas:

| cenário | run 1 | run 2 | run 3 | barra | spike 21/08 | hoje (sem incremental) |
|---|---:|---:|---:|---:|---:|---:|
| nada mudou | **0,189** | 0,188 | 0,189 | 0,300 | 0,269 | 23,8 |
| 10 % mudou | **0,613** | 0,619 | 0,587 | 1,000 | 0,953 | — |

⭐⭐ **A versão SEGURA bateu o spike**, e isso responde uma pergunta que esta fase teve de abrir:
a `ph2d-ecs` tem `#![forbid(unsafe_code)]`, e o caminho rápido do bevy (ler a coluna de ticks da
tabela) devolve `&[UnsafeCell<Tick>]` — inalcançável aqui. ⇒ **a cerca escolheu o algoritmo**, e a
pergunta *"isolar o scan numa crate que permita `unsafe`?"* (o precedente do Opus, ADR-0116)
**não precisa de ser feita**. O que fechou a diferença não foi acesso mais cru: foi **tirar as
buscas de mapa do caminho comum** — quatro por entidade na 1.ª versão, **zero** hoje (a cache guia
e o mundo responde).

⚠️ **A variação entre corridas é ±0,001 ms parado**, e é isso que distingue a medição da leitura:
a MESMA implementação deu `0,290` a `load 3,4` e `0,627` a `load 17`.

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

## §F3 — O Inspector mostra o que o objeto TEM + o `+` e a paleta + objeto vazio — ⭐ walking skeleton

> **Governada pelo [ADR-0166](../architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)**
> (instruções do Enio de 2026-08-24). Esta fase cresceu: ela já era *"criar objeto vazio + Add
> Component"*; passa a incluir **tirar do painel o que não é do objeto base**.

**Objetivo:** o artista cria um objeto vazio, vê **duas** seções (não doze), e acrescenta o que
precisa por **uma** porta — um `+` que abre a paleta que o Motion já usa, com categorias coloridas,
busca, e **filtrada pelo tipo do objeto selecionado**.

**Pronto quando (o smoke que o Enio roda):** criar objeto vazio na raiz (o botão **Add** da
Hierarquia — hoje morto, `§0.3 do doc 01`) → o Inspector mostra **Name + Transform, e mais nada** →
`+` → a paleta abre com as categorias → buscar "Rigid" → o diálogo mostra **"RigidBody — traz junto:
Collider"** → aplicar → a seção Physics **aparece** → salvar → reabrir → Ctrl+Z remove o componente
**e a seção some**. E, na sprite: selecionar uma imagem, abrir o `+`, e ver que **9-Slice é oferecido**;
selecionar um objeto vetorial e ver que ele **não é** (fica sob *Show all*, esmaecido, com a razão).

**As quatro peças (a ordem entre elas é lei — vide o ⛔ no fim):**

1. **O Inspector passa a pintar por PRESENÇA.** A cascata literal (hoje: `populate()` é uma lista de
   19 funções e o paint é uma sequência escrita à mão, com `any_live_section(flags: [bool; 11])`)
   passa a derivar do descritor + do que a entidade tem. **Base = `Transform` + `Name`.** Tudo o mais
   — ordering, sampling, blend, folha, 9-slice, âncoras, animação, física, joint, roda, player —
   aparece **se, e só se**, o componente estiver lá.
2. **O `+` no cabeçalho do Inspector** (`INSP_ADD_COMPONENT`, id novo — §0.2) abre a paleta.
3. ⭐ **A paleta NÃO é um modal novo** — é o
   [`ph2d_editor_core::widget::command_palette`](../../crates/ph2d-editor-core/src/widget/command_palette.rs),
   que já é **genérico por desenho** (o doc-comment dele diz *"reusable by any future
   browse-everything picker"*: ele conhece só `PaletteModel`, e quem abriu mapeia o `id` de volta).
   Já tem scrim, cascata de entrada, busca (`item_matches` — **um** predicado servindo o filtro
   pintado e o `Enter`), sub-clusters e promoção a 2 colunas. **Construa só o MODELO**, copiando os
   dois precedentes: [`motion_bridge_library.rs`](../../shells/desktop/src/render_loop/motion_bridge_library.rs)
   (a biblioteca de nós) e [`global_palette.rs`](../../crates/ph2d-editor-core/src/screens/hero/global_palette.rs)
   (o `Ctrl+K`). ⚠️ **O dreno do pick é CONDICIONAL** (`take_command_pick_if`): o canal já tem dois
   consumidores, e um `take` incondicional faria quem recebe o pick ser *a ordem dos drenos no
   quadro*. O terceiro dreno reconhece só os **seus** ids.
4. **O filtro por tipo de objeto.** A paleta abre filtrada pelo `ObjectKinds` do selecionado
   (derivado do marcador — F0). O inaplicável **não some**: fica sob *Show all*, **esmaecido e com a
   razão nomeada**. ⛔ Nem no-op silencioso (DIRETIVA §2), nem apagar da lista (um componente que
   existe e é invisível lê-se como defeito). *Esmaecido ainda despacha* — aqui, a explicação.

**Toca:** handler do `HIERARCHY_ADD` (spawn `Transform+Name+StableId+RootOrder`) ·
`ph2d-panel-inspector` (a cascata; `populate`/`paint`/`sync` deixam de ser listas literais) ·
`shells/desktop` (o modelo da paleta + o 3º dreno; `snapshots.rs` deixa de publicar seções que a
entidade não tem) · `ph2d-editor-core/src/ids/` (o `+`) · `#[require]`-equivalente por descritor
(`requires: &[type_id]`; ⚠️ **a cascata é MOSTRADA antes de aplicar** — correção da crítica medida ao
Bevy, doc 02 §1.4) · `EditorCommand::Spawn`/`SetComponent` já existem — é wiring.

**As CINCO portas de hoje são subsumidas, não mantidas ao lado:** `INSP_PLAYER_ADD` ·
`INSP_ANCHOR_ADD` · `INSP_ANIM_ADD` · `INSP_PHYS_ADD` + o botão de anexar da §5 9-Slice. Duas
respostas a *"como se adiciona um componente?"* é a divergência que esta fase existe para apagar. (Um
botão de anexar pode sobreviver como atalho **da seção já visível**; o que não sobrevive é ser a
única rota.)

**Testes:** seam completo (pintado/populado/clicado/**sequência** — as 4 perguntas) sobre o `+` e
sobre um pick; **censo de alcance nos dois sentidos** — todo `Authored` aparece na paleta de algum
tipo de objeto **e** nenhum `Machinery` aparece; o gate de presença (`SliceNine` ausente ⇒ zero
widgets da §5 no hit-index — ⚠️ não basta "não pinta": um id órfão no índice continua clicável);
anexar é **inerte** (bytes do componente == default) e **desfazível** (1 passo); a11y (HR-12);
colisão de id (`node_id_collisions` cobre o `INSP_ADD_COMPONENT`).

⛔ **A ORDEM DENTRO DA FASE É LEI, e a razão está medida:** a peça **1 não pode ir antes das 2–4**.
Hoje várias seções são a **única rota** para a feature delas (a §14 Player publica `Some` para todo
corpo dinâmico **com ou sem** o componente, com o comentário *"porque o botão dela é o que faz o
comportamento existir"* — [`snapshots.rs`](../../shells/desktop/src/render_loop/snapshots.rs)).
Apagar a face vazia antes de a porta nova estar viva e testada torna a feature **inalcançável** — e
é assim que a lei da face vazia foi paga da primeira vez
([memória](../../project-memory/feedback_the_three_ui_seam_questions_miss_the_fourth_the_sequence.md)).
⇒ **porta primeiro, poda depois**, com o censo verde entre as duas.


---

### ✅ F3 — FECHADA em 2026-08-25. O que ela mediu (e o que mudou por causa disso)

⭐ **A ordem foi honrada e ela pagou-se:** porta primeiro (o `+`, a paleta, o filtro), **censo verde
entre as duas** (`component_reach_tests`: **0** autorados que o registo constrói e nenhuma paleta
oferece), poda depois.

⚠️ **TRÊS seções têm DUAS metades, e a 1.ª redacção da poda apagava a UI de um dos lados.** Cada uma
foi apanhada por um gate que já existia, e o mecanismo é o mesmo — *a seção descreve uma RELAÇÃO, e
os dois lados dela são componentes diferentes*:

| Seção | O que a 1.ª redacção gateava | O lado que ela apagava |
|---|---|---|
| §12 Anchors | `NamedAnchorList` (quem OFERECE âncoras) | `AnchorMount` (quem ANDA numa âncora do pai) |
| §11 Animation | `SpriteAnimations` (a biblioteca) | `SpriteAnimator` (o transporte) |
| §11 Physics | `RigidBody` (o corpo) | `Collider` sozinho (uma PEÇA de um corpo ancestral) |

⚠️ **E a §11 Physics guarda ainda o `rig_parts > 0`**, que não é folga: o gesto canónico do *Rig* é
marcar o TRONCO do personagem, que costuma ser um nó de organização **sem corpo nem collider** — e é
um gesto sobre uma SUBÁRVORE, coisa que a paleta de componentes não sabe exprimir.

⭐ **A emenda da F0 virou código: os DOIS seeds.** *Nem toda porta por-seção é redundante com o `+`;
as que SEMEIAM do valor vivo fazem o que a paleta genérica não pode.* O `insert_default` é
type-erased — constrói o `Default` do tipo e **não conhece a entidade**:

| Componente | O `Default` sozinho | O seed |
|---|---|---|
| `Collider` | uma **bola de meio metro** debaixo de um sprite quadrado (o desencontro de 2026-07-18) | as meias-extensões do `Sprite` |
| `PlatformPlayer` | a cápsula canónica **tangente** ao chão | a altura que de facto paira |

⛔ **A meia-extensão estava escrita TRÊS vezes** (`Add`, `AddShape`, o seed) — hoje é
`sprite_half_extents`, uma porta. E o `component_seed` tinha um `match` **ao lado** de uma lista de
nomes que o gate percorria: duas respostas à mesma pergunta, hoje **uma tabela** que os dois leem.

⭐ **O `requires` não era opcional.** Sem ele, anexar `PlatformPlayer` a um objeto sem corpo punha o
componente lá e a §14 **não aparecia** — a poda abria um buraco próprio. A cascata viaja no **rótulo
do item**, e é **FECHADA** (`Platform Player — brings Rigid Body, Collider`): mostrar só o 1.º salto
seria a queixa do Bevy um nível abaixo. ⛔ Só o **estrutural** entra: a barra é *o componente é
inerte sem aquele*, e a ponte da física consulta `(RigidBody, Collider, Transform)` — uma query, não
uma opinião.

⛔ **O que MORREU:** `PlayerFieldEdit::Add` + `INSP_PLAYER_ADD` («Make Platform Player») e a face
vazia da §14 — o botão vivia dentro da seção que hoje só se pinta **com** o componente, logo a porta
ficaria fechada sobre a própria chave. ⚠️ **E ele revelou a armadilha:** o `apply_player_edit`
usava a mesma função como guarda de EDIÇÃO, então a poda fechou a porta sobre o gesto que a abre —
*«a seção aparece?» e «esta edição é legal?» deixaram de ser a mesma pergunta*.

⚠️ **REVERSÃO deliberada:** a §14 deixou de exigir `BodyKind != Static`. Aquela era a condição de
OFERECER O BOTÃO, e o botão mudou-se para a paleta; mantê-la produziria o pior dos dois mundos — o
artista anexa pelo `+` e **nada aparece**.

✅ **O que FICA, com motivo medido, contra o que o plano previa:** `INSP_ANCHOR_ADD` e
`INSP_ANIM_ADD` **não são portas de componente** — eles acrescentam uma LINHA (uma âncora, uma tag)
dentro de um componente já presente. O plano listava-os como redundantes; a medição diz que não.
`INSP_PHYS_ADD` sobrevive como atalho da §11 quando ela está visível (o caso do rig).

⏳ **Ficou de fora, com o motivo:** a §4 **Sprite Sheet** NÃO foi gateada no `SpriteGrid`. Ela é uma
sub-seção do `sprite_info` e hospeda também o **Flip X / Flip Y**, que são campos da `Sprite` base —
gateá-la na grelha tornaria os dois inalcançáveis. *A poda pára onde a seção deixa de descrever um
componente só.*

**Entregue:** o `+` (`INSP_ADD_COMPONENT`) · a paleta filtrada por tipo de objeto com *Show all* ·
o objeto vazio na raiz (`HierAddRoot`) · a poda de **8** seções · os **2** seeds · o `requires` com
a cascata no rótulo · **32** gates novos (13 combinações de presença nos dois sentidos · o censo de
alcance de dois lados · 7 de seed/cascata · 5 de SEQUÊNCIA · 4 de catálogo · 3 de widget/chrome).


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

### Estado das FATIAS da F4 (a linha atualiza; a fase só fecha com os três smokes-gate)

| Fatia | O quê | Estado |
|---|---|---|
| F4.1 | O mestre existe e é **INERTE** — a condição (a) da refutação 1 | ✅ 2026-08-25 |
| F4.2 | **Instanciar**: cópia profunda + remap de identidade e de referências | ✅ 2026-08-26 — smoke-gate **1** |
| F4.3 | Sync vivo mestre→instância (`set_if_neq`, ordem determinística, `pose_owner`) | ✅ 2026-08-26 — smoke-gate **2** |
| F4.4 | Override **por componente** (`ObjectInstance`) — ⚠️ *por campo* foi refutado | ✅ 2026-08-26 |
| F4.5 | Destacar / Redefinir / Aplicar ao mestre + **os verbos na UI** | ✅ 2026-08-26 |
| F4.6 | O `VecInstance` subsumido (doc 04 §2.9) + degrau de schema | 🟨 **a** (documentos clonados) ✅ · **b** (geometria por conteúdo) ✅ **smoke OK 2026-08-27** (a causa do ✗ era a receita vetorial ainda DESENHAR — sobreposta à cópia) · **c** (matar o `InstanceLive` + migração) ⛔ **BLOQUEADA na F5** — ver abaixo |
| F4.7 | Lane do `physics_ecs_c9` com mestre+instância; ponto fixo sob física | ✅ 2026-08-27 — smoke-gate **3** |

**O que a F4.1 mediu e o plano não dizia:** ⚠️ **a refutação nomeia CINCO `QueryState` da ponte
(`bridge.rs:84-127`); são SEIS.** Ela cita uma *faixa de linhas de um ficheiro*, e a `WheelQuery`
nasceu noutro (`bridge/rope.rs`) depois disso. Era a pior de faltar: uma roldana é alcançada **pelo
nome da corda**, então uma dentro da biblioteca não só entraria no sistema vivo como **disputaria a
resolução** com a da cena. *Uma referência por faixa de linhas envelhece à velocidade do ficheiro.*

**O que a F4.2 mediu e o plano não dizia:**
- ⚠️ **A nota do catálogo sobre o `AnchorMount.anchor` está REFUTADA** (`catalog/image.rs` dizia
  *"é `RefKind::Object` quando a F1 migrar"*): o campo nomeia uma âncora **do próprio PAI**, e uma
  cópia profunda leva o pai junto — o nome continua a resolver dentro da cópia. Declará-lo pediria
  um remap que estragaria o que já funciona. *A estrutura da cópia apaga o caso especial.*
- ⚠️ **A ORDEM entre remapear e ligar é load-bearing**, e o erro é mudo: o mapa contém
  `mestre → cópia do mestre` (tem de conter — uma junta ancorada na raiz precisa dele), então
  inserir o `InstanceOf` **antes** do remap fá-lo apontar para a **própria cópia**. Gate:
  `the_instance_points_at_the_master_not_at_itself` (mutação mata).
- ⚠️ **A cópia profunda tem de perder `RootOrder`/`SiblingOrder` da raiz** — dois irmãos com a
  mesma ordem é o empate que a casa não tem. As peças **mantêm** os delas (é a receita).
- ⭐⭐⭐ **A correspondência peça↔peça é DURÁVEL, e por isso o `InstanceOf` está em TODA peça**, não
  só na raiz (F4.3). Emparelhar por posição na árvore era o caminho barato e **errado**: no dia em
  que o mestre ganha uma peça no meio, os índices deslizam e cada peça recebe os bytes da vizinha,
  em silêncio. ⚠️ A raiz deixa de precisar de um segundo componente — ela é *a peça cujo `master` é
  um `MasterRoot`*.
- ⭐⭐ **E ela NÃO pode levar o id de um documento POSSUÍDO.** Copiar bytes verbatim é a coisa certa
  para 100 dos 104 tipos e a **errada** para os quatro de `catalog/bridges.rs` (`PaintedDoc` ·
  `VecPathRef` · `BakedForm` · `FlipObjectRef`): o id é opaco, então a cópia ficaria a escrever no
  **mesmo** documento — duplicar uma sprite pintada devolvia um sósia que apaga a tinta do
  original. ⚠️ *A cópia rasa que existia acertava nisto por acidente* (ela levava quatro
  componentes e nenhum era ponte), e a profunda tinha de o decidir de propósito. ⇒ `ComponentDesc
  .owned_document`, com o gate `the_bridges_are_the_owned_documents` a prender a família à flag.
  ⛔ **Não é `Attach::Machinery`**, que é 17 tipos e treze deles copiam-se muito bem.

**O que a F4.3 mediu e o plano não dizia:**
- ⭐⭐⭐ **A LEI que a prova de mutação achou: *o que não PROPAGA não se REMAPEIA*.** O sync
  remapeava tudo, e o `InstanceOf.master` da raiz **é** a identidade do mestre — logo uma chave do
  mapa. O 1.º passe reescrevia-o para a identidade da **própria instância**; do 2.º quadro em
  diante a instância dizia-se instância de si mesma, o sync deixava de a encontrar, e **nada mais
  propagava**. ⚠️ **Todos os gates estavam verdes**: nenhum corria o passe DUAS vezes antes de
  medir. A régua que faltava é `the_link_survives_a_sync_so_the_next_edit_still_arrives`.
- ⭐⭐ **Para quem carrega REFERÊNCIA, a comparação por bytes não é decidível na propagação.** A
  junta do mestre nomeia os corpos do mestre e a da instância nomeia os dela **de propósito**:
  comparar dá *«diferente»* para sempre, e o passe reescrevia a junta todo o quadro (medido: 3
  escritas por quadro, uma por instância, com o mundo parado). ⇒ esses escrevem-se, **religam-se**,
  e só então se pergunta se mudou — e o passe volta a ser ponto fixo.
- ⛔ **DECLARADO: a pose de repouso de uma peça DINÂMICA não propaga.** O dono do `Transform` de um
  corpo dinâmico é o solver **sempre** (a resposta sai do `BodyKind`, que não sabe se a cena está
  tocando), então mover o braço da receita não move o das instâncias, nem depois de um Reset. É a
  condição (b) da refutação à letra, e a irmã da limitação que o plano já declara para a config de
  física. Gate com o nome inteiro:
  `the_rest_pose_of_a_simulated_piece_does_not_propagate_and_that_is_declared`.
- ⚠️ **O passe corre no EPÍLOGO do quadro** (`main.rs`, entre o render loop e o `post_frame_undo`),
  e as duas metades são a razão: *antes da captura* senão a escrita vira um passo de undo que
  ninguém deu; *depois do quadro* porque é aí que as edições do Inspector chegam ao mundo
  (`apply_editor_commands` corre no fim do laço) — pô-lo antes faria as instâncias andarem **um
  quadro atrás do mestre**.
**O que a F4.4 mediu e o plano não dizia:**
- ⛔⛔⛔ **«Override capturado por DIFF» é impossível, e a refutação é de uma linha:** um diff só diz
  *«estão diferentes»*. Se o mestre mudou, `mestre != instância`; se a instância mudou, **também**.
  Ler o diff como *«a instância mexeu-se»* transformaria cada edição da receita num override em
  todas as instâncias (a difusão pararia no gesto que a pediu); lê-lo ao contrário desfaria toda
  edição do artista no quadro seguinte. ⇒ o passe guarda **o eco do mestre** (o que a receita tinha
  no passe anterior), e aí as duas perguntas separam-se. ⚠️ **O eco custa o MESTRE, não as
  instâncias** — mil cópias partilham uma entrada.
- ⛔ **E o instrumento óbvio — o change tick — é CEGO à operação que mais dói:** a refutação 3 já o
  tinha medido (*«remover componente não muda tick de ninguém»*), e tirar um componente da receita
  é exatamente o que tem de chegar às instâncias.
- ⛔ **A granularidade é o COMPONENTE, não o campo** — e a refutação 3 já o dizia: *«sem
  `patch_field` por tipo, "campo tocado bloqueia propagação" vira "**componente** tocado bloqueia
  propagação"»*. Consequência a dizer em voz alta: mexer na posição de uma peça da instância
  congela também a **escala** e a **rotação** dela, porque as três vivem no mesmo `Transform`.
- ⭐⭐ **`ObjectInstance` é um CONJUNTO, não um mapa de bytes** — o plano copiava o Unity, onde os
  bytes são obrigatórios porque a instância **não é uma entidade real**. Aqui ela é (é a tese do
  ADR-0164): o valor já vive no componente da peça e viaja no ficheiro pela porta de sempre.
  Guardá-lo outra vez criaria duas fontes para o mesmo número. *A representação apaga o caso
  especial.*
- ⛔ **DECLARADO: um componente que carrega REFERÊNCIA propaga mas nunca CAPTURA override.** Medido:
  o solver **escreve dentro do `PhysicsJoint`** (semeia `local_a`/`local_b`, vira o `anchored`), e
  de fora isso é indistinguível de uma edição do artista — a 1.ª versão dava a toda instância com
  junta um override no primeiro tique, deixando-a surda à receita para sempre. ⇒ editar a junta de
  uma instância vale **até o mestre mexer na dele**.
- ⚠️ **O REVERT não é «tirar a chave»** — um gate disse-o: no passe seguinte a peça ainda difere e o
  mestre não mexeu, que é a assinatura de *«a instância mexeu-se»*, e o override renascia. O verbo
  era um **no-op visível**. ⇒ o revert **apaga o eco daquela chave**, e o passe cai na regra do 1.º
  encontro (*o mestre ganha*), que já estava escrita. *A saída não precisou de uma regra nova:
  precisou de esquecer.*
- ⚠️ **O EMPATE está declarado: os dois mudam no mesmo passe ⇒ a RECEITA ganha**, e não fica
  override. Editar o molde é uma difusão deliberada.

**⛔⛔ O que a MEDIÇÃO da F4.6c achou, e que muda a fatia (2026-08-27):**
- **Ela não é um porte — é um porte MENOS três features.** O plano diz *«verbos/UI ficam»*, e o
  sistema vetorial tem **seis** verbos contra os quatro do geral: além de *Create/Place/Detach/
  Reset*, ele tem **`Swap`** (trocar de mestre) e **`UpdateMain`**, e mais os **variants** —
  `vec_variants.rs`, que deriva o conjunto de *«os `VecComponentMain` irmãos do mesmo pai»* e é o
  `VecInstance` quem escolhe qual. ⇒ **matar o `VecInstance` hoje apaga os variants**, e variants
  são a **§F5** deste plano. *Uma fatia que se declara «porte» tem de contar os dois lados antes.*
- **Medida a superfície:** `VecInstance` em **24 ficheiros**. ⛔⛔ **E os dois números que aqui
  estavam eram os ERRADOS** (re-medido em 2026-08-30): *«~1 210 LOC de produção em quatro
  módulos»* omite o **`vec_variants.rs` (281 LOC)**, que é **precisamente a feature que a fatia
  existe para portar** — o núcleo são **1 658** e a superfície de produção inteira **≈ 2 961**
  (com a UI, os ids, as duas cenas de smoke e os ~291 de pontos de toque). E *«44 gates»* são
  **102** (92 em ficheiros de teste + 10 inline nos smokes). *Uma medição que exclui a feature que
  a fatia nomeia orça o trabalho errado.*
- ⛔⛔ **E a contagem de VERBOS também estava errada, nos dois lados.** *«Seis contra os quatro do
  geral»* conta **botões**; as rotas do `ComponentEdit` são **oito** e os gestos de autoria **nove**.
  ⚠️ As duas que a conta deixa de fora — `toggle_piece_visible` e a swatch de cor por peça — são
  **a única porta que PRODUZ um override** no sistema vetorial. *Uma fatia que conte seis apaga a
  porta de autoria e fica com o modelo do outro lado* — que é, ao contrário, exactamente o defeito
  que aquele módulo nasceu a curar. E o geral tem hoje **seis** verbos de menu, não quatro.
- ⭐ **E a lei que o plano manda re-medir DISSOLVE-SE.** O `vec_instance_follow` existe porque o
  modelo derivado perde a translação do mestre (`D(p) = (p − Tm)·I + Ti`), então uma alça ancorada
  paga a âncora numa translação que a cópia não herda. No mecanismo geral **não há delta**: a
  instância é uma sub-árvore REAL, a pose da raiz dela é dela (`ROOT_IS_ITS_OWN`) e a de cada peça
  chega verbatim. ⇒ nada a portar — *o substrato apaga a cura*. ⚠️ Vale a pena escrever isto porque
  a nota do módulo apresenta a compensação como uma lei do produto, e ela é uma lei do MODELO.
- ⇒ **A ordem certa é F5 antes de F4.6c**, e a F4.7 foi feita primeiro por ser independente.
- ⭐⭐ **RECONFERIDA em 2026-08-27, depois de as variantes existirem** (§0.0: *quem move o número que
  tornava algo inalcançável tem de reconferir a nota*). Das **três** features que o vetor tinha e o
  geral não, **duas e meia** passaram a existir:
  - **`Swap`** ⇒ `instance_variant::swap`, e com re-key determinístico, que o `Swap` do vetor não tem;
  - **`UpdateMain`** ⇒ `instance_verbs::apply_to_master`, que já existia desde a F4.5;
  - **variants** ⇒ existem, **menos os EIXOS de propriedade**. O `vec_variants.rs` lê
    `Size=Small, State=Idle` do `Name` e pinta **uma fileira por propriedade**; o cartão geral pinta
    **uma fileira só**, com um chip por versão. ⚠️ A lei dos eixos é do NOME, não do `VecInstance` —
    o `parse_combo` é genérico e re-hospeda-se na família geral.
  ⇒ **A F4.6c deixou de estar bloqueada; ela passou a CONTER uma fatia**: portar os eixos para o
  cartão antes de apagar os 24 ficheiros. *Um porte que apaga uma feature não é um porte.*

**O que a F4.6a/b mediram e o plano não dizia:**
- ⛔⛔ **Saltar os documentos possuídos era a resposta certa e METADE do trabalho.** Uma peça
  vetorial saltada pela cópia profunda não fica *«sem o vínculo»*: fica **sem geometria nenhuma** —
  uma linha na Hierarquia que não desenha um pixel. ⇒ `instance_docs::clone_owned_documents` clona
  o `VecPath` e aponta a cópia para o clone; o par `path ⟺ entidade` entra **junto**, senão o
  `vec_entities::sync` cunha uma segunda entidade para o clone.
- ⭐ **E isso cura um defeito IRMÃO, anterior às instâncias:** duplicar um GRUPO com formas
  vetoriais dentro devolvia as peças sem geometria (uma forma SOZINHA já era roteada para a porta
  do documento; um grupo caía na cópia profunda).
- ⚠️ **A propagação de um documento é por CONTEÚDO, e não por bytes de componente** — e isto não é
  uma excepção, é a definição: o `VecPathRef` de uma instância aponta para o path **dela**, então os
  bytes diferem para sempre, de propósito (é a família da junta). O id da instância **nunca** se
  mexe; o que se escreve é o conteúdo do mestre dentro do path dela. ⇒ o *Apply* também tem de ser
  por conteúdo, senão o mestre passa a apontar para o path da cópia.
- ⚠️ **Os outros três documentos possuídos (`PaintedDoc` · `BakedForm` · `FlipObjectRef`) continuam
  DROPADOS, e agora com NOME:** cada cópia devolve um relatório do que deixou cair e o chamador
  põe uma linha no log — *um importador que ignora em silêncio é pior que um que recusa*. O censo
  de dois lados faz um bridge novo reprovar em vez de nascer mudo.
- ⚠️ **Um gate meu SOBREVIVEU à mutação:** o do *«clone sem deslocamento»* comparava `subpaths`, e
  `translate_path` mexe nos **vértices**. *Comparar a estrutura não é comparar a geometria.*
- ⛔⛔ **E a F4.6b REPROVOU no smoke** (*«ao mudo o path, as instâncias não mudaram»*) com os gates
  todos verdes: eles provam que a **porta** faz a coisa certa, não que a cena que o artista monta
  chega àquela porta nesse estado. Os quatro suspeitos, por custo de medição, estão no
  [handoff §14](handoffs/HANDOFF_INTEGRACAO_line_components_F4_2026-08-26.md) — o primeiro é que a
  **receita vetorial ainda desenha** (o `off_canvas` gateia o extract de *sprites*; um `VecPath` sai
  pelo renderer vetorial), então mestre e cópia ficam sobrepostos e a edição pode estar a cair na
  cópia. ⛔ E falta o instrumento: **não há cena de smoke com receita vetorial** — o
  `PH2D_INSTANCE_SMOKE=1` monta um ragdoll de sprites.

**O que a F4.7 mediu e o plano não dizia (2026-08-27):**
- ⚠️ **A lane não cabe toda no `physics_ecs_c9`.** Aquele binário vive em `ph2d-physics-ecs`, que
  não vê a shell — e o *ponto fixo sob física* precisa do `sync_instances`, que é da shell. ⇒ a
  fatia é **duas**: a lane de determinismo (cross-OS, no binário) e o ponto fixo (gate de shell,
  `the_sync_is_a_fixed_point_while_the_solver_runs`, com o solver a correr 120 tiques).
- ⭐⭐ **O `body_count` que o binário imprime NUNCA foi afirmado**, e a comparação do CI é entre os
  três SO. ⇒ uma receita que entrasse no solver mudaria o hash **igualmente nas três máquinas** e o
  `sort -u | wc -l` continuava verde. *A comparação entre máquinas não vê um defeito que as três
  máquinas cometem.* A lane traz gate próprio (`the_recipe_stays_out_of_the_solver_and_the_copies_swing`),
  e a mutação que o mata é a LEI (`NotAMaster = ()` na ponte), não o andaime.
- ⚠️ **A ordem do `assign_master_pieces` na lane é DUAS chamadas**, e a 1.ª mutação que tentei
  sobreviveu por causa disso: a 2.ª re-marcava. *Uma mutação neutralizada a jusante não mede nada.*
- ⚠️ O binário **não** chama `assign_master_pieces` — quem o faz no produto é o
  `render_loop::physics_bridge::dispatch`, na shell. A lane chama-o ela própria, e o comentário diz
  porquê.

**O que a F4.5 mediu e o plano não dizia:**
- ⭐ **O *Redefinir* da tabela do doc 04 já existia** — é o *Revert to Master* que a F4.4 entregou.
  A F4.5 fecha os outros três: **Make Component** · **Instantiate** · **Detach from Master** ·
  **Apply to Master**, os quatro no menu de botão-direito da Hierarquia.
- ⚠️⚠️ **O *Criar componente* esconde a RECEITA, e isso forçou uma linha nova na lei do sync.**
  Sem esconder, o artista faz o gesto e vê **dois objetos empilhados** — um que cai e outro que não
  —, que se lê como defeito (o Unity põe o prefab *asset* fora da cena pela mesma razão). ⇒ a
  `Visibility` entrou no `ROOT_IS_ITS_OWN`: sem essa metade o `hidden` da receita **propagava** e
  toda instância nascia invisível — o gesto apagaria da tela o objeto que o artista acabou de
  transformar em componente. ⛔ É da RAIZ e não do tipo: esconder uma PEÇA dentro da receita é
  autoria, e propaga.
- ⚠️ **O *Destacar* solta a instância INTEIRA**, mesmo clicando numa peça. Uma instância com metade
  das peças ligadas não é nada que se saiba nomear — o sync propagaria a metade que ficou, e o
  artista veria um objeto que obedece pela metade. *O Unity também não tem meia-instância.*
- ⚠️ **A ordem do *Aplicar* é escrever no mestre e SÓ ENTÃO limpar a chave.** Ao contrário, o passe
  que corre no meio veria a instância sem excepção e diferente da receita, e achataria a edição que
  o gesto existe para promover.
- ⚠️ **Uma linha minha estava MORTA e só a mutação o disse:** o verbo reescrevia o `Transform` da
  receita na instância *«porque a pose é `InstanceLocal` e o sync nunca a traria»* — verdade sobre o
  sync e **irrelevante**, porque a cópia profunda leva o `Transform` verbatim: a instância **nasce**
  no lugar. ⛔ A `Visibility` é o caso contrário e por isso fica (a cópia é feita **depois** de a
  receita ser escondida). *Duas linhas vizinhas, uma paga e outra não — e só a mutação as separa.*
- ⛔ **`Make Component` recusa uma subárvore DENTRO de uma instância** — fazer dela receita partiria
  o elo da cópia que a contém. É a fronteira da F5, nomeada em vez de descoberta.

- ⚠️ **A recusa de ciclo é no GESTO e devolve uma RAZÃO** (`Refusal::NotAMaster` ·
  `WouldNestInItself`), não um `None`: *duas recusas que devolvem o mesmo `None` produzem o mesmo
  aviso inútil*. ⛔ E não é um tecto de profundidade — um limite numérico transformaria um erro de
  autoria numa contagem.

**⭐⭐⭐ O que o SMOKE do Enio devolveu (2026-08-26) — três reports, e o plano não previa nenhum**
(mecanismo, gates e mutações no [handoff §9](handoffs/HANDOFF_INTEGRACAO_line_components_F4_2026-08-26.md)):

- ⚠️⚠️ **Um objeto VAZIO não era agarrável por gesto nenhum** — `build_view` respondia `None` para
  toda entidade sem geometria própria, e o objeto que a F3 aprendeu a criar era o único do app sem
  gizmo. Hoje: caixa = **união dos filhos visíveis** (no espaço LOCAL do pai, pelos QUATRO cantos de
  cada um), ou o **marcador do vazio** com meia-extensão derivada do `HANDLE_SIZE_PX`. ⛔ Junta,
  roldana e peça de modelagem 3D ficam de fora — elas já têm alças, e uma caixa por cima engole o
  clique nelas.
- ⚠️ **O *Revert to Master* teletransportava a peça.** Decisão do Enio: ele devolve tudo **menos a
  pose**. A pose continua a ser override (senão o passe seguinte reescrevia por cima do arrasto) —
  é a lei do `ROOT_IS_ITS_OWN` descida um nível.
- ⚠️⚠️ **Pintar uma cópia não chegava às irmãs — e isso mudou o modelo.** Os PIXELS são um **asset**
  e sobem até à receita; o `tint`, a pose e a máscara continuam a ser da cópia. Duas razões: em todo
  motor 2D pintar a textura muda quem a usa, e a receita está **escondida**, pelo que pintá-la não
  era alcançável por gesto nenhum. ⛔ Fronteira nomeada: *Detach from Master* primeiro, para pintar
  uma cópia sozinha. ⚠️ Não vira override, por construção — o ponto fixo do sync fica intacto.

**⭐⭐ E a 2.ª volta do mesmo smoke devolveu um defeito DESTA fase que o gate não via**
([handoff §10](handoffs/HANDOFF_INTEGRACAO_line_components_F4_2026-08-26.md)):

- ⛔⛔ **`Visibility` é per-entidade neste motor e NÃO desce aos descendentes** (o `sim_extract`
  di-lo pelo nome). ⇒ o *Criar componente* da F4.5, que escondia só a RAIZ do mestre, **nunca
  escondeu uma receita que fosse um grupo** — as peças continuavam a desenhar, e o artista via os
  dois objetos empilhados que a nota dizia ter evitado. ⚠️ **O gate era verde porque media a MARCA
  (`Visibility` na raiz) em vez do FIM (o que se desenha).** A cura é o extract não desenhar quem é
  `MasterPiece` — marca **derivada**, logo incapaz de discordar da árvore —, e o gesto deixou de
  tocar em visibilidade nenhuma.
- ⚠️ **O anel é o CORPO de um objeto sem pixels, não uma marca de seleção**: ele vale para todo
  objeto vazio da cena, com filhos ou sem, e só some por não estar na cena (olho fechado · peça de
  receita · o modo de jogo, quando existir).
- ⚠️ **E ele PEGA** — 4.ª fonte da porta única de pick, por último na lista para o contêiner não
  roubar o clique dos filhos.
- ⛔ **RECUSA DE PRODUTO (3.ª volta):** a caixa de um grupo **não** é a união dos filhos. Foi
  construída (a lei do container do envelope generalizada) e rejeitada pelo Enio — *«o objeto vazio
  deve permanecer com seu gizmo original»*: a moldura mudava sozinha sempre que um filho se mexia. A
  árvore vive em `828bc88f4`. ⚠️ Não custa a função: o que move o conjunto é o gizmo escrever o
  `Transform` do PAI, não o tamanho da moldura.
- ⚠️ E o anel **deixou de esmaecer** fora da seleção (*«quase invisível»*): a seleção já é dita pela
  caixa e pelas oito alças, e meio tom no único canal de um traço de 1,5 px só apaga o corpo.
- ⭐⭐ **E o PRIMEIRO CLIQUE passou a ser de quem já está selecionado** (4.ª volta): um filho desenha
  por cima do pai, então arrastar um grupo selecionado escolhia um filho. ⚠️ Não revoga a lei do
  contêiner — ela diz a ORDEM dos candidatos, esta diz por onde o ciclo COMEÇA. ⛔ Nunca com
  modificador (o `Shift`+clique alternaria o pai). ⚠️ E o ciclo passou a estar atado à seleção,
  senão escolher o pai na Hierarquia e clicar no mesmo ponto continuava o ciclo antigo.

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

**O que a F5.1 mediu e o plano não dizia (2026-08-27):**
- ⭐⭐ **O aninhamento JÁ propagava** — medido por sonda antes de escrever uma linha: `B → instância
  de B dentro de A → instância de A na cena` leva uma edição de B até à cena **num passe**, e o
  passe assenta. A ordem topológica sai de graça porque `live_instances` ordena por `StableId` e a
  ordem de criação coincide com a de dependência. ⚠️ *Coincide* — não é derivada; a ordem por
  dependência continua por escrever, e o preço de não a ter é **N passes em vez de um**, não um
  resultado errado.
- ⛔⛔ **O que faltava era outra coisa, e a sonda deu-a de graça:** `a_inst tem 0 filho(s) depois do
  passe`. A tabela do §2.6 promete *«adicionar peça → materializa em todas»* e **nada o fazia** —
  o passe de valores percorre PARES, e uma peça que só existe do lado do mestre não forma par
  nenhum. Para o artista: *«acrescentei uma peça ao componente e as cópias não mudaram»*, que é a
  quarta vez que esta linha ouve essa frase por um mecanismo diferente.
- ⇒ `instance_structure::reconcile`, **antes** do passe de valores (é isso que dá a promessa de UM
  quadro: a peça materializada forma par já a seguir e recebe os bytes no mesmo passe). As duas
  metades juntas — acrescentar sem remover deixa na cena um objeto que o artista apagou da
  biblioteca —, com a fronteira do que o ARTISTA pendurou na cópia (sem elo ⇒ não é sobra) e a da
  instância órfã (mestre inteiro ausente ⇒ lei antiga, intocada).
- ⚠️ **O gate `only_the_instantiate_door_calls_the_deep_copy` apanhou-me a escrever uma SEGUNDA
  montagem** dentro do passe estrutural. Ele estava certo: a operação mudou-se para a porta
  (`instantiate::materialise_piece`) — *uma cópia profunda tem uma porta*.
- ⚠️ **E uma mutação minha SOBREVIVEU por fixtura plana:** *«a peça nova aterra na raiz em vez de no
  pai certo»* passa quando a peça nova é filha da raiz, porque os dois ramos dão o mesmo. O gate
  que a mata tem uma peça NETA. *Uma fixtura de um nível não pode medir de que nível a peça é.*

---

**O que a F5.3 mediu e o plano não dizia (2026-08-27):**
- ⛔⛔ **A F5.1 abriu o buraco que a F5.3 fecha**, e a sonda mostrou-o inteiro:
  ```text
  depois da excepcao:                 overrides=1
  depois de apagar a peca do mestre:  overrides=1  pecas na copia=[]
  depois do undo no mestre:  tint da copia = [1,1,1,1]   <- a excepcao era [0.9,…]
  ```
  Antes da F5.1 ninguém despawnava a peça, então a excepção vivia no componente dela e o
  re-encontro era automático. Com a peça a morrer ficava **a chave sem o valor**: a cópia perdia a
  excepção **e ficava surda à receita para sempre** (o passe salta o que a instância possui).
- ⭐⭐ **E isto RE-ABRE, com a premissa mudada, a refutação da F4.4** (*«guardar bytes cria duas
  fontes para o mesmo número»*). Ela valia porque *«a instância É uma entidade real»* — e a F5.1
  tornou a peça **destruível**. Numa peça órfã não há segunda fonte: há a **única**. ⇒
  `ObjectInstance.orphans: BTreeMap<OverrideKey, Vec<u8>>`, escrito **só** quando o alvo morre.
  *Quem move o número que tornava algo inalcançável tem de reconferir a nota.*
- ⭐ **«Volta a pegar» é verdade por causa do `StableId`** — a chave é a mesma depois do respawn do
  undo, e é a propriedade que o id compra sobre um caminho de nomes (a §0 do endereçamento).
- ⛔ **Nunca se apagam sozinhos** (a lei do *«unused overrides»* do Unity): sair por causa de um
  `Delete` no mestre é perder trabalho do artista em silêncio.
- ⚠️ **`PROJECT_SCHEMA` 99 → 100**, sem degrau de migração (decisão do Enio, 26/08). ⚠️ A **tripla**
  do `project_schema_tests` **não vê este degrau** — os bytes mudaram *dentro* de um `ComponentBlob`,
  que para ela é opaco. É o mesmo cego do 98→99, e está escrito lá porque é a primeira coisa que a
  próxima pessoa vai olhar.
- ⏳ **A superfície dos órfãos existe PELA METADE** — e a nota que aqui esteve dizia *«a superfície
  não existe»*, o que ficou falso no mesmo dia (conferido contra o código em 2026-08-30, em
  [`sections/instance.rs`](../../crates/ph2d-panel-inspector/src/sections/instance.rs)): o cartão
  mostra a **contagem** e o gesto (*Clear N unused override(s)*), e **não mostra QUAIS**. ⚠️ O
  critério 3 pede que o override *«apareça na secção»*, e uma contagem responde *«há três»* à
  pergunta *«quais três?»*. ⛔ Limpar sem ver o que se limpa é o gesto destrutivo mais barato deste
  painel. É a mesma família do item *«nada na tela mostra que campo está overridado»*, que **fechou**
  — os overridados aparecem, os órfãos ainda não.

---

**O que a SECÇÃO da F5.3 mediu e o plano não dizia (2026-08-27):**
- ⭐ **Ela fecha DOIS abertos com uma superfície:** o critério 3 da F5 (a lista de *Overrides sem
  alvo*) e a linha que este módulo carregava desde 26/08 — *«nada na tela MOSTRA que campo está
  overridado»*. O modelo existia desde a F4.4 e era **inteiramente invisível**: lia-se pelo que ele
  IMPEDE. *Um estado que só se lê pelo que ele impede não é um estado que o artista possa gerir.*
- ⚠️ **Uma seção do Inspector custa SEIS sítios** (modelo em `ph2d-editor-core` · ids · construtor
  na shell · publish em `snapshots` · `LiveSnapshots` + painter · dreno da acção), e a **7.ª** é a
  que morde: o `populate.rs`. O gate `hit_indexed_ids_are_registered` apanhou o botão **morto sob o
  dedo** — pintado, a acender no hover, clicável a nada. É a **terceira** vez que esta casa paga a
  mesma costura (o `+` da F3, os chips da booleana do Vector, agora este).
- ⚠️ **A catraca de LOC cobrou os dois painéis, e as duas curas foram estruturais**: a escada de
  cliques de um id virou **tabela** (`SINGLE_ID_CLICKS`, `apply_event_impl` 276 → **273**) e a
  MOLDURA do corpo saiu inteira para o ficheiro irmão `paint_body.rs` (`open_body` + `close_body`
  simétrico; `paint_inspector` 289 → **284**). ⛔ Ela não podia ir para o `paint_frame.rs`, que
  estava a 600 e recebeu-a a 636 — *curar um tecto estourando o outro não é curar*.
- ⭐ E o `paint.rs` virou **orquestrador puro** — deixou de nomear um primitivo de a11y **porque
  deixou de pintar um pixel**, o que exigiu declará-lo no `PANEL_A11Y_DELEGATE_OK`. *A ausência ali
  é a consequência de o ficheiro ter mudado de trabalho.*
- ⚠️ **Duas mutações minhas SOBREVIVERAM, e o gate estava certo**: o construtor tem DUAS guardas (o
  elo e a raiz) e um objeto solto falha as duas, então cada mutação era neutralizada pela outra. *A
  mutação honesta tem de tirar as duas juntas* — e essa mata.

---

---

**O que as VARIANTES mediram e o plano não dizia (2026-08-27):**
- ⭐⭐⭐ **A cadeia derivada JÁ propagava — medida por sonda antes de uma linha de código.** Uma
  variante é um `MasterRoot` que também é `InstanceOf`, e o `live_instances` já procurava *toda*
  entidade cujo elo aponta para um mestre vivo. Editar a base alcança a variante **e as instâncias
  dela num passe** (2 escritas). ⇒ o mecanismo custou **zero**; o que faltava era o **gesto** e o
  **re-key**. *A peça que falta pode já estar construída.*
- ⭐⭐ **O mapa de re-key já vive no mundo: são os próprios elos.** As peças da variante dizem de que
  peça da base nasceram, então `base → variante` lê-se invertendo `InstanceOf`. A pesquisa chamava a
  isto *«a operação que nenhum outro sistema consegue, porque nenhum tem chave mestre-relativa com
  caminho»* — aqui é mais barato do que ela previa, e a razão é a tese do ADR-0164: **a instância é
  uma entidade real**. ⛔ Sem nomes, sem caminhos, sem heurística.
- ⭐ **A troca é um RE-KEY e mais nada.** Ela muda o elo da raiz, os elos das peças e as chaves de
  override, e **para**: materializar, apagar e sepultar já é o passe estrutural da F5.1 + F5.3. É
  isso que a torna **reversível de graça** — a peça que o alvo não tem é sepultada e **exumada** na
  volta.
- ⚠️⚠️ **Duas leis que só o vermelho deu:**
  1. **A troca tem de ESQUECER o eco** das peças do mestre novo. Senão a diferença contra o mestre
     NOVO lê-se como *«a instância mexeu-se»* e a cópia congela com o valor do **velho**. É o mesmo
     mecanismo do *Revert*, que já o tinha escrito. ⚠️ Colateral nomeada: o eco é do MESTRE, logo
     alcança as irmãs — e o único caso que perde é uma edição feita **no mesmo quadro, antes do
     passe**, que são dois gestos num quadro em duas cópias.
  2. **Uma chave de override SEM imagem fica como está.** Apagá-la perdia a excepção **antes** de o
     `entomb` da F5.3 lhe serializar os bytes. *A troca não tem de saber o que é um byte: ela deixa a
     chave onde o sepultador a procura.*
- ⛔ **Uma variante NÃO pode perder uma peça da base** — medido, e é a cerca que torna o mapa total
  numa direcção: o passe estrutural põe a peça de volta. É a regra do Unity (*uma Prefab Variant não
  apaga um objeto herdado*) dita por outro caminho, e a 1.ª versão de um gate escolheu essa direcção
  impossível.
- ⚠️ **Uma fixtura minha media um mundo que o app nunca produz:** ela deitava fora o **eco** entre
  passes, e um eco novo cai na regra do 1.º encontro (*o mestre ganha*), o que apagava a excepção da
  variante. *O eco sai da fixtura com o mundo* — custou dois vermelhos.
- ⭐⭐⭐ **ACHADO REPO-WIDE: o `hit_indexed_ids_are_registered` é CEGO aos chips guiados por TABELA.**
  Ele só lê `.register(ids::LITERAL, …)`, e uma fileira passa a **variável do laço** — o doc dele
  di-lo, e ninguém tinha lido isso como um buraco. A mutação que apagava o `populate` dos chips
  **SOBREVIVEU**. ⚠️ E o controlo do filtro salvou a leitura: a 1.ª corrida deu *«ok»* sobre **zero**
  testes, porque o gate vive noutra crate. ⇒ gate novo `table_driven_chips_are_registered_too`, com
  **catraca**: 9 tabelas por registar em 4 painéis de outras linhas ficam **nomeadas e datadas**, e a
  lista só encolhe. ⭐ A metade *«só encolhe»* apanhou-me a mim: **duas** das 11 iniciais não
  descreviam nada.
- ⚠️ **Dois tectos de LOC, curados por ASSUNTO:** a **fila** saiu do `action_bus.rs` (54 linhas no
  FIM, onde nenhuma linha paralela escreve). ⛔ O corte óbvio — tirar o `EditorAction`, que é o que
  cresce — poria **toda** linha que acrescenta uma acção em conflito textual: *ao criar foundational,
  projecte-o para isolamento*.

---

## §F5-bis — ⭐⭐⭐ Os EIXOS DE PROPRIEDADE chegam ao cartão geral (2026-08-30)

**O que mudou para o artista:** uma família nomeada `Size=Small, State=Idle` deixa de ser uma
fileira plana de quatro nomes e passa a ser **duas fileiras, uma por pergunta** — e um chip muda
**exactamente um eixo**.

### Porque isto era a fatia que a F4.6c pedia

O mapeamento de 2026-08-30 mediu a paridade verbo a verbo, e o sistema geral já era **igual ou
superior** em cinco dos seis: `Create` faz mais (deixa uma cópia e cria variante), `Place` tem duas
leis de arte, `Detach` é trivial porque a instância já é geometria real, `Revert` é por chave e por
escopo, e o `Swap` geral tem **re-key determinístico**, sepulta órfãos e **RECUSA** um mestre não
aparentado — onde o vetorial aceita qualquer `VecComponentMain` sem parentesco nenhum.

⇒ **o que não existia era só isto**, e é o que impedia apagar a maquinaria duplicada.

### ⚠️ Duas modalidades, UMA representação

Quando os nomes não são combinações (ou discordam nas chaves), a família devolve **um** eixo
chamado `Variant` com os nomes crus — que é exactamente a fileira que o cartão já desenhava. ⇒ o
painel tem um caminho só, e a modalidade é um **facto dos dados**. ⛔ Duas representações para a
mesma fileira seriam dois sítios a discordar sobre o que está escolhido.

### ⚠️ Sem componente novo: a fonte é o `Name`

⛔ Um `VariantAxisSet` guardado seria a segunda resposta a *«que versões existem?»*, e divergiria no
dia em que alguém renomeasse um mestre. O gesto de autoria é **renomear na Hierarquia**, que já
existe. *A estrutura é o que a estrutura diz.*

### As leis, e o que cada uma impede

| Lei | O que ela impede |
|---|---|
| **Alcançável = difere de mim só neste eixo** | um chip que muda dois eixos de uma vez sem o artista pedir |
| **Igualdade de conjunto de chaves**, nunca interseção | perder um eixo em silêncio quando um membro declara menos |
| Um eixo com **um valor só** não se pinta | uma pergunta sem respostas ocupando uma linha do cartão |
| O excedente da tabela de ids é **contado e escrito** | um catálogo que some, e em que o artista deixa de confiar |
| Sem âncora (mestre vigente fora da família) ⇒ **nada** | uma fileira que mostra opções sem dizer onde se está |

### ⚠️ E a ESTRUTURA não se muda de sítio junto com a lei

O shell responde *«quem é da família»* (elos no mundo, `piece_map`) e o
[`variant_axes`](../../crates/ph2d-editor-core/src/screens/hero/variant_axes.rs) responde *«que
perguntas ela faz»* (só nomes). Separá-las é o que torna a lei testável **sem um mundo** — e o que a
deixa sobreviver ao apagar do sistema vetorial, de onde ela veio.

⚠️ **Consequência de autoria, medida e NÃO óbvia:** agrupar três mestres numa moldura **não** os
torna uma família aqui. No geral uma família nasce de *Make Prefab sobre uma instância* — o
parentesco lê-se dos **elos**, não da hierarquia. É a diferença contra o vetorial (irmãos do mesmo
pai), e ela não estava escrita em lado nenhum.

**Smoke:** `PH2D_BUILD_SMOKE=79` — quatro versões, duas fileiras, e o chip medido pelo ponteiro.

### ⭐⭐⭐ As CHAVES — o report do Enio que corrigiu o desenho (2026-08-30)

A 1.ª versão lia as propriedades do nome **inteiro**: `Size=Small, State=Idle` **era** o nome do
objecto. Report imediato, e certo em dois pontos:

1. *«criar nomes de objetos que não exprimem o que o objeto realmente é é muito estranho»*
2. *«os nomes ficam grandes demais e nem cabem direito na hierarquia»*

⛔⛔ **E o Figma não tem o 2.º problema por uma razão que o porte deixou de fora.** Lá aqueles nomes
existem — a sintaxe `Propriedade=Valor` é dele —, mas vivem **dentro de um contêiner** (o
*component set*), cujo nome é o comum: na lista de camadas vê-se `Casa` fechado, e os nomes
compridos só aparecem ao abrir. *Eu portei os nomes e não portei o contêiner.*

⇒ **a autoria passa a ser `Casa {Size=Small, State=Idle}`** (ideia do Enio), e a hierarquia mostra
**`Casa`**. As chaves fazem o trabalho do contêiner **sem o contêiner** — nenhuma estrutura nova,
nenhum gesto novo.

⭐ **E elas resolvem uma ambiguidade que o Figma tem:** um objecto legitimamente chamado `A=B` era
lido como um eixo. Sem chaves, não há propriedades.

⚠️ **O documento guarda o nome INTEIRO; só o pintor da linha deriva o curto.** É isso que mantém a
renomeação a editar as chaves (ela semeia do `Name` da entidade, não da linha) e a busca a
encontrar por valor de propriedade. ⛔ Nenhum dos 9 sítios que constroem uma linha da Hierarquia
mudou — *o que se guarda é a autoria; o que se mostra é uma leitura dela*.

### O 2.º report, com foto — **o selo, e o `(1)` que eu comia**

⛔⛔ **O sufixo de CÓPIA vem DEPOIS das chaves, e eu cortava a partir do `{`.** O app acrescenta
`(1)`, `(2)` … para desempatar nomes, e `Casa {Size=Small, State=Idle} (1)` desenhava-se **`Casa`**:
duas cópias ficavam com a linha idêntica, e o número que as distinguia era exactamente o que se
perdia. *Cortar por um delimitador de ABERTURA assume que ele é o fim da linha.* ⇒ tira-se o **vão**
das chaves e guardam-se os dois lados: `Casa (1)`.

⭐ **E o selo entrou**: `Casa (1) *²`. Ele conta **definições**, não versões — é o que o pedido diz
(*«sendo o número a quantidade de definições»*), e é a única coisa honesta que um número sozinho
pode prometer. ⚠️ Sem propriedades **não há selo**: um marcador permanentemente aceso é ruído que o
artista aprende a ignorar.

⚠️ **Eu tinha lido «vamos esquecer a tag» como o selo**, e era o `Tag=City` do exemplo. O report
seguinte corrigiu-me em uma linha — *e é o argumento para o smoke existir*.

⏳ **Fica aberto:** o selo diz **quantas**, não **quais** — quatro versões da mesma casa continuam a
ler-se `Casa *²` nas quatro linhas. O Figma resolve-o com o contêiner; a saída barata aqui é mostrar
**o que difere** (`Casa · Small, Idle`), desenhada e não implementada.

---

### A auditoria (2026-08-30) — **duas REGRESSÕES minhas, e duas leis perdidas no porte**

| # | Achado | Mecanismo | Cura |
|---|---|---|---|
| **A1** | ⛔⛔ uma família de DUAS versões «na diagonal» perdia a fileira **inteira** | com `Small/Idle` + `Big/Run` nenhuma é alcançável **num passo**, os dois eixos caem por ter um valor só ⇒ cartão **sem fileira**. *O artista tinha duas versões e nenhuma superfície para trocar* — e a fileira plana de ontem mostrava-as | o modo plano passa a ser a **REDE**: matriz esparsa demais para perguntas volta a ser uma lista. Pior de ler, e **alcançável** |
| **A2** | ⛔⛔ a truncagem podia deixar a fileira **sem vigente aceso** | o `current` é um `bool` por opção; truncar às cegas com o mestre corrente depois do teto apagava a resposta — *«mostra as opções e esconde a resposta»*, a frase que o pintor tem escrita | o vigente **sobrevive ao corte** |
| **A3** | o `beyond` contava opções e a frase dizia *«variant(s)»* | com eixos, o que se perde pode ser uma **pergunta inteira**, e o vigente era contado como *«mais uma versão»* sobre ele próprio | conta sem o vigente; a frase diz **«option(s)»** |
| **A4** | a cerca da âncora só existia em **metade** da lei | o modo plano devolvia a lista inteira com nenhum chip aceso | a âncora é a mesma pergunta nos dois modos |
| **A6** | HR-15: `"Variant"` era literal **fora do alcance do gate** | o `hr15_no_hardcoded_ui_strings` varre `widget/` e `ph2d-panel-*`; o literal estava em `screens/hero/` | a lei devolve o nome **vazio** e o painel nomeia-o — que é o que o original já fazia |
| **A7** | `MAX_INSTANCE_VARIANTS` ficou **órfão** | o `INSP_INSTANCE_VARIANT` que ele media foi apagado; sobrou uma constante sem um único leitor | apagada (⛔ órfão ≠ knob morto: a cura é apagar, não religar) |

⛔⛔ **E DUAS das seis são leis que se perderam NO PORTE, não buracos novos.** O `vec_variants.rs`
carrega exactamente duas correcções — `ax.selected.min(cap−1)` e o `unwrap_or(0)` do modo plano — e
**nenhuma veio**. Três das quatro funções foram portadas quase verbatim; as duas linhas que o
ficheiro original tinha aprendido a duras penas ficaram para trás, sem nota. ⚠️ *Um porte que copia
a forma e deixa as correcções é um porte que re-descobre os defeitos.*

⚠️ **E o gate da truncagem escolhia `me = 0`** — o primeiro valor, que nunca cai fora do teto: *uma
fixtura ordenada a favor da lei*, a mesma classe que esta fatia já tinha corrigido uma vez no
mesmo dia (`a_chip_changes_exactly_one_axis` passava pela ORDEM da família).

**Medido e ILIBADO:** a altura do cartão cabe (50→118 px com 4 eixos) · os chips **não** colapsam a
zero (seria preciso um painel de 91 px contra um mínimo de 220) · a porta `instance_axis_option`
custa **3 ns** · o gate `table_driven_chips_are_registered_too` **vê** a grelha de 32 ids · e o
smoke não tem passo que perturbe o que o seguinte mede.

⏳ **Aberto e nomeado:** o rótulo do eixo come **25 %** da fileira (8 chips ficam com 11,6–19,2 px
no painel mínimo) · o cartão de instância inteiro **não tem um único `seam_*`** — a prova
ponta-a-ponta é o smoke, que só imprime.

⏳ **O que a fatia NÃO fez:** apagar o `VecInstance`. Falta portar a **porta que PRODUZ** os
overrides vetoriais (a lista de peças com interruptor e swatch por peça), que a medição do plano
não contava — ver o §F4.

### ⭐⭐⭐ O 3.º report — **as chaves não tinham LEITOR no Inspector** (2026-08-31)

> *«quando mudo o conteúdo entre `{}` o inspector não muda»*

E estava certo. Medido no código antes de tocar em nada, as chaves tinham **dois** leitores em todo
o app:

| leitor | condição para existir |
|---|---|
| o selo `*²` da Hierarquia | nenhuma — basta o nome |
| a fileira de troca do cartão de instância | `InstanceOf` **e** uma família de **≥ 2** receitas |

⇒ num objecto **solto**, ou numa cópia de um **mestre único**, reescrever as chaves não mudava um
pixel. E o selo prometia, na linha ao lado, que alguém as lia. *Uma declaração sem leitor é
decoração* — a mesma lei que a memória já tinha por outro caminho
([`feedback_a_declaration_with_a_default_is_decoration…`](../../project-memory/feedback_a_declaration_with_a_default_is_decoration_until_something_reads_it.md)).

**A cura:** um **cartão de PROPRIEDADES** próprio
([`sections/properties.rs`](../../crates/ph2d-panel-inspector/src/sections/properties.rs) +
[`inspector_properties.rs`](../../shells/desktop/src/render_loop/inspector_properties.rs)), que
existe sempre que o nome declara alguma coisa **ou** a família pergunta alguma coisa.

⚠️ **As fileiras MUDARAM DE DONO — não foram duplicadas.** Deixá-las no cartão de instância e
acrescentar uma cópia aqui poria os **mesmos ids** registados por dois pintores no mesmo quadro: o
segundo `register` ganha, e o artista clicaria num chip para ver outro acender. ⇒ o
`InspectorInstanceInfo` **perdeu** `axes`/`variants_beyond`.

| de onde vem a fileira | o que é | como se pinta |
|---|---|---|
| `axes_for` (a família) | *«que outras versões existem?»* | chips que **trocam** |
| `declared_axes` (o nome) | *«o que este objecto DIZ que é»* | o valor, em **texto** |

⛔ **Um valor único é TEXTO, nunca um botão aceso** — pintá-lo como chip seria um controlo morto da
1.ª espécie da caça de 30/08: o clique existe, o artista carrega, e nada acontece.

⚠️ **A pergunta vence a declaração na MESMA chave** — nunca duas fileiras com o mesmo nome, senão a
de baixo está sempre desactualizada.

⚠️ **A declaração é do MESTRE, não do exemplar.** Uma propriedade é do componente: renomear a cópia
para `Bob` não pode apagar as propriedades dela. Gate com esse nome inteiro.

⭐ **E isto fecha o aberto que o 2.º report deixou** — *«o selo diz QUANTAS, não QUAIS»*. O selo
continua a dizer quantas; o Inspector passa a dizer **quais**, que é onde a pergunta pertence.

⭐ **Efeito colateral medido:** no modo plano o chip mostrava o nome **cru** — `Casa {Size=Small}` —
e com o nome comum igual em toda a família isso dava quatro chips a dizer `Casa` mais ruído. Hoje
ele mostra o **miolo** das chaves (`chip_label`), que é o que difere.

**Smoke:** `PH2D_BUILD_SMOKE=79`, agora com o objecto **solto** e o **gesto do report** (reescrever
as chaves) nos quadros 40–52.

### ⭐⭐⭐ O 4.º report — *«me mostre o fluxo inteiro de criar variações»* (2026-08-31)

> *«Parece que ainda não funciona.»*

⚠️ **A segunda frase é o achado, e ela não é sobre o cartão:** as chaves **DECLARAM** propriedades;
elas **não criam** uma família. Uma família nasce dos **ELOS**, e dois objectos irmãos com chaves no
nome não são variantes um do outro por mais parecidos que os nomes sejam. *O modelo que o artista
tinha na cabeça e o do produto divergiam, e nada na tela dizia qual era qual.*

⛔ **Não se responde a isto com uma explicação** — responde-se com o fluxo **medido**:
`PH2D_BUILD_SMOKE=80` corre os verbos pela MESMA porta que o menu drena
([`instance_verbs::drain`]) e imprime, a cada passo, a **voz do app** + o que a Hierarquia e o
cartão mostram.

E a 1.ª corrida achou **dois buracos, os dois reais**:

#### ⛔⛔ 1. O passo 3 recusava no caminho NORMAL — duas decisões deliberadas a desfazerem-se

| passo | gesto | antes | depois |
|---|---|---|---|
| 2 | *Make Prefab* | `Made a prefab — an instance took its place` | igual |
| **3** | *Instantiate* | ⛔ `mudou=false` · *«Not a prefab — pick the prefab row»* | ✅ `Instantiated` |

O *Make Prefab* **move a selecção para a cópia de propósito** (o doc do `select_out` explica-o: é o
que o artista vê e continua a editar, como o Figma e a Unity). O *Instantiate* pedia a **receita**.
⇒ *o app punha o artista numa linha e o verbo seguinte só funcionava noutra* — e as duas lêem-se
quase igual na Hierarquia (`Casa` e `Casa (1)`).

⚠️ **A cerca não caiu, mudou de sítio:** o `master_subject` resolve `cópia → receita` pela **mesma**
travessia que o *Apply*, o *Revert* e o *Detach* já faziam. ⛔ Uma linha que não é nem receita nem
cópia continua a recusar, com a mesma voz — e há gate com esse nome
(`instantiate_on_a_stranger_still_refuses`), porque *uma cura só se prova com o caso em que ela NÃO
pode agir*.

#### ⛔⛔ 2. Dois chips IDÊNTICOS — e o defeito era meu, do bloco anterior

No passo 4 a fileira plana dava `Size=Small` **e** `Size=Small`. Uma variante nasce com o nome da
base mais um sufixo (`Casa {Size=Small} Variant`), e o meu `chip_label` devolvia **só o miolo das
chaves** — deitando fora exactamente a parte que as distinguia.

*O modo plano existe para separar quem o modo de eixos não separou; um rótulo que colapsa duas irmãs
falha na única coisa que tem para fazer.* Hoje: `Size=Small` e `Size=Small Variant`.
⛔ Isto **não** garante injectividade — nada que olhe um nome de cada vez garante —, mas deixa de
**fabricar** colisões que o nome não tinha.

#### ✅ E o fluxo, medido de ponta a ponta, fecha

```
1. «Casa {Size=Small}» — objecto normal
2. Make Prefab            -> Made a prefab — an instance took its place
3. Instantiate            -> Instantiated
4. Make Prefab (na cópia) -> Made a variant — it still follows its base
                             cartao: Variant: Size=Small [Size=Small Variant]
5. renomear a variante    -> cartao: Size: Small [Big]      ⭐ a fileira por PROPRIEDADE
```

#### ⏳ ABERTO, medido e NÃO curado — é decisão do Enio

No passo 5 a Hierarquia mostra `["Casa (1) *¹", "Casa (2) *¹", "Casa *¹", "Casa *¹"]`: **duas linhas
lêem-se exactamente `Casa *¹`** — a receita base e a receita da variante. O selo diz **quantas**
propriedades, e as duas têm uma; ele não diz **quais**, então as duas receitas da mesma família são
indistinguíveis **na lista onde o artista tem de escolher qual renomear**.

⚠️ **Mostrar o VALOR na linha contradiz o desenho que o próprio Enio pediu** (*«só o nome Casa
fica aparecendo na hierarquia junto com um `*³`»*) — traz de volta o nome comprido que ele recusou.

### ⭐⭐⭐ O 5.º report (foto) — e ele achou o **furo do fluxo**, não um defeito do cartão

> *«Após renomear para big, Big não mudou o Botão que continua mostrando Small Variant. Card com
> Labels emboladas. Label dos botões emboladas»*

**Três defeitos numa foto, e o primeiro é o que interessa.**

#### ⛔⛔⛔ 1. Ele renomeou a CÓPIA — porque a receita era inalcançável

O cartão dizia `Instance of "Canvas{Size=Small} Variant"` e a caixa do nome dizia
`Canvas{Size=Big} (2)`. ⇒ ele renomeou **a cópia**, e o cartão continuou a mostrar `Small`
— **correctamente**: uma propriedade é do COMPONENTE, e a cópia herda-a.

⚠️ **Isto não é erro dele; é o preço do aberto que o bloco acima nomeou.** As duas receitas da
mesma família lêem-se `Casa *¹` e `Casa *¹`, e o passo do fluxo que faz uma variante valer alguma
coisa — *renomear a receita dela* — era um **palpite entre duas linhas iguais**.

⭐⭐ **A cura não alonga nome nenhum: é um SELO.** `HierarchyEntry::is_master` (campo novo, no fim
da struct) → o `hero_bridge` põe `badge: Some("PRF")`. ⛔ **O código já existia e ninguém o
produzia** — o `badge_tone` da Hierarquia conhece `PRF` (tom `Accent`) desde que existe, e o único
`badge: Some(…)` de todo o repo estava numa **fixtura de teste**. *Um canal declarado sem produtor
é decoração, e este esteve a decorar exactamente enquanto o artista não achava a linha.*

⭐ **E a segunda metade: o cartão passa a dizer DE QUEM são as propriedades** (`Properties of
"Canvas"`), porque na tela liam-se um nome a dizer `Big` e uma linha a dizer `Small` sem nada entre
os dois. ⛔ A alternativa — ler o nome da cópia — quebraria a lei que faz os chips funcionarem (a
família compara nomes de RECEITAS) e daria a cada cópia uma verdade própria.

#### ⛔⛔ 2. «Card com Labels emboladas» — uma altura CONTADA sobre um texto que quebra

`Instance of "Canvas{Size=Small} Variant"` não cabia na largura do cartão e desenhava **duas**
linhas; a altura era `line * rows`, com uma linha por frase ⇒ o resumo era pintado **por cima da
segunda**. *Uma altura contada em linhas mente sobre todo texto que pode quebrar.*

⇒ duas curas, e as duas são precisas: a frase passa a usar o **nome curto** (a mesma lei da
Hierarquia — as propriedades já vivem no cartão de baixo) **e** a altura das duas primeiras linhas
passa a ser **medida** (`text_system.layout(...).height()`), com o avanço do `y` a sair da mesma
medida. ⚠️ As linhas dos componentes overridados ficam na conta: são nomes de catálogo, curtos por
construção.

#### ⛔⛔ 3. «Label dos botões emboladas» — e a regra certa é do CONJUNTO

Eu mandava `Size=Small Variant` para um botão de meia largura. ⚠️ **As duas regras plausíveis
falham sozinhas, cada uma no caso da outra:**

| regra | falha quando |
|---|---|
| `display_name` (curto) | a família partilha o nome comum ⇒ chips todos iguais |
| `chip_label` (o miolo) | as irmãs partilham o miolo ⇒ chips iguais **e** compridos |

⇒ o `flat_axis` tenta o **curto** e só cai no longo quando ele de facto **colide**. *A informação
que decide — «estes dois são iguais?» — só existe onde os membros estão todos à mão; uma função que
olha um nome de cada vez não pode responder.*

### ⭐⭐⭐ O 6.º report — *«Faça as coisas direito!»* e a causa estava no GESTO, não no cartão

> *«Properties of "Nome do objeto na Hierarquia". Variant deveria ser Size. Nos botões deveríamos
> ter Small e Big. SE Há botões, as label sobre a propriedade podem ser retiradas.»*

Ele estava a olhar para `Variant: [Canvas] [Canvas Variant]` com um `Size  Small` de texto por
baixo — duas fileiras, nenhuma útil.

⛔⛔⛔ **A causa não é o cartão: é o `make_master`.** O *Make Prefab* sobre uma cópia dava à variante
o nome `<base> Variant`, então as **duas** receitas declaravam `{Size=Small}`. Com valores iguais o
eixo `Size` tem uma resposta só, cai (*«um eixo com um valor só não é uma pergunta»*), e a família
desce ao **modo plano** — que mostra NOMES. *O app criava uma versão nova e não lhe dava o que a
torna uma versão.*

⭐⭐ **A variante nasce com o valor SEGUINTE na 1.ª chave** (`variant_axes::variant_name`):
`Casa {Size=Small}` faz `Casa {Size=Small 2}` — a lei do Figma, que numera o valor ao duplicar uma
variante. ⚠️ **O sufixo `Variant` FICA para famílias sem chaves** (o idioma do Unity), e a lei é que
escolhe qual dos dois.

⚠️ **A unicidade compara COMBINAÇÕES, não nomes:** duas receitas com nomes diferentes e a mesma
combinação voltariam a colapsar o eixo, que é o defeito de origem.

E as três queixas caem de uma vez, sem uma linha de pintor:

| a queixa | porque cai |
|---|---|
| *«Variant deveria ser Size»* | com valores distintos o `multi_axis` produz o eixo, e o modo plano não é alcançado |
| *«nos botões, Small e Big»* | os chips do eixo são **valores**, nunca nomes de receita |
| *«se há botões, tire o texto»* | o `rows_for` já salta a chave declarada que **um eixo cobre** — a fileira de texto existia porque o eixo tinha caído |

Medido (`PH2D_BUILD_SMOKE=80`, passo 4): `Variant: Casa [Casa Variant]` → **`Size: Small [Small 2]`**;
e no passo 5, com a variante renomeada, `Size: Small [Big]`.

⭐ **E o título passa a nomear o objecto SELECIONADO**, como a Hierarquia o mostra. ⚠️ A versão
anterior punha ali o nome do COMPONENTE (a fonte das propriedades) para explicar por que o cartão
dizia `Small` sobre uma cópia renomeada para `Big`; ele pediu o outro. *Um título que nomeia uma
coisa que não está seleccionada faz o artista procurar onde ela está.*

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


⚠️⚠️ **A F6 e a F7 deixaram de ser duas fases — são UMA etapa, e ela parte noutro sítio (30/08).**
A regra de trabalho nova do Enio (*«cada etapa deve ao fim ter um smoke»*) **proíbe a F6 como fatia
própria**: o critério dela é headless, sem um pixel, e não existe frase *«faça X e veja Y»*. O corte
passa a ser pelo **gesto** — **A: achar e usar** · **B: arrastar** —, e a ordem é obrigatória (o
arrasto é o primeiro deste app a atravessar um painel, e não tem onde ser medido sem o painel).
⇒ plano vivo da etapa: [`07_plano_do_navegador_de_assets.md`](07_plano_do_navegador_de_assets.md),
com as 10 decisões de desenho e a lei da [`06`](06_pesquisa_o_navegador_como_interface.md) que
escolheu cada uma.

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
5. ⭐ **O Inspector mostra o que o objeto TEM, no modelo do Unity** (Enio, 2026-08-24; ADR-0166). O
   objeto nasce com **duas** seções e cresce por escolha. Alternativa nomeada se o smoke recusar:
   *"seções vazias recolhidas"* (todas presentes, dobradas, esmaecidas) — que é o meio-termo do
   Godot; ⛔ mas ele reintroduz exatamente o que o Enio pediu para tirar, então só entra se o smoke
   disser que a descoberta ficou pior.
6. ⭐ **Uma porta, não seis.** O `+` do Inspector é a rota; as cinco portas por-seção de hoje viram
   atalhos ou somem. Alternativa nomeada: manter a porta por-seção **também** — recusada por
   antecipação (duas respostas à mesma pergunta), reabrível se o smoke mostrar que o `+` não é achado.
7. ⭐ **O filtro por tipo de objeto é a VISTA, não uma cerca.** O inaplicável fica sob *Show all*,
   esmaecido e com a razão. Alternativa nomeada: **recusar** a anexação de um componente inaplicável
   (hard block). Fica de fora da v1 porque exige saber que *"não se aplica"* é sempre verdade — e a
   declaração `applies_to` é uma afirmação de produto, não uma prova. *Mostrar a razão é honesto com
   menos risco do que proibir com base numa tabela escrita à mão.*

## §10 — O que este plano NÃO faz (com o degrau nomeado)

| Fica para depois | Degrau |
|---|---|
| Clipe de timeline do mestre propagar às instâncias | `SequencePlayer` (TOP-20 #19 do [doc 00](00_levantamento_componentes.md)) |
| Config de física do mestre pegar mid-play | re-descrever corpo preservando velocidade (item na ponte) |
| Um ficheiro por asset (Git) | wave própria + ADR próprio; o índice da F6 nasce compatível |
| Thumbnails animados | depois dos estáticos; só pasta visível |
| Bibliotecas partilhadas/instaladas | lado a lado, nunca fundidas (ADR-0165 §5) |
