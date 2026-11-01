# E10-B2 — o painel: as linhas visíveis dos controlos de lei (01/11)

Especificação: `F3_controlos.md` §8 e §9; lado da ferramenta: `E10b1_ferramenta_e_presets.md`. Padrão do Inspector
(`card_row`: rótulo + número com setas), secções de hoje; nenhuma secção removida ou movida; Tilt e Spacing
«Continuous» intocados; nenhum knob vivo do avançado saiu.

## Linhas

| sítio | antes | depois |
|---|---|---|
| volumes do topo, `crates/ph2d-panel-painter-layers/src/paint_wetpaint_presets.rs` `paint_volumes` (:76) | Pincel: Size · Water · Opacity (0..1) | Pincel (`DepositoMolhado`): Size · **Dilution** (0..1, passo 0,001, 3 casas, sem chip rápido) · **Volume (µm)** (0..1000, passo 0,01, 2 casas, chip rápido em µm). Ferramenta Wet (`AguaDoVolume` sem `DepositoMolhado`): Size · Water, como antes |
| helper `linha_e10` (:153) | — | valor, faixa e passo pela ferramenta (`WetKnobs::linha_e10`, da `KNOB_DEFS`); `vol = None` desenha o campo sem chip à mesma largura |
| popover rápido (:445) | volume 2 em «%» | volume 2 em «{n} µm» (`quick_um`) |
| cartão Wet Paint, `paint_wetpaint.rs` | 1.ª linha **Pigment** (0..2000) | **Medium** (0..1) no lugar dele; **Hardness** (0..1) logo depois do Pickup; **Body** (0..1) logo depois do Leveling. Tabela `E10_NO_CARTAO` (:54) + `linhas_e10_depois` (:74), chamada em :158 (abre o cartão), :213 (linhas curadas) e :251 (knobs do cartão). Só no Pincel (`DepositoMolhado`, a mesma guarda das vizinhas). `DO_DEPOSITO` passou a const do módulo (:37) — o corpo da secção ficou abaixo das 200 LOC |
| cartão comum **Mixing**, `paint_brush_sections.rs` (:128) | nunca no Wet Paint | no Wet Paint com a lei `Pigmento` e a ferramenta que move tinta pela lei da sim (`MisturaDoPigmento`: Pincel e Blow) |

`PIGMENT_MAX`/`PIGMENT_STEP` saíram de `paint_wetpaint.rs`.

## Lado da ferramenta (mínimo, para o painel)

- `tp/media.rs` `offers_pigment_mixing_in(mode, eraser, lei)`: o Wet Paint oferece a fileira só com
  `LeiDaTinta::Pigmento`, `PaintMode::WetPaint` e sem borracha; os outros meios como antes.
- `tp/snapshot.rs` (:459): passa `self.paint.wetpaint.lei_da_tinta` (uma linha; ficheiro do agente C — aviso ao
  coordenador).
- `tp/wetpaint_settings/controlos_e10.rs`: `WetKnobs::linha_e10(id) -> Option<(valor, mín, máx, passo)>` (Hardness
  0..1 no passo do Body, por ser vista sem knob próprio).
- `crates/ph2d-tool-painter/src/ids/painter_wetpaint.rs`: `PAINTER_WETPAINT_PIGMENT` fora de
  `PAINTER_WETPAINT_FIELDS` (**41 → 40**, contado). A const fica: `sonda_varredura_oleo.rs:274` (agente C) ainda a usa.
- `ids/painter_wetpaint_presets.rs`: `VOL_QUICK_UM = [0.1, 0.3, 1, 3, 10, 30, 100, 300, 1000]` µm,
  `vol_quick_values(2)` devolve-a (F3b §4: impasto real 100–1000 µm; o Wet Round de fábrica dá 0,31 µm).
- `WetKnobs::pigment()` **fica**: `tp/wetpaint/tests.rs:1122` ainda o lê (B1).

## Chaves i18n

`crates/ph2d-i18n/src/painter_layers.rs`: `wetpaint.dilution` «Dilution», `wetpaint.volume_um` «Volume (µm)»,
`wetpaint.quick_um` «{n} µm», `wetpaint.medium` «Medium», `wetpaint.body` «Body», `wetpaint.hardness` «Hardness»;
saíram `wetpaint.vol_opacity` e `wetpaint.pigment` (sem uso). Dicas em `painter_wet_cartao.rs`: `medium_tip`,
`dilution_tip`, `volume_tip`, `body_tip`, `hardness_tip` (os textos do briefing), ligadas em `populate_wetpaint.rs`.
`seccoes.rs`: BRUSH ganha `dilution`/`volume_um` (perde `vol_opacity`); WETPAINT_CARTAO ganha `medium`, `hardness`,
`body` (perde `pigment`).

⚠️ Hardness: a dica segue a conta (`paperGate = (1 − H)·1,5`; `deposit = stamp − (1 − tooth)·gate`): **0 = só os
picos, 1 = enche os vales**. A sugestão do B1 («hard tips only touch the peaks») dizia o contrário — conferido no
`trail/deposit.rs:195`.

## Testes (por nome, `--profile smoke`)

| teste | resultado |
|---|---|
| `o_pigment_mix_do_wet_paint_so_aparece_na_lei_pigmento` (novo, `tp/media/pigment_mix_lei_tests.rs`) | verde: Velha → escondido; Pigmento → à vista; borracha → escondido; de volta à Velha → some |
| `seam_wetpaint::the_knob_rows_are_offered_only_while_armed` | vermelho → verde: o Water saiu do Pincel (filtro com motivo) |
| `seam_wet_presets::os_valores_rapidos_mudam_o_volume` | vermelho → verde: prova que o chip do Water não aparece no Pincel e que a Dilution sim; o rápido do Water corre com a ferramenta Wet |
| `seam_wet_presets::a_opacity_e_um_volume_do_wet_paint` | reescrito para o Volume µm (campo 50 µm, rápido 1000 µm); verde |
| `o_painel_mostra_o_que_a_ferramenta_le::cada_ferramenta_da_agua_so_ve_o_que_le` | vermelho → verde: a Hardness escondida mudava **3048 texels** no Blend da água (o Blend lê o `paperGate`, `trail.rs:603`) → a Hardness fica à vista no Pincel **e no Blend**, e sai da lista `DEPOSITO_MOLHADO` do teste (20 → 22, contado: −Pigment, +Dilution, +Medium, +Body) |
| `seam_wetpaint*` (todos), `seam_wet_presets*`, `seam_pigmento*`, `cada_nome_deste_painel_cabe_na_coluna_da_seccao*`, `hr12_widgets_a11y` | verdes (60 testes na corrida) |
| `ph2d-i18n` `a_tabela_inglesa_fala_ingles` | verde (4) |
| `o_censo_so_encolhe::o_censo_dos_controlos_so_encolhe` | **vermelho, não é deste lote**: `REWET`, `REWET_DRAG`, `SOLUBILITY` «mudam o ajuste e NÃO mudam a tinta» no Pincel do Wet Paint. A visibilidade destas três não mudou aqui (só a lista `DO_DEPOSITO` virou const, mesmo conteúdo); suspeita: o motor em obra (A1/A2/C — receita/depósito). Por investigar com quem possui o motor |
| `censo_ferramenta_x_linha`, `censo_dos_controlos_tabela` | casados pelo filtro da 1.ª corrida (`censo_dos_controlos`, `censo_ferramenta_x_linha`): sem falha própria além do `o_censo_dos_controlos_so_encolhe` |

## Foto

Há sonda: `crates/ph2d-panel-painter-layers/src/foto_wetpaint_tests.rs` (`#[ignore]`, padrão da `foto_jitter`):
`PH2D_FOTO_DIR=docs/Painter/Arquitetura/fotos/e10 cargo test -p ph2d-panel-painter-layers --profile smoke foto_wetpaint
-- --ignored` grava `painel_wet_paint.png` (volumes do topo + cartão Wet Paint, Pincel de fábrica). **Não corrida**
nesta janela: precisa da placa e o prazo do agente acabou — fica para o coordenador (a VEZ da placa).

## Em aberto

- **Chip rápido da Dilution:** o slot 1 do rápido escreve o Water na ferramenta (`wetpaint_preset.rs`
  `aplicar_volume`); a Dilution ficou sem chip para não haver chip morto. Um slot rápido próprio é do lado da
  ferramenta.
- `PAINTER_WETPAINT_PIGMENT` e `WetKnobs::pigment()` saem quando a sonda e o teste do B1 deixarem de os ler.
- A **borracha** da água também lê o `paperGate` (`wp/tools.rs:106`) e a Hardness não lhe aparece: não medido (o
  censo da água não a gesticula). Se medir > 0, juntar `brush.eraser` à guarda de `linhas_e10_depois`.
- O `o_censo_dos_controlos_so_encolhe` vermelho (acima).
- `paint_wetpaint.rs` está em 502 linhas (perto do tecto de 500 dos painéis, se valer para ele).
