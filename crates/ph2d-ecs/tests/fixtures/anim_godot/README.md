# As pontas da animação: corridas do **Godot 4.7.2 (MIT)**, sem interface

Medido nesta máquina em 2026-11-07 (`/usr/bin/godot`, `4.7.2.stable.arch_linux.ed1daf0bf`). Só se correu o
binário, o fonte não foi lido (`CLAUDE.md §0.9`). Cada CSV leva no cabeçalho a versão, a data, o comando e o
relógio medido; o gate `the_anim_ends_match_godot` confere-os.

## Como se regera

```
cd crates/ph2d-ecs/tests/fixtures/anim_godot
PH2D_GPU=1 bash ../../../../../scripts/ph2d-run.sh sh -c '
  env DUR_MS=100 godot --headless --fixed-fps 40 --script oraculo.gd > d100_passo25ms.csv
  env DUR_MS=110 godot --headless --fixed-fps 40 --script oraculo.gd > d110_passo25ms.csv
  env DUR_MS=100 godot --headless --fixed-fps 80 --script oraculo.gd > d100_passo12_5ms.csv'
```

| Ficheiro | Duração do quadro | Passo | Para quê |
|---|---|---|---|
| `d100_passo25ms.csv` | 100 ms | 25 ms | o caso pedido; o relógio cai exatamente em cada troca (empate) |
| `d110_passo25ms.csv` | 110 ms | 25 ms | o relógio nunca cai numa troca: separa a regra do empate da lei |
| `d100_passo12_5ms.csv` | 100 ms | 12,5 ms | mostra quanto da virada do ping-pong depende do passo |

Colunas: `caso,no,tick,quadro,pos,tocando,fim_no_tick`. A linha `tick = k` é lida depois de o nó processar `k`
vezes; `pos` é `frame_progress` no `AnimatedSprite2D` e a posição em segundos no `AnimationPlayer` (`-1` parado);
`fim_no_tick` é o tique do sinal `animation_finished` (`-1` = ainda não).

## Os nós e o que é comparável

- **`AnimatedSprite2D` é o par comparável:** lista de quadros com uma duração, como o nosso. ⚠️ Ele TEM ping-pong
  nesta versão (`SpriteFrames.LOOP_PINGPONG`, censo no cabeçalho do CSV). Casos `as2d_once_n*`, `as2d_pingpong_n*`.
- **`AnimationPlayer` não é o mesmo objeto:** é uma trilha de chaves num tempo, e o resultado depende de como se
  autora. Casos `ap_<modo>_n<n>_len<passos>_<disc|cont>`, com `length = passos·d` e a trilha discreta ou contínua
  com `INTERPOLATION_NEAREST`. Numa animação de 4 quadros a 100 ms, 25 ms:

  | Arranjo | Corridas em tiques |
  |---|---|
  | `len3_disc` | `0x5 1x4 2x5 3x3 2x4 1x5 0x3 …`, período 600 ms, o único que casa com o nosso em ordem e período |
  | `len4_disc` | a ponta 3 fica `12` tiques (300 ms) |
  | `len3_cont` | a ponta 3 aparece `1` tique só, e o 0 fica dobrado |

  Fica fora da comparação passo a passo: até na grade sem empate o quadro dele troca 1 tique depois de
  `AnimatedSprite2D` (110 ms: `0x6` contra `0x5`). A causa não foi medida.

## O que foi medido no `AnimatedSprite2D`

**A ordem é a mesma do nosso:** `0 1 2 3 2 1 0 1 2 …`, a ponta aparece uma vez. Once pára no último quadro.

**O empate:** com `frame_progress` a `1,000000` o quadro ainda não trocou; troca no tique seguinte. O controlo com
número exato em binário (`DUR_MS=125`, `--fixed-fps 32`, passo `0,03125`) dá a mesma sequência, logo não é ruído de
vírgula flutuante. Sem empate (110 ms), Once casa com o nosso em todos os tiques.

**A virada do ping-pong custa tempo, e o quanto depende do passo.** Tiques de cada quadro na ponta:

| Passo | Ponta | Quadro do meio | Ciclo de 4 quadros | Nosso ciclo |
|---|---|---|---|---|
| 25 ms | `6` = 150 ms | `4` = 100 ms | 700 ms | 600 ms |
| 12,5 ms | `10` = 125 ms | `8` = 100 ms | 650 ms | 600 ms |
| 1 ms (`TICKS=1500 --fixed-fps 1000`, não gravado) | `101` = 101 ms | `100` = 100 ms | 602 ms | 600 ms |

⚠️ **`--fixed-fps` não basta:** o 1.º delta do processo não é o pedido (1/60 a 40 fps), e a 80 fps o delta variou
até `Engine.physics_jitter_fix = 0`. O `oraculo.gd` trata os dois; `delta_min`/`delta_max` vão no CSV.
