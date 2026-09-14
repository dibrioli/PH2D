# Plano — `Tags` (TOP-20 #9): o objecto PERTENCE a tags numa árvore, e um sinal fala com a tag

> **Estado:** PLANO APROVADO com as decisões do dono (2026-09-13, §2.5) — nenhuma linha de produto
> escrita ainda. `line/components`. Fila: [levantamento §7](00_levantamento_componentes.md) #9 ·
> [síntese estrutura_ai §3](pesquisa/sintese_estrutura_ai.md) · [plano 05 §11.2](05_plano_de_implementacao.md).
> Molde das waves: o `Timer` (§12 do plano 05).
>
> ⚠️ **Tudo o que aqui tem número foi CORRIDO nesta máquina antes de ser escrito** — dois oráculos
> sem interface e uma sonda Rust sobre o ECS, versionados:
> [`ferramentas/godot_groups_probe.gd`](ferramentas/godot_groups_probe.gd) ·
> [`ferramentas/blender_collections_probe.py`](ferramentas/blender_collections_probe.py) ·
> [`crates/ph2d-ecs/tests/it/measure_tag_scan.rs`](../../crates/ph2d-ecs/tests/it/measure_tag_scan.rs).
>
> ⛔⛔ **A 1.ª redacção deste plano (commit `352b84b4f`) propunha tags como TEXTO com hierarquia por
> ponto (`enemy.flying`).** O dono escolheu *«como o Blender»*, e o desenho mudou de natureza: a
> pertença passou a ser uma **identidade**, a hierarquia vive numa **árvore do projecto**, e o texto
> é só o nome que se mostra. As secções abaixo são a versão aprovada; a antiga fica no git.

---

## §0 — O que o artista passa a conseguir fazer

1. **Criar tags numa árvore do projecto** — `Enemy`, e dentro dela `Flying`, e dentro `Boss` —,
   renomeá-las e **arrastá-las para dentro de outra** sem que nenhum objecto perca a tag.
2. **Marcar** qualquer objecto com uma ou mais tags no Inspector, escrevendo com sugestões —
   `inimigo`, `Inimigo` e `inímigo` são **a mesma** tag.
3. **Falar com a tag por um sinal, sem script:** uma linha do *Signal Actions* passa a ter alvo **por
   tag** — *«quando `alarm` disparar, esconde tudo o que é `Enemy`»* — e isso apanha também `Flying`
   e `Boss`.
4. **Filtrar quem dispara uma armadilha:** um sensor com *On Hit* só grita quando quem entra pertence
   à tag pedida (*«só `Player` abre a porta»*).
5. **Seleccionar tudo o que pertence a uma tag**, e ver quantos objectos cada tag tem.

⛔ *Um componente sem consumidor lê-se, numa varredura, exactamente como uma feature pronta* (plano 05
§11.2). ⇒ a tag nasce **com** os seus leitores (o sinal, o filtro, a selecção), nunca antes deles.

---

## §1 — Estado da arte, e o que cada um TENTOU e ABANDONOU

### §1.1 — Medido nesta máquina (oráculos corridos sem interface; só a SAÍDA é usada)

**Godot 4.7.2 (MIT)** — `godot --headless --script docs/Components/ferramentas/godot_groups_probe.gd`, load `≈3,6`:

| pergunta | saída |
|---|---|
| ordem de `get_nodes_in_group` (inserção `B, C, A`; árvore `A, C, B`) | `["A", "C", "B"]` — **ordem da ÁRVORE** |
| `add_to_group` repetido | contagem fica `3` — **idempotente** |
| `is_in_group("Enemy")` num nó de `enemy` | `false` — **sensível a maiúsculas** |
| um nó em `enemy.flying.boss` responde a `get_nodes_in_group("enemy")`? | **não** — grupos **planos**, o ponto é texto |
| `call_group` | pela ordem da árvore, **inclui quem chama** se estiver no grupo |
| nó retirado da árvore | continua `is_in_group == true` e **sai** das consultas |
| `PackedScene.pack` + `instantiate` | só sobrevivem os grupos `persistent = true` |
| grupos do PROJECTO | o binário traz `Global Groups`, `global_group/`, `add_global_group`, `remove_global_group` e `GroupSettingsEditor::_confirm_rename` (`strings /usr/bin/godot`) |
| custo, 100 000 nós no grupo | `get_nodes_in_group` **6,665 ms** a 1.ª, **0,957 ms** a 2.ª (índice por `SceneTree`) |

**Blender 5.2.1 (GPL — parede: saída apenas)** — coleções, `blender -b --factory-startup --python docs/Components/ferramentas/blender_collections_probe.py`, e duas perguntas a mais corridas por `--python-expr`:

| pergunta | saída |
|---|---|
| `bpy.data.groups` existe? | **`False`** — os *Groups* **foram retirados** (2.80) |
| um objecto em duas coleções | `['enemy', 'flying']` — **pertença múltipla** |
| objecto só na coleção-filha aparece no pai? | `parent.all_objects = ['Bat', 'Goblin']` — **contenção HIERÁRQUICA** |
| renomear a coleção-filha | o objecto passa a dizer `['winged']` — a pertença é **referência**, não texto |
| uma coleção com DOIS pais | aceite (`['a', 'b']`) — as coleções são um **grafo**, não uma árvore |
| um ciclo (`c` pai de `a`, que é pai de `c`) | **recusado**: `Collection 'a' already in collection 'c'` |
| apagar a coleção de um objecto | o objecto **vive** e perde só aquela pertença (`users_collection = []`) |
| `Inimigo`, `inimigo`, `inímigo` | **três** coleções distintas — o Blender **não** dobra maiúsculas nem acentos |
| criar uma segunda `enemy` | vira **`enemy.001`** |

**Blender 5.2.1 — catálogos de assets** (o ficheiro de dados instalado,
`/usr/share/blender/5.2/datafiles/assets/blender_assets.cats.txt`, formato `UUID:caminho/do/catálogo:nome`):

| pergunta | saída |
|---|---|
| a identidade | um **UUID** por catálogo; a hierarquia é o **caminho** (`Brushes/Mesh Sculpt/General`) |
| gémeos | **2 dos 63** caminhos aparecem **duas vezes com UUIDs diferentes** (`Brushes/Mesh Sculpt/General/Utilities`, `Geometry Nodes/Generate`) — o modelo do Blender **admite** dois catálogos com o mesmo caminho, no ficheiro que ele próprio distribui |

⭐ **E este é o modelo que a casa JÁ tem:** `ph2d_asset_index::CatalogTree` é o *Asset Browser* do
Blender — `{ id, path }`, renomear reescreve um prefixo, apagar leva os descendentes, as atribuições
guardam o id —, gravado no `ProjectState` com undo (`project_library.rs`).

### §1.2 — Da documentação pública (sem artefacto instalado; ⚠️ não medido)

| app | modelo | o que tentou e abandonou (ou manteve contra a corrente), e porquê |
|---|---|---|
| **Unreal** | *Gameplay Tags* hierárquicos registados num dicionário do projecto; `MatchesTag("A")` é verdade para `A.B`; `FName` sem distinção de maiúsculas; *redirects* para renomear — [dossiê](pesquisa/dossie_unreal.md) l.233 | **As *Actor Tags* planas (`TArray<FName>`) ficaram e perderam o papel**: uma string solta não se valida (gralha) e não tem hierarquia |
| **Unity** | **UMA** tag por GameObject + 32 *Layers* | **O contra-exemplo** ([síntese interacao_fluxo §8.3](pesquisa/sintese_interacao_fluxo.md) l.511): ficou com uma só; a comunidade usa componentes-marcador, que o artista não autora sem código |
| **Godot** | grupos planos por nó + (medido) grupos globais do projecto com renomear | a gralha dos grupos-texto levou-o a um **registo do projecto** |
| **Blender** | coleções + catálogos (medido) | **Groups e as 20 camadas fixas foram abandonados** pelas coleções (2.80); o *Asset Browser* nasceu depois com **identidade por UUID e hierarquia por caminho** |
| **Construct 3** | *Families* (por TIPO) + *instance tags* com filtro no *Solid* e na *Physics* — [dossiê](pesquisa/dossie_construct_gdevelop.md) l.124, l.290, l.344 | as *Families* são por tipo; o filtro por instância teve de chegar à parte |
| **GameMaker** | objecto-pai como grupo — [dossiê](pesquisa/dossie_gamemaker_defold.md) l.228 | amarra o grupo à herança: **um** pai |
| **Phaser** | `Group` não-exclusivo + *pooling* — [dossiê](pesquisa/dossie_cocos_phaser.md) l.259 | — |
| **Bevy** | componente-marcador | idioma certo de um ECS, inalcançável sem código |
| **After Effects** | *Labels*: **uma** cor por camada (16), *Select Label Group* | etiqueta **exclusiva** — a limitação do Unity |
| **Illustrator** | sem tags de objecto: camadas, grupos, *Select ▸ Same* | agrupa pelo que o objecto **parece**, não pelo que ele **é** |
| **Rive** | não conheço grupos consultáveis na cena; o runtime endereça por nome | ⚠️ **não verificado** — sem artefacto nesta máquina |

### §1.3 — As leis que saem daqui

1. **Pertença múltipla** — Blender, Godot, Unreal, Phaser, Construct. Unity e AE são o contra-exemplo.
2. **A pertença é uma IDENTIDADE, não texto** — Blender (medido: renomear mantém a pertença),
   Unreal (*redirects* existem porque o texto parte).
3. **Hierarquia por contenção** — o objecto de `Flying` pertence a `Enemy` (Blender medido,
   `all_objects`; Unreal `MatchesTag`).
4. **Uma ÁRVORE, não um grafo** — o *Asset Browser* do Blender e o Unreal são árvores; as coleções
   são grafo (medido) porque também são **unidades de instância**, que uma tag não é.
5. **Sem gémeos** — o Blender admite-os e distribui dois (medido); quem cria uma tag que já existe
   **recebe a que existe**.
6. **Consulta com ordem DETERMINISTA** — a do `StableId`, porque a árvore da cena não sobrevive a um
   `Ctrl+Z` (o undo respawna).

---

## §2 — O desenho: UMA porta por pergunta

### §2.1 — O modelo

| peça | onde | o quê |
|---|---|---|
| **a dobra** | crate-folha nova `ph2d-label-fold` | `fold(label) -> String`: NFD → *case folding* completo (`icu_casemap`) → NFD → sem marcas não-espaçadoras (`GeneralCategory::NonspacingMark`, `icu_properties`) → espaços colapsados. `Inimigo` = `inimigo` = `inímigo` = `INÍMIGO`; `Straße` = `STRASSE`; `é` pré-composto = `e` + U+0301 |
| **a álgebra de caminhos** | crate-folha nova `ph2d-label-path` (zero deps) | `SEP = '/'` · normalizar · prefixo **por segmento** · reescrever prefixo · ordenar por segmento com uma chave dada — **extraída** do `CatalogTree`, que passa a usá-la sem mudar comportamento |
| **a árvore** | crate-folha nova `ph2d-tags` (sem ECS, sem serde) | `TagId(u64)` · `Tag { id, path }` · `TagTree { tags, next_id, revision }` — o molde exacto do `CatalogTree` |
| **a pertença** | `ph2d_ecs::Tags(BTreeSet<TagId>)`, módulo irmão `tags.rs` | componente REGISTADO; CONFIG, nunca estado vivo |
| **o documento** | `ProjectState.tags` (bytes com versão própria, `ph2d-app-components::tags_doc`) + a árvore viva no `AppGfx` | o molde do `LibraryDoc`/`catalogs`: undo, gravação e cache por revisão |

⚠️ **O nome mostra-se como foi ESCRITO** (`Inimigo Voador`) e compara-se **dobrado** — a regra do
`FName` do Unreal e de um sistema de ficheiros *case-preserving*. Quem escreve primeiro escolhe a
grafia; quem escreve `inimigo voador` depois **recebe a tag que existe**.

⚠️ **`TagId` é `u64` e não o `u128` do `CatalogId`**: o `CatalogId` tem a largura de um UUID do
Blender; um `TagId` é guardado **por objecto**, num conjunto, e 8 bytes a menos por pertença é o que
a pertença múltipla multiplica.

⚠️ **As ICU já estão no programa**: `icu_normalizer` e `icu_properties` 2.3 entram por `parley`
(`compiled_data`). `icu_casemap` 2.3 (Unicode-3.0) é o **único pacote externo novo**, e é o que dá o
*case folding* completo — com `to_lowercase` o `ß` e o `SS` ficariam tags diferentes.

### §2.2 — As portas

| pergunta | porta ÚNICA | quem chama | proibido |
|---|---|---|---|
| *estes dois nomes são o mesmo?* | `ph2d_label_fold::fold` | a árvore (criar · renomear · mover · restaurar) · a busca do painel | `to_lowercase` à mão; uma segunda tabela de acentos |
| *este caminho é o próprio ou descendente daquele?* | `ph2d_label_path::is_self_or_descendant` | `TagTree` e `CatalogTree` | `starts_with` sem a fronteira de segmento (`Hero` / `Heroine`) |
| *criar `Enemy/Flying`* | `TagTree::create(path) -> Result<TagId, TagError>` — cria os ancestrais; um caminho que já existe **dobrado** devolve o id existente | painel · smoke | criar sem dobrar |
| *renomear* | `TagTree::rename(id, label)` — reescreve o prefixo, os ids ficam | painel | tocar num `Tags` de objecto |
| *mover para dentro de outra* | `TagTree::move_under(id, Option<TagId>)` — recusa a própria subárvore (o ciclo que o Blender recusa, medido) | painel (arrastar) | — |
| *apagar* | `TagTree::delete(id) -> BTreeSet<TagId>` (a subárvore) + `ph2d_ecs::tags::scrub(world, &ids)` **no mesmo gesto** | painel | apagar a árvore sem tirar a pertença (um id órfão nos objectos) |
| *que tags estão debaixo desta?* | `TagTree::subtree(id) -> BTreeSet<TagId>` | todas as consultas | expandir à mão |
| *este objecto pertence a `q`?* | `ph2d_ecs::tags::belongs(tags, tree, q)` | o filtro da física | ler o `BTreeSet` fora do módulo |
| *quem pertence a `q`?* | `ph2d_ecs::tags::tagged(world, tree, q) -> Vec<Entity>`, ordem do `StableId` | `SignalActions::resolve` · *Select Tagged* · a contagem do painel | uma segunda varredura; um índice guardado |
| *quem sofre esta acção?* | `signal_actions::targets_of(world, tree, source, &action) -> Vec<Entity>` | `resolve` | resolver o alvo na shell |
| *este sinal de colisão passa o filtro?* | `ph2d_physics_ecs::signal_passes(sim, tree, source, other)` | `PhysicsBridge::signal_events` (chegada **e** saída) | um filtro por fase |
| *ler um documento com gémeos* | `TagTree::restore(..) -> (TagTree, Remap)` + `ph2d_ecs::tags::remap(world, &remap)` | o load | perder a pertença do gémeo descartado |

⭐ **Sem índice, e é MEDIDO** (§6.1).

### §2.3 — Os consumidores

1. **`SignalActions`, alvo por tag** — `SignalAction` ganha `target_by: SignalTarget { Named, Tagged(TagId) }`
   **apendado no fim**. `Named` é o de hoje, **byte a byte** (vazio = este objecto). `Tagged(id)`:
   todos os que pertencem à subárvore; id que já não existe = **ninguém** (a lei do alvo que não
   existe). Ordem: *reactor (StableId) → linha escrita → alvo (StableId)*. ⭐ A shell que aplica **não
   muda** (`render_loop::signal_actions::apply` já faz um efeito por alvo); ela passa a árvore ao
   `resolve`. Inclui quem reage, se pertencer — o `call_group` do Godot (medido).
2. **Filtro de colisão** — `ph2d_physics_ecs::SignalTagFilter(TagId)`, componente **irmão** do
   `SignalOnHit` (o cabeçalho do `signal.rs` diz porque não um campo do `Collider`). Ausente = sem
   filtro, o mundo de hoje **byte-idêntico**. Tag apagada = **não passa ninguém** (falha fechada; o
   painel diz *«Missing tag»*). Consultado **dentro** do `signal_events`, que é quem tem o `other`;
   a chamada de produção é **uma** (`render_loop/fase_signal_outbox.rs:63`).
3. **O editor** — *Select Tagged* (a selecção múltipla existe: `add_to_selection`/`extra_selection`)
   e a contagem por tag no painel.

### §2.4 — Fora desta wave, com o degrau nomeado

| fica de fora | degrau |
|---|---|
| Descrição por tag | D3 (o dono: *«pode ser»* sem ela) |
| *Families* com traits | etapa 2 da síntese |
| `Team` | P1; decide-se com a percepção |
| `CameraFollow` por tag | outro layout ⇒ outro bump; entra com o Spawner |
| Os **catálogos** da biblioteca passarem a dobrar acentos | ⚠️ **decisão do dono** — a porta fica pronta e partilhada, e o `CatalogTree` continua a comparar como hoje |
| Tags vindas de uma biblioteca **de outro projecto** | ADR-0165 §5 (bibliotecas lado a lado) |

### §2.5 — As decisões do dono (2026-09-13)

| # | pergunta | decisão | o que ela fixou no desenho |
|---|---|---|---|
| **D1** | hierarquia | *«Faça como o Blender»* | pertença por **identidade**, hierarquia por **árvore**, renomear/mover não toca nos objectos — o modelo do *Asset Browser*, que a casa já tem no `CatalogTree`. ⚠️ **Divergências declaradas do Blender**: árvore e não grafo (as coleções aceitam dois pais — medido — porque são instanciáveis); sem gémeos (o Blender distribui dois — medido) |
| **D2** | nomes | *«Maiúscula não importa, letra acentuada não importa»* | a dobra do §2.1; ⚠️ **divergência declarada do Blender e do Godot**, que distinguem as três grafias (medido) |
| **D3** | lista do projecto com descrição | *«pode ser»* — sem descrição | a árvore do projecto existe (é o D1), sem campo de descrição |
| **D4** | prefab | *«sim»* | a tag da receita vai para as cópias; a cópia pode ter a sua lista |

---

## §3 — Contrato congelado e schema

### §3.1 — Contratos congelados (§6 do `CLAUDE.md`): **nenhum é tocado**

Prova por grep sobre `4ecaddb5f`:

```text
crates/ph2d-nodegraph/src/node.rs            ocorrências de tag|group: 0
crates/ph2d-editor-core/src/tool.rs          ocorrências de tag|group: 1  → «RadioGroup» num doc-comment (l.36), alheio
crates/ph2d-vector-doc · ph2d-vector-traits  ocorrências de Tags: 0
```

### §3.2 — Os contadores, em DELTA (nunca o literal — `CLAUDE.md` §5.0)

| contador | hoje | delta | porquê |
|---|---:|---:|---|
| `ph2d-ecs` `reg.len()` (`registry_tests.rs:191`) | 85 | **+1** | `Tags` |
| espelhos `ph2d-render` / `ph2d-script` | 86 | **+1** | a mesma conta |
| `ph2d-physics-ecs` `reg.len()` (`lib.rs:198`) | 32 | **+1** | `SignalTagFilter` |
| `PROJECT_SCHEMA` | 128 | **+1** | um degrau para quatro razões: o campo `ProjectState.tags` (postcard posicional), dois componentes **novos** (a regra do degrau 122→123) e o **layout** do `SignalAction` |
| `LIVE_SECTIONS` / `any_live_section` | 20 / 15 | **+1 / +1** | a secção *Tags* do Inspector |
| painéis registados (`EXPECTED_TYPED`) | — | **+1** | o painel *Tags* (W4) — ícone `IconId::Tag` **já existe** (`tag.svg`) |
| `Cargo.lock` | — | **+ `icu_casemap`, `icu_casemap_data`** | o único pacote externo novo |
| `FLIP_SCHEMA` · `VEC_SCENE_SCHEMA` · `DOC_VERSION` · `FIELD_DOC_VERSION` | — | 0 | não tocados |

### §3.3 — A migração

Um degrau `128 → 129` com **uma** migração de blob, pelo precedente exacto do v97→v98
(`project_migrate_sprite.rs`): um `SignalActionV1` congelado `(on, target, verb, arg)` lê os bytes
antigos e escreve `target_by = Named`. A lei do re-encode vive no `ph2d-ecs`
(`signal_actions::migrate_v1_blob`); a shell só a chama. Um v128 não tem `ProjectState.tags` ⇒ árvore
vazia, que é o que ele era.

---

## §4 — A UI: as QUATRO condições, independentes

| superfície | 1. EXISTE | 2. PINTADO e REGISTADO | 3. o clique chega ao BARRAMENTO | 4. a SEQUÊNCIA leva a algum lugar |
|---|---|---|---|---|
| **Secção *Tags*** do Inspector (logo abaixo da *Identity*; `C::Identity`, `O::ANY`, anexa-se pela paleta — ADR-0166) | `Tags` registado + descritor | chips `widget::tag::Tag` (nome da folha; o caminho no balão) + `widget::combobox::Combobox` com a busca dobrada e a linha *Create “…”* quando nada casa | `EditorAction::InspectorTagsEdit { entity, op: Add(TagId) \| Create(String) \| Remove(TagId) }` | a pertença entra em `tagged` (gate) e um *Signal Actions* por tag passa a atingir o objecto (smoke) |
| **Alvo do *Signal Actions*** | `SignalTarget` | segmentado `Name \| Tag`; com `Tag`, um `Combobox` das tags | `InspectorActionEdit` ganha o op `TargetBy` | `resolve` devolve N efeitos e a shell aplica-os (gate + smoke) |
| **Filtro da armadilha** (secção *Physics*, por baixo do *On Hit* / *On Leave*) | `SignalTagFilter` | linha *Only for tag* com `Combobox` e limpar | op novo no evento da física | `signal_events` **decide** (§5.0: o leitor decide, ou entrega a quem descarta?) |
| **Painel *Tags*** (docado, fechado por omissão) | a `TagTree` do documento | árvore (`widget::tree_view`) com a contagem por tag; *+ New* · *+ Child* · renomear (duplo clique) · apagar (*«remove from N objects»*) · arrastar para mover · *Select Tagged* | `EditorAction::Tag{Create,Rename,Move,Delete,SelectAll}` | a árvore muda e os objectos **não perdem** a pertença (gate); apagar é **um** passo de undo com a pertença (gate); a selecção passa a ser `tagged` |

⚠️ **O erro diz-se NA LINHA, nunca num toast** (*«A tag with this name already exists here»*, *«Cannot
move a tag inside itself»*) — a lei do L-System (`TextRow.problem`), com gate de **glifo**.
⚠️ **E o painel novo tem CINCO sítios de registo** (memória `feedback_docked_panel_registration_four_sites`:
crate+sync · feature proxy da shell · z-walk · visibilidade · `cursor_over_hero_panel`) — reconferidos
contra o código na W4, não contra a nota.

---

## §5 — Os gates, red-first, e as fixtures que CONTÊM o fenómeno

### §5.1 — As fixtures

**A dobra** — uma tabela de pares, os que TÊM de colapsar e os que NÃO podem:

| colapsam | não colapsam |
|---|---|
| `Inimigo` · `inimigo` · `INIMIGO` · `inímigo` · `ÍNIMIGO` | `inimigo` ≠ `inimiga` |
| `é` (U+00E9) · `e` + U+0301 | `Enemy` ≠ `Enemies` |
| `Straße` · `STRASSE` · `strasse` | `a b` ≠ `ab` |
| `Inimigo  Voador` · `inimigo voador` | — |

**A árvore** — `Enemy` › `Flying` › `Boss`, a raiz irmã `Statue`, e os objectos Goblin A/B (`Enemy`),
Bat A/B (`Flying`), Dragon (`Boss`), Statue (`Statue`), Hero (`Player`). ⚠️ Com duas perturbações:
um componente alheio inserido no Goblin A (muda a ordem de arquétipo) e um restore do snapshot (bits
novos).

**Os oráculos com cabeçalho** — a saída da sonda v2 do Blender
([`blender_tags_hierarchy_probe.py`](ferramentas/blender_tags_hierarchy_probe.py)) **sobre a cena
acima**, em quatro estágios (`BASE` · `RENAME` · `MOVE` · `CYCLE`) mais a linha `CASE`, gravada em
[`crates/ph2d-ecs/tests/it/fixtures/blender_tags_hierarchy.txt`](../../crates/ph2d-ecs/tests/it/fixtures/blender_tags_hierarchy.txt).

⚠️ **Correcção à 1.ª redacção (W1, 2026-09-13): as 63 linhas do ficheiro de catálogos NÃO vêm
para cá.** Aquele ficheiro é um DADO distribuído com o programa, não a SAÍDA de uma entrada nossa
(CLAUDE.md §0.9 — a porta livre é a saída). O que o gate precisa é do FENÓMENO medido nele (2 de 63
caminhos duplicados, com UUIDs diferentes), e esse está reproduzido numa fixture nossa, com o gémeo
dobrado que a D2 acrescenta (`a_document_with_twins_merges_them_and_keeps_every_member`).

### §5.2 — Os gates, por wave (escritos ANTES da porta, vistos VERMELHOS contra um *stub*)

**W1 — as folhas e a lei:**
1. `the_fold_collapses_case_and_accents_and_nothing_else` — a tabela da dobra, as duas colunas.
2. `the_fold_is_idempotent_and_normalisation_blind` — `fold(fold(x)) == fold(x)`; NFC e NFD dão o mesmo.
3. `the_catalog_tree_behaves_exactly_as_before_the_extraction` — os **12** gates do `catalog_tests.rs` intocados.
4. ⚠️ `renaming_a_catalog_onto_an_existing_sibling_is_refused` — **defeito latente ACHADO ao ler o
   `CatalogTree::rename`**: ele não confere se o caminho novo já existe, e o `delete` filtra por
   caminho ⇒ dois gémeos apagam-se **juntos** e os assets do outro saem da gaveta. Red-first contra o
   `main`.
5. `creating_a_tag_that_exists_folded_returns_the_existing_one` — `Enemy` e `énemy`.
6. `renaming_or_moving_a_tag_never_touches_a_member` — no `ph2d-ecs`, com a fixture.
7. `moving_a_tag_carries_its_subtree_and_refuses_a_cycle` · `a_rename_that_collides_with_a_sibling_is_refused_and_a_new_spelling_is_not` · `the_cycle_verdicts_are_the_ones_blender_measures` (os dois vereditos contra o oráculo).
8. `deleting_a_tag_takes_its_subtree_and_the_membership_in_one_gesture` · `scrubbing_only_stamps_the_objects_it_changes`.
9. `a_restored_tree_never_recycles_an_id` (a lei do `CatalogTree`) · `a_document_with_twins_merges_them_and_says_where_each_went` (a árvore) · `a_document_with_twins_merges_them_and_keeps_every_member` (a pertença) — com o fenómeno medido no ficheiro do Blender, reproduzido numa fixture nossa (§5.1).
10. `the_hierarchy_is_the_one_blender_measures` — conjunto a conjunto nos três estágios do oráculo · `the_blender_keeps_three_spellings_and_the_owner_asked_for_one` (a divergência D2 nas duas metades).
11. `the_query_order_is_the_identity_not_the_archetype` (com o controlo de que a perturbação perturba, e um piso de 7 linhas na captura) · `tags_bytes_do_not_depend_on_insertion_order`.
12. `only_the_door_reads_tags` — censo sem comentários/strings, com **piso de população** (HOWTO §2.7) · `the_stripper_keeps_code_and_drops_comments_strings_and_chars` (o controlo do instrumento).
13. `the_tree_sorts_by_folded_level` — `Ártico` antes de `Zebra`; o pai sempre antes dos filhos.
    **E as leis da porta que a W1 acrescentou:** `belonging_reaches_the_subtree_and_never_the_sibling_root` · `a_tag_that_no_longer_exists_reaches_nobody` · `a_refused_gesture_does_not_move_the_revision`.

**W2 — consumidores, documento, schema:**
14. `a_signal_to_a_tag_reaches_the_whole_subtree_and_the_sibling_root_stays` (com o controlo de que a perturbação de arquétipo perturba) · `the_reactor_is_reached_when_it_belongs_to_the_tag` (o `call_group` do Godot, medido).
15. `a_deleted_tag_target_reaches_nobody` (a tag apagada, o id de outro documento e o reservado `0`) · `a_named_target_resolves_the_same_whatever_the_tree`.
16. `a_v128_signal_action_loads_as_a_named_target` — bytes congelados de um v128, com o controlo de que o tipo VIVO os recusa · `a_blob_that_is_not_v128_is_left_alone` · e na shell `a_frozen_v128_file_migrates_its_signal_actions` + `the_live_type_cannot_read_a_v128` (o ficheiro inteiro, montado campo a campo).
17. `a_filtered_trap_ignores_a_non_member_on_arrival_and_departure` · `an_unfiltered_trap_is_byte_identical` · `a_missing_filter_tag_passes_nobody`.
18. `the_tag_tree_travels_in_the_project_and_through_undo` (um `Ctrl+Z` depois de apagar devolve a árvore **e** a pertença, e o gesto mexe em `["world", "tags"]` e em mais nada) · `an_untouched_tag_tree_does_not_register_a_step` — mais os cinco do FORMATO em `ph2d_app_components::tags_doc`: `a_tag_tree_survives_its_own_bytes` · `the_next_id_travels_so_a_deleted_tag_is_never_recycled` · `an_unreadable_or_foreign_blob_opens_an_empty_tree` · `a_document_with_twins_merges_and_hands_back_the_remap` · `the_cache_encodes_once_per_revision_and_again_after_invalidate`.
19. Contadores: `85→86`, `86→87` (×2), `PROJECT_SCHEMA` `128→129` e a tripla `(129, 13, 22)` — o `32→33` da física vai com a W3 (ver a nota do §7).
20. `a_recipe_tag_reaches_every_copy` (D4) — a tag da receita chega às cópias, uma cópia nova nasce com ela, e a lista própria de uma cópia sobrevive ao passe.
    ⛔⛔ **E a prova de mutação desta achou um campo MORTO no substrato:** a 1.ª mutação pôs o
    descritor do `Tags` em `Propagation::InstanceLocal` e **SOBREVIVEU** — porque a política do
    descritor **não tem um único leitor na workspace** (`grep -rn 'Propagation::'` fora do próprio
    catálogo: zero; `\.policy`: zero). Quem decide o que propaga é o REGISTO (o passe compara os
    BYTES dos componentes registados), e o papel que a `InstanceLocal` descreve é hoje uma **lista
    escrita à mão** dentro do passe (`ROOT_IS_ITS_OWN`, `instance_sync.rs`). ⇒ a alavanca da
    mutação é tirar o `Tags` do registo. ⚠️ *Um campo declarado em todo descritor e lido por
    ninguém é uma promessa que a próxima wave vai acreditar* — fica nomeado no handoff.

**W3 — o Inspector (seam, `ph2d-ui-testkit`, gesto REAL):**
21. escrever `inimigo` + `Enter` com `Inimigo` existente ⇒ o chip é o **existente**, e nada é criado;
22. escrever um nome novo ⇒ *Create “…”* cria e marca, num passo;
23. o `×` do chip remove;
24. o segmentado `Name | Tag` e o *Only for tag* chegam ao barramento;
25. `architecture_panel_wiring_parity` e `hit_indexed_ids_are_registered` verdes.

**W4 — o painel *Tags* e o smoke:**
26. criar, criar filho, renomear, arrastar para mover e apagar — pelo ponteiro;
27. a colisão de nomes e o ciclo dizem-se a **glifo**;
28. *Select Tagged* selecciona a subárvore inteira.

Cada gate diz, no doc-comment, a mutação que o sangra; os de W1/W2 são corridos (`/pd-mutacao`).

---

## §6 — O smoke, com os números MEDIDOS antes de escrito

### §6.1 — A sonda Rust (`measure_tag_scan`, `--release`, mediana de 25)

**A PORTA REAL** (W1 — `ph2d_ecs::tags::tagged` sobre a `TagTree` já com a chave guardada, §6.1-bis;
load `2,77`):

| objectos | com tags | acertos | consulta por tag | alvo por NOME, hoje |
|---:|---:|---:|---:|---:|
| 100 | 10 | 7 | 0,0004 ms | 0,0018 ms |
| 100 | 100 | 67 | 0,0013 ms | 0,0018 ms |
| 1 000 | 100 | 67 | 0,0013 ms | 0,0050 ms |
| 1 000 | 1 000 | 667 | 0,0059 ms | 0,0030 ms |
| 10 000 | 1 000 | 667 | 0,0059 ms | 0,0241 ms |
| 10 000 | 10 000 | 6 667 | 0,0565 ms | 0,0253 ms |
| 100 000 | 10 000 | 6 667 | **0,0565 ms** | 0,2332 ms |
| 100 000 | 100 000 | 66 667 | **0,5624 ms** | 0,2548 ms |

⇒ **a varredura cabe** — o pior caso é `3,4 %` de um quadro de `16,7 ms`, e o custo mora nos ACERTOS
(a ordenação por identidade), não nos objectos: 100 000 objectos com 10 000 marcados custam o mesmo
que 10 000 com 10 000. O teste irmão `the_probe_reaches_the_subtree_and_not_the_sibling_root` fixa a
lei da sonda (`20` de `30`).

⚠️ **A 1.ª redacção desta tabela mediu um SUCEDÂNEO** (um componente local e a expansão da subárvore
escrita à mão, load `4,98`): `0,0432` e `0,4464 ms` nas duas linhas a negrito. A porta real custa
~`25 %` mais no pior caso — a query leva o `Option<&StableId>` e a subárvore é a da árvore verdadeira —,
e a decisão *«sem índice»* **mantém-se**. *Uma sonda que mede um sucedâneo para sempre mede outro
programa.* Uma corrida do sucedâneo a load `30,55` deu o dobro dos tempos com a mesma forma (carga,
não o modelo); o modelo por strings da 1.ª redacção dava `1,4702 ms` no pior caso (load `1,38`).

### §6.1-bis — A ÁRVORE a crescer (`measure_tag_tree_scale`), e a cura que ela PEDIU

A tabela de cima tem **4 tags**. Esta pergunta o que as portas custam com a árvore a crescer, com três
relógios porque são três chamadores de frequência diferente: o `restore` (o LOAD), a `subtree` (cada
consulta de `tagged`) e o `belongs` (o filtro da física, **a cada evento de colisão**).

**Antes** — a árvore dobrava os caminhos a CADA comparação (load `2,57`):

| tags | `restore` | `subtree(Raiz 0)` | `belongs`, 1 objecto |
|---:|---:|---:|---:|
| 10 | 0,060 ms | 0,00248 ms | 0,00248 ms |
| 73 | 6,451 ms | 0,01793 ms | 0,01803 ms |
| 584 | **2 461 ms** | 0,13742 ms | 0,13743 ms |

⛔ **Pior que quadrático** (`8×` as tags, `107×` o tempo): cada tag lida procurava o gémeo dobrando a
árvore inteira, e abrir um projecto com 584 tags custava **dois segundos e meio**. A linha de 2 048 não
acabou em minutos e foi interrompida. O `belongs` era linear em chamadas à ICU.

**Depois** — cada entrada guarda a sua chave dobrada, escrita só pelas portas que mudam a árvore; o
`restore` indexa num mapa local; a subárvore é um intervalo contíguo; o `belongs` pergunta
`TagTree::reaches` sem construir conjunto (load `1,37`):

| tags | `restore` | `subtree(Raiz 0)` | `belongs`, melhor caso | `belongs`, a ÚLTIMA tag |
|---:|---:|---:|---:|---:|
| 10 | 0,012 ms | 0,00021 ms | 0,00004 ms | 0,00004 ms |
| 73 | 0,057 ms | 0,00057 ms | 0,00002 ms | 0,00004 ms |
| 584 | **0,522 ms** | 0,00059 ms | 0,00002 ms | 0,00028 ms |
| 2 336 | 2,302 ms | 0,00063 ms | 0,00002 ms | 0,00096 ms |
| 9 344 | 9,242 ms | 0,00066 ms | 0,00002 ms | **0,00387 ms** |

⇒ **o load de 584 tags passa de `2 461` a `0,522 ms` (~`4 700×`)** e cresce ~linearmente; nenhuma
consulta chama a ICU. ⚠️ **O pior caso do `belongs` continua LINEAR nas tags** (as duas procuras por
id): `3,9 µs` a 9 344 tags, logo 100 eventos de colisão filtrados num quadro custam `0,39 ms`
(`2,3 %`). ⛔ Um mapa `id → posição` tirava-o, e é estado derivado que cada reordenação invalidaria;
**a medição não o pede** — uma árvore real tem dezenas a centenas de tags (`0,28 µs` a 584).
⚠️ A coluna do melhor caso sozinha escondia isto: a 1.ª versão da sonda media uma tag do início da
ordem, que as duas procuras acham à primeira.

### §6.2 — As cenas `PH2D_TAGS_SMOKE=1..2` (W4; família `components`, `max_level` contado)

1. **`=1`** sobe com a fixture do §5.1 lado a lado, cada objecto com o nome e as tags escritos por
   baixo, o painel *Tags* aberto a mostrar `Enemy (5)` › `Flying (3)` › `Boss (1)`, `Statue (1)`,
   `Player (1)`.
2. Um objecto vazio *Scene Brain* com um `Timer` `alarm` de **2 s** e *on `alarm` → Tag `Enemy` → Hide*.
3. **O que tem de acontecer:** aos 2 s somem **5** objectos; ficam **Statue** e **Hero**. O terminal
   imprime `[tags] alarm: 5 escondidos · Statue intacta`.
4. **Renomear `Enemy` para `Inimigo` no painel e voltar a correr** — continuam a sumir os mesmos 5.
5. **`=2`**: uma armadilha (sensor, *On Hit* `trap`, *Only for tag* `Player`); um Goblin atravessa-a
   primeiro (**nada**), depois o Hero (a porta abre). O terminal imprime `[tags] trap: 0 → 1`.
6. **Como saber que deu errado:** a Statue some (a raiz irmã casou) · menos de 5 somem (a hierarquia
   não chegou) · depois de renomear nada some (a pertença era texto) · a porta abre com o Goblin (o
   filtro não decide).

---

## §7 — As waves, e a ordem

| wave | entrega | acaba em |
|---|---|---|
| **W1** | `ph2d-label-fold` · `ph2d-label-path` (e o `CatalogTree` a usá-la, com o defeito do gémeo curado) · `ph2d-tags` · `ph2d_ecs::tags` (portas, registo, catálogo) | gates 1–13 vistos vermelhos · headless |
| **W2** | `SignalTarget` + `targets_of` · `ProjectState.tags` (formato, cache, undo, load com gémeos) · `PROJECT_SCHEMA` +1 com a migração · contadores | gates 14–16, 18, 20 · headless |
| **W3** | secção *Tags* do Inspector · alvo por tag · **`SignalTagFilter` + `signal_passes`** e a linha *Only for tag* | gates 17, seam 21–25 + smoke do Enio |
| **W4** | painel *Tags* · `PH2D_TAGS_SMOKE=1..2` | gates 26–28 + smoke do Enio |

### §7.1 — A W3 partiu-se em três, e a W3a FECHOU (2026-09-13)

| | entrega | estado |
|---|---|---|
| **W3a** | o TECTO do objecto (`TAGS_MAX`) · a costura na shell (instantâneo + commit) · a **secção *Tags*** do painel (chips com `×` · caixa de escolha com busca · `+ Create "…"`) | ✅ 7 gates na shell · 8 no painel · **18 mutações** |
| **W3b** | a árvore sobe para porta própria · o alvo por tag nas *Signal Actions* (`Name \| Tag` + caixa) | ✅ 12 gates na shell · 3 no painel · 6 mutações |
| **W3c** | `SignalTagFilter` + `signal_passes` + a row *Only for tag* (fecha o gate 17) | ✅ 4 gates na física · 6 mutações |

### §7.3 — O que a W3c decidiu, e é tudo sobre COMO UMA COISA FALHA

| estado | resposta | porquê |
|---|---|---|
| sem componente | **passa todos** | o mundo de hoje, byte-idêntico |
| filtro por escolher (`0`) | **passa todos** | um filtro por acabar não pára a armadilha |
| tag viva | só quem **pertence** (com a subárvore) | a porta `belongs`, nunca o conjunto directo |
| **tag apagada** | **não passa ninguém** | falha FECHADA — ver abaixo |

⛔⛔ **A falha fechada é a decisão, e a aberta seria pior:** com ela, apagar uma tag no painel *Tags*
faria uma armadilha calibrada para o jogador passar a gritar com TUDO o que lhe toque. *Entre «deixa
de funcionar» e «funciona para toda a gente», a segunda é a que estraga uma cena sem ninguém
perceber.* ⚠️ E ela distingue-se do filtro **por escolher**: tratá-los igual faria anexar o
componente PARAR a armadilha até o artista adivinhar porquê.

⭐ **O `(any)` da lista é a MESMA edição com `0`, e DESANEXA o componente** (o idioma do
*presence-override* que a `WalkSurface` já usa). ⛔ Sem essa entrada, anexar o filtro era um caminho
sem volta pelo painel — *um controlo que não se desfaz é pior que um que não existe*. E guardar
`Some(0)` no documento poria lá um no-op que o `signal_passes` teria de aprender a ignorar.

⚠️ **E o gate `every_registered_physics_component_has_a_ui_writer` fez o trabalho dele à letra:**
acusou o `SignalTagFilter` no minuto em que ele foi registado sem row no painel. É por causa dele
que esta fatia saiu da W2 — o componente, a lei e a row viajam juntos, senão fica vermelho no
intervalo entre as duas waves.

⛔⛔ **DUAS correcções ao que este plano afirmava, as duas medidas na W3a:**

1. **O chip com `×` EXISTE** — `ph2d_editor_core::widget::tag` (`Tag`, `WidgetKind::Tag`,
   `InteractiveState::Tag`, e um `close_rect()` que dá ao `×` zona de acerto própria). O plano dizia
   *«não existe widget de chip»* e mandava construí-lo com botões. ⚠️ O **único** consumidor era o
   *showcase*, e é por isso que um levantamento feito a partir dos painéis lê zero — *uma ausência
   afirmada pela população de consumidores é um palpite com cara de medição*. ⚠️ Preço real de um
   widget por estrear: não há `store.tag_visual()` como há `button_visual()`, porque nunca houve
   quem o pedisse; usa-se a porta `Tag::visual((estado, hover))`.
2. **O tamanho da lista de escolha é MEDIDO, e o recurso não é o modelo** — a árvore não tem cap
   (`9 344` tags abrem em `9,2 ms`), logo o array de ids não pode copiar um cap que não existe. O
   recurso é o POPOVER, preso à altura da coluna do Inspector:

   | janela | região do popover | linhas visíveis |
   |---|---:|---:|
   | workstation `2560×1440` | `1376,0 px` | **62** |
   | portátil `1600×900` | `836,0 px` | 38 |
   | tablet `1280×800` | `736,0 px` | 33 |

   ⇒ **`INSP_TAGS_OPT = 64`**, o ecrã maior mais dois, com o gate
   `a_lista_de_tags_cobre_um_ecra_cheio` a imprimir a tabela e a reprovar se alguém mexer na altura
   da linha. ⛔ **E o que passa daqui é CONTADO na tela** (*«Showing 64 of N — type to narrow»*):
   um chooser que esconde metade dos resultados em silêncio ensina que a tag não existe.

### §7.2 — A W3b começa por CORRIGIR onde a lista da árvore vive

⛔⛔ **O `InspectorTagsInfo.all` está no sítio errado, e só a SEGUNDA superfície o mostra.** A lista
inteira da árvore viaja hoje dentro do instantâneo da secção *Tags*, que é **por objecto** e nasce
`None` para quem não tem o componente (ADR-0166). A secção *Signal Actions* precisa da mesma lista
para escolher o alvo — e um objecto com `SignalActions` **pode não ter `Tags` nenhum**. ⇒ com a
lista onde está, a caixa de escolha do alvo abre vazia exactamente no caso normal.

⚠️ **A árvore é um documento do PROJECTO, não dado de um objecto** — a `VecScene` e o `FlipDoc` já
viajam assim. ⇒ a W3b abre com a lista a subir para uma porta própria
(`set_current_tag_tree(Vec<InspectorTagRow>)`), lida pelas DUAS superfícies; o
`InspectorTagsInfo` fica só com o que é do objecto (`on_object`, `full`, `selected_count`).

⭐ *Não foi um erro de leitura do plano: foi a forma certa enquanto houve um consumidor só.* O
segundo consumidor é que revela o nível a que um dado pertence — e é por isso que esta correcção
vem **antes** da feature dela, e não depois.

⭐⭐ **E a busca dobra pela porta da ÁRVORE** (`ph2d_label_fold::fold`), o que fez o
`ph2d-panel-inspector` ganhar essa folha como dependência. ⛔ Um `to_lowercase` no painel seria a
segunda resposta à mesma pergunta: `Énemy` deixaria de ser encontrado por `enemy` na busca enquanto
a árvore continuava a tratá-los como a mesma tag — ou seja, a decisão D2 do dono valeria no
documento e não na única superfície onde ele a exerce.

⚠️⚠️ **O FILTRO DA FÍSICA mudou-se da W2 para a W3 na implementação (2026-09-13), e o motivo é um
gate:** o `every_registered_physics_component_has_a_ui_writer` (em `shells/desktop/tests/it/`) exige
que todo componente de física REGISTADO seja nomeado por um caminho de escrita da UI — ele existe
precisamente para impedir o órfão *«funciona em toda cena de smoke e é inalcançável no produto»*.
Registar o `SignalTagFilter` na W2, com a linha *Only for tag* a nascer só na W3, deixaria esse gate
VERMELHO no intervalo entre as duas. ⇒ o componente, a porta `signal_passes` e a row viajam juntos.

⚠️ **O degrau do `PROJECT_SCHEMA` traz uma MIGRAÇÃO, e ela não estava no §3.3:** aquele texto previa
uma travessia de blobs com o tipo VIVO (o precedente do v97→v98), e isso só vale quando a forma do
ficheiro não muda. O campo `ProjectState.tags` cai no MEIO do fluxo de bytes (o `state` é o primeiro
campo do `ProjectFile`), logo o v128 pede um tipo CONGELADO — o precedente do `ProjectFileV95`.

⚠️ **O custo na shell é uma linha por fase** (o dreno do barramento, a publicação do snapshot, o
campo do `ProjectState`, a chamada da migração) — as leis e as pontes vivem nas folhas e na família.
A catraca `the_shell_only_shrinks` tem folga depois da 5.ª rodada (191 016 contra 196 990).
⚠️ **E as ICU passam a entrar no fecho de dependências do `ph2d-ecs`** — MEDIDO na W1 antes de o
aceitar (`check` a frio num `CARGO_TARGET_DIR` vazio, `date +%s%N`, 2026-09-13):

| `check` a frio | ms | load (antes / depois) |
|---|---:|---|
| `ph2d-label-fold` sozinha (as três ICU) | 3 954 | 1,29 / — |
| `ph2d-ecs` **sem** `ph2d-tags` | 18 354 | — / 8,21 (as duas corridas em série) |
| `ph2d-ecs` **com** `ph2d-tags` | 18 589 | 3,59 / 4,93 |

⇒ **`+235 ms` (`+1,3 %`), dentro do ruído a esta carga.** A cadeia das ICU (~4 s) é mais curta do
que a do `bevy_ecs` que o `ph2d-ecs` já espera, então compila em paralelo e não alonga o caminho
crítico. Aceite.
