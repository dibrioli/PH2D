# ADR-0175 — O depósito de um dab do Wet Paint corre em linhas disjuntas, e o bico do transfer também

- **Status:** Accepted
- **Data:** 2026-09-24
- **Linha:** `line/PainterWatercolor`
- **Amparo:** ADR-0145 (a 2.ª excepção ao «sem `rayon`», a desta crate, que manda que **todo uso
  novo** de `rayon` no `ph2d-wet-paint` tenha ADR próprio) · ADR-0109 (os TRÊS invariantes) ·
  ADR-0147 (o solver independente de ordem, que já corre em linhas)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui.

## Contexto

Ordem do dono (2026-09-23): *«avaliar seriamente os modos watercolor e wet paint e descobrir se com o
que aprendemos podemos melhorar a performance»*; a aquarela fechou no ADR-0173, e em 2026-09-24 o
dono mandou seguir.

**A régua é nova e é do PRODUTO:** `crates/ph2d-tool-painter/examples/mede_o_wet_paint.rs`. Todas as
sondas `measure_wetpaint_*` do Painter correm sob `cfg(test)`, onde o journal do undo está vivo — e o
composite do Wet Paint passa `area: None`, logo ali cada composite fotografa a TELA INTEIRA, o que o
app em release nunca faz. O exemplo entra só pelas portas do app, com o ritmo real (16 eventos por
quadro, espera pelo vsync a sério, porque a simulação corre numa thread própria a perseguir o relógio
de parede). E o `wet_diag` ganhou a metade do GESTO (`note_stamp`/`take_stamp`): com a caneta em
baixo a simulação está PARADA, e o log do produto lia zero exactamente onde o artista sentia o peso.

Medido (tela 4096², máquina calma): **pintar** é que pesa; depois de largar o pincel a água já corre
aos `40` passos por segundo nominais, com `~2 ms` por quadro.

| raio | quadro a pintar | dabs/quadro | depósito por dab |
|---|---|---|---|
| 100 | `~12,5 ms` | `~12` | `0,90 ms` |
| 250 | `~31 ms` (≈33 FPS) | `5` | `5,7 ms` |
| 400 | `~45 ms` (≈22 FPS) | `~3` | — |

O perfil (`amostra_gdb.py`, raio 250) punha **`53 %`** das amostras no laço do depósito
(`accumulate_paint_impl` → a silhueta do hospedeiro por célula, `~23 ns`) e `~15 %` no
`transfer_paint`, **tudo na thread principal**, com o pool do motor parado ao lado.

## Decisão

**1. O depósito de um dab do produto caminha as linhas do dab em série ou em paralelo**
(`Trail::deposit_shaped`, `crates/ph2d-wet-paint/src/trail/deposit.rs`), escolhido por
`Rows::pick` com o piso medido `MIN_CELLS_DEPOSIT`. As três condições do ADR-0109 valem por
construção:

1. cada linha escreve **só a própria linha** — o carimbo de umidade na linha do grid e o pigmento/água
   na linha da janela do rasto; cada célula é visitada uma vez por dab;
2. o que ela lê (`susp`, `sett`, `paper`, `film`, a silhueta, o grão), **ninguém escreve** neste passe;
3. a única redução é de **caixas envolventes** (a extensão da janela e o `wrote`): mínimos e máximos
   inteiros, associativos **e** comutativos.

A lei da célula é **UMA função** (`DepositLaw::cell`) com dois chamadores — este caminho e o do
próprio motor em série, que a impressão digital da sessão fixa ao bit —, e a amostragem de uma linha
também (`brush::stamp_row_shaped`, com `for_each_stamp_pixel_shaped` a ser ela num laço).

**2. A silhueta e o grão do hospedeiro passam de `&mut dyn FnMut` a `&(dyn Fn + Sync)`**
(`brush::CellFn`). Sempre foram funções puras da célula; o `FnMut` só não o dizia, e um hospedeiro que
guardasse estado nelas passa a ser **erro de compilação** em vez de uma corrida. Os chamadores
(`dab_route.rs` no Painter, a borracha, as portas do motor) só perderam um `mut`.

**3. Os passos 1–2 do `transfer_paint` — o BICO (a auto-limpeza da janela inteira e a recolha da cor
do canvas nas linhas tocadas) — correm por linhas** (`Trail::tip_rows`), num corpo por linha que faz
os dois pela ordem de sempre, com o piso `MIN_CELLS_TIP`. ⛔ **Os passos 3 e 4 ficam em série, e não
por gosto:** o 3 é uma SOMA em `f64` (a ordem muda os bits) e o arrasto do 4 lê `susp[si]`/`sett[si]`/
`film[si]` de células que o mesmo laço pode já ter escrito — é Gauss-Seidel, e trocá-lo por Jacobi
mudaria a tinta (a mesma fronteira que o ADR-0147 só atravessou com decisão de produto).

**3-bis. O PAPEL no nascimento da sessão também corre por linhas** (acrescentado 2026-09-24, mesma
linha): o bake do tile do motor na grade (`paper::bake_paper_rows`) e a semente do papel do HOSPEDEIRO
(`Engine::seed_paper_with_rows`). As três condições do ADR-0109 valem por construção — cada linha
escreve só a própria linha do `paper`, lê só o tile (que ninguém escreve) ou chama a lei do hospedeiro
com as células dela, e não há redução nenhuma. A lei do hospedeiro passa de `&mut dyn FnMut` a
`&(dyn Fn + Sync)` pela mesma razão do ponto 2. **Medido no produto** (`mede_o_wet_paint pousos … [papel]`,
4096²): o 1.º traço de uma sessão com um papel do artista era **`126–308 ms`** (a semente em série,
16,7 M chamadas) e passa a **`28–43 ms`**; sem papel, `~30 → ~18 ms`. Pisos em `par.rs`
(`MIN_CELLS_PAPER_BAKE` · `MIN_CELLS_PAPER_SEED`, com a tabela de `tests/it/measure_paper_rows.rs`);
gates em `tests/it/paper_rows.rs` (as duas rotas ao bit, o laço de antes congelado como oráculo, e o
corte à faixa do dente), **6 de 6 mutações a sangrar**. ⛔ **O tile (`generate_paper_tile`, `~12 ms`)
fica em série:** a caminhada das fibras consome o gerador aleatório em ordem e as somas são `f64` de
ordem fixa — é a impressão digital do motor.

### Os pisos, medidos (`tests/it/measure_deposit_rows.rs`, release, `load 1,55`, as duas rotas ALTERNADAS na mesma ronda, mínimo de 15)

```text
 raio   celulas |  dep ser   dep par  razao
    8       289 |    0,002     0,019  0,11x
   24     2_401 |    0,015     0,026  0,56x   <- ultimo em prejuizo
   40     6_561 |    0,048     0,035  1,38x   <- primeiro em ganho
   64    16_641 |    0,159     0,039  4,05x
  100    40_401 |    0,396     0,080  4,97x
  250   251_001 |    2,444     0,312  7,84x
  400   641_601 |    6,294     0,614 10,25x
```

⇒ `MIN_CELLS_DEPOSIT = 6 × 1024`. É **muito** mais baixo que os pisos do solver (`128`–`512 k`): a
célula aqui avalia a silhueta do hospedeiro e a lei do depósito (`~10 ns`), contra `~2 ns` de um passo
de Jacobi, logo o pool paga-se com menos delas. O bico é mais barato por célula (três lerps; a recolha
salta o papel em branco): prejuízo com a janela do raio 40 (`8 649` células), ganho com a do raio 64
(`19 881`) ⇒ `MIN_CELLS_TIP = 16 × 1024`.

## Consequências

**No produto** (`mede_o_wet_paint`, tela 4096², máquina calma, quadro a pintar):

| raio | antes | depois |
|---|---|---|
| 100 | `~12,5 ms` | **`~3,8 ms`** |
| 250 | `~31 ms` | **`~8 ms`** |
| 400 | `~45 ms` | **`~10,5 ms`** |

A imagem é **a mesma ao bit** — não há tolerância em lado nenhum:

- `tests/it/deposit_rows.rs`: o mesmo traço pelas duas rotas forçadas (com a cerda do motor e com o
  grão do hospedeiro) deixa o grid inteiro, a janela, o bico, as extensões e os retângulos declarados
  iguais byte a byte; e — o oráculo **independente** — a caminhada por linhas com uma silhueta que
  reproduz a queda do motor pousa o mesmo traço que o `accumulate_paint` do próprio motor, que nunca
  passou pelo código novo; e o bico de cada transfer bate os laços de ANTES, congelados no teste;
- a impressão digital da sessão do motor (`tests/it/fingerprint.rs`) inalterada.

⚠️ **A fixtura contém o fenómeno, e isso é afirmado** (`a_fixtura_contem_o_fenomeno`): a caixa passa o
piso, o traço pousa tinta, toca os QUATRO limites do canvas, cai fora da janela, faz duas transferências
e o bico recolhe cor. Cada uma dessas premissas foi escrita por uma mutação que **sobreviveu** à 1.ª
redacção (a caixa só recortada à direita; a recolha sobre papel em branco e sem tinta assente; uma
transferência só, com o bico ainda limpo). **12 de 12 mutações sangram** no fim.

**O que sobra no quadro a pintar** (raio 250, `~8 ms`): os passos 3–4 do transfer (`~1,9 ms` por
transfer, em série, e há ~2 por quadro) e o composite do carimbo (`~0,4 ms` por entrega). Os dois são
Gauss-Seidel ou somas de ordem fixa — o próximo degrau não é paralelismo byte-idêntico.

⚠️ **O pen-down continua caro** (`~28–48 ms` no início de cada traço) e não foi tocado por esta wave.
