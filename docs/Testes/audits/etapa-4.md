# Audit adversarial — Wave 10 / Etapa 4

**Data:** 2026-05-24
**Auditor:** 1 agente `general-purpose` (escopo isolado: 3 codegens novos)

---

## Achado CRITICAL (corrigido pré-commit)

### [C-1] panel-sync emitia linha que rustfmt reformatava → staleness gate quebraria

**Onde:** `tools/ph2d-panel-sync/src/lib.rs::render_register_lines`
**Sintoma:** Tipos longos (`ph2d_panel_color_equalization::ColorEqualizationPanel` etc.) ultrapassavam 100 cols. Codegen emitia 1-linha; `cargo fmt` reformatava pra multi-linha (3 linhas: `ErasedPanel::new::<\n        type,\n    >()`). Próxima execução do staleness gate compara on-disk (multi-linha) vs render fresh (1-linha) → mismatch → CI vermelho.

**Repro pelo audit:** rodar `cargo run -p ph2d-panel-sync` → gate verde; `cargo fmt -p ph2d-panel-registry-init` → gate VERMELHO. Confirmado.

**Fix padrão-ouro (não-frágil):**

1. **Gate compara semanticamente** via `extract_registered_panels` helper novo no `panel-sync` lib. Parser whitespace-tolerante extrai `(crate_ident, struct_name)` pairs do bloco entre markers. Comparison de pairs vs fresh scan — formatação irrelevante.
2. **Main do sync chama `cargo fmt -p ph2d-panel-registry-init`** ao final do splice, deixando o arquivo on-disk em estado canônico imediatamente. Próximo pre-commit não muta o arquivo.

Combinação dos dois: o gate é robusto a formatação (não falha) E o on-disk é canônico (não pisa em pre-commit).

**Status:** ✅ FIXED com testes verdes (`cargo test -p ph2d-panel-registry-init --tests` → 4/4).

---

## Achados MÉDIO/BAIXO (anotados — Etapa 5+ ou aceitos)

### [M-1] `default = [...]` array em `panel-registry-init/Cargo.toml` é hand-written

Cenário: adicionar painel novo → sync regenera deps + features mas o `default` array NÃO. Painel some silenciosamente em runtime se esquecer de adicionar lá.

**Mitigação Etapa 4:** doc-comment explícito no Cargo.toml + smoke G9 do Enio captura. **Fix completo (sync também regenera `default`)** fica como follow-up Etapa 5 — exige decisão de política (todo painel novo entra em default? Coord-A decide).

**Status:** 📝 ANOTADO

### [M-2] `EXPECTED_TYPED` const hand-written em `panel-registry-init/src/lib.rs`

Hand-mantido com `#[cfg(feature)] { n += 1; }` x 9. Mesma classe de drift que M-1.

**Mitigação:** existing test `build_typed_registry_matches_enabled_features` cobre consistência interna. Drift por adição de painel novo pego pelo staleness gate principal (semântico). Fix completo: sync regenera com marker separado.

**Status:** 📝 ANOTADO

### [M-3] `dispatch_all_referenced_handlers` parser tem false-positive com `// foo::apply` em comentário

Token `io_menu::apply` dentro de comment `// removed: io_menu::apply` seria contado como dispatched. Hoje não ocorre; risco baixo.

**Status:** 📝 ANOTADO

### [B-1..B-5] Pequenas observações de robustez

Listadas no relatório original — body parser frágil a bloco aninhado, `panel_struct_name` não cobre `#[derive] pub struct` same-line, etc. Todos cenários hipotéticos sem ocorrência real.

**Status:** 📝 ANOTADO

---

## Tests gaps reconhecidos

- Falta sub-test default-array-in-sync (cobre M-1).
- Falta sub-test EXPECTED_TYPED-vs-scan (cobre M-2).
- Falta fmt-roundtrip test "rendered output sobrevive `cargo fmt` sem mudar" — substituído pelo approach semantic-comparison.

---

## Veredito final

**Pronto para commit como Etapa 4 padrão-ouro.** C-1 fixado robustamente (não-frágil), 5 follow-ups menores anotados. Todos os 6 staleness gates verdes; workspace + clippy + fmt clean.

**Smoke do Enio** crítico em G6-G9 do README §E4 (verifica que drop-in funciona pra cada family). Especialmente G9 (regressão visual completa) — confirma que a Etapa 4 não tocou runtime path.
