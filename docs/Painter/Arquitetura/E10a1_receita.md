# E10-A1 — a receita lê Dilution · Volume · Medium, e a limpeza do registo (01/11)

Spec: [`F3_controlos.md` §9](F3_controlos.md) · base: [`E4_receita_e_quantidade.md`](E4_receita_e_quantidade.md).
Vale nas DUAS leis (`Velha`, a do produto, e `Pigmento`): ambas pedem a receita a `Engine::receita_do_dab`.

## 1. O que mudou

| ficheiro | mudança |
|---|---|
| `crates/ph2d-wet-paint/src/painter/receita.rs` | `Controlos { diluicao, volume_um, meio }`; `do_motor` lê `Knob::Dilution`, `Knob::Volume`, `Knob::Medium` — deixa de ler `PigmentPerDab`, `WaterPerDab`, `Intensity`, `sliders.water` e `sliders.pigmento`. `receita()`: `φ = clamp(1 − Dilution, 0, 1)`, `m = clamp(Medium)`, `h = max(Volume, 0) · M_UM`. Nova `migra_para_controlos(pigmento, agua_por_dab, agua, carga, opacidade) -> (Dilution, Volume µm)`. Ficam `veiculo`, `K_W`, `H_REF`, `migra_da_lei_velha`. |
| `crates/ph2d-wet-paint/src/painter/receita_tests.rs` | os portões reescritos sobre Dilution/Volume, mais a migração e o arranque. |
| `crates/ph2d-wet-paint/src/tuning.rs`, `tuning/defs.rs` | saem `ExtRakeLight`, `ExtValleyGran`, `ExtWetSheen`, `ExtGrainDither`, `ExtEdgeTint` (`KNOB_COUNT` 148 → 143); `PigmentPerDab`, `PaperGate`, `Intensity`, `WaterPerDab`, `LevelingOwnStroke` passam a `KnobGroup::Hidden` (não se apagam). |
| `crates/ph2d-wet-paint/src/render.rs` | `render_region` perde os cinco ramos de Look (no ponto de omissão estavam todos inertes: 0, e `edgeTint` 0,5 → 0), e o `use rng::hash2`; o parâmetro `ext_bypass` fica (as chamadas em `tests/it/{measure,perf}_experimental.rs` não mudam). |
| `crates/ph2d-wet-paint/src/trail/espalha_tests.rs` | o motor do teste passa por `migra_para_controlos(3000, 1, 0, 10, 1)` em vez de PPD/Intensity/Opacity. |
| `crates/ph2d-panel-wet-tuning/src/rows.rs` | o censo: 96 → 91 linhas (Paint 22 → 19, Water 8 → 7, Stroke 12 → 11); a lista dos escondidos ganha os 4 controlos E10 e os 5 fundidos; a lista `MORTOS` sai. |
| `crates/ph2d-tool-painter/tests/it/familias/{aguarela,guache}.rs` | os `extras` com as chaves mortas saem (o `presets_de_fabrica` assere «knob existe»). |

Nenhuma escrita por número de knob fora de `KNOB_COUNT` (o GPU lê por `Knob::`; presets e macros gravam por CHAVE).

## 2. As fórmulas

- `φ = 1 − Dilution` · `h = Volume · M_UM` (`M_UM = 1374`, massa por µm) · `m = Medium`.
- Migração exacta (com `P'` = o PPD do preset, que a E4 já dobrou com a Opacity):
  `Dilution = 1 − P'/(P' + K_W·veiculo(W, WPD))` (0 pigmento e 0 veículo → 1, onde a E4 dava φ 0);
  `Volume = Op · carga · H_REF / M_UM`.
- Arranque do motor: as omissões do coordenador (`Dilution = 1 − 600/950`, `Volume = 950/1374`) dão AO BIT a
  receita da E4 de arranque — por isso nenhum hash muda.

## 3. Testes (todos `--profile smoke`, por nome)

| teste | antes | agora | número medido |
|---|---|---|---|
| `migra_para_controlos_da_a_receita_da_e4` (novo) | — | verde | 1536 receitas (PPD × WPD × Water × Op × carga): `|Δφ|` máx `5,6e-17`, `Δh` relativo máx `1,3e-16` (tecto 1e-12) |
| `o_arranque_do_motor_e_a_receita_da_e4` (novo) | — | verde | φ `0.63157894736842102` = E4, h `950` = E4, Δ = 0 nos dois |
| `volume_fixo_varrer_diluicao_nao_muda_o_volume_depositado` | (era sobre PPD/Water) | verde | receita: espalhamento 0 (< 1e-7); depósito: `< 1e-6` sobre 6 Dilutions (tecto f32 1e-4) |
| `diluicao_fixa_varrer_volume_nao_muda_a_concentracao` | (era sobre PPD/Opacity) | verde | receita 0; depósito φ = 0,6 ± 3e-9; volume 165 → 577 → 1138 |
| `a_pressao_so_toca_a_quantidade` | verde | verde | φ 0,6 ± 6e-9 sobre 9 (pressão × b) |
| `advect_jacobi_` | verde | verde | 4 passaram, 1 ignorado — hashes intactos |
| `o_invariante_soma_massa` | verde | verde | 1 |
| `tinta` | verde | verde | 67 passaram, 1 ignorado |
| `espalha` (`o_oleo_grosso_sobre_o_oleo_grosso_…`) | vermelho com a receita nova sem migrar o teste | verde | — |
| `bingham`, `product_rewet`, `lei_da_tinta`, `deposito_tinta`, `tuning` | — | verdes | — |
| `every_visible_knob_has_a_row`, `os_escondidos_sao_so_os_de_outra_superficie_ou_de_calibracao` (panel-wet-tuning) | — | verdes | — |

Não houve hash a regravar: a prova é o arranque igual ao bit (linha 2 da tabela).

## 4. Leitores dos controlos velhos (produção)

| leitor | o que é | feito |
|---|---|---|
| `wp/painter/receita.rs` `Controlos::do_motor` | a receita | mudado (lê os knobs E10) |
| `wp/painter/doors.rs:23` `water_unit = s.water * 0.35` | o braço da ferramenta Wet | intocado (não é a receita) |
| `wp/tools.rs:194` `apply_wet` lê `Knob::WaterPerDab` | a ferramenta Wet | intocado — por isso o WPD fica no registo (Hidden) |
| `wp/sim.rs:133` `pigmento_por_deposito` cai no `PigmentPerDab` cru | portas que não passam por `com_receita` | intocado; ⚑ ver §5 |
| `tp/wetpaint/session.rs:227-234` escreve `sliders.water`/`sliders.pigmento` | sessão (B1) | não tocado — o Water ainda serve o Wet; a Opacity deixou de mexer na receita |
| `tp/wetpaint_preset.rs:112-119`, `wetpaint_preset/preview.rs:473-480` (`migra_da_lei_velha` → PPD/Intensity) | migração E4 dos presets (B1) | não tocado — B1 tem de escrever Dilution/Volume (`migra_para_controlos`) |
| `tp/wetpaint_settings.rs:51,53,236,572` (PPD 78, Intensity 0,4505, linha Pigment) | omissões e ids (B1) | não tocado |
| `tp/wetpaint_settings/tabelas.rs:78` `Knob::Intensity` | tabela (B1) | não tocado |
| `tp/wetpaint_preset/medida_oleo_real.rs:75` | sonda (B1) | não tocado |

## 5. Em aberto

1. **B1**: `tp/wetpaint_preset/receita_e4_tests.rs:47,147` constrói `Controlos { pigmento, agua, agua_por_dab, volume, carga, meio }`; com a forma nova não compila. A troca é `migra_para_controlos` + `Controlos { diluicao, volume_um, meio }`. As omissões do produto (`wetpaint_settings.rs:51,53`, PPD 78 · carga 0,4505) têm de virar `Dilution`/`Volume` pela mesma função; até lá o produto pinta com a receita de ARRANQUE do motor (φ 0,632, h 950), não com a do Wet Round.
2. **Sessão (B1)**: `sliders.pigmento` (Opacity) já não tem leitor no motor — o volume do topo tem de escrever `Knob::Volume`.
3. **Coordenador**: `wp/tests/f3_medicoes.rs:73-74` carrega um preset por PPD/Water/Opacity; precisa da migração para os knobs E10 senão mede a receita de arranque.
4. **A2**: `grid/meio_corpo_tests.rs:38` `o_meio_viaja_do_controlo_ao_deposito` reprova («Medium 0: o dab não pousou») — teste novo da A2, em curso; não investigado por mim.
5. `wp/sim.rs:133`: o `PigmentPerDab` cru continua a ser o pigmento das portas que não chamam `com_receita`; quando o produto deixar de escrever o PPD, essas portas pintam com 600. Censo dessas portas por fazer.
6. Os 55 presets de fábrica `assets/brushes/WetPainter/**.ph2d-brush.png` levam as 5 chaves mortas (no ponto inerte, o leitor ignora chave desconhecida); somem na próxima regravação dos presets.
