# ESTADO da missão Pincéis — MISSÃO 2 EM CURSO (J6, 2026-10-28) (≤ 8 KB)

> Missão 1 encerrada na J5 (18 aprovados; o que trava é MOTOR; [integração](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md)).
> Contrato: [`00_MISSAO.md`](00_MISSAO.md) · [regras de custo](REGRAS_DE_CUSTO.md). Detalhe: `HANDOFF_JANELA_<n>.md`, `exames/<família>.md`.

## Feito

- **J1** ([handoff](HANDOFF_JANELA_1.md)): bateria, oráculos fotografados, catálogo (112), bancada, `03_MOTORES.md`.
- **J2** ([handoff](HANDOFF_JANELA_2.md)): as lacunas de motor L1–L19 e as 4 leis sistémicas (`06_LEIS_DAS_LACUNAS.md`).
- **J3** ([handoff](HANDOFF_JANELA_3.md)): fábrica por família, `BRIEFING_FAMILIA.md`; A–E, J: 6 aprovados. Fábrica 34.
- **J4** ([handoff](HANDOFF_JANELA_4.md)): F–M: **11 aprovados** (Filbert, Blender Soft; Hard/Kneaded Eraser, Halftone,
  Stars, Bokeh, Dodge, Burn, Color Glaze, Ballpoint). Fábrica 44.
- **J5** ([handoff](HANDOFF_JANELA_5.md)): **Pastel Side aprovado**; curados não aprovados e reprovados no handoff;
  biblioteca (matriz, custo) e tutorial PDF; fecho até `b74b608d0e`. **Total: 18 aprovados, fábrica 45.**

- **Óleo Real W2 E4** (27/10, [`W2_Knife_E4.md`](W2_Knife_E4.md)): **+4 facas aprovadas** — Knife Smooth 3, Oily, Grainy,
  Dry. **Total: 22 aprovados, fábrica 49.**

- **Óleo Real W3 P** (27/10, [`W3_P_Knife.md`](W3_P_Knife.md) · [`W3_P_Dabber.md`](W3_P_Dabber.md) ·
  [`W3_P_Metallic.md`](W3_P_Metallic.md)): **+5 aprovados** — Knife Textured, Knife Stain, Dabber 4, Metallic Flat,
  Metallic Splats. **Total: 27 aprovados, fábrica 54.** Candidatos (`EM_ESTUDO_W2`): Knife Smooth · Smooth 2 · Soft ·
  Dry 2; Dabber 2 · 3 · Fan; Metallic Round. Prancha dos 13: [`pranchas/W3_presets.jpg`](pranchas/W3_presets.jpg).

- **Smoke do dono 27/10** ([`01_CATALOGO §3g`](01_CATALOGO.md)): saem da fábrica o **Metallic Splats** (reprovado: «muito
  inferior ao do Rebelle») e o **Oil Bristle** («a princípio»; medição W2 × W3 em
  [`W3_oil_bristle_W2_x_W3.md`](W3_oil_bristle_W2_x_W3.md)) — candidatos com receita (`EM_ESTUDO_W2`). Knife Textured
  sem os cortes do 1.º traço (Contrast da textura 0,25). **Total: 26 aprovados, fábrica 52** (Wet 36, Digital 16).

## Decisões de produto (com o porquê)

1. Filbert/Flat Oily/Knife Smooth só voltam com o exame e o Rebelle ao lado; a lâmina do Rebelle é FIXA no eixo do
   preset (06 §1). J4: o Filbert voltou (aprovado); as lâminas continuam em estudo.
2. A matriz é TRIAGEM (d ≥ 0,596; série graduada: degrau ≥ 0,40; `matriz/escala.json`); o veredito é prancha + cega.
   J4: ferramentas (tiram/movem tinta) têm grupo e escala próprios (`matriz/escala_ferramenta.json`).
3. Os pré-existentes passam o mesmo exame: cura contra a origem, ou fundem/saem; «curado, não aprovado» fica.
4. Oráculo que manda: Rebelle; ArtRage/MyPaint/Krita onde ele não tem. GIMP fora.
5. A cauda do taper volta só no Digital. ⏳ AGUARDA o smoke do dono (pergunta no PDF).
6. Dual brush só no Digital e no Wet.
7. Modos = W3C em sRGB; Saturation/Linear Burn/Dodge do Rebelle a 10× não se perseguem.
8. Tecto de tamanho por preset (Max Size do Rebelle). ⏳ AGUARDA o smoke do dono (pergunta no PDF).
9. A rampa de pressão é a do DONO, não a do Rebelle (piso revogado 18/10).
10. 2H = HB com menos pigmento. 11. Round Pressure → Hard Round. 13. Canetas duras fundem pela matriz.
12. «Diferença visível» = confiança média/alta, ou repetida por dois revisores novos.
14. Grafites ficam no Spacing 0,1 + Continuous (dono 18/10).
15. A régua da rampa não reprova a entrada tardia do oráculo — FEITO na lei (J4, `399a387c6d`): abaixo do 1.º decil
    que o oráculo pinta, erro 0.
16. Foto nova do Rebelle vence número antigo gravado num portão (Square Hard).
17. O Dabber ficou como está (matriz final da J5: sem par < 0,596).
18. (J5) Gouache Flat fica (0,763 do Wet Flat; a cura partia 3 leis). Wet Dry Brush não funde (vizinho Airbrush 1,236).

## MISSÃO 2 — JANELA 6 (28/10, [`JANELA_6.md`](JANELA_6.md)): A FILA (censo J6, por pincéis destravados)

0. ⭐ Dono 28/10: Oil/* com o Real Oil LIGADO (`e314e58cec` `d94e19a57e` `105f28791d` `78fb8c7c77` `5470ce0969`): óleo 2×
   mais rápido ao bit; bancada fotografa o óleo do traço. Cega (9 óleos): Rebelle achado 11/27 (acaso 9) ⇒ seguem
   aprovados com óleo Filbert 396 · K. Textured 374 · K. Dry 362 · K. Smooth 3 352 · K. Oily 342 (régua > sem óleo).
   Mais longe na régua (EM CURSO, motor: o óleo derrete o grão fino): K. Stain 162 · K. Grainy 226 · Dabber 4 322 ·
   Metallic Flat 326. O OOM era paralelismo + 19 GB de /tmp órfão (11,5 GB de targets apagados).
1. Borda da mancha molhada ~10 px + traço de 200 px do filme (`exames/aguarela.md`): Wet Liner, Wet Wash, Mop — EM CURSO.
2. ✅ Pontas a direito (`10eee50b0d` motor, `fd73b349ca` r2; [`J6_pontas_rectas.md`](J6_pontas_rectas.md)): **Soft Pastel
   APROVADO** (cega r2: o revisor tomou o nosso pelo Rebelle nas 6 linhas). Square Hard curado não aprovado (r2: faixa
   larga sólida «borda de régua», média). Charcoal/Wax Scrape: o Rebelle os acaba REDONDOS (curvatura 0,50/0,28) ⇒ não
   é esta lacuna. Fila: o traço largo esbate-se ~100–150 px nas pontas; densidade/poeira que mudam com o tamanho.
   Motor 1 (borda/filme/tampas, `8394f62eed` `5f6ae47051` `59f64037c1` `5cd611289a` `987763fc42`): Mop REPROVADO;
   Wet Liner e Wet Wash curados não aprovados (cega r2, média: traço largo chapado com tampa dura; Wash com contorno
   fino escuro do edge-darkening 100). Fila: a tampa do Rebelle esbate ~60 px para dentro do corpo do traço.
4. ✅ Misturador perto da tinta (`eac8182beb` `db69ce13f2`, knob `blendReach`; alcance 52→36 px, Rebelle 30): Blender
   Grainy curado não aprovado (preset de estudo; os grumos do Rebelle são grossos e rasgados — grão da máscara: fila).
3. ✅ Secador nivela (`48da05baa5` `0ea881a460`, knob `dryerLevel`; cruzamento 0,35–1,63 → 0,98–1,10, Rebelle 0,9–1,14):
   Dry/Lift curado não aprovado (falta a linha escura 1,3 nos lados; o anel rouba tinta de fora — fila).
6. ✅ Guache cobre (`600152c535` `20a320bbc9`, knob `wetCover`; resto Flat 0,36 → 0,10, Rebelle 0,11): Gouache Wash
   e Dry curados não aprovados (cega: sem fiapos de cerda, tampa redonda, 200 px chapado — cerdas que crescem com o
   diâmetro: fila); Gouache Flat com `wetCover` — EM CURSO.
5. ✅ Fio do arrasto (`3f6377fcec` `49211e2779`, `smear_fio`; linhas vazias 35/118 → 0, Rebelle 0): Smudge Finger
   curado não aprovado (fio pálido 1,25 × 2,14; língua oca sob a faixa). Portão sem_glaze: verde (`e42004f3e8`; o
   vermelho vinha de `5291edbc75`, 25/10, lei deliberada do tubo — repintado com o porquê). Tampas do traço largo (Square Hard, Wet Liner, Wet Wash) — EM CURSO. 8. Particles/Glitter: SEM oráculo que as
   pinte (efeitos.md «Para a fila» 4–5) ⇒ ficam fora até haver foto.
7. ✅ Lâmina (`a6dcb2afa1` `883ea9d7c2`, [`J6_lamina.md`](J6_lamina.md)): o Rebelle NÃO gira — cada plano fica fixo no
   seu eixo; sem knob novo, gate `a_lamina_de_cada_preset_e_a_do_oraculo`. Craft Flat, Broad Nib, Compressed Charcoal,
   Bristle Watercolor ficam em estudo (o que se vê é outra lacuna); receitas Oil em `J6_lamina_oil_receitas.md`.
   (Smudge Finger) · 6. azul sobre vermelho molhado (Gouache Wash/Dry/Round) · 7. lâmina que gira (C38/L3: A-4, B-9,
   D-3, F-2, G-5..G-8, I-8; L16 depende) · 8. partículas · ondulação (Oil Bristle) · mip das pontas (Foliage, Noise Grit).
9. Óleo Real: `ESTADO_MISSAO_W3.md` «Fila para a próxima onda» + HANDOFF_W3b §Aberto 2–5.
10. Só valores (sem motor): Wax Crayon (F-10), Wood Stick (C-15), Blur (L-5).
- **Mistura em luz linear: FORA** (decisão do dono; ele não estava presente — ordem de desligar o PC no fim).
- ⚠️ Censo: o disco tem 65 PNG de fábrica (44 Wet, 21 Digital/Watercolor) × 52 declarados — conferir por
  `the_factory_presets_are_counted`. Também: brecha `arch_color_space_typed`; Ink Brush a vigiar (p95 12,54 ms).

## Bancada

[`04_BANCADA.md`](04_BANCADA.md) §«Notas da bancada» (exame.sh, PH2D_EXAME_*, fábrica, custo).
