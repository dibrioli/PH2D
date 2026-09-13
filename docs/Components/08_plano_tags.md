# Plano — `Tags` (TOP-20 #9): o objecto PERTENCE a grupos, e um sinal fala com o grupo

> **Estado:** PLANO — nenhuma linha de produto escrita. `line/components`, 2026-09-13.
> Fila: [levantamento §7](00_levantamento_componentes.md) #9 · [síntese estrutura_ai §3](pesquisa/sintese_estrutura_ai.md)
> · [plano 05 §11.2](05_plano_de_implementacao.md). Molde das waves: o `Timer` (§12 do plano 05).
>
> ⚠️ **Tudo o que aqui tem número foi CORRIDO nesta máquina antes de ser escrito** — dois oráculos
> sem interface (Godot, Blender) e uma sonda Rust sobre o ECS. Os instrumentos estão versionados:
> [`ferramentas/godot_groups_probe.gd`](ferramentas/godot_groups_probe.gd) ·
> [`ferramentas/blender_collections_probe.py`](ferramentas/blender_collections_probe.py) ·
> [`crates/ph2d-ecs/tests/it/measure_tag_scan.rs`](../../crates/ph2d-ecs/tests/it/measure_tag_scan.rs).

---

## §0 — O que o artista passa a conseguir fazer

1. **Marcar** qualquer objecto com uma ou mais tags no Inspector (`enemy`, `enemy.flying`, `pickup`),
   com sugestões das que já existem na cena.
2. **Falar com o grupo inteiro por um sinal, sem script:** uma linha do *Signal Actions* passa a ter
   alvo **por tag** — *«quando `alarm` disparar, esconde todos os `enemy`»* — e `enemy` apanha também
   `enemy.flying` e `enemy.flying.boss`.
3. **Filtrar quem dispara uma armadilha:** um sensor com *On Hit* só grita quando quem entra tem a
   tag pedida (*«só o `player` abre a porta»*).
4. **Renomear uma tag em todo o lado** e **seleccionar todos os que a têm**, a partir da própria tag.

⛔ *Um componente sem consumidor lê-se, numa varredura, exactamente como uma feature pronta* (plano
05 §11.2, sobre o `SensorZone`). ⇒ as quatro frases acima são **uma** wave de produto: a tag nasce
com os três leitores, nunca antes deles.

---

## §1 — Estado da arte, e o que cada um TENTOU e ABANDONOU

### §1.1 — Medido nesta máquina (oráculos corridos sem interface; só a SAÍDA é usada)

**Godot 4.7.2 (MIT)** — `godot --headless --script docs/Components/ferramentas/godot_groups_probe.gd`,
load `≈3,6`:

| pergunta | saída |
|---|---|
| ordem de `get_nodes_in_group` (inserção `B, C, A`; árvore `A, C, B`) | `["A", "C", "B"]` — **ordem da ÁRVORE**, não da inserção |
| `add_to_group` repetido | contagem fica `3` — **idempotente** |
| `is_in_group("Enemy")` num nó de `enemy` | `false` — **sensível a maiúsculas** |
| um nó em `enemy.flying.boss` responde a `get_nodes_in_group("enemy")`? | **não** — o ponto é texto, **não há hierarquia** |
| nome com espaço (`"with space"`) | aceite |
| `call_group` | chama pela ordem da árvore, **inclui quem chama** se estiver no grupo |
| nó retirado da árvore | continua `is_in_group == true` e **sai** das consultas; volta ao re-entrar |
| `PackedScene.pack` + `instantiate` | só sobrevivem os grupos `persistent = true` |
| grupos do PROJECTO | o binário traz `Global Groups`, `global_group/`, `add_global_group`, `remove_global_group` e `GroupSettingsEditor::_confirm_rename` (`strings /usr/bin/godot`) |
| custo, 100 000 nós no grupo | `get_nodes_in_group`: **6,665 ms** a 1.ª, **0,957 ms** a 2.ª (índice cacheado por `SceneTree`) |

**Blender 5.2.1 (GPL — parede: saída apenas)** — `blender -b --factory-startup --python docs/Components/ferramentas/blender_collections_probe.py`:

| pergunta | saída |
|---|---|
| `bpy.data.groups` existe? | **`False`** — os *Groups* **foram retirados** (2.80), trocados pelas coleções |
| um objecto em duas coleções | `['enemy', 'flying']` — **pertença múltipla** |
| objecto só na coleção-filha aparece no pai? | `parent.objects = ['Goblin']`, `parent.all_objects = ['Bat', 'Goblin']` — **contenção HIERÁRQUICA** |
| renomear a coleção-filha | o objecto passa a dizer `['winged']` — a pertença é **referência**, não texto |
| criar uma segunda coleção `enemy` | vira **`enemy.001`** |
| tags de ASSET (`asset_data.tags`) com o mesmo nome duas vezes | `['Enemy', 'Enemy.001']` — **não deduplica: renomeia** |

### §1.2 — Da documentação pública (sem artefacto instalado; ⚠️ não medido)

| app | modelo | o que tentou e abandonou (ou manteve contra a corrente), e porquê |
|---|---|---|
| **Unreal** | *Gameplay Tags* hierárquicos (`State.Stunned`), registados num dicionário do projecto; `MatchesTag("A")` é verdade para `A.B`; comparação por `FName` (sem distinção de maiúsculas); *redirects* para renomear — [dossiê](pesquisa/dossie_unreal.md) l.233 | **As *Actor Tags* planas (`TArray<FName>`) continuam lá e perderam o papel** — os Gameplay Tags nasceram por cima delas por dois motivos: a gralha (uma string solta não é validada) e a falta de hierarquia (uma regra sobre `Damage` tinha de listar cada `Damage.*`). |
| **Unity** | **UMA** tag por GameObject (TagManager) + 32 *Layers* | **O contra-exemplo** ([síntese interacao_fluxo §8.3](pesquisa/sintese_interacao_fluxo.md) l.511): ficou com uma só tag e a comunidade resolve com componentes-marcador — que um artista não autora sem código. |
| **Godot** | *Groups* planos por nó + (medido) grupos globais do projecto com renomear | Os grupos por-nó são texto livre e sensíveis a maiúsculas (medido acima); o editor ganhou **um registo do projecto e um renomear** — que é a cura da gralha que o Unreal pagou antes. |
| **Blender** | coleções (medido) | **Groups e as 20 camadas fixas foram abandonados** e fundidos nas coleções (2.80): pertença múltipla + hierarquia + identidade por referência. |
| **Construct 3** | *Families* (ao nível do TIPO de objecto, carregam behaviors) + *instance tags* com filtro no *Solid* e na *Physics* (`Collision filter tags + mode`) — [dossiê](pesquisa/dossie_construct_gdevelop.md) l.124, l.290, l.344 | As *Families* são por **tipo**; o filtro por instância teve de chegar **à parte** (*instance tags*), porque «este Solid bloqueia o inimigo e não o jogador» é uma pergunta sobre a INSTÂNCIA. |
| **GameMaker** | objecto-pai como grupo — [dossiê](pesquisa/dossie_gamemaker_defold.md) l.228 | Mata as listas manuais, mas amarra o grupo à herança: um objecto tem **um** pai. |
| **Phaser** | `Group` não-exclusivo + *pooling* — [dossiê](pesquisa/dossie_cocos_phaser.md) l.259 | — |
| **Bevy** | componente-marcador (tipo zero) | O idioma certo de um ECS, e inalcançável para o artista sem código. |
| **After Effects** | *Labels*: **uma** cor por camada (16), *Select Label Group* | Uma etiqueta **exclusiva** — a mesma limitação do Unity, no mundo do motion. |
| **Illustrator** | sem tags de objecto: camadas, grupos e *Select ▸ Same* por atributo | Agrupa pelo que o objecto **parece**, não pelo que ele **é**. |
| **Rive** | não conheço um sistema de grupos consultáveis na cena; o runtime endereça por nome | ⚠️ afirmação **não verificada** — não há artefacto nesta máquina. |

### §1.3 — As cinco leis que saem daqui

1. **Pertença múltipla, sem hierarquia de objectos** — Blender, Godot, Unreal, Phaser, Construct. O
   Unity e o AE (uma só) são o contra-exemplo com número de anos.
2. **Hierarquia por SEGMENTO, não por texto** — Unreal e Blender (medido: `all_objects`). ⛔ E o
   prefixo de TEXTO não é pai: `enemy` **não** apanha `enemyx` (fixture da sonda).
3. **A identidade de uma tag é normalizada numa porta** — o Godot é sensível a maiúsculas (medido) e
   precisou de um registo do projecto para a gralha; o Unreal compara sem distinção.
4. **A consulta tem ordem DETERMINISTA** — o Godot ordena pela árvore (medido); aqui a árvore não é
   estável entre um `Ctrl+Z` e o seguinte (o undo respawna), ⇒ a ordem é a do `StableId`, a mesma
   que o `SignalActions::resolve` já usa.
5. **Renomear é um verbo de primeira classe** — Blender não precisa dele (a pertença é referência);
   Godot e Unreal, que guardam TEXTO, ganharam-no no editor. ⇒ quem guarda texto tem de ter o verbo.

---

## §2 — O desenho: UMA porta por pergunta

### §2.1 — O modelo

- **`ph2d_ecs::Tags(BTreeSet<TagPath>)`** — componente REGISTADO, módulo **irmão** `crates/ph2d-ecs/src/tags.rs`
  (append-only, isolamento B'). `BTreeSet` ⇒ bytes canónicos independentes da ordem de inserção (o
  undo regista por diff de bytes) e duplicado impossível por construção (o Godot dá o mesmo:
  idempotente; o Blender **não** — renomeia para `.001`).
- **`TagPath`** — `String` canónica, só construível por [`TagPath::parse`]: segmentos separados por
  `.`, cada um de letras/dígitos Unicode ou `_`, **em minúsculas** (D2), sem segmento vazio, sem
  espaço. ⭐ O `.` está livre: o sufixo de nome único desta casa é ` (n)`
  (`ph2d-unique-name`, `format!("{stem} ({n})")`), e não o `.001` do Blender.
- **CONFIG, nunca estado vivo** — a lei do `Timer` (§12.1): nada escreve `Tags` por conta própria.

### §2.2 — As portas

| pergunta | porta ÚNICA | quem chama | proibido |
|---|---|---|---|
| *isto é uma tag válida, e qual é a forma dela?* | `TagPath::parse(&str) -> Result<TagPath, TagError>` | o painel ao escrever · o `Deserialize` ao ler · o `rename_tag` | normalizar noutro sítio (duas regras discordam sobre `Enemy`) |
| *a tag `t` satisfaz a consulta `q`?* | `TagPath::matches(&self, q)` — `t == q` ou `t` começa por `q` **seguido de `.`** | `Tags::has` e mais ninguém | comparar strings à mão |
| *este objecto tem `q`?* | `Tags::has(&q)` | o filtro da física · o `tagged` | ler o `BTreeSet` fora do módulo |
| *quem tem `q`?* | `tags::tagged(world, &q) -> Vec<Entity>`, ordem do `StableId` | `SignalActions::resolve` · *Select Tagged* · (futuro: Spawner, percepção) | uma segunda varredura; um índice guardado |
| *que tags existem na cena?* | `tags::known_tags(world) -> BTreeSet<TagPath>` (inclui os pais implícitos) | as sugestões do painel | uma lista escrita à mão |
| *renomear `a` para `b` em todo o lado* | `tags::rename_tag(world, &a, &b) -> RenameReport` | o menu da tag | reescrever `Tags` sem reescrever os alvos e os filtros que a citam |
| *este sinal de colisão passa o filtro?* | `ph2d_physics_ecs::signal_passes(world, source, other) -> bool` | `PhysicsBridge::signal_events` (chegada **e** saída) | um filtro para a chegada e outro para a saída |
| *quem sofre esta acção?* | `signal_actions::targets_of(world, source, &action) -> Vec<Entity>` (substitui o `target_of`) | `resolve` | resolver o alvo no painel ou na shell |

⭐ **Sem índice, e é MEDIDO** (§6.1): a varredura custa `0,064 ms` a 100 000 objectos com 10 000
marcados e `1,47 ms` no pior caso (100 000 marcados, 66 667 acertos) — contra um quadro de `16,7`.
O Godot mantém um índice por árvore; aqui um índice seria **estado derivado a manter coerente depois
de todo restore**, a recusa que o `stable_id.rs` já escreve para o nome.

### §2.3 — Os três consumidores

1. **`SignalActions`, alvo por tag** — `SignalAction` ganha `target_by: SignalTarget { Named, Tagged }`
   **apendado no fim** (migração trivial, §3). `Named` é o de hoje, **byte a byte** (vazio = este
   objecto). `Tagged`: o `target` é uma consulta; vazio ou inválido = **ninguém** (silêncio, a lei do
   alvo que não existe). A ordem dos efeitos passa a ser *reactor (StableId) → linha escrita → alvo
   (StableId)*. ⭐ **A shell não muda**: ela já aplica um efeito por alvo
   (`render_loop::signal_actions::apply`), e um grupo são N efeitos. Inclui quem reage, se tiver a
   tag — o mesmo que o `call_group` do Godot (medido).
2. **Filtro de colisão** — `ph2d_physics_ecs::SignalTagFilter(String)`, componente **irmão** do
   `SignalOnHit` (registado, o cabeçalho do `signal.rs` diz porquê: apendar no `Collider` é bump).
   Vazio = sem filtro (o mundo de hoje, byte-idêntico). Consultado **dentro** do `signal_events`, que
   é quem tem o `other` — o `resolve` só recebe nomes. Uma consulta inválida **não passa ninguém**
   (falha fechada; o painel diz porquê).
3. **O editor** — *Rename Tag…* e *Select Tagged* no menu de contexto de uma tag (a selecção múltipla
   já existe: `HeroScreen::add_to_selection` / `extra_selection`).

### §2.4 — Fora desta wave, com o degrau nomeado

| fica de fora | degrau |
|---|---|
| Registo de tags do PROJECTO com descrição (o *Global Groups* do Godot) | D3 — pede campo novo no `ProjectFile`; as sugestões desta wave saem do mundo |
| *Families* com traits (tag que carrega componentes) | a síntese já o marca como etapa 2 |
| `Team` | TOP-20 fora do #9; pode ser uma tag, decide-se com a percepção |
| `CameraFollow` por tag (o `GameCamera` tem `target: String`) | mudar o layout do `CameraFollow` é outro bump; entra com o Spawner, que é quem cria alvos em runtime |
| Spawner / SightSense | #11 / P1 — consumidores futuros da mesma porta `tagged` |

### §2.5 — Decisões de PRODUTO (do Enio), com a recomendação

| # | pergunta | recomendação | porque |
|---|---|---|---|
| **D1** | Tags com hierarquia por ponto (`enemy.flying` pertence a `enemy`)? | **sim** | Unreal + Blender (medido); a síntese e a crítica já o pediam desde o dia 1; mudar depois é migrar todo projecto |
| **D2** | `Enemy` e `enemy` são a mesma tag (mostrada em minúsculas)? | **sim** | o Godot trata-as diferentes (medido) e precisou de um registo para a gralha; o Unreal não distingue |
| **D3** | Lista de tags do projecto com descrição, agora? | **não nesta wave** — sugestões das que já estão em uso + renomear em todo o lado | é campo novo no ficheiro do projecto, e sem ele nada fica inalcançável |
| **D4** | Uma tag posta numa receita (prefab) vai para todas as cópias? | **sim, e uma cópia pode ter a sua lista** (override da lista inteira) | é o comportamento de todo componente hoje; o override por-elemento de uma LISTA é a pergunta aberta da família (plano 05 §12.6) |

---

## §3 — Contrato congelado e schema

### §3.1 — Contratos congelados (§6 do `CLAUDE.md`): **nenhum é tocado**

Prova por grep sobre a árvore `4ecaddb5f` (os ficheiros que o desenho **não** abre):

```text
crates/ph2d-nodegraph/src/node.rs            ocorrências de tag|group: 0
crates/ph2d-editor-core/src/tool.rs          ocorrências de tag|group: 1  → «RadioGroup» num doc-comment (l.36), alheio
crates/ph2d-vector-doc · ph2d-vector-traits  ocorrências de Tags: 0
```

Os ficheiros que o desenho abre: `ph2d-ecs` (`tags.rs` novo, `signal_actions.rs`, `scene/registry.rs`),
`ph2d-physics-ecs` (`components/signal.rs`, `bridge/signals.rs`, `lib.rs`), `ph2d-component-desc`
(`catalog/core.rs`, `logic.rs`, `physics.rs`), `ph2d-panel-inspector`, `ph2d-editor-core`
(`action_bus.rs`, `ids/live_sections.rs`), `ph2d-app-components` e a shell (uma linha por fase).
Nenhum é superfície de §6.

### §3.2 — Os contadores, em DELTA (nunca o literal — `CLAUDE.md` §5.0)

| contador | hoje | delta | porquê |
|---|---:|---:|---|
| `ph2d-ecs` `reg.len()` (`registry_tests.rs:191`) | 85 | **+1** | `Tags` |
| espelhos `ph2d-render` / `ph2d-script` | 86 | **+1** | a mesma conta |
| `ph2d-physics-ecs` `reg.len()` (`lib.rs:198`) | 32 | **+1** | `SignalTagFilter` |
| `PROJECT_SCHEMA` | 128 | **+1** | três razões num degrau: dois componentes **novos** (a regra do degrau 122→123: um blob desconhecido recusa o load, e o número transforma isso numa frase de versão) e o **layout** do `SignalAction` (postcard é posicional) |
| `LIVE_SECTIONS` / `any_live_section` | 20 / 15 | **+1 / +1** | a secção *Tags* |
| `FLIP_SCHEMA` · `VEC_SCENE_SCHEMA` · `DOC_VERSION` · `FIELD_DOC_VERSION` | — | 0 | não tocados |

### §3.3 — A migração, e porque ela existe desta vez

⛔ **Um bump sem degrau recusaria todo projecto gravado com *Signal Actions*** desde 2026-09-09 — e a
decisão do Enio de 26/08 (*«não há projetos salvos»*) é **anterior** a esse componente. ⇒ degrau
`128 → 129` que **reescreve um blob**, pelo precedente exacto do v97→v98
(`shells/desktop/src/project_migrate_sprite.rs`): um `SignalActionV1` congelado `(on, target, verb, arg)`
lê os bytes antigos e escreve `target_by = Named`. ⭐ A lei do re-encode vive no `ph2d-ecs`
(`signal_actions::migrate_v1_blob`); a shell só a chama no degrau — a shell só desce.

---

## §4 — A UI: as QUATRO condições, independentes

| superfície | 1. o componente EXISTE | 2. PINTADO e REGISTADO | 3. o clique chega ao BARRAMENTO | 4. a SEQUÊNCIA leva a algum lugar |
|---|---|---|---|---|
| **Secção *Tags*** (logo abaixo da *Identity*; catálogo `C::Identity`, `O::ANY`, anexa-se pela paleta — ADR-0166) | `Tags` registado + descritor | chips `widget::tag::Tag` (removível) em fileiras que dobram + `widget::combobox::Combobox` para escrever, com a lista filtrada de `known_tags`; ids em `ph2d-panel-inspector/src/ids/inspector_tags.rs`, `populate_tags.rs` | `EditorAction::InspectorTagsEdit { entity, op: Add(String) \| Remove(String) }` → a lei aplica em `ph2d-app-components::tags_edit` | a tag escrita **aparece** em `tagged` (gate), e um *Signal Actions* por tag passa a atingir o objecto (smoke) |
| **Alvo do *Signal Actions*** | `SignalTarget` no `SignalAction` | segmentado `Name \| Tag` na linha aberta; com `Tag`, o campo de alvo vira `Combobox` de `known_tags` | `InspectorActionEdit` ganha o op `TargetBy` | `resolve` devolve **N** efeitos e a shell aplica-os (gate + smoke: 5 escondidos, o chamariz intacto) |
| **Filtro da armadilha** (na secção *Physics*, por baixo do *On Hit* / *On Leave* — `INSP_PHYS_SIGNAL*`) | `SignalTagFilter` registado | uma linha *Only for tag* com `Combobox` | op novo no evento da física | `signal_events` **decide** (§5.0: o leitor decide, ou entrega a quem descarta?) — o inimigo passa calado, o herói grita |
| **Menu da tag** (clique direito num chip) | `ContextMenuKind::TagChip { entity, tag }` | duas linhas: *Rename Tag…* · *Select Tagged* | `EditorAction::TagRename { from, to }` · `TagSelectAll { tag }` | `rename_tag` reescreve Tags + alvos + filtros num passo de undo; a selecção passa a ser `tagged(q)` |

⚠️ **O erro de escrita diz-se NA LINHA, nunca num toast** (`TagError` com a razão: *«spaces are not
allowed»*, *«empty segment»*) — a lei do L-System (`TextRow.problem`), que já provou que uma queixa tem
de chegar a PIXEL, com gate de glifo.

---

## §5 — Os gates, red-first, e a fixture que CONTÉM o fenómeno

### §5.1 — A fixture

`tags_fixture(world)` — sete objectos e um chamariz, construídos pelo **mecanismo** e não à mão:

| objecto | tags | o que ele prova |
|---|---|---|
| Goblin A, Goblin B | `enemy` | o caso exacto |
| Bat A, Bat B | `enemy.flying` | a hierarquia de um nível |
| Dragon | `enemy.flying.boss` | a hierarquia de dois níveis |
| **Statue** | `enemyx` | ⛔ **o prefixo de TEXTO que não pode casar** |
| Hero | `player` | o filtro da armadilha |

⚠️ E duas perturbações dentro da mesma fixture: **inserir um componente alheio no Goblin A** (muda a
ordem de arquétipo — a ordem da consulta tem de sobreviver) e **um restore do snapshot** (bits novos
— o alvo tem de continuar a resolver).

### §5.2 — Os gates, por wave (todos escritos ANTES da porta, vistos VERMELHOS contra um *stub*)

**W1 — a lei (`ph2d-ecs`):**
1. `a_parent_query_matches_its_descendants_and_not_a_text_prefix` — fixture inteira: `enemy` → 5,
   `enemy.flying` → 3, `enemy.flying.boss` → 1, `enemyx` → 1.
2. `parse_refuses_what_cannot_be_a_tag_and_says_why` — `""`, `" enemy"`, `"en emy"`, `".enemy"`,
   `"enemy."`, `"enemy..boss"`, cada um com o `TagError` certo.
3. `a_tag_is_normalised_at_the_one_door` — `Enemy.Flying` e `enemy.flying` dão os mesmos bytes.
4. `a_duplicate_tag_is_one_tag` — contra o oráculo (Godot `COUNT_AFTER_DUP 3`) e contra o Blender
   (`Enemy.001` — divergência declarada).
5. `tags_bytes_do_not_depend_on_insertion_order` — o undo regista por bytes.
6. `the_query_order_is_the_identity_not_the_archetype` — com a perturbação de arquétipo.
7. `a_malformed_tag_in_a_file_is_refused_loudly` — o `Deserialize` passa pela porta.
8. `only_the_door_reads_tags` — censo textual (sem comentários, sem strings) de
   `query::<…Tags…>` / `get::<Tags>` fora de `tags.rs`, com **piso de população** (§2.7 do HOWTO).
9. `the_hierarchy_is_the_one_blender_measures` — fixture do oráculo (`CHILD_ONLY … all_objects
   ['Bat', 'Goblin']`, com cabeçalho) contra `matches`.

**W2 — consumidores + schema:**
10. `a_signal_to_a_tag_reaches_every_member_and_the_statue_stays` — 5 efeitos, pela ordem declarada.
11. `an_empty_or_invalid_tag_target_reaches_nobody`.
12. `a_named_target_is_byte_identical_to_before` — os gates de hoje do `signal_actions_tests`, intocados.
13. `a_v128_signal_action_loads_as_a_named_target` — bytes congelados de um v128.
14. `a_filtered_trap_ignores_the_untagged_body_on_arrival_and_departure`.
15. `an_unfiltered_trap_is_byte_identical` + `an_invalid_filter_passes_nobody`.
16. `renaming_a_tag_rewrites_tags_targets_and_filters_in_one_step` — inclui a colisão (renomear para
    uma que já existe **funde**, e o relatório di-lo).
17. Contadores: `85→86`, `86→87` (×2), `32→33`, `PROJECT_SCHEMA` + a tripla.
18. `a_recipe_tag_reaches_every_copy` (D4) · a cópia profunda leva o `Tags` (não está no `DROPPED`).

**W3 — painel (seam, `ph2d-ui-testkit`, gesto REAL):**
19. escrever `enemy.flying` + `Enter` cria o chip e a acção chega ao barramento;
20. o `×` do chip remove;
21. `en emy` + `Enter` **não escreve nada** e a razão chega a **glifo**;
22. as sugestões vêm de `known_tags` (incluindo o pai implícito `enemy`);
23. o segmentado `Name | Tag` do *Signal Actions* e o *Only for tag* da física chegam ao barramento;
24. `architecture_panel_wiring_parity` e `hit_indexed_ids_are_registered` verdes com os ids novos.

**W4 — menu + smoke:**
25. *Rename Tag…* e *Select Tagged* pelo ponteiro (clique direito num chip).

Cada gate diz, no doc-comment, **a mutação que o sangra** — e as de W1/W2 são corridas (`/pd-mutacao`).

---

## §6 — O smoke, com os números MEDIDOS antes de escrito

### §6.1 — A sonda Rust (`measure_tag_scan`, `--release`, mediana de 25, **load 1,38**)

| objectos | com tags | acertos | consulta por tag | alvo por NOME, hoje |
|---:|---:|---:|---:|---:|
| 100 | 100 | 67 | 0,0018 ms | 0,0023 ms |
| 1 000 | 1 000 | 667 | 0,0060 ms | 0,0031 ms |
| 10 000 | 10 000 | 6 667 | 0,0614 ms | 0,0252 ms |
| 100 000 | 10 000 | 6 667 | **0,0638 ms** | 0,2884 ms |
| 100 000 | 100 000 | 66 667 | **1,4702 ms** | 0,2831 ms |

A 1.ª corrida, a load `5,74`, deu a mesma ordem de grandeza nas duas linhas que decidem (`0,0606` e
`1,6317` contra `0,0638` e `1,4702`) — logo a conclusão não é carga. ⇒ **a varredura cabe**: o pior caso é `8,8 %` de um quadro, e o custo mora nos ACERTOS (a
ordenação), não nos objectos. O teste irmão `the_probe_matches_the_hierarchy_and_not_the_text_prefix`
fixa a lei da sonda (`20` de `30`).

### §6.2 — A cena `PH2D_TAGS_SMOKE=1` (a construir na W4; família `components`, `max_level` contado)

1. Sobe com a fixture da §5.1 lado a lado, cada objecto com o nome e as tags escritos por baixo.
2. Um objecto vazio *Scene Brain* com um `Timer` `alarm` de **2 s** e um *Signal Actions*:
   *on `alarm` → Tag `enemy` → Hide*.
3. **O que tem de acontecer:** aos 2 s somem **5** objectos (Goblins, Bats, Dragon); ficam **Statue**
   e **Hero**. O terminal imprime `[tags] alarm: 5 escondidos · Statue intacta`.
4. Cena `=2`: uma armadilha (sensor, *On Hit* `trap`, *Only for tag* `player`); um Goblin atravessa-a
   primeiro (**nada**), depois o Hero (a porta abre). O terminal imprime `[tags] trap: 0 → 1`.
5. **Como saber que deu errado:** a Statue sumir (casou o prefixo de texto) · menos de 5 sumirem (a
   hierarquia não chegou) · a porta abrir com o Goblin (o filtro não decide).

---

## §7 — As waves, e a ordem

| wave | entrega | acaba em |
|---|---|---|
| **W1** | `tags.rs` (as portas `parse`/`matches`/`has`/`tagged`/`known_tags`), registo, catálogo | gates 1–9 verdes vistos vermelhos · headless |
| **W2** | `SignalTarget` + `targets_of` · `SignalTagFilter` + `signal_passes` · `rename_tag` · `PROJECT_SCHEMA` +1 com o degrau · contadores | gates 10–18 · headless |
| **W3** | secção *Tags* · alvo por tag · filtro da física | seam 19–24 + smoke do Enio |
| **W4** | menu da tag · `PH2D_TAGS_SMOKE=1..2` | gate 25 + smoke do Enio |

⚠️ **O custo na shell é uma linha por fase** (o dreno do barramento e a publicação do snapshot da
secção chamam `ph2d_app_components::tags_edit`/`tags_snapshot`) — a lei e a ponte vivem na família.
A catraca `the_shell_only_shrinks` tem folga depois da 5.ª rodada (191 016 contra 196 990).
