/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-10c_wet_wash_mix_desempenho.md

Estado (10/11): linha `line/PainterWatercolor`, HEAD `5a02ad9c9d`, base `main` `f254e42bf1`, por integrar. O anterior:
`PROMPT_CONTINUACAO_2026-11-10b_mistura_previa_secos.md`.

FEITO nas janelas anteriores (detalhe no 10b): **Bug #81** wet blend causal (`f0ef026f0e`, miniaturas `b8e515936f`) · **Bug #82** Finger Tip esgota no papel vazio (`01b3678a88`) · grão do Pastel e Chalk/Crayon a encher o dente (`288616b743`) · **Bug #83** prévia do pincel (`16e4206ce2`) · Color Mixing em cartão próprio (`2df0c78389`) · **Paint Mixing** num cartão com um Pigment só (`bed0cc5f64`, `b4998ef0a7`, `ea5f04878a`, `640ed27ae5`; tecto do painel `painter_layers` 1838 px por ordem do dono) · «Save as default» (`6c0c7a59ca`, feature `float_roundtrip` do serde_json).

FEITO nesta janela (desempenho do Wet Wash Mix, **Bug #84**; dono: «muito mais lento que o aceitável»):
- `a1ac3c6b58` — a cópia do véu no mesmo centro salta-se (`ReservatorioOpaco::repete_o_centro`): 1,9×. **Recusa MEDIDA:** afinar o véu a ¼–½ passo (dá listras).
- Auditoria de 2 lentes. Achados: o gate de custo do app `o_custo_no_caminho_do_app` só media Filbert/Express Oil · a repartição de custo do 10b estava velha · o comentário da receita do Wet Wash Mix (`csp_secos.rs` ~142) diz «véu 2× e espalhamento de 2 diâmetros», mas o código tem `veu_escala 4.5` e jitter 0 (NÃO corrigido, só comentário) · `color_mixing.rs` só arrasta 1 dos 5 números do cartão Paint Mixing e a doc dele diz «seis números».
- `38985f2b15` — **ADR-0200** (número por RECONTAR na integração): película da mistura em faixas de 16 linhas no pool rayon, taxa por tabela de 256, kernel `band_split`/`parallel_band_cached` no pool (`faixas_no_pool`): 3,8×. Gate `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` (razão WetBlend÷Off ≤ 1,3).
- `9f256bc878` — o plano do traço compõe uma vez por quadro (`PlanoDoTraco::pendente`). Gate `o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela`.
- `334d76f347` — cache do grão Tiled (`texture/grao_cache.rs`): 3× no kernel, ao bit.
- `5a02ad9c9d` — a passagem lisa funde-se na do grão (`dab/liso.rs`), ao bit. ⚠️ O gate da razão passa, mas com margem curta (1,26–1,29 contra 1,3 sob carga): risco de flake, ver [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md).
- Sonda `crates/ph2d-tool-painter/tests/it/sonda_custo_do_veu.rs` (`#[ignore]`; drena a cada 3 eventos, como o app). Wet Wash Mix, traço de 1 200 px: 30 px 2,50 → ~0,31 s; 80 px 3,64 → ~0,46 s.
- **Miniaturas POR APROVAR pelo dono** (alarme vermelho; só ele regrava): Wet Wash Mix 25 347 px; Dry Gouache 10, Gouache 10, Gouache Blender 30, Paint And Apply 1, Round Mixing 1, Thick Oil Paint 3 (a amostra soma por faixa: muda a última casa). Relatório: `target/relatorio-pinceis/index.html`.
- Deps/features: o rayon em `ph2d-painter-brush` ganhou usos (ADR-0200) e há a `float_roundtrip` do 10b ⇒ na integração: suíte inteira.

ABERTO:
- ⛔⛔ **PRIMEIRO — smoke do dono 10/11 depois de 5a02ad9c9d: «bem melhor mas inaceitável em traços rápidos (30 px = 6 fps)».**
  A sonda (`sonda_custo_do_veu`, só o `on_canvas_pointer` + drenagem a cada 3 eventos, sem GPU nem quadro) dava ~0,26 ms
  por px ⇒ ~35 fps a 90 px/quadro: o APP paga ~5× o que a sonda vê. NÃO cure às cegas: meça no caminho do app
  (`PH2D_PAINT_PERF=1`, o `PaintFrameTimer`, `measure_input_cost.rs`/`measure_preview_drain.rs`, `o_custo_no_caminho_do_app`):
  quantos px/eventos por quadro a caneta real entrega num traço rápido, o `take_preview_arc` (composição das camadas, upload
  à GPU da caixa do véu), e o acordar do pool rayon com o render a correr (ADR-0173: 1–2 ms por lote, 20–40 ms com um
  trabalhador desagendado). Depois ponha o Wet Wash Mix no gate do caminho do app com teto de 16 ms/quadro.
1. **Desempenho do Wet Wash Mix:** ainda acima de 16 ms/quadro a 90 px/quadro. Próximo corte EXATO = a película (`mistura_opaca_carimbo`, ~45 % da rota). Proposta NÃO exata, à espera do dono: o véu a meia resolução (~3× no kernel; o alarme de miniatura decide). Risco: acordar o pool no app com trabalhador desagendado (ADR-0173) — o smoke do dono decide.
2. Corrigir o comentário da receita do Wet Wash Mix (`csp_secos.rs` ~142) · gate que arraste os CINCO números do cartão Paint Mixing (e corrigir «seis números» na doc) · alargar `o_custo_no_caminho_do_app` a mais pincéis.
3. Smoke do dono PENDENTE: prévia do pincel (Bug #83), com as duas decisões tomadas sem ele (Sync ⇒ menu sem nome; sem Sync, voltar reaplica o preset e ajustes fora de tamanho/opacidade/modo perdem-se) · cartão Paint Mixing · «Save as default» do Wet Wash (`6c0c7a59ca`) · fluidez do Wet Wash Mix e as miniaturas acima.
4. Herdados: Tooth fill não age na ponta de píxel (L18), no Shape de camadas coloridas e na máscara de stamp em cache (controlo morto: fechar ou esconder) · curva de pressão do K1 a p 0,5–0,65 abaixo da do Krita · traço pálido no branco (a `mistura_opaca` apanha o branco opaco) · fila anterior: memória do 1.º traço, marca do Wet por camada, Splash, `SPRAY_FIXO_MAX` com Wet blend, painel a 245 px, dicas longas, papel de imagem sem dente com Grain. O integrador confere os tectos do painel (1694→1718→1838 px).

Fecho: bateria por nome — `ph2d-painter-brush` `wet_blend` e `texture::`; `ph2d-tool-painter` `o_dente_enche`, `curas_da_mistura`, `mistura_opaca`, `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela`, `o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela`; `ph2d-panel-painter-layers` `paint_mixing_e_um_cartao_com_um_pigment_so`; `ph2d-app-painter` `o_menu_nomeia_o_pincel_que_esta_na_mao`; `ph2d-brush-presets` `um_f64_calculado_volta_ao_bit_pelo_json`; alarmes `a_largura_de_fabrica_e_a_aprovada` e `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`; recompilar o binário smoke no fim. Medir sempre em `--profile smoke`, mínimo de 3, carga abaixo de ~5.
Inegociáveis: os do CLAUDE.md. Aparências dos pincéis: aprovadas pelo dono (só ele regrava).
