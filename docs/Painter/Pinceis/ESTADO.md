# ESTADO da missão Pincéis — MISSÃO 2 EM CURSO (J6, 2026-10-28) (≤ 8 KB)

> Missão 1 encerrada na J5 (18 aprovados; [integração](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md)). Contrato: [`00_MISSAO.md`](00_MISSAO.md) · [custo](REGRAS_DE_CUSTO.md) · `HANDOFF_JANELA_<n>.md` · `exames/<família>.md`. Handoff da J6: [`HANDOFF_JANELA_6.md`](HANDOFF_JANELA_6.md).

**Total: 26 aprovados (+ Soft Pastel na J6), fábrica 64 PNG medidos em `assets/brushes/{WetPainter,Digital}` (52 declarados; conferir por `the_factory_presets_are_counted`).**

## Feito

- J1 [h](HANDOFF_JANELA_1.md) bateria, oráculos, catálogo (112) · J2 [h](HANDOFF_JANELA_2.md) lacunas L1–L19 · J3 [h](HANDOFF_JANELA_3.md) 6 aprov./fábrica 34 · J4 [h](HANDOFF_JANELA_4.md) 11/44 · J5 [h](HANDOFF_JANELA_5.md) 18/45 (`b74b608d0e`).
- Óleo Real 27/10: [W2 E4](W2_Knife_E4.md) 22/49 · W3 P ([Knife](W3_P_Knife.md) · [Dabber](W3_P_Dabber.md) · [Metallic](W3_P_Metallic.md)) 27/54 · smoke do dono ([§3g](01_CATALOGO.md)) saem Metallic Splats e [Oil Bristle](W3_oil_bristle_W2_x_W3.md), Knife Textured Contrast 0,25 ⇒ **26/52**. Candidatos `EM_ESTUDO_W2`: Knife Smooth/Smooth 2/Soft/Dry 2, Dabber 2/3/Fan, Metallic Round. [Prancha](pranchas/W3_presets.jpg).

## Decisões de produto

1–18 em [`HANDOFF_JANELA_5.md`](HANDOFF_JANELA_5.md#decisões-de-produto-movidas-do-estado-em-2810-j6). Valem: (1) lâmina FIXA no eixo — confirmado na J6; (2) matriz é triagem, veredito = prancha + cega; (12) «diferença visível» = confiança média/alta. Pendentes do dono: 5 (cauda do taper), 8 (tecto de tamanho), no PDF.

## J6 ([`JANELA_6.md`](JANELA_6.md)): a fila

- **0 Óleo** ⭐ dono 28/10: Oil/* com Real Oil (`e314e58cec` `d94e19a57e` `105f28791d` `78fb8c7c77` `5470ce0969`): 2× mais rápido ao bit; bancada fotografa o óleo do traço. Cega (9 óleos) Rebelle achado 11/27 (acaso 9): seguem aprovados Filbert 396 · K. Textured 374 · K. Dry 362 · K. Smooth 3 352 · K. Oily 342. EM CURSO (motor: o óleo derrete o grão fino): K. Stain 162 · K. Grainy 226 · Dabber 4 322 · Metallic Flat 326. OOM = paralelismo + 19 GB de /tmp órfão.
- **1–2 Borda/pontas** (`8394f62eed` `5f6ae47051` `59f64037c1` `5cd611289a` `987763fc42`; `10eee50b0d` `fd73b349ca`, [pontas](J6_pontas_rectas.md)): **Soft Pastel APROVADO** (cega r2, 6 linhas). Square Hard curado não aprovado (faixa sólida «borda de régua», média). Mop REPROVADO. Wet Liner, Wet Wash curados não aprovados (cega r2, média: chapado, tampa dura; Wash contorno escuro, edge-darkening 100). Charcoal/Wax Scrape: o Rebelle os acaba REDONDOS (0,50/0,28) ⇒ outra lacuna.
- **11 Tampas esbatem** (`5b8c03e2ca` `93fd2137d6`, `pontas_esbate`; erro Square Hard 0,224→0,123, Wet Wash 0,182→0,096; Liner igual): cega — Square Hard (pontas arredondadas) e Wet Wash (cápsula, quadrados nos cruzamentos) seguem não aprovados.
- **3 Secador** (`48da05baa5` `0ea881a460`, `dryerLevel`; cruzamento 0,35–1,63 → 0,98–1,10, Rebelle 0,9–1,14): Dry/Lift curado não aprovado (falta linha escura 1,3; anel rouba tinta de fora).
- **4 Misturador** (`eac8182beb` `db69ce13f2`, `blendReach`; 52→36 px, Rebelle 30): Blender Grainy curado não aprovado (grumos grossos rasgados: grão da máscara).
- **5 Fio do arrasto** (`3f6377fcec` `49211e2779`, `smear_fio`; linhas vazias 35/118 → 0): Smudge Finger curado não aprovado (fio pálido 1,25×2,14; língua oca). Portão sem_glaze verde (`e42004f3e8`; o vermelho vinha de `5291edbc75`, 25/10, lei do tubo).
- **6 Guache cobre** (`600152c535` `20a320bbc9`, `wetCover`; resto Flat 0,36→0,10, Rebelle 0,11): Wash e Dry curados não aprovados. Gouache Flat com `wetCover` 1: régua 404→424, NÃO entrou (`cc49c5616a`): regerar faz a tabela de largura ver o traço r 1,5–1,8 (linha pálida 3 px, P44 diz 0) ⇒ PRÓXIMO: medir a largura do fino pela cobertura útil, regerar Gouache/Wet Flat.
- **9 Texturas** (`f9fbbe7c26`, `DabJitter::subpixel_area`; 5 px 0,258→0,131, Krita 0,122): Spray Can curado não aprovado (faltam salpicos escuros, discos grandes translúcidos). Foliage, Grass, Stipple, Noise Grit, Splatter sem lacuna comum ([texturas](exames/texturas.md)).
- **10 Cerdas** (`a7cd157000`, `bristleLaneSizeExp`; fio a 160 px 7→11–12, Rebelle 10): só Gouache Dry (régua 382→364, receita fica); [guache](exames/guache.md).
- **7 Lâmina** (`a6dcb2afa1` `883ea9d7c2`, [lamina](J6_lamina.md)): o Rebelle NÃO gira; gate `a_lamina_de_cada_preset_e_a_do_oraculo`. Em estudo: Craft Flat, Broad Nib, Compressed Charcoal, Bristle Watercolor; [receitas Oil](J6_lamina_oil_receitas.md).
- **8** Particles/Glitter: sem oráculo, fora. **9b** Óleo Real: `ESTADO_MISSAO_W3.md` «Fila» + HANDOFF_W3b §Aberto 2–5. **10b** Só valores: Wax Crayon (F-10), Wood Stick (C-15), Blur (L-5).
- Motor restante: azul sobre vermelho molhado (Gouache Wash/Dry/Round) · lâmina que gira (C38/L3) · partículas · ondulação (Oil Bristle) · mip das pontas (Foliage, Noise Grit). Mistura em luz linear: FORA (dono). ⚠️ Brecha `arch_color_space_typed`; Ink Brush a vigiar (p95 12,54 ms).

## J6 passo 3 — pincel a pincel (16 só por valores; 16 travados por motor; 7 sem oráculo)

- Digital ✅ (`8a9703db68` `07c644af37` `c8d4ad953d`, [`exames/digital_j6.md`](exames/digital_j6.md)): C-4 Studio Pen (Hard Round 0,283) e C-6 Technical Pen (Pixel 1px 0,330) FUNDEM; K-1 Clouds curado não aprovado; motor B-6 (rampa sob o tecto), L-5 (blur puxa o papel); M-7 sem oráculo. **Cega Digital: B-7 Rough Sketch e I-12 Optical Wash curados NÃO aprovados** (Rough Sketch: o revisor tomou o NOSSO pelo pincel de fora em todas as linhas — discos nítidos onde o MyPaint tem grão de pontos e pontas rectas; Optical Wash: achado em todas as linhas, média em 2 — contorno forte e contínuo, o Krita tem franja fraca irregular). ⚠️ Biblioteca `j6_wet` incompleta (28/66) ⇒ d antigos suspeitos; usar `/tmp/claude-1000/j6_digital/biblioteca`.
- Canetas/marcadores Wet ✅ (`221fb75e43`, `EM_ESTUDO_J6_WET`, [`exames/canetas_j6.md`](exames/canetas_j6.md)): Reed Pen,
  Wood Stick, Marker Chisel, Dry Marker curados não aprovados (diferenças medidas e conhecidas — cega não gasta). Fila:
  dual deita fora a ponta achatada (EM CURSO) · tamanho em pé · velocidade→tamanho no Wet · riscas do carimbo.
- Aguarela/texturas ✅ (`6999f3ec3d`, [`exames/aguarela_texturas_j6.md`](exames/aguarela_texturas_j6.md)): **F-10 Wax Crayon
  APROVADO** (cega: o Rebelle nunca achado — chamado «imitação» nas 6 linhas; o nosso tomado pela referência em 3,
  média/alta; régua 436, Graphite 8B d 0,783) ⇒ sobe no lote. I-10 Fringe Bloom curado não aprovado (achado em todas).
  K-9 Grunge curado (riscas contínuas × traços partidos). Motor: I-9 Granulation (poros de 1–2 px), I-11 Spatter
  (o spray do Wet pousa discos longe do traço + OOM a jitter 0,8 — EM CURSO).
- **LOTE DE PROMOÇÃO** (no fim, uma regeração): Wax Crayon.
- Aprovados pela cega sobem à fábrica num lote único no fim. Sem oráculo: C-13, C-14, D-2, D-4, G-13, J-4, M-5. Motor: CAP-19 cerdas que esgotam (C-11, G-9, G-11, I-13), L17, L15, L20, L10.

## Bancada

[`04_BANCADA.md`](04_BANCADA.md) §«Notas da bancada».
