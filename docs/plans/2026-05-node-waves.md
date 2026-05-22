# Plano de waves — sistema de nós node-centric (neck → freeze → fan-out)

**Data:** 2026-05-22
**Status:** em execução (neck). Versão versionada do plano antes mantido fora do repo.
**Arquitetura:** ADR-0030..0038 + `docs/Migracao/2026-05-node-centric-architecture.md`.
**Substrato multi-agente:** `docs/Migracao/2026-05-foundational-parallelism-three-bottlenecks.md`.

Os tags `W1.Tx` / `W2.Tx` que aparecem em comentários de código referenciam este doc.

## Forma: funil

Um **neck serial** (o contrato compartilhado, Coordenador-only — nenhum nº de agentes acelera) → **FREEZE** → um **fan-out paralelo** (N agentes, um node-crate cada, sem colisão). A velocidade total depende de congelar o contrato cedo e não abrir o fan-out antes de uma vertical estar provada end-to-end.

## WAVE 0 — destravar o motor de execução
- **W0.T1** — `CARGO_TARGET_DIR` por slot de agente (build paralelo sem lock do `target/`). ✅ medido (MAC_EXTERNO 1.7Ti livre, ~20 slots) + validado na prática (build em `target/slot-coord` isolado). Falta: documentar a convenção em `DIRETRIZ.md`.
- **W0.T2** — decompor os 15min (check vs build-downstream vs gates). ⏳ pendente (não bloqueia o trabalho de nós).

## WAVE 1 — o NECK (contrato compartilhado) · SERIAL
- **W1.T2** — `ph2d-nodegraph`: contrato (port algébrico domínio+dim+clock, effect/membrana, graph acíclico+`pre`), attribute stream colunar, cook demand-driven incremental, formato textual diffável, arch-gate de surface. ✅ **auditado + remediado 2× (2026-05-21/22)**. Commits `9d6a7ec`, `6489f70`, + remediação da 3ª auditoria.
- **W1.T3** — `ph2d-node-registry` (OpResolver + colisão) + `ph2d-node-registry-init` (gerado) + `tools/ph2d-node-sync` (codegen) + staleness gate + 1º node-crate (`ph2d-node-debug-const`). ✅ cadeia end-to-end provada. Commits `35fd1f3`, `751b974`. **`Cargo.toml` members agora é glob (`crates/*`,`tools/*`) — zero edit central ao adicionar crate (audit codegen-A1).**
- **W1.T4** — `ph2d-expr`: compute compartilhado (Fields inline + escape textual → Luau). ⏳ **PRÓXIMO.** A auditoria (S1) determinou: fazer ANTES de qualquer freeze, e usar para **fixar a forma de `lowerings[]` + um campo `params` no `NodeManifest` + slot de política de cache** — os itens do contrato que ainda faltam. Congelar antes disso seria prematuro.
- **W1.T1** — `SceneDoc` id de entidade estável (ADR-0037). ✅ **verificado: a engine JÁ cumpre** — `SceneDoc`/`WorldSnapshot` são index-based + postcard + versionado + `ComponentTypeId` blake3; os `to_bits` são handles opacos transientes (HR-8), não save. O anti-padrão era do protótipo MiniCavalry. Nenhum código necessário; nota em ADR-0037. `ph2d-save` segue stub (snapshot canônico vive em `ph2d-ecs::scene::save`).
- **W1.T5** — template canônico `ph2d-node-debug-wave` (input + param + `Temporal` + `ph2d-expr`/`eval_column` + golden test) + `docs/IntegracaoMultiAgente/briefing-node-crate.md`. ✅ commit `e87edb0`.

## 🔒 FREEZE (gate do fan-out)
Depois de W1.T4 fixar o contrato e a vertical Motion (W2) provar end-to-end: congelar a superfície de `ph2d-nodegraph` + `ph2d-expr` (arch-gate de cap ativo), declarar estável, mudanças viram evento raro Coordenador-only. **Decisão do Enio + smoke visual necessários aqui** — ponto de parada da operação autônoma.

## WAVE 2 — provar UMA vertical: MOTION · SERIAL (precisa do Enio no fim)
- **W2.T1** — avaliador de motion (pull-no-playhead) → lowering p/ `ph2d-render` instancing.
- **W2.T2** — 3 nós (generator/cloner/modifier).
- **W2.T3** — arch-gate da membrana (rodar `Graph::validate` no load; recusar `Stateful` no lado pull) + view de editor mínima + live-preview.
- **W2.T4** — FREEZE.

## WAVE 3+ — FAN-OUT · PARALELO (pós-freeze)
Tracks independentes, um node-crate por sessão, governados pelo briefing de W1.T5: mais nós de Motion · Shader (→WGSL) · Sound (sync-dataflow) · Gameplay (blocos + node-programming → Luau, `ph2d-collision2d`) · ops de `ph2d-expr` · cook path · ferramentas imperativas (ADR-0027).

## Achados de auditoria ainda abertos (não-bloqueantes, follow-up)
- `would_cycle` é O(V²) por `connect` — irrelevante p/ grafos de autoria; otimizar se surgirem grafos grandes (audit M2).
- A2/A3 do codegen: o template W1.T5 deve carregar o teste `register`+golden e o briefing deve mandar `cargo test -p ph2d-node-registry-init` pós-sync (o gate real é a compilação de registry-init).
- ADR-0032 §2 item 7 ("registry de tipos de porta"): na prática venceu o enum fechado `Domain/Dim/Clock` (mais seguro/determinístico) — ADR a ser anotado.
