# A água na placa — degrau 2 (31/10): onde parou

Continuação de [PLACA_degrau1.md](PLACA_degrau1.md). Crate `crates/ph2d-wet-gpu`.

## Feito (verde nas duas placas)

- `momentum` = `momentum_rows` (escreve `vel`; o fluxo na origem do retro-traço por coordenada
  inteira+fração exata, + gravidade × film, clamp `maxVelocity`).
- `suaviza` = `smooth_velocity_rows` **sem gota e sem paredes** (o gate corre com `traco` vazio e a
  gravidade do `Sim::default`; a gota é `pub(crate)` e não se afirma de fora). O limiar `film > 0,05`
  (f64) vira `>= 0.05` em f32 — o menor f32 acima de 0,05 f64 é 0,05f32.
- `PLANO_SUAVE = suaviza → momentum → prepare → outflow → transport → commit` (um quadro de
  suavização sem secagem), `AdvectGpu::encode_plano/step_plano/readback_fluxo`.
- Gate `o_quadro_de_suavizacao_na_placa_casa_com_a_cpu` (`rf` 1 e 4, gravidade (0,004; 0,012)):

| placa | passos | film máx | vel/flow máx | cor máx | Σ rel. (placa) |
|---|---|---|---|---|---|
| RTX | 1 | 3,6e-7 | 1,8e-7 | 1,8e-7 | 1,7e-9 |
| RTX | 8 | 6,0e-7 | 3,0e-7 | 4,2e-7 | 4,5e-8 |
| RTX | 64 | 5,7e-5 | 2,3e-6 | 3,6e-3 ⚠️ | 2,4e-7 |
| Radeon | 64 | 5,5e-5 | 1,3e-6 | 3,6e-3 ⚠️ | 1,2e-7 |

  Afirma 1 passo ≤ 1e-6 por campo (ADR-0127 D4), massa ≤ `1e-7·n`, espelho a 8 passos ≤ 8e-6 (medido
  6,0e-7 RTX, 4,8e-7 Radeon). ⚠️ A cor a 64 passos: `rgb = Σcp/Σp` onde `susp` é ínfimo é mal
  condicionada — o médio é 4,5e-6; decidir se o gate de trajetória pesa a cor pela massa.

## ms de placa (mínimo, `mede_advect`, timestamps por passada)

| medida | RTX 5060 Ti | Radeon RADV |
|---|---|---|
| quadro suave 4096×2560 | **5,95** | 119,8 |
| quadro suave região 1024² | 0,52 | 12,1 |
| quadro suave região 1024² + readback da região (parede) | 4,1 | 17,2 |

O readback da região (~3,6–5 ms de parede) é já o custo dominante na RTX.

## Onde parou — próximo passo exacto

1. `build_flow_field_jacobi` (`solver/flow_jacobi.rs:149`, quadros `n % 4 == 0`): o maior — backrun
   (`flow_jacobi/backrun.rs`), bloom, mistura, gota, paredes. Por portar.
2. `project` (`solver/project.rs`, `n % 3 == 0`) + `apply_boundaries` (estágios 4/5) — Jacobi, deve ser
   direto. Por portar.
3. `sedimenta` (só com `p.sedimenta > 0`) e `rebuild_active_region` (`n % 2 == 0`; candidato:
   `ph2d-gpu-cook/src/scan.rs`). Por portar.
4. Só depois: o gate do PASSO SEM SECAGEM inteiro (sim_step_stage 0,2–5) a 8 e 64 passos.
5. Pequenos: a gota e as paredes no `suaviza`; a K–M no `transport`.
