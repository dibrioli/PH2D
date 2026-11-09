---
name: project_script_editor_bancada_owner_decisions_2026_11_08
description: "Dono 08/11 — editor de scripts vira a «Bancada» (molde Godot + grelha Who has what + frase do pé, idioma da aba Collision Layers); cor por objecto; menu da aba; depurador sob medição"
metadata:
  node_type: memory
  type: project
  originSessionId: f47a77ca-ea40-46ce-a671-5d804e1ad113
  modified: 2026-11-08T19:03:10.819Z
---

O dono aprovou em 08/11, todas na recomendação, o plano `docs/CodeEditor/13_plano_bancada.md` (linha `line/ScriptEditor`, que continua o CodeEditor):
- **Rumo: Bancada** — maquete B da crítica (`GameEngineSamples/critica_ph2d.html#editor_scripts`) + grelha «Who has what» (objectos × propriedades, editável) + frase fixa no pé + marcas na margem «em quem».
- **Cor por OBJECTO** que usa o script (não por script) — a mesma na grelha, margem, Console e contorno no canvas.
- **Menu da própria aba** (File · Edit · Find · Go To · Run, ~24), saindo do File/Edit geral do app.
- **Ponto de parada / valores ao vivo: sim, se não pesar** — kill-criterion 2 % do quadro medido antes.
- **08/11 (depois da medição: tudo ≤ 2 %, pior +1,13 %)**: valor no fim da linha **só para o script da aba aberta**, só no Play; F9 + Stopped at line + Continue/Next line + Pause on Error como propostos (decisão 11 de `00_decisoes_do_dono.md`).

**Why:** pediu «funcional como o Godot, mas belo e original como o editor de Collision Layers da GEF02».
**How to apply:** Onda A (problema único texto+Play com objectos e ×N, print com objecto e hora, Ctrl+/, contagem na lista, luau-lsp no pacote) antes; a Onda B só depois da GEF02 integrar (os tokens de cor das camadas vivem lá). Registo também em `docs/CodeEditor/00_decisoes_do_dono.md` linha 10. Relacionado: [[project_layer_chip_no_double_click_buttons_above_collides_2026_11_08]].
