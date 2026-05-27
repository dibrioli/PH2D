# HANDOFF — `ph2d-color` OklchColor serde derives

**Origem:** Painter T1.6 R9 audit, lens W1 (savefile / wire ABI), finding W1-C1.
**Severidade:** CRITICAL.
**Owner sugerido:** dono(a) do crate `ph2d-color`.
**Status:** NÃO FIXADO — fora do escopo Painter; handoff documentado.
**Data:** 2026-05-27.

---

## Resumo

O `OklchColor` canônico em [`crates/ph2d-color/src/oklch.rs:20`](../crates/ph2d-color/src/oklch.rs#L20) **não tem** `#[derive(Serialize, Deserialize)]`. O stub local em [`crates/ph2d-tool-painter/src/params.rs:50`](../crates/ph2d-tool-painter/src/params.rs#L50) tem. Per o contrato HR-14 forward-compat documentado em `params.rs` linhas 10–34, quando T1.3 / W2+ substituírem o stub pelo canon, savefiles v1 produzidos por T1.1/T1.2 **não vão deserializar** porque o canon não sabe falar postcard.

## Reprodução

```bash
grep -n "Serialize\|Deserialize" crates/ph2d-color/src/oklch.rs
# (nada) — confirma ausência dos derives.

grep -n "Serialize\|Deserialize\|pub struct OklchColor" crates/ph2d-tool-painter/src/params.rs | head -3
# 39: #[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
# 40: pub struct OklchColor { l, c, h, a: f32 }
```

O stub e o canon têm a MESMA estrutura (`pub struct OklchColor { l, c, h, a: f32 }` — 4 campos f32), então o postcard byte-encoding seria idêntico — bastam os derives.

## Fix sugerido

1. Adicionar serde como dep em [`crates/ph2d-color/Cargo.toml`](../crates/ph2d-color/Cargo.toml):
   ```toml
   serde = { version = "1", features = ["derive"] }
   ```
2. Derivar em [`crates/ph2d-color/src/oklch.rs:20`](../crates/ph2d-color/src/oklch.rs#L20):
   ```rust
   #[derive(Copy, Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
   pub struct OklchColor { ... }
   ```
3. (Opcional, T1.3 cutover) Implementar o gate de transição que o stub doc já promete: `PainterParams_v1_postcard_deserializes_in_t13` — round-trip que serializa via stub e deserializa via canon. Audit W1-M1 da R9 marca isso como deferred desde W0; T1.3 é o momento.

## Por que o auditor da R9 marcou como CRITICAL

Sem os derives, qualquer ferramenta a jusante que serializar `PainterParams` (que embute `OklchColor active_color`) e depois trocar do stub pro canon ganha um arquivo binário que não abre mais. Não há gate executável que pegue isso hoje — só vira observável no momento exato da substituição em T1.3+.

## O que o Painter T1.6 vai fazer enquanto isso não landa

Nada — o T1.6 usa o stub direto, o save-format ainda é o do stub. O risco materializa só quando T1.3 substitui (provavelmente W2 sidebar). Coordenar a substituição com este fix.

## Cross-ref

- Audit transcript: `private/tmp/.../341efa42-.../tasks/a695f31d42da0d078.output` (lens W1).
- Commit Painter R9 que NÃO incluiu este fix: `7fed63b`.
- ADR-0044 §2.8 (`BrushHandle` stub vs canon — mesma família de gates de transição).
- Memory `feedback_audit_scope_discipline` (a regra que me fez parar de fixar isso).
