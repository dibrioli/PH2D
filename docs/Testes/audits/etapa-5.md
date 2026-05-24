# Audit adversarial — Wave 10 / Etapa 5

**Data:** 2026-05-24
**Auditor:** 1 agente `general-purpose` (escopo: 5 sub-etapas — 5 gates UI estendidos + LOC cap + 4 gates ortogonais + ph2d-color + arch_color_space_typed)

---

## Achado CRITICAL (corrigido pré-commit)

### [C-1] `Premultiplied<SrgbRgba>::unmultiply` faltava

**Onde:** `crates/ph2d-color/src/premultiplied.rs` (impl `Premultiplied<SrgbRgba>`)
**Sintoma:** O crate-level doc (`lib.rs:34`) anunciava o pipeline `unmultiply → linear math → premultiply` na forma sRGB on-disk, e a mensagem de erro do `arch_color_space_typed` apontava consumers para `Premultiplied<&[SrgbRgba]>`. Mas o impl `Premultiplied<SrgbRgba>` só tinha `premultiply` — sem o inverso. Primeiro consumer no sweep §5.4 bateria em método ausente.

**Fix padrão-ouro:** adicionar `unmultiply` com byte-math `(x * 255 + a/2) / a` + guarda zero-alpha (`a == 0 ⇒ (0,0,0,0)`, paridade com `Premultiplied<LinearRgba>::unmultiply`). Documentação alerta que sRGB premultiplied não é estritamente correto sob transferência gamma — para fidelidade de filtro, usar `Premultiplied<LinearRgba>`.

**Testes adicionados:** `srgb_round_trip_within_quantization_error` (bounded por `ceil(256/a) + 1` — captura o erro intrínseco do 8-bit) + `srgb_transparent_unmultiply_is_zero` (cobre o guard).

**Status:** ✅ FIXED com 17/17 tests verdes em `cargo test -p ph2d-color`.

---

## Achados MEDIUM (anotados — refinos Etapa 6)

### [M-1] `arch_mode_has_reconcile` aceita qualquer `.method(` como evidência

**Sintoma:** O detector `body_has_method_call` aceita QUALQUER `.<ident>(` como sinal de reconciliação. Um setter bugado `self.mode = on; self.toast.show("...");` PASSARIA na gate (`.show(` é method call) mas mantém o padrão de bug §2 (Image Tools Bugs README:126-140) vivo.

**Mitigação Etapa 5:** os 6 setters atuais do workspace estão TODOS legítimos (4 fazem reconciliação real via wgpu/atlas/sampler; 2 são BENIGN explícitos). O gate previne novos setters bare-field-write — vale parcialmente.

**Fix completo Etapa 6:** trocar pra exigência explícita (`reconcile_*` / `invalidate_*` / `reset_*` keyword), ou per-symbol REQUIRED list que liga cada setter ao reconcile companion esperado. Esforço: 2-3 horas.

**Status:** 📝 ANOTADO para Etapa 6.

### [M-2] `arch_color_space_typed` BASELINE usa `canonicalize().ok() == ok()`

**Sintoma:** Se `canonicalize()` falhar em qualquer lado (link quebrado, arquivo deletado), `None == None` poderia silenciosamente tratar TODO mismatch como hit OU todo match como miss. Os 10 baseline files existem hoje — risco zero atual.

**Fix:** trocar pra comparação direta de path normalizada (`workspace_root.join(b) == path`). Esforço: 5 minutos.

**Status:** 📝 ANOTADO para refino curto.

---

## Achados LOW/INFO (anotados — sem ação imediata)

### [L-1] Markers detectados via `line.contains("...")` permitem smuggling via string literal

Marcadores (`LITERAL-PX-OK`, `CLAMP-OK`, etc.) são matched com `line.contains()` cru. Linha `let s = "CLAMP-OK";` se auto-exemptaria. Zero código atual faz isso; risco baixo. Fix opcional: exigir marker em comment trailing `//`.

### [L-2] CEQ render order observation: scrollbar paints OVER popovers (pré-existente)

CEQ paint.rs original (pre-Etapa 5) JÁ pintava scrollbar depois de popovers (linhas 524 vs 616+). O split preservou byte-por-byte. Mas `docs/Testes/README.md` G10 dizia "popover em cima do scrollbar" — descrição estava errada OU bug visual pré-existente.

**Veredito:** comportamento do split idêntico ao pré-existente; smoke G10 reescrito (esta etapa) com texto correto.

### [L-3] Marker em string literal `"0.00"` no bgremoval/paint.rs (ruído harmless)

Linha 136 tem marker no `"0.00".to_string()`. O `no_magic_numeric` matcher só procura padrões `\d+\.\d+` em FLOAT literals, não chars dentro de strings — então o marker é desnecessário ali. Ficou após o fix C-1 da Etapa 4. Harmless.

### [L-4] `ph2d-color` ainda sem consumer

Nenhum `Cargo.toml` declara `ph2d-color` como dep. Documentado no plan: migração dos 10 sites baseline é Etapa 5.4 follow-up (1-2 semanas). Aceitável como intent.

### [L-5] Token parity confirmed

Auditor verificou: `ROW_H_PX=28`, `Spacing::Lg=12`, `Spacing::Sm=6`, `TypeToken::Base=13`, `Spacing::Xl3=32`, `Density::Compact=22`, `StrokeToken::Default=1.5`, `Spacing::Xl4=48`, `Spacing::Xs=4`, `Spacing::Xl=16`. Todos os 60+ substitutions resolvem ao mesmo valor numérico do literal original.

`safe_clamp` preserva o hand-rolled swap-guard do panel_chrome (`min > max ⇒ swap`).

### [L-6] Cargo.lock + docs/Painter_projeto/

- Cargo.lock diff é APENAS a entrada do `ph2d-color` package. Legítimo.
- `docs/Painter_projeto/` é WIP pré-existente (mtimes 23/maio) — NÃO incluído neste commit.

---

## Veredito final

**Pronto para commit como Etapa 5 padrão-ouro.** C-1 fixado robustamente (unmultiply + 2 novos tests), 2 follow-ups MEDIUM anotados para Etapa 6, 6 LOW/info documentados.

**Stats:**
- **Testes automáticos:** 17/17 ph2d-color + 11/11 gates UI estendidos + 12/12 gates ortogonais + 2/2 LOC cap = **42 sub-tests verdes** novos/estendidos
- **Sites tocados:** 69 violations migradas + 7 safe_clamp/markers + 6 panel-* + CEQ split (3 arquivos) = ~85 LOC de mudanças cirúrgicas
- **Novo código:** ph2d-color (~470 LOC + 17 tests) + math.rs (~50 LOC + 3 tests) + 5 gates ortogonais novos
- **Gates ativos pós-Etapa 5:** 5 UI estendidos + 4 ortogonais + 1 LOC cap + 1 color-space = **11 gates novos/estendidos**

**Smoke do Enio crítico em G10-G14 do README §E5** (especialmente G10 CEQ pós-split — confirma que paint orchestration preserva render order byte-equal).
