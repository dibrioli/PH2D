---
name: project-node-centric-decision-2026-05-21
description: Decisão arquitetural — PH2D passa a ser construída em torno de um sistema de nós multi-domínio; norte para todas as implementações futuras
metadata: 
  node_type: memory
  type: project
  originSessionId: f278e60f-d10d-4a34-b6e9-83d47d4db46e
---

Em 2026-05-21 o Enio decidiu que a implementação futura da PH2D gira em torno de um **sistema de nós multi-domínio** (modelo Houdini/Unreal/Blender), referenciando o protótipo MiniCavalryV2 (não é port). Arquitetura definida em 4 rodadas de opinião + investigação, documentada em [`docs/Migracao/2026-05-node-centric-architecture.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/Migracao/2026-05-node-centric-architecture.md). Doc irmão (substrato multi-agente, pré-requisito): [`docs/Migracao/2026-05-foundational-parallelism-three-bottlenecks.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/Migracao/2026-05-foundational-parallelism-three-bottlenecks.md).

**Decisões-chave (a virar ADR-0030..0038):**
- **Houdini NÃO é padrão-ouro** — é referência de poder. A arquitetura é síntese: modelo de atributos (Houdini) + Fields/atributos anônimos (Blender) + compile-to-shader/MetaSounds (Unreal) + UX result-named/viewport-first (Substance/TouchDesigner).
- **Substrato unificado + avaliadores plurais.** UNIFICADO: `ph2d-nodegraph` (modelo de atributos, portas algébricas que carregam domínio+dimensionalidade+RELÓGIO, sistema de efeitos Pure/Temporal/Stateful, delay `pre`/aciclicidade-por-construção, formato textual diffável, registry de portas) + `ph2d-expr` (compute compartilhado, Fields + escape textual → WGSL|Luau). PLURAL: avaliadores por modelo-de-avaliação (shader pull→WGSL, audio sync-dataflow relógio-fixo, motion pull-no-playhead, gameplay push→Luau) + views/editores por domínio.
- **Regra de decisão porta-vs-domínio:** "dois nós ligados cozinham no mesmo agendador sem mudar relógio nem alvo de compilação?" Sim→porta tipada mesma região; Não→domínios separados + travessia de membrana tipada.
- **Membrana mão-única CHECADA POR TIPO** (arch-gate): só o domínio gameplay escreve SimWorld → só ele exige HR-5 determinismo; motion/shader/sound isentos (visuais, como Radiance Cascades).
- **Isolamento FBP = unidade multi-agente = nó teoricamente ótimo.** Nó = caixa-preta (portas tipadas + efeito + lowerings, zero estado compartilhado). É a forma final do tool-as-crate (ADR-0027); a engine cresce por adição de crate isolado, wire por codegen.
- **Duas famílias:** nós (declarativos, por domínio) + ferramentas imperativas (`ph2d-tool-*`, painter/Image Tools — TERMINAIS, não rampa pro grafo).

**Estado já pronto (confirmado em código):** mundo imperativo/gameplay ~80% — `ph2d-script::host` já tem `EntityWrite`/`drain_writes` (escrita diferida), `SpawnQueue`, `StateTable`, `InputSnapshot`, messaging Defold; ADR-0021 (Sim/Present + extract!) e ADR-0025 (GameObject=ECS-composition) cobrem o substrato. **Greenfield:** todo o mundo declarativo (nodegraph).

**A inventar (nenhuma ferramenta faz):** membrana de determinismo como tipo · replay determinístico do grafo de gameplay · save/migração versionada de grafos · budget-aware graph · formato de grafo textual p/ LLMs.

**Próximos passos pendentes (não iniciados):** ratificar ADRs 0030-0038; ordem de implementação = três-gargalos (pré-req) → `ph2d-save` → `ph2d-nodegraph` + codegen registry → vertical Motion (avaliador + 3 nós) → `ph2d-expr` → demais domínios. Há também a opção de mandar a versão final do doc de volta ao agente MiniCavalry (formato §6 + 5 perguntas de "specifique melhor").

Relaciona-se com [[project-multi-agent-v6-2026-05-19]] (modelo operacional) e [[feedback-codificacao-rapida]].
