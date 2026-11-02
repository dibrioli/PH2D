# J13b P8 — o pen-down manda a água herdada pela dívida

Sobre o HEAD `643957ef7d` (o compositor da óptica ligado por omissão, [J13 §6](J13_fps_do_real_oil.md) «Aberto» 1).

## O mecanismo, medido

- O worker só tira o sujo do motor quando serve um pedido (`DoWorker::serve`, `offthread_otica.rs`). Entre esse momento e o
  pen-down a água anda: o sujo dos passos fica no motor (`painter.rs` `step_simulation` marca a caixa do fluido inteira).
- No pen-down o lote traz o motor para casa (`dab_route.rs` `bring_home`), deposita, e a composição de casa
  (`wetpaint_entrada_da_otica`, `composite.rs`) faz `take_dirty`: leva o sujo da água com o do traço.
- O sujo é **uma caixa só** (`Dirty::Rect`). Um traço longe da poça faz compor a UNIÃO das duas, maior que qualquer uma.
- **Há UMA composição pesada por traço**: a 1.ª. As seguintes só levam o traço. Medido no portão (192², Real Oil, traço a
  `y = 0,1`, poça a `y ≈ 0,4–0,6`), as células do motor por evento do 1.º quadro foram:

  | | Down | Move 1 | Move 2 | Move 3 |
  |---|---|---|---|---|
  | antes | **21 411** (render 29 568) | 1 722 | 1 634 | 1 935 |
  | depois | **1 089** (render 6 790) | 1 722 | 1 634 | 1 935 |

  O sujo herdado sozinho era 9 050 células.
- Além disso, a imagem que está no ar no pen-down toca a `uniao_local` (essa caixa grande) e é descartada para a dívida:
  é uma imagem a mais, mas sai do fio da tela.
- **Com o traço aberto a água não anda.** `sim_should_run` (`painter.rs:481`) é falso com `stroke_down`, salvo Blow ou Live
  Water. `hand_off_sim` (`offthread.rs:368`) também não entrega o motor ao worker com o traço aberto e sem Live Water. Logo
  a dívida só é servida depois do pen-up. Até lá a região herdada mostra a última imagem aplicada. Esse é o último estado
  composto, e o motor não simulou desde o pen-down: o portão confere que `sim_steps` não mudou.

## A mudança

- `otica_no_tique.rs:65` `WetSession::a_agua_herdada_vai_pela_divida`: `take_dirty` → `caixa_da_grade` → `otica.deve`.
- `dab_route.rs:114–122`: depois do `bring_home` do lote, chama-se o método quando três condições valem:
  - `wet_otica_fora()`;
  - a sessão não nasceu neste lote (o `Full` do nascimento tem regra própria);
  - `!composite_pendente` (não há sujo de traço por compor; o pendente do quadro é do traço).
- O sujo que o motor traz do worker é todo «não-traço»: água, factos e as pontas que pousam no fim. Mandá-lo pela dívida
  dá o mesmo resultado que deixá-lo no motor para o `serve`, porque o `serve` junta a dívida ao sujo com
  `merge_dirty_otica` antes do `take_dirty`.
- A mesma porta cobre o Live Water: com ele, o motor vai ao worker em cada quadro do traço, e cada lote que o traz deixa
  a água pela dívida.
- Com o interruptor desligado (`PH2D_WET_OTICA_FORA=0`), `fora` é falso e nada muda.

## Portões

`otica_pen_down_tests.rs` `o_pen_down_manda_a_agua_herdada_pela_divida` corre com a thread real e `forca_no_teste`, sobre
a cena do P6:
1. A água corre.
2. Faz-se um pedido e espera-se 150 ms sem tique.
3. A imagem no ar é recebida com o motor em casa. Assim nenhuma imagem descartada pelo traço cobre a região herdada.
4. Mede-se o sujo herdado e devolve-se ao motor.
5. Faz-se um traço curto no alto, longe da poça.

O que o portão exige:
- (b) custo: nenhuma composição de casa do 1.º quadro passa de `caixa_do_traco` (3 450 células).
- A água não anda com o traço aberto, e a dívida espera o pen-up.
- (a) ao byte: depois do pen-up assenta-se sem passo novo. A região herdada mudou na tela (3 155 píxeis, mostrados pela
  dívida), e a tela é igual à composição de casa da tela inteira.

| mutação | resultado |
|---|---|
| M1: não mover (`if false && agua_pela_divida`) | vermelho: a 1.ª composição levou 21 411 células |
| M2: tirar o sujo e largá-lo (sem `deve`) | vermelho: a tela difere da de casa em 5 920 bytes, e a região herdada não mudou (0 píxeis) |

A 1.ª versão do portão deixava a imagem no ar ser descartada pelo traço. A caixa dela cobria a região herdada e a M2
passava. Por isso a imagem é recebida antes do traço (passo 3).

## App (1024², Radeon, `PH2D_WETPAINT_SMOKE=3`, A B A B intercalados, load 5–14)

Coluna `max` (ms) por rodada 0–5:

| fase | antes 1 | depois 1 | antes 2 | depois 2 |
|---|---|---|---|---|
| dabber | 65,9 · 114,7 · 68,1 · 72,5 · 86,4 · 58,2 | 73,1 · 38,7 · 42,7 · 34,6 · 39,4 · 43,8 | 67,3 · 49,3 · 65,1 · 72,3 · 107,6 · 48,0 | 69,5 · 39,1 · 46,8 · 50,4 · 47,1 · 77,5 |
| wet drip | 119,9 · 36,5 · 77,8 · 85,7 · 86,8 · 105,8 | 35,2 · 73,8 · 44,1 · 37,9 · 42,6 · 50,1 | 39,9 · 66,8 · 50,1 · 75,4 · 105,5 · 114,9 | 34,9 · 48,2 · 55,0 · 47,1 · 64,4 · 38,2 |

Período p50 com as mãos paradas, em ms:

| fase | antes 1 | depois 1 | antes 2 | depois 2 |
|---|---|---|---|---|
| seca | 16,6–16,9 | 16,4–17,1 | 16,3–17,1 | 16,4–17,2 |
| corre | 16,6–17,1 | 16,6–17,0 | 16,6–17,0 | 16,6–17,0 |

Leitura das tabelas:
- Nas fases de traço, a mediana dos `max` cai de ~72 para ~45 ms nas rodadas 1–5, e o pico que crescia com a poça (105–115
  ms nas rodadas 4–5) sai.
- As paradas ficam como estavam, a 60 fps.
- **Fica um `max` de 35–78 ms** em cada fase de traço, e não é o sujo herdado (ver Aberto).

## Testes por nome, antes × depois

O «antes» é o binário com a M1, que se comporta como o HEAD. O «depois» tem a cura. `--test-threads=8`.

| filtro | antes | depois |
|---|---|---|
| `otica` | 8 ok · 1 ✗ (o portão novo) | 9 ok |
| `offthread` | 13 ok | 13 ok |
| `por_quadro` | 16 ok | 16 ok |
| `showwet` | 2 ok | 2 ok |
| `undo_tinta_viva` | 9 ok · 3 ign | 9 ok · 3 ign |
| `smooth_edges` | 15 ok | 15 ok |
| `oleo_cache_tests` | 2 ok | 2 ok |
| `macro` | 94 ok · 1 ✗ (`o_splash_tocado…`, conhecido) | o mesmo |

Comparadas linha a linha (`diff` das listas ordenadas de `test … ok/FAILED`), a única diferença é o portão novo, que passa
de vermelho a verde. Os `ignored` com razão não entram na contagem. `cargo check -p ph2d-tool-painter --profile smoke`,
com e sem `--tests`, dá zero avisos.

## Aberto

1. O `max` de 35–78 ms que fica nas fases de traço, por medir. Suspeitos:
   - o `preset_aplicar` do 1.º quadro da fase;
   - o laço de píxel da imagem aplicada no fio da tela (`wet_aplica_imagem`), que corre sobre a caixa inteira da água;
   - a imagem descartada a mais no pen-down.
2. A imagem no ar no pen-down ainda é descartada quando a caixa crescida dela toca a do traço: o alcance do óleo é ~32
   células. Continua a ser o item 2 do J13 §6 (carimbar a cópia).
3. Outras portas de casa ainda levam o sujo da água para a composição local: a varredura do painel (`oleo_no_quadro.rs`),
   o Smooth Edges, o undo e o fim de sessão. Nenhuma está no caminho do traço.
