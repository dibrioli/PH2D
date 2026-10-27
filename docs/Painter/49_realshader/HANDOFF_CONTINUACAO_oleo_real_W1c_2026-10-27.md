# Handoff de continuação — Óleo Real W1c (line/PainterWatercolor, 26–27/10)

## Veredito do smoke (dono, 27/10)

*«Realmente muito bom. tão bom que merece um backup.»* — aprovado. Backup: tag `backup/oleo-real-W1c-2026-10-27`
(sobre `c3365fb579`) + `~/Documentos/Projetos/PH2D-backups/oleo-real-W1c-2026-10-27/` (bundle `main..line`,
verificado; tar das entradas/saídas do oráculo, gitignoradas).

## O que a W1c entregou (commits)

| commit | o quê |
|---|---|
| `be1ae30db3` | oráculo Cycles sobre a nossa tinta; fenda = GTAO + multi-ricochete (Jimenez); reflexo de ambiente HDRI CC0 pré-filtrado (só no especular, oclusão na direcção reflectida); realce `rola` que enche a folga até 1 |
| `7ce93d890b` | sonda de custo por ambiente (Clássico 1 547 ns/célula, HDRI ≈ 1 870) + README do oráculo |
| `c3365fb579` | seletor **Environment** (Classic/Studio/Interior/Courtyard) no cartão Real Oil |

Lei, números, recusas medidas e comandos: [`oraculo/README.md`](oraculo/README.md) · régua [`oraculo/regua_W1c.txt`](oraculo/regua_W1c.txt)
· folhas [`out/W1c_folha.jpg`](out/W1c_folha.jpg). Código: `crates/ph2d-wet-paint/src/{oleo_real,oleo_ambiente}.rs`,
gates em `oleo_real_tests.rs` (10 verdes), sonda `medida_oleo_real_oraculo` (`#[ignore]`).

## O que o dono pediu a seguir (27/10) — vira a MISSÃO W2

1. Material realista como o Cycles, com todos os ajustes de material.
2. Ajuste de contraste da sombra de fenda (tipo AO).
3. HDRI como meia-esfera sobre o papel, iluminando tudo (não só o reflexo).
4. Depois: presets parecidos com os do Rebelle — Metallic Flat 3 · Round 3 · Splats; os 5 Dabber; as 10 Knife.

Prompt: [`PROMPT_MISSAO_W2_material_ambiente_pinceis.md`](PROMPT_MISSAO_W2_material_ambiente_pinceis.md).

## Fica aberto (medido)

- Brilho do Estúdio mais fraco que o Cycles (p99 0,26 × 0,49; área > 0,3: 0,73 % × 2,0 %); os poros do Dabber cintilam menos de propósito (filtro de Kaplanyan, cura dos pontinhos brancos da W1b).
- Difuso da tinta 0,71 × 0,83: falta o rebote da LÂMPADA nas paredes vizinhas (+11 % p50 no Cycles).
- Oil Bristle: viés de visibilidade +0,11 (os sulcos dos fios).
- Passe de placa + pirâmide de máximos; facas (lâmina que gira, C38) e rebordo.
- Vermelho pré-existente `censo_dos_controlos::a_linha_esmaecida_e_a_que_nao_age` (4 linhas do Digital), ver
  [handoff W0](HANDOFF_CONTINUACAO_oleo_real_W0_W1cpu_2026-10-26.md) linha 64 — não é desta onda.
