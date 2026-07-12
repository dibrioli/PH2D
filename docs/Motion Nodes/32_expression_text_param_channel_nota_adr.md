# 32 — Nota-ADR: Expression + o canal de TEXT PARAM (M1 — o escape-hatch de fórmula)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Autorizado por:** Enio ("abre a expression"). **Escopo:** **FOUNDATIONAL + 1 drop-crate.** Toca o substrato
`ph2d-nodegraph` (Graph + EvalCtx + cook) de forma **ADITIVA** — e **PROVA que o contrato congelado segue
intacto** (`architecture_contract_surface` verde: `NodeManifest=8`/`NodeOp=2`/`OpResolver=1`). É a realização
**mais isolada** do M4.N1 ("ParamSpec tipado"): sem bumpar o contrato, sem quebrar as 68 crates de nó.

> **⚠ Reporte ao Enio (§0.2):** esta é a **PRIMEIRA fatia da linha a tocar o substrato** (`ph2d-nodegraph`).
> A story anterior era "zero substrato". A mudança é aditiva e o contrato congelado foi provado intacto, mas o
> integrador precisa saber (handoff §2 atualizado). **Não** foi um caso de PARE (não mexeu no contrato
> congelado nem em mesmo-símbolo) — a autorização "abre a expression" cobriu.

---

## 1. O problema

A `motion.expression` (fórmula editável) precisa de um **param string** — mas `ParamSpec { name, default: f32 }`
é **f32-only** (congelado por ADR-0039; o gate conta `NodeManifest ≤ 8 pub fields`, e adicionar campo quebra
TODAS as crates de nó que escrevem `NodeManifest {...}` literal). O M4.N1 planejava um `ParamSpec` tipado
(`ParamValue{F32,Vec2,Color,Enum,Bool}`) — mas isso **bumparia o `NodeManifest`** (fan-out breakage + gate
vermelho). Como dar um canal de string a UM nó sem quebrar o contrato de 68?

## 2. A decisão (a alternativa isolada ao M4.N1)

**Um canal de TEXT PARAM paralelo, aditivo, FORA do `NodeManifest`.** Os params f32 já vivem em
`Graph.node_params: BTreeMap<NodeId, BTreeMap<String, f32>>` — **não** no `NodeManifest`. Então:

- **`Graph.node_text_params: BTreeMap<NodeId, BTreeMap<String, String>>`** (campo novo) + `set_text_param` +
  `node_text_param_overrides`. Coberto por `Clone`/`PartialEq` derivados → **undo funciona**.
- **`EvalCtx.text_overrides` + `EvalCtx::text_param(name) -> Option<&str>`** — o nó lê seu texto no cook.
- **`Fingerprint.text_params`** (FNV-1a length-prefixed, como o dos params f32) → uma fórmula editada
  **re-cozinha** (não retorna stream stale).
- **`NodeManifest` INTOCADO** (8 campos). `NodeOp` (2) e `OpResolver` (1) intocados. `ph2d-expr` (também
  congelado) **consumido, não alterado**.

**Prova:** `cargo test -p ph2d-nodegraph --test architecture_contract_surface` = **3 pass** (8/2/1) DEPOIS da
mudança. Os 70 testes do nodegraph verdes. **Nenhum gate congelado disparou.** Esta é a lição: o contrato é a
*superfície* (NodeManifest/NodeOp/OpResolver), não o *armazenamento* de params — dá pra estender o segundo sem
tocar o primeiro. Recomendo ao Enio adotar isto como a forma canônica de "params não-f32" (ratificar como ADR
real, superseding o plano M4.N1 de bumpar o `NodeManifest`).

## 3. O que foi adicionado (fatia)

**Plumbing foundational (`ph2d-nodegraph`, aditivo):** `graph.rs` (node_text_params + set/get + remove_node) ·
`cook.rs` (EvalCtx.text_overrides + text_param + Fingerprint.text_params + text_params_fingerprint + threading
nos 2 sites de construção). Projetado p/ isolamento: campos/métodos **append-only**, comportamento existente
inalterado.

**`ph2d-node-motion-expression` (drop-crate, dep `ph2d-expr`):** `(in) → out(VALUE)`. Lê a fórmula via
`text_param("expr")`, **parseia** (módulo irmão `parse.rs`: parser VEX-lite recursive-descent → o `ph2d_expr::
Expr` congelado) uma vez por cook, avalia por elemento com `ph2d_expr::eval` (HR-5-**exempt**, presentation-side
por contrato do `ph2d-expr` — os transcendentais vivem lá, não no meu código). **Vars:** `i`/`n`/`t`/`f`
(índice normalizado) + colunas escalares do input + params `a`/`b`/`c`/`d`. **Erro de parse → campo zero**
(fallback; o editor badgeia). `Effect::Temporal` (lê `t`). `Pure` não serve — a fórmula pode variar no tempo.

**Cena boot — DUAS cenas** (`motion_demo_strobe.rs`, 12 nós):

```
ESQUERDA (spiral): grid → expr("cos(f*a+t)*f*4")/expr("sin(f*a+t)*f*4") → make_point → tint → move(−6) → output
DIREITA  (wave):   grid → expr("sin(t*2+f*a)*0.5+0.5") → color_ramp(t, Ice) → move(+6) → output
```

- **spiral** (x≈−6): 144 pontos plotados por fórmulas cos/sin (reusa o `make_point` da fatia 31); ambas leem
  `t` → o espiral **gira**.
- **wave** (x≈+6): um grid colorido por uma onda de expressão `sin(t·2 + f·a)` alimentando o `t` de um ramp Ice
  → a cor **rola** no tempo.

**Testes (11 unit + 3 integração):** parse.rs (3: precedência ×>+, funções/select/erros, unário/parênteses);
node (5: **fórmula-por-elemento** [prova o plumbing text-param ponta-a-ponta], lê-colunas-e-params, funções/n/
select, **erro→zero**, registra). Integração no shell: `the_spiral_is_plotted_and_rotates` (144 + espalha
radial + gira + esquerda) · `the_colour_wave_scrolls` (100 + >5 cores Ice + rola no tempo + direita) ·
`the_default_document_replays_deterministically` (bit-idêntico **na mesma máquina**).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco |
|---|---|---|
| **`Graph.node_text_params` + `set_text_param` + `node_text_param_overrides`** | `ph2d-nodegraph/graph.rs` (**SUBSTRATO**, aditivo) | mesmo-símbolo se outra linha tocar graph.rs; foundational-integrate.sh cobre |
| **`EvalCtx.text_param` + `.text_overrides` + `Fingerprint.text_params`** | `ph2d-nodegraph/cook.rs` (**SUBSTRATO**, aditivo) | idem |
| crate `ph2d-node-motion-expression`, tipo `motion.expression` (dep `ph2d-expr`) | nova | nome novo; `ph2d-expr` já no workspace |
| `ph2d-node-registry-init` regenerado (69 crates) | codegen | conflito → `cargo run -p ph2d-node-sync` |
| cena boot + motion_state* | shell | módulo Motion |

**Contrato congelado: INTOCADO e PROVADO (8/2/1).** `ph2d-expr` consumido, não alterado.

## 5. Disclosures (o que fica / caveats) — LER

- **✅ Serialização textual FECHADA (adendo 2026-07-11, "fecha a serialização"):** o `format.rs` ganhou o
  record **`x <id> <name> <formula…>`** — a fórmula é o **campo livre final** (tudo após o 3º espaço, espaços
  interiores preservados; o MESMO padrão do título de backdrop `b` do `ph2d-motion-doc`, **sem inventar
  escaping**). Um text param **bumpa o header pra `v2`** (record pós-freeze, pela política do próprio `format.rs`);
  grafo sem text param fica **byte-idêntico `v1`**; `from_text` aceita ambos. `MotionDoc` **delega** ao format
  (só anexa `[backdrop]`) → funciona transparente. **Round-trip testado** (fórmula com espaços+operadores;
  header v2; v1 ainda carrega; rejeições de nó-inexistente/malformado) + **prova end-to-end no shell** (a
  fórmula da boot doc sobrevive a `MotionDoc::to_text/from_text`). Limitação: fórmulas são **single-line**
  (whitespace de borda da linha é trimado). Contrato de nó **intocado** (só a gramática de serialização ganhou
  um record aditivo versionado).
- **⚠ Replay-hash cross-máquina:** `ph2d_expr::eval` usa transcendentais f32 (libm), que **variam entre
  máquinas/libm** (por contrato o `ph2d-expr` é presentation-side/HR-5-exempt). Dentro de um processo é
  determinístico (o teste de replay passa). Se algum **golden de replay-hash cross-máquina** cobrir a boot doc,
  a expression (sin/cos) pode divergir — re-lockar ou a boot doc evita fórmulas transcendentais no hash.
- **UI de texto no editor DEFERIDA:** não há `ParamWidget::Text` (isso tocaria `ph2d-node-registry`); a fórmula
  é setada por código (`set_text_param`) na cena. O campo de texto no painel de params é follow-up (editor).

## 6. O que fica

A `motion.expression` era o item de MAIOR valor que restava da cauda M1 — e o único ADR-gated. Feito (via o
caminho aditivo). O restante da cauda M1 é subsumido/marginal. Follow-ups desta fatia: ~~serialização textual~~
(FEITA, §5) · **UI de texto no painel** (editor; precisa `ParamWidget::Text` em `ph2d-node-registry`) ·
(opcional) lowering WGSL da expression (o `ph2d-expr` já tem `to_wgsl` — combina com o motor GPU futuro).
Fronteiras inalteradas: **M4** (Rig+FX) · **M5** (GPU). É hora de **integrar** as 17 fatias.
