# HANDOFF — Janela 9 (30/10), linha `line/PainterWatercolor`

## Feito (commits)
- `ba682757d1` **Rebelle como oráculo do Over Oil**: fixture `NewFeaturesWetPaint/fixtures/rebelle/over_oil/` (+ `cabecalho.json`),
  `over_oil_rebelle{.py,_tela.sh}` (a cena da sonda, deslocada +380,+240 numa tela 1000×640), `over_oil_regua.py` (cor, desvio,
  transição, patamar, escuros no cruzamento), `over_oil_simula.py` (variantes A–E por retoque). `gotas_inicia/sessao.sh`: a guarda
  do oráculo de 26/10 (a antiga recusava a sessão legítima). Medido no Rebelle: sobre o óleo a aguada é VÉU (α 0,35 × 0,38 no
  papel; assentada 0,20, nos vales), molhado = seco, escuros 0, transição 8 px — **o Rebelle NÃO mistura a aguada com o óleo**.
- `aedb6f4adc` `d6352cf6ae` simulação E (a lei descrita pelo dono: mistura pela quantidade) — superada pela ordem seguinte.
- `776a9ff173` **Over Oil revertido** (`1e9a32d0af`: dropdown, preset, `OleoCelula::aguada`, ids, i18n) + duas curas em
  `ph2d-wet-paint/src/assenta.rs`: (1) `Ecra::Cobre::oleo` / `Ecra::com_oleo` (rampa `OLEO_PLENO` 0,04; com o Real Oil na mão
  `PINCEL_DE_OLEO` = lei da obra): sobre o óleo o tapar extra sai, a cobertura é a do pigmento; (2) `na_caixa_das_tintas`: o
  branco do tapar extra vira transparência (doc 47), sobre o branco o mesmo píxel; o render passa a lei da CÉLULA à caixa.
  Gates `o_tapar_extra_fica_no_segmento_das_tintas`, `sobre_o_oleo_assentar_preserva_o_ecra_seco`,
  `o_tapar_extra_nao_pinta_branco_no_papel_de_cor`. Sondas `diag_aguada_por_cima_do_oleo` (bege `p_*` e branco `b_*`),
  `diag_a_faixa_abaixo_do_oleo`. Cena bege: claros 2 972 → 186 a meio (brilhos), 447 → 0 assentada; barras do Rebelle verdes.

## Smoke do dono (30/10, depois do `776a9ff173`)
Foto [`pranchas/J9_smoke_dono_aguada_sobre_oleo.png`](pranchas/J9_smoke_dono_aguada_sobre_oleo.png): sem creme, sem preto.
**O dono: «o main não é a referência — a nossa implementação ATUAL é superior».** Fica como base. O que falta (palavras dele):
- **A — durante o traço:** «antes da simulação, a tinta colocada anteriormente já deveria estar sendo afetada, mesclada, como
  se um smear atuasse» (a aguada arrasta e mistura o óleo molhado).
- **B — ao simular:** «a mescla faria efeito, parecido com duas tintas molhadas se misturando, levando em consideração a média
  das consistências»; «as bordas duras da tinta azul não deveriam aparecer: na borda teria uma área de transição»; «a tinta azul
  parece INSOLÚVEL e não deveria ser assim».
- **C — New Image:** mais uma opção, **papel texturizado**, com as definições da foto
  [`pranchas/J9_smoke_dono_papel_texturizado.png`](pranchas/J9_smoke_dono_papel_texturizado.png): Color `#E6D9CC`
  (0,902 · 0,851 · 0,800), Paper **Cold Press**, Relief 0,50, Roughness 0,50, Angle 0, Offset 0/0, Size 1/1, Tooth 1,00,
  Smooth Edges 0, Contrast 0,50, Brightness 0,50.

⚠️ A e B vão ALÉM do Rebelle (lá o óleo é insolúvel à aguada): a lei é a do dono
([feedback](../../../project-memory/feedback_the_owners_reference_product_decides_the_law.md)). Oráculo possível para «duas
tintas molhadas pela média das consistências»: óleo molhado sobre óleo molhado no Rebelle, ou a mistura molhada nossa
(aguarela × aguarela, `mistura_tests`). Motor a ler antes: `tools/smear.rs` (o arrasto), `pickup_assentado` (sim), o
`solver::ladeira_do_oleo` (#55: a água não sobe o óleo — A/B podem pedir exceção), `grid::oleo_celula` (o campo NÃO advecta —
a tinta azul que a aguada leva perde o óleo: a faixa a meio abaixo do óleo mostra o papel).

## Aberto do `776a9ff173` (fecho, 1× no fim)
Regravar 11 miniaturas (1 nível sobre o branco: `sonda_miniatura_antiga_e_de_hoje`) e 2 impressões ao byte
(`opcoes_tests` S·bancada, `shape_color_tests` secção desligada); mutação das duas curas; censo do papel de cor e BUGS #54.
Rebelle: teste com 7 dias em 30/10 (acaba ~06/11).
