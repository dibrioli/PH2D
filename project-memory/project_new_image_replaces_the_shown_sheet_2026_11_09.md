---
name: project_new_image_replaces_the_shown_sheet_2026_11_09
description: "Dono 09/11 — soltar/escolher imagem nova no cartão Image com folha extra à vista troca a folha À VISTA, não a 1.ª"
metadata:
  node_type: memory
  type: project
  originSessionId: 4ff5a426-8a7d-4b9c-b2d4-fe26c9ffc4b4
  modified: 2026-11-09T11:10:55.037Z
---

Dono, 09/11, respondendo ao aberto (2)(4) do doc 27 §3 / handoff GEF «várias folhas»: soltar uma imagem no objeto ou
escolhê-la no seletor do cartão **Image** (`image_choice.rs`), com uma folha extra à vista no ▾, **troca a folha que está
à vista** — hoje troca sempre a folha 0. Resposta: «SIM».

**Why:** é a mesma lei de 08/11 das ferramentas de imagem e do Painter: tudo age na folha visível.

**How to apply:** linha GameEngineFuncional, na onda depois da integração de 09/11, junto com
[[project_edit_mode_turns_sprite_still_2026_11_08]]. Passe pelo funil `ph2d-anim-sheets::shown_edit` (`shown_sheet`), com
upload antes de trocar a folha e undo pelo diff do projeto. Relacionado: [[project_image_tools_act_on_visible_sheet_2026_11_08]].
