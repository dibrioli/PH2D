# Handoff de continuação — Óleo Real W2 (line/PainterWatercolor, 27/10)

## Veredito do smoke (dono, 27/10)

*«MUito bom!»* — aprovado. Backup: tag `backup/oleo-real-W2-2026-10-27` (sobre `086c4f7d31`) +
`~/Documentos/Projetos/PH2D-backups/oleo-real-W2-2026-10-27/` (bundle `main..line`, verificado; tar das
entradas/saídas do oráculo, gitignoradas).

## O que a W2 entregou

Tudo, com commits, números, recusas e o que ficou aberto: [`ESTADO_MISSAO_W2.md`](ESTADO_MISSAO_W2.md). Em resumo:
E1 força/alcance da fenda · E2 ambiente no difuso por sector de fatia · E3 material por célula (`MaterialCelula`) +
sombreado tipo Principled × Cycles · E4 faca com rebordo (`knifeRidge`), 4 facas aprovadas (fábrica 49 / 22), 13
candidatos fora da fábrica (`docs/Painter/Pinceis/W2_{Metallic,Dabber,Knife}_candidatos/` + `EM_ESTUDO_W2`) · bancada
com `PH2D_EXAME_OLEO=1` · custo do óleo a escorrer 28 → 17 s (bit-idêntico). Folhas: `out/W2_folha.jpg`,
`Pinceis/pranchas/W2_18_presets.jpg`. Oráculo e provas de mutação: [`oraculo/README.md`](oraculo/README.md).

## Decisão do dono depois do smoke (27/10) — vira a MISSÃO W3

O material passa a usar os NOMES e as propriedades universais do **Principled BSDF** do Blender, simplificado e
**com SSS**: exactamente **Metallic · Roughness · IOR Level · Coat · Coat Roughness · Sheen · Subsurface · Subsurface
Radius**, no PINCEL (por célula) e no cartão Real Oil (tela inteira), com os mesmos nomes. Saem Spec tint,
Anisotropy, Translucency, Reflectivity, Map scale, Map strength (o mapa de reflexo fica no seletor Environment).
Depois: o motor que falta e os 13 presets, sozinho, testando com o Rebelle.

Prompt: [`PROMPT_MISSAO_W3_material_motor_presets.md`](PROMPT_MISSAO_W3_material_motor_presets.md).

## Vermelhos que já vinham da base (não são desta linha)

`a_largura_e_a_prometida` (10 linhas: Blender Soft/Textured, Smear Shattered/Wet) · `a_altura_de_abertura_de_um_painel_so_encolhe`
(painter_layers 1618 × 1570; inspector 956 × 831) · `as_letras_perdidas_no_degrau_estreito_so_encolhem` e
`a_divida_do_degrau_estreito_so_encolhe` (painter_layers declara 4, mede 3) · `sem_glaze_cada_preset_de_fabrica` (Wet
Wash + 2 Wet Liner) · `censo_dos_controlos::a_linha_esmaecida_e_a_que_nao_age` (4 linhas do Digital).
