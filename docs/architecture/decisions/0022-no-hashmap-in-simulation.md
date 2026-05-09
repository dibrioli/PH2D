# ADR-0022: Banimento de `HashMap`/`HashSet` iteration em simulation crates

**Status:** Accepted
**Data:** 2026-05-08
**Decisor:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Origem:** auditoria pré-plano (sugestão LLM1; extensão de HR-5).

## Contexto

HR-5 (determinismo) bane FMA, fast-math, RNG sem seed, GPU compute em pipeline simulado, etc. Mas **omite o vetor mais comum de divergência cross-platform em Rust:** iteração de `std::collections::HashMap` / `HashSet`.

`std::collections::HashMap` usa SipHash com seed *random* por instância (proteção contra DoS). Iteração é ordem-da-bucket — **não-determinística entre runs**. Mesmo com `BuildHasher` fixo, a ordem depende da capacidade inicial e sequência de inserts/removes.

HR-5 fala em "ordem fixa de operações f32" mas se o caller usa `for (k, v) in some_map.iter() { acc += v; }`, a ordem das adições varia → divergência floating-point cross-platform mesmo sem violar nenhuma das outras regras de HR-5.

HR-16 já proíbe `pairs()` em Luau (mesma razão, lado script). Falta o equivalente Rust.

## Decisão

**Banimento de `std::collections::HashMap` e `std::collections::HashSet` em crates de simulação**, enforcement via `clippy::disallowed_types` no `clippy.toml` do workspace.

**Crates afetados (simulation path):**
- `ph2d-ecs` (canônico — wrapper bevy_ecs já usa `EntityHash` que é determinístico)
- `ph2d-physics`
- `ph2d-physics-soft`
- `ph2d-script` (especialmente `lateral_storage` que persiste para snapshot HR-14)
- `ph2d-net` (state diff packets devem ser determinísticos)
- `ph2d-save` (serialização de state)

**Crates NÃO afetados (presentation, IO, dev tools):**
- `ph2d-render`, `ph2d-vector`, `ph2d-text`, `ph2d-light`, `ph2d-editor` — vivem em `PresentWorld` (ADR-0021), não entram em snapshot determinístico.
- `ph2d-asset`, `ph2d-mcp`, `ph2d-i18n`, `ph2d-telemetry`, `ph2d-host`, `ph2d-input`, `ph2d-a11y` — IO ou metadata, não-simulation.

**Substitutos canônicos por uso:**

| Caso de uso | Substituto |
|---|---|
| Lookup O(1) por chave determinística (Entity, Handle, MessageId) | `bevy_ecs::EntityHashMap` (uses `EntityHash` deterministic) ou `Vec<Option<T>>` indexed por `id.0` (mais cache-friendly se ids são densos) |
| Set de chaves stable | `BTreeSet` (iter ordenado por chave) |
| Map de chaves stable arbitrárias | `BTreeMap` (idem) |
| Hash table determinística com hash custom | `hashbrown::HashMap<K, V, ahash::AHasher>` com seed fixa explícita (ainda assim ordem de iteração precisa de wrap manual em `BTreeMap` para ser determinística) |

**Iteração ordenada quando precisar:** `BTreeMap` resolve por construção. Para `HashMap` em crates não-simulation que precisam ordem ocasional, oferecer função `pairs_sorted_by_key()` análoga ao `pairs_sorted()` do Luau (HR-16).

## Implementação (`clippy.toml` no workspace root)

```toml
disallowed-types = [
    { path = "std::collections::HashMap", reason = "use bevy_ecs::EntityHashMap, hashbrown com hasher fixo, ou BTreeMap (HR-5 + ADR-0022)" },
    { path = "std::collections::HashSet", reason = "use BTreeSet (HR-5 + ADR-0022)" },
    { path = "std::collections::hash_map::RandomState", reason = "RandomState é não-determinístico cross-platform (HR-5)" },
]
```

**Escopo do lint:** lint roda em workspace inteiro por default. Crates não-simulation (presentation, IO) que precisarem usar `HashMap` legitimamente escapam via `#[allow(clippy::disallowed_types)]` localizado, **sempre acompanhado de comentário citando "// ok: presentation, fora de simulation path"**.

## Consequências

**Aceitas:**
- LLM gerando código vai topar com erro cedo se usar `HashMap` em simulation crate. Erro mensagem clara via `reason` no clippy.toml.
- `bevy_ecs::EntityHashMap` torna-se padrão para lookup por Entity (já é o canônico em bevy_ecs 0.18 idiomatic).
- Custo zero em runtime (lint, não código gerado).

**Negadas:**
- Não vamos banir `HashMap` do workspace inteiro — uso legítimo em ph2d-asset (file path → handle), ph2d-mcp (tool name → schema), etc.
- Não vamos forçar `BTreeMap` quando `EntityHashMap` é mais idiomático para Entity keys.

## Alternativas consideradas

- **Não fazer nada (deixar HR-5 cobrir):** descartado — HR-5 já existe há meses e ninguém percebeu a omissão até a auditoria. Sem lint, primeiro PR de simulation introduz divergência silenciosa.
- **Permitir `HashMap` com `BuildHasher` fixo:** descartado — ainda assim iteração depende de bucket internals, não dá determinismo cross-platform.
- **Lint de iteration explícita (forbid `for x in hashmap.iter()`)** descartado — burocrático, false positives demais.

## Próximos passos

1. Criar `clippy.toml` na raiz do workspace com a config acima.
2. Adicionar `cargo clippy` step no CI já cobre (RUSTFLAGS=-D warnings + `clippy.toml` aplicado automaticamente).
3. Atualizar HR-5 do SKILL adicionando bullet "iteração de `std::HashMap`/`HashSet` proibida em simulation crates — ver ADR-0022".
4. Quando `ph2d-physics`, `ph2d-net`, etc. forem populados, validar que código não tem `HashMap` (lint pega; teste manual cega).
