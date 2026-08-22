# measure

## SPIKE — captura incremental por change ticks (bevy_ecs 0.18.1)
ITERS=25 (mediana/mín), WARM=3; build = release — rodada final (3ª), `cargo test -p ph2d-ecs --release --test zz_spike_incremental_capture -- --nocapture`

### n = 1000 entidades (Transform + Name + Sprite [+ StableId não registrado em A])
loadavg antes: 15.20 10.83 9.83
| (C′) `get_mut` sem escrita em 10 % (k=100) | 0.087 ms | reserializadas=100 (esperado 100: `changed` espúrio do DerefMut) |
| (C) despawn 1 % + spawn 1 % (k=10) | 0.024 ms | reserializadas=10 · removidas=10 |
| (C″) `remove::<Sprite>` em 1 % (k=10) | 0.015 ms | reserializadas=0 (FALSO NEGATIVO se 0) · `world.removed::<Sprite>()` enumera 10 |

| medida (n=1000) | mediana | mín | nota |
|---|---:|---:|---|
| (A) `world_to_snapshot` (reg 58) | 0.501 ms | 0.490 ms | 225893 B postcard (225 B/ent.) |
| (A) `canonicalize` (verbatim) | 1.233 ms | 1.211 ms | sort por bytes de conteúdo |
| (A′) `canonicalize` sobre entrada JÁ canônica | 0.121 ms | 0.106 ms | piso O(n) do sort adaptativo (pós-restore) |
| **(A) total baseline** (ordem de criação) | **1.734 ms** | 1.701 ms | doc 01 §7.3 dizia ~0,69 ms |
| (A′) total baseline (entrada já canônica) | 0.623 ms | 0.595 ms | |
| (E) `PartialEq` snapshot inteiro (iguais) | 0.007 ms | 0.007 ms | o `undo_baseline == current` do shell |
| (B) `world_to_snapshot` (reg 59, +StableId) | 0.534 ms | 0.526 ms | 238766 B (238 B/ent.) |
| (B) `sort_by_stable_id` | 0.007 ms | 0.007 ms | vs canonicalize 1.233 ms ⇒ 173.4× mais barato |
| **(B) total** snapshot + sort sid | **0.541 ms** | 0.533 ms | |
| **(B2) build direto** query→sort(sid)→serialize | **0.432 ms** | 0.427 ms | sem DFS, sem sort de conteúdo |
| (C) 1ª captura (cache frio, tudo serializa) | 0.806 ms | — | cache = 238763 B (238 B/ent.) |
| (C) `clear_trackers()` por frame | 0.000 ms | 0.000 ms | = increment tick + swap dos buffers de remoção |
| (C) scan de ticks puro (0 mudadas, sem cache) | 0.006 ms | 0.006 ms | 6 ns/entidade |
| **(C) captura incremental, 0 mudadas (k=0)** | **0.012 ms** | 0.012 ms | delta = 0 B (0 linhas) |
| **(C) captura incremental, 1 mudadas (k=1)** | **0.012 ms** | 0.012 ms | delta = 244 B (1 linhas) |
| **(C) captura incremental, 1 % mudadas (k=10)** | **0.019 ms** | 0.019 ms | delta = 2440 B (10 linhas) |
| **(C) captura incremental, 10 % mudadas (k=100)** | **0.085 ms** | 0.084 ms | delta = 24490 B (100 linhas) |
| (C) materializar snapshot inteiro do cache (clone) | 0.013 ms | 0.013 ms | |
| (D) bytes: snapshot inteiro vs delta 1 % | 238763 B | 2440 B | 97.9× menor |
loadavg depois: 15.20 10.83 9.83

### n = 10000 entidades (Transform + Name + Sprite [+ StableId não registrado em A])
loadavg antes: 15.20 10.83 9.83
| (C′) `get_mut` sem escrita em 10 % (k=1000) | 0.952 ms | reserializadas=1000 (esperado 1000: `changed` espúrio do DerefMut) |
| (C) despawn 1 % + spawn 1 % (k=100) | 0.354 ms | reserializadas=100 · removidas=100 |
| (C″) `remove::<Sprite>` em 1 % (k=100) | 0.275 ms | reserializadas=0 (FALSO NEGATIVO se 0) · `world.removed::<Sprite>()` enumera 100 |

| medida (n=10000) | mediana | mín | nota |
|---|---:|---:|---|
| (A) `world_to_snapshot` (reg 58) | 5.093 ms | 5.000 ms | 2268893 B postcard (226 B/ent.) |
| (A) `canonicalize` (verbatim) | 18.731 ms | 17.383 ms | sort por bytes de conteúdo |
| (A′) `canonicalize` sobre entrada JÁ canônica | 1.179 ms | 1.103 ms | piso O(n) do sort adaptativo (pós-restore) |
| **(A) total baseline** (ordem de criação) | **23.824 ms** | 22.384 ms | doc 01 §7.3 dizia ~6,89 ms |
| (A′) total baseline (entrada já canônica) | 6.273 ms | 6.103 ms | |
| (E) `PartialEq` snapshot inteiro (iguais) | 0.076 ms | 0.075 ms | o `undo_baseline == current` do shell |
| (B) `world_to_snapshot` (reg 59, +StableId) | 5.285 ms | 5.210 ms | 2398766 B (239 B/ent.) |
| (B) `sort_by_stable_id` | 0.088 ms | 0.084 ms | vs canonicalize 18.731 ms ⇒ 214.0× mais barato |
| **(B) total** snapshot + sort sid | **5.372 ms** | 5.294 ms | |
| **(B2) build direto** query→sort(sid)→serialize | **4.343 ms** | 4.316 ms | sem DFS, sem sort de conteúdo |
| (C) 1ª captura (cache frio, tudo serializa) | 7.369 ms | — | cache = 2398763 B (239 B/ent.) |
| (C) `clear_trackers()` por frame | 0.000 ms | 0.000 ms | = increment tick + swap dos buffers de remoção |
| (C) scan de ticks puro (0 mudadas, sem cache) | 0.059 ms | 0.059 ms | 6 ns/entidade |
| **(C) captura incremental, 0 mudadas (k=0)** | **0.269 ms** | 0.261 ms | delta = 0 B (0 linhas) |
| **(C) captura incremental, 1 mudadas (k=1)** | **0.262 ms** | 0.260 ms | delta = 244 B (1 linhas) |
| **(C) captura incremental, 1 % mudadas (k=100)** | **0.334 ms** | 0.330 ms | delta = 24490 B (100 linhas) |
| **(C) captura incremental, 10 % mudadas (k=1000)** | **0.953 ms** | 0.944 ms | delta = 246763 B (1000 linhas) |
| (C) materializar snapshot inteiro do cache (clone) | 0.135 ms | 0.134 ms | |
| (D) bytes: snapshot inteiro vs delta 1 % | 2398763 B | 24490 B | 97.9× menor |
loadavg depois: 15.20 10.83 9.83
test zz_spike_incremental_capture ... ok

(Reprodutibilidade entre as 3 rodadas, n=10k: `world_to_snapshot` 4,952 / 5,064 / 5,093 ms · `canonicalize` 19,371 / 19,301 / 18,731 ms · (C) 0 mudadas 0,271 / 0,281 / 0,269 ms · (C) 10 % 0,934 / 0,989 / 0,953 ms — a 1ª rodada foi a loadavg 5,5, as outras a ~15.)

## reading
## Leitura (pt-BR, densa; tags [CODE] lido da fonte · [INF] inferência · [DOC] doc oficial)

### 1. O baseline do doc 01 §7.3 está SUBESTIMADO em ~3,5× no regime de edição — o `canonicalize` depende da ORDEM de entrada
- `world_to_snapshot` **reproduz** o doc (0,50 / 5,09 ms vs 0,483 / 4,808). `canonicalize` (cópia verbatim de `shells/desktop/src/undo.rs:152-180`) **não**: 1,23 / 18,7 ms contra 0,208 / 2,083 do doc.
- Mecanismo [CODE+medido]: a chave (`Vec<u8>` = concat de `(type_id, bytes)`) é construída **dentro do comparador** do `sort_by` — duas alocações+cópias de ~230 B **por comparação**. O `sort_by` do Rust é adaptativo: em entrada já ordenada faz ~n comparações; em ordem de criação (DFS por `to_bits` = ordem de spawn, que não é ordem lexicográfica do conteúdo) faz ~n·log₂n ≈ 133 k comparações a 10 k ⇒ ~266 k alocações e ~61 MB de memcpy ⇒ 18,7 ms.
- (A′) mede o piso: sobre entrada **já canônica** (o que o mundo fica logo depois de um restore, porque `snapshot_to_world` respawna em ordem de linha e a DFS por `to_bits` sai ordenada) o `canonicalize` custa 0,12 / 1,18 ms — compatível com os 2,08 ms do doc 01. [INF] O spike apagado do doc 01 mediu o regime ordenado (ordem de spawn casando com a ordem de conteúdo — p.ex. nomes zero-padded); **não determinado** qual foi a ordem de spawn dele.
- Consequência para o doc 04 §0.1: o custo real por frame-com-input a 10 k entidades é **6,3 ms (pós-restore) a 23,8 ms (cena construída nesta sessão, ou depois de qualquer criação/reordenação)** — 38 % a **143 % de um frame de 16,6 ms**. O `PartialEq` do shell (E) é desprezível (0,076 ms); o clone de `VecScene`/`FlipDoc`/etc. não foi medido.

### 2. (B) Ordenar por `StableId` elimina o `canonicalize` — e elimina também a DFS
- `sort_by_stable_id` (mesma mecânica de remap de `parent`, chave = `u64` decodificado do blob): **0,007 / 0,088 ms** — 173× / 214× mais barato que o `canonicalize` em ordem de criação e 13× mais barato que o piso (A′). Registrar o `StableId` custa +0,19 ms no `world_to_snapshot` e +13 B/entidade (blob de 8 B + cabeçalho `type_id`+len).
- (B2) construir as linhas **direto** (query `(Entity, &StableId)` → `sort_unstable_by_key(sid)` → serializar; `parent` = `StableId` do pai via `ChildOf`) custa **4,34 ms @10k, menos que o próprio `world_to_snapshot` (5,09)**: a DFS por `Children` + o `BTreeMap<Entity,u32>` `index_of` valem ~0,75 ms e deixam de ser necessários quando o `parent` é `StableId` e não índice de linha. Isto é bump de `WorldSnapshot::VERSION` (1→2: `parent: Option<u32>` → `Option<u64>`) e de `PROJECT_SCHEMA`; o `state_hash` (replay/3-OS) passa a ser função da ordem por `StableId`, determinística se a alocação do id for um contador persistido no projeto (número que se CONTA, §5.0).
- [INF] Com `StableId` como chave, o empate entre linhas byte-idênticas do `canonicalize` (duas gêmeas com filhos diferentes — a ordem é arbitrária e o `parent` aponta para UMA delas) desaparece, porque a chave é única por construção.

### 3. (C) Captura incremental por change ticks: **0,27 ms @10k com nada mudado, 0,95 ms com 10 % mudado** — 25× a 88× abaixo do baseline
- Scan puro de ticks (iterar a query, `archetype().components()`, `get_change_ticks_by_id` + `is_changed`): **6 ns/entidade** (3 componentes presentes ⇒ ~2 ns por componente), 0,059 ms @10k.
- Captura completa com cache `BTreeMap<u64, {bytes, seen}>`: 0 mudadas 0,269 · 1 mudada 0,262 · 1 % 0,334 · 10 % 0,953 ms. Custo marginal por linha re-serializada ≈ **0,68 µs** ((0,953−0,269)/1000); o overhead fixo de ~0,21 ms acima do scan é o `get_mut` no `BTreeMap` por entidade + o `retain` (despawn) — [INF] um `Vec` indexado por `entity.index()` ou um cache em `Vec` ordenado cortaria a maior parte. Break-even com o rebuild ordenado (B2): ~(4,34−0,27)/0,00068 ≈ **6 000 linhas sujas por frame (60 %)** — abaixo disso o incremental ganha.
- 1ª captura (cache frio): 7,37 ms @10k (serializa tudo + insere no `BTreeMap`), uma vez por load/restore. Materializar o snapshot inteiro a partir do cache (clone dos bytes em ordem de `StableId`): 0,135 ms — para quem ainda quiser um snapshot completo; a pilha de undo deveria guardar **deltas** (ver §5).
- Spawn/despawn: detectado por pertença ao cache (novo `StableId` ⇒ serializa; `seen != stamp` ⇒ `retain` descarta). 1 % despawn + 1 % spawn @10k = 0,354 ms, com os 100 novos serializados.
- **Protocolo de tick que funcionou** [CODE]: na captura, `last_run = world.last_change_tick()` e `this_run = world.read_change_tick()` (a MESMA convenção que o bevy usa em `Mut`/`Ref`, `world/unsafe_world_cell.rs:380-381,437,554`); depois da captura, `world.clear_trackers()` (= `removed_components.update()` + `last_change_tick = increment_change_tick()`, custo 0,000 ms). `Tick::is_newer_than` é **estrita** (`ticks_since_system > ticks_since_insert`, `tick.rs:52-62`) ⇒ um componente carimbado com tick == `last_run` NÃO conta como mudado — logo o `clear_trackers` tem de rodar **depois da captura e antes de qualquer mutação do frame seguinte** (o fim de `post_frame_undo` serve). ⚠️ Hoje o PH2D **nunca** avança o tick (0 sítios — confirmado pelo agente irmão; o spike partiu de `Tick(1)` em tudo e a 1ª captura não depende de tick, só da ausência no cache). `check_change_ticks()` deve ser chamado no mesmo ponto; é no-op abaixo de `CHECK_TICK_THRESHOLD = 518 400 000` ticks (`change_detection/mod.rs:21`) ≈ 100 dias a 60 Hz.

### 4. Dois comportamentos do change detection que o desenho tem de absorver (ambos medidos)
- **(C′) Falso positivo — `get_mut` sem escrita conta como mudado**: 10 % de entidades tocadas por `world.get_mut::<Sprite>()` sem alterar valor ⇒ **1 000 linhas re-serializadas (byte-idênticas)**, 0,952 ms. [CODE] O `DerefMut` de `Mut<T>` carimba `changed = this_run` incondicionalmente. Cura barata no escritor: `DetectChangesMut::set_if_neq` (`change_detection/traits.rs:180`) ou `bypass_change_detection` (`:130`) para escritas derivadas; cura no leitor: comparar os bytes novos com o cache antes de marcar a linha como delta (a serialização já foi paga; a comparação é ~memcmp de 240 B). **Relevante para a hipótese**: o passe de sync mestre→instâncias, a timeline e qualquer writeback de física que escrevam em componentes do `SimWorld` por `Mut` sujam as linhas TODO frame — correto para o undo se o valor mudou (é estado), espúrio se não mudou.
- **(C″) Falso negativo — REMOVER um componente não muda tick de ninguém**: `remove::<Sprite>()` em 1 % ⇒ **0 linhas re-serializadas** (a entidade trocou de archetype, mas os componentes que sobram mantêm os ticks). `world.removed::<Sprite>()` enumera os 100 até o próximo `clear_trackers` (`world/mod.rs:1775`; o double-buffer vira em `clear_trackers`). A captura tem de (a) unir `removed_with_id(cid)` para cada `ComponentId` registrado ao conjunto sujo (58 iterações, baratas), **ou** (b) guardar o `ArchetypeId` por linha no cache e comparar com `eref.archetype().id()` (`archetype.rs:455`) — uma remoção sempre muda o archetype, e isto cobre add e remove com zero API extra [INF, não medido; custo esperado ≤ 1 compare/entidade]. Despawn NÃO tem esse problema (cai pelo `retain`).

### 5. (D) Memória: 239 B/entidade; delta a 1 % é 97,9× menor que o snapshot
- Snapshot/cache inteiro @10k = 2,40 MB (2,27 MB sem `StableId`); delta a 1 % = 24,5 KB (100 linhas × (236 B + 8 B de chave)); a 10 % = 247 KB.
- [INF] Pilha de undo `UNDO_CAP = 256` a 10 k: snapshots inteiros ≈ 614 MB; passos-delta a 1 % guardando **antes+depois** (necessário para reverter sem re-derivar) ≈ 2 × 24,5 KB × 256 ≈ **12,5 MB** (49×); a 10 %, ≈ 126 MB. O `PartialEq` por frame (E, 0,076 ms) é substituído por "delta vazio?" (grátis).

### 6. O que isto diz sobre a hipótese (instância MATERIALIZADA + link por StableId + sync por tick + undo incremental)
1. **O custo do undo deixa de ser o teto do número de entidades.** Com captura incremental, o frame-com-input custa ~0,27 ms + 0,68 µs por linha suja @10k (contra 6,3–23,8 ms hoje). Multiplicar entidades por materialização paga: no scan (6 ns/ent.), na 1ª captura (0,74 µs/ent., uma vez), e no delta dos frames em que as instâncias são re-sincronizadas.
2. **O preço real da materialização no undo é o delta de uma edição no mestre**: editar um mestre com 100 instâncias × 10 peças suja 1 000 linhas ⇒ ≈ 0,95 ms + 247 KB por passo (antes+depois ≈ 0,5 MB). [INF] Alternativa de desenho não medida: a captura pular linhas cujos componentes são inteiramente derivados do link (re-deriváveis no restore pelo mesmo sync) — troca bytes por um sync extra no undo; a decisão é de produto/arquitetura, não de medição.
3. **O `StableId` como chave de linha é o que apaga o `canonicalize`** (B), e como `parent` apaga a DFS (B2). Os dois exigem bump de `WorldSnapshot::VERSION` e `PROJECT_SCHEMA` e tornam o `state_hash` dependente da alocação determinística do id — o mesmo `StableId` que a hipótese já precisa para `{master_root, master_piece}` e para migrar `stable_name_id` (joints/roldanas/`WireId`).
4. **O sync por change-tick e o undo incremental partilham o mesmo instrumento** (scan de 6 ns/ent.) e as mesmas duas armadilhas do §4: o sync deve escrever nas instâncias com `set_if_neq` (senão o undo vê 100 % das instâncias sujas todo frame) e deve ler remoções de componentes do mestre por `removed_with_id`/archetype (senão "tirar um `RigidBody` da peça do mestre" nunca propaga). [INF] Ordem topológica mestre→variant→instância é uma ordem sobre `StableId`s de `master_root`, computável uma vez por mudança estrutural.
5. **Refutação parcial do doc 04 §0.1**: o fato "6,89 ms" era o piso, não o teto; e o fato "qualquer modelo que multiplique entidades bate neste teto primeiro" **cai** se o undo for incremental — o limite passa a ser o tamanho do delta de uma edição propagada, que é função do fan-out (instâncias × peças), não da contagem total.

## api
- bevy_ecs 0.18.1 vendorizado em ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src (MIT/Apache — leitura de código permitida)
- World::last_change_tick(&self) -> Tick — world/mod.rs:3023 (usado como `last_run`)
- World::read_change_tick(&self) -> Tick — world/mod.rs:3001 (usado como `this_run`; `change_tick(&mut self)` em :3011 é a variante não-atômica, confirmada, não usada)
- World::clear_trackers(&mut self) — world/mod.rs:1599-1602 (= removed_components.update() + last_change_tick = increment_change_tick()); increment_change_tick(&mut self) -> Tick — :2989
- World::removed::<T>(&self) -> impl Iterator<Item = Entity> — world/mod.rs:1775 (válido desde o último clear_trackers); removed_with_id(ComponentId) — :1785
- World::entity(Entity) -> EntityRef; EntityRef::archetype(&self) -> &Archetype — world/entity_access/entity_ref.rs:67
- Archetype::components(&self) -> &[ComponentId] — archetype.rs:530 (iter_components em :538; id() -> ArchetypeId em :455, citado para a cura do falso negativo, não usado)
- EntityRef::get_change_ticks_by_id(&self, ComponentId) -> Option<ComponentTicks> — world/entity_access/entity_ref.rs:141 (USADA); EntityRef::get_change_ticks::<T>(&self) -> Option<ComponentTicks> — :129 (confirmada, não usada: a variante by_id sobre o archetype cobre qualquer componente registrado sem conhecer o tipo)
- ComponentTicks { added: Tick, changed: Tick } — change_detection/tick.rs:137-143; ComponentTicks::is_changed(&self, last_run: Tick, this_run: Tick) -> bool — tick.rs:156 (delega em Tick::is_newer_than, tick.rs:52-62, comparação ESTRITA com clamp MAX_CHANGE_AGE)
- Caminhos de import: bevy_ecs::change_detection::{Tick, ComponentTicks} (re-export `pub use tick::*` em change_detection/mod.rs:10); bevy_ecs::component::ComponentId (component/info.rs:179)
- World::get_mut::<T>(Entity) -> Option<Mut<T>> — world/mod.rs:1340 (o DerefMut de Mut carimba `changed = this_run` mesmo sem escrita — medido em C′)
- DetectChangesMut::set_if_neq — change_detection/traits.rs:180; bypass_change_detection — :130; set_changed — :99 (citados como cura do falso positivo; não usados no spike)
- Constantes: CHECK_TICK_THRESHOLD = 518_400_000 — change_detection/mod.rs:21; MAX_CHANGE_AGE = u32::MAX - (2*CHECK_TICK_THRESHOLD - 1) — :26; World::check_change_ticks(&mut self) -> Option<CheckChangeTicks> — world/mod.rs:3153
- Também usados (API corrente, sem linha): World::spawn, World::despawn, EntityWorldMut::remove::<T>(), World::query::<(Entity, &StableId)>() / QueryState::iter(&World), EntityRef::get::<ChildOf>()
- Lado PH2D (verificado abrindo os arquivos): ph2d_ecs::scene::world_to_snapshot (crates/ph2d-ecs/src/scene/save.rs:114-192), EntitySnapshotRow/WorldSnapshot (:32-48, VERSION=1 em :53), ComponentRegistry::{register, iter, get_by_name} com campos públicos `serialize`/`type_id` de ComponentTypeEntry (crates/ph2d-ecs/src/scene/registry.rs:103-107,132), canonicalize copiado verbatim de shells/desktop/src/undo.rs:152-180, Sprite::atlas(key, size, tint) (crates/ph2d-render/src/sprite/component.rs:269), register_render_components (crates/ph2d-render/src/registry.rs:14)

clean: True

## caveats
- Carga da máquina: a 1ª rodada correu a loadavg 5,5, as duas seguintes (incluindo a tabela reportada) a ~15 — acima do teto de confiança do §5.0 do CLAUDE.md. As medianas ficaram dentro de ±5 % entre as 3 rodadas e os mínimos dentro de ±3 %, então as RAZÕES são sólidas; os valores absolutos podem estar ~5-10 % inflados.
- Mundo plano: 10k raízes sem `ChildOf`, 3 componentes por entidade (Transform + Name + Sprite) + o StableId local. Cenas reais têm mais componentes por entidade (RootOrder, Visibility, Vec*, física) ⇒ linhas maiores, mais ticks por entidade (o scan custa ~2 ns por componente presente) e uma DFS real no `world_to_snapshot`. Os números de (B2)/(C) com hierarquia não foram medidos.
- A discrepância com o doc 01 §7.3 (canonicalize 2,08 vs 18,7 ms @10k) é explicada pelo regime de ordenação da entrada (A′ reproduz ~o piso); NÃO determinado qual ordem de spawn o spike apagado do doc 01 usou. O doc 01/04 deve registrar os DOIS regimes (6,3 / 23,8 ms).
- (A) mediu contra um registro de 58 tipos (ecs+render) sem o StableId registrado; (B)/(C) contra 59 (com StableId). A diferença (+0,19 ms no world_to_snapshot) está na tabela.
- Harness caseiro (Instant, 25 iterações, mediana/mín, 3 de aquecimento), sem criterion; não há bench permanente de captura no repo (confirmado pelo agente irmão) — se o número for virar gate, precisa de um bench em `crates/ph2d-ecs` com `canonicalize` movido para lá.
- O cache incremental guarda uma cópia inteira dos bytes das linhas (2,4 MB @10k) — mesma ordem de grandeza de UM snapshot; o benefício de memória está na pilha de undo (deltas), não no cache.
- A estimativa de memória da pilha de undo com deltas (12,5 MB @1 %) e o break-even de ~60 % de linhas sujas são [INF] derivados dos números medidos, não medidos diretamente.
- Não medido: o custo de unir `removed_with_id` por ComponentId registrado ou de comparar `ArchetypeId` por linha (a cura do falso negativo C″); custo esperado desprezível (≤1 compare/entidade ou 58 iteradores vazios/frame).
- `cargo test` gerou 1 warning no spike (campo `components` de `SidRow` nunca lido — o black_box segurava a struct inteira); irrelevante para a medição. O binário compilado em target/release/deps/zz_spike_incremental_capture-* também foi apagado.
- `git status --short` após o rm devolve apenas `?? docs/Components/` — nenhum arquivo de produção foi tocado em momento algum (o spike viveu só em crates/ph2d-ecs/tests/).