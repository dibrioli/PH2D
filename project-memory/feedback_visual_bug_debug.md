---
name: feedback-visual-bug-debug
description: "Bug visual/layout/posicional — fazer aritmética de pixels CEDO, não procurar bug lógico"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e332270d-c182-4fcb-9ca8-8a86e5b96276
---

**Bug visual/layout: faça aritmética de pixels CEDO. Não leia código procurando bug lógico antes.**

**Why:** sessão 2026-05-19 — submenu cascade do Settings caía fora da tela (viewport 1024×768, anchor em 962+200=1162). Gastei tempo lendo `close_context_menu`, timing de hit_index, ordem de dispatch. O bug do CLAMP era 100% aritmético: anchor = `row.x + row.w` num parent já clamped à borda direita ⇒ submenu pós-clamp coincide com posição do parent (overlap invisível). E o bug RAIZ era ainda mais simples: faltava registrar `CTX_MENU_SETTINGS_UNIT` em `populate_global_context_menu`, então Click event NEM era emitido. eprintln experimental revelou em 1 ciclo o que 4 análises estáticas não pegaram.

**How to apply:**

1. **Aritmética primeiro.** Quando bug é visível/posicional (menu fora, widget sobreposto, click em lugar errado), calcule os valores reais (anchor, viewport, width, hit rect) ANTES de procurar bug lógico. Cálculo de 30s mata 50% das hipóteses falsas.

2. **Não declare fix "pronto pra smoke" sem simular o resultado visual.** Clamp resolve "fora da tela" mas pode criar overlap com parent — solução parcial vendida como completa fez Enio voltar 2x. Pergunte-se: "depois do meu fix, em que pixel exato o usuário vê o widget?"

3. **Pergunte VISUALMENTE antes de mergulhar em código.** "Não aparece" é ambíguo: parent sumiu? algo apareceu em outro canto? lampejo? Pergunte coords/sintomas concretos no primeiro turno, não no quarto.

4. **Instrumentação experimental >> leitura estática.** Para bugs visuais, `eprintln!` num handler crítico responde "esse código rodou?" em segundos. Não enrole em leitura de fluxo dispatcher quando 5 linhas de print resolvem.

5. **Verifique pré-condições de evento, não só lógica de handler.** Quando handler "não roda", a causa frequente é input que nunca chega: id não registrado em populate, hit_index sem entrada, set_active não disparado. Não é sempre o handler que tem bug — às vezes o evento nem é emitido.

Vide [[feedback-commit-cadence]] sobre não fragmentar commits, e [[feedback-codificacao-rapida]] sobre cadência de validação.
