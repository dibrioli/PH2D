# DIÁRIO DA LINHA — `line/PainterWatercolor`, 2026-09-20 → 2026-09-25

> ⚠️ **O documento do INTEGRADOR não é este:** é o
> [`HANDOFF_INTEGRACAO_line_PainterWatercolor_A_LINHA_2026-09-25.md`](HANDOFF_INTEGRACAO_line_PainterWatercolor_A_LINHA_2026-09-25.md)
> (superfície de colisão, contadores, ADRs a recontar, prova de fecho, smokes).
>
> Este é o **diário** da linha, uma secção por wave. As §1–§38 (a costura do retorno, a lei da tinta,
> a pilha do Composite Brush, o Blur, a aquarela, 20/09 → 24/09) foram **arquivadas VERBATIM** em
> [`docs/archive/docs-2026-09-24/painter/`](../../archive/docs-2026-09-24/painter/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md)
> pelo `doc-split.py` (sha256 da remontagem conferido; o doc tinha 207 KB). Aqui ficam as §39–§43, o
> Wet Paint de 24/09.
>
> ### ⛔ Recusas MEDIDAS arquivadas — leia antes de propor uma otimização ou um desenho no Painter
>
> | § | recusa | linha no arquivo |
> |---|---|---|
> | §15 | o **Self Pickup** (o pincel esgotado re-captar a própria tinta) — construído e RETIRADO por veredito do dono: a *distribuição da carga* estava errada, não a cerca | §15 |
> | §16 | um raio PRÓPRIO para a cor por-dono — construído e retirado (os miolos saem byte-idênticos a `2·4·8`) | 541 |
> | §17 | o K–M na AQUARELA — construído, medido e revertido (contra a base de papel o `K/S` satura e o knob deixa de modular) | 628 |
> | §22 | a tabela de preço da recomposição por camada (`load 29`) — RETIRADA, a válida é a §22.6-bis | 1362 |
> | §27 | um limiar sobre o COMPRIMENTO do salto para partir sub-figuras — recusado por mecanismo | 1924 |
> | §28 | a alocação dos intermédios da pilha — ILIBADA (`0,00 ms`) | 1978 |
> | §28 | o cover por BLOCOS da pilha — a família inteira, pior que a caixa envolvente | 1984 |
> | §31 | as três hipóteses com que a auditoria do watercolor abriu — REFUTADAS | 2268 |
> | §33 | a morte da premissa do `uma_banda_nunca_fica_abaixo_do_piso` — declarada de manhã, desmentida | 2446 |
> | §34 | *«um rato de 1 kHz manda mil composições por segundo»* — caiu na 1.ª coluna | 2775 |
> | §38 | as recusas medidas do Pigment molhado sobre molhado | 3031 |

---

## §39 — O Wet Paint: pintar com raio 250 passa de ~31 para ~8 ms por quadro, ao bit (2026-09-24)

Ordem do dono: *«smoke OK. Siga»* — a 2.ª metade da ordem de 23/09 (*«avaliar seriamente os modos
watercolor e wet paint»*). Mecanismo, tabela dos pisos e consequências:
[ADR-0175](../../architecture/decisions/0175-o-deposito-do-dab-do-wet-paint-corre-em-linhas-disjuntas.md).

### §39.1 — A régua do PRODUTO não existia

Todas as sondas `measure_wetpaint_*` correm sob `cfg(test)`, onde o composite do Wet Paint (`area:
None`) fotografa a tela inteira para o undo — o app não. ⇒ `examples/mede_o_wet_paint.rs` (portas do
app, 16 eventos por quadro, vsync a sério porque a sim persegue o relógio de parede) e a metade do
GESTO no `wet_diag` (`note_stamp`/`take_stamp`): com a caneta em baixo a sim está parada e o log lia
zero onde o artista sentia o peso. Medido: depois de largar o pincel a água já corre aos `40` passos/s
com `~2 ms` por quadro; **pintar** é que pesa — raio 250 `~31 ms` (5 dabs × `5,7 ms`), raio 400 `~45`.

### §39.2 — A cura: o depósito e o bico por linhas (`ph2d-wet-paint`)

`trail/deposit.rs` (a lei da célula numa função, `DepositLaw::cell`, e a caminhada por linhas) +
`brush::stamp_row_shaped`/`shaped_bounds` + `Trail::tip_rows` (a limpeza e a recolha do bico, os passos
1–2 do transfer). A silhueta e o grão do hospedeiro passam a `brush::CellFn = &(dyn Fn + Sync)`. Pisos
MEDIDOS: `MIN_CELLS_DEPOSIT = 6 k`, `MIN_CELLS_TIP = 16 k` (`tests/it/measure_deposit_rows.rs`).

| raio | antes | depois |
|---|---|---|
| 100 | `~12,5 ms` | `~3,8 ms` |
| 250 | `~31 ms` | `~8 ms` |
| 400 | `~45 ms` | `~10,5 ms` |

Gates (`tests/it/deposit_rows.rs`): as duas rotas forçadas byte a byte (cerda e grão) · o **oráculo
independente** — o caminho do próprio motor, com uma silhueta que reproduz a queda dele, pousa o mesmo
traço — · o bico de cada transfer contra os laços de ANTES, **congelados no teste** · e as premissas.
**12 de 12 mutações sangram**; três sobreviveram à 1.ª redacção e cada uma escreveu uma premissa
(a caixa só recortada à direita; a recolha sobre papel sem tinta assente; uma só transferência, com o
bico ainda limpo). ⚠️ A mutação «saltar a coluna 0 da limpeza» é **equivalente**: a coluna 0 da janela
nunca fica suja (o dab cabe dentro da meia-largura), e foi trocada por «saltar meia linha».
`nextest-impacted` `15 707/15 707`, clippy `-D warnings` limpo, a impressão digital da sessão do motor
inalterada.

### §39.3 — Aberto

- **Os passos 3–4 do transfer** (`~1,9 ms` cada, `~2` por quadro a raio 250) ficam em série: o 3 é uma
  soma `f64` de ordem fixa e o arrasto do 4 é Gauss-Seidel. Mudá-los muda a tinta ⇒ decisão do dono.
- **O pen-down** custa `~28–48 ms` no início de cada traço (o 1.º da sessão cria o grid de `~944 MB`);
  não medido por dentro ainda.
- A M7 original fica registada como mutante equivalente, não como gate em falta.

## §40 — O pen-down do Wet Paint: a absorção do escorrido deixa de copiar a tela (2026-09-24)

Ordem do dono (*«smoke ok. siga»*, depois do §39). O item aberto era o **pen-down** (`~25 ms` em cada
traço que começa com a água ainda a correr).

### §40.1 — A régua e a atribuição

`examples/mede_o_wet_paint.rs` ganhou o modo **`pousos [n]`**: traços curtos em sequência com meio
segundo de água a correr entre eles, e o pen-down de cada um impresso (o 1.º à parte — ele cria o grid).
Instrumentado por dentro (marcadores temporários, retirados antes do commit), a 4096² o pen-down era:

| parte | ms (load ~24) |
|---|---|
| detector da absorção (`PlaneDeltas::split` cursor × before, tela inteira, paralelo) | `3,4–7,9` |
| **materialização** do `before` do topo (cópia de 67 MB + blit) | `3,7–4,6` |
| **re-split** (varre os 67 MB outra vez para achar a janela) | `4,9–6,8` |
| 1.º carimbo (espera do `bring_home` + depósito + composite com o fork do canvas) | `7–10` |

A absorção existe desde 26/07 (o escorrido do Wet Paint fica sem dono no undo sem ela) e é
**correcta**; o preço é que ela redescobria, com dois passes de tela inteira, uma janela que o topo e o
detector **já conheciam**.

### §40.2 — A cura: o canvas do topo re-parte-se só dentro da caixa `U` (`undo_delta_absorb.rs`)

`StoredPlane::absorbed(cursor, after, drip, stride)`: com `W` a janela do topo e `D` a janela EXACTA do
detector, fora de `U = W ∪ D` os dois lados do re-split são iguais byte a byte (fora de `W` o `before`
materializado é o cursor; fora de `D` o cursor é o `after`), logo a janela exacta está dentro de `U` e
o **mesmo `diff_window`** sobre o recorte dá a mesma caixa. A absorção materializa o topo **sem** o
canvas, dá ao `split` o mesmo `Arc` dos dois lados (ele não varre) e escreve o plano calculado.
**Os outros 18 planos seguem pela porta de sempre.** Recusa (e corre o caminho caro inteiro) onde a
igualdade não é demonstrável barato: um lado `Whole`, tamanhos ou stride diferentes, ou `U` com meio
plano ou mais — aí o caro pode guardar `Whole`.

Medido A/B alternado (binário com o caminho barato desligado contra o que shipa, mesma carga,
`pousos 24`, raio 100, tela 4096²; ⚠️ a máquina esteve a `load 5–23` — outras linhas a correr suítes —
e não houve janela calma):

| corrida | antes p50 / p90 | depois p50 / p90 |
|---|---|---|
| 1 | `26,6 / 46,7 ms` | `21,1 / 35,3 ms` |
| 2 | `29,5 / 38,3 ms` | `16,6 / 22,6 ms` |
| 3 | `34,6 / 51,9 ms` | `15,6 / 18,2 ms` |

Gates (`undo_absorb_tests.rs`, filho de `undo_tests`): cada cenário corre **duas vezes** — uma com
`ONLY_THE_FULL_PATH` — e compara a pilha de undo INTEIRA (Debug), o cursor e o livro de bytes; e afirma
que o caminho barato **correu** (`CHEAP_FIRED`), senão a igualdade seria entre dois caros. Cenários: o
escorrido longe / a encostar / dentro do traço · o que devolve parte do traço ao fundo (a janela
exacta encolhe) · o que apaga o traço inteiro · o topo que não mexeu no canvas · o topo gravado com
janela DECLARADA maior que a exacta · o run coalescido · a caixa de meio plano (o barato tem de
recusar) · e desfazer/refazer através de uma absorção barata. **6 de 6 mutações de lei sangram**; a 7.ª
(não igualar o `Arc` antes do `split`) **sobrevive de propósito e está NOMEADA**: ela só devolve o
custo (o `split` volta a varrer e a entrada é sobrescrita na linha seguinte), que é o que a tabela
acima mede. Suíte do Painter `1347` + `it` verdes em debug, clippy `-D warnings` limpo.

### §40.3 — Aberto

- **O detector** continua a varrer a tela inteira (`3–8 ms`): é ele que responde *«alguém escreveu fora
  da história?»*, e a resposta só é segura sem janela informada (o cabeçalho do `undo_absorb` explica
  porquê). Dar-lhe a região suja da simulação seria um atalho que não falha alto — não feito.
- **O 1.º carimbo** (`7–10 ms`) inclui o fork do canvas (o undo segura o `Arc`) — não atacado.
- **O 1.º traço da sessão** (`~40–60 ms`) cria o grid da simulação; é uma vez por sessão.
- Flake de carga vista uma vez nesta corrida e verde na seguinte:
  `mask_gate_tests::the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (gate de razão,
  `load ~22`, zero linhas deste diff no módulo dele).

## §41 — O composite do Wet Paint DECLARA onde escreve: o commit e o detector deixam de varrer a tela (2026-09-24)

Ordem do dono: *«smoke ok. siga»* (o §40 aprovado). O item aberto era o **detector** da absorção
(`3–8 ms` por pen-down, a tela inteira).

### §41.1 — A causa: o único escritor de canvas do Wet Paint não declarava

Sonda no `absorb_foreign_writes` (retirada): a cada pen-down `hint_for(...) = None` com
**`16–26` acessos de escrita abertos sem declaração** — todos do `wetpaint_composite_veiled`, que abre o
canvas pelo `fork_canvas` (acesso não-declarado por construção, `plane_fork.rs`) e nunca dizia onde
escreveu. Consequência dupla: o **commit do traço** (pen-up) varria a tela para derivar a janela, e o
**detector da absorção** também. As duas passadas que escrevem (composite e véu) só tocam
`px0..px1 × py0..py1`, que é a região que o `mark_dirty` já publicava.

### §41.2 — A cura, em duas metades

1. **`self.declare_wrote(Some(region))`** no fim do composite (a porta de `stamp_preview.rs`). Com ela o
   commit do Wet Paint passa a ser declarado (`record_structural_hinted`), com a rede de DEBUG do
   `split` a conferir que a janela verdadeira cabe na declarada. ⭐ **A rede foi provada:** declarar
   `1 × 1` reprova **seis** gates do `wetpaint` com *«a janela declarada nao contem a verdadeira»*.
2. **O detector lê o canvas só dentro da janela declarada** (`StoredPlane::split_exact_within`, em
   `undo_delta_absorb.rs`): a janela acumula desde o último commit, logo é um superconjunto de tudo o que
   mudou desde o cursor quando `hint_for(cursor.writes)` a oferece, e o mesmo `diff_window` sobre o
   recorte dá a janela EXACTA. ⚠️ **Não é o `split` com dica:** esse guardaria a declarada tal como
   veio, e uma janela declarada sobre bytes iguais (outro `Arc`, mesmo conteúdo) faria a absorção
   disparar onde o caminho de sempre não dispara. Meio plano ou mais, ou janela que não serve ao plano
   ⇒ o `split` de sempre. Em DEBUG a janela verdadeira tem de caber na declarada.

### §41.3 — Medido (A/B alternado, `pousos 24`, raio 100, 4096², `load 5–9`)

| | sem declarar | declarando |
|---|---|---|
| pen-down p50 | `11,6 / 12,5 / 13,3 ms` | **`9,2 / 9,8 / 8,9 ms`** |
| pen-down p90 | `13,6 / 14,4 / 15,2 ms` | **`12,7 / 12,0 / 11,1 ms`** |
| **pen-up** p50 | `6,8 / 6,4 / 6,4 ms` | **`3,6 / 4,1 / 3,8 ms`** |

(O `pousos` passou a imprimir o **pen-up** também — ele é o commit.) Somado ao §40, o pen-down de um
traço sobre água a correr foi de `~27–35 ms` (sob carga) para `~9 ms`.

### §41.4 — Gates e provas

`undo_absorb_tests.rs` ganhou cinco: a mesma entrada com a janela justa e com uma folgada · uma janela
declarada sobre bytes iguais **não** faz a absorção disparar · a de meio plano cai no detector de
sempre (com um escorrido de mais de meio plano, onde o de sempre guarda `Whole`) · a janela mais NOVA
que o cursor não é lida (a proveniência do `hint_for`) · e a rede de DEBUG (`should_panic`) com uma
janela que não contém o escorrido. `DETECTED_WITHIN` prova que o detector de dentro correu (o
`split_exact_within` devolve se a resposta saiu da janela; contar na chamada contaria recusas). **6 de 6
mutações sangram** — a da proveniência **sobreviveu** à 1.ª redacção e escreveu o último gate. A linha
que igualava os `Arc` do canvas antes do `split` dos outros planos saiu: era inerte (o plano do canvas é
sobrescrito a seguir). Suíte do Painter `1352` verdes; duas flakes de razão conhecidas
(`the_mask_stroke_cost_does_not_follow_the_canvas`, já na lista do §5.0, e
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`) reprovaram no fan-out e passam 3/3
sozinhas a `load ~7`. Clippy `-D warnings` limpo.

### §41.5 — Aberto

- **O resto do pen-down (`~9 ms`)** é o 1.º carimbo, e dentro dele o **fork do canvas**: o cursor do undo
  segura o `Arc`, logo a 1.ª escrita depois de um commit copia os 67 MB. É o preço do undo por snapshot
  com canvas inteiro; curá-lo pede canvas em ladrilhos — arquitectura, não wave.
- **O 1.º traço da sessão** (`~40 ms`) cria o grid da simulação.

## §42 — O 1.º traço da sessão: a folha inteira deixa de ser recomposta, e o §41 deixa de guardar dois planos (2026-09-24)

Ordem do dono: *«smoke ok. siga»* (o §41 aprovado).

### §42.1 — O que a sonda mostrou, e o defeito que o §41 tinha trazido

Sonda temporária no 1.º traço (4096², raio 100): `ensure_wet_session` `27 ms` = **tile do papel `12 ms`
+ bake do papel na grade `18 ms`** (o `Grid` nasce em `0,06 ms`: páginas preguiçosas, o toque é no
bake) — e depois **um composite de `4096 × 4096` em `20 ms`**, contra `0,1–0,2 ms` dos seguintes. O motor
nasce com `Dirty::Full` (o `rebake_paper` do construtor, e o `reconcile_facts` re-coze outra vez quando
os knobs autorados do papel diferem dos de fábrica), e numa grade sem tinta nem água esse composite
escreve em cada pixel o que ele já tinha.

⛔ **E o §41 tinha piorado isto em MEMÓRIA:** com o composite a declarar a região, a janela declarada do
1.º traço era a tela inteira e o commit guardava o canvas `Whole` — **`524 288` bytes contra um plano de
`262 144`** numa tela de 256² (gate vermelho). A rede de debug não o apanha, e com razão: a janela
declarada era VERDADEIRA (um superconjunto), só era larga — *uma declaração honesta e larga não falha,
custa memória, e custo de memória ninguém vê sem medir*.

### §42.2 — A cura

No `stamp_dabs_wetpaint`, uma sessão nascida NESTE lote descarta o sujo depois do papel e do
`reconcile_facts` (`take_dirty`). ⚠️ **É ali e não no nascimento:** a 1.ª redacção pôs o descarte também
no `ensure_wet_session`, e a mutação que o apagava SOBREVIVEU — o descarte do lote já o cobre, e o
`reconcile_facts` vem depois do nascimento. Ficou uma porta só.

Gates (`wetpaint/birth_tests.rs`): o 1.º traço guarda menos de ¼ de plano, com o papel de fábrica e com
um papel AUTORADO (`PaperContrast` no máximo — o re-cozido do `reconcile_facts`); e a premissa — o
composite da folha inteira no nascimento não muda um byte, com e sem o véu do Show Wet. A mutação que
apaga o descarte reprova no caso do papel autorado.

**Medido:** 1.º traço da sessão `~40 → ~30 ms` (`30,5 / 32,4 / 29,8`, `load ~5`).

### §42.3 — Aberto

- **O papel no nascimento (`~30 ms`):** o tile de 512² (`12 ms`, com a caminhada das fibras e as somas
  em `f64` em série — a ordem é a impressão digital do motor) e o bake na grade (`18 ms`, uma cópia por
  linha, paralelizável ao bit — pede o `rayon` no `ph2d-wet-paint`, que exige ADR).
- **Com um papel do ARTISTA** o `seed_paper_with` amostra a textura do painter célula a célula em série
  (16,7 M chamadas a 4096²) — não medido.

## §43 — O papel do nascimento por linhas: o 1.º traço com um papel do artista deixa de congelar (2026-09-24)

Continuação do §42.3. Sonda (retirada) com um papel do artista no pincel: o `seed_paper_with` — a lei
de textura do painter chamada célula a célula, em série, 16,7 M vezes a 4096² — custava **`95–277 ms`**
no 1.º traço de cada sessão (e em toda troca de papel a meio). ⇒ o bake do tile e a semente passam a
correr por linhas (ADR-0175 **§3-bis**, `par::walk_rows`), byte-idênticos por construção, com a lei do
hospedeiro a passar a `&(dyn Fn + Sync)`.

| 1.º traço da sessão (4096², raio 100, `load ~7`) | antes | depois |
|---|---|---|
| sem papel | `~30 ms` (§42) | **`17,7 ms`** |
| papel `1` / `2` / `5` | `308,7 / 229,7 / 126,4 ms` | **`42,7 / 35,4 / 27,6 ms`** |

O `mede_o_wet_paint` ganhou o 5.º argumento `[papel]` (o `kind` do slot, como o painel o escreve).
Pisos MEDIDOS e diferentes (`MIN_CELLS_PAPER_BAKE = 128 k`, uma cópia; `MIN_CELLS_PAPER_SEED = 2 k`, a lei
do hospedeiro). Gates `tests/it/paper_rows.rs`: as duas rotas ao bit, o **laço de antes congelado** como
oráculo do bake (as duas rotas partilham o corpo por linha, e compará-las uma com a outra não apanha um
corpo errado — o tile é o da folha `1` contra o `0` do construtor, senão uma linha saltada ficava com o
papel certo por acaso), a cobertura do anel na semente e o corte à faixa do dente. **6 de 6 mutações
sangram**; três sobreviveram à 1.ª redacção (o oráculo independente e o corte faltavam). Suítes do motor
e do Painter verdes, clippy `-D warnings` limpo.

**Aberto:** o tile do motor (`~12 ms`, em série de propósito — impressão digital) e, com um papel do
artista, o bake do tile do motor que o papel do hospedeiro sobrescreve logo a seguir (trabalho deitado
fora, `~2 ms` depois desta wave).
