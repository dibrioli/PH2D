# A água na placa — degrau 5 (31/10): a SECAGEM (onde parou — prazo de 45 min)

Continuação de [PLACA_degrau4.md](PLACA_degrau4.md). Crate `crates/ph2d-wet-gpu`, ficheiros novos
`src/seca.wgsl`, `src/seca.rs` (`SecaGpu`, `SecaU`), gate `tests/seca.rs`.

## Feito

`drying_pass_jacobi_rows` → `dry_cell` → `assenta`/`levanta` (lei **Cobre**, o `seco()` ⇒ molhada 1) em f32,
duas passadas: `borda` (janela 3×3 de `susp > 10` sobre o susp de ANTES — Jacobi, igual a `edge_rows`) e
`seca` (uma thread por célula; faixas por linha = `span_x_of` subidas do host). Vela/vidrado/óleo por
célula (`com_vela/com_vidrado/com_oleo`). Cortes ao bit: **SALTA** (levanta com coberturas iguais devolve
`x0`) e **SEM** (volta sem canal corrigido sai). Tabela de opacidade = a `OPACITY` f32 da CPU.
`tapa` é calculado eager (a CPU é lazy — o valor é o mesmo). A derivada avalia o ecrã inteiro (na placa o
CANAL não compra quase nada: `d` e `kv` precisam dos 3 canais).

**f32 «compensado» no ponto crítico**: `entre_perfis` = `sqrt(1 − cos²)` cancela em f32 perto de cos² = 1
(cores parecidas ⇒ erro ~2e-4 em `d`). Na placa: `1 − cos² = |a×b|²/(|a|²|b|²)` (identidade de Lagrange,
exacta em aritmética real) — knob `SecaU::cross` (1 = Lagrange, 0 = literal) para a régua A/B.

Não portado: as leis Porto / PorCima / Camada (o `SecaU::new` recusa com panic), `km_mixing`.

## Gate `a_secagem_na_placa_casa_com_a_cpu` (RTX 5060 Ti, Vulkan)

Cena = a aguada de `sonda_secagem::wash` (`nucleos_wash`, Cobre) em **512×320** (não 4K — prazo); cena 1 =
filme 0,3 + Glaze `vela 0,8` na metade de cima. 150 secagens; cor = `sobre_branco` da CPU sobre os dois
estados; régua em níveis de 8 bits. Critério invisível máx ≤ 1 · p99.9 ≤ 0,5 — **cumpre sem Kahan nem
double-float**.

| cena | `entre` | 1 secagem: film · susp · sett · sett_rgb máx | 150: máx · p99.9 · p99 · u8 máx · Δ>0,5 |
|---|---|---|---|
| 0 wash | Lagrange | 6,0e-8 · 2,4e-4 · 2,0e-4 · 0 | **0,70 · 0,10 · 2e-4 · 1 · 1** |
| 0 wash | literal | idem | 0,86 · 0,33 · 0,13 · 1 · 59 |
| 1 filme+Glaze | Lagrange | idem · sett_rgb 66,5 (1 célula) | **0,63 · 0,11 · 9e-4 · 1 · 4** |
| 1 filme+Glaze | literal | idem | 0,84 · 0,30 · 0,06 · 1 · 42 |

Leitura: o cancelamento do `entre_perfis` era 2/3 do p99.9; com Lagrange o f32 fica 5× abaixo do critério
sob Glaze. O `sett_rgb` 66 numa célula a 1 secagem (cena 1) é um ramo do Newton que vira (cego/clamp) — não
sobrevive à cor final; o gate de campos de 1 secagem ainda NÃO afirma ε (só imprime).

## Falta (por ordem)

1. **Radeon**: `WGPU_ADAPTER_NAME=radeon` não escolhe a placa (o `GpuContext::new` ignora) — correr o gate
   como o degrau 4 correu na Radeon e pôr a linha na tabela.
2. Afirmar ε da 1 secagem (film/susp/sett ~2,4e-4 = 1 ulp de massas ~2000 — medir a massa relativa) e a
   célula do ramo (contar células com Δ rgb > 1 nível).
3. **ms**: secagem 4096×2560 (montar a aguada 4K num `examples/mede_seca.rs`) e passo inteiro com secagem 1
   em 6 = `7,74 + t_seca/6` RTX (medir corrido, não somado), e a Radeon.
4. Cena 4K no gate de cor (a régua da sonda é 4096×2560; aqui 512×320).
