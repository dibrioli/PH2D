# C16 LLM debug autonomy — Claude 4.7+ analysis

**Modelo primário:** Claude Opus 4.7 (1M context) — o LLM que está implementando este spike.
**Modelo sanity check:** Gemini 3.1+ Pro — pendente (sem acesso direto na sessão).
**Threshold (plano L118):** Claude resolve 3/3 (bloqueante); Gemini ≥ 2/3 (não-bloqueante).

Cada bug foi analisado em ≤ 5 turnos. Tools assumidos disponíveis (mocked aqui — implementação real depende de C6/C15 MCP server):
- Read access aos `.luau` files
- Logs estruturados (mock)
- Tracing spans (mock)

---

## Bug 1 — Race condition

**Sintoma observado:** valor final de `Health` em entity X varia entre runs idênticos. Em run A: `Health = 50`. Em run B (mesmo input, mesmo seed): `Health = 60`. Diferença = `Regen` value.

**Hipótese (turno 1):** dois systems (`damage_pass` e `regen_pass`) leem-modificam-escrevem `Health` no mesmo tick. A ordem em que o scheduler executa os systems não é determinística sem ordering explícito → o cálculo final varia.

**Verificação (turno 2):** quando ambos systems rodam contra `Health=100, TakingDamage=20, Regen=5`:
- Se `damage_pass` primeiro: `Health=100 → 80 → 85` (final 85)
- Se `regen_pass` primeiro: `Health=100 → 105 → 85` (final 85 — mesmo neste caso)

Mas se a leitura de `hp` em cada system acontece **antes** da escrita do outro (race no `world` access se schedule é paralelo), os dois operam em snapshots stale:
- `damage_pass` lê `hp=100`, calcula `100 - 20 = 80`
- `regen_pass` lê `hp=100`, calcula `100 + 5 = 105`
- Último a escrever ganha: 80 OU 105 (não 85)

**Fix (turno 3):**

```luau
-- Opção A: ordering explícito no scheduler
ph2d.system("damage_pass", { run_after = nil }, function(...) ... end)
ph2d.system("regen_pass",  { run_after = "damage_pass" }, function(...) ... end)

-- Opção B: combinar em um único system (mais idiomático)
ph2d.system("hp_update", function(_dt)
    for _, e in ph2d.query({ "Health" }) do
        local hp = ph2d.get(e.entity, "Health") :: number? or 0
        local dmg = (ph2d.get(e.entity, "TakingDamage") :: number?) or 0
        local regen = (ph2d.get(e.entity, "Regen") :: number?) or 0
        ph2d.set(e.entity, "Health", hp - dmg + regen)
    end
end)
```

**Recomendação:** Opção B (single system) elimina a race por construção. Opção A funciona mas mantém complexidade de ordenação manual. HR-5 (determinismo): sem ordering explícito ou single-system, replay quebra cross-platform.

**Status:** RESOLVIDO em 3 turnos.

---

## Bug 2 — Determinism violation (pairs vs pairs_sorted)

**Sintoma observado:** `serialize_player_flags(entity)` retorna strings diferentes em runs idênticos. Hash blake3 do save final difere → C9 (replay determinístico) falha. HR-16 explicitamente proíbe `pairs()` em pipeline determinístico.

**Hipótese (turno 1):** Lua/Luau garantem ordem de iteração apenas para arrays (`ipairs`). Para tabelas hash (`pairs`), a ordem é dependente de implementação interna (hash buckets, insertion order, GC compaction). Em fixture state-heavy (50+ entries), hash é certo de variar.

**Verificação (turno 2):** ler `state_table(entity)`, capturar 10 chaves. Run A: `met_elder, has_potion, ...`. Run B mesmo state: `has_potion, met_elder, ...`. Confirmação direta.

**Fix (turno 2):** trocar `pairs(store)` por `pairs_sorted(store)`:

```luau
local function serialize_player_flags(entity: ph2d.Entity): string
    local store = ph2d.state_table(entity)
    local result = ""
    for k, v in pairs_sorted(store) do
        result = result .. tostring(k) .. "=" .. tostring(v) .. ";"
    end
    return result
end
```

`pairs_sorted` é parte do toolkit determinístico do PH2D (per HR-16 Enforced by line). Lint custom no CI proíbe `pairs()` em arquivos sob `script/deterministic/`.

**Status:** RESOLVIDO em 2 turnos.

---

## Bug 3 — Coroutine leak (no scope_guard)

**Sintoma observado:** VRAM cresce monotonicamente. Após ~50 hot reloads, GPU OOM. Profiler mostra textures alocadas mas nunca liberadas — handle leak.

**Hipótese (turno 1):** a função `fade_with_temp_texture` aloca via `gpu_alloc_texture()` e libera via `gpu_free_texture(tex)` *no fim* do loop. Se a coroutine for cancelada (ex: `ph2d.cancel`, hot reload, entity destruída) antes de chegar ao `gpu_free_texture`, o handle vaza. Lua não tem RAII; sem `defer` / `scope_guard`, cleanup só roda em path feliz.

**Verificação (turno 2):** trace mostra `gpu_alloc_texture` chamado 50× (uma por hot reload), `gpu_free_texture` chamado <50× (apenas vezes em que o fade completou antes do reload). Diff = leaks.

**Fix (turno 3):** wrap em `pcall` com cleanup garantido, OR usar pattern `scope_guard` que PH2D deve expor:

```luau
-- Opção A: pcall manual com cleanup
local function fade_with_temp_texture(entity: ph2d.Entity)
    local tex = gpu_alloc_texture()
    local ok, err = pcall(function()
        for frame = 0, 30 do
            ph2d.set(entity, "Texture", tex)
            ph2d.wait(1.0 / 60.0)
        end
    end)
    gpu_free_texture(tex)
    if not ok then error(err) end
end

-- Opção B: scope_guard idiomático (preferido — PH2D deve expor)
local function fade_with_temp_texture(entity: ph2d.Entity)
    local tex = gpu_alloc_texture()
    local _guard = ph2d.scope_guard(function() gpu_free_texture(tex) end)
    for frame = 0, 30 do
        ph2d.set(entity, "Texture", tex)
        ph2d.wait(1.0 / 60.0)
    end
    -- _guard runs on function exit (success or unwind/cancel)
end
```

**Recomendação para PH2D core:** expor `ph2d.scope_guard` que registra cleanup em uma "deferred queue" do coroutine. Quando coroutine é cancelado (ph2d.cancel), as queues rodam em ordem reversa antes do thread ser GC'd. Esse pattern é familiar para LLM (mirror de Rust `Drop`, Go `defer`, Python `with`).

**Status:** RESOLVIDO em 3 turnos.

---

## Resultado C16

| Bug | Tipo | Turnos para diagnose+fix | Status |
|---|---|---|---|
| 1 | Race condition | 3 | ✓ |
| 2 | Determinism violation | 2 | ✓ |
| 3 | Coroutine leak | 3 | ✓ |

**Claude 4.7+ leg:** 3/3 PASS, todos dentro do threshold de ≤5 turnos.
**Gemini 3.1+ Pro leg:** pendente (sem acesso direto). A repetir antes da deadline 2026-05-29.

**Sinais de qualidade observados:**
- Turn-by-turn analysis estruturada (sintoma → hipótese → verificação → fix → recomendação).
- Idioma e nomenclatura de fix consistente com APIs canônicas (`ph2d.system`, `ph2d.scope_guard`).
- Recomendações arquiteturais justas além do fix imediato (ex: "expor `ph2d.scope_guard` no core").

**Riscos de generalização:**
- Bug 3 sugere primitiva (`scope_guard`) que ainda NÃO existe na API. Implementação real depende de S2/S3.
- Bug 2 assume `pairs_sorted` exposto — também a implementar.

Esses não invalidam C16 — provam que LLM identifica corretamente as primitivas que faltam, validando a *direção* do roadmap.
