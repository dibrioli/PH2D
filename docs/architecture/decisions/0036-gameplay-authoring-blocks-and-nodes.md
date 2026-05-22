# ADR-0036 — Autoria de gameplay: blocos e node-programming → Luau; colisão 2D lite

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto).
**Depende de:** ADR-0019 (Luau), ADR-0025 (GameObject), ADR-0021 (Sim/Present), ADR-0030.

## 1. Contexto

O domínio gameplay já está ~80% pronto: `ph2d-script::host` tem escrita diferida (`EntityWrite`/`drain_writes`), `SpawnQueue`, `StateTable` (HR-16), `InputSnapshot`, messaging Defold, Scheduler/corrotinas. A API `ph2d.*` do protótipo espelhou a PH2D existente com fidelidade. O Enio quer **dois** estilos de autoria de gameplay: blocos (Scratch-style, sem fio) **e** node-programming (Blueprint-style, com fio).

## 2. Decisão

**Duas superfícies de autoria, uma IR, um runtime mínimo:**
- **Blocos** — `ph2d-blocks`, **authoring-time** (vive no editor). Compila a IR de blocos → Luau.
- **Node-programming** — domínio do grafo (ADR-0030), compila via `ph2d-expr`/`ph2d-script`.
- Ambos baixam para **Luau/bytecode**; o runtime (`ph2d-script`) não conhece a diferença. Reusa HR-10 (paridade MCP) e HR-17 (examples). **Compilador fora do runtime.**

**Hats → handlers:** `on_start`/`on_tick`/`on_msg_*` registro direto (existe); `on_state_*` via componente FSM lendo `state_table`; `on_collide_<tag>` via `ph2d-collision2d` emitindo **mensagem** (colisão = sistema ECS produtor de eventos, não callback síncrono).

**Colisão de gameplay = `ph2d-collision2d` (novo), não rapier:** grid broadphase + tags + ordem por bits de entidade, determinística no fixed-step. **Rapier (ADR-física/M10) fica reservado pra dinâmica/corpos rígidos.**

A API canônica `ph2d.*` é a **existente** (`set`=`EntityWrite` diferida, `get`=ReadSnapshot, `spawn/despawn`=`SpawnQueue`, `state_table`, `input`, messaging, `espere/deslize` via Scheduler+`coroutine.yield` com passo fixo `1/60`).

## 3. Consequências

**Aceitas:**
- Superfície de runtime mínima (só Luau); duas UIs de autoria sem dois runtimes.
- Determinismo de gameplay preservado (escrita diferida + passo fixo + sem HashMap iter).

**Riscos:**
- Manter blocos e node-programming gerando IR consistente → IR única, dois front-ends.

## 4. Alternativas consideradas

- **Compilador de blocos no runtime da engine:** rejeitado — incha o runtime; o portável é o Luau/bytecode emitido.
- **Rapier para colisão de gameplay casual:** rejeitado — rapier é dinâmica; colisão-evento por tag/grade é mais leve e determinística.
- **Só blocos OU só nós:** rejeitado — o Enio quer as duas superfícies; a IR única as concilia.
