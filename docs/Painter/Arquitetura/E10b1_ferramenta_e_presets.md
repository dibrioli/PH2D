# E10-B1 — a ferramenta: ids, roteamento, omissões, sessão e presets (01/11)

Especificação: `F3_controlos.md` §3, §8, §9. Abreviaturas: `tp/` = `crates/ph2d-tool-painter/src/tool/paint/`.

## O que mudou

| sítio | mudança |
|---|---|
| `crates/ph2d-tool-painter/src/ids/painter_wetpaint.rs` | ids novos `PAINTER_WETPAINT_{DILUTION, MEDIUM, BODY, HARDNESS}`; `PAINTER_WETPAINT_FIELDS` 37 → 41. `PAINTER_WETPAINT_PIGMENT` fica (o painel ainda o pinta) mas **sem roteamento** |
| `tp/wetpaint_settings/controlos_e10.rs` (novo) | acessores `dilution()`, `medium()`, `volume_um()`, `body()`, `hardness()`, `set_hardness()`; `set_controlo_e10` encaminha DILUTION → `Knob::Dilution`, MEDIUM → `Medium`, BODY → `Body`, HARDNESS → `PaperGate = (1 − v).clamp(0,1)·máx` (o máx lido da `KNOB_DEFS`, 1,5), VOL_OPACITY → `Knob::Volume` (µm); `const fn dilution_e_volume` (a migração em `const`) |
| `tp/wetpaint_settings.rs` | `painter_default_knobs`: Dilution e Volume do Wet Round pela fórmula exacta (PPD 78, WPD 1, Water 1, carga 0,4505, Op 1) → Dilution 0,817757, Volume 0,3115 µm; Medium 0, Body 0. A rota do Pigment saiu |
| `tp/wetpaint_settings/tabelas.rs` | `CERDAS` perde a `Intensity` (sem leitor desde a receita da E10); o corpo do Wet ganha `WaterPerDab`, que passa a `LIDOS_SO_PELA_FERRAMENTA` (só a água do Wet o lê, `wp/tools.rs:194`) — medido por `o_corpo_de_cada_ferramenta_da_agua_e_o_que_ela_le` |
| `tp/wetpaint/session.rs`, `tp/wetpaint/authored_actions.rs` | `WetEngineFacts::opacidade` (→ `sliders.pigmento`) saiu; entra `pigment_mix` = `BrushSpec.pigment_mix` com `pigment` ligado, senão 0 → `e.sim.pigment_mix` (o `Params::pigment_mix` da E10c). `km_mixing` intacto |
| `tp/wetpaint_preset/controlos_e10.rs` (novo) | `CHAVE_CONTROLOS = "controlosE10"`, `BODY_DO_LEVELING = 0.16` (E10-A2), `meio_da_familia`, `familia_do_preset`, `migra_para_controlos`; `migra_a_receita` (E4) e a faixa das miniaturas (`knobs_da_faixa`, agora também com Dilution/Volume) mudaram para aqui por LOC |
| `tp/wetpaint_preset.rs` | carregar: a E10 corre depois da E4 sem a marca; a `evaporation` não se lê (fica a de agora); a Opacity de fallback é `OPACIDADE_DE_FABRICA`. Gravar: escreve as chaves novas e a marca, apaga `evaporation`, `volumes.opacity = None`. Volumes vivos: Dilution e Volume no `Volumes.extra` (chaves `dilution`, `volume`) — a política «lembrar volumes» guarda Size · Water · Dilution · Volume. O volume 2 (o «Opacity» do topo, SetValue e quick) escreve `Knob::Volume` |

## Fórmulas

- `Dilution = 1 − P'/(P' + K_W·veículo(W, WPD))`, `P' = PPD·Op` (E4), `Volume = Op·carga·H_REF / M_UM` µm.
- `Medium` (§3.2): `Oil/*`, `ExpressOil/*`, `Pastel/Oil`, `Pastel/Scrape` → 1; `Acrylic` → 0,5; o resto 0.
- `Body = BODY_DO_LEVELING (0,16)` se `levelingOwnStroke = 1` **e** Medium 0 (o óleo tem a cedência do meio: Body 0).
- Hardness = `1 − paperGate/1,5` (vista; não migra).

## Testes (por nome)

| teste | resultado |
|---|---|
| `cada_preset_de_fabrica_mapeia_por_funcao` | verde: **55** presets Wet Paint CONTADOS (45 de tinta + 10 ferramentas da água); (a) φ e h = E4 a 1e-6 nos 55; (b) Medium: 36 × 0, 1 × 0,5, 18 × 1; (c) **nenhum preso** pela faixa (Volume máx 1,458 µm, Broad Nib/Dabber); (d) gravar → recarregar → gravar igual nos 45. Body 0,16: Wet Liner, Wet Wash, Gouache Opaque |
| `um_controlo_da_tela_nao_viaja_no_preset` | verde (Ink Brush traz evaporation 3: fica 0,4; a luz do Real Oil `oleo_real_valores` não muda; o Tilt viaja por decisão W1e — não mexido) |
| `a_dureza_e_uma_vista_exacta_do_paper_gate` | verde (ida e volta < 1e-12 em toda a grelha do passo) |
| `os_knobs_mortos_do_preset_sao_ignorados` | verde (as 5 chaves fora do registo não mudam knob nenhum; regravam-se intactas como campos desconhecidos) |
| `o_pincel_sem_preset_e_o_wet_round_migrado` | verde |
| `receita_e4_tests` (4), `tests_doc22` (25), `corpo_da_agua_tests` (2), `showwet_tests` (2), `pincel_nao_recompoe_tests`, `the_engine_boots…`, `a_turned_knob…`, `the_knobs_survive…` | verdes depois de passarem de Pigment/Opacity a Dilution/Volume |
| `wetpaint_preset::tests::a_opacity_do_wet_paint_e_a_quantidade_de_pigmento` | **vermelho**: a 26 % do Volume a razão do escuro dá 17/47 = **0,3617**, a barra é 0,18..0,36 — por investigar se já vinha das mudanças do motor (A1/A2) ou da quantização inteira da mediana |

## Para a E10-B2 (painel + i18n)

- Linhas novas: `PAINTER_WETPAINT_DILUTION` (0..1, passo 0,001, volumes do topo no lugar do Water do Pincel), `_MEDIUM` (0..1, 0,01, cartão), `_BODY` (0..1, 0,01, junto do Leveling), `_HARDNESS` (0..1, cartão); valores por `WetKnobs::{dilution, medium, body, hardness}()`; faixas da `KNOB_DEFS` (Hardness: 0..1).
- `PAINTER_WETPAINT_VOL_OPACITY` agora é o **Volume em µm** (`volume_um()`, faixa 0..1000): o rótulo e o campo; os quick values do volume 2 (`ids::vol_quick_values(2)`, hoje `VOL_QUICK_UNIT` 0..1) precisam de uma lista em µm (`ids/painter_wetpaint_presets.rs`).
- Tirar a linha Pigment, `PAINTER_WETPAINT_PIGMENT` do `PAINTER_WETPAINT_FIELDS` e o acessor `WetKnobs::pigment()`.
- Tooltips: o `populate_wetpaint.rs` mora no painel (não há chaves i18n desta linha); sugestões em inglês: Dilution «How much water or medium thins the paint: 0 = pure pigment, 1 = no pigment.» · Medium «What the paint is made with: 0 = water (dries and lightens), 1 = oil (stays and keeps its body).» · Volume «How much paint each dab lays down, in micrometres.» · Body «How stiff the paint is: it holds the brush marks instead of levelling out.» · Hardness «How hard the tip is: hard tips only touch the paper's peaks.»
- `seam_wetpaint`/`o_painel_mostra_o_que_a_ferramenta_le` no painel: o Pigment já não tem leitor na ferramenta.

## Em aberto

- A família vem da `library_folder`; a ponte (`crates/ph2d-app-painter/src/painter_bridge_presets.rs::aplicar`) aplica o preset sem ela, e então vale o fallback pelos grupos de fábrica (`familia_do_preset`, verificado igual nos 55). A ponte devia pôr `library_folder = e.partes()[1]` antes de aplicar (fora do B1).
- `WetPaintState::opacidade` ficou sem leitor no motor (snapshot, macro e preview ainda o tocam: `snapshot.rs`, `macro_estado.rs`, `preview.rs`) — remover com quem possui esses ficheiros.
- `sonda_varredura_oleo.rs:274` encaminha `PAINTER_WETPAINT_PIGMENT` (agora no-op).
- O Volume é partilhado pelo Pincel e pelas ferramentas da água que o leem pela linha `DepositoMolhado` (antes a Opacity).
