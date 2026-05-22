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
