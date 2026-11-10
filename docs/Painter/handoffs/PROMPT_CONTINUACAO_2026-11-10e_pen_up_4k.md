/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-10e_pen_up_4k.md

Estado (10/11): linha `line/PainterWatercolor`, base `main` `f254e42bf1`, por integrar. O anterior:
`PROMPT_CONTINUACAO_2026-11-10d_veu_meia_definicao.md` (tudo o que ele lista como FEITO continua feito).

FEITO na janela 10d:
- ⭐ Bug #84 cura 6 — a mistura do Wet Wash Mix a MEIA definição (`4bb5ffaca3`, `stamp_route/mistura/meia.rs`,
  `MisturaOpaca::meia_definicao`, só a receita liga). 360 px/quadro 43,5 → 24 ms. **APROVADA pelo dono 10/11.** Gates
  `o_veu_a_meia_definicao_fica_perto_da_inteira` (4 mutações vermelhas, sobrevive o grão sem escala — documentado) e
  `o_wet_wash_mix_a_velocidade_do_dono_vai_pela_meia_definicao` (red-first). Os dois ficheiros acima de 700 LOC partidos
  (`mistura_opaca/carimbo.rs`, `stamp_route/mistura/lote.rs`). Chave morta `"pigment"` fora de 8 receitas;
  `po_do_papel` no censo do preview. O `color_mixing.rs` arrasta os 5 números. Guia: `Pinceis/COMO_ACELERAR_UM_PINCEL.md`
  §2 (cura 6) e §5 (como aplicar a outro pincel). Memória: `project_wet_wash_mix_half_resolution_approved_2026_11_10.md`
  (escrita no `project-memory/` do primário, por rastrear lá).
- Dono 10/11 aceitou a PROPOSTA (não ordenada ainda): medir todos os pincéis à velocidade dele e oferecer a meia definição
  aos lentos que forem largos e macios, lado a lado, pincel a pincel.

ABERTO — ⭐ o travamento no MOUSE-UP numa imagem 4K, mesmo num traço pequeno (dono 10/11). Medido, NÃO é:
- a ferramenta: `sonda_pen_up_4k` (`tests/it/`): Up ≤ 2 ms a 3840×2160 em 5 pincéis (Filbert 7,5); 1.º traço igual;
- o caminho da prévia do app (`measure_o_quadro_do_pen_up_em_4k`, `ph2d-app-painter/src/painter_preview_measure.rs`, com a
  placa e `bind_document`): quadro do Up 0,8 ms, os 3 seguintes ~0;
- a releitura da folha de animação (`imagem_aplicada`, chamada por `sheet_cache::follow_live` ~30 quadros depois do Up,
  `fase_snapshots_publish.rs:94`): 1,6–7 ms a 4K — mas o que vem DEPOIS dela (`SheetImage::new`, o `guess` da grelha, as
  miniaturas, a textura da folha) NÃO foi medido.
Suspeitos por medir (explorador, leitura): `shapebake` do PANEL (`painter_bridge.rs:527`, `capture_layers` O(tela ×
camadas) no 1.º quadro sem traço, só com Forma capturada de camadas); o resto do `follow_live`; autosave
(`ph2d-recovery`, por período); trabalho da shell fora do `dispatch` (invisível ao `PH2D_PAINT_PERF`).
PRÓXIMO: perguntar ao dono (3 respostas decidem) — o travamento é NO soltar ou ~meio segundo depois? acontece numa imagem
4K NOVA, de uma camada só? e com o editor/cartão de animação fechado? — e medir o suspeito que sobrar no MESMO processo
(4K × 1024²). Não dá para conduzir o app numa tela virtual (o XTest é ignorado na Xwayland, `fotografa_cena.sh`).

Herdados: smoke do dono PENDENTE da prévia do pincel (Bug #83), do cartão Paint Mixing e do «Save as default»
(`6c0c7a59ca`); os do item 4 do prompt 10c.

Fecho: bateria por nome do 10d (`ph2d-painter-brush` `wet_blend` `texture::` `dab::` `mistura_opaca`; `ph2d-tool-painter`
`o_dente_enche` `curas_da_mistura` `mistura_opaca` `o_veu_a_meia_definicao_fica_perto_da_inteira`
`a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` `o_wet_wash_mix_a_velocidade_do_dono_vai_pela_meia_definicao`
`o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela` `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao` e os dois
alarmes de pincel; `ph2d-panel-painter-layers` `a_mistura_aparece_no_digital_e_o_clique_chega`) — tudo verde em 10/11. Na
integração, suíte inteira (código movido entre ficheiros, deps). Binário smoke compilado no fim (2×).
Inegociáveis: os do CLAUDE.md.
