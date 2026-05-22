# ADR-0037 — Id de entidade estável no SceneDoc (postcard), desacoplado do `to_bits` do bevy

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto).
**Depende de:** HR-6 (blake3), HR-14 (save versionado), ADR-0025.

## 1. Contexto

O protótipo MiniCavalry espelha o `Entity::to_bits()` do bevy (`gen<<32 | index`) para serializar entidades no `SceneDoc`, e usa um mapa JSON `{nome:{version,data}}` para componentes. O layout interno do `to_bits` do bevy_ecs **muda entre versões** — não é formato de wire estável. E save de jogo precisa migrar pra frente (HR-14), o que JSON+to_bits não garante byte-a-byte.

## 2. Decisão

- **Id de entidade próprio, estável e versionado** no `SceneDoc` — desacoplado do `to_bits` de runtime (que é detalhe de implementação do bevy, pode mudar).
- **`SceneDoc` em postcard** (binário determinístico) + tabela `nome→stableTypeId` onde `stableTypeId = blake3(nome_canônico)[..8]` (HR-6), **não** JSON. Ordem determinística.
- Generaliza para o **formato de grafo** (ADR-0032): grafos salvos em save de jogador também migram pra frente; IDs estáveis + ordenação determinística + layout segregado.

## 3. Consequências

**Aceitas:**
- Cook bate byte-a-byte com o `ComponentRegistry`; save portável cross-platform (HR-14); merge/diff de grafos entre agentes.
- O protótipo deve mudar (to_bits→id estável, JSON→postcard) para o cook bater.

**Riscos:**
- Mapeamento id-estável ↔ `Entity` de runtime precisa ser reconstruído no load → tabela de remap no `World::restore`.

## 4. Alternativas consideradas

- **Espelhar `to_bits` do bevy no save:** rejeitado — não é formato estável entre versões do bevy; quebra save de jogador.
- **JSON name-keyed (modelo do protótipo):** rejeitado pro canônico — postcard é determinístico + menor; JSON fica só pra dev (cenas/configs), nunca shipping (§10.1 do SKILL).

## Nota de implementação (2026-05-22, W1.T1)

**A engine PH2D já cumpre este ADR — o anti-padrão (`to_bits` + JSON) era do protótipo MiniCavalry, não da engine.** Verificado em código:

- **Formato de save persistido** = índices, não `to_bits`:
  - `ph2d-asset::SceneDoc` (`scene.rs`): hierarquia via `ChildOfPair { parent_index, child_index }` indexando `instances[]`; postcard; `version: u32` (HR-14).
  - `ph2d-ecs::scene::save::WorldSnapshot` (`save.rs`): `EntitySnapshotRow { parent: Option<u32> }` (índice no snapshot, DFS order); "portable across world instances by design"; postcard; versionado. É o formato de save canônico.
  - `ComponentTypeId = blake3(nome_canônico)[..8]` (`registry.rs`), nunca `std::any::TypeId` (HR-6).
- **Os usos de `Entity::to_bits` são handles opacos TRANSIENTES**, nunca persistidos: `scene::snapshot` (HierarchySnapshot/ComponentSnapshot — editor, "hex-id form so the editor never holds a live Entity", HR-8) e `scene::commands` (payloads de comando/MCP de sessão). Corretos por HR-8 (handle opaco de sessão), fora do caminho de save.

**Conclusão:** ADR-0037 satisfeito pela arquitetura existente; **nenhuma mudança de código necessária**. `ph2d-save` permanece stub (o snapshot/restore canônico vive em `ph2d-ecs::scene::save`; consolidar em `ph2d-save` é cosmético, M13+ por demanda). O grafo de nós (`ph2d-nodegraph::format`) também já é estável (NodeIds estáveis + formato textual diffável, W1.T2). A diretriz do ADR aplica-se ao **port** das ideias do MiniCavalry, não a um conserto da engine.
