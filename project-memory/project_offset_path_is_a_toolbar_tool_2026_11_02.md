---
name: project_offset_path_is_a_toolbar_tool_2026_11_02
description: "Dono 02/11 (smoke F48b OK): o Offset Path vira FERRAMENTA na barra vertical (não secção do Inspector) e o Result ganha «desenhar pela frente» além de por trás e Replace"
metadata:
  node_type: memory
  type: project
  originSessionId: 0fe7a069-5ac9-452e-a528-9a4abb95deb9
  modified: 2026-11-02T19:35:35.621Z
---

Dono, 02/11, depois de aprovar o smoke da cena 51 (Offset Path no padrão do Inspector, linha UIUX F48b):
1. *«melhor transformar numa tools na barra de tools»* — o Offset Path sai da secção do Inspector e vira uma
   ferramenta da coluna ① (barra vertical da esquerda), com as opções na fila ② e a alça na tela.
2. *«acrescentar a opção de desenhar pela frente, não só por trás»* — o Result tem três: por trás, pela frente
   e Replace.

**Why:** mesma lei das regras CAD ([[project_drawing_rules_are_a_tool_with_explicit_gizmos_2026_10_26]]): um
gesto sobre a forma é FERRAMENTA com gizmo, não lista no Inspector.
**How to apply:** ferramenta nova no idioma do Width Tool (modo próprio, opções na ②, arrastar a borda); o
Inspector deixa de ter a secção. Handoff: `docs/UI_New_and_Simple/handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F48b.md`.
