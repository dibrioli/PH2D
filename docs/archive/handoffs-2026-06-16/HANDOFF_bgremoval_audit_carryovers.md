# HANDOFF — `ph2d-tool-bgremoval` audit carryovers (R7+R8+R9 incidente Painter)

**Origem:** Painter T1.6 audits R7/R8/R9 — múltiplas lentes adversariais (J1, T1, V1).
**Owner sugerido:** dono(a) do crate `ph2d-tool-bgremoval`.
**Status misto:** parte JÁ commitada (sem owner review), parte NÃO fixada (handoff).
**Data:** 2026-05-27.

---

## Context — por que o Painter agent tocou no bgremoval

Durante o T1.6 audit (Painter R7), as lentes adversariais expandiram fora do escopo Painter pra crates adjacentes (bgremoval, ph2d-color, etc.). Eu segui as recomendações e fiquei tocando bgremoval em R7, R8 e parte da R9. **Enio apontou o scope creep em R9 e revertei** — mas parte do trabalho R7 já estava no histórico via commits `5f7680c` e `90abf85` (este último absorveu meu commit via parallel-agent collision — vide `feedback-parallel-agent-collision`).

Resultado: bgremoval recebeu mudanças importantes **sem o owner do crate revisar**. Este HANDOFF lista o que landou e o que ficou de dívida, pra você decidir entre adopt-as-is, refactor, ou revert.

---

## Parte 1 — Mudanças JÁ commitadas em main (sem owner review)

Todas em commits Painter R7/R8 absorvidos:
- `5f7680c fix(painter): T1.6 R7 audit remediations — 5 lenses, padrão-ouro`
- `90abf85 feat(color-eq): Domain Transform denoise ...` (parallel-agent swallow)

### 1.1. `try_run_pipeline` + `PipelineError` em [algorithm/mod.rs](../crates/ph2d-tool-bgremoval/src/algorithm/mod.rs)

R7 J1-4: o `run_pipeline` panicava em 3 `assert_eq!` (rgba length / protect length / force_remove length). Eu adicionei variante fallible:

```rust
pub fn try_run_pipeline(...) -> Result<(), PipelineError>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PipelineError {
    BufferShape { which: &'static str, actual: usize, expected: usize },
}
```

R8 N1-4 adicionou 4 tests em `algorithm::tests` (rgba mismatch + protect mismatch + force_remove mismatch + ok happy path).

R7 M1-2 / R8 R9 documenta que `scratch.ensure` NÃO é chamado antes dos shape checks — caller MUST NOT assumir scratch coerente após Err.

**Decisão do owner:** manter, refactor, ou revert? Se manter, considerar migrar os 3 sites de `tool.rs` (vide Parte 2 §2.2) pra usar a fallible.

### 1.2. `#[non_exhaustive]` em `BgRemovalUiEdit` em [params.rs:371](../crates/ph2d-tool-bgremoval/src/params.rs#L371)

R7 I1-1: enum recebeu `+ToggleAddArea` e `+ClearAddedAreas` em recovery commits, indo de 16 → 18 variants. Sem `non_exhaustive` é breaking change pra downstream que faz match exhaustive. Adicionei.

**Decisão do owner:** intencional? Adopt-as-is provavelmente correto, mas você decide.

### 1.3. `bgremoval_preview::dispatch` ganhou `toasts: &mut ToastQueue` param

R7 J1-3: em [`shells/desktop/src/render_loop/bgremoval_preview.rs:60`](../shells/desktop/src/render_loop/bgremoval_preview.rs#L60). GPU upload failure em :297 antes era `eprintln!` silencioso, agora vira `Toast::error(...)`. Caller em `render_loop/mod.rs` passa o `toasts` que já tinha em escopo.

**Decisão do owner:** OK; mas o pattern de toast inconsistente entre dispatch sites (apenas bgremoval_preview tem) é V1-MED da R9 — vide §3.5.

---

## Parte 2 — Achados NÃO fixados (dívida pendente)

### 2.1. T1-C1 CRITICAL — overflow em [`scratch.rs::ensure`](../crates/ph2d-tool-bgremoval/src/scratch.rs#L168)

```rust
let n = (w as usize) * (h as usize);    // unchecked multiply
let guide_n = if color_guide { n * 3 } else { n };  // unchecked
self.output_rgba.resize(n * 4, 0);    // unchecked × 4
```

PH2D ships 64-bit desktop, então `usize::MAX = u64::MAX` e `u32 × u32` sempre cabe. **Mas:**
- A multiplicação é `usize × usize`, não `u32 × u32`. Em hypothetical 32-bit port saturaria silenciosamente.
- Mesmo em 64-bit, se um caller passar `w = u32::MAX` (improvável mas não impossível), `n * 4` ainda cabe (16 EB) — porém o `Vec::resize` daria OOM panic antes.

**Fix sugerido:** `checked_mul()` + `expect("...")` claro, ou retornar `PipelineError::BufferShape` do `try_run_pipeline` antes de chamar `scratch.ensure`.

**Por que não fixei:** scope creep — não toquei mais nada em bgremoval em R9 quando vi a regra.

### 2.2. T1-H9 HIGH — `tool.rs` ainda chama `run_pipeline` (panic) em 3 sites

[`tool.rs:956, 1033, 1331`](../crates/ph2d-tool-bgremoval/src/tool.rs#L956). A versão fallible `try_run_pipeline` (§1.1 acima) já existe mas nenhum site usa. Em prática, o bridge gerencia source_w/source_h cuidadosamente então o panic é raro — mas qualquer regressão em `silhouette` mudando ROI ou `color_equalization_bridge::collect_live_bakes` passando slices mismatched aborta o render thread.

**Fix sugerido:** wrap cada call em `if let Err(err) = try_run_pipeline(...) { eprintln!(...); /* clear out; return early */ }`. Esquema (R9 lens T1 detalha):
- `run_full_resolution` (linha 944): retornar `(source_w, source_h)` com `out.clear()` em err.
- `run_canvas_preview` (linha 1018): `return (cw, ch)` com `out.clear()`.
- `rerun_preview` (linha 1328): early-return com `preview_rgba.clear()`.

**Por que não fixei:** mesmo motivo da §2.1.

### 2.3. V1-MED — toast queue param inconsistente entre dispatch sites

Hoje só [`bgremoval_preview::dispatch`](../shells/desktop/src/render_loop/bgremoval_preview.rs#L60) recebe `toasts: &mut ToastQueue`. Outros bridges (painter_bridge, color_equalization_bridge, padding_bridge) não. Isso significa que `bgremoval` consegue surfaçar errors via toast no dispatch path; outros tools não, têm que rotear via drain function.

**Decisão do owner:** se quiser uniformizar, é um sweep de assinatura cross-tool. Talvez melhor refactorar via `ToolContext { toasts, sim, renderer, ... }` em vez de adicionar 1 param a cada call site.

---

## Validação

- Painter T1.6 R9 commit `7fed63b` valida painter-brush 154/154 + tool-painter 28/28 + smoke_env_contract 11/11 (escopo Painter only).
- Bgremoval test suite continua passando: `cargo test -p ph2d-tool-bgremoval --lib --tests` → 130+/130+ (incluindo os 4 novos `try_run_pipeline` tests da R8 N1-4).
- **Workspace check NÃO está verde hoje** — `ph2d-panel-color-equalization` tem WIP alheia (campo `denoise_method` referenciado mas removido do snapshot). Não é bgremoval mas convém checar antes de qualquer commit.

## Cross-ref

- Audit transcripts: `/private/tmp/.../tasks/a64d080512cb95689.output` (R7 J1), `/private/tmp/.../tasks/abca463d37bb26cf7.output` (R9 T1), `/private/tmp/.../tasks/af31041832a38e98d.output` (R9 V1).
- Commits afetados: `5f7680c`, `90abf85`, `61a1428` (attribution note R8), `7fed63b` (R9 Painter scope correto).
- Memory `feedback_audit_scope_discipline` (regra que me fez parar).
- Memory `feedback_parallel_agent_collision` (explicação do swallow `5f7680c → 90abf85`).
