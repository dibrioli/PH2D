/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-10b_mistura_previa_secos.md

Estado (10/11): linha `line/PainterWatercolor`, HEAD `01b3678a88`, base `main` `f254e42bf1`, por integrar. O anterior:
`PROMPT_CONTINUACAO_2026-11-10_pastel_grao_e_mistura.md` (itens 3 e 4 FEITOS; 1 e 2 NÃO).

FEITO nesta janela:
- `f0ef026f0e` — **Bug #81**: a mistura Wet blend é CAUSAL. Amostra só a metade de trás da pegada; o texel atrás do centro recebe o
  estado do reservatório de quando o centro passou (`Historico`, 32 marcos, `crates/ph2d-painter-brush/src/mistura_opaca.rs`);
  DESFASAGEM 1,0→1,25 (varrida para guardar o patamar: fração do fundo a Amount 0,8 disco 0,136 → 0,123). Gate
  `wet_blend_a_cor_de_baixo_vai_no_sentido_do_traco` (mutações sangram). Foto `material_2026-11-10_mistura_causal/antes_depois.png`.
- `b8e515936f` — 14 miniaturas de mistura regravadas, aprovadas pelo dono («as aparências dos pincéis parecem OK»).
- `01b3678a88` — **Bug #82**: o Finger Tip esgota no papel vazio (a película só recarrega se mais velha que DESFASAGEM+2 raios; tende
  ao `corpo`). Rasto 255→31 em 75 px; patamar intacto; 3 miniaturas (1–2 px) regravadas. Gate `wet_blend_o_dedo_esgota_no_papel_vazio`.
  `os_pinceis_que_espalham_enchem_com_a_mistura` estava vermelho desde a K1 (`3d8d15e0eb`): Pastel saiu da lista. BUGS_painter.md tem #81 e #82.
- Binário smoke compilado após `f0ef026f0e`; há commits depois ⇒ recompilar.

SMOKE DO DONO 10/11 (2.ª rodada) — ABERTOS, ordem sugerida **3+4 · 1 · 5 · 2**:

3+4. **Pastel «ainda não está com o grain que solicitei» + Chalk/Crayon enchem (dono: SIM)** — UM bloco, rodada já medida em
   `material_2026-11-10_secos/` (README, `var_secos.tsv`, `corre.sh`, `folha_secos.png`). Grão do dono: Noise Tiled 120×120, Angle 0,
   Depth 1,0, Contrast 0,55, Brightness 0,55, Detail 0,30, Roughness 0,50, Warp 0,50 ⇒ params `[0.55,0.55,0.30,0.50,0.50,0.5]`
   (ordem conferida em `texture/patterns/specs.rs:84-90`; Size do cartão = `texture.size`, `brush_texture_settings.rs:95,164`).
   Receita em `crates/ph2d-tool-painter/tests/it/familias/csp_secos.rs`, const `PAPEL_DO_PASTEL` (hoje size 32, `[0.4,0.5,0.3,0.5,0.5,0.5]`, fill 0.5) — NÃO alterada.
   Cobertura na rampa p .35·.5·.65·.8, 1.ª → 5.ª passada:
   - P_hoje 0.17·0.32·0.60·0.91 → 0.64·0.90·0.99·0.99 · P_dono 0.32·0.43·0.69·0.87 → 0.74·0.97·0.99·0.99 (grão mais fino e salpicado).
   - Ch_hoje (Multiply, papel 80, depth 0.3) 0.60·0.66·0.73·0.78 → 0.99 · Ch_H03 ≈0 · Ch_H06 0.09·0.13·0.29·0.26 → 0.34·0.49·0.82·0.84 · Ch_H10 0.58·0.69·0.79·0.89 → 0.99·1.00.
   - Cr_hoje (Multiply, papel 48) 0.33·0.46·0.58·0.71 → 0.86·0.95·0.99·1.00 · Cr_H03 ≈0 · Cr_H06 0.02·0.02·0.05·0.20 → 0.07·0.10·0.25·0.67 · Cr_H10 0.19·0.38·0.58·0.79 → 0.68·0.96·0.99·1.00.
   Leitura: o Height só presta com depth 1,0. O Multiply de hoje já enche, mas liso. `H10` é o candidato. FALTA: o dono escolher pela
   folha (P_dono sim/não; H10 vs hoje), depois regravar receitas + miniaturas e rodar os alarmes por nome.
1. **BUG: a prévia do pincel «nem sempre corresponde; o Pastel mostra errado às vezes»** (foto: preset Pastel + três faixas
   vermelho/amarelo/azul com S borrado = `so_mexe_tinta`). MEDIDO: `preset_aplicar` de qualquer dos 100 presets e depois o Pastel dá a
   prévia PINTADA idêntica ao bit à do Pastel limpo; a thread da prévia não entra em pânico; 1.º quadro do Pastel 2 ms. A
   `ChaveDoPreview` difere entre «A→B» e «limpo→B» em campos sem efeito na imagem (WetKnobs, radius_px, CompositeLayer, contadores
   de versão): ruído, não causa. HIPÓTESE (por leitura, NÃO reproduzida): o dropdown mostra `aplicado[gaveta]`
   (`crates/ph2d-app-painter/src/painter_bridge_presets.rs:174-186,418`; `cards_topo_pinta.rs:109-127`), não o pincel vivo; `aplicar`
   (~l.229-234) reaplica o `paint_mode` LEMBRADO (`lembrados`, política VolumesAndMode, `ph2d-brush-presets/src/estado.rs:145-170`) depois
   de `preset_aplicar` e antes de `usar` ⇒ o Pastel pode reabrir em Smear/Blur sem aviso. Próximo: reproduzir pela ponte
   (`painter_bridge_presets_tests.rs`: `atender(Escolher)`, `aplicar`, `usar`, `lembrar_o_que_se_deixa`) e só então curar; se não
   reproduzir, perguntar ao dono a sequência exata.
5. **Color mixing num cartão PRÓPRIO** (item 4 do prompt anterior): receita `docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`; hoje é opção
   dentro do cartão Basics (`crates/ph2d-panel-painter-layers/src/cards_mistura.rs`); o teste de costura acompanha.
2. **Wet Wash Mix pesado (queda de fps)**. MEDIDO (traço 1 200 px, 200 eventos, smoke, mín. de 3): WWM 30 px mistura 2,5 s · Off 1,0 s;
   80 px 4,8 s · 1,3 s; Round Mixing 80 px 0,29 · 0,04 s. A lei causal custa o MESMO que a de ontem. Repartição: kernel da pegada com
   grão 41 % · pegada lisa 20 % · lei 38 % · limpeza 2 %. Causa: o véu (`dab_jitter.veu 1.0, veu_escala 4.5`, `stroke/spray.rs:~225`)
   sai em TODO ponto com 20× a área, ao passo de 3 % do diâmetro NOMINAL; com distância 0 o `k_passo` do carimbo cai ao mínimo (0,0075).
   O véu É o look (sem ele o traço é outro). Curas candidatas: (a) véu ao passo do PRÓPRIO diâmetro com k escalado para guardar o
   depósito por distância (~3–4×; o alarme de miniatura decide); (b) linhas do carimbo em paralelo (soma por linha em ordem fixa;
   depósito por faixas de ladrilho de 64). Grátis e ao byte: saltar a amostra quando `dist == 0` com reservatório cheio
   (retem = 1, nova = 0). `perf` não tem permissão aqui (`perf_event_paranoid`).

Herdados abertos: Tooth fill não age na ponta de píxel (L18), no Shape de camadas coloridas e na máscara de stamp em cache (controlo
morto: fechar ou esconder) · curva de pressão do K1 a p 0,5–0,65 abaixo da do Krita · traço pálido no branco (a `mistura_opaca` apanha
o branco opaco) · fila anterior: memória do 1.º traço, marca do Wet por camada, Splash, `SPRAY_FIXO_MAX` com Wet blend, painel a 245 px,
dicas longas, papel de imagem sem dente com Grain.

Fecho: bateria por nome — `ph2d-painter-brush` `wet_blend` e `texture::`; `ph2d-tool-painter` `o_dente_enche`, `curas_da_mistura`,
`mistura_opaca`; alarmes `a_largura_de_fabrica_e_a_aprovada` e `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`; recompilar o binário smoke no fim.
Inegociáveis: os do CLAUDE.md. Aparências dos pincéis: aprovadas pelo dono (só ele regrava).
