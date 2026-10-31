# A água na placa — degrau 3 (31/10): onde parou

Continuação de [PLACA_degrau2.md](PLACA_degrau2.md). Crate `crates/ph2d-wet-gpu`. Prazo de 45 min do
agente; parou depois do estágio 1.

## Feito — estágio 1: o CAMPO de fluxo (`build_flow_field_jacobi`), verde nas duas placas

- `campo` = `flow_rows` (leveling, capilar, viscosidade, freio por look-ahead sobre o `wet` do INÍCIO
  do passo) e `carimbo` = `wet_stamp_rows` (corre depois). Buffers novos: `paper`, `wet` (u32 por
  célula), `sett`, uniforme `K` (`Knobs::from_params`).
- **Só o caminho por omissão do produto**: sem ladeira do óleo (`g.oleo` vazio), sem paredes (traço
  vazio), sem gota (`drip_head = 0` ⇒ `Gota::new` = `None`), sem fingering e sem backrun (knobs a 0).
  Esses cinco ficam POR PORTAR (o backrun é o passe `backrun_rows` inteiro).
- Limiar da viscosidade: a CPU compara `film as f64 > 0.2`; a placa compara `>=` com
  `f32_acima(0.2)` (o menor f32 cujo f64 passa) — o mesmo predicado bit a bit.
- `PLANO_CAMPO = campo → carimbo → momentum → prepare → outflow → transport → commit`.
- Gate `o_quadro_de_campo_na_placa_casa_com_a_cpu`: cena `cena::poca_papel` (papel com dente,
  `sett`, `wet` semeado, mancha com `film > 3` para o carimbo), cadência do produto (quadro `k` é de
  campo se `k % 4 == 0`, o 1.º medido é de campo), `rf` 1 e 4, gravidade (0,004; 0,012):

| placa | quadros | film máx (médio) | flow máx | cor pesada por susp | wet diferente | Σ rel. (placa) |
|---|---|---|---|---|---|---|
| RTX | 1 | 9,5e-7 (2,2e-8) | 1,8e-7 | 4,8e-8 | 0 | 3,0e-9 |
| RTX | 8 | 2,4e-6 (6,2e-8) | 1,5e-7 | 1,1e-7 | 0 | 3,3e-8 |
| RTX | 64 | 5,9e-3 (1,4e-6) | 1,1e-3 | 2,9e-7 | 0 | 2,5e-7 |
| Radeon | 1 | 9,5e-7 (2,1e-8) | 1,8e-7 | 4,7e-8 | 0 | — |
| Radeon | 64 | 5,9e-3 (1,5e-6) | 1,1e-3 | 3,7e-7 | 0 | — |

  Afirma: 1 quadro ≤ 1e-6 por campo (ADR-0127 D4) e `wet` byte-idêntico; massa ≤ `1e-7·n`. A 64
  quadros só se relata: o máximo 5,9e-3 (rf 1) vem do `trunc` do passo do freio — uma célula cujo
  `ex·s_len` cruza um inteiro sonda outro pixel; ~690 de 64 000 células > 1e-6, o médio fica 1,4e-6.
- **Espelho**: a LEI da CPU não é espelhada — o deslocamento do freio `i + dxs + dys·s` salta de
  linha nas bordas; CPU e placa desviam **o mesmo** 5,008e-2 (médio 3,12e-5) a 8 quadros. O gate
  afirma |espelho placa − espelho CPU| ≤ 8e-6 (é a CPU que tem de ser curada, se for o caso — não
  é da placa).
- **Critério da cor (o pedido 5)**: `desvio_cor_pesado = Σ susp·máx|Δc| / Σ susp`. A cor é
  `Σcp/Σp`; onde `susp` é ínfimo um ulp de `p` vira uma cor inteira sem peso visível nenhum (os 3,6e-3
  do degrau 2). Pesada pela massa: 2,9e-7 a 64 quadros. É este o número a afirmar no gate do passo
  inteiro.

## ms de placa (mínimo, `mede_advect`, loadavg ~30 — a placa não depende dela)

| medida | RTX 5060 Ti | Radeon RADV |
|---|---|---|
| quadro suave 4096×2560 | 5,95 | 124,6 |
| **quadro de campo 4096×2560** | **6,57** | 137,1 |
| quadro de campo região 1024² | 0,59 | 14,1 |
| quadro de campo 1024² + readback da região (parede) | 4,48 | 19,7 |
| CPU `advect_jacobi` 4096×2560 (rayon, sob carga) | 71,7 | — |

Rebelle: 37,3 ms/passo água 4K na CPU dele. O readback da região continua a ser o dominante na RTX.

## Onde parou — próximo passo exacto

1. `project` + `apply_boundaries` (`solver/project.rs`, `n % 3 == 0`; estágios 4/5). Por portar.
2. `sedimenta` (só com `p.sedimenta > 0`) e `rebuild_active_region` (`n % 2 == 0`; candidato
   `ph2d-gpu-cook/src/scan.rs`). Por portar.
3. O gate do PASSO SEM SECAGEM inteiro (estágios 0, 2–5) a 8 e 64 passos, com a cor pesada pela
   massa (critério acima) e a medida final 4K / 1024² com readback só da região suja.
4. Extensões do campo: ladeira do óleo, paredes, gota, fingering, `backrun_rows`.
