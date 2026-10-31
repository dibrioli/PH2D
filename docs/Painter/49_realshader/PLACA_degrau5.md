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

## Ronda 2 (31/10)

**Radeon** = `VK_ICD_FILENAMES`/`VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json` (README §2b).

### Secagem única — tolerâncias asseridas (`tests/seca.rs`, 512×320)

| placa | cena | massa rel. máx (tecto 1e-4) | células Δrgb > 1 (tecto 1/400) | ecrã Δ máx (tecto 1,0) | 150: máx · p99.9 |
|---|---|---|---|---|---|
| RTX | 0 | 2,3e-5 | 0 | 0,18 | 0,70 · 0,10 |
| RTX | 1 Glaze | 2,3e-5 | 148 / 163 840 | 0,64 | 0,63 · 0,11 |
| Radeon | 0 | 1,9e-5 | 0 | 0,22 | 0,70 · 0,10 |
| Radeon | 1 Glaze | 1,9e-5 | 126 / 163 840 | 0,64 | 0,63 · 0,10 |

Porquê o ramo vira: o teste refaz a massa nas células divergentes. `sett+dm` e `susp−dm` caem no MESMO índice
truncado de `alpha_of_mass` em f32 e em f64, por isso não é a massa. Todas são `assenta` com `a_s = 0` (sett 0),
Glaze 0,8 e `a_s2 = α(9..30)` pequeno: a cor guardada mal muda o ecrã (`∂ecrã/∂x` perto de
**`CEGO_COM_GLAZE = 0,01`**), e o Newton pára a 0,01 nível do ecrã. O canal decide «cego / não cego» nesse
limiar, por f32 × f64; o resultado são 3–45 níveis de cor GUARDADA que quase não se veem. Na cena 0 (sem Glaze)
o ecrã Δ 0,18–0,22 vem do outro limiar: a massa de ±1 ulp (2,4e-4) que muda de índice truncado da tabela de
opacidade (um degrau = 0,002 × 255 = 0,51 nível).

### ms (parede, mínimo de 7 rondas, aguada 4096×2560, `examples/mede_seca.rs`)

| placa | secagem | passo sem secagem (ciclo 12) | **PASSO com secagem 1 em 6** | loadavg |
|---|---|---|---|---|
| RTX 5060 Ti | **5,16** | 7,95 | **9,24** | 10–30 |
| Radeon RADV | 85,7 | 133,3 | 142,2 | 17–30 |

CPU (sonda, rodada 3): ~243 ms por secagem 4K com SEM; passo médio ~100 ms. A secagem e o passo usam buffers
SEPARADOS (`SecaGpu` × `AdvectGpu`): a soma é a de passadas em série; no produto partilharão o estado (o custo
cai, nunca sobe).

### Gate de cor na cena 4K (4096×2560, 150 secagens, `SECA_COR=1`)

| placa | cena | máx | p99.9 | p99 | u8 máx | Δ>0,5 | veredito |
|---|---|---|---|---|---|---|---|
| RTX | 0 | 0,873 | 0,103 | 2e-4 | 1 | 151 | ✓ |
| RTX | 1 filme+Glaze | **1,0055** | 0,116 | 9e-4 | 1 | 345 | ⛔ máx (por 0,006) |
| Radeon | 0 | 0,703 | 0,107 | 2e-4 | 1 | 119 | ✓ |
| Radeon | 1 filme+Glaze | **1,0057** | 0,118 | 9e-4 | 1 | 402 | ⛔ máx (por 0,006) |

⛔ Na cena 4K com Glaze, o máx passa o critério por 0,006 nível nas duas placas, com o p99.9 5× abaixo. Não baixei
o critério. Os candidatos são o mesmo ramo da secagem única, acumulado (uma célula), e não um erro difuso: o p99
é 9e-4. Por isso o Kahan / double-float das somas (a), (b) NÃO atacam isto. Falta medir e tentar: (1) a decisão
`|d| < cego` com histerese/margem idêntica nas duas (mudaria a CPU — não), ou (2) a derivada e o passo do Newton
em double-float SÓ quando `|d|` está a 2× do `cego` (imita a decisão da CPU); localizar a célula do máx primeiro.

## Ronda 3 (31/10) — a célula do máx 4K (diagnóstico; ⛔ ainda NÃO curado)

`SECA_COR=1 SECA_DIAG=1 … --example mede_seca` (RTX): o máx 1,0055 é a célula **(493, 655)** e as suas
cópias a cada 512 px em x (a cena repete-se), vela 0,8. Estado final: sett 817,316 · susp 0 · film 0 (tudo
assentado); `sett_rgb` R **CPU 49,30 × placa 50,55** (o ecrã vê 1,0055 nível).

**NÃO é o `cego` nem um degrau do Newton.** Re-sincronizado (a placa parte do estado da CPU em cada secagem)
a célula diverge ≤ 0,002 em TODAS as 150 secagens. Livre (a placa sobre o próprio estado): gap 5e-4 até à
secagem 17 e salta para **2,83 na secagem 18** (depois 0,83 → 1,25 e congela quando o filme seca, ~34).
⇒ uma descontinuidade da PRÓPRIA lei (a CPU também saltaria) tocada pelo estado da placa que derivou
~1 ulp/secagem nas MASSAS. Na secagem 18 a CPU tem `susp − dm` = **396,9906** — a 0,009 do degrau 397 do
`alpha_of_mass` truncado (`a_f2` muda um degrau da tabela). Hipótese mais forte (falta imprimir as massas
do estado livre para fechar): a massa da placa (f32 acumulado) passou o 397 e a da CPU não.

**Próximo passo (a cura que imita a CPU):** a CPU guarda `film`/`susp`/`sett` em f32 mas calcula-os em f64
(`dm`, `susp − dm`, `sett + dm`, `lift`, `new_film/f`) e só arredonda no fim. Fazer essas ~10 operações de
massa em **double-float** no `seca.wgsl` (two-sum/two-prod com `fma`; `retention`, `evap_base`,
`edge_dark`, `base_evap`, `ext_gran`, `rewet_base` com a parte `lo` no uniforme — `SecaU` cresce) e
arredondar a f32 no fim ⇒ massas ao bit (quase sempre) com a CPU ⇒ sem deriva ⇒ o índice da tabela decide
igual. Depois: gate de cor 4K nas duas placas, 1 secagem e 512×320 verdes, ms da secagem 4K (≤ +10 % de 5,15).
ms 4K nesta ronda (RTX, sem mudança de código no shader): secagem 5,15 · passo 7,94 · com secagem 9,16.

## Falta (por ordem)

1. **Radeon**: `WGPU_ADAPTER_NAME=radeon` não escolhe a placa (o `GpuContext::new` ignora) — correr o gate
   como o degrau 4 correu na Radeon e pôr a linha na tabela.
2. Afirmar ε da 1 secagem (film/susp/sett ~2,4e-4 = 1 ulp de massas ~2000 — medir a massa relativa) e a
   célula do ramo (contar células com Δ rgb > 1 nível).
3. **ms**: secagem 4096×2560 (montar a aguada 4K num `examples/mede_seca.rs`) e passo inteiro com secagem 1
   em 6 = `7,74 + t_seca/6` RTX (medir corrido, não somado), e a Radeon.
4. Cena 4K no gate de cor (a régua da sonda é 4096×2560; aqui 512×320).
