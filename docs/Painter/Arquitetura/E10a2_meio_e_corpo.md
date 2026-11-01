# E10 A2 — o meio e o corpo viajam na célula (01/11)

Spec: `F3_controlos.md` §8–§9 · `F3b_medicoes.md` §(1) Body · `RELEVO_oleo_ondas_e_crateras.md` §6.

## O que mudou (`wp/` = `crates/ph2d-wet-paint/src/`)

| ficheiro | mudança |
|---|---|
| `wp/grid/oleo_celula.rs` | `OleoCelula` ganha `meio` (`Knob::Medium`) e `corpo` (`Knob::Body`), 16 → 24 B; misturam-se pela massa como o resto (`pousa`). `do_pincel` lê-os com ou sem Real Oil ⇒ Medium > 0 ou Body > 0 **aloca** o campo sem Real Oil. `oleo` fica SÓ a luz/relevo (`ganho = Depth × Real Oil`). `restaura_janela` só larga o campo se meio e corpo também são 0. |
| `wp/solver/ladeira_do_oleo.rs` | `tensao(…, m, …)`: `m` = o MEIO da célula (era o Real Oil). Nova `tensao_com_corpo` + `TAU_CORPO = 0.25` + régua `PAPEL_CORPO` (bits f32; `0` = o produto). |
| `wp/solver/flow_jacobi.rs` | a única rota com Bingham: lê `cel.meio`/`cel.corpo` por `tensao_com_corpo`, com o campo presente (`τ₀ ≤ 0` deixa só o corpo). Parede retirada. |
| `wp/solver.rs` | `parede_do_traco` apagada; `build_flow_field` e `smooth_velocity` sem parede. `ladeira_do_oleo` passa a `pub(crate) mod` (teste em `grid/`); re-exporta `PAPEL_CORPO`. |
| `wp/trail/transfer.rs` | não escreve mais `g.traco` (a lei `LevelingOwnStroke` não é lida); o depósito Pigmento leva `meio: Knob::Medium`. |
| `wp/trail/transfer/deposito_tinta.rs` | doc do `meio`. |
| `wp/grid/traco.rs` | `separa` e o seu teste apagados; doc diz que o campo fica vazio. |
| `wp/solver/bingham_tests.rs` | a cena do óleo põe `Medium = 1` (o `m` é o meio agora). |
| `wp/grid/oleo_celula_tests.rs` | literais com `meio`/`corpo`; tamanho 24 B. |
| `wp/grid/meio_corpo_tests.rs` (novo) | os 3 testes abaixo. |
| `crates/ph2d-wet-paint/tests/f3_medicoes.rs` | `f3_body_dois_tracos` varre `Knob::Body` (`PH2D_F3_BODY=0,0.1,…`) em vez do lev 1/0. |

## A lei

`τ_y = τ₀·max_v·m·((φ−φ_c)/(1−φ_c))₊² + TAU_CORPO·max_v·corpo` (o 2.º termo só com `massa = susp > 0`; não precisa
de `φ > φ_c`). `corpo = 0` ⇒ a tensão do meio ao bit (teste `sem_meio_sem_corpo_e_o_de_sempre`).

## Medição do Body (`f3_body_dois_tracos`, lei Pigmento, Engine directo, 1 processo)

Presets dos `.ph2d-brush.png` (Wet Liner / Wet Wash / Gouache Opaque sem `size_px` no ficheiro ⇒ 29 px; os 2 óleos com
`medium=1` acrescentado, como fará a migração). Valores em unidades de `TAU_CORPO = 1` (a medição); a coluna «Body
(produto)» é o mesmo ÷ 0,25. **fuga** = massa fora das duas faixas ÷ massa do 1.º; **borda** p99 µm.

| τ·max_v (Body produto) | Wet Liner fuga / borda | Wet Wash fuga / borda | Gouache Opaque fuga / borda |
|---|---|---|---|
| 0 (0) | 2,10 % / 2,62 | **44,3 %** / 1,84 | 0,01 % / 2,71 |
| 0,002 (0,008) | 0,53 % / 2,81 | 40,4 % / 2,07 | 0 / 2,56 |
| 0,005 (0,02) | 0 / 3,12 | 33,5 % / 2,35 | 0 / 2,46 |
| 0,01 (0,04) | 0 / 3,71 | 24,4 % / 2,67 | 0 / 2,51 |
| 0,02 (0,08) | 0 / 3,79 | 10,4 % / 3,40 | 0 / 2,11 |
| **0,04 (0,16 = B₁)** | 0 / 1,91 | **0,12 %** / 4,44 | 0 / 0,80 |
| 0,07 (0,28) | 0 / 0,74 | 0 / 4,30 | 0 / 0,75 |
| 0,1 (0,4) | 0 / 0,73 | 0 / 3,57 | 0 / 0,74 |
| 0,25 … 4 (1 … 16) | 0 / 0,66 | 0 / 3,57 | 0 / 0,74 — **iguais à 3.ª casa de 0,25 a 4** |

Seco (o 1.º traço seco): a fuga é ≤ 0,01 % em todos os Body; o corpo só engrossa o cruzamento (cruz p99 2,6 → 4,6 µm
no Wet Liner) porque a tinta deixa de espalhar. Óleos de controlo (Filbert, Thick Impasto, `medium=1`): **idênticos ao
bit em todos os Body** (o óleo não escoa — o meio já o segura, F3b R1).

- **B₁ = 0,16** (produto, `TAU_CORPO = 0,25`): fuga ≤ 0,12 % nos 3 de água, borda p99 0,8–4,4 µm (contra os 12–39 µm
  da parede binária, Bug #56). O Wet Wash fica 1 µm acima do alvo de 3 µm na borda (a 0,07·max_v também; é a tinta que
  deixa de espalhar, não um degrau) — ⚑ a confirmar no olho do dono.
- **Faixa 0..1 (produto)**: Body 1 = `τ = 0,25·MaxVelocity`, a partir de onde o fluido **já não se move** (as
  leituras são iguais de 0,25 a 4·max_v): o recurso que acaba a faixa é a física — `τ_y` maior que todo incremento de
  velocidade que o passe produz (o nivelamento é presa a `LevelClamp`, a velocidade a `MaxVelocity`). Acima disso o
  controlo é morto.

## Testes (por nome)

| teste | antes | depois |
|---|---|---|
| `o_meio_viaja_do_controlo_ao_deposito` | não existia | ✓ |
| `o_corpo_viaja_na_celula_e_segura_a_tinta` | não existia | ✓ |
| `sem_meio_sem_corpo_e_o_de_sempre` | não existia | ✓ |
| `oleo_celula` (3 antigos) | ✓ | ✓ (tamanho 16 → 24 B) |
| `bingham` (`o_oleo_grosso_com_pouca_agua_nao_escorre_fora_do_pincel`) | ✓ | ✓ (com `Medium = 1`) |
| `advect_jacobi` (hashes fixos) | ✓ | ✓ — neutro ao bit |
| `o_invariante_soma_massa` | ✓ | ✓ |

## Aberto

1. `Knob::Body` em `wp/tuning/defs.rs`: faixa **0..1** (já é), passo 0,01, default 0; doc `TAU_CORPO = 0,25`. (A1/coord.)
2. Migração (B1): `levelingOwnStroke = 1` → `body = 0.16` nos 3 de água; óleos → `medium = 1`, `body = 0`.
3. O campo `Grid::traco` e `Sim/Params::{traco_actual, paredes}` ficam vivos mas sem leitor/escritor (vazio, custo 0):
   tirá-los toca `grid.rs`, `grid/patch.rs`, `grid/snapshot.rs`, `grid/canvas_ops.rs`, `painter/instantaneo.rs`,
   `painter.rs:488`, `painter/stage.rs:30`, `painter/historia_tests.rs`, `sim.rs`, `painter/doors.rs` e testes do
   `tp/` — fora do meu caminho. `Knob::LevelingOwnStroke` também (o coord).
4. `wp/trail/bico_tinta.rs:94` ainda faz `com_meio(p.oleo)` (o pick-up da ponta na lei Pigmento) — deve ler
   `Knob::Medium` como o depósito; fora do meu caminho.
5. `build_flow_field` (rota não-Jacobi, `order_invariant = false`) nunca teve Bingham — continua sem meio e sem corpo.
6. Preset com Real Oil e Medium 0 perde o Bingham (intencional até a migração B1).
