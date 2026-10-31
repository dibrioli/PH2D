# B — quantos núcleos o Wet Paint usa num wash 4096×2560 (contra o Rebelle) — 30/10

**Comando.** Build: `bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke -p ph2d-wet-paint --example nucleos_wash`
(`smoke` é otimizado, MEDIR_VELOCIDADE §6). Corrida: o binário
`target/smoke/examples/nucleos_wash 30` **directamente, fora do `ph2d-run.sh`** (o cgroup cortaria a CPU). Amostragem: `bash scripts/amostra_cpu.sh <pid> b.csv 75`; largura FP: `perf stat -D 36000 -e fp_ops_retired_by_width.*:u`. Commit base `bdbb414984`. Máquina: Ryzen 9 9950X,
`nproc` = 32 (16C/32T).

**loadavg (1 min).** Amostragem: antes `2,90`, depois `11,63` (inclui o próprio binário). `perf stat`: antes `9,05`, depois `15,30`. GPU `0 %` → `4–5 %`.
Os passos/s batem nas duas corridas (`7,50` e `7,48`).

**Cena.** `Engine::new(4096, 2560)` montada como `ensure_wet_session` (razão 1, fluxo 1 — os defaults do
produto; velatura, `Lei::Cobre`, `pickup_assentado 0`); botão Wet (`wet_canvas_now`); wash em faixas
horizontais de 12 px pela porta do produto `dispatch_pressure_dab(x, y, b=6, r=60)` entre
`pointer_down/up`. Cobertura: água `100 %`, pigmento suspenso `85,4 %` (janela do trail 123², r ≤ 61). Depois 30 s de relógio só de água:
`step_simulation` em laço livre (o produto pede `SIM_HZ = 40`; secagem adaptativa 1 em `dry_every` = 6
nesta cena, `sim.rs:487`, caminhos `*_jacobi` com rayon). Sem óleo, sem render.

**Resultado da água:** `228` passos em `30,41 s` = **7,50 passos/s** (`133 ms/passo`; o produto pede 40/s).

| | PH2D Wet Paint (água) | Rebelle (oráculo) |
|---|---|---|
| núcleos, mediana / pico | **29,2 / 30,8** (de 32 threads lógicas) | ~16 (OpenMP plano a 16) |
| threads com > 0,1 núcleo (mediana / máx) | 32 / 32 | 16 |
| thread mais ocupada (% de um núcleo, mediana) | 95 % | — |
| largura FP escalar / 128 / 256 / 512 | **37,5 / 62,5 / 0,0 / 0,003 %** | 25 / 47 / 28 / — |
| B/px (RSS máx / 4096·2560) | **428 B/px** (4,39 GB) | 215–234 B/px |

FP (20 s, user): escalar 774,9 G · 128 1 293,1 G · 256 `4` · 512 72 M.

**Pintura do wash** (antes do `READY`, 33 s): `4,7` núcleos (pico `4,9`).

## Ronda 2 — onde vão os ~130 ms (perf record)

`perf record -F 999 -g -D 36000` no binário com símbolos (`CARGO_PROFILE_SMOKE_STRIP=none`,
`..._DEBUG=line-tables-only`, target à parte; o código é o mesmo), 11 s só de água: 86 passos, 128 ms/passo,
261 k amostras, loadavg 6,00 → 8,89. Só user (`paranoid = 2`); o `stime` da água é 0,25 % do tempo.

- **Espera não é:** rayon + crossbeam + futex = **0,99 %** das amostras. Núcleos ÚTEIS ≈ 29,2 × 0,99 ≈ **28,9**.
- **Por passe** (partilha × 128 ms/passo médio; sem frame pointers: por nome da função):

| passe | % | ms/passo (média) |
|---|---|---|
| secagem → `assenta` (`drying.rs:210` → `assenta.rs:437`) | 45,7 (+ 6,4 `diferenca_de_pigmento` + 2,9 `pow`, quase todo daqui) | ~58–70 → **~350–420 ms em cada passo que seca** (1 em 6) |
| advecção (`advect_jacobi.rs`) | 33,7 | ~43 |
| campo de fluxo (`flow_jacobi.rs:179`) | 2,8 | ~3,6 |
| projecção | 2,3 | ~2,9 |
| suavizar velocidade | 1,95 | ~2,5 |
| fill / f32→f64, região activa, sedimenta | 3,0 | ~3,9 |

**As 8 funções de trabalho (self):** `assenta::sobre_branco_tapando` (`assenta.rs:334`) 11,1 % ·
`advect_jacobi::transport_rows` (`advect_jacobi.rs:381`) 10,9 % · `commit_rows` (`:512`) 8,0 % ·
`outflow_rows` (`:314`) 5,9 % · `assenta::cor_e_alfa` (`assenta.rs:168`) 4,7 % ·
`colorops::diferenca_de_pigmento` (`colorops.rs:294`) 3,4 %+ · `prepare_rows` (`:155`) 3,2 % ·
`cor_e_alfa_tapando` (`assenta.rs:184`, closures) ~3 % · (`libm pow` 2,9 %).

**O que os números dizem:** os 29 núcleos são trabalho, não rodar em vazio. Num wash a 100 %, mais de
metade do passo é o assentar da secagem (a lei de cor `Cobre`/tapando, f64, célula a célula); a advecção
fica com ~43 ms — a 10,5 Mpx activos, contra os ~5,2 ms da poça da sonda C, que só tem três faixas activas.
A largura 256 continua zero.
