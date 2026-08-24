# Decisão — modelo de objeto composável e sistema de assets (Fases C e D) — **v2, reconsiderada**

> **Histórico deste doc:** a v1 (2026-08-21, tarde) recomendou uma instância *derivada* (um objeto só,
> desenho derivado por quadro) e aceitou três limitações — sem física na cópia, sem aninhamento na v1,
> e o custo do desfazer "não piora, mas não se resolve". **O Enio vetou as três** (2026-08-21, noite):
> *"o que buscamos é o estado da arte, a possibilidade infinita, sem limitações… Encontre um modo da
> cópia ser física… Cópia dentro de cópia: tudo deve ser possível… O problema de velocidade deve ser
> resolvido."*
>
> **Esta v2 responde a isso com pesquisa nova, medição nova e verificação adversarial.** Método
> (2026-08-21, noite): 6 agentes de pesquisa (Unity/Godot em código e doc · flecs/Bevy em código ·
> Houdini/USD/Rive em doc · undo incremental — Blender por notas de release, bevy_ecs em código ·
> endereçamento de override aninhado · fatos do código do PH2D) → 1 agente de **medição** (spike
> descartável, apagado, árvore limpa) → **3 refutadores** adversariais sobre as três afirmações-chave.
> **Os três refutaram a hipótese como enunciada e devolveram a versão reparada** — as condições que
> eles impuseram **são** a arquitetura abaixo. Evidência integral em
> [`pesquisa/instancias_2026-08-21/`](pesquisa/instancias_2026-08-21/) (10 arquivos, com `file:line` e URL
> em cada afirmação).
>
> **Este doc PARA no fim.** A escolha é do Enio. ADR e plano de implementação **não foram escritos**.

---

## 📍 Índice

| § | assunto |
|---:|---|
| **§0** | O que o Enio pediu × o que a pesquisa respondeu |
| **§1** | Os fatos novos (medidos ou lidos no código) que mudam a decisão |
| **§2** | ⭐ A arquitetura: **MATERIALIZADA E VIVA** — o modelo, peça a peça |
| **§3** | As 12 perguntas da Fase C, respondidas de novo (só o que mudou) |
| **§4** | As candidatas reconsideradas, e por que a v1 errou |
| **§5** | O que ainda se perde — honesto, e menor |
| **§6** | Sequenciamento, com o critério de pronto de cada fase |
| **§7** | A trilha de verificação: as 3 refutações e onde cada condição entrou |

---

## §0 — O que o Enio pediu × o que a pesquisa respondeu

| Pedido (verbatim) | Resposta | Onde está a prova |
|---|---|---|
| *"duplica e a duplicata é um objeto independente; as instâncias são vinculadas completamente"* | ✅ **Dois verbos, dois resultados.** `Duplicar` = cópia profunda com ids novos e **sem vínculo**. `Instanciar` = objetos reais **ligados peça a peça** ao mestre; editar o mestre muda todas, ao vivo. É o que Unity, Godot, Houdini e Figma fazem — e é o que o módulo vetorial do PH2D já faz, **mas só para desenho**. | §2.1; [propagacao_unity_godot §(e)](pesquisa/instancias_2026-08-21/propagacao_unity_godot.md) |
| *"Encontre um modo da cópia ser física"* | ✅ **A instância é feita de objetos REAIS.** Cada peça é uma entidade comum com seus componentes; a ponte da física a vê por *query*, como vê qualquer outra — **lido no código: zero caso especial** (`bridge.rs:84-89`). Uma instância de ragdoll cai, colide, tem juntas. O que o v1 errou foi tornar a instância *um objeto só* com desenho derivado. | §2.5; [fatos_de_codigo §(2)](pesquisa/instancias_2026-08-21/fatos_de_codigo_ph2d.md) |
| *"Cópia dentro de cópia: tudo deve ser possível"* | ✅ **Aninhamento sem limite de profundidade**, na v1. A única recusa é **ciclo** (A contém instância de B que é-a A). Unity levou 13 anos porque inventou a UI "a qual mestre aplico?" — a resposta documentada deles (override vive no mais externo; *apply to inner* reverte o externo) é adotada como está. | §2.6; [enderecamento §3.7, §5.1](pesquisa/instancias_2026-08-21/enderecamento_override_aninhado.md) |
| *"O problema de velocidade deve ser resolvido"* | ✅ **Medido: 23,8 ms → 0,27 ms a 10 mil objetos (88×).** A captura do desfazer passa a custar o tamanho da *edição*, não o tamanho do *mundo*. E a pilha de undo cai de ~614 MB para ~12,5 MB. É pré-requisito, não otimização: materializar instâncias multiplica entidades, e esta é a conta que o paga. | §1.1, §2.7; [medicao](pesquisa/instancias_2026-08-21/medicao_captura_incremental.md) |
| *"o padrão ouro, o estado da arte… o menos limitado, o mais poderoso e intuitivo para artistas"* | ✅ **A arquitetura EXCEDE as sete referências num ponto só** — override por campo de peça interna com propagação viva nos campos não tocados. Houdini evita isso desligando tudo (unlock); USD proíbe (proxy read-only); Rive só permite o que o mestre expôs; Unity/Godot têm o campo mas propagam por *reinstanciar tudo* (Godot apaga o histórico de undo ao fazê-lo). **Nenhum ECS permissivo faz sync incremental no mesmo mundo.** | §4; [houdini_usd_rive §5](pesquisa/instancias_2026-08-21/houdini_usd_rive.md), [flecs_bevy §K](pesquisa/instancias_2026-08-21/flecs_bevy_internals.md) |

---

## §1 — Os fatos novos que mudam a decisão

### §1.1 — ⚠️ O custo do desfazer era MAIOR do que eu disse — e a cura está medida

[Medição, spike apagado](pesquisa/instancias_2026-08-21/medicao_captura_incremental.md) — release, 25 iterações, mediana, n = 10.000 entidades com `Transform+Name+Sprite`:

| O que | Custo por quadro-com-input | Nota |
|---|---:|---|
| **Hoje, cena recém-criada** (`world_to_snapshot` 5,09 + `canonicalize` 18,73) | **23,8 ms** | **143 % de um quadro.** O `canonicalize` constrói a chave (`Vec<u8>` de ~230 B) **dentro do comparador** do sort: ~266 k alocações a 10 k |
| Hoje, logo após um restore (entrada já ordenada) | 6,27 ms | é o número do doc 01 §7.3 — era o **piso**, não o teto |
| **Ordenar por `StableId`** em vez de por bytes | **0,088 ms** | 214× mais barato que o `canonicalize` |
| Construir as linhas direto por `StableId` (sem DFS, sem `index_of`) | 4,34 ms | menos que o próprio `world_to_snapshot` |
| ⭐ **Captura INCREMENTAL por change ticks — nada mudou** | **0,269 ms** | scan = 6 ns/entidade |
| ⭐ 1 entidade mudou | 0,262 ms | delta = 244 B |
| ⭐ 1 % mudou (100) | 0,334 ms | delta = 24,5 KB |
| ⭐ 10 % mudou (1.000) | 0,953 ms | 0,68 µs por linha suja |
| Pilha de undo (256 passos) a 1 % de mudança | **~12,5 MB** | hoje ~614 MB (snapshot inteiro por passo) |

**Duas armadilhas do relógio de mudanças, ambas medidas e ambas com cura de uma linha:**
- **Falso positivo:** `get_mut` sem escrever **carimba** mudança (1.000 linhas re-serializadas, bytes idênticos). Cura: `set_if_neq` em quem escreve, e **comparar bytes** antes de emitir delta — o tick é **pré-filtro, nunca verdade**.
- **Falso negativo:** **remover um componente não carimba ninguém** (`remove::<Sprite>` em 1 % ⇒ 0 linhas). Cura: guardar o `ArchetypeId` por linha (toda remoção muda o archetype), ou ler `removed_with_id` **antes** do `clear_trackers`.

⚠️ **E o fato que ninguém sabia:** o PH2D **nunca avança o change tick do bevy** — zero chamadas a `clear_trackers`/`increment_change_tick` em todo o repo. A change detection está dormente; hoje todo componente tem `changed == Tick(1)`. O primeiro passo do plano é **um `clear_trackers()` por CAPTURA** (não por quadro — ver §2.7).

### §1.2 — O que o código do PH2D é, de fato (lido, com `file:line`)

[fatos_de_codigo_ph2d.md](pesquisa/instancias_2026-08-21/fatos_de_codigo_ph2d.md):

1. **A instância vetorial de hoje é UMA entidade** (retângulo-suporte + `VecInstance`), geometria derivada em `LiveGeometry`; as peças do **mestre** são entidades reais. ⚠️ **Aninhamento NÃO renderiza** no modelo derivado: `cook_one` lê `src.cooked()`, nunca o `live` — uma instância dentro do mestre vira um retângulo na cópia (`instance_live.rs:149-152`). A única recusa é `main == at`; profundidade **capada em 64** sem erro.
2. **`Detach` materializa só GEOMETRIA** — as peças destacadas perdem **todo** componente paramétrico (`VecShape`, `VecFilter`, `RigidBody`…); só a curva cozida viaja (`vec_component_edit.rs:322-381`). Não serve de molde.
3. **`Duplicate` é RASO:** copia `Transform+Sprite+Name+ChildOf`, sem filhos, sem os outros 87 componentes (`render_loop/hierarchy.rs:171-238`). Cópia profunda de subárvore **não existe**; o substrato (`extract_component_snapshot` + `insert_from_bytes`) existe com zero consumidores.
4. **A ponte da física é por QUERY a cada dispatch** (`BodyQuery = (Entity,&RigidBody,&Collider,&Transform)`, pose de mundo pela cadeia `ChildOf`, `bodies: BTreeMap<Entity,_>` só de runtime). **Peças materializadas com `RigidBody` funcionam sem tocar a ponte.** O que quebra é a metade por **NOME**: `PhysicsJoint.body_a/b`, `PulleyWheel.rope/body` e o `WireId` da timeline são `stable_name_id`; a cópia recebe nome `" (1)"` ⇒ **a junta da cópia prende os corpos do MESTRE**.
5. **`stable_name_id` tem DUAS famílias de consumidores** (não três — correção ao doc 01 §1.3: `VecLabel.host` é `VecPathId`): **timeline** (`binding.rs:25`, `timeline_persist.rs:38`, `frame_solve.rs:139/218/251`, `persist.rs`) e **física** (`joint.rs:117-121`, `components/rope.rs:99/146/405`, `bridge/joints.rs:152/315`, `bridge/rope.rs:130`, `joint_group.rs:139` + 12 sítios de inspector). ~26 linhas de produção em 13 arquivos.
6. **A ordem de irmãos NÃO é dado persistido.** Nenhum componente a guarda (só `RootOrder` para raízes); o restore reconstrói `Children` na ordem das linhas, que o `canonicalize` ordena por conteúdo ⇒ **reordenar irmãos não é desfazível nem sobrevive a um restore** (classe BUGS #15, pré-existente).
7. **Não existe bench de captura** no repo; `canonicalize` é privado na shell; o bench tem de mirar `world_to_snapshot + canonicalize` em `crates/ph2d-ecs`.

### §1.3 — O que as engines fazem, de verdade (lido em código onde permissivo)

- **Unity [DOC]:** instâncias são objetos reais; o ficheiro de cena guarda só `PrefabInstance{ m_SourcePrefab, m_Modifications[], m_RemovedComponents, m_AddedGameObjects… }` e **toda carga é um merge asset+diff**; propagação = *re-merge batched ao fim do frame* no editor (`MergePrefabInstance`); override chaveado por **`(fileID do objeto no asset, propertyPath)`** — renomear/reordenar não quebra; apagar vira **"unused override", que a Unity NUNCA limpa sozinha** (*"because you might have moved the object… temporarily or in error"*); nesting: override vive no **mais externo**, *"Apply to Prefab 'Vase'"* × *"Apply as Override in Prefab 'Table'"*, e aplicar no interno **reverte** o externo para o valor não mudar; `Unpack` = 1 nível, `Unpack Completely` = todos; **duplicar instância gera instância**. `Rigidbody` não serializa velocidade ⇒ estado de solver nunca é override.
- **Godot [CODE, MIT]:** overrides **NÃO são armazenados — são derivados por diff no `pack()`** contra a pilha de `SceneState`s, chaveados por `NodePath + propriedade`; alvo sumido = **descartado em silêncio com WARN**; desde **4.6** (PR #106837, 2025-10-06, fecha 7 issues desde 2018) há `unique_id` por nó **só como fallback** do caminho. Propagação = **reinstanciar a cena inteira** (`reload_scene_from_memory`) — e **apaga o histórico de undo da aba**. `Make Local` = 1 nível. `RigidBody2D.linear_velocity` é armazenável e **pode** ser override.
- **flecs [CODE, MIT]:** tem **dois storages com semânticas opostas**: no `ChildOf` a peça instanciada é **cópia byte a byte sem vínculo**; no `Parent` (o recomendado) cada peça carrega **`(IsA, filho_do_prefab)`** — o vínculo real — e é endereçada por **índice posicional + validação**, com erro explícito se a instância foi reestruturada. Componentes `Inherit` chegam ao vivo (storage partilhado); `Override` fica **stale por desenho**; e mudar os filhos de um prefab já instanciado **ABORTA** (*"cannot change children of prefab after it has been instantiated"*). **Propaga eventos pela `IsA`, nunca dados.**
- **Bevy 0.18/0.19 [CODE]:** `bevy_scene` liga instância↔asset por `SceneInstance(uuid v4)` + mapa privado; hot-reload = **despawn total + respawn com ids novos**, edições perdidas, sem override. O BSN 0.19 tem patch por campo e herança, mas resolve **só no spawn**. Nenhuma crate do ecossistema implementa instância vinculada com override — as cinco que tentaram estão mortas.
- **Houdini [DOC]:** *"New digital asset instances are normally **locked**… they **automatically update when the asset's definition changes**. An unlocked instance is editable, does not update"*; *"When you edit the nodes inside an asset, you're editing the definition **in memory**. Other instances of the same asset will get the same changes."* `Match Current Definition` descarta o conteúdo e **preserva os valores de parâmetro**. *"Editability does not 'bubble up'"* através de nesting.
- **OpenUSD [DOC]:** *"editing scene description via instance proxies… is not allowed"* — só a **raiz** da instância é editável; **RB.005**: corpo rígido só na raiz, *"because instanceable prims prohibit changes to their internal prims"*; peça interna que precise ser corpo próprio força *deinstance*. Nesting ilimitado, **medido**: 44.408 prims → 1.711 → 1.482. *"The higher you move your instancing up… at the cost of authoring flexibility."* `instanceable=false` **não corta a referência** — o único "detach" que continua a receber.
- **Rive [DOC+CODE MIT]:** só a interface que o mestre expôs é editável na instância; `nest()` recursivo **sem guarda de ciclo** no runtime; `Detach` só para Libraries e irreversível.
- **Blender [DOC-indireto, notas de release]:** o *undo speedup* 2.83 **só relê os data-blocks mudados** (diff de chunk vazio = preservar no lugar); o passo 4 *"write only changed datablocks"* **nunca foi feito** porque o Blender **não confia nas tags de mudança**; o push só baixou 2–5× via *implicit sharing* (4.2, 2024). **O PH2D tem o que o Blender não tem: ticks impostos pelo ECS em toda escrita rastreada.** Library overrides: *resync* **automático no load**, conta os overrides apagados, **desfazível antes de salvar**.

---

## §2 — ⭐ A arquitetura: **MATERIALIZADA E VIVA**

> Uma frase: **uma instância é feita de objetos reais, cada um ligado por id à peça do mestre que o
> originou; o mestre é um asset; a propagação mestre→instâncias é um passe por quadro dirigido pelo
> relógio de mudanças do ECS; o override é esparso, por campo, chaveado por id, e vive na raiz da
> instância; o desfazer captura só o que mudou.**

### §2.1 — Vocabulário (o que aparece na tela)

| Verbo / coisa | O que é | Precedente |
|---|---|---|
| **Duplicar** (Ctrl+D) | cópia profunda, ids novos, referências internas remapeadas, **sem vínculo** | o pedido do Enio; Unity *Unpack Completely* sobre uma cópia |
| **Criar componente** | a seleção vira **mestre** (asset na biblioteca) e **uma instância fica no lugar dela** | Unity *Create Prefab* (o objeto da cena vira instância); Rive *Convert to Component* |
| **Instanciar** / *Place* | põe uma instância: objetos reais, ligados peça a peça | Unity/Godot/flecs-`Parent` |
| **Mestre** | o template; **vive na biblioteca de assets**, não simula, não anima por si; abre em *Editar mestre* (em contexto, como o Unity Prefab Mode *In Context*) | Unity prefab asset; Houdini definition; USD prototype |
| **Peça** | cada entidade de um mestre (a raiz e toda a sub-árvore) | Unity *corresponding object*; flecs *prefab child* |
| **Override** | *"nesta instância, este campo desta peça vale X"* — esparso, por campo | Unity `PropertyModification`; Figma override; `VecInstance.overrides` hoje |
| **Destacar** (1 nível) / **Destacar tudo** | corta o vínculo; os objetos continuam iguais, só deixam de seguir | Unity *Unpack* / *Unpack Completely*; Godot *Make Local* |
| **Aplicar ao mestre** | empurra um override (ou todos) para dentro do mestre | Unity *Apply*; vetor `Update Main` hoje |
| **Redefinir** | limpa overrides; volta a ser o mestre | Blender *Reset*; vetor `Reset` hoje |
| **Trocar mestre** | religa a instância a outro mestre (variant ou não), com re-key determinístico quando aparentado | vetor `Swap Main` hoje; Unity `ReplacePrefabAssetOfPrefabInstance` |
| **Overrides sem alvo** | lista de overrides cujo alvo sumiu do mestre — **nunca apagados sozinhos**, removidos por gesto | Unity *Unused overrides* |

### §2.2 — Modelo de dados (componentes novos, todos registrados ⇒ persistem e desfazem de graça)

```
StableId(u64)            ← TODA entidade editável; contador monotônico do documento (fora do undo)
SiblingOrder(u32)        ← todo filho (gêmeo do RootOrder); reorder vira dado, desfazível, overridável
MasterRoot               ← marcador na raiz de um mestre
MasterPiece              ← marcador em TODA peça (raiz + sub-árvore), mantido pelo sync
InstanceOf { master_root: StableId, master_piece: StableId }   ← em toda peça de uma instância
ObjectInstance { overrides: BTreeMap<OverrideKey, Bytes> }      ← SÓ na raiz da instância

OverrideKey = (path: [StableId…]   // só para peças sob instância ANINHADA (prefixo)
              , piece: StableId     // id da peça NO ESCOPO DO MESTRE (como VecInstance.sub hoje)
              , type_id: u64        // blake3(nome canônico)[..8] — já existe
              , field_id: u16)      // DECLARADO no descritor, append-only — NUNCA posicional
```

- **Tudo é `Ord` total de inteiros de largura fixa** ⇒ a ordem do `BTreeMap` é a ordem dos bytes ⇒ duas instâncias logicamente iguais são byte-iguais (a lei que o `VecInstance::set` já cumpre e o `canonicalize` exige).
- **Zero bits de `Entity`, zero nomes, zero índices** na chave e no valor.
- ⚠️ **`field_id` posicional foi REFUTADO** ([refutação 3 §1-b](pesquisa/instancias_2026-08-21/refutacao_3_override_aninhado.md)): postcard é posicional — trocar `Collider::Ball{radius}` por `Cuboid{hx,hy}` faria o override de `radius` **re-alvejar `hx` em silêncio**. Com id declarado, o override vira "sem alvo" (detectável), e os outros continuam certos. É o `FormerlySerializedAs` do Unity de graça.
- **O `ComponentDesc`** (o descritor do C1, que a v1 já pedia para o inspector) **ganha três papéis a mais**: (a) `field_id` estável por campo; (b) **política de propagação** por (tipo, campo); (c) **quais campos são referências** (`Ref<StableId>`, `Ref<VecPathId>`) para remap. **Um descritor serve inspector, override, remap e propagação** — é a convergência que torna o catálogo de ~150 componentes barato.

### §2.3 — O passe por quadro (editor) — a ordem é load-bearing e tem gate

```
[1] gestos do quadro mutam entidades (gizmo, inspector, timeline…)   → ticks carimbados
[2] CAPTURA DE OVERRIDES  (peças de instância com tick novo)
      para cada campo: bytes(peça) ≠ bytes(mestre resolvido)?  → entrada em ObjectInstance.overrides
      (pula campos RuntimeOwned; pula campos InstanceLocal)
[3] SYNC mestre→instâncias  (mestres com tick novo; ordem TOPOLÓGICA; recusa ciclo)
      para cada instância, peça, campo NÃO overridado e NÃO RuntimeOwned:  set_if_neq(valor do mestre)
      peça nova no mestre → materializa em toda instância (id novo, SiblingOrder, Link, refs remapeadas)
      peça removida do mestre → despawn nas instâncias (override "sem alvo" fica, inerte)
[4] post_frame_undo: CAPTURA INCREMENTAL (§2.7) → clear_trackers() UMA vez
```

⭐ **Por que overrides são capturados por DIFF e não por uma "porta"** ([refutação 3 §2.3](pesquisa/instancias_2026-08-21/refutacao_3_override_aninhado.md)): com peças materializadas, **todo escritor existente escreve a ENTIDADE** — gizmo → `Transform`, inspector → `SetComponent`, timeline → `Transform` por quadro, readback da física → `Transform` por tique, *Arrange* → `ZIndexOverride`. Rotear cada gesto por uma porta é *"um guard que enumera os seus consumidores apodrece"* ([memória](../../project-memory/feedback_a_condition_that_enumerates_its_readers_rots.md)); sobrescrever a peça a partir do mestre perde a edição. O diff é a única saída honesta — e é o que o Unity faz (*"At the end of the frame Unity diffs the state"*). O risco do Godot (*"o editor cria overrides que ninguém pediu"*, #111807) vem de **escritores de runtime** — e é exatamente o que a política por tipo (§2.4) exclui.

### §2.4 — Política de propagação por (tipo, campo) — o `OnInstantiate` do flecs, no descritor

| Política | Significado | Exemplos |
|---|---|---|
| **Propaga** (default) | segue o mestre; overridável por campo | quase tudo: `Sprite.tint`, `Collider.shape`, `VecShape.*`, `RigidBody.kind`… |
| **Local da instância** | nunca propaga; sempre por instância; **não conta como override** | `Transform` da **raiz** da instância, `Name` da raiz, `RootOrder`, `SiblingOrder` da raiz — os *"default overrides"* do Unity (`IsDefaultOverride`) |
| **Do runtime** | nem propaga nem é capturado como override; o dono é um sistema | `Transform` de peça cujo `pose_owner ∈ {Solver, Player}` (ponte da física, `pose_owner.rs:209-220`); `Transform` de peça dirigida por binding da timeline |

⚠️ **"Do runtime" é por ENTIDADE, com predicado vindo da ponte** — a ponte exporta **uma** porta `pose_owner(entity)` (ela já a tem internamente); o sync e a captura a consultam, nunca re-derivam ([refutação 1, condição (b)](pesquisa/instancias_2026-08-21/refutacao_1_sync_determinismo.md)). Sem isto, o sync escreveria na célula que o solver possui, e o readback marcaria um override por tique.

### §2.5 — Física, sem limitação — e as três condições que a ponte impõe

A instância **é** física porque as peças são entidades comuns. As condições vêm do código da ponte, lido ([refutação 1](pesquisa/instancias_2026-08-21/refutacao_1_sync_determinismo.md)):

1. **O mestre não simula.** As 5 `QueryState`s cacheadas da ponte (`BodyQuery`/`PartQuery`/`JointQuery`/`SurfaceQuery`/`NoClingQuery`, `bridge.rs:84-127`) ganham `Without<MasterPiece>`; a resolução de nomes da timeline também. **É uma edição na ponte**, e o bin `physics_ecs_c9` ganha **uma lane com mestre + instância** no hash 3-OS — hoje o gate não cobre o cenário (*"verdadeiro por vacuidade"*). Sem isto, um mestre com `RigidBody` no mesmo `World` seria simulado, o readback carimbaria o `Transform` dele por tique, e o sync propagaria a **pose simulada do mestre** a todas as instâncias; pausado, o `settle` (`hold.rs:98-110`) **teleportaria** todas as instâncias para a pose do mestre a cada quadro.
2. **Referências internas são remapeadas em TODA propagação**, não só na instanciação — porque o sync reescreve o componente sempre que o mestre muda. `PhysicsJoint.body_a/b`, `PulleyWheel.rope/body`, `VecTextPath/VecPatternPath/VecEnvelope.path`, `VecLabel.host`, bindings da timeline: todos declarados como `Ref<…>` no descritor; o sync traduz mestre-escopo → instância-escopo pelo mapa de links. **Gate obrigatório:** *"a junta da instância prende os corpos da INSTÂNCIA"*. ⚠️ **Isto pressupõe a migração `stable_name_id → StableId` ANTES de materializar** — não é um "depois".
3. **Config de física propagada chega ao solver no próximo Reset** (`bodies.rs:169-173` só re-descreve corpos `at_rest`). É comportamento de produto a **declarar** na v1 — e um item do plano para levantar (re-descrever mid-play preservando velocidade). O Unity tem o mesmo limite para mudanças de forma de collider.

**Entradas por TIQUE (pose cinemática de curva, parâmetro de joint keyframado) NÃO passam pelo sync** — o sync é por quadro; a ponte dirige por tique dentro do laço (`dispatch.rs:156-181`, a cura do bug de 3,4 cm do `rewind.rs:82-91`). A peça da instância tem de ter **o próprio binding** para ser dirigida por tique. Hoje a timeline é documento-único com bindings por entidade ⇒ **um clipe autorado no mestre NÃO propaga às instâncias na v1** — a cura nomeada é o **`SequencePlayer`** (item 19 do [TOP-20](00_levantamento_componentes.md)): um componente que referencia um clipe por asset **é** componente, logo propaga. Esta é a única limitação real de física/animação, e tem degrau nomeado (§5).

### §2.6 — Aninhamento e variantes — sem limite, com as regras escritas

- **Instanciação recursiva**: um mestre pode conter instâncias; instanciá-lo materializa tudo, em **ordem de travessia por `StableId`** (determinística, replay 3-OS).
- **Sync topológico**: mestres ordenados por dependência (B antes de V-que-é-a-B antes de instâncias de V), desempate por `StableId`. **Ciclo é recusado no gesto** (`Place` que fecharia ciclo é erro com mensagem), não capado — hoje só `main == at` é recusado e a profundidade é capada em 64 **sem erro** (64 níveis de trabalho por quadro).
- **Override vive na raiz mais externa de cada documento** (na cena: a instância raiz; dentro de um mestre: o nó de instância aninhada). Precedência **externo > interno > mestre**, sem empate (Unity, Figma, USD `Local`). A visão por nível é uma *range query* pelo prefixo da chave.
- **"Aplicar ao mestre interno" apaga o override do mesmo campo em todo mestre intermediário** — senão é um no-op visível (V sombreia B). É a regra documentada do Unity (*"this override in the 'Table' Prefab is reverted at the same time"*). Consequência a declarar: o verbo muta dois mestres.
- **Destacar 1 nível redistribui** os overrides com prefixo `[N,…]` para a raiz de N, que passa a ser a mais externa. Roda implicitamente em *Destacar*, em *apagar aninhada* e em *Trocar mestre*.
- **Dois tipos de variante, ambos com os verbos que existem:**
  - **Variante irmã** (Figma; o `vec_variants.rs` de hoje): mestres irmãos sob o mesmo pai, eixos lidos do `Name` (`Size=Small, State=Idle`), **zero schema**. Trocar = *Trocar mestre* restrito aos irmãos, com **re-key determinístico** (irmãos partilham o base — caso 2 do §3.7 do endereçamento).
  - **Variante derivada** (Unity/flecs `IsA`): um mestre cuja **raiz é uma instância** do base — *Instanciar* + *Criar componente*. Mudanças no base chegam ao variant e às instâncias dele (sync topológico). Overrides do variant vivem no nó-raiz dele. Trocar variant↔base **insere/remove um nível** na chave, deterministicamente — *"nenhum outro sistema consegue isso, porque nenhum tem chave mestre-relativa com caminho"*.
- ⛔ **Trocar para mestre NÃO aparentado**: só heurística. Oferecer os três modos do Unity (`Nenhum` default · `Por nome` só com nomes únicos · `Por hierarquia`) + relatório do que foi mantido/descartado; o resto vai para *Overrides sem alvo*. **Nunca automático** — com nomes duplicados o resultado não é reproduzível (HR-5), e o Unity o diz com todas as letras.

**O que sobrevive a cada operação no MESTRE** (a tabela que decide, [enderecamento §2](pesquisa/instancias_2026-08-21/enderecamento_override_aninhado.md)):

| Operação no mestre | Override sobrevive? | O que a UI faz |
|---|---|---|
| renomear peça | ✅ (nome não está na chave) | nada |
| reordenar irmãos | ✅ (e vira dado com `SiblingOrder`) | nada |
| mover peça dentro do mestre | ✅ chave · ⚠️ `Transform` local passa a ser relativo a outro pai | informa *"N overrides de Transform passam a ser relativos a X"* |
| adicionar peça | ✅ | materializa em todas |
| **apagar peça** | ❌ → **sem alvo**, inerte, **volta a pegar** se a peça voltar (undo no mestre) | lista *Overrides sem alvo*; remover só por gesto |
| **remover componente da peça** | ❌ → sem alvo | idem; ⚠️ o sync tem de ver remoções (archetype/`removed_with_id`) |
| **mudar a forma de um componente** (variante de enum, campo removido) | ✅ os outros campos · ❌ o campo removido → sem alvo | **só porque `field_id` é declarado** |
| mover peça PARA FORA do mestre | ❌ (é um delete para este mestre) | sem alvo |
| desempacotar uma aninhada dentro do mestre | ❌ caminhos `[N,…]` | re-key pelo mapa de links, relatório |
| trocar a aninhada por outro mestre | ✅ re-key se aparentado (variant/base) · ❌ senão | §3.7 |

### §2.7 — ⭐ O desfazer incremental — o protocolo, com as 6 condições que o refutador impôs

[Refutação 2](pesquisa/instancias_2026-08-21/refutacao_2_captura_incremental.md) derrubou o enunciado *"byte-idêntico ao de hoje"* (impossível: a ordem e o `parent` mudam de forma) e devolveu o protocolo correto. **Adotado integralmente:**

1. **Cache** `BTreeMap<StableId, Row { archetype: ArchetypeId, parent: Option<StableId>, bytes }>`, linhas **`Arc`** (clone de 10 k linhas: 0,038 ms vs 0,776 ms).
2. **Sujo** = algum `ComponentId` registrado **ou `ChildOf`** tem `changed/added` mais novo que o tick da **ÚLTIMA CAPTURA**, **ou** o `ArchetypeId` difere do cacheado (cobre remoção de componente e de `ChildOf`). Alternativa equivalente: unir `removed_with_id` por id registrado, lido **antes** do `clear_trackers`.
3. **Tick é pré-filtro; bytes são a verdade** — re-serializa só as sujas, **compara com o cache**, emite delta só se diferir (absorve o falso positivo do `DerefMut`).
4. **Spawn** por ausência no cache; **despawn** por carimbo `seen`.
5. ⚠️ **`clear_trackers()` roda exatamente UMA vez por CAPTURA — nunca nos quadros de retorno precoce** (gesto segurado): `is_newer_than` é **estrito**, e um clear por quadro durante um arrasto de 10 quadros faria só a mutação do último quadro entrar no passo. O restore reinicializa o cache do snapshot restaurado e dá o clear dele.
6. **`StableId`**: alocado num hook `on_add` de `Transform` (ou `#[require]`) a partir de **contador persistido monotônico, fora do `ProjectState`** (senão um undo reusa ids vivos na pilha de redo); **único por documento** (gate + prova de mutação); **remapeado em toda cópia de blobs** (`extract_component_snapshot`+`insert_from_bytes` copiam verbatim hoje); `restore()` despawna `With<StableId>` (hoje `With<Transform>`).

**Formato:** `WorldSnapshot::VERSION` 1→2 (linhas em ordem de `StableId`; `parent: Option<StableId>`; blob `StableId` registrado) + `PROJECT_SCHEMA` 84→85 — **a primeira migração da história do repo**, com corpus de fixtures v84 e gate de round-trip byte-equivalente.

**O que fica para a wave seguinte (medido, não adivinhado):** o **restore** continua O(mundo) (despawn-all + respawn) — a cura é o passo 1 do Blender: diff do snapshot-alvo contra o cache por `StableId`, aplicar k linhas. E as partes **fora do ECS** (`VecScene`/`FlipDoc`/`GuideSet`/`StateSets`) continuam clone+compare inteiros — precisam de contador de versão ou `Arc` por path. Os dois entram no plano como fases com bench.

**Gate de ponto fixo** (a memória *"undo só faz uma etapa"*): `capture → frame → capture` idêntico, incluindo **uma instância com peça dinâmica mid-play e um Ctrl+Z sobre ela** — o caso que o refutador 1 construiu.

### §2.8 — O mestre É um asset — e isto unifica com o Asset Browser

- *Criar componente* põe o mestre na **biblioteca** (índice de assets, catálogos por UUID, preview) e deixa uma instância no lugar. O mestre tem `LogicalId` (a identidade lógica do C8) e aparece no navegador como qualquer pincel, paleta ou malha.
- *Editar mestre* abre em contexto (as peças ficam no mesmo `World`, marcadas `MasterPiece`, **excluídas** da simulação/timeline, visíveis só neste modo ou dimmed) — é o Unity Prefab Mode *In Context* e o *"editing the definition in memory"* do Houdini, **sem** o problema das duas instâncias unlocked divergentes (há um mestre, não N superfícies).
- Arrastar do navegador para o canvas = *Instanciar*. O mesmo payload de arrasto (`DragAsset(AssetRef)`) serve timeline, grafo e inspector (C10).
- ⚠️ O módulo vetorial hoje mostra o mestre **no canvas** (Figma). Na unificação, a vista "mestre no canvas" **é** o modo *Editar mestre*; fora dele o que está no canvas é sempre instância. É uma mudança de UX do vetor a medir com o Enio no smoke — e é a única mudança de produto que esta arquitetura impõe ao que já existe.

### §2.9 — O vetor: `VecInstance` é SUBSUMIDO, não mantido ao lado

Manter o modelo derivado (1 entidade) **e** o materializado seria *"duas respostas a 'o que é uma instância?'"* — a divergência que o `vec_component.rs` existe para impedir. Decisão: `VecInstance` → `ObjectInstance`; as peças de uma instância vetorial viram entidades (com `VecPathRef` próprio, `VecPathId` novo); o produtor `InstanceLive` de `LiveGeometry` **morre** (as peças desenham-se sozinhas); `OverrideSlot::{Fill, Hidden}` → `(peça, VecPath, fill)` e `(peça, Visibility, hidden)`; os verbos e a UI (Create/Place/Detach/Reset/Update Main/Swap/Variant) **ficam**, reescritos sobre o mecanismo geral; `MAX_INSTANCE_PIECES = 16` some (o inspector é derivado). ⚠️ A lei do `vec_instance_follow.rs` (ΔTi = ΔTm·I_lin, aplicada na mudança) precisa de **re-medição** sob o sync — ela existia porque o suporte era um retângulo; com peças reais, redimensionar o mestre propaga pelo sync. Migração: documentos com `VecInstance` são **materializados no load** (degrau do `PROJECT_SCHEMA`).

---

## §3 — As 12 perguntas, respondidas de novo (só o que mudou)

| # | v1 dizia | **v2 diz** |
|---|---|---|
| **C1** ponte | descritor comum, sem fundir nó e componente | **igual** — e o descritor ganha `field_id`, política e refs (§2.2). Continua folha sem `bevy_ecs`/`nodegraph` |
| **C2** objeto vazio | `Transform+Name+StableId+RootOrder` | **+ `SiblingOrder`** quando é filho. ~68 B de dados |
| **C3** tipo Rust × schema | tipo Rust + derive | **igual**; `ScriptProperties` continua a porta do utilizador |
| **C4** Sprite | 3 cortes + 1ª migração | **igual** — e agora a migração é a MESMA wave do `StableId` (um degrau, não dois) |
| **C5** presets | cascata visível; preset = instância | **igual**, e mais forte: preset = mestre na biblioteca; *Instanciar* ou *Duplicar* |
| **C6** modelo de prefab | propriedade nomeada **derivada** (1 entidade) | ⭐ **propriedade nomeada MATERIALIZADA**: objetos reais + chave por id + diff-capture + sync vivo. Toma do diff estrutural o "objetos reais"; da propriedade nomeada a chave robusta; do USD a regra de força (template < default, e `Local` mais forte) — sem a máquina de camadas |
| **C7** aninhamento/variantes | variantes sim; aninhamento **não** | ⭐ **os dois na v1**, sem limite de profundidade; recusa só ciclo (§2.6) |
| **C8** identidade | 3 níveis + `StableId` | **igual**, com o protocolo de alocação escrito (§2.7 item 6) e a migração das **duas** famílias de `stable_name_id` |
| **C9** disco | um ficheiro por asset como destino | **igual**; mestres entram no índice desde a v1 |
| **C10** browser | subsistema | **igual**; *Instanciar* é o arrasto do mestre (§2.8) |
| **C11** undo | "já é de graça; não estragar" | ⭐ **reescrito: incremental**, protocolo de 6 condições (§2.7), bench-gate |
| **C12** fora da v1 | 11 exclusões | **reescrito** — ver abaixo |

### C12 — o que fica para DEPOIS (sequência com degrau nomeado) × o que fica FORA (decisão)

⛔ **Nada do que o Enio pediu fica de fora.** O que há é ordem:

| Depois, com degrau nomeado | Degrau |
|---|---|
| Restore do undo incremental (hoje O(mundo)) | fase F9 — diff por `StableId` (Blender passo 1) |
| `VecScene`/`FlipDoc` fora do clone inteiro | F9 — contador de versão ou `Arc` por path |
| Clipe de timeline autorado no mestre propagar às instâncias | **`SequencePlayer`** (TOP-20 #19) — componente ⇒ propaga |
| Config de física do mestre alcançar corpos em play sem Reset | re-descrever mid-play preservando velocidade (item na ponte) |
| Um ficheiro por asset (Git-friendly) | wave do C9, depois do índice |
| Thumbnails animados | depois dos estáticos; só pasta visível |

| Fora, por decisão | Por quê |
|---|---|
| Componentes definidos em runtime pelo utilizador | ADR-0075; a porta é `ScriptProperties` |
| Busca visual por similaridade (ML) | fronteira dura do áudio; **cor dominante fica** |
| Coleta automática de órfãos de asset | nenhuma das 4 engines faz; relatório, não colheita |
| Bibliotecas partilhadas com sombreamento | mostram-se lado a lado, não se fundem |
| Casar overrides **automaticamente** por nome ao trocar para mestre não-aparentado | HR-5; o Unity proíbe com nomes duplicados |
| Apagar *Overrides sem alvo* automaticamente | Unity: *"you might have moved the object… temporarily or in error"* |

---

## §4 — As candidatas reconsideradas, e por que a v1 errou

| | C1 Materializada (diff estrutural) | C2 Derivada (v1) | C3 Camadas (USD-lite) | ⭐ **C4 Materializada e VIVA** |
|---|---|---|---|---|
| Física na instância | ✅ | ⛔ **não** | ✅ | ✅ |
| Aninhamento v1 | ✅ nativo | ⛔ **não** (nem renderiza hoje) | ✅ | ✅ sem limite, recusa ciclo |
| Override sobrevive a reestruturar o mestre | ❌ (Unity *unused*, Godot *vanished*) | ✅ | ✅ | ✅ chave por id + `field_id` declarado |
| Propagação ao vivo | ❌ bloqueada no campo tocado (e só em re-merge) | ✅ | ✅ | ✅ por campo, por quadro |
| Custo do undo | ⛔ agrava (N entidades) | não agrava, não resolve | ✅ resolve | ✅ **resolve** (0,27 ms @10k, medido) |
| 2ª máquina de resolução | não | não | ⛔ **sim** | não — é o ECS + um passe |
| Reusa o provado na casa | `PrefabDoc` (morto) | `VecInstance` | nada | `VecInstance` (verbos, UI, lei de chave) + ponte + registro |
| Inédito? | não | não | não | **sim** — nenhum ECS permissivo faz sync incremental no mesmo mundo |

**Onde a v1 errou, em uma linha:** tomou a *implementação* do módulo vetorial (um objeto só, desenho derivado — um atalho que funciona porque desenho é derivável) pela *arquitetura*, e importou a limitação para domínios onde não cabe. A lei certa do vetor era outra: **chave por id, esparsa, canônica, derivada-por-construção** — e ela sobrevive intacta no C4.

**O que o C4 tira de cada um:** do C1, *objetos reais* (física, filhos, tudo); do C2, a *chave robusta* e *propagar por construção*; do C3, **a única ideia que valia o preço** — *o custo do undo é o da edição* — obtida por ticks do ECS + cache, **sem** a máquina de camadas.

---

## §5 — O que ainda se perde — honesto, e menor

1. **Um clipe de timeline autorado no mestre não propaga às instâncias** até existir `SequencePlayer`. Hoje a timeline é documento-único com bindings por entidade; a peça da instância precisa de binding próprio para ser dirigida por tique. Degrau nomeado (TOP-20 #19).
2. **Config de física propagada chega ao solver no Reset**, não mid-play (limite da ponte, `bodies.rs:169`). Declarado; item para levantar.
3. **O restore do undo continua O(mundo) até a F9.** A captura fica O(edição) na F2; o Ctrl+Z em si fica rápido uma fase depois.
4. **A migração é a maior da história do repo**: `StableId` em toda entidade, `SiblingOrder`, `WorldSnapshot` v2, `PROJECT_SCHEMA` 85, duas famílias de `stable_name_id`, e a materialização dos `VecInstance` existentes. Sem projetos publicados, o custo é interno — mas é uma wave não isolável.
5. **O mestre não simula.** É o template, na biblioteca. O que simula é a instância que *Criar componente* deixa no lugar. Não há perda prática — é a mesma regra do Unity/Houdini/USD — mas muda a UX do vetor (§2.8).
6. **Overrides sem alvo existem** e ficam até alguém os remover (Unity). É o preço de *"volta a pegar se a peça voltar"*.
7. **Trocar para mestre não-aparentado** é heurístico com relatório. Todos os sistemas.
8. **Editar um mestre com muitas instâncias suja muitas linhas** do undo: 100 instâncias × 10 peças = 1.000 linhas ≈ 0,95 ms + 247 KB por passo. É linear no fan-out, não no mundo. Otimização nomeada e **não medida**: pular no snapshot as linhas cujos componentes são inteiramente derivados do link (re-deriváveis no restore pelo sync).
9. **Ordem de irmãos passa a ser dado** (`SiblingOrder`) — o que *corrige* um bug pré-existente mas obriga a decidir se a ordem é overridável por instância (Unity sim; Figma não). Decisão: **sim** — é um campo como outro.

---

## §6 — Sequenciamento (cada fase mergeável, engine nunca quebrada)

| Fase | O quê | Critério de pronto (observável) |
|---|---|---|
| **F0** | `ComponentDesc` (`field_id` append-only · política · refs) + `insert_default` no registro | inspector **derivado** para 1 componente; gate de snapshot da tabela de `field_id` |
| **F1** | `StableId` + `SiblingOrder` + `WorldSnapshot` v2 + `PROJECT_SCHEMA` 85 + **1ª migração** + migrar `WireId`/joints/roldanas + `restore()` por `StableId` + corte da Sprite (C4) | round-trip v84→v85 byte-equivalente sobre corpus; gate `no_two_rows_share_a_stable_id` + prova de mutação; **reordenar irmãos é desfazível** (visível ao Enio) |
| **F2** | ⭐ Undo incremental (protocolo §2.7) + linhas `Arc` + pilha de deltas + bench permanente em `crates/ph2d-ecs` | **bench-gate: ≤ 0,3 ms @10k parado · ≤ 1,0 ms @10 % sujo** (máquina calma); ponto fixo `capture→frame→capture`; mutação: remover componente ⇒ passo registrado |
| **F3** | *Add Component* com **cascata visível** + criar objeto vazio na raiz (o `HIERARCHY_ADD` morto) | ⭐ **walking skeleton**: objeto vazio → componente → salvar → reabrir → Ctrl+Z |
| **F4** | Núcleo de instância: `Duplicar` profundo · `Criar componente` · `Instanciar` · sync topológico + recusa de ciclo · captura por diff · política por tipo · `Destacar`/`Redefinir`/`Aplicar ao mestre` · filtro `Without<MasterPiece>` na ponte + porta `pose_owner` + lane no c9 · remap de refs · **`VecInstance` subsumido** (migração) | ragdoll-mestre: instância cai e a junta prende os **seus** corpos (gate); hash 3-OS com mestre+instância; Ctrl+Z sobre instância dinâmica mid-play = ponto fixo |
| **F5** | Aninhamento + variantes (irmã e derivada) + *aplicar ao interno* + re-key + *Overrides sem alvo* (UI) | instância de instância de instância; trocar variant↔base preserva overrides; relatório ao trocar não-aparentado |
| **F6** | Índice de assets (`ph2d-assetdb`: kind, catálogo UUID, tags, deps, preview assíncrono) — mestres entram | `query`/`deps`/`preview` sem UI; bench 10 k assets |
| **F7** | Painel Asset Browser + `DragAsset` único (canvas/timeline/grafo/inspector) + `AssetRef<Kind>` como `ParamRow` | arrastar mestre → instância; arrastar textura → campo |
| **F8** | Restore incremental + versionamento de `VecScene`/`FlipDoc` | Ctrl+Z ≤ 1 ms @10k (bench-gate) |
| depois | um ficheiro por asset · física mid-play · `SequencePlayer` · thumbnails animados | — |

**Por que esta ordem:** F1/F2 são invisíveis mas obrigatórias — materializar (F4) sem o undo incremental (F2) multiplica o custo que hoje já estoura o quadro; e `StableId` (F1) é pré-condição tanto do undo (chave de linha) quanto das instâncias (chave de override e de remap). F3 é o primeiro degrau que o Enio testa. Mestre-como-asset (F4) é o que faz o navegador (F6/F7) nascer com conteúdo.

---

## §7 — A trilha de verificação

| Afirmação submetida | Veredito | O que entrou no desenho por causa dela |
|---|---|---|
| *"Materializada + sync por ticks não viola HR-5/ADR-0021/ADR-0131 e não entra no hot path"* | **REFUTADA** (3 de 4 cláusulas) | §2.5 (1)-(3): mestre fora da ponte; porta `pose_owner`; entradas por tique fora do sync; migração antes de materializar; tick como pré-filtro; gate de ponto fixo com instância dinâmica |
| *"Captura incremental é byte-idêntica à de hoje; spawn/despawn/reparent bastam; determinística no restore"* | **REFUTADA** (4 buracos) | §2.7 integral: archetype/`removed_with_id`; `clear_trackers` por captura; `StableId` com hook + contador fora do undo + remap em cópia + unicidade gateada; `SiblingOrder` como dado |
| *"Chave `[path]+peça+(tipo,campo)` sobrevive a reestruturar; só delete e troca-de-aninhada pedem resync; variant sem mecanismo novo"* | **REFUTADA** (lista curta; `field_id` posicional; 3 mecanismos novos) | §2.2 `field_id` declarado; §2.6 tabela completa de operações + *Overrides sem alvo*; ordem topológica + recusa de ciclo; re-key para variant↔base; remap de refs em toda propagação; *aplicar ao interno* apaga intermediários |

*"Refutado" aqui não significa "errado de propósito" — significa que a versão de uma frase não bastava e a versão de uma página é a que está acima.*

---

> ✅ **APROVADA (Enio, 2026-08-24 — "levantar um agente para implementação").**
> Os entregáveis das Fases E e F existem:
> **[ADR-0164](../architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md)**
> (objeto/instância/undo, governa F0–F5) ·
> **[ADR-0165](../architecture/decisions/0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md)**
> (identidade de asset + browser, governa F6–F7) ·
> **[plano vivo 05](05_plano_de_implementacao.md)** (fases, critérios de pronto, gates).
> Executor: linha `line/components` (Modo L). As duas decisões de produto (mestre na biblioteca;
> ordem de irmãos como dado overridável) estão embutidas — o smoke da F4 é onde o Enio as sente, e o
> plano §9 nomeia a alternativa de cada uma.
