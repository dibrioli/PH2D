/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-10d_veu_meia_definicao.md

Estado (10/11): linha `line/PainterWatercolor`, HEAD = o commit deste prompt (depois de `6537d5309a`), base `main` `f254e42bf1`, por
integrar. O anterior: `PROMPT_CONTINUACAO_2026-11-10c_wet_wash_mix_desempenho.md` (tudo o que ele lista como FEITO continua feito).
O guia do método e das curas: [`docs/Painter/Pinceis/COMO_ACELERAR_UM_PINCEL.md`](../Pinceis/COMO_ACELERAR_UM_PINCEL.md) — LEIA-O
antes de medir; a história com números está no `BUGS_painter.md` #84 (curas 1–5).

FEITO na janela 10c (Bug #84, o Wet Wash Mix lento em traço rápido):
- Medido no APP (`PH2D_PAINT_PERF`, tela 1024², papel + camada transparente): 8,74 ms por entrega × 9,4 entregas/quadro, período
  96,5 ms (6 fps); prévia/painéis/chrome/upload ~0. A caneta do dono entrega ~40 px por evento (~4 000 px/s). A sonda
  `sonda_custo_no_app` com `PH2D_SONDA_EVQ=9 PH2D_SONDA_VELS=40 PH2D_SONDA_TAMS=30 PH2D_SONDA_TELA=1024` reproduz o app (65–67 ms).
- Por ablação: o véu (2.ª partícula do spray, 4,5× o raio) é ~83 % do traço. Por carimbo de véu (~10 k texels): pegada 107 µs
  (32 ns/visita em série), película 76–86 µs (17 em série); o tecto era o fork-join POR CARIMBO.
- `6537d5309a` — as pegadas do lote em paralelo ENTRE os dabs (`pegada_textured_masked_po_com_liso` → `PegadaLocal`,
  `DabCtx::ox`, `Pegada::Mascara::{org, largura}`; a cópia no centro da anterior não se calcula). AO BIT: gate
  `a_pegada_na_caixa_e_a_da_tela_ao_bit` (vermelho por mutação), `.ppm` da `sonda_custo_do_veu` iguais em `cmp`, alarmes verdes.
  360 px/quadro 67 → 41 ms; 90 px/quadro 17,8 → 12,7 ms. **Smoke do dono 10/11: «muito melhor e aceitável».**
- Bateria verde por nome: `ph2d-painter-brush` `wet_blend` `texture::` `dab::` `mistura_opaca`; `ph2d-tool-painter`
  `o_dente_enche` `curas_da_mistura` `mistura_opaca` `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela`
  `o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela` `a_largura_de_fabrica_e_a_aprovada` `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`.
- ⛔ Recusas MEDIDAS (no guia): faixas mais finas no kernel (satura ~60 µs); pintar a película por ladrilho 64×16 (o mesmo
  tempo); pegadas do lote em série por dentro (mais LENTO, 0,29 → 0,77 s); afinar o véu a ¼–½ passo (listras).
- ⚠️ `perf`/`samply` BLOQUEADOS nesta máquina (perf_event_paranoid 2): ablação + `Instant` temporários (removê-los antes do commit).

PRÓXIMO (ordem):
1. ⭐ **O véu a MEIA definição** (dono 10/11: «vamos tentar»; só este pincel): a película é agora ~60 % e sequencial entre
   carimbos — o único corte grande que resta. Nota medida: SÓ a pegada a meia definição não serve (a pegada já saiu do caminho
   crítico); tem de ir a PELÍCULA também. Nota medida: no Wet Wash Mix só os carimbos de véu chegam à película (a cópia salta-se),
   logo o plano do traço inteiro é véu ⇒ desenho proposto: no plano do traço (`acumulando_no_plano`, `base` Some), um plano + `Sombra`
   a meia resolução para este pincel (carimbo com centro/raio/caixa ÷ 2, o grão Tiled com `size` ÷ 2 e a tela ÷ 2 — a memória do
   grão chaveia pelo tamanho; base e «visível» reduzidas 2×2 uma vez por traço, são constantes no traço), e a ampliação bilinear
   (pré-multiplicada) para o plano inteiro só na caixa tocada, uma vez por quadro (`PlanoDoTraco::pendente`). As distâncias da película
   (`DESFASAGEM × raio`, `passo`, `arco`) escalam juntas. Ligá-lo por um campo do motor sem UI na receita do Wet Wash Mix
   (`tests/it/familias/csp_secos.rs` ~142) e regravar o preset de fábrica pelo caminho de sempre. Medir como VARIANTE EM EXECUÇÃO na
   `sonda_custo_no_app`/`sonda_custo_do_veu` (inteira × meia, intercaladas, o mínimo de 3, `--profile smoke`) e mostrar ao dono a
   mancha lado a lado (`target/sonda-veu/*.ppm` → PNG) ANTES do smoke; o alarme de miniatura vai vermelho e só ele regrava.
2. Corrigir o comentário da receita do Wet Wash Mix (`csp_secos.rs` ~142: diz «véu 2× e espalhamento de 2 diâmetros», o código tem
   `veu_escala 4.5` e jitter 0) · gate que arraste os CINCO números do cartão Paint Mixing (e a doc que diz «seis números») ·
   pôr o Wet Wash Mix no `o_custo_no_caminho_do_app` (só Filbert/Express Oil hoje) com a velocidade do dono (9 × 40 px).
3. Smoke do dono PENDENTE: prévia do pincel (Bug #83), cartão Paint Mixing, «Save as default» (`6c0c7a59ca`).
4. Herdados: os do item 4 do prompt 10c (Tooth fill na ponta de píxel/Shape colorido/stamp em cache, curva de pressão do K1, traço
   pálido no branco, a fila anterior; tectos do painel 1694→1718→1838 px para o integrador).

Fecho: a bateria acima por nome + os testes novos; deps/features ganharam usos (rayon na rota da mistura; ADR-0200; `float_roundtrip`
do 10b) ⇒ na integração, suíte inteira. Recompilar o binário smoke no fim (2×, 2.ª saída «Finished» sem «Compiling»).
Medir sempre em `--profile smoke`, mínimo de 3, carga abaixo de ~5. Aparências dos pincéis: só o dono regrava.
Inegociáveis: os do CLAUDE.md.
