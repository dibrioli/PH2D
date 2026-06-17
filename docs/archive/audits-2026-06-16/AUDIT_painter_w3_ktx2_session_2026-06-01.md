# Auditoria multi-agêntica — Painter W3 Block 2 + persistência v2 + KTX2 magenta (2026-06-01)

**Método:** workflow adversarial — 6 lentes paralelas (WGSL/paridade · Rust+HR-3/5 ·
ABI/frozen/DoS · spec/determinismo/honestidade · regressão/integração ·
cobertura-vs-claims) → verificação adversarial de CADA achado (tentar refutar;
default refutado se incerto) → síntese por severidade. **33 agentes, ~1.94M tokens.**

**Escopo (commits):** `6ba3ed7` (compositor GPU), `249735e` (persistência v2),
`385e7e2` (KTX2 magenta), `dc2f765` (cap), `411b3ae` (fmt).

**Resultado bruto:** 27 achados → **15 confirmados, 12 refutados** (a verificação
derrubou falsos-positivos: ex. "GPU diverge do compositor da tool" — refutado, os
encoders são bit-idênticos no domínio alcançável; "gate só testa literais" —
refutado, é a convenção honesta do projeto + 3 caveats documentados).

---

## CRITICAL (1) — CORRIGIDO ✅ `4368a77`

**Stack-overflow DoS no `load()` de um `.ph2d-painter` forjado.** O `LayerNode` v2 é
recursivo (`Group{children: Vec<LayerNode>}` + `mask: Option<Box<LayerNode>>`); o
`Deserialize` derivado do postcard recursa sem limite e `load()` desserializa ANTES
de `validate_caps_post_deserialize`. Um file de ~600 KB com cadeia profunda estourava
a pilha (SIGABRT incatchável) ANTES do cap `MAX_GROUP_DEPTH` rodar — **a mitigação
documentada (§2.5 do ADR) era ineficaz**. **Provado empiricamente** por dois
verificadores independentes (SIGABRT signal 6).

**Fix:** `Deserialize` hand-written pro `LayerNode` com guarda de profundidade
(thread-local + RAII) que erra acima de `MAX_LAYER_NODE_DESERIALIZE_DEPTH=32` →
`Err`, nunca aborta. `Serialize` segue derivado → wire + cook-hash inalterados.
3 testes de regressão (cadeia de grupo + de mask → `Err`; árvore legítima carrega).
ADR-0046-amд-1 §2.5 atualizado.

> Severidade: um verificador classificou CRITICAL, outro HIGH (não há caller de
> produção HOJE — a ponte savefile não está fiada). Tratei como **fix imediato**:
> é falha de design de formato congelado, melhor corrigir agora (nenhum file v2
> existe ainda) que pós-ship.

## LOW/MEDIUM confirmados — CORRIGIDOS ✅ (`4368a77` + `834b840`)

| # | Achado | Fix |
|---|--------|-----|
| mask-depth | `validate_layer_node` recursava mask no MESMO depth (bypassava o cap de grupo) | bound em `depth+1`, igual a grupo |
| opacity clamp | GPU foldava `opacity` sem clamp; CPU clampa → divergia em opacity>1 | `flatten_layer_ops` clampa `[0,1]` |
| HR-3 allocs | `distinct_layer_count` + `alloc_slice` alocavam `BTreeSet` por chamada no caminho real-time do `composite()` | ambos allocation-free |
| active-flag | invariante "≤1 ACTIVE" não-validado (doc dizia "exatamente 1") | valida ≤1 (0 legal, mapeia `Option<LayerId>`); doc corrigida; teste de rejeição |
| deep-nest gap | `cs_grouped` `stack[d]` d=1..7 (WGSL mais arriscado) sem paridade vs CPU (só depth-1 testado) | teste GPU de paridade depth-8 (≤1 byte ✓) |
| readback | `readback_rgba8` ignorava resultado do `map_async` → panic opaco | checa o resultado (test-path) |
| WGSL comment | comentário "bit-for-bit port" overstateava (omite `sanitize01` NaN→0) | corrigido pra "port over finite inputs" + razão (inputs LUT-bounded-finite, divisões guardadas — NaN inalcançável) |

## Aceitos como LOW (sem fix agora — com justificativa)

- **Eviction LRU + version-bump re-upload sem teste** (cobertura): lógica estática
  sólida; **zero caller de produção** do `composite()` GPU hoje; exigiria
  white-box accessors. Follow-up quando o shell fiar o compositor no render loop.
- **`Region::clamped` sem teste** (cobertura): pure fn trivial, verificavelmente
  correta (clamp de `x` antes do subtrai evita underflow); risco só de regressão.
- **opacity NaN / malformed-leaf round-trip pins** (cobertura, formato congelado):
  o formato é lossless-por-design; o consumidor (ponte runtime) ainda não existe;
  NaN passaria o blake3 mas quebraria PartialEq do round-trip — pin futuro.
- **Magenta bridge integration sem teste de automação**: é debug-aid, coberto pelo
  smoke do Enio, sem seam de CI (convenção `#[ignore]` GPU da casa).

## Refutados notáveis (a verificação adversarial pegou)

- "GPU compara contra re-impl, não a fonte-de-verdade" → os encoders alpha são
  **bit-idênticos no domínio alcançável** (provado por sweep de 100M amostras);
  ambos orquestram o MESMO `apply_blend`.
- "validate recursa node-count só no fim → DoS" → refutado: postcard já
  materializa tudo antes (bounded por `MAX_CANON_BYTES`); checar mid-walk não
  reduz o pico.
- "PoC de stack-overflow deixado no tree" → refutado: arquivo não existe (só
  artefatos `target/` gitignored).
- "gate de naga não confirma os 2 entry points" → refutado: o teste **panica**
  se faltar `cs_flat`/`cs_grouped` (naga 28 só lista entry points stage-decorated).

## Status pós-remediação

Todos os gates verdes: `ph2d-painter-stroke` + `ph2d-render` suites; clippy
`--all-targets`; paridade GPU (22 modos + grupos + deep-nest) ≤1 byte; DoS fechado
com 3 regressões. Commits locais: `4368a77`, `834b840` (+ ADR `0046-amд-1`).
