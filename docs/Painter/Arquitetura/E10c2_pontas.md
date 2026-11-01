# E10-C2 — as pontas soltas da E10 (agente C, 01/11)

Abrev.: `wp/` = `crates/ph2d-wet-paint/src/`, `tp/` = `crates/ph2d-tool-painter/src/tool/paint/`.

## 1. Portas que caem no `PigmentPerDab` cru

Censo (grep `gather_params` · `params_do_deposito` · `com_receita` · `pigmento_por_deposito`): o ÚNICO leitor do
cru é `DepositLaw::new` (`wp/trail/deposit.rs:124`), alcançado só por `Trail::accumulate_paint*`. Todas as portas
de produção que POUSAM tinta já passam por `params_do_deposito` (= `com_receita`):

| porta | onde |
|---|---|
| `dispatch_pressure_dab` (o rasto do motor) | `wp/painter.rs:282` |
| `dispatch_pressure_dab_lane_cor` (as lanes do produto) | `wp/painter/doors.rs` |
| `land_direct_remainders` (a janela que sobra) | `wp/painter/doors.rs` |
| `deposit_mask` (Solid / fios) | `wp/painter/mascara.rs:45` |
| `pousa_grao_solto` (o pó) | `wp/painter/po.rs:25` |

As portas com `gather_params` (borracha, Blend, Wet/Dry/Blow/Smear, secar a tela, o misturador da janela) não pousam
pigmento: não leem o `gain`. Nenhuma porta de `tp/` chama `accumulate_paint` directamente. O fallback `None` → PPD
fica só para os testes do rasto (constroem `Params` à mão).

Portão novo: `toda_porta_que_pousa_tinta_le_a_receita` (`wp/painter/receita_tests.rs:214`) — cada uma das 5 portas
pinta com PPD 600 e com PPD 5 (Dilution 0,4, Volume `H_REF/M_UM`): Σ pigmento (grade + rastos) igual AO BIT.
Verde. **Mutação**: `deposit_mask` com `gather_params` → vermelho («deposit_mask: o PigmentPerDab cru chegou ao
depósito»); reposto.

## 2. A parede binária: restos apagados

- `Grid::traco`, o ficheiro `wp/grid/traco.rs` e o `mod`; leitores/escritores em `wp/grid/{canvas_ops,patch,snapshot}.rs`
  (o `comprimentos_vazios` passa de 10 a 9 planos, índices refeitos), `wp/painter/instantaneo.rs`,
  `wp/painter/historia_tests.rs`.
- `Sim/Params::{traco_actual, paredes}` (`wp/sim.rs`), `Engine::paredes_valem` e o número do traço em
  `begin_direct_stroke` (`wp/painter/doors.rs`), `wp/painter.rs` `step_simulation`, `wp/painter/stage.rs`.
- `Knob::LevelingOwnStroke` + a linha do registo (`wp/tuning.rs`, `wp/tuning/defs.rs`); **`KNOB_COUNT` 143 → 142**.
  O censo do painel (`crates/ph2d-panel-wet-tuning/src/rows.rs`, `NOUTRA_SUPERFICIE` 38 → 37, contado) verde.
- A migração lia a parede **pelo Knob** (`k.get(Knob::LevelingOwnStroke)`): passa a ler a chave do mapa do preset,
  `CHAVE_PAREDE = "levelingOwnStroke"` (`tp/wetpaint_preset/controlos_e10.rs:18,102`). Equivalente: o `k` parte de
  `WetKnobs::DEFAULT` (parede 0) e só o mapa a escrevia.
- `tp/wetpaint/undo_vivo.rs` (o `Relogio` guardava `traco_actual`), `tp/wetpaint/undo_tinta_viva_tests.rs`,
  `tp/wetpaint_preset/medida_oleo_real_faca.rs` (a variante «lev2» usa agora `Knob::Body = BODY_DA_PAREDE`).
- **Fora da lista do briefing, necessário**: `crates/ph2d-wet-gpu` lia `g.traco` e `p.paredes`
  (`src/lib.rs`, `src/transfere.rs`, `src/passo.rs`, `src/passo/hashes.rs`) — deixam de ler; `Knobs::isola` fica
  sempre `0`. `tests/passo.rs`: o portão de placa das paredes sai (já não compilava: chamava
  `grid::traco::separa`, apagado pela A2); `aguada_com_paredes` → `aguada_que_se_toca`.

Bits: nenhum hash mudou (`advect_jacobi_` 4/4, `historia` 6/6, `snapshot`, `patch` verdes). O campo estava vazio
desde a A2 (sem escritor), por isso apagá-lo não muda nenhum plano.

## 3. `WetPaintState::opacidade` (morto)

Removido de `tp/wetpaint.rs` e da canalização: `tp/wetpaint_preset/preview.rs` (as duas chaves da cena e o restauro;
a faixa usa `OPACIDADE_DE_FABRICA`), `tp/macro_estado.rs`. **Formato da macro**: deixa de gravar `"opacidade"`; ao
LER uma macro antiga a chave é ignorada (comentário no sítio). Testes que punham `opacidade = 1.0` («cor cheia») sem
efeito desde a E10: linhas apagadas (`blend_tests`, `grid_ratio_tests`, `wetpaint/tests.rs`, `mistura_camadas_tests`
`Cond::opacidade`, `sonda_camadas`, `sonda_presets`); `painel_so_no_futuro_tests` perde a linha (o Volume já entra
pelo laço do registo); `preview_tests` troca «opacity» por «volume» (`Knob::Volume = 300`).
⚠️ **Fica** `BrushSettingsSnapshot::wet_opacidade` (`tp/brush_settings.rs`), sempre `OPACIDADE_DE_FABRICA` e
marcado ⛔ morto: o painel (B2) ainda o nomeia em `ph2d-panel-painter-layers/src/brush_fallback.rs:296` e
`tests/it/seam_wet_presets.rs:150-166` — sai quando a B2 apagar essas linhas.

## 4. Sondas

- `wp/tests/f3_medicoes.rs`: `motor()` passa pela migração do carregador — `receita::controlos_do_preset` (E4 se o
  preset não traz `receitaE4`, depois E10) → Dilution/Volume; Medium por `receita::meio_da_familia` com a 6.ª coluna
  opcional `Família/Grupo` do ficheiro (sem ela, o `realOil` do preset); Body = `BODY_DA_PAREDE` se
  `levelingOwnStroke = 1` e meio 0. `sliders.pigmento` e os `traco_actual` saíram; a variante `c` varre `Body = 0`.
- Para haver UM só sítio, `meio_da_familia` e `BODY_DA_PAREDE` mudaram para `wp/painter/receita.rs` (`:137`, `:147`);
  `tp/.../controlos_e10.rs` delega. `controlos_do_preset` (`:153`) é nova.
- `tp/wetpaint_preset/sonda_varredura_oleo.rs`: envia `PAINTER_WETPAINT_DILUTION` com a MESMA `φ` da carga `pig`
  sobre a água da cena (`migra_para_controlos(pig, WPD, agua, 1, 1).0`) em vez do Pigment (no-op).
- `tp/wetpaint_preset/medida_oleo_real.rs`: a coluna `Pigment` (PPD) → `Dilution` + `Vol_um`.
- `tp/wetpaint_preset/medida_oleo_real_faca.rs`: as duas variantes «pig500» (PPD 500, no-op) → `pig500()` = a
  Dilution com a mesma `φ`.
- `tp/wetpaint/sonda_presets.rs`: o parâmetro `opacidade` sai; a 4.ª coluna do ficheiro é ignorada.
- Compilação das sondas: `cargo test --profile smoke --no-run -p ph2d-wet-paint -p ph2d-wet-gpu -p ph2d-tool-painter
  -p ph2d-panel-wet-tuning -p ph2d-app-painter --all-targets` — verde.

## 5. A ponte

`crates/ph2d-app-painter/src/painter_bridge_presets.rs::aplicar`: preset sem `library_folder` aplica-se com
`library_folder = e.partes()[1]` (cópia), então o Medium migra pela pasta real em vez do fallback dos grupos.

## 6. Avisos

`sonda_relevo_oleo.rs:15` (`RasterEditTool` não usado) limpo. O `preview.rs:487` já não aparece. Resta um aviso
ALHEIO: `set_wet_lei_da_tinta`/`wet_lei_da_tinta` sem chamador na lib (`tp/wetpaint.rs:266`, só testes/sondas) —
a porta do produto que a E12 liga; não silenciei.

## Testes por nome (`--profile smoke`)

Verdes: wp `advect_jacobi_` 4 · `o_invariante_soma_massa` 1 · `tinta` 68 · `historia` 6 · `meio_corpo` 3 ·
`oleo_celula` 6 · `bingham` 1 · `receita` 6 · `snapshot` 1 · `patch` 5 · `canvas` 1 · `tuning` 3 · `knob` 4 ·
`toda_porta` 1; tp `controlos_e10` 5 · `cada_preset_de_fabrica_mapeia_por_funcao` 1 · `tests_doc22` 25 ·
`snapshot` 10 · `macro_estado` 6 · `receita_e4` 4 · `undo_tinta_viva` 8 · `mistura_camadas` 2; panel `rows` 6.

Vermelhos, **não causados por esta ponta** (nenhum lê `traco`/`paredes`, e a `opacidade` não tinha leitor no motor):
- `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` — «óleo true: … `["real_oil"]`» (ligar o Real Oil muda a
  tinta pousada: o meio passou ao `Knob::Medium`, B1/A2).
- `preview::tests::a_chave_muda_com_cada_parametro_e_o_traco_tambem` — «água: o preview não mudou» (a receita já
  não lê o Water desde a A1).
- `blend_tests::a_identidade_do_modo_nao_pinta_sobre_a_tela` — Screen muda 5458 texels até Δ 9.
- `grid_ratio_tests::smooth_edges_no_zero_e_o_desligado_de_antes_ao_byte` — hash `17109263309163082569` ≠
  `17933416236263888171` à razão 1.

## Em aberto

1. GPU: o `parede()`/`separa()` do `advect.wgsl`, o binding 18 (`traco`) e `Knobs::isola` ficam INERTES (sempre 0,
   o shader dá o mesmo resultado) — tirá-los muda os layouts de `usa` (o passe `suaviza` perde o binding 15) e pede o
   portão da placa, que este agente não pode correr.
2. B2: apagar `wet_opacidade` em `brush_fallback.rs:296` e `seam_wet_presets.rs:150-166`, e a chave i18n
   `panel.wet_tuning.knob.levelingOwnStroke` (`ph2d-i18n/src/wet_tuning_avancado.rs:63`), agora órfã; depois sai
   `BrushSettingsSnapshot::wet_opacidade`.
3. `tp/wetpaint/mistura_camadas_tests.rs` (controlo «antes») ainda põe `PigmentPerDab = 800` no motor — no-op desde
   a E10; o controlo continua a reprovar pelas outras leis (o teste está verde), mas a linha não mede nada.
4. `git rm --cached` deixou a remoção de `wp/grid/traco.rs` JÁ no índice (`D `): quem comitar por caminhos inclui-a.
5. Os 4 vermelhos acima são do dono do motor/preset (B1/A2/E12).
