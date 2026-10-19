# HANDOFF DE INTEGRAÇÃO — `line/PainterWatercolor`, prompt 51: Spacing 0,1 + Continuous em todos os pincéis (2026-10-19)

> Leitor: o integrador e a próxima janela da linha. Mecanismo e tabelas: [51_medicao](../NewFeaturesWetPaint/51_medicao_spacing_continuous.md)
> (não repetidos aqui) · [prompt 51](../NewFeaturesWetPaint/51_PROMPT_spacing_continuous_em_todos_os_pinceis.md) · anterior:
> [integração 17/10](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-17.md).
> Smoke do dono: **PENDENTE** (integrar não é aprovar).

## §1 — Coordenadas

- Branch `line/PainterWatercolor`, rebaseada nesta sessão sobre o `main`; merge-base `100d34c8e0`; zero conflitos. **Não enviada.**
- 70 commits sobre o main (P44–P51). Desta sessão: `ee96ac72ad` (WIP 3) · `8a71657269` (WIP 4) · `aa5d1ac634` (fmt/clippy/régua)
  · `c7e6dd942c` (os três do fecho) · o commit de docs final. Antes: P44 [formas](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P44_FORMAS.md)
  · P45 [esmaecer](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P45_ESMAECER.md) · P46 [caneta](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P46_CANETA.md)
  · P47 [W6](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P47_W6.md) · P48 [fios](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P48_W2_FIOS.md)
  · P49 [relevo](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P49_W3_RELEVO.md) · P50 [sem escada](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P50_W3b_SEM_ESCADA.md)
  · P51 WIP 1–2 [WIP](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-19_P51_WIP.md) (`2146515ce9`, `9393b7ba71`).

## §2 — O que entrou nesta sessão (a lei final: [51 §1](../NewFeaturesWetPaint/51_medicao_spacing_continuous.md))

- A janela do rasto volta à CORDA entre centros (`direct_segment`) — REVERTE o WIP §2.4 (sub-passo natural).
- Sub-passo arredondado para BAIXO (`capsula::sub_passo_sem_batimento`); a lei antiga vive em `sub_passo_mais_perto`, escolhida
  pelo knob oculto `capsuleRoundNearest` (`KNOB_COUNT` **89**) em Graphite 8B, Charcoal, Soft Pastel, Oil Pastel e Airbrush.
- Jitter de posição por sub-dab (`capsula::Salto`, em `Ante.salto`): só a rota `sub_dabs` do Wet Paint.
- Airbrush `pressureSizeCurve` 0,7 por ORDEM DO DONO (19/10, «afinar para o Rebelle», depois de ver que as pontas dele nunca
  passavam com régua justa) — [51 §5](../NewFeaturesWetPaint/51_medicao_spacing_continuous.md).
- Régua da mídia de pó = MEDIANA de 7 tiragens (`PainterTool::set_semente_dos_tracos`, porta INTERNA), também na rampa.
- `wetpaint/lote.rs` cortado de `dab_route.rs` (792 → 698 LOC, tecto 700): puro movimento.
- Censo «pega do ÂNGULO do carimbo» (WetPaint) REMOVIDO dos inertes: agora MUDA a tinta (95 px, máx. 7 níveis). Bissectado:
  só era inerte com o default pré-51 (0,025) sem Continuous; a ponta de cerdas segue a pegada rodada.

## §3 — O que um merge pode partir

- **Repinados de propósito** (a lei fechada): `a_lei_da_tinta` CENAS (22) · `opcoes_tests` ESPERADO (6) · `shape_color_tests`
  HOJE (6; cenas 1 e 2 COINCIDEM, pela lei) · `o_po_do_aerografo::sem_po` ANTES (1). Se a outra ponta repinar o mesmo, o
  número é CONTADO na árvore combinada, nunca escolhido.
- **20 presets regenerados** (`assets/brushes/WetPainter/**.ph2d-brush.png`, binário). Em conflito: REGENERAR, nunca fundir à mão:
  `PH2D_REGERA_PRESETS=1 cargo test -p ph2d-tool-painter --profile smoke --test it gera_os_presets_de_fabrica -- --ignored --nocapture`
  (`PH2D_REGERA_SO="a,b"` regera só alguns). O gerador imprime as 8 linhas de `LARGURA_DO_WET_ROUND`
  (`wetpaint/largura.rs`) — colar.
- Contadores como DELTA (esta sessão): `KNOB_COUNT` 89 no fim da linha (era 56 na integração de 17/10; +1 só nesta sessão: `capsuleRoundNearest`); sem
  `PROJECT_SCHEMA`, sem contrato §6, sem ADR, sem `Cargo.lock`. Quem também somar knobs em `tuning.rs` conta de novo.
- Censo da regra: `presets_de_fabrica::a_regra_do_spacing_da_fabrica` (os 20 + o pincel sem preset).

## §4 — Prova de fecho

- Suíte de fecho `nextest-impacted` (`PH2D_SUITE=fecho`, BASE = merge-base) em `aa5d1ac634`: 19 340 testes, 19 336 verdes,
  3 vermelhos + 1 TIMEOUT. Os 3 curados em `c7e6dd942c` e reverificados POR NOME, verdes: `macro_portas`,
  `architecture_workspace_file_loc_cap`, `o_censo_so_encolhe` (+ `a_lei_da_tinta::sem_glaze`, `o_relevo_nao_tem_escada`,
  `o_pigmento_por_comprimento`).
- TIMEOUT `o_pigmento_por_comprimento::a_tinta_nao_muda_com_o_spacing`: carga (180 s a `load` 21; sozinho 69 s e 53 s, verde).
  Candidato a [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) (duas escritas: tabela + faixa) — NÃO promovido; promova se repetir.
- `fmt` ✓ · `clippy -D warnings --all-targets` ✓ (painter-brush, wet-paint, brush-presets, tool-painter, panel-painter-layers).
- ⚠️ A suíte NÃO correu de novo depois de `c7e6dd942c` (cadência): é movimento puro + 2 edições de lista, hashes de identidade
  intactos. O gate da árvore combinada do integrador cobre.
- Custo (release, `load` ~4, [51 §6](../NewFeaturesWetPaint/51_medicao_spacing_continuous.md)): os 20 ≤ 11,8 ms/quadro (orçamento
  16,7). Oil Bristle a 103 px 14,5 → 7,3 (P50: 13,7). Alguns finos ficam mais lentos que antes, dentro do orçamento (Wet Round
  40 px 7,1 → 11,8). Mutação medida: [51 §7](../NewFeaturesWetPaint/51_medicao_spacing_continuous.md).

## §5 — Smoke do Enio (release, porque o smoke é de velocidade; binário JÁ compilado nesta worktree)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_WETPAINT_SMOKE=2 ./target/release/ph2d-host-desktop`
   (2.ª compilação, saída colada): `▸ linha line_painterwatercolor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s` /
   `Finished \`release\` profile [optimized] target(s) in 0.32s`.
2. O Painter abre sozinho em Wet Paint com o Oil Bristle a 103 px e dois traços já pintados (o roteiro que o app
   imprime). Risque traços RÁPIDOS no espaço livre; troque de pincel (Graphite 8B, Marker Bullet, Airbrush) e repita.
3. Deve: traço rápido sem pesar (≈ 59 fps ao abrir), sem escada nas cerdas, a tinta igual ao mexer no Spacing.
4. Deu errado se: o traço arrasta atrás da mão, aparecem degraus regulares nas cerdas, ou o Airbrush perde o pó.
   Foto: [`51_smoke_cena.png`](../NewFeaturesWetPaint/51_smoke_cena.png) (+ [`.log`](../NewFeaturesWetPaint/51_smoke_cena.log));
   prancha `../NewFeaturesWetPaint/51_prancha_antes_agora.png`.

## §5b — O Brush dust no cartão Wet Paint (dono 19/10, após o smoke OK)

O knob `Dust` saiu do Tuning (`KnobGroup::Hidden`) para o cartão Wet Paint do Inspector, abaixo do Brake, pelo
caminho do Brake: id `PAINTER_WETPAINT_DUST` em `PAINTER_WETPAINT_FIELDS` (18), `KNOBS_NO_CARTAO` (ex-
`FISICA_NO_CARTAO`, 5) e `knobs_no_cartao()`, a linha em `paint_wetpaint.rs`/`seccoes.rs`, i18n
`panel.painter_layers.wetpaint.dust` = «Brush dust». Tuning: 44 linhas (Paint 25). Gates: `seam_wetpaint::
o_po_do_pincel_edita_se_pelo_cartao` (literal; mutação «o id fora de FIELDS» SANGRA: «morreu no painel»),
`a_fisica_da_agua_do_cartao_chega_ao_motor_e_muda_a_tinta` (+ Dust 0,8), censos, colunas, colisões, i18n verdes.
Os presets gravam-no como antes (`engine["dust"]`). Binário release refeito
(a 2.ª compilação em 0,25 s). Smoke: Inspector › Painter › Wet Paint.

## §6 — ABERTO (próxima janela)

- Relógio por PARTE do traço rápido em calma: só há razões a `load` 22 (sem relevo −19 %, sem cor das cerdas −16 %) —
  [`traco_rapido_por_partes_load22_2026-10-19.txt`](../NewFeaturesWetPaint/51_regua/traco_rapido_por_partes_load22_2026-10-19.txt).
- O carimbo ainda corre no tratador do ponteiro, sem orçamento de quadro.
- O pó é por dab: um «pó por comprimento» libertaria o Airbrush do `capsuleRoundNearest`.
- `Dab::capsula` (a cápsula das rotas de COR) ainda usa o jitter RÍGIDO; só a rota `sub_dabs` do Wet Paint ganhou `Salto`.
- Promover o TIMEOUT de §4 a FLAKES_DE_CARGA se repetir. Resto: [51 §8](../NewFeaturesWetPaint/51_medicao_spacing_continuous.md)
  e [26](../NewFeaturesWetPaint/26_ABERTOS.md).
