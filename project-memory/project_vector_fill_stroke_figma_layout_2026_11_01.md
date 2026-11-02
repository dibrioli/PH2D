---
name: project_vector_fill_stroke_figma_layout_2026_11_01
description: "Dono 01/11 (F44) — Fill e Stroke do Vector no layout Figma, igual a Effects/Filters; Cap/Join/Align com ícones"
metadata:
  node_type: memory
  type: project
  originSessionId: a98d30f9-ee9a-4340-afaa-6d0093e49fe5
  modified: 2026-11-01T21:00:35.709Z
---

Dono 01/11, aprovando a proposta F44 ("Vamos fazer como sugere"): Fill e Stroke do Inspector do Vector
seguem o layout do Figma, o MESMO padrão de Effects/Filters (F43, [[project_vector_effects_filters_figma_layout_2026_11_01]]):
- **Fill**: «+» no cabeçalho; cada preenchimento é uma linha (swatch, nome/cor, opacidade %, olho, «−»); clicar abre o
  painel à ESQUERDA com tipo (Solid/Linear/Radial/Multi/Pattern), cor e degradê — a secção «Fill type» some para dentro dele.
- **Stroke**: linhas de cor + UMA linha fixa Position (Inside/Center/Outside, dropdown) e Weight; botão de ajustes abre
  «Stroke settings» à esquerda (Cap, Join, Miter, tracejado, pontas, perfil de largura).
- A pilha `paints` (fills/strokes extras) vira linhas dentro de Fill/Stroke, nunca secção à parte.
- **Cap / Join / Align modernizados**: segmentado com ÍCONES inteligíveis (como o Figma: cantos desenhados), não botões de texto.
- Auto Key em todo campo; o popover de traço da barra continua a abrir o mesmo conteúdo.

**Smoke do dono 01/11 (F44 reprovada, 5 pontos — foto `img_F44/smoke_dono_*.png`):**
1. o TIPO (Linear…) fica FORA da caixa de cor — a caixa é só amostra + hex, bem definida;
2. TODAS as linhas iguais: a base e as extras têm TODAS as opções da espécie (fill: tipo/degradê/padrão;
   stroke: cap/join/tracejado/pontas…) e TODOS os botões (olho, «−») — nada de base «especial»;
3. ícones de Position em padrão CIRCULAR (círculo com o traço dentro/centro/fora, como o Figma);
4. botão de ajustes também no Fill (consistente com o do Stroke);
5. ALINHAMENTO: colunas iguais em todas as linhas (caixa, %, olho, «−» no mesmo x), como o Figma.

**Why:** Inspector curto e um só idioma nas quatro secções; os botões de texto Butt/Round/Square pareciam antigos.
**How to apply:** referências em `docs/UI_New_and_Simple/handoffs/img_F44/` (hoje × Figma). Conferir no código como a
pilha `paints` aparece hoje antes de mover (é a parte de maior risco).
**Dono 02/11 («parte só quando precisa»):** a linha (cor · Weight · %) fica numa fila como na F44c; só quando o VALOR ATUAL do Weight não cabe (ex. 100) passa a duas filas — o salto ao arrastar de 99 para 100 é aceite; o número nunca é cortado e a unidade fica no rótulo ([[feedback_units_live_in_the_label_not_in_the_box]]).
