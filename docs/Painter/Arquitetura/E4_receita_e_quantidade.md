# E4 — receita × quantidade (31/10)

Regra do dono (ADR-0197 §1.1): Pigmento + Água → SÓ a receita `φ`; Volume (o slider Opacity) → SÓ a quantidade `h`;
a pressão só toca a quantidade. Fonte única: `crates/ph2d-wet-paint/src/painter/receita.rs` (`receita`,
`forca_do_carimbo`, `agua_do_traco_fino`, `migra_da_lei_velha`). Feita sobre a massa de hoje (não depende da E3).

## Fórmulas

| sítio | antes | depois |
|---|---|---|
| `wp/painter/doors.rs:21-24` (`pressure_dab`) | `I = clamp(b·0,5·(0,2+0,7·s.pressure), 0, 3) · Knob::Intensity` | `I = forca_do_carimbo(b, s.pressure)` (sem `Intensity`) |
| `wp/painter/doors.rs:25-29` | `água = 2(0,35·W)² ÷ (0,2+0,7·s.pressure+0,01) · WaterPerDab` (pressão INVERTIDA) | `água = receita.agua() = (1−φ)(1−m)·h ÷ K_W` |
| `wp/sim.rs:86` `com_pigmento` (← `painter.rs:287`, `doors.rs:169,462`, `po.rs:28`, `mascara.rs:48`) | `PigmentPerDab *= sliders.pigmento` | `com_receita(r)`: `pigmento_do_dab = φ·h`; via `Engine::params_do_deposito()` |
| `wp/trail/deposit.rs:123` | `gain = k(PigmentPerDab)` | `gain = p.pigmento_por_deposito()` (= `φ·h`) |
| `wp/trail.rs:565-566`, `deposit.rs:408-409` | `pig += dep·PPD·Op`, `água += dep·água(W,p)` | sem mudança de código: `pig += dep·φ·h`, `água += dep·(1−φ)(1−m)h/K_W` |
| `wp/trail.rs:614` | `stamp = fall·texv·I` (I com `Intensity`) | `stamp = fall·texv·forca_do_carimbo` |
| `doors.rs:172`, `po.rs:32`, `mascara.rs:51` | `água *= agua_do_dab` / `*= 1/peso` | `agua_do_traco_fino(água, fração)` — **aberto** (não é controlo: correcção do traço fino do hospedeiro; vai ao filme na E6a) |

Receita: `φ = P ÷ (P + K_W·veículo(W, WaterPerDab))`, `veículo = 2(0,35W)²·WaterPerDab ÷ 0,70` (a lei velha à
pressão de referência `0,7`), `h = Volume · carga · H_REF`, `m = 0` (nenhum meio antes da E10). `K_W = 1000`,
`H_REF = 600 + K_W·veículo(1,1) = 950` — o motor de arranque (`PPD 600`, W 1, Intensity 1, pressão 0,7) pousa o mesmo.

## Testes (porta da faixa do produto, um dab, janela do rasto)

| teste | vermelho (código de 31/10) | verde |
|---|---|---|
| `volume_fixo_varrer_pigmento_e_agua_nao_muda_o_volume_depositado` (P 50..1500 × W 0,2..1) | volume variou **×28,9** (76,9 → 2221,5) | espalhamento `< 1e-7` |
| `pigmento_e_agua_fixos_varrer_volume_nao_muda_a_concentracao` (Volume 0,25/0,5/1) | φ **0,373 → 0,704** (+88,7 %) | φ = 0,7042 fixo; volume 285 → 570 → 1141 |
| `a_pressao_so_toca_a_quantidade` (slider 0,1/0,7/1 × `b` 2/5/8) | φ **0,488 → 0,756** (+54,9 %) — só o slider; `b` já não mexia na receita | φ fixo; volume 0,08 → 3783 |
| `cada_preset_de_fabrica_pousa_o_mesmo_pigmento_e_agua_por_dab` | — | 45/45 iguais (tol. relativa `1e-6`) |
| `um_preset_gravado_na_lei_nova_nao_migra_outra_vez` | — | verde |

## Migração dos presets

`wetpaint_preset.rs` `migra_a_receita`: preset sem a marca `receitaE4` → `PigmentPerDab' = PPD·Opacity_do_preset`,
`Intensity' = (PPD·Op + K_W·veículo) ÷ (Op·H_REF)`. Gravar escreve `receitaE4 = 1` (não migra duas vezes). Resultado:
45 pincéis de tinta com o mesmo pigmento e água por dab; as 10 ferramentas da água (Blend ×4, Smear ×5, Water Brush)
não usam o depósito de tinta (só o corpo delas vai ao slot) e ficam de fora. Cargas: 0,011 (Airbrush) … 2,109
(Broad Nib/Dabber) — dentro de `0..4`, nenhuma presa.

**À outra pressão:** a pressão da caneta (`b`, e o `PressureOpacity` → `dab.opacity`) nunca tocou a receita, e o
produto nunca move `s.pressure` (fica `0,7`) ⇒ no produto o pincel de fábrica é o mesmo a TODA pressão. O que morre é
a água invertida do slider do motor: antes a água a `s.pressure` 0 / 0,7 / 1 era **×3,33 / ×1 / ×0,77** da de
referência; agora é ×1.

**O que muda de verdade:** mover a Opacity (Volume) agora escala pigmento E água juntos (antes só o pigmento:
clareava diluindo); mover Pigment/Water muda a concentração com o volume fixo.

## Controlo → grandeza única

| controlo | grandeza | destino |
|---|---|---|
| Opacity (`sliders.pigmento`) → **Volume** | `h` (quantidade por dab) | dono único da quantidade (rótulo na E10) |
| Pigment (`PigmentPerDab`) | parte de pigmento de `φ` | só receita |
| Water (slider) | veículo de `φ` | só receita |
| `WaterPerDab` | escala do veículo | funde-se ao Water |
| `Intensity` | carga de base de `h` | sai do carimbo; funde-se ao Volume (a migração escreve-a) |
| `PressureOpacity` (`mao.rs`) | curva `h(p)` (força do dab) | já era só quantidade: único dono de `h(p)` |
| pressão `b` / slider `pressure` | força do carimbo (área × quantidade) | nunca a receita |

## Em aberto

- `WetKnobs::DEFAULT` (`tp/wetpaint_settings.rs:51`, fora da E4): o Wet Paint SEM preset tem PPD 78 e Intensity 1 —
  na lei nova pousa mais (carga certa = `0,4505`). Um preset de fábrica aplicado já corrige.
- `agua_do_dab`/`peso` do traço fino ainda mudam só a água (E6a, filme).
- Miniaturas/imagem: `φ·h` ≠ `PPD·Op` ao bit ⇒ portões que comparam ao BIT (fingerprints `sim_resumable_tests`,
  `flow_bit_tests`, `product_tool_doors`) e as miniaturas de fábrica mexem ao arredondamento; os que usam
  `sliders.pigmento ≠ 1` ou `s.pressure ≠ 0,7` mudam de verdade (água). Regerar é do coordenador.
