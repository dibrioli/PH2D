---
name: feedback_convention_vs_inertia
description: "Não apresentar estado-atual incidental como regra; isolar-em-crates é o norte, checar se \"convenção\" é gate-enforced ou inércia"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 783df84b-72de-4862-8f31-b1174cbe1825
---

Em 2026-05-22, ao planejar a consolidação do BgRemoval, apresentei "tools stateful vivem em editor-core" como se fosse uma convenção/restrição arquitetural. O Enio questionou: "por que essa regra? não seria melhor isolar tudo em crates?" — e estava certo. NÃO havia regra: o gate `architecture_cycle_prevention` só proíbe editor-core→panel e panel→shim; **tool→editor-core é direção permitida**. Os `Tool` impls stateful (BgRemovalTool/PaddingTool/brush/move) estavam em editor-core por **inércia histórica** (a migração convention-by-discovery puxou manifests+algoritmos puros pros crates mas deixou os Tool impls pra trás porque dependem de `Tool`/`FloatingPanel`/`widget::*`).

**Why:** isolar features em crates satélite (foundation fina e congelada) É o norte documentado do projeto (tese FBP do node-centric, [[project_node_centric_decision_2026_05_21]]). editor-core ser god-crate de 42k é o que essa arquitetura quer matar. Apresentar inércia como princípio leva a recomendar o caminho MENOS alinhado ao norte.

**How to apply:** antes de chamar algo de "convenção"/"regra" e usar como restrição num plano, verificar se há GATE que o enforça (arch-test) ou se é só o estado atual por inércia. Se for inércia, dizer isso — e pesar a decisão contra o norte de isolamento-em-crates, não contra o status quo. Direção default ao refatorar: mais isolamento (crate satélite), não manter peso em foundational. Vide [[feedback_communication_style]] (recomendação primeiro, mas a recomendação tem que estar alinhada ao norte real).
