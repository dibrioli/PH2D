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
| F1 | `StableId` + `SiblingOrder` + snapshot v2 + **a 1ª migração** + corte da Sprite | ⬜ |
| F2 | O undo vira incremental (protocolo das 6 condições) | ⬜ |
| F3 | O Inspector passa a mostrar o que o objeto TEM · o `+` e a paleta · objeto vazio na raiz — **walking skeleton** | ⬜ |
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
| Componentes novos no registro | `StableId` · `SiblingOrder` (F1) · `SpriteCornerTint` · `SpriteSheet` · `SpriteRegion` (F1, corte da Sprite) · `MasterRoot` · `MasterPiece` · `InstanceOf` · `ObjectInstance` (F4) | `ph2d-ecs` **69 → 71 → 74 → 78**; espelhos render/script **70 → …** (+1 cada); boot **107 → 116** |
| `WorldSnapshot::VERSION` | 1 → **2** (F1) | `save.rs` |
| `PROJECT_SCHEMA` | ✅ **96** (F1, feito e integrado) — ⚠️ hoje o topo é **97** (a `line/Vector` entrou depois); o **próximo** degrau é o **98** | escada + tripla |
| Ids de widget novos | `INSP_ADD_COMPONENT` (F3, o `+` do cabeçalho do Inspector) | `ph2d-editor-core/src/ids/` + o gate `node_id_collisions` |
| **Superfície pública nova (F0, feita)** | `ComponentRegistry::register_default::<T>` · `ComponentTypeEntry::insert_default` · `ComponentTypeEntry::desc` | ⚠️ **`register_inner` é privado** — as duas portas públicas são `register` (sem default) e `register_default` |
| **Sítios de chamada convertidos (F0, feita)** | **109** `reg.register::<T>` → `register_default::<T>`, menos **27** revertidos (sem `Default`) = **82** convertidos | ⚠️ 5 arquivos: `ph2d-ecs/scene/registry.rs` (70, um deles num teste) · `-render` (1) · `-script` (1) · `-physics-ecs` (32) · `-field-ecs` (5). **É a maior superfície de colisão desta linha** — uma linha que acrescente um componente toca o mesmo arquivo |
| **Dependências novas (F0, feita)** | `ph2d-ecs` → `ph2d-component-desc` · `shells/desktop` → idem · `ph2d-panel-inspector` → idem | ⚠️ conta para o `machete` no `ship.sh` |
| **Arquivos de teste novos (F0, feita)** | `shells/desktop/tests/every_registered_component_is_described.rs` (5 censos) · `ph2d-panel-inspector/tests/the_ordering_labels_come_from_the_descriptor.rs` (2) | nomes novos, sem colisão |
| Componentes acrescentados na F0 | **nenhum** — a F0 não move contador | ✅ a **F1** moveu: `ph2d-ecs` 69 → **70** (só o `SiblingOrder`; o `StableId` ficou FORA do registo), espelhos 70 → **71** |
| Envs de smoke | `PH2D_INSTANCE_SMOKE=<n>` (F4+) · `PH2D_ASSET_BROWSER_SMOKE` (F7) | roteador de cenas próprio |
| Campo novo no `ProjectFile` | `stable_id_counter` (F1 — FORA do `ProjectState`, undo não rebobina) | conta no degrau do schema |
| Teto do ADR-0074 | +3 opcionais no corte da Sprite (o teto é 32) | `architecture_*` do Sprite |

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
5. **As duas famílias de `stable_name_id`, na MESMA wave** (o `name.rs:73-79` proíbe metade):
   timeline (`WireId` = `StableId`; `timeline_persist.rs:38`, `frame_solve.rs:139/218/251`,
   `persist.rs:28/54/88`) e física (`PhysicsJoint.body_a/b`, `PulleyWheel.rope/.body`,
   `bridge/joints.rs:152/315`, `bridge/rope.rs:130`, `joint_group.rs:139` + 12 sítios de inspector
   + fixtures do `physics_ecs_c9`). `stable_name_id` fica **deprecado com nota**, não removido.
6. Corte da Sprite (doc 04 C4): `per_corner_tint` → `SpriteCornerTint` · folha inline →
   `SpriteSheet` · região → `SpriteRegion` — **ausência = default benigno**, criar sprite continua
   1 gesto. ⚠️ Os três somam nos contadores (regra §0.1.2) e o cap do ADR-0074 (≤32 opcionais)
   recebe +3.
   ⭐ **E o corte ganhou uma SEGUNDA razão, que não é tamanho** (ADR-0166): enquanto o dado for
   *campo* de um componente que todo objeto-imagem tem, **não há como não o mostrar** no Inspector.
   Um campo só pode desaparecer da vista quando é um componente que pode estar ausente. ⇒ o critério
   de corte deixa de ser *"a Sprite é grande"* e passa a ser ***"isto pertence ao objeto-imagem BASE,
   ou é uma escolha que o artista faz?"***. Os três acima são escolhas; ⚠️ **releia os 20 campos com
   esta pergunta antes de cortar** — o resultado pode não ser exatamente três, e se não for, **corrija
   esta linha com a razão** (regra do plano vivo).

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
