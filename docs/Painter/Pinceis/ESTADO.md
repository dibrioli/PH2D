# ESTADO da missão Pincéis — MISSÃO 2 — linha REABERTA 30/10 (J7: bugs do smoke) (≤ 8 KB)

> **Handoff de integração:** [`HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-29_PINCEIS2.md`](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-29_PINCEIS2.md) (mudanças, abertos, smoke). Abaixo: J6 (28/10).

> Missão 1 encerrada na J5 (18 aprovados; [integração](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md)). Contrato: [`00_MISSAO.md`](00_MISSAO.md) · [custo](REGRAS_DE_CUSTO.md) · `HANDOFF_JANELA_<n>.md` · `exames/<família>.md`. Handoff da J6: [`HANDOFF_JANELA_6.md`](HANDOFF_JANELA_6.md).

**Total: 28 aprovados (26 + Soft Pastel + Wax Crayon), fábrica 66 PNG em `assets/brushes` (WetPainter 45 · Digital 20 · Watercolor 1; contagem oficial: `the_factory_presets_are_counted`).**

## Feito

- J1 [h](HANDOFF_JANELA_1.md) bateria, oráculos, catálogo (112) · J2 [h](HANDOFF_JANELA_2.md) lacunas L1–L19 · J3 [h](HANDOFF_JANELA_3.md) 6 aprov./fábrica 34 · J4 [h](HANDOFF_JANELA_4.md) 11/44 · J5 [h](HANDOFF_JANELA_5.md) 18/45 (`b74b608d0e`).
- Óleo Real 27/10: [W2 E4](W2_Knife_E4.md) 22/49 · W3 P ([Knife](W3_P_Knife.md) · [Dabber](W3_P_Dabber.md) · [Metallic](W3_P_Metallic.md)) 27/54 · smoke do dono ([§3g](01_CATALOGO.md)) saem Metallic Splats e [Oil Bristle](W3_oil_bristle_W2_x_W3.md), Knife Textured Contrast 0,25 ⇒ **26/52**. Candidatos `EM_ESTUDO_W2`: Knife Smooth/Smooth 2/Soft/Dry 2, Dabber 2/3/Fan, Metallic Round. [Prancha](pranchas/W3_presets.jpg).

## Decisões de produto

1–18 em [`HANDOFF_JANELA_5.md`](HANDOFF_JANELA_5.md#decisões-de-produto-movidas-do-estado-em-2810-j6). Valem: (1) lâmina FIXA no eixo — confirmado na J6; (2) matriz é triagem, veredito = prancha + cega; (12) «diferença visível» = confiança média/alta. Pendentes do dono: 5 (cauda do taper), 8 (tecto de tamanho), no PDF.

## J7 (30/10, [handoff](HANDOFF_JANELA_7.md)): papel fora do Desfazer `0d18a1e9ad` · #53 `cd07881b37` · censo do papel de cor `b1b0cdb513` (ABERTO #54 Marker Bullet) · #55 ladeira do óleo `9318d76827`.

## J8 (30/10)

- **Dívida da J7** `1a7c1aa31c`: o #53 mudou 2 portões ao byte que a J7 não correu (`shape_color_tests` cena 6, `opcoes_tests` S inclinado); sobre o branco ≤ 2 níveis ⇒ regravados.
- **Item 2 OKLCH** `3c283ce666`: `layers::rampa_nova()` (Grain Colors, Shape Color, camada de textura, Reset); `ColorRamp::default()` fica RGB; preset/projeto antigo traz o modo gravado.
- **Item 1 Over Oil** `1e9a32d0af` : Tint · **Mix** (fábrica) · Blend em `OleoCelula::aguada`. **Smoke do dono 30/10: REPROVADO** (chapado, degraus, pretos) ⇒ [handoff J8](HANDOFF_JANELA_8.md).

## J9 (30/10): o Over Oil contra o Rebelle — À ESPERA DO «SERVE» DO DONO

- **Oráculo** (`fixtures/rebelle/over_oil/` + cabeçalho; `over_oil_{rebelle,regua,simula}`): no Rebelle a aguada é
  um VÉU (α-sobre, α 0,35 como no papel; assentada 0,20, nos vales), escuros 0, transição 8 px. Nós (Mix): K-M escurece,
  corte de 1,3 px, escuros = fio de óleo invisível + K-M (hipóteses b, c do J8 confirmadas). Véu simulado (D): [prancha](pranchas/J9_over_oil_hoje_proposta_rebelle.jpg).
- **Dono 30/10 recusa o véu (D) e dá a LEI:** óleo e aguada são a MESMA tinta com diluições diferentes; misturam pela
  QUANTIDADE (a pouco diluída dilui, a muito diluída engrossa); sem creme, preto nem degraus. Simulação **E**
  (`mistura`: `f = q_w/(q_o+q_w)`, subtractiva, papel fora da média, relevo × `1 − f`): escuros 0, transição 6 px;
  [prancha](pranchas/J9_over_oil_mistura_do_dono.jpg).
- **Dono 30/10 (ordem final): «a versão do MAIN, sem áreas brancas ou bege».** Over Oil REVERTIDO (`1e9a32d0af`); lei do
  main + duas curas em `assenta`: (1) sobre o óleo (`Ecra::com_oleo`, rampa `OLEO_PLENO` 0,04, sem o Real Oil na mão:
  `PINCEL_DE_OLEO`) o tapar extra sai — a cobertura é a do pigmento (Rebelle: véu); (2) `na_caixa_das_tintas`: o branco
  do tapar extra vira transparência (o papel aparece; no branco o mesmo píxel). Gates `o_tapar_extra_fica_no_segmento_das_tintas`,
  `sobre_o_oleo_assentar_preserva_o_ecra_seco`, `o_tapar_extra_nao_pinta_branco_no_papel_de_cor` (célula medida, +31 → ≤ 2).
  Cena bege: claros 2 972 → 186 a meio (brilhos do óleo), 447 → 0 assentada. Barras do Rebelle verdes. [Prancha](pranchas/J9_main_sem_branco.jpg).
  **Smoke do dono 30/10: a versão ATUAL fica (melhor que o main); pedidos A (arrasto durante o traço), B (mistura
  molhada pela média das consistências, borda com transição) e C (New Image: papel texturizado)** ⇒ [handoff J9](HANDOFF_JANELA_9.md).

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

## J6 — óleo, grão fino (`9fba4165d8` `37529e35de` `b00ee7cbdc`)

`borda_suave` só suaviza borda RECTA (gate `o_grao_de_um_px_sobrevive_ao_oleo`); números e METODOLOGIA no
[handoff](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-29_PINCEIS2.md) e em [`REGRAS_DE_CUSTO.md`](REGRAS_DE_CUSTO.md).

## J6 — pedidos do dono no smoke (28/10): fechados 29/10 (`a5e727f203` `b17f8d64c3` `1d0ccb1524` `eabfc63ccd`).

## J6 passo 3 — pincel a pincel: [movido para o handoff da J6](HANDOFF_JANELA_6.md#movido-do-estado-j9--j6-passo-3--pincel-a-pincel-16-só-por-valores-16-travados-por-motor-7-sem-oráculo)

## Bancada

[`04_BANCADA.md`](04_BANCADA.md) §«Notas da bancada».
