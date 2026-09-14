# 110 — CICLO 6: VALOR & PULSO, o cérebro

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, e o **tutorial é o smoke**.
> **Premissa do tutorial:** *«Um número que manda em tudo»*.
>
> ⚠️ Este doc é o do CICLO. O que ele mede vive nas sondas de
> [`motion_valor_probe.rs`](../../crates/ph2d-app-motion/src/motion_valor_probe.rs); o mecanismo de
> cada wave vai para o handoff dela.

---

## §1 — O grupo, CONTADO

`value.*` + `pulse.*`, **derivadas do registry** e não de uma lista escrita à mão (uma lista aqui
envelhece em silêncio no dia em que um nó da família nasce — a lei que o ciclo 5 já pagou):

| família | nós |
|---|---|
| `value.*` | 26 |
| `pulse.*` | 9 |
| **total** | **35** |

⭐ **É o maior grupo até hoje** — os cinco anteriores tiveram `10 · 8 · 13 · 7 · 12`. Gate
`both_value_families_are_derived_and_not_empty` (piso de população nas duas metades).

---

## §2 — O RETRATO (sonda `audit_the_value_group`)

```text
  nó                        | params | no cartão | device | portas | efeito
  --------------------------|--------|-----------|--------|--------|--------
  value.attribute           |      1 |         1 |  sim   | 1->1   | Pure
  value.cursor              |      0 |         0 |  NAO   | 1->2   | Pure
  value.curve               |      5 |         6 |  sim   | 1->1   | Pure
  value.gain                |      2 |         2 |  sim   | 1->1   | Pure
  value.instance_field      |      4 |         1 |  sim   | 1->1   | Pure
  value.lfo                 |      9 |         8 |  sim   | 1->1   | Temporal
  value.map_range           |      7 |         6 |  sim   | 1->1   | Pure
  value.math                |      3 |         1 |  sim   | 3->1   | Pure
  value.median              |      2 |         2 |  sim   | 1->1   | Pure
  value.mix                 |      4 |         4 |  sim   | 3->1   | Pure
  value.noise               |     15 |        11 |  sim   | 1->1   | Temporal
  value.normalize           |      1 |         1 |  sim   | 1->1   | Pure
  value.number              |      3 |         2 |  NAO   | 0->1   | Pure
  value.pattern             |     11 |        12 |  sim   | 1->1   | Pure
  value.percentile          |      2 |         2 |  sim   | 1->1   | Pure
  value.quantize            |      3 |         3 |  sim   | 1->1   | Pure
  value.reduce              |      1 |         1 |  sim   | 3->1   | Pure
  value.slope               |      1 |         1 |  sim   | 1->1   | Pure
  value.smooth              |      3 |         3 |  sim   | 1->1   | Pure
  value.step                |      4 |         3 |  sim   | 1->1   | Pure
  value.switch              |      2 |         2 |  sim   | 5->1   | Pure
  value.table               |      2 |         5 |  NAO   | 1->1   | Temporal
  value.time                |      3 |         3 |  sim   | 1->1   | Temporal
  value.unary               |      1 |         1 |  sim   | 1->1   | Pure
  value.wave                |      5 |         5 |  sim   | 1->1   | Pure
  value.wrap                |      3 |         3 |  sim   | 3->1   | Pure
  pulse.adsr                |      9 |         9 |  NAO   | 2->1   | Pure
  pulse.beat                |      6 |         5 |  NAO   | 2->1   | Temporal
  pulse.compare             |      3 |         3 |  NAO   | 3->1   | Pure
  pulse.counter             |      4 |         4 |  NAO   | 3->2   | Pure
  pulse.level               |      0 |         0 |  NAO   | 1->1   | Pure
  pulse.on_change           |      2 |         2 |  NAO   | 2->1   | Pure
  pulse.sample_hold         |      0 |         0 |  NAO   | 4->1   | Pure
  pulse.signal              |      0 |         1 |  NAO   | 1->1   | Pure
  pulse.threshold           |      5 |         5 |  NAO   | 2->1   | Pure
```

**O que ele já diz, sem uma linha de código escrita:**

1. ⛔ **A família `pulse.*` está FORA do dispositivo — 9 de 9.** Nenhum kernel registado.
2. ⛔ **Três `value.*` também**: `cursor` · `number` · `table`. ⚠️ O `value.number` é **uma
   constante**, o nó mais simples do catálogo.
3. ⚠️ **Três cartões estão VAZIOS** (`value.cursor` · `pulse.level` · `pulse.sample_hold`): zero
   controlos. Num mundo em que *os params vivem no cartão* (§2 do 103), um cartão vazio é um cartão
   que não diz nada sobre si — e os três são nós cuja resposta é só o fio. **Decidir**, não assumir.
4. ⚠️ **`value.instance_field` mostra `1` de `4` params** e o `value.math` `1` de `3` — o resto está
   atrás de gates. Pode estar certo (modo) ou ser um controlo inalcançável; a folha de conferência
   do grupo (doc 89, folha 15) tem de ser recorrida nó a nó.

---

## §3 — ⛔⛔ O ACHADO QUE DECIDE O CICLO: dirigir um param CUSTA O DISPOSITIVO

A razão de existir de um `value.*` é **dirigir um param** de outro nó. O
[doc 102 §2](102_o_outro_patamar_plano_dos_nos_2026-09-04.md) afirmava-o por escrito (*«um param
dirigido OU um pino ≠ 0 ligado derrubam o nó para a CPU»*), e uma afirmação que decide um ciclo
inteiro **mede-se antes de se escrever uma linha**. Sonda
`probe_does_a_value_chain_stay_on_the_device`, sobre a cadeia `motion.grid → motion.scale →
motion.output`:

| cadeia | onde corre |
|---|---|
| sem valor nenhum (o controlo) | **dispositivo** |
| com `value.number` a dirigir o `amount` | ⛔ CPU |
| com `value.lfo` a dirigir o `amount` | ⛔ CPU |
| com `value.math` a dirigir o `amount` | ⛔ CPU |
| com `value.time` a dirigir o `amount` | ⛔ CPU |
| com `value.noise` a dirigir o `amount` | ⛔ CPU |
| com `pulse.beat` a dirigir o `amount` | ⛔ CPU |

⭐⭐⭐ **6 de 6, e o primeiro deles é uma CONSTANTE.** Não é propriedade de nó nenhum deste grupo —
é do **planeador**: ele recusa cegamente qualquer nó com param dirigido. ⇒ *ligar um fio de valor a
um grafo troca o caminho de `3,85 ms` pelo de `195,9 ms`* (a razão de `50,9×` do
[doc 98](98_auditoria_de_performance_2026-09-01.md), medida a 4,19 M objectos).

⚠️ **E isto inverte o que o ciclo 6 É.** Ele não pode começar por *«acrescentar controlos aos nós de
valor»*: com a lei 1 do §2 do protocolo (*«todo nó do grupo tem de dizer onde corre, e um que caia
para a CPU sai do ciclo com a razão nomeada»*), **o grupo inteiro cai, e a razão é uma só, e está
fora dele**. Acrescentar poder a 35 nós cujo uso derruba o dispositivo seria construir por cima do
defeito.

⏳ **O relógio deste A/B ainda não foi lido** — a workstation esteve a `load 50–75` durante a
auditoria, e *nenhuma leitura de relógio desta máquina vale nada acima de `~5`* (`CLAUDE.md` §5.0).
A sonda `probe_the_price_of_driving_one_param` está escrita e espera máquina calma. ⚠️ **A 1.ª
redacção dela mentia**: ela tirava o **mínimo de cinco corridas** sobre uma cadeia `Pure`, que
**memoiza** na primeira — as outras quatro custavam ~zero e a tabela imprimia `0,00 ms` para 102 400
objectos **dos dois lados**. *Uma régua que lê zero não está a medir a lei, está a medir a cache.*
Hoje cada amostra reconstrói a árvore e **lê** a saída.

---

## §4 — O VOCABULÁRIO (sonda `the_value_vocabulary`): 6 divergências

Mesma chave, rótulos diferentes — o artista vê dois nomes para a mesma pergunta:

| chave | rótulos | onde |
|---|---|---|
| `clamp` | `Clamp` · `Clamp Factor` | `value.map_range` · `value.mix` |
| `interp` | `Interp` · `Interpolation` | `value.pattern` · `value.table` |
| `mode` | **(sem hint)** · `Mode` | `value.attribute` · 8 outros |
| `phase_stagger` | `Phase Stagger` · `Stagger` | `pulse.beat` · `value.lfo` |
| `step` | `Increment` · `Step` | `pulse.counter` · `value.quantize` |

⚠️ **O `mode` do `value.attribute` não tem hint nenhum** — é a única entrada da tabela sem rótulo.
⚠️ **E o `step` é o caso INVERSO dos outros quatro:** ali as duas coisas são genuinamente
diferentes (*quanto somar por pulso* · *o tamanho da grelha*), logo a cura não é igualar o rótulo —
é a **chave** ser outra. *Duas perguntas diferentes com a mesma chave é o defeito; dois rótulos são
o sintoma.*

---

## §5 — A fila proposta (a decidir com o dono)

1. ✅ **W1a — O PARAM DIRIGIDO NÃO DERRUBA O NÓ** — **FEITA** (§6). ⏳ A metade (b) (porta ≠ 0) fica. Já está **especificada ao detalhe** no
   [doc 102 §W1](102_o_outro_patamar_plano_dos_nos_2026-09-04.md): o param dirigido passa a ser um
   **uniform copiado no device** (um buffer `drv` por nó, `copy_buffer_to_buffer` de 4 bytes por
   fonte, zero passes a mais), e a porta ≠ 0 vira o **complemento de um `Compact`**. Gates de
   paridade bit-a-bit já nomeados, contrato **não congelado** (`gpu.rs` é side-metadata).
   ⇒ *Sem ela, todo o resto deste ciclo assenta num caminho que custa 50×.*
2. **W2 — a família `pulse.*` no dispositivo** (9 de 9 fora hoje), ou a razão nomeada e o preço
   medido para cada uma que fique.
3. **W3 — o cartão e o vocabulário**: as 6 divergências do §4, os três cartões vazios, e os params
   que o cartão não alcança.
4. **W4 — o poder que falta**, nó a nó, contra o estado da arte (a folha 15 da conferência + as
   referências), que é o passo 2 do protocolo por nó.
5. **W5 — a MEDIÇÃO** (passo 5) e **W6 — o TUTORIAL em PDF** (passos 6 e 7, o smoke do dono).

⚠️ **A ordem 1→2 não é preferência: é a lei 1 do §2 do protocolo.** Um grupo cujo uso normal
derruba o dispositivo não fecha um ciclo com «tem mais botões».

---

## §6 — W1a FEITA: um fio de valor já não custa o dispositivo

> Ordem do dono, 2026-09-14: *«sim. faça o que for melhor.»*

### O que mudou, em uma frase

O planeador deixou de recusar cegamente um nó com param dirigido; o que ele passou a exigir é o
**número**, entregue por quem coze o condutor.

### A régua, antes e depois

Mesma sonda (`probe_does_a_value_chain_stay_on_the_device`), mesma cadeia
`motion.grid → motion.scale → motion.output`:

| condutor a dirigir o `amount` | antes | **depois** | o valor que chegou |
|---|---|---|---|
| — (o controlo) | dispositivo | dispositivo | — |
| `value.number` | ⛔ CPU | **dispositivo** | `1` |
| `value.lfo` | ⛔ CPU | **dispositivo** | `0` |
| `value.math` | ⛔ CPU | **dispositivo** | `1` |
| `value.time` | ⛔ CPU | **dispositivo** | `0` |
| `value.noise` | ⛔ CPU | **dispositivo** | `-1` |
| `pulse.beat` | ⛔ CPU | **dispositivo** | vazio ⇒ cai no default |

⚠️ **A terceira coluna nasceu de um engano meu e ficou por isso.** A primeira corrida depois da cura
deu `4 de 6`, com o `value.math` e o `pulse.beat` ainda em CPU — e as duas causas eram diferentes e
**liam-se iguais numa coluna só**: o `math` opera sobre uma corrente e, desligado, não produz número
nenhum; o `pulse.beat` só fala no instante em que dispara. *Uma recusa do planeador e um condutor
vazio não são a mesma coisa, e sem a coluna do VALOR a tabela ensinaria que a wave não funcionou.*

### O desenho, e a única falha grave que ele tinha de impedir

⭐⭐ **As duas rotas a desenharem documentos diferentes.** É a falha que nenhum gate de rota vê: o nó
fica no dispositivo, o dispositivo lê o default, e o artista vê outra cena. A cura é a regra ser
sobre a **chave** e não sobre o valor:

- **chave ausente** = *«ninguém consultou este fio»* ⇒ o nó **não é encenado** (a lei de sempre);
- chave presente com **`None`** = *«consultei, e o condutor não deu número»* ⇒ o param cai no
  override/default — exactamente o que o `driven_value` da CPU faz — e o nó **fica**.

⛔⛔ **Colapsar as duas faria a cena ENGASGAR**, e foi a medição que o mostrou: um `pulse.*` só dá
número quando dispara, logo o plano trocaria de rota a cada tique. Gate
`a_wire_that_gives_no_number_never_flips_the_route` (12 instantes, a mesma resposta em todos).

⭐ **E a `plan` continua a ser a `plan_driven` com o mapa vazio** — sem valores, nenhum nó com fio é
encenado, que é o que este planeador fazia antes da wave. *Uma porta nova cujo caso vazio reproduz a
lei antiga não precisa que ninguém confie nela*, e o gate antigo
(`a_driven_param_puts_the_boundary_at_the_driven_node`) passou sem uma linha mudada.

### ⚠️ Divergência DECLARADA do desenho do doc 102 W1(a)

O doc 102 desenha um buffer `drv` por nó com `copy_buffer_to_buffer` de 4 bytes e substituição de
`params.<name>` por `drv[k]` no WGSL. **Não foi esse o caminho.** O valor entra pelo **uniform que
já existe**, porque a CPU o tem em mão: é o caminho que o próprio doc 102 nomeia como *«o escalar
avaliado na CPU entra no `drv` como fallback (zero cópias)»*. ⇒ zero buffers novos, zero passes
novos, zero mudanças no codegen — e o gate `no_params_dot_survives_for_a_driven_name` que aquele
desenho pedia **não se aplica a este**.

⏳ **O que fica por fazer, nomeado:** quando o condutor é ele próprio um estágio de GPU, o número
faz uma volta pela CPU que uma cópia de 4 bytes device-side evitaria. ⚠️ **Não é uma regressão** — o
condutor já era cozido na CPU hoje, só que arrastava o consumidor e os milhões de objectos dele
atrás. *O que esta wave tira do caminho lento é o consumidor.*

⏳ E a metade **(b)** do W1 (porta ≠ 0 = o complemento de um `Compact`) fica inteira por fazer: ela
é outra pergunta (o `sim.lifetime.pulse`), e não é a que a medição do §3 acusou.

### O que segura isto

| gate | onde | o que afirma |
|---|---|---|
| `a_driven_param_with_its_value_in_hand_keeps_the_node_on_the_device` | `plan_analysis` | com o valor, a cadeia é do dispositivo |
| `a_param_nobody_consulted_keeps_the_whole_chain_on_the_cpu` | `plan_analysis` | sem ele, recua — **por param**, não por nó |
| `an_empty_driver_keeps_the_node_and_falls_back_like_the_cpu` | `plan_analysis` | consultado e vazio ainda é consultado |
| `a_driven_param_puts_the_boundary_at_the_driven_node` | `plan_analysis` | a `plan` sem mapa é a lei ANTIGA, intacta |
| ⭐ `the_device_reads_the_driven_param_and_agrees_with_the_cpu` | `gpu_cpu_parity_driven` | **as duas rotas concordam a `1e-4` num dispositivo REAL**, com o controlo que prova que o device lê o fio |
| `a_value_wire_no_longer_costs_the_device` | `motion_valor_probe` | os 6 condutores, pela porta da produção |
| `a_wire_that_gives_no_number_never_flips_the_route` | `motion_valor_probe` | 12 instantes, a mesma rota |
| `the_driven_values_follow_the_instant` | `motion_valor_probe` | a porta lê o TEMPO |
| `the_recusals_run_in_the_right_place_relative_to_the_plan` | shell | os valores derivam-se **depois** das recusas |

**Provas de mutação: 5/5 mortas.** ⚠️ Uma delas só morreu depois de eu mudar o CÓDIGO em vez do
gate: congelar o instante no laço dos tiques sobrevivia a tudo o que eu tinha escrito, e a cura não
foi um gate novo — foi pôr o instante do plano **dentro de um bloco**, onde o laço não lhe chega.
*O erro que não compila não precisa de gate*; o gate que ficou (`the_driven_values_follow_the_instant`)
segura a outra metade, que é a porta ler o tempo.

### Tectos e arrumação

⚠️ O `ph2d-gpu-cook/src/lib.rs` passou o tecto de LOC (`714 > 700`) e a cura foi **corte por
responsabilidade**: o ESTADO do sequenciador (26 campos) saiu para
[`estado.rs`](../../crates/ph2d-gpu-cook/src/estado.rs) — *ali está o que ele FAZ, aqui o que ele
TEM* — e o `lib.rs` ficou em `601`. ⚠️ Mover o tipo para fora da raiz muda **quem lhe pode ler os
campos** (um campo privado é visível ao módulo que o declara e aos descendentes): daí o
`pub(crate)`.

⚠️ E a `stage_window` passou a **oito** argumentos: os quatro que andam sempre juntos (`graph`,
`node`, `manifest`, `driven`) viraram uma `count::No`. *É a mesma razão pela qual o solver de
contacto tem uma `Pecas`* — dois `&`-de-mesma-forma numa lista posicional trocam de lugar sem o
compilador dizer nada.

### ⚠️ DOIS candidatos NOMEADOS à família das flakes de carga

A suíte de GPU inteira (`--ignored`, 784 s) deu **198 verdes e 3 vermelhos**. Um é o pré-existente
abaixo; os outros dois são **gates de CUSTO**, e correram numa janela em que esta workstation esteve
a `load 26–75` — *nenhuma leitura de relógio desta máquina vale nada acima de `~5`*
(`CLAUDE.md` §5.0):

| gate | sozinho, com a carga ao lado |
|---|---|
| `gpu_collide::crossing_the_reach_boundary_does_not_step_the_cost` | **1/1 verde** a `load 4,55` |
| `gpu_cpu_parity_sim::readback_tap_cost_probe` | **3/3 verde** a `load 4,55 · 4,26 · 4,00` |

⇒ assinatura completa da família: verde isolado, vermelho no pico do fan-out, e **zero linhas do
diff desta wave naquelas crates**. ⚠️ **Ficam NOMEADOS de propósito** — *uma flake sem nome não entra
numa lista, e quem a encontrar outra vez recomeça do zero*. A promoção à lista do `CLAUDE.md` §5.0 é
**pedido da linha e escrita do integrador** (o precedente de 10/09).

### ⚠️ Um VERMELHO pré-existente, medido e NÃO meu

A suíte de paridade de GPU desta máquina (o adapter existe, e por isso ela correu de verdade) dá
**142 verdes e 1 vermelho**: `value_slope_kernel_matches_the_cpu_on_the_device`, com
`max |d| = 1,05023384e-4` contra a barra de `1e-4` — **fora por 5 centésimos de por cento**.

⛔ **Medido nos dois lados antes de escrever isto:** com a wave posta de lado (`git stash`), o
mesmo teste dá **exactamente o mesmo número, ao bit**. Não é desta wave. ⚠️ É a espécie que o
`CLAUDE.md` §5.0 nomeia: um gate `#[ignore]` que **o CI nunca correu**, a viver a um ULP da barra —
e quem lhe tocar primeiro herda-o. *Fica registado com o número, não curado aqui: uma barra de
paridade mexe-se com a medição do autor dela ao lado, não de passagem noutra wave.*

⏳ **O RELÓGIO continua por ler** (§3): a workstation esteve a `load 26–75` toda a jornada. A sonda
`probe_the_price_of_driving_one_param` está escrita, corrigida e à espera de máquina calma.
