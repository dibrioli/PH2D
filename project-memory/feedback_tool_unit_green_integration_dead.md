---
name: feedback_tool_unit_green_integration_dead
description: tool pode estar unit+CI verde e 100% morta no produto — só audit e2e pega o gap de wiring
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3f646c1a-4045-4e59-ad75-6e8198db6003
---

Uma tool/feature pode ter todos os testes unitários passando + CI verde e estar **completamente morta no produto** porque o WIRING de integração nunca fechou. Auditoria e2e (2026-06-02, Vector W2) achou: 4 das 5 pills vetoriais (Pencil/Shape/Select/Direct) **nunca funcionaram** — pintadas + no hit_index mas **não registradas no `WidgetStore::populate()`** → `is_focusable=false` → Down não seta active → Up não emite Click → toggle não dispara ActivateTool → tool morta. Só a Pen (registrada) funcionava. Nenhum gate testa paridade de registro → CI verde.

**Why:** o loop multi-agente (Impl entrega lógica → Coord wira → "smoke depois") acumula código unit-verde sem nunca fechar o smoke e2e. Testes unitários por construção NÃO pegam falta de registro/wiring no shell+chrome. Foi a causa do Enio dizer "nada funciona" após sessões de trabalho "verde".

**How to apply:** ao entregar tool/widget novo, trace o caminho COMPLETO clique→ativar→rotear→mutar→renderizar com evidência file:line, não confie em unit+CI. Para pills: confira register em [[feedback_panel_populate_register]] (populate.rs) E o toggle E o drain de ActivateTool. Smoke e2e real é insubstituível ([[feedback_smoke_at_end]]); "claimed green" ≠ funciona ([[project_painter_t19_latent_red_macos_2026_05_28]]). Outro gap recorrente do mesmo round: tolerâncias em screen-px comparadas contra distâncias world sem dividir pelo camera-scale (Pen/Pencil saíram sem isso; Direct/Select tinham).
