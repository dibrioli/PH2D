---
name: feedback_foundational_editable_design_for_isolation
description: Modo L — foundational É editável pela sua linha (agentes duvidam disso); mas ao CRIAR foundational novo projete p/ isolamento + anote símbolos novos no handoff de integração
metadata:
  type: feedback
---

Dúvida recorrente dos agentes-de-linha: "posso tocar os arquivos foundational
(centrais — `ph2d-core`/`ph2d-editor-core`/`ph2d-tokens`/`ph2d-host`/`shells/*`)?"
**Sim — no Modo L você PODE e DEVE**, com cuidado (ADR-0107). O que os trava é a
memória velha do Modo C ("foundational = Coord-only"); no `workstation` isso caiu.

**Duas regras que acompanham essa liberdade:**

1. **Ao CRIAR arquivo foundational novo, projete-o para ISOLAMENTO.** A foundation
   tem arquitetura de isolamento *de propósito* — é o que deixa N linhas a
   estenderem em paralelo sem colidir. Prefira **módulo/arquivo irmão novo** a
   engordar um arquivo compartilhado; exponha **ponto de extensão append-only**
   (lista ordenada, marcador de codegen tipo `ph2d-chrome-sync`, `mod` por
   responsabilidade) onde a próxima linha pluga, não um site central que todas
   editam. Menos superfície compartilhada = menos conflito de merge na integração.

2. **Todo símbolo novo compartilhado vai no handoff de integração** (DIRETRIZ
   §1.5.9): id/const/variant/token com o **valor literal** (ex.: `NodeId(832)`).
   Foi exatamente uma colisão de `NodeId(832)` (audio e Vector pegaram o mesmo
   "próximo livre" independentemente) que o integrador teve que renumerar p/ 833
   na jornada de 2026-07-07. O handoff é o que deixa o integrador grepar e achar
   a colisão antes que ela vire bug silencioso.

**Why:** o gate da árvore combinada (`cargo check --workspace`) pega quebra de
build, mas NÃO pega dois símbolos numericamente iguais com nomes diferentes
(Mergiraf une textualmente, ambos compilam) — só um teste de unicidade ou o
grep do integrador pega. Isolamento na criação + declaração no handoff é a
prevenção barata.

**How to apply:** editar foundational existente = livre (com cuidado); criar
foundational = pense "como a próxima linha estende isto sem me tocar?". Sempre
liste no handoff o que criou de compartilhado. Vide
[[feedback_integration_only_enio_command_end_of_all_lines]] (handoff + integrador
dedicado), [[project_multiagent_modo_l_2026_07_05]] (Modo L),
[[feedback_panel_arch_gates_scope_and_clamp_const]] (gates que escaneiam TODO o crate).
