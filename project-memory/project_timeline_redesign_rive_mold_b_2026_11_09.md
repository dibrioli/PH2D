---
name: project_timeline_redesign_rive_mold_b_2026_11_09
description: "Dono 09/11 — Timeline redesenhada SEM mudar funções, versão B da crítica (chips de clipes à Rive); «Keys | Curves» fica de fora"
metadata:
  node_type: memory
  type: project
  originSessionId: f397c650-7a6c-473a-9a9e-87e322ea1049
  modified: 2026-11-09T18:41:11.841Z
---

Dono 09/11 (linha `line/TimelinePrincipal`): redesenhar o painel Timeline buscando aparência, organização e
intuitividade, **sem mudar funções**. Fonte: `~/Documentos/GameEngineSamples/critica_ph2d.html#timeline`.
Escolheu a **versão B** (molde do Rive: clipes/containers em chips, ▶ = Autoplay, menu do clipe, popover
«Timeline options» na engrenagem). **«Keys | Curves» fica de fora por ora.** Função nova da crítica (tocar trecho
entre marcadores, «Animating · Done», «quem ouve», losango de 3 estados, chave no canvas, container aberto no
lugar) NÃO entra. Plano: `docs/Timeline/15_plano_redesenho_molde_rive.md`.

**Why:** a barra tinha 32 comandos; o motor já é forte, o problema é a barra dizer o que faz.
**How to apply:** só a barra em cartões muda; `PH2D_CARDS=0` fica como referência. Mesmos ids e eventos.
