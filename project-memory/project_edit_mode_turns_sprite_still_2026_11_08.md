---
name: project_edit_mode_turns_sprite_still_2026_11_08
description: Dono 08/11 — entrar no modo Edit volta a sprite a Still; operações que mudam tamanho/margem avisam; DEPOIS da integração da GEF 26
metadata:
  node_type: memory
  type: project
  originSessionId: 4ff5a426-8a7d-4b9c-b2d4-fe26c9ffc4b4
  modified: 2026-11-09T00:53:00.958Z
---

Dono, 08/11, depois do smoke das ferramentas de imagem na folha visível (GEF 26, doc 27): «Não faz sentido utilizar as
operações do modo Edit numa sprite pronta, muito menos de um tipo diferente de Still.»

Decisão:
1. Ao entrar no **modo Edit** de um objeto Image, a sprite **volta ao uso Still** (a imagem aberta, inteira).
2. As operações que mudam **tamanho ou margens** da imagem (Trim, Padding, Make Square, Upscale…) emitem um **alerta**:
   a mudança provavelmente estraga a configuração das animações e de outros aspectos do objeto.

**Why:** cortar ou redimensionar uma folha já cortada em animações desfaz o corte, os quadros e o Pivot.

**How to apply:** só **depois** da integração da linha GameEngineFuncional (GEF 26), numa onda nova. Ela torna discutível
a hipótese registada no doc 27 §3.3 (Trim na folha 0 de um Animated esticaria o boneco). O funil da folha visível
(`ph2d-anim-sheets::shown_edit`) fica como está até lá. Relacionado: [[project_image_tools_act_on_visible_sheet_2026_11_08]].
