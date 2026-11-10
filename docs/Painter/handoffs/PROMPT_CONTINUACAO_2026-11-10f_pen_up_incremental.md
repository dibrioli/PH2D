/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-10f_pen_up_incremental.md

Estado (10/11): linha `line/PainterWatercolor`, base `main` `f254e42bf1`, HEAD = o commit docs que traz este prompt, sobre `569be991bd`, por integrar. O anterior:
`PROMPT_CONTINUACAO_2026-11-10e_pen_up_4k.md` (o que ele lista como FEITO continua feito; o seu ABERTO do travamento está curado).

FEITO na janela 10e/f:
- ⭐ **Pen-up a 4K curado** (Bug #85, `569be991bd`): `apply_impasto_light` por faixas de 32 linhas no pool, moldura de 1 px,
  ao bit. Quadro do pen-up 1 650–2 070 ms → **108–130 ms** no app (Digital, Water, Impasto, Wet); `sonda_luz_4k` 1 776,5 → 153,9 ms.
  **Dono: «ótimo».** Gate `as_faixas_dao_o_passe_por_pixel_ao_bit` (2 mutações vermelhas). Instrumentos:
  `PH2D_QUADRO_LENTO_MS`, `PH2D_COMPOSITE_TELA=<w>x<h>`, `ferramentas/mede_pen_up_4k.{sh,py}` (fixture de sonda de `fde1d09969`;
  as sondas antigas mediam tela SEM relevo de papel — 1,6–7 ms —, por isso diziam «não é»). Método: `Pinceis/COMO_ACELERAR_UM_PINCEL.md` §6.
- ⭐ **Auditoria de 2 lentes** do diff `f254e42bf1..fde1d09969`, 3 achados com gate red-first (Bug #85, bloco final):
  `95685615e1` o dente do papel na meia definição do Wet Wash Mix (Δ médio 2,239 → 0,338; só ladrilho preso à tela arma a meia);
  `be24b0a239` `follow_live` com `read()` a `None` (releitura a cada quadro, selo preso) → `SheetCache::falhou`;
  `7d4c1caa2b` `CardCtx::pair`/`compact` com Choice sozinha cortava → vai pela `row`.

ABERTO, em ordem:
1. ⭐ **Leitura INCREMENTAL de `imagem_aplicada`** (o que resta do pen-up, ~120 ms: a tela inteira refeita a cada traço; o perfil
   espalha-se por `texture::sample`, `height_at`, `material_at`, `Rig::shade`, blend, encode). Desenho (do explorador, por leitura —
   confira no código): hoje NÃO existe área suja consultável «desde R». `content_revision()` = (`layers_revision`, `pixel_clock`)
   (`runtime.rs:642`) diz SE mudou, não ONDE; `mark_dirty` (`stamp_preview.rs:38`) acumula `dirty_rect`, mas ele é CONSUMIDO pelas duas
   pistas (CPU `compoe_a_saida`; GPU `layers/preview.rs:~120`, que também faz `composited = None`). Em `PainterTool`, ao lado de
   `composited` (`tool/mod.rs:209`): dois buffers — composto NÃO aceso e aceso (a luz não é idempotente) — e um acumulador próprio de
   área suja, alimentado por `mark_dirty` e zerado só pela leitura; na leitura, recompor a suja (`composite_region` `compose.rs:30` /
   `composite_region_sobre_o_papel` `vidro.rs:120` / `compoe_sobre_o_papel` `papel.rs:236`) e re-acender a suja dilatada de 1 px.
   **Invalidação TOTAL** em: `invalidate_composite` (`layers/cache.rs`: visibilidade/opacidade/blend/ordem/camadas/`impasto_depth`),
   `set_source` (documento/resize), undo total (`layers/undo.rs:191-213`; o confinado `restore_model_confined` vai por `mark_dirty`),
   `reconcile_substrate` (`substrate_relief.rs:361` — só corre nas drenagens!), ajustes (`layers/adjustments.rs`), vidros/máscaras.
   A LUZ (`impasto_rig`, `impasto_show`, lâmpadas) não entra em `content_revision` → chave própria (rig + `SubstrateKey`).
   **Gate obrigatório:** incremental == inteiro ao bit depois de CADA tipo de mudança acima. Medir com `mede_pen_up_4k.sh` antes e
   depois (4K × 4 painéis). Alternativa a medir primeiro (mais barata): cache do plano do dente do papel (estático por `SubstrateKey`
   + tamanho).
2. A **proposta aceite e não ordenada** (dono 10/11): medir TODOS os pincéis à velocidade dele (`sonda_custo_no_app`, 9 × 40 px) e
   oferecer a meia definição aos lentos largos e macios, lado a lado, pincel a pincel (`Pinceis/COMO_ACELERAR_UM_PINCEL.md` §5).
3. Nota: a barra da cena a 4K mesmo parada tem frame p50 ~18–30 ms (não é o pen-up; meça se for pedido).

Herdados — smoke do dono PENDENTE: a prévia do pincel (Bug #83); o cartão Paint Mixing; «Save as default» (`6c0c7a59ca`); o Wet Wash
Mix em papel com relevo (`95685615e1`; ele não entendeu o roteiro — reescreva em passos); o seletor Color mixing em coluna estreita
(`7d4c1caa2b`).

Fecho: a bateria por nome do 10e (`ph2d-painter-brush` `wet_blend` `texture::` `dab::` `mistura_opaca`; `ph2d-tool-painter`
`o_dente_enche` `curas_da_mistura` `mistura_opaca` `o_veu_a_meia_definicao_fica_perto_da_inteira`
`a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` `o_wet_wash_mix_a_velocidade_do_dono_vai_pela_meia_definicao`
`o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela` `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao` e os dois alarmes de
pincel; `ph2d-panel-painter-layers` `a_mistura_aparece_no_digital_e_o_clique_chega`) MAIS os gates novos:
`as_faixas_dao_o_passe_por_pixel_ao_bit` (`ph2d-tool-painter`), `a_meia_definicao_leva_o_dente_do_papel_a_escala_da_tela` e
`o_grao_que_segue_o_pincel_vai_a_definicao_inteira` (`ph2d-tool-painter`), `a_leitura_vazia_nao_se_repete_nem_prende_o_selo`
(`ph2d-anim-sheets`), `a_choice_alone_in_a_pair_or_compact_row_still_wraps` (`ph2d-editor-core`). Na integração, suíte inteira.
Binário smoke compilado no fim (2×). Responder ao dono SEMPRE em PT-BR, curto, smoke em passos numerados.
Inegociáveis: os do CLAUDE.md.
