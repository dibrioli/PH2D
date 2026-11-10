# Handoff de integração — line/PainterWatercolor, pen-up 4K e mistura (10/11)

Leitor: o agente integrador. O mecanismo de cada wave está no doc da wave (links no fim); aqui só o que evita colisão e
regressão. Nada integrado nem enviado.

## 1. Identidade

- Branch `line/PainterWatercolor`, merge-base com o main `f254e42bf1`, 55 commits (09/11 a 10/11), 194 ficheiros.
- HEAD do código `07873cfdcf`; **o commit deste handoff vem por cima** (só docs + a linha do Painter no `CLAUDE.md` §5.1).
- Anterior: [mistura e papel, 08/11](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-08_MISTURA_E_PAPEL.md) (já no main). Prompt do estado
  atual: [10f](PROMPT_CONTINUACAO_2026-11-10f_pen_up_incremental.md).
- Tocado, por crate (linhas + / −): `ph2d-tool-painter` 2127/521 (41 ficheiros) · `ph2d-painter-brush` 1531/355 (17) ·
  `ph2d-panel-painter-layers` 433/115 (19) · `ph2d-app-painter` 480/519 (7) · `ph2d-brush-presets` 22/2 (3) · 14 PNG de miniatura
  em `assets/brushes` · `docs/Painter` (71) · resto = superfície partilhada, §2.

## 2. Superfície de colisão (`bash scripts/collision-surface.sh`, 10/11)

```
merge-base f254e42bf1 · 55 commit(s) · 194 arquivo(s)
PROJECT_SCHEMA 225 (base 225) · tripla (225, 13, 29) (base igual) · VEC 29 · FLIP 13 · DOC_VERSION 25 — todos IGUAIS ao base
ph2d-ecs 119 (base 119) · ✗ SONDA CEGA ph2d-render/ph2d-script = defeito conhecido do SCRIPT, a linha não toca componentes
Contrato congelado §6: node.rs intocado · tool.rs intocado
ADR: último no disco 0200 · esta linha CRIA o ADR 0200 — reconte contra o main do dia
Cargo.lock: nenhum '+name' novo · marcadores de conflito: nenhum
TETOS DE LOC: ✗ 735/700 ph2d-painter-brush/src/dab.rs · ✗ 710/700 ph2d-tool-painter/src/tool/paint/traco_plano_tests.rs
```

⛔ **VERMELHO NO FECHO (achado ao escrever este handoff, 10/11):** `architecture_workspace_file_loc_cap::workspace_src_files_under_loc_cap`
(`ph2d-editor-core`) REPROVA com exactamente esses dois ficheiros (base: `dab.rs` 643, `traco_plano_tests.rs` 658). Os «132/133 censos»
da árvore combinada não o apanharam. Cura = CORTAR por responsabilidade (nunca subir o tecto): fica para a linha antes de integrar —
o integrador não deve receber a linha com este gate vermelho. Corra-o por nome:
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/cargo-test-narrow.sh ph2d-editor-core architecture_workspace_file_loc_cap`.

Ficheiros PARTILHADOS fora do Painter que a linha mexeu (todos aditivos salvo a nota):

| Onde | O quê | Risco |
|---|---|---|
| `ph2d-editor-core/src/panel/cards_compacta.rs` (+ `_tests`) | `CardCtx::pair`/`compact`: a célula SOZINHA que é Choice vai pela `row` (cortava quando não cabia) — `7d4c1caa2b` | widget partilhado: muda só a Choice sozinha que não cabia; varredura por consumidor verde (editor-core `compacta` 22, panel-painter-layers, panel-vector, panel-widget-gallery); gate `a_choice_alone_in_a_pair_or_compact_row_still_wraps` |
| `ph2d-editor-core/src/widget/spinner.rs`, `widget/mod.rs` | peça do selo: `ProgressPhase`/`paint_progress_badge` ao lado do `Spinner` (+156 linhas, re-export) | aditivo; `widget/*` tem tecto 500 |
| `ph2d-editor-core/src/screens/hero/inspector_model_anim_card.rs` | 4 linhas do selo no cartão Animation | aditivo |
| `ph2d-anim-sheets/src/sheet_cache.rs` (+ `_tests`) | `follow_live` ganha o parâmetro `agora_ns` (relógio do selo); `QUADROS_PARADOS = 30`, `QUADROS_A_GIRAR = 10`, `VIDA_DO_CHECK_NS`; `SheetCache::falhou` (leitura vazia) | **assinatura mudou**: o único chamador é `fase_snapshots_publish.rs` (abaixo); outra linha que chame `follow_live` quebra |
| `shells/desktop/src/render_loop/fase_snapshots_publish.rs` | +2 linhas: `let agora = hero.store.clock_ns();` e o argumento novo | colide só se outra linha mexer nessa chamada |
| `ph2d-panel-inspector` (`cards_anim.rs`, `cards_sprite_folhas.rs`, `ids/menus.rs`, `sections/anim.rs`, `sections/anim_card.rs`, `tests/it/as_folhas_do_animated.rs`) | o selo no cartão Animation | aditivo |
| `ph2d-app-sprite-anim/src/card.rs` | +3 linhas do selo | aditivo |
| `ph2d-i18n` (`anim_card.rs`, `painter_layers.rs`, `painter_ponta.rs`) | chaves novas: dica do selo, Paint Mixing, Grain softness / Tip contact / Tooth fill | aditivo (en-only) |
| `ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs` | `painter_layers` 1694 → 1718 → **1838 px** (cartão Paint Mixing nasce aberto), **ORDEM DO DONO 10/11** (`640ed27ae5`); o tecto soma com outras linhas que mexam em `painter_layers` | reconferir |
| `ph2d-brush-presets/Cargo.toml` | `serde_json` com `features = ["float_roundtrip"]` (`6c0c7a59ca`) | **deps/features unificam no workspace** ⇒ suíte inteira; `Cargo.lock` sem nome novo |
| `ph2d-painter-brush/Cargo.toml` | só comentário: usos 4 e 5 do `rayon` (ADR-0200) | nenhum |
| `docs/architecture/decisions/0200-o-carimbo-de-mistura-e-o-kernel-em-faixas-no-pool-do-rayon.md` + `README.md` | **ADR-0200 NOVO**. O main tem até 0199 hoje; se outra linha tomar 0200 antes, renumere este e os links (`BUGS_painter.md` #84, `COMO_ACELERAR_UM_PINCEL.md`, comentários) | **recontar** |
| `project-memory/reference_topic_owner_product_decisions.md` | +1 linha (dono 09/11, Pastel K1 + enchimento do dente) | append |

A memória `project_wet_wash_mix_half_resolution_approved_2026_11_10.md` foi escrita no `project-memory/` do PRIMÁRIO (por rastrear
lá), não nesta worktree: o integrador confere se entrou no índice.

## 3. Números que se CONTAM (DELTA)

- **`PROJECT_SCHEMA`: DELTA 0** (225 = base; tripla `(225, 13, 29)` igual). O JSON dos presets ganhou campos (`meia_definicao` em
  `mistura_opaca`, `softness`, `tip_depth`, `fill`, `tip contact` da ponta) mas é o formato do PRESET (`.ph2d-brush.png`), regerado e
  com round-trip ao byte; nenhum `*_SCHEMA`/`*_VERSION` mudou no diff. Não há degrau a recontar.
- **ADR: +1** (0200, §2). **Registo de componentes: 0.** **Pacotes externos: 0.**
- **Tectos de LOC:** `ph2d-app-painter` 24 380 ≤ 24 429 (catraca `the_shell_only_shrinks`; foi 24 831 > 24 429 e curou-se em
  `07873cfdcf`, comentários históricos movidos para [`historia_comentarios_app_painter_2026-11-10.md`](../historia_comentarios_app_painter_2026-11-10.md);
  folga 49, que SOMA com outra linha que cresça o app-painter). Tecto do painel do pincel 1718 → 1838 (acima). Tecto de ficheiro: os dois
  vermelhos de §2. Os dois ficheiros partidos pela linha (`mistura_opaca/carimbo.rs`, `stamp_route/mistura/lote.rs`) já estão sob 700.

## 4. O que entra, por tema

| Tema | Commits | Nota |
|---|---|---|
| Selo da prévia no cartão Animation + folha ao vivo | `8b6b4a253a` `394a84dc09` `7029da236c` `0126220dfa` `4c8843e605` | gira enquanto a tinta anda, enche por `QUADROS_PARADOS`, vira check que some (1,2 s); a folha só se lê com a tinta parada. Smoke 09/11 do dono: **OK** |
| «×» da busca do Preset; sondas | `02341d1b0a` `aeefe44d21` `819702134f` | `PH2D_COMPOSITE_PAPEL`/`PILHA`/`RELEVO` e `mede_rato_xvfb.sh` só medem |
| Pastel K1 (softness, tip_depth) | `8d9aca8d7a` `3d8d15e0eb` | painel da ponta: «Grain softness», «Tip contact» (Flat · Rounded), só no Height |
| Tooth fill (o dente enche) | `956935388c` `b8114c427b` | `GrainLaw::fill`, `texture/po.rs`, `tool/paint/po_do_papel.rs`; custo 11 ms a 4K |
| Pastel com o grão do dono, Chalk/Crayon enchem | `288616b743` | Noise Tiled 120; presets regerados |
| Mistura causal (Bug #81) | `f0ef026f0e` `b8e515936f` | a cor de baixo aparecia ~1,4 raio antes da borda; 14 miniaturas aprovadas |
| Finger Tip esgota no papel vazio (#82) | `01b3678a88` | 3 miniaturas |
| Prévia/menu de presets (#83) | `16e4206ce2` | menu nomeia o pincel na mão (gaveta × slot) |
| Cartão Paint Mixing | `2df0c78389` `bed0cc5f64` `b4998ef0a7` `ea5f04878a` `640ed27ae5` | Color Mixing + Mixing num cartão, um Pigment só; tecto 1838 |
| «Save as new/default» f64 | `6c0c7a59ca` | `float_roundtrip`; teste `um_f64_calculado_volta_ao_bit_pelo_json` |
| Wet Wash Mix desempenho (#84), curas 1–6 | `a1ac3c6b58` `38985f2b15` `9f256bc878` `334d76f347` `5a02ad9c9d` `6537d5309a` `4bb5ffaca3` `017162eb22` `33e7ab4401` `fa35f92e0c` `75633921f8` `2a5b90d571` | ADR-0200; cura 6 = meia definição, **aprovada pelo dono 10/11**; miniaturas aprovadas |
| Auditoria 10/11 | `95685615e1` `be24b0a239` `7d4c1caa2b` | dente na meia definição · leitura vazia da folha · Choice no pair/compact |
| Pen-up 4K (#85) | `569be991bd` `fde1d09969` `48eb3e8504` | `apply_impasto_light` por faixas de 32 linhas no pool: 1,7 s → ~120 ms no app; dono: «ótimo» |
| Fecho | `4c9929be50` `07873cfdcf` | clippy (Faixa, `Amostrador` em Box, `as_chunks`); comentários históricos; a sonda `measure_o_quadro_do_pen_up_em_4k` foi RETIRADA |

## 5. Prova de fecho (10/11)

Nada de suíte completa nem `nextest-impacted` (dono 23/10). fmt ✓ nas 6 crates. `clippy -D warnings --all-targets` ✓ em
`ph2d-painter-brush`, `ph2d-tool-painter`, `ph2d-panel-painter-layers`, `ph2d-anim-sheets`, `ph2d-editor-core`, `ph2d-app-painter`
(depois de `4c9929be50`). Testes por nome, todos verdes:

- `ph2d-painter-brush`: `wet_blend` 12 · `texture::` 66 · `dab::` 26 · `mistura_opaca` 12.
- `ph2d-tool-painter`: `o_dente_enche` 2 · `curas_da_mistura` 11 · `mistura_opaca` 14 · `o_veu_a_meia_definicao_fica_perto_da_inteira` ·
  `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` · `o_wet_wash_mix_a_velocidade_do_dono_vai_pela_meia_definicao` ·
  `o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela` · `cada_campo_do_pincel_vai_para_o_preview_ou_diz_porque_nao` ·
  `as_faixas_dao_o_passe_por_pixel_ao_bit` · `a_meia_definicao_leva_o_dente_do_papel_a_escala_da_tela` ·
  `o_grao_que_segue_o_pincel_vai_a_definicao_inteira` · `substrate_relief` 26 · `impasto_light` 6 · `shape_relief` 3 · `use_as_relief` 8 ·
  `impasto_fingerprint` 3 · `watercolor_flow` 18. Alarmes de pincel `a_largura_de_fabrica_e_a_aprovada` e
  `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`: sem alarme.
- `ph2d-panel-painter-layers`: `a_mistura_aparece_no_digital_e_o_clique_chega` 1 · `color_mixing` 6 · `seam_wet_presets` 22.
- `ph2d-anim-sheets`: `sheet_cache` 4. `ph2d-editor-core`: `compacta` 22 · `the_shell_only_shrinks` 2.
- `ph2d-app-painter`: `painter_preview` 9 · `composite_smoke` 3 · `paint_perf` 9.
- Censos da árvore combinada 132/133; o único vermelho (`the_shell_only_shrinks`) curado por `07873cfdcf` e conferido por nome.
- **Mutação:** `as_faixas…` (2 vermelhas) · meia+dente e grão que segue o pincel (red-first) · leitura vazia (red-first) · Choice no pair (red-first).
- ⛔ **NÃO verde:** `architecture_workspace_file_loc_cap` (§2) — fora da bateria acima; achado depois.
- `agent-loop-profile`: paralelismo 1,09/passo ✗ (alvo ≥ 1,5) · test:check 2,4× ✗ · contexto 225 mil/passo ✓ · edições sem conferir 0,8 % ✓.
- **NA INTEGRAÇÃO, suíte inteira:** código movido entre ficheiros, `float_roundtrip` (deps/features) e o formato gravado dos presets.

## 6. Smokes do dono

Binário: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && ./target/smoke/ph2d-host-desktop` (depois da integração, o do main).

**Feitos (dono, 10/11 salvo nota):** pen-up 4K «ótimo» · meia definição do Wet Wash Mix «muito melhor e aceitável» e aprovada ·
miniaturas dos 14 pincéis de mistura e do Wet Wash Mix (`b8e515936f`, `2a5b90d571`) · Pastel/Chalk/Crayon: o dono fez a triagem das
receitas («P_dono SIM, Ch/Cr H10», `288616b743`) — o smoke do resultado final desses presets regerados não está registado nos prompts ·
selo da prévia OK (09/11, `4c8843e605`) · enchimento do dente OK (10/11).

**PENDENTES (integrar não é aprovar):**
1. Prévia do pincel / menu de presets (#83): com as duas decisões tomadas SEM ele (Sync ⇒ menu sem nome; sem Sync, voltar reaplica o preset e ajustes fora de tamanho/opacidade/modo perdem-se).
2. Cartão Paint Mixing (Pigment + Color mixing, tecto 1838 já ordenado por ele; o resto do cartão não foi visto).
3. «Save as default» do Wet Wash (`6c0c7a59ca`).
4. Wet Wash Mix em papel com relevo (`95685615e1`): ele não entendeu o roteiro — reescrever em passos.
5. Seletor Color mixing em coluna estreita (`7d4c1caa2b`).

## 7. ABERTO depois da integração

1. ⭐ **Leitura incremental de `imagem_aplicada`** (os ~120 ms que sobram do pen-up a 4K). Desenho e invalidações TOTAIS listadas no [10f](PROMPT_CONTINUACAO_2026-11-10f_pen_up_incremental.md); gate obrigatório «incremental == inteiro ao bit depois de cada tipo de mudança». Alternativa mais barata a medir antes: cache do plano do dente do papel por `SubstrateKey` + tamanho.
2. A **proposta aceite e não ordenada**: medir todos os pincéis à velocidade do dono (`sonda_custo_no_app`, 9 × 40 px) e oferecer a meia definição aos lentos largos e macios, lado a lado ([COMO_ACELERAR_UM_PINCEL.md](../Pinceis/COMO_ACELERAR_UM_PINCEL.md) §5).
3. Barra da cena a 4K parada com frame p50 ~18–30 ms (não é o pen-up; só se pedido).
4. Herdados: Tooth fill não age na ponta de píxel (L18), no Shape de camadas coloridas e na máscara de stamp em cache (controlo morto: fechar ou esconder) · curva de pressão do K1 a p 0,5–0,65 abaixo da do Krita · traço pálido no branco · marca do Wet por camada · Splash · `SPRAY_FIXO_MAX` com Wet blend · painel a 245 px · dicas longas · papel de imagem sem dente com Grain · memória do 1.º traço · lacunas baixas da auditoria (Save as Default sem verificação da vista; Paint Mixing sem teste de troca de preset).

## 8. Riscos para o integrador

- `pair`/`compact` é widget partilhado: só a Choice sozinha que não cabia mudou; reconfira a varredura por consumidor se o main trouxe consumidor novo.
- `TextureSettings` cresceu (lei do grão): o `Amostrador` do `watercolor_flow` pôs as settings em `Box` (`4c9929be50`) — outra linha que construa `Amostrador` à mão reconcilia.
- `follow_live` mudou de assinatura (§2).
- `PH2D_QUADRO_LENTO_MS` e `PH2D_COMPOSITE_TELA` são só medição (sem efeito no produto).
- O gate de custo `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` (razão ≤ 1,3) mediu 1,26–1,29 sob carga: candidato a flake ([FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md)).
- Miniaturas e pincéis: só o dono regrava; alarme vermelho na integração = «estes pincéis mudaram», relatório em `target/relatorio-pinceis/index.html`.

## 9. Onde está o mecanismo

[BUGS_painter.md](../BUGS_painter.md) #81–#85 e o bloco «Auditoria de 2 lentes, 10/11» · [COMO_ACELERAR_UM_PINCEL.md](../Pinceis/COMO_ACELERAR_UM_PINCEL.md) ·
[ADR-0200](../../architecture/decisions/0200-o-carimbo-de-mistura-e-o-kernel-em-faixas-no-pool-do-rayon.md) · prompts
[pastel e mistura](PROMPT_CONTINUACAO_2026-11-10_pastel_grao_e_mistura.md) · [10b](PROMPT_CONTINUACAO_2026-11-10b_mistura_previa_secos.md) ·
[10c](PROMPT_CONTINUACAO_2026-11-10c_wet_wash_mix_desempenho.md) · [10d](PROMPT_CONTINUACAO_2026-11-10d_veu_meia_definicao.md) ·
[10e](PROMPT_CONTINUACAO_2026-11-10e_pen_up_4k.md) · [10f](PROMPT_CONTINUACAO_2026-11-10f_pen_up_incremental.md).
