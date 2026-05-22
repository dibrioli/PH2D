# ADR-0031 — Nó e ferramenta como unidade de feature (isolamento FBP = unidade multi-agente)

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto).
**Estende:** ADR-0027 (convention-by-discovery / tool-as-crate) às duas famílias de feature.
**Depende de:** ADR-0030 (substrato), e do doc irmão [`2026-05-foundational-parallelism-three-bottlenecks.md`](../../Migracao/2026-05-foundational-parallelism-three-bottlenecks.md).

## 1. Contexto

O objetivo declarado do projeto é **agentes paralelos sem colisão**. A teoria de dataflow (Flow-Based Programming) e a investigação convergiram numa percepção: um nó bem-tipado e puro — caixa-preta FBP com **tipos de porta + classe de efeito como único contrato, zero estado compartilhado** — é **ao mesmo tempo** o design teoricamente ótimo de nó **e** exatamente a unidade que um agente paralelo constrói em isolamento e testa sozinho. O problema multi-agente e o sistema de nós são o **mesmo princípio**.

## 2. Decisão

A engine cresce por **duas famílias de crate isolado**, ambas wire-adas por codegen (ADR-0027 + Gargalo 2 do doc irmão):

1. **Nós** — `ph2d-node-<domínio>-<slug>/`: declarativos, pull/sync, caixa-preta FBP. Contrato do agente = `(portas_in, portas_out, classe_de_efeito, clock, lowerings[])`. O agente **não** vê o resto do grafo, nem estado compartilhado, nem o agendador.
2. **Ferramentas** — `ph2d-tool-<slug>/`: imperativas, push, manipulação direta (painter, Image Tools, brush, bgremoval). ADR-0027 existente. São tools **terminais** (ADR-0038), não rampa pro grafo.

Cada unidade: crate isolado + (eval puro | handler) + **uma linha codegen'd** de registro + dep só do contrato fino. Zero toque no core, zero colisão, build incremental pequeno, compila no slot do agente.

**Bridge bidirecional** entre as famílias: saída de nó pintável-por-cima; máscara pintada usável como input de nó; manipulação direta edita parâmetros de nó; bake/flatten de resultado de nó pra camada imperativa.

## 3. Consequências

**Aceitas:**
- O vocabulário compartilhado (registry de tipos de porta + sistema de efeitos do ADR-0032) é o que faz nós de **agentes diferentes, em crates diferentes, encaixarem** — maximiza superfície de contrato, minimiza superfície de coordenação.
- Cada nó puro é um **teste de propriedade autocontido** (golden input → golden output) — o agente valida sem rodar o grafo inteiro.
- A engine, daí em diante, cresce por adição de crate isolado — o "desimpedimento total para múltiplos agentes" buscado desde a abertura da discussão.

**Riscos:**
- Sem o substrato fino + codegen + build-por-slot (três gargalos), node/tool-as-crate **não** é paralelo. Os três gargalos são pré-requisito.

## 4. Alternativas consideradas

- **Carregamento dinâmico (WASM/dylib) para UI first-party:** rejeitado (sem ABI Rust estável; UI cruzando boundary GPU/allocator/AccessKit; atrito com HR-3). A lane de extensibilidade sandboxed é o Luau.
- **`linkme`/`inventory` para o registry:** rejeitado (ordem dependente de link-order, atrito com determinismo). Codegen dá lista explícita/diffável/determinística (Gargalo 2 do doc irmão).
- **Nós como módulos dentro de um god-crate** (estilo widget/ atual em editor-core): rejeitado — é exatamente o anti-padrão que o doc dos três gargalos identifica.
