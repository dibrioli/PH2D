# ARQUIVO — 112_ciclo_7_aparencia.md (história, 106 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`112_ciclo_7_aparencia.md`](../../Motion%20Nodes/112_ciclo_7_aparencia.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-461,516-535,588-651` do original.
>
> ⚠️ **A única alteração ao corpo:** 3 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### Os DOIS relógios — a sonda nova

⚠️ **Os ciclos 5 e 6 mediram só a CPU de referência** e escreveram que o relógio do dispositivo não
tinha sonda. Este grupo está inteiro na placa, então
[`motion_bridge_aparencia_relogio.rs`](../../../crates/ph2d-app-motion/src/motion_bridge_aparencia_relogio.rs)
(`measure_the_fx_group_on_both_engines`) coze a MESMA cadeia pelos dois motores, quadro a quadro,
**em regime** (mediana de `9` quadros depois de `120` tiques), com três cuidados que uma tabela
ingénua deste grupo não teria:

- **o estado** — a cadeia passa pela canalização do PRODUTO (`plumbing::reconcile_after`, a porta que
  o editor corre ao largar o nó), e a coluna das linhas prova que a cauda encheu (`n × 9`);
- **o neutro** — o nó é medido **acordado** (`motion_ciclo_preco::acordar`);
- **o pulso** — uma porta chamada `pulse` (lida do manifesto) recebe um `pulse.beat`.

⭐ **E a sonda confere-se:** a CPU e o dispositivo emitiram o **mesmo número de linhas** em todas as
células (um `⚠️` na coluna seria um desacordo de contagem).

`grid lado² → oscillator → X → output`, `--release`, lançada por
[`medir_quando_calmo.sh`](../../Motion%20Nodes/ferramentas/medir_quando_calmo.sh) (quatro amostras seguidas de `load` ≤
`4,5`; durante as corridas o 1-min foi de `3,75` a `5,47`). Cita-se o **menor de duas corridas**:

```text
  1 000 000 objectos     │ linhas    │ disp ms │ CPU ms │ ns/linha disp
  (sem X — a base)       │ 1 000 000 │   2,05  │   3,48 │  2,05
  fx.drop_shadow         │ 1 000 000 │   1,89  │   3,22 │  1,89   ← sombra macia DESLIGADA pelo tecto
  fx.glow                │ 1 000 000 │   2,03  │   3,41 │  2,03   (passa-tudo)
  fx.rgb_split           │ 3 000 000 │   5,42  │  41,54 │  1,81
  motion.color_array     │ 1 000 000 │   1,81  │  16,16 │  1,81
  motion.color_ramp      │ 1 000 000 │   1,83  │  26,21 │  1,83
  motion.slit_scan       │ 1 000 000 │   3,49  │  41,22 │  3,49   (era 16,33 — ver abaixo)
  motion.strobe          │ 1 000 000 │   1,97  │  18,44 │  1,97
  motion.sub_uv          │ 1 000 000 │   2,24  │  15,59 │  2,24
  motion.tint            │ 1 000 000 │   1,86  │  17,09 │  1,86
  motion.trail           │ 2 000 000 │   4,31  │  44,52 │  2,15   ← cauda CORTADA a 2 gerações pelo tecto

  102 400 objectos       │ linhas    │ disp ms │ CPU ms
  (sem X — a base)       │   102 400 │   0,19  │   0,29
  fx.drop_shadow         │ 1 740 800 │   3,05  │  21,82
  fx.glow                │   102 400 │   0,19  │   0,34
  fx.rgb_split           │   307 200 │   1,09  │   2,47
  motion.color_array     │   102 400 │   0,20  │   1,90
  motion.color_ramp      │   102 400 │   0,22  │   2,84
  motion.slit_scan       │   102 400 │   0,43  │   4,32   (era 1,80)
  motion.strobe          │   102 400 │   0,22  │   1,97
  motion.sub_uv          │   102 400 │   0,21  │   1,42
  motion.tint            │   102 400 │   0,20  │   1,77
  motion.trail           │   921 600 │   2,35  │  23,21
```

⇒ **~1,8–2,3 ns por linha na placa** para nove dos dez, contra `7–15×` isso na CPU nos que mudam
colunas. ⚠️ **As duas linhas de um milhão com `←` não medem o efeito inteiro:** acima do tecto a
sombra macia desliga-se e a cauda encurta — é o comportamento declarado no §4-bis/§4-quater, e é
ele que as mantém abaixo dos `6 ms`; a `102 400` os dois efeitos estão inteiros.

### A cena — três fileiras, uma pergunta por fileira

```text
  CIMA    A COR      Tint                      |  Color Ramp (no lugar dele)
  MEIO    O RASTO    cada peça anda em roda    |  + Trail
  BAIXO   O TEMPO    Slit Scan: ORDEM          |  Slit Scan: CAMPO (+ Falloff linear)
```

Seis panos `6 × 6` iguais ([`motion_state_aparencia_demo.rs`](../../../crates/ph2d-app-motion/src/motion_state_aparencia_demo.rs)).
A de cima é **parada de propósito** (a cor não precisa de tempo). Em cima muda um cartão (troca), no
meio um cartão a mais; em baixo mudam **a linha `Delay By` e o campo que ela lê** — o anúncio diz as
duas, porque o modo novo SEM campo não tem nada para mostrar. Os dois `Slit Scan` têm **nome**
(`set_label`, o molde da `=115`). ⛔ O brilho, a sombra e a separação ficam de fora: o brilho acende
a cena INTEIRA e não teria par; o tutorial ensina os três no mapa do grupo.

**Catorze gates** (doze da cena, dois do tutorial), um por promessa do anúncio —
incluindo as que pedem ao dono que MEXA num controlo (os gates mexem no mesmo param que a linha do
cartão escreve):

| promessa | a régua | medido |
|---|---|---|
| cima: uma cor contra uma por peça | cores distintas | `1` contra `35` |
| passo 3: o `Tint` pinta a esquerda inteira e não a direita | a coluna `tint` | — |
| meio: roda, e só a direita com cauda | o RAIO de cada linha ao centro da sua célula; a contagem | `±0,1 %` do raio; `36` contra `432` |
| passo 6: `Length` no fim fecha o anel; `Tail Alpha` 1 não apaga | o maior buraco angular numa célula; a alfa mínima | `231°` → `< 30°`; `1` |
| `Tail Size` 1 não encolhe | o `size` de toda linha | `= PECA` |
| baixo: ordem contra lugar | o espalhamento DENTRO de uma coluna | `0,24` contra `1,2e-7` |
| passo 8: `Field` sem campo faz a onda sumir (e o pano continua a mexer) | os dois espalhamentos; duas alturas | `< 1e-4` |
| passo 9: `Invert` troca o lado | qual coluna anda com o oscilador sem atraso | `0` → `5` |
| `Circle`: os cantos primeiro | o canto = vivo; o meio ≠ vivo | — |
| cada par difere pelo que o anúncio diz | contagem de tipos + os params dos dois `Slit Scan` | `36` nós |

**Mutações:** `Delay By` igual dos dois lados · o `phase_stagger = 0` apagado (o default do
oscilador já faz uma onda sozinho) · o rasto retirado · o raio do campo a cobrir os cantos — as
quatro **RED**.

⭐⭐ **Três coisas que só a construção revelou:**

1. ⚠️ **A peça `0` nasce em BAIXO** — a 1.ª redacção do anúncio dizia *«a onda corre como quem lê um
   texto»*; a grade é row-major a partir do `y` menor, então ela SOBE o pano. A frase mudou e há gate.
2. ⚠️⚠️ **A FIGURA do tutorial mostrou um defeito da CENA:** com a peça a `0,16` e a roda a `0,1` o
   rasto saía uma **mancha colada à peça**, não um arco — e era isso que o dono veria. A cena passou
   a peça `0,10`, roda `0,14`, passo `0,40` (o anel inteiro, `0,38`, cabe no passo).
3. ⛔ **Um passo do tutorial era impossível e saiu antes de ser escrito:** *«arraste `Spacing` para a
   cauda virar uma fila de peças»* — doze cópias de `0,1` não cabem separadas num anel de `0,88` de
   perímetro, e com o `Spacing` alto a cauda dá mais de uma volta e as cópias SOBREPÕEM-SE. Trocado
   por `Tail Size`, com gate.

⚠️ E o `every_row_the_appearance_tutorial_names_is_on_the_card` reprovou sobre um
texto certo na 1.ª redacção: a linha `End` do `Tint` só aparece com `Mode = Gradient`, que é
exactamente a ordem do passo 12 — o gate passou a perguntar ao cartão NESSE estado.

