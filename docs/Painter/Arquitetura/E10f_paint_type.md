# E10f — Paint type (Water · Oil) no lugar do Medium contínuo

Decisão do dono (01/11): o `Knob::Medium` (0 água … 1 óleo) passa a ser um seletor **Paint type** com duas
opções, **Water** e **Oil**. Só esses dois pontos são testados e garantidos. Acrílico = Water com Re-wet `0`;
o Fan Brush (o único preset Acrylic) passa a Water. A UI e a camada da ferramenta estão aqui; a tabela do motor
(`ph2d_wet_paint::painter::receita::meio_da_familia`, Acrylic → 0) é do coordenador.

## O que mudou

| Onde | O quê |
|---|---|
| `crates/ph2d-tool-painter/src/ids/painter_wetpaint.rs:290-300` | `PAINTER_WETPAINT_MEDIUM` passa a ser o id do chip; novos `PAINTER_WETPAINT_MEDIUM_COUNT = 2` e `painter_wetpaint_medium_option_id(i)` |
| `…/ids/painter_wetpaint.rs:404` | `PAINTER_WETPAINT_MEDIUM` sai de `PAINTER_WETPAINT_FIELDS` (41 → 40): um menu não é um campo numérico |
| `crates/ph2d-tool-painter/src/tool/paint/wetpaint_settings/controlos_e10.rs:37` | `no_degrau(knob, v)`: Medium `> 0,5` → `1`, senão `0`; os outros knobs passam intactos |
| `…/wetpaint_settings.rs:230` | `WetKnobs::set` aplica `no_degrau`. É a ÚNICA porta de escrita (painel, preset, macro, estado gravado), por isso o degrau vale em todo o lado |
| `…/wetpaint_settings/controlos_e10.rs:49` | `route_wet_select`: `SelectOption(MEDIUM, "0"/"1")` → `set_wet_knob_value(MEDIUM, i)`; também leva o Environment do Real Oil (o braço saiu de `wetpaint_settings.rs:666`, que agora tem 696 linhas, abaixo do tecto de 700) |
| `…/wetpaint_settings/controlos_e10.rs:80` | `WetKnobs::e_oleo()` = `medium() > 0,5`: o que o seletor mostra |
| `…/wetpaint_settings/controlos_e10.rs` `linha_e10` | o Medium sai (deixou de ser linha numérica) |
| `crates/ph2d-tool-painter/src/tool/paint/wetpaint_preset/controlos_e10.rs:101-107` | `migra_para_controlos`: Medium pelo degrau; família `Acrylic` → `Knob::Rewet = 0`; o doc do wrapper `meio_da_familia` diz Water · Oil |
| `crates/ph2d-panel-painter-layers/src/paint_wetpaint/tipo_de_tinta.rs` (novo) | a linha `Paint type` (`paint_dropdown_row_de_caixa`) + o popover |
| `crates/ph2d-panel-painter-layers/src/paint_wetpaint.rs:51-158,403` | `E10_NO_CARTAO` 3 → 2 (o Medium sai); o seletor abre o cartão, só quando a ferramenta pousa tinta (`DepositoMolhado`), como antes |
| `…/paint_wetpaint/oleo.rs:182` | `paint_wetpaint_popovers` também pinta o popover do Paint type |
| `…/event/decode.rs:166`, `…/event/option_route.rs:13,164` | `decode_wet_medium_option` + a linha da tabela de rotas (35 → 36) |
| `…/populate_dropdowns.rs:44` | o chip regista-se como Dropdown |
| `…/populate_wetpaint.rs:104` | tooltip → `panel.painter_wet_cartao.paint_type_tip` |
| `…/seccoes.rs:432` | a chave do rótulo na coluna do cartão → `…wetpaint.paint_type` |
| `crates/ph2d-i18n/src/painter_layers.rs:591-593` | `Paint type`, `Water`, `Oil` (a chave órfã `…wetpaint.medium` foi removida) |
| `crates/ph2d-i18n/src/painter_wet_cartao.rs:66` | tooltip em inglês simples (a `medium_tip` órfã foi removida) |

## Testes

| Teste | Antes | Depois |
|---|---|---|
| `o_paint_type_escreve_agua_ou_oleo_e_o_meio_antigo_carrega_como_agua` (novo, `controlos_e10_tests.rs:245`) | vermelho (sem rota de `SelectOption`; `0,5` ficava `0,5`) | verde. Mutação `no_degrau` = identidade → vermelho em `SetValue 0.5` |
| `cada_preset_de_fabrica_mapeia_por_funcao` (Acrylic → 0, Re-wet 0) | — | verde (55 Wet Paint, 45 de tinta) |
| `um_controlo_do_pincel_nao_recompoe_a_tela` (medium 0,5 → 1,0 + `SelectOption` Oil) | — | verde |
| `o_pincel_sem_preset_e_o_wet_round_migrado` | — | verde |
| panel `cada_linha_visivel_age_na_ferramenta_ou_diz_porque_nao` (o menu Paint type age no Pincel) | — | verde (795 s, load 61→31) |
| panel `nenhuma_linha_escondida_mudaria_a_ferramenta` | — | verde (143 s) |
| panel `seam_wetpaint*` (31 testes, inclui `the_knob_rows_are_offered_only_while_armed` com a FIELDS de 40) | — | verde |
| panel `cada_nome_deste_painel_cabe_na_coluna_da_seccao` (6, o rótulo «Paint type») | — | verde |

O ficheiro `o_painel_mostra_o_que_a_ferramenta_le` corrido INTEIRO bateu no prazo de 900 s do `ph2d-run` com load
87 (contém `cada_ferramenta_da_agua_so_ve_o_que_le`, de 600 s, que não toca o Medium); por isso só os dois testes
acima, por nome. O de 600 s fica por correr na integração.

## Em aberto

- `meio_da_familia` no motor (Acrylic 0,5 → 0) é do coordenador. Enquanto não muda, o degrau de `WetKnobs::set`
  já leva o 0,5 para Water, por isso o resultado na ferramenta é o mesmo.
- A miniatura do Fan Brush (Medium 0,5 → Water, Re-wet 0) pode mudar; não foi regenerada (ordem). Se o portão das
  miniaturas de fábrica ficar vermelho no Fan, o coordenador regenera.
- `F3_controlos.md` §9 (linha Medium, «0..1») ainda descreve o controlo contínuo; fica para o coordenador.
