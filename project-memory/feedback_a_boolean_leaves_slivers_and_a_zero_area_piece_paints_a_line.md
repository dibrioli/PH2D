---
name: feedback-a-boolean-leaves-slivers-and-a-zero-area-piece-paints-a-line
description: Subtrair uma curva dela mesma depois de um round-trip deixa lasca de área ~0 — e uma peça sem área pinta uma LINHA, não uma forma pequena
metadata:
  type: feedback
---

O Enio: *"a ferramenta Build funciona bem, mas está deixando pedaços de linha sobrando."*

O Shape Builder devolve a sobra como `fonte − união(faces levadas)`. As faces vêm do **arranjo**:
a borda que elas devolvem é uma **re-derivação** da borda da fonte, não os mesmos bytes. Subtrair
uma curva **dela mesma** depois dessa volta deixa resíduo — medido, uma peça de **144 vértices**,
bbox do tamanho das formas e **área ~1e-13**. (A booleana de 2 operandos é limpa; o resíduo é da
composição arranjo→união→subtração.)

**Why:** uma peça sem área **não é uma forma pequena — é uma LINHA**. Sem área não há
preenchimento para pintar, então o que aparece na tela é o *traço* da fonte percorrendo a borda e
voltando. Medir bbox, comprimento ou nº de vértices não a distingue de arte nenhuma; e "é
geometria de verdade, o Illustrator faz igual" é a desculpa que a deixa passar.

**How to apply:**
1. **Toda composição de booleanas precisa de um filtro de lasca**, com piso **relativo** (fração
   da área do operando — escala-livre) e `>` estrito (uma referência degenerada não deixa passar
   o degenerado). Ache o piso **medindo as duas populações**: aqui, arte ≥ 6,5% da fonte e resíduo
   ≤ 0,30% — piso em 0,5%, com 13× de margem. O número sai da tabela, não de um chute.
2. **Descartar geometria em silêncio é pior que a lasca** — logue o que caiu e por quê
   (`PH2D_BUILD_LOG=1`).
3. **O gate quase nasceu MORTO:** `assert!(area > 0.0)` fica **verde** com o bug vivo — a hairline
   tem 1e-13, não 0. O oráculo tem de modelar a **aparência** (uma peça sem *espessura* pinta uma
   linha ⇒ **densidade** = área/bbox: 0,00 na lasca, ≥ 0,41 na arte), nunca a regra do filtro
   (isso é circular). Só a **mutação do código** revelou o gate morto.
   ([[feedback_oracle_must_model_appearance_not_implementation]] ·
   [[feedback_mutate_the_code_not_just_the_test]])
4. **O fixture escondeu o bug:** 11 gates verdes sobre **polígonos**, e a hairline **só nasce em
   borda CURVA**. Um universo de quadrados só prova coisas sobre quadrados — o fixture tem de ter
   a geometria que o produto entrega. ([[feedback_test_with_product_numbers_not_convenient_ones]])
5. E o **irmão de presença**: um filtro que apaga tudo deixa o gate de ausência verde. Pine o piso
   dos dois lados — uma lua crescente **fina, mas real**, tem de sobreviver.
   ([[feedback_absence_gate_needs_a_presence_sibling]])
