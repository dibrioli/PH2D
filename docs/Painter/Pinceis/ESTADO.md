# ESTADO da missão Pincéis — MISSÃO 2 EM CURSO (J6, 2026-10-28) (≤ 8 KB)

> Missão 1 encerrada na J5 (18 aprovados; o que trava é MOTOR; [integração](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md)).
> Contrato: [`00_MISSAO.md`](00_MISSAO.md) · [regras de custo](REGRAS_DE_CUSTO.md). Detalhe: `HANDOFF_JANELA_<n>.md`, `exames/<família>.md`.

**Total: 26 aprovados (+ Soft Pastel na J6), fábrica 64 PNG medidos em `assets/brushes/{WetPainter,Digital}` (52 declarados; conferir por `the_factory_presets_are_counted`).**

## Feito

- **J1** ([h](HANDOFF_JANELA_1.md)): bateria, oráculos, catálogo (112), bancada, `03_MOTORES.md`. **J2** ([h](HANDOFF_JANELA_2.md)): lacunas L1–L19, 4 leis (`06_LEIS_DAS_LACUNAS.md`).
- **J3** ([h](HANDOFF_JANELA_3.md)): A–E, J: 6 aprovados, fábrica 34. **J4** ([h](HANDOFF_JANELA_4.md)): F–M: 11 aprovados, fábrica 44.
- **J5** ([h](HANDOFF_JANELA_5.md)): Pastel Side; 18 aprovados, fábrica 45; fecho `b74b608d0e`.
- **Óleo Real W2 E4** (27/10, [`W2_Knife_E4.md`](W2_Knife_E4.md)): +4 facas (Smooth 3, Oily, Grainy, Dry) ⇒ 22 / 49.
- **W3 P** (27/10, [Knife](W3_P_Knife.md) · [Dabber](W3_P_Dabber.md) · [Metallic](W3_P_Metallic.md)): +5 (Knife Textured, Knife Stain, Dabber 4, Metallic Flat, Metallic Splats) ⇒ 27 / 54. Candidatos `EM_ESTUDO_W2`: Knife Smooth · Smooth 2 · Soft · Dry 2; Dabber 2 · 3 · Fan; Metallic Round. Prancha: [`pranchas/W3_presets.jpg`](pranchas/W3_presets.jpg).
- **Smoke do dono 27/10** ([`01_CATALOGO §3g`](01_CATALOGO.md)): saem Metallic Splats («muito inferior ao do Rebelle») e Oil Bristle ([`W3_oil_bristle_W2_x_W3.md`](W3_oil_bristle_W2_x_W3.md)); Knife Textured sem cortes (Contrast 0,25) ⇒ **26 / 52** (Wet 36, Digital 16).

## Decisões de produto

1–18 em [`HANDOFF_JANELA_5.md`](HANDOFF_JANELA_5.md#decisões-de-produto-movidas-do-estado-em-2810-j6). Valem hoje: (1) lâmina FIXA no eixo do preset — confirmado na J6; (2) a matriz é triagem, o veredito é prancha + cega; (12) «diferença visível» = confiança média/alta. Pendentes do dono: 5 (cauda do taper) e 8 (tecto de tamanho), no PDF.

## MISSÃO 2 — JANELA 6 (28/10, [`JANELA_6.md`](JANELA_6.md)): A FILA

0. ⭐ Dono 28/10: Oil/* com o Real Oil LIGADO (`e314e58cec` `d94e19a57e` `105f28791d` `78fb8c7c77` `5470ce0969`): óleo 2× mais rápido ao bit; bancada fotografa o óleo do traço. Cega (9 óleos): Rebelle achado 11/27 (acaso 9) ⇒ seguem aprovados: Filbert 396 · K. Textured 374 · K. Dry 362 · K. Smooth 3 352 · K. Oily 342 (régua > sem óleo). Mais longe na régua (EM CURSO, motor: o óleo derrete o grão fino): K. Stain 162 · K. Grainy 226 · Dabber 4 322 · Metallic Flat 326. O OOM era paralelismo + 19 GB de /tmp órfão (11,5 GB de targets apagados).
1. Borda da mancha molhada ~10 px + traço de 200 px do filme (`exames/aguarela.md`): Wet Liner, Wet Wash, Mop — ✅ ver «Motor 1» abaixo.
2. ✅ Pontas a direito (`10eee50b0d` motor, `fd73b349ca` r2; [`J6_pontas_rectas.md`](J6_pontas_rectas.md)): **Soft Pastel APROVADO** (cega r2: tomou o nosso pelo Rebelle nas 6 linhas). Square Hard curado não aprovado (r2: faixa larga sólida «borda de régua», média). Charcoal/Wax Scrape: o Rebelle os acaba REDONDOS (curvatura 0,50/0,28) ⇒ outra lacuna. Fila: traço largo esbate ~100–150 px nas pontas; densidade/poeira que mudam com o tamanho.
   Motor 1 (borda/filme/tampas, `8394f62eed` `5f6ae47051` `59f64037c1` `5cd611289a` `987763fc42`): Mop REPROVADO; Wet Liner e Wet Wash curados não aprovados (cega r2, média: traço largo chapado com tampa dura; Wash com contorno fino escuro, edge-darkening 100). Fila: a tampa do Rebelle esbate ~60 px para dentro do corpo.
3. ✅ Secador nivela (`48da05baa5` `0ea881a460`, knob `dryerLevel`; cruzamento 0,35–1,63 → 0,98–1,10, Rebelle 0,9–1,14): Dry/Lift curado não aprovado (falta a linha escura 1,3 nos lados; o anel rouba tinta de fora — fila).
4. ✅ Misturador perto da tinta (`eac8182beb` `db69ce13f2`, knob `blendReach`; alcance 52→36 px, Rebelle 30): Blender Grainy curado não aprovado (preset de estudo; grumos do Rebelle grossos e rasgados — grão da máscara: fila).
5. ✅ Fio do arrasto (`3f6377fcec` `49211e2779`, `smear_fio`; linhas vazias 35/118 → 0, Rebelle 0): Smudge Finger curado não aprovado (fio pálido 1,25 × 2,14; língua oca sob a faixa). Portão sem_glaze verde (`e42004f3e8`; o vermelho vinha de `5291edbc75`, 25/10, lei deliberada do tubo — repintado com o porquê). Tampas do traço largo (Square Hard, Wet Liner, Wet Wash) — EM CURSO.
6. ✅ Guache cobre (`600152c535` `20a320bbc9`, knob `wetCover`; resto Flat 0,36 → 0,10, Rebelle 0,11): Gouache Wash e Dry curados não aprovados (sem fiapos de cerda, tampa redonda, 200 px chapado — cerdas que crescem com o diâmetro: fila). Gouache Flat: `wetCover` 1 leva-o ao Rebelle (régua 404→424) mas NÃO entrou (`cc49c5616a`): regerar faz a tabela de largura ver o traço de r 1,5–1,8 (linha pálida de 3 px, tabela P44 diz 0) ⇒ PRÓXIMO depois do óleo: medir a largura do traço fino pela cobertura útil, regerar Gouache/Wet Flat.
9. ✅ Texturas (`f9fbbe7c26`, `DabJitter::subpixel_area`: marca < ½ px pousa pela área; traço de 5 px 0,258 → 0,131,
   Krita 0,122): Spray Can curado não aprovado (cega média/alta: faltam salpicos escuros e discos grandes translúcidos).
   Foliage, Grass, Stipple, Noise Grit, Splatter: sem lacuna comum (receita/carimbo/aguarela) — `exames/texturas.md`.
7. ✅ Lâmina (`a6dcb2afa1` `883ea9d7c2`, [`J6_lamina.md`](J6_lamina.md)): o Rebelle NÃO gira — plano fixo no seu eixo; sem knob novo, gate `a_lamina_de_cada_preset_e_a_do_oraculo`. Craft Flat, Broad Nib, Compressed Charcoal, Bristle Watercolor em estudo (outra lacuna); receitas Oil em `J6_lamina_oil_receitas.md`.
8. Particles/Glitter: SEM oráculo que as pinte (efeitos.md «Para a fila» 4–5) ⇒ fora até haver foto.
9. Óleo Real: `ESTADO_MISSAO_W3.md` «Fila para a próxima onda» + HANDOFF_W3b §Aberto 2–5. 10. Só valores (sem motor): Wax Crayon (F-10), Wood Stick (C-15), Blur (L-5).
- Fila de motor restante: azul sobre vermelho molhado (Gouache Wash/Dry/Round) · lâmina que gira (C38/L3: A-4, B-9, D-3, F-2, G-5..G-8, I-8; L16 depende) · partículas · ondulação (Oil Bristle) · mip das pontas (Foliage, Noise Grit).
- **Mistura em luz linear: FORA** (decisão do dono; ordem de desligar o PC no fim).
- ⚠️ Brecha `arch_color_space_typed`; Ink Brush a vigiar (p95 12,54 ms).

## Bancada

[`04_BANCADA.md`](04_BANCADA.md) §«Notas da bancada» (exame.sh, PH2D_EXAME_*, fábrica, custo).
