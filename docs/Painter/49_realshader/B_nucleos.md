# B — quantos núcleos o Wet Paint usa num wash 4096×2560 (contra o Rebelle) — 30/10

**Comando.** Build: `bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo build --profile smoke -p ph2d-wet-paint --example nucleos_wash`
(o `smoke` é otimizado; MEDIR_VELOCIDADE §6 diz que serve para régua de CPU). Corrida: o BINÁRIO
`target/smoke/examples/nucleos_wash 30` **directamente, fora do `ph2d-run.sh`** (o cgroup da linha
cortaria a CPU e falsificaria a contagem de núcleos). Amostragem: `bash scripts/amostra_cpu.sh <pid> b.csv 75`
(a régua do oráculo, 1 Hz por thread); largura FP: `perf stat -D 36000 -e fp_ops_retired_by_width.*:u`
(`perf_event_paranoid = 2` ⇒ só user-space, sem sudo). Commit base `bdbb414984`. Máquina: Ryzen 9 9950X,
`nproc` = 32 (16C/32T).

**loadavg (1 min).** Amostragem: antes `2,90`, depois `11,63` (o depois inclui as ~29 threads do próprio
binário). `perf stat`: antes `9,05` (resto da corrida anterior), depois `15,30`. GPU `0 %` → `4–5 %`.
Os passos/s batem nas duas corridas (`7,50` e `7,48`), logo a carga vizinha não mexeu no número.

**Cena.** `Engine::new(4096, 2560)` montada como `ensure_wet_session` (razão 1, fluxo 1 — os defaults do
produto; velatura, `Lei::Cobre`, `pickup_assentado 0`); botão Wet (`wet_canvas_now`); wash em faixas
horizontais de 12 px pela porta do produto `dispatch_pressure_dab(x, y, b=6, r=60)` entre
`pointer_down/up`. Cobertura: água `100 %`, pigmento suspenso `85,4 %` (o dab de r=60 deposita
pigmento numa faixa estreita; a janela do trail é 123², r ≤ 61). Depois 30 s de relógio só de água:
`step_simulation` em laço livre (o produto pede `SIM_HZ = 40`; secagem adaptativa 1 em `dry_every` = 6
nesta cena, `sim.rs:487`; `order_invariant = true` ⇒ caminhos `*_jacobi` com rayon). Sem óleo, sem render.

**Resultado da água:** `228` passos em `30,41 s` = **7,50 passos/s** (`133 ms/passo`; o produto pede 40/s).

| | PH2D Wet Paint (água) | Rebelle (oráculo) |
|---|---|---|
| núcleos, mediana / pico | **29,2 / 30,8** (de 32 threads lógicas) | ~16 (OpenMP plano a 16) |
| threads com > 0,1 núcleo (mediana / máx) | 32 / 32 | 16 |
| thread mais ocupada (% de um núcleo, mediana) | 95 % | — |
| largura FP escalar / 128 / 256 / 512 | **37,5 / 62,5 / 0,0 / 0,003 %** | 25 / 47 / 28 / — |
| B/px (RSS máx / 4096·2560) | **428 B/px** (4,39 GB) | 215–234 B/px |

Contagens FP (20 s de água, user): escalar `774 870 257 331` · 128 `1 293 122 696 747` · 256 `4` ·
512 `72 092 036`.

**A pintura do wash** (o mesmo binário, antes do `READY`, 33 s): mediana `4,7` núcleos, pico `4,9`.

## Onde é serial

Na água, nada de relevante: o pico (30,8) não é ≪ 16, as 32 threads do pool trabalham, por isso o
`perf record` do passo 5 não se correu. O que os números dizem é outra coisa: **usa ~2× os núcleos do
Rebelle e a largura 256 é zero** (o build é x86-64-v2: só SSE, 128 bits; o v3 à mão está recusado no
ADR-0174) — o passo custa `133 ms` com 29 núcleos. O RSS é ~1,9× o do Rebelle por pixel.
A pintura (`dispatch_pressure_dab`) é a parte que não escala (4,7 núcleos).

Ficheiros: `crates/ph2d-wet-paint/examples/nucleos_wash.rs`, `scripts/amostra_cpu.sh`.
