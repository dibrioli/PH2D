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
| F4 | Núcleo de instância: Duplicar/Criar componente/Instanciar/sync/Destacar + física | 🟨 F4.1–F4.5 ✅ (+ os 3 reports do smoke de 26/08); faltam F4.6–F4.7 |
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
| F4.6 | O `VecInstance` subsumido (doc 04 §2.9) + degrau de schema | 🟨 **a** (documentos clonados) ✅ · **b** (geometria por conteúdo) ⛔ **gates verdes, SMOKE REPROVOU** — [handoff §14](handoffs/HANDOFF_INTEGRACAO_line_components_F4_2026-08-26.md) · **c** (matar o `InstanceLive` + migração) ⬜ |
| F4.7 | Lane do `physics_ecs_c9` com mestre+instância; ponto fixo sob física | ⬜ smoke-gate 3 |

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
