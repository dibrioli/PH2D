# Sondas C (30/10) — f32 no solver e despacho v3, FORA do produto

- **Comando:** `bash scripts/ph2d-run.sh env PH2D_RELEASE="mede velocidade" cargo rustc --profile smoke -p ph2d-wet-paint --example sonda_f32_v3 -- --cfg sonda_v3 --cap-lints warn`, depois `bash scripts/ph2d-run.sh target/smoke/examples/sonda_f32_v3`. Usei o perfil `smoke` porque o MEDIR_VELOCIDADE §6 diz que ele já é otimizado para uma régua de CPU. Uma só compilação; a escolha da variante é feita em execução.
- **Máquina:** Ryzen 9 9950X (32 fios), rayon com 16 fios (fatia do `ph2d-run`), placa a 0 %. **loadavg** `0.75 0.83 5.57` antes e `1.83 1.10 5.53` depois.
- **Commit base** `341341b233`. **Cena:** a poça do ADR-0147 §4 (`scene_big`, 4096², janela x 998..3563 × y 1139..3137). No instantâneo: film máx. 0,27, `km_mixing` desligado, `dry_every` 6.
- **Código:** `crates/ph2d-wet-paint/examples/sonda_f32_v3.rs` mais `sonda_f32_v3/{kern,par,k64,k32,k64v3}.rs`. A cópia `k64` dá o **mesmo hash** que a original nas rotas série e paralela, o que prova que a cópia é fiel.
- **Régua:** 1 rodada de aquecimento, depois 7 rodadas intercaladas com o início rodado. Em cada rodada correm 3 advecções seguidas e 3 secagens, cada secagem sobre o instantâneo restaurado. O resumo é o mínimo, com a mediana ao lado.

| variante | advect ms (mín / med) | drying ms (mín / med) | hash após N=12 passos (cadência do produto) |
|---|---:|---:|---|
| original f64, paralela | 5,21 / 5,35 | 0,68 / 0,72 | `57728ff4f25d0ebf` |
| cópia f64, paralela | 5,22 / 5,36 | 0,70 / 0,73 | `57728ff4f25d0ebf` |
| **cópia f32, paralela** | **5,17 / 5,31** | 0,68 / 0,73 | `a07ba6913bc5e708` |
| original f64, série | 32,92 / 33,97 | 4,91 / 5,05 | `57728ff4f25d0ebf` |
| cópia f64, série (v2) | 35,67 / 36,90 | 4,92 / 5,07 | `57728ff4f25d0ebf` |
| **cópia f32, série** | **34,27 / 35,24** | 4,90 / 5,00 | `a07ba6913bc5e708` |
| **cópia f64, série, v3** | **32,65 / 33,67** | **4,85 / 4,96** | `57728ff4f25d0ebf` |

**Desvio f32 × f64**, rota paralela, na janela inicial. Cada célula diz máx. |Δ| / médio |Δ|, e entre parênteses a % de células com bits diferentes.

| campo | N=12 passos | N=96 passos |
|---|---|---|
| film | 5,3e-3 / 7,8e-9 (0,01 %) | 0 (a poça já secou) |
| susp | 7,7 / 9,4e-6 (0,01 %) | 0 |
| sett (máx. 294) | 8,1 / 3,2e-5 (1,36 %) | 1,37 / 3,7e-5 (1,36 %) |
| vel_x / vel_y | 3,4e-5 / 2,8e-5 (≈4,8 %) | 3,4e-5 / 9,3e-6 (≈4,8 %) |
| susp_rgb / sett_rgb | **inf** (1,0–1,6 % das células ≠) | **inf**, igual |

**Notas**
- No f32 aparece cor **infinita**: `inv = 1/want` estoura quando `want` é subnormal em f32 (em f64 o número cabe). É um defeito da variante f32, não ruído de arredondamento.
- No v3 os bits não mudam. O binário tem cerca de 1 300 instruções VEX: o código v3 foi mesmo gerado, porque o resto do binário é v2 e não usa VEX. Há só 12 `vfmadd`, vindas de `mul_add` da crate, e essas são exatas nos dois caminhos. O Rust não contrai `a*b+c` sem pedido, por isso o FMA não entra sozinho.
- A rota série da cópia `k64` sai 8 % mais lenta que a original (35,67 × 32,92 ms). A diferença vem dos meus caminhantes de linhas com `#[inline(always)]`. Por isso compare v3 e f32 com a **cópia** `k64`, que é o mesmo fonte, e não com a original.
- O v3 só entra na rota **série**: as tarefas do rayon não herdam o `#[target_feature]`.
- As leis de cor (K–M, `assenta`/`levanta`, `velatura`, `alpha_of_mass`) e a amostragem do campo de fluxo continuam f64 também na variante f32.
- Chamar uma função v3 exige `unsafe`, que a workspace proíbe. O C.2 só compila com `--cfg sonda_v3 --cap-lints warn` nesta invocação. Sem essa opção o exemplo compila sob a proibição e corre só o C.1.

**Leitura**
1. f32 contra a cópia f64: advecção −1 % em paralelo e −3,9 % em série; secagem igual. O desvio fica em 1–5 % das células, e a cor dá infinito em cerca de 1,5 % delas.
2. v3 contra a cópia v2: advecção −8,5 % e secagem −1,4 % em série, com bits idênticos. Contra a original série a advecção é só −0,8 %.
3. Na rota paralela do produto (5,2 ms por advecção) nenhuma das duas sondas muda o relógio além do ruído.
