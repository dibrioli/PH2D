# ADR-0039 — FREEZE do contrato de nós (`ph2d-nodegraph` + `ph2d-expr`) — W2.T4

**Status:** Accepted (2026-05-22)
**Decisor(es):** Enio ("faça o melhor, vá até o fim") + Claude (arquiteto/implementador).
**Depende de:** ADR-0030 (decisão-mãe node-centric), ADR-0032 (substrato `ph2d-nodegraph`), ADR-0033 (`ph2d-expr`), ADR-0034 (avaliadores plurais).

## 1. Contexto

O modelo de execução do sistema de nós é um **funil**: um *neck serial* (o contrato compartilhado, que nenhum número de agentes acelera) → **FREEZE** → um *fan-out paralelo* (N agentes, um node-crate isolado cada, sem colisão). O fan-out só é seguro depois que o contrato está provado end-to-end e congelado — senão cada mudança no contrato rippla em todos os nós já escritos e re-serializa o paralelismo (o exato modo de falha que o ADR-0030/0031 existe para evitar).

A vertical Motion (W2) provou o contrato inteiro num caminho real:

- **W2.T1+T2** — avaliador `ph2d-eval-motion` (grafo → `Vec<RenderInstance>`, headless) + 3 nós reais `motion.{grid,transform,clone}`. Auditados 3× adversarialmente + re-auditados 2× a erro-zero (falhas silenciosas e overflow de cast remediados; `param_default`/`param_as_count` extraídos ao contrato).
- **W2.T3** — arch-gate da membrana rodando `Graph::validate` na registry real (recusa `Stateful`→pull e dim-mismatch) + smoke visual confirmado pelo Enio (27 sprites na tela, posições corretas).
- **Último gap de autoria** — param overrides por-instância (`Graph::set_param` + `EvalCtx::param` + `p` record no formato + `Violation::UnknownParam` + fingerprint que invalida o memo ao editar). Sem isso, todo grid era forçado 3×3: um "v1 que dá pro gasto". Como toca o contrato, foi landado **antes** do freeze (pós-freeze seria evento Coordenador-only com ripple).

## 2. Decisão

**Congelar a superfície de `ph2d-nodegraph` + `ph2d-expr`.** Concretamente:

1. **Caps do arch-gate apertados ao tamanho atual, sem folga** (`crates/ph2d-nodegraph/tests/architecture_contract_surface.rs`): `NodeOp` ≤ 2 métodos, `OpResolver` ≤ 1, `NodeManifest` ≤ 8 campos. Qualquer adição à superfície *que os node-crates implementam* passa a tripar o gate — é o que faz o freeze "morder".
2. **Marcadores 🔒 nos `lib.rs`** dos dois crates declarando o estado congelado e a data.
3. **Formato textual `v1` é a grammar congelada** (inclui o registro `p` de params). Qualquer registro novo pós-freeze bumpa para `v2`.

Mudar o contrato depois disto é um **evento raro, Coordenador-only**: bumpar o cap + escrever um ADR + re-provar a paridade CPU↔WGSL (no caso do `ph2d-expr`).

## 3. Consequências

**Aceitas:**
- O fan-out (ADR-0031 §fan-out, `docs/IntegracaoMultiAgente/briefing-node-crate.md`) está **aberto**: adicionar feature = largar um crate isolado, sem editar nada central (glob de `workspace.members` + `register_all_nodes` gerado).
- Crescimento acidental do contrato vira erro de CI (gate vermelho), não um drift silencioso.
- O que NÃO está coberto pelo cap (APIs aditivas de `Graph`/`Cook`/`EvalCtx` que os nós *consomem* mas não implementam) pode crescer aditivamente sem ripple — o cap mira de propósito só as superfícies *implementadas* (`NodeOp`/`OpResolver`) e o literal que todo nó escreve (`NodeManifest`).

**Custos / dívidas conhecidas (não bloqueiam o freeze; viram fan-out ou eventos Coordenador):**
- **Identidade de textura/atlas** nas instâncias Motion é uma coluna de stream sem produtor (revelado pelo smoke). É extensão de *convenção* (coluna nomeada), não mudança de contrato — item de fan-out.
- **Params vetor/enum**: hoje só `f32` escalar (casa com `ph2d-expr`). Estender o tipo de param é mudança de contrato.
- **Lowering Luau + gate HR-5**, **avaliador shader + runtime WGSL** para o domínio `Instances`, **paridade CPU↔GPU real** (device headless): tudo fan-out/pós-freeze.
- `would_cycle` O(V²) por `connect` — otimização só se surgirem grafos grandes.

## 4. Alternativas consideradas

- **Congelar antes dos param overrides** (abrir o fan-out mais cedo): rejeitado — congelaria um contrato onde params são decorativos, e adicionar autoria depois ripplaria em todos os nós do fan-out. O mandato padrão-ouro proíbe o "v1 que dá pro gasto".
- **Não apertar os caps** (deixar a folga de 10/4/2): rejeitado — sem apertar, o freeze não morde (adições caberiam na folga sem sinal). Apertar ao atual torna o gate o mecanismo real do freeze.
