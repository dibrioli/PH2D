# ADR-0032 — `ph2d-nodegraph`: substrato unificado (atributos, portas algébricas, efeitos, formato textual)

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto).
**Depende de:** ADR-0030 (decisão-mãe).

## 1. Contexto

A reconciliação do ADR-0030 exige um substrato **único** que carregue tudo que é unificado entre domínios, deixando só avaliação/lowering/view como plural. Esse substrato é o contrato compartilhado por todos os agentes que escrevem nós; precisa ser fino e **estável** (raramente tocado), senão vira god-crate e mata o paralelismo.

## 2. Decisão

Criar o crate `ph2d-nodegraph` com **sete primitivos**:

1. **Modelo de atributos** (sabor Houdini): arestas carregam colunas tipadas sobre um domínio (pixels, samples, instâncias, entidades), não um escalar.
2. **Tipos de porta algébricos** que carregam **domínio + dimensionalidade + relógio** — ex.: `Field<f32, clock=Frame>` ≠ `Field<f32, clock=Audio>`. (Conserta o vazamento "mesmo tipo, taxa de amostragem diferente".)
3. **Sistema de efeitos** `Pure | Temporal | Stateful` + regra da membrana (`Stateful`/push não conecta no lado pull), **checado em compile-time**.
4. **Aciclicidade por construção** + operador de delay tipado de 1-tick (`pre`, à la Lustre). Feedback temporal é `pre`, nunca aresta-de-volta. Sem detecção de ciclo em runtime.
5. **Fields / atributos anônimos** compartilhados (sabor Blender) — sede do compute reusável; ver ADR-0033.
6. **Formato textual diffável/mergeável**: lista estável de nós + arestas, **IDs estáveis**, **layout segregado da semântica** (posição num campo que nunca afeta o cook). Requisito multi-agente + save/migração.
7. **Registry de tipos de porta**: vocabulário compartilhado que faz nós de crates diferentes encaixarem.

**Motor de cook único genérico:** topo-sort + dirty-propagation + cache (modelo incremental/self-adjusting), com **política de cache explícita por nó + orçamento de memória**.

## 3. Consequências

**Aceitas:**
- Os três vazamentos da reconciliação viram **erros de compilação**, não bugs de runtime (clock no tipo, efeito checado, lowering plural).
- Formato textual habilita `git diff`/merge legível entre agentes e migração de save de grafos (ADR-0037 generaliza pro `SceneDoc`).

**Riscos:**
- Crescimento descontrolado do contrato → arch-gate de surface (espelha `architecture_panel_host_surface` do ADR-0029); mudança de substrato = evento raro Coordenador-only.
- Cache implícito universal (problema #1 de perf do Houdini) → política explícita por nó desde o dia 1.

## 4. Alternativas consideradas

- **Portas escalares simples (sem domínio/clock no tipo):** rejeitado — é a raiz do vazamento de taxa de amostragem; o `Field` a 60Hz alimentando synth a 48kHz compila e produz lixo.
- **Detecção de ciclo em runtime:** rejeitado — custa e é não-determinístico se a quebra for ad-hoc; `pre` + DAG resolve por construção.
- **Dirty-bit ingênuo:** aceitável como v1, mas o alvo é incremental composável (Adapton-like) para reuso de subcomputação sob mutação.

## 5. Nota de implementação (2026-05-22, pós-auditoria)

- **§2 item 7 ("Registry de tipos de porta") → enum fechado.** Na implementação, o "vocabulário de portas" virou os enums fechados `Domain`/`Dim`/`Clock` em `port.rs`, **não** um registry dinâmico. É a escolha superior: enum fechado é determinístico, checável em compile-time e não tem superfície de extensão dinâmica para um agente desincronizar. O "registry" do crate `ph2d-node-registry` é de **node ops**, não de tipos de porta. Decisão ratificada: o enum vence; este item do §2 fica como histórico.
- **Membrana + tipos são aplicados por `Graph::validate(&dyn OpResolver)`**, não por `connect` (que só garante invariantes estruturais — aciclicidade, uma-aresta-por-porta). Adicionado na remediação da 1ª auditoria (`6489f70`).
- **`pre` (synchronous-dataflow):** `Cook::advance_tick(graph, ops, playhead)` cozinha as fontes de `pre` antes do snapshot — uma fonte de `pre` sem consumidor forward ainda avança (correção da auditoria 2026-05-22).
- **`NodeManifest` ainda não tem `lowerings[]` nem `params`** — chegam com `ph2d-expr` (ADR-0033) ANTES do freeze (vide `docs/plans/2026-05-node-waves.md`). Congelar o contrato antes disso é prematuro.
