---
name: feedback_brush_oracle_must_include_blank_canvas
description: Comparar pincel com o original só sobre blocos de cor escondeu que os pincéis Multiply não pintam na tela vazia (dono 08/11)
metadata:
  node_type: memory
  type: feedback
  originSessionId: 5bf7d55e-1b2b-4e27-994e-b4b560e96da9
  modified: 2026-11-08T10:39:22.634Z
---

Os 16 pincéis CSP de 08/11 foram afinados contra o Clip Studio só sobre uma cena de blocos coloridos opacos. O dono achou no uso real que os de Blend: Multiply (Wet Wash Mix, Watercolor Splash…) **não pintam na tela branca/camada vazia** no PH2D, e no CSP todos pintam.

**Why:** a cena de blocos é o caso fácil; o caso do dono é pintar numa camada transparente sobre o papel. Uma régua que não tem o caso de uso real aprova o defeito.

**How to apply:** toda comparação de pincel com o original corre em pelo menos três fundos — camada VAZIA sobre o papel, branco opaco e blocos de cor — com o mesmo traço, e uma célula por fundo na folha lado a lado. Ver [[feedback_the_owners_reference_product_decides_the_law]].

**Revisão v3 (08/11, `docs/Painter/REVISAO_CSP_V3_2026-11-08.md`):**
- Pincel em modo de cor (Multiply…) que não pinta na camada vazia = a receita sem `blend_sobre_vazio: true`. O motor já tem a lei (W3C geral); NÃO é defeito de motor.
- O CSP de avaliação NÃO exporta nem copia: a saída é a FOTO a 100 % de uma tela de 800×880 por fundo, aberta uma vez, com Ctrl+Z ×4 entre pincéis e a foto conferida contra a tela em branco. Ferramentas: `cena_v3.py`, `folha_v3.py`, `afina_v3.py` e a sonda `sonda_csp_v3_revisao` (`PH2D_V3_VARIANTES` afina receitas sem regravar presets).
- Pincéis do CSP que misturam pintam cor CHEIA na camada vazia (nada para apanhar) e mais pálida no branco: a palidez sobre o branco é real, não é erro do oráculo.
