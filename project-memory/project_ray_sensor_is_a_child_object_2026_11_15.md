---
name: project_ray_sensor_is_a_child_object_2026_11_15
description: "Dono 15/11 — Ray Sensor = versão B da crítica E deixa de ser componente: vira OBJETO filho na Hierarquia, como o Collision Shape (ADR-0199)"
metadata:
  node_type: memory
  type: project
  originSessionId: 5ee6d4f3-972f-4da9-816a-e5361061957b
  modified: 2026-11-15T10:45:35.149Z
---

Dono 15/11 (GEF 07, ficha `docs/GameEngineFuncional/componentes/07_ray_sensor.md`): escolheu a **versão B** (Angle no lugar de
Direction X·Y, Layer por nome + aviso de camada vazia, cartão Signals, leitura «Sees X at Y» à vista) — **sem** a alça da C
(recomendada, não escolhida). E: *«assim como o collision shape deixou de ser componente e passou a ser objeto que aparece na
hierarquia, também deve ser o Ray»*.

**Why:** coerência com [[ADR-0199]] — o que é «uma coisa no mundo» com posição própria é objeto, não componente.
**How to apply:** Ray sai do Add Component e entra no Add ▸ objeto (filho do selecionado); a origem é a posição do objeto;
dois raios = dois filhos. Relacionado: [[project_physics_critique_answers_2026_11_09]].

**Depois (15/11):** pediu «Turns with facing» como OPÇÃO (feito: caixa, lado = `PlayerState::facing` do dono). Sobre
exclusões (pesquisa Godot/Unity/Unreal/GameMaker/Construct/GDevelop/Roblox): escolheu a **C** e disse *«quero algo que
abranja todas as possibilidades»* ⇒ o filtro do raio cobre: Ignore my body · Ignore my other parts (família do dono) ·
Only see (Everything / only tag / all except tag, paredes bloqueiam ou não) · Hit triggers · Starts inside · See all hits.
