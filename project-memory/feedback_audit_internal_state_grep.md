---
name: feedback-audit-internal-state-grep
description: "Generaliza [[no-industrial-claims-without-verification]] para verificações INTERNAS do próprio repo — antes de escrever ADR, sweep-grep cada pub-prefixed identifier citado"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: dce46045-2a41-4790-9b17-7d34ca1c1315
---

Antes de escrever ADR que cita `pub`-prefixed identifiers (enums, structs, traits, métodos, módulos, constantes), executar **sweep-grep preventivo** verificando que CADA símbolo nomeado existe no código real.

**Why:** ADR-0055 Round 1 (2026-05-27) caiu por afirmar fatos sobre crates externos sem verificação (`basis-universal-rs >= 0.4`, "Embark Studios", Unity/Unreal adoption). Memória [[no-industrial-claims-without-verification]] foi escrita. Round 2 (mesma sessão) **eliminou** os 13 findings externos via `cargo info` / WebFetch / etc., mas **introduziu 6 NOVOS findings críticos do MESMO padrão para símbolos INTERNOS**:

- `Ktx2Blob` (real `Ktx2Image`) — 8 sites no ADR
- `DeviceTier` enum (real é vapor; gate silently-passing)
- `MemoryBudget::Render { ... }` enum variant (real é struct com 3 u32 fields)
- `Plugin::init` / `PluginBuilder::declare_budget` (real: trait não existe ainda)
- `AssetDb::resolve_for_tier` (real: só `get(&AssetId)`; content-addressed colide com multi-tier)
- `Ktx2::keyValueData` parser preservation (real: parser descarta kvd)

Padrão: a regra "verify external claims" não generalizou para "verify internal claims". Round 2 lentes scoraram 6.5/6.5/5.2 vs target 9.0 — melhoria líquida marginal +2.0/+0.2 sobre Round 1.

**How to apply:** ANTES de escrever um ADR ou plano que cita símbolos do codebase, executar checklist:

```bash
# Para cada `pub`-prefixed identifier citado no draft:
grep -rn "pub (enum|struct|trait|fn|const|type|mod) <SYMBOL>" crates/ --include="*.rs"
# Resultado:
#   - Match encontrado → cite line:file no §pre-flight do ADR
#   - Zero matches    → símbolo é VAPOR (não citar como existente; ou
#     declarar como "novo W1.TX" + escopo realista)
```

Para enums sob arch-gate FROZEN:
```bash
# Confirmar gate não-vacuous (alguns gates usam `if let Some(n) = count_enum_variants(...)`
# que silenciosamente passam quando enum não existe):
grep -A 5 "<gate_name>" crates/<crate>/tests/architecture_*.rs
# Se gate usa `if let Some`, gate é vacuous quando enum vapor — desabilitar
# como prova de "FROZEN" até enum materializar.
```

Para APIs:
```bash
# Verificar método/trait existe:
grep -rn "fn <method>\b\|trait <Trait>\b" crates/
# Sample uso real (não só declaration):
grep -rn "<method>(" crates/
```

**Quando aplicar (mandatory)**:
- Novos ADRs com seções §2 Decisão / §3 Estrutura citando tipos do repo.
- Amendments propondo "extend enum X" sem confirmar shape atual.
- Snippets de código no ADR que parecem implementáveis ("aqui está o código novo W1.T4").

**Quando NÃO precisa**:
- ADRs puramente strategic/policy (sem código snippet).
- Discussões de trade-off arquitetural sem nomear tipos específicos.

**Vinculado a**: [[no-industrial-claims-without-verification]] (caso externo), [[feedback-perfection-no-deferrals]] (gaps não diferidos), [[ktx2-phase1-done-phase2-aborted-2026-05-26]] (ADR-0055 v1 abortado pelo mesmo padrão).
