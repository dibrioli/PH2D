# 05 — Nota-ADR: escopos de tempo no Cook (`cook_scoped`) — M2.N1/N4

**Data:** 2026-07-09 · **Linha:** `line/MotionNodes` · **Status:** implementado, gates verdes.
**Escopo:** a única mexida no substrato prevista pelo plano (§1.5). **Aditiva** — nenhuma API
existente mudou de assinatura ou de comportamento.
**Contratos congelados:** intocados (`NodeOp=2` / `OpResolver=1` / `NodeManifest=8` seguem
exatamente iguais; o gate `architecture_contract_surface` só conta esses três).

---

## 1. O problema

`motion.time_remap` precisa reamostrar a **sub-árvore acima dele** em outro tempo (slow-mo no
impacto, rig em loop, still congelado com física correndo por baixo, explosão em reverso).
Nenhum nó pode fazer isso sozinho: um nó só enxerga seus inputs já resolvidos (`EvalCtx`, caixa-preta
FBP do ADR-0031). **Quem escolhe o playhead do upstream é o puxador**, não o nó. Logo, o remap
mora no `Cook`.

## 2. O que foi adicionado

**`ph2d-nodegraph/src/time.rs` (módulo novo, folha, zero deps internas):**
- `TimeMode { Scale, Loop, PingPong, Freeze, Reverse }` — vocabulário do catálogo de referência.
- `TimeMap { mode, scale, offset, duration }` com `apply(t) -> t'`, `is_identity()`, `hash_into()`.
- HR-5: só `+ - * floor/rem_euclid` em `f64`. Sem transcendentais → replay remapeado é bit-exato
  cross-platform. `duration ≤ 0` (ou não-finita) **degrada para `Scale`** em vez de gerar `NaN`
  (um `NaN` no playhead envenenaria todo fingerprint downstream: `NaN != NaN` ⇒ o memo nunca mais
  acerta; teste `a_degenerate_duration_degrades_to_scale_instead_of_nan`).

**`ph2d-nodegraph/src/cook.rs` (aditivo):**
- `type ScopeKey = u64` + `const SCOPE_ROOT = 0` + `type TimeScopes = BTreeMap<NodeId, TimeMap>`.
- `Cook::cook_scoped(...)` e `Cook::advance_tick_scoped(...)`. Os antigos `cook`/`advance_tick`
  **delegam com um mapa vazio** → toda linha paralela que nunca remapeia compila e se comporta
  exatamente como antes (isolamento por extensão, CLAUDE.md §0.2).
- **Cache re-chaveado `(NodeId, ScopeKey)`.** Um nó alcançado por duas cadeias de escopo no mesmo
  frame (diamante com um braço cruzando o remapper) cozinha uma vez **por cadeia**.
- `CookError::SequentialInTimeScope { node }` — variante nova (não gateada).
- **Poda de lanes**: `advance_tick_scoped` descarta as lanes não visitadas no frame.

**`ph2d-node-motion-time-remap` (crate nova, drop-crate):** o nó (passthrough puro) + `MODE_LABELS`
+ `time_map_from()` + `time_scopes(graph, ops)` + `scopes_a_sequential_node(graph, node)`.
O **substrato não conhece tipos de nó** (escopos são chaveados por `NodeId`); quem traduz
"este tipo significa remap" é a crate que possui o tipo. Regenerado por `ph2d-node-sync` (24 crates).

**`ph2d-eval-motion`:** `MotionCookPump::pump_scoped(...)` (aditivo; `pump` delega) + `last_error()`.
**Shell:** monta os escopos por frame e **recusa o fio** que arrastaria um nó sequencial para
dentro de um escopo, com a razão no toast.

## 3. As três decisões que valem registro

### 3.1 Nó sequencial dentro de escopo remapeado = **recusado** (não "adaptado")

Um `spring`/`integrate` integra uma recorrência sobre o **tick externo**. Sob `Loop` ele seria
convidado a reviver ticks que já integrou; sob `Freeze`, a avançar enquanto o tempo não anda.
Não existe leitura correta — então o cook **erra alto** (`SequentialInTimeScope`) em vez de
produzir uma trajetória plausível e errada. É a restrição v1 que o plano §1.5 já previa.

O editor não deixa chegar lá: a guarda de conexão recusa o fio e **explica**. O erro de cook fica
como backstop (documento carregado de disco, edição por MCP). Falsificado em
`a_wire_dragging_a_sequential_node_under_a_time_remap_is_refused`: a guarda recusa **exatamente**
o grafo em que `cook_scoped` erra.

### 3.2 "Loop vem de cache" — a promessa do plano §1.5 é **parcialmente** verdadeira

O plano dizia: *"Loop → subtree vem do cache quando `t'` repete (o MVP recomputava)"*. Medi.

- **Freeze É de graça** — `t'` constante ⇒ o fingerprint do upstream nunca muda ⇒ **uma cook**
  para a vida inteira do still, memo daí em diante. Provado por contador em
  `a_freeze_scope_costs_one_cook_no_matter_the_playhead` (1 eval em 10 frames; sem escopo, 10).
- **Loop NÃO vem do cache entre voltas.** O memo é **single-slot por `(node, scope)`**: guarda a
  última cook, não um mapa `playhead → valor`. Uma volta que retorna 2 s depois encontra o slot
  com outra fase e recomputa. Fazer voltas serem de graça exige um memo **multi-slot chaveado por
  playhead com orçamento de evicção** — exatamente a *"política de cache explícita por nó +
  orçamento de memória"* que o ADR-0032 §3 já registra como trabalho futuro.

O que o `Loop` entrega hoje é **correção** (janela cíclica, diamante correto) e o custo é o mesmo
de antes. Não inflei o teste para casar com a promessa: `a_loop_scope_cycles_the_upstream_clock`
assevera o ciclo e documenta a ausência do ganho de cache.

### 3.3 Poda de lanes: um bug de memória que o design escopado cria

Cada valor de param do remapper gera um `ScopeKey` novo (o hash inclui os bits do `TimeMap`).
**Arrastar um slider** produziria uma lane por valor intermediário, cada uma segurando `Stream`s
completos da sub-árvore — vazamento silencioso pela vida do processo. `advance_tick_scoped` agora
descarta, a cada tick, as lanes não visitadas naquele frame; a lane raiz (`SCOPE_ROOT`) nunca é
podada, pois é o memo do grafo. Gate: `stale_scope_lanes_are_pruned_when_the_tick_advances`
(20 valores de offset ⇒ ≤ 4 entradas, não 40+).

## 4. HR-18 — `cook.rs` dividido, allowlist **retirada**

Minhas adições passaram `cook.rs` de 864 → 1269 LOC (cap gravado 864). Em vez de subir a entrada,
**dividi por responsabilidade** (a instrução do próprio gate): engine em `cook.rs` (459 LOC, agora
sob o cap simples de 700), testes em `cook_tests.rs` (harness + suíte existente) e
`cook_scope_tests.rs` (suíte de escopos). A entrada `("ph2d-nodegraph/src/cook.rs", 864)` foi
**deletada** de `FILE_OVERAGE_OK`.

## 5. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| `time::{TimeMap, TimeMode}` | `ph2d-nodegraph` (módulo novo) | baixo (nome novo) |
| `cook::{ScopeKey, SCOPE_ROOT, TimeScopes}` | `ph2d-nodegraph` | baixo |
| `Cook::cook_scoped` / `advance_tick_scoped` | `ph2d-nodegraph` | **aditivo** (antigos delegam) |
| `CookError::SequentialInTimeScope` | `ph2d-nodegraph` | variante nova; `match` exaustivo alheio quebra |
| `cook.rs` → + `cook_tests.rs` + `cook_scope_tests.rs` | `ph2d-nodegraph` | **arquivo dividido** (merge textual) |
| `FILE_OVERAGE_OK` sem a entrada de `cook.rs` | `ph2d-editor-core/tests` | linha removida |
| `MotionCookPump::pump_scoped` / `last_error` | `ph2d-eval-motion` | aditivo |
| crate `ph2d-node-motion-time-remap`, tipo `motion.time_remap` | nova | nome novo |
| `ph2d-node-registry-init` regenerado (24 crates) | codegen | **conflito provável** com outra linha que adicione nó |

## 6. O que isto destrava

`cook_scoped` é o gargalo **M2.N1** do plano. Com ele no chão:
- `motion.time_remap` (entregue);
- **trail/echo scrub-safe** ao estilo Houdini (recozinhar `t-k` em vez de acumular estado);
- a **Zona de Simulação** (O4 do [`03_reentrada_integrate_estudo_padrao_ouro.md`](03_reentrada_integrate_estudo_padrao_ouro.md)),
  que precisa exatamente de "cozinhar esta sub-árvore sob outro regime".

Follow-ups honestos, nomeados: **(a)** memo multi-slot chaveado por playhead + evicção (torna
`Loop`/scrub baratos e é pré-requisito do `Cook::checkpoint/restore`, M2.N2); **(b)** badge visual
no nó recusado (hoje o aviso é um toast na conexão + `last_error()` disponível ao shell).
