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
2. ✅ **W2 — a família `pulse.*` no dispositivo** — **FEITA** (§8): **9 de 9** com kernel e
   paridade num adapter real. ⛔ E a medição diz que isso ainda **não muda rota nenhuma**: a faixa
   acaba nos CONSUMIDORES (`sim.spawn` recusa a porta · `motion.strobe`/`motion.step` sem kernel),
   os três nomeados com mecanismo no §8.6.
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

### A cena `=116` — onde isto se VÊ

⚠️ **O que mudou não é uma aparência: é ONDE a cena corre.** Uma cena de A/B por quadrantes
ensinaria que isto é uma questão de desenho — os dois lados desta comparação são a **mesma** cena
com o interruptor `PH2D_MOTION_DRIVEN_GPU` num sítio e noutro.

- **102 400 peças e UM fio** (um `value.lfo` a comandar o `motion.scale`): a cadeia mais simples
  que existe com um nó de valor, e exactamente a que caía.
- ⚠️ **Ela é grande porque o sujeito é o CUSTO.** As `=111`..`=115` têm dezenas de peças porque
  ensinam uma LEI; um custo de `50×` sobre dez peças não se vê.
- **Medido no app a correr**, as duas corridas:

```text
  normal                        → [motion-route] device: o plano inteiro (fully-GPU)
  PH2D_MOTION_DRIVEN_GPU=0      → [motion-route] CPU: fronteira sem estagio de GPU que despache
```

⚠️ **E o anúncio teve de deixar de imprimir a linha que manda procurar**: ele citava-a *verbatim*, e
a citação e a linha de verdade liam-se iguais no terminal. Agora ele diz que a de verdade está
**colada à margem**, e nomeia a diferença. *Um smoke que manda procurar um texto não pode ser o
próprio produtor desse texto.*

Gates: `the_wire_scene_is_claimed_by_the_device` · ⛔ `with_the_switch_off_the_same_scene_falls_to_the_cpu`
(sem esta metade, o passo 3 do anúncio manda comparar duas corridas iguais) ·
`the_scene_has_exactly_one_wire_and_it_drives_the_scale` · `the_cloth_is_as_big_as_the_announcement_says` ·
`the_cloth_actually_breathes`.

### ⛔⛔ A sonda do PREÇO foi APAGADA — ela media outro programa

`probe_the_price_of_driving_one_param` imprimia uma razão para *«o preço de dirigir um param»*. **Os
dois lados dela corriam o cozedor da CPU** (`pump.cook`) e o plano dela era o antigo: ela nunca
tocou no dispositivo. O que media era o custo de cozer o CONDUTOR — a parte que esta wave não muda.

⚠️ **E ela devolveu um número plausível — `1,21×` — que é o que a tornava perigosa.** Uma régua que
devolve `0,00 ms` desconfia-se; uma que devolve `1,21×` cita-se. *Uma régua que mede outro programa
é pior que régua nenhuma.* ⇒ o instrumento do preço passa a ser a cena `=116` com o registo de rota,
e a paridade que corre as duas rotas a sério. Um A/B de relógio device-contra-CPU pede um
`GpuContext` no arnês e é wave própria.

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


---

## §7 — Report do dono: *«usando shape (exemplo: star) fps cai para 27»* (2026-09-14)

Observado no smoke da `=116`. **Reproduzido, medido, e a causa NÃO é a que o código faz supor.**

⚠️ **A cena passou a saber fazê-lo** (`PH2D_FIO_FORMA=<kind>`, um knob de DIAGNÓSTICO): sem uma
porta assim o report do dono não é reproduzível por mais ninguém, e a forma é **argumento** de
[`build_com`] — um ramo de ambiente dentro da lei tornaria o caso inalcançável de um teste, e foi um
teste que precisou dele primeiro.

### O que o app diz sozinho

```text
[motion-route] CPU: o grafo traz uma FORMA vectorial viva (source.shape)
[frame] total=39,77ms (~25 fps) | cpu-encode=39,80ms | acquire=0,05ms
```

### As três medições, lado a lado

| caso | rota | cook | quadro | fps |
|---|---|---|---|---|
| quads, como a cena shipa | dispositivo | 2,87 ms | **16,92 ms** | 59 |
| quads, **forçados** à CPU (`PH2D_MOTION_DRIVEN_GPU=0`) | CPU | — | **16,68 ms** | 60 |
| **estrelas** (`PH2D_FIO_FORMA=5`) | CPU (forma viva) | 4,49 ms | **39,77 ms** | **25** |

⭐⭐⭐ **A leitura óbvia está ERRADA, e a segunda linha é o controlo que a derruba.** O código tem uma
recusa declarada — *«o grafo traz uma FORMA vectorial viva»* — e é tentador concluir que os 27 fps
são a queda para a CPU. **Não são:** as MESMAS 102 400 peças forçadas à CPU seguram `60 fps`. A
rota custa uns milissegundos; o que custa `~31 ms` é outra coisa.

### E não é o que eu supus também

| hipótese | medida | veredito |
|---|---|---|
| *«o carimbo multiplica a contagem (`formas × pontos`)»* | `102 400` **nos dois casos** | ⛔ refutada |
| *«o cozimento fica caro»* | `2,87 → 4,49 ms` (`+1,6`) | ⛔ refutada (4% do quadro) |
| **o DESENHO** | `39,80 ms` de encode com `4,49` de cook ⇒ **`~35 ms` fora do cozimento** | ✅ é aqui |

⇒ **Cada estrela é ARTE DESENHADA — um caminho com pontos, construído e codificado a cada quadro**,
contra um quad com uma textura. A 102 400 delas, isso é o quadro inteiro.

⏳ **A cura já tem nome e número na fila:** é o item **10** do [doc 103 §5](103_dinamica_dos_ciclos.md)
— *«o CARIMBO NO DISPOSITIVO (`source.shape` + `motion.duplicator`)»* —, que está **deliberadamente
no fim** porque é wave de substrato (toca o planeador, por onde todos os ciclos passam). ⚠️ E o §0.0
manda reconferir a nota de quem move o número: a W1a mexeu no planeador, e **não** desbloqueia isto
— o que ali falta é a forma chegar ao dispositivo como coisa desenhável, não um param.

---

## §8 — W2 FEITA: os **nove** `pulse.*` no dispositivo — e o que isso ainda NÃO compra

> Ordem do dono, 2026-09-14: *«seguir o plano … agora: 9 nós de pulso.»*

### §8.1 — A medição veio primeiro, e mudou a pergunta

A W2 pedia *«a família `pulse.*` no dispositivo (9 de 9 fora hoje), **ou** a razão nomeada e o preço
medido para cada uma que fique»*. Antes de escrever um kernel (§0.0), a sonda
`probe_where_the_pulse_seam_falls` perguntou ao planeador **onde é a costura**:

| cadeia | stages | onde a CPU ainda coze |
|---|---|---|
| `grid → scale → output` (o controlo) | 3 | — (tudo no dispositivo) |
| `grid → beat → sim.spawn(pulse) → output` | **1** | **`sim.spawn:0`** |
| `grid → sim.spawn` (**porta de pulso solta**) | 3 | — (tudo no dispositivo) |
| `grid → threshold → strobe(pulse) → output` | **1** | **`motion.strobe:0`** |

⭐⭐⭐ **A linha 2 com a linha 3 é o achado:** o `sim.spawn` **tem** kernel e é reivindicado — até
lhe ligarem um metrónomo. *Um nó sem kernel não custa o que ele custa: custa a SIMULAÇÃO inteira*,
que é o caminho medido em `50,9×` pela [auditoria 98](98_auditoria_de_performance_2026-09-01.md).

### §8.2 — Os nove kernels (ADR-0126 — side-metadata; `NodeManifest` intocado)

| nó | o que o kernel é | a lei de borda que ele teve de portar |
|---|---|---|
| `pulse.level` | máscara `0/1` | a coluna `pulse` é **`Consume`**: a `level` da CPU **larga** o pulso |
| `pulse.signal` | **`GpuKernel::PASSTHROUGH`** | o NOME vive num `text_param` e não viaja na corrente |
| `pulse.compare` | Schmitt + referência por linha | a referência é **`ReadBroadcast`** (desligada ⇒ params · `1` ⇒ todo o campo · `N` ⇒ por linha) |
| `pulse.on_change` | duas colunas de estado | `direction` ilegível cai no **NEUTRO**, não na variante `0` |
| `pulse.sample_hold` | amostrador por borda | as DUAS portas de pulso em `ReadBroadcast` (um gatilho global vale para o campo) |
| `pulse.threshold` | Schmitt sobre um CANAL + abrandador | NaN no `channel` escolhe **X** e ±inf escolhe **Size** (`f32::NAN as i32` vale `0` em Rust) |
| `pulse.counter` | aritmética Euclidiana inteira | `rem_euclid`/`div_euclid` portados; o **carry** é porta ≠ 0 ⇒ o planeador recua |
| `pulse.beat` | metrónomo por linha | uma linha **sem história** dispara ⇒ `!HAS_state_beat_cycle` no meio da condição |
| `pulse.adsr` | envelope puro na idade | o `bias` de Schlick com as **duas** guardas de não-finito (`f32::clamp` devolve NaN para NaN) |

⚠️ **Todo selector repete o arredondamento do Rust** (`f32::round` = meio para LONGE do zero; o
`round` da WGSL é **meio-par**). Um enum escolhido pelo braço errado responde *plausível*.

⚠️ **E um param chamado `count` colide com o uniform de contagem**: o corpo escreve
`params.count_` (o `wgsl_field` dá ao param colidente um campo próprio). Sem o sublinhado a placa
recusa o módulo inteiro — foi assim que este defeito apareceu, no `pulse.beat`.

### §8.3 — ⭐⭐⭐ O `dt` passou a ser UNIFORM, e isso dissolveu DUAS recusas no mesmo dia

O `pulse.adsr` avança `age + dt` e o abrandador do `pulse.threshold` conta `cool − dt`. O módulo
gerado **não tinha `dt`**: o `motion.integrate` deriva o dele de uma coluna de estado (`sim_t`) que
a CPU **também** escreve, e dar uma a estes dois mudaria a lei da CPU.

⛔ A saída barata era um `applicable` a recuar acima do neutro — e o `pulse.threshold` shipou assim
durante meia wave. A saída **certa** custou quatro sítios: `codegen::kernel_module` declara
`dt: f32` a seguir ao `playhead`, o `encode` escreve-o em `uni[8..12]` e os params passam a começar
em **`PARAMS_AT = 12`** (uma constante, porque o deslocamento é lido em **quatro** sítios daquela
função). *O bloqueador era o SUBSTRATO e não a lei — que é exactamente a espécie de limite que o
§0.0 manda medir em vez de aceitar.*

⚠️ Um campo opcional teria custado uma declaração nova no `GpuKernel`, que é construído
**literalmente em ~50 crates-nó**; quatro bytes num uniform de slot custam zero.

### §8.4 — Os gates, e as DUAS metades que este commit precisou

`gpu_cpu_parity_pulse` — **11 testes**, adapter real, `6` voltas com o estado a atravessar o `pre`:

- **a não-vacuidade**: a coluna da CPU tem de ter **disparado E ficado calada** — senão «as duas
  rotas concordam» é a afirmação vazia de duas colunas de zeros;
- ⛔⛔ **o FIO DA NAVALHA, com número**: um nó de pulso é uma **comparação**, e uma comparação não
  tem ε. Medido, **`3` de `384`** amostras discordam — `v_cpu = 0,5000009` contra
  `v_dev = 0,49999955` (delta `1,34e-6`, o ε que toda a família `value.*` já declara) com o limiar
  `0,5` **exactamente entre os dois**. A régua é **ESTRUTURAL**
  (`min(v_cpu, v_dev) ≤ limiar ≤ max(…)`), logo não há constante para afrouxar: uma linha que
  discordasse com os dois valores do **mesmo lado** é defeito de lei e reprova.

⚠️ **E três correcções que foram da RÉGUA, não do produto:**

1. os cinco primeiros gates reprovaram na volta **1** com divergência `1,0` — faltava o
   `cook.advance_tick`, que é o que **publica** o `pre` do lado da CPU (o device publica o dele
   sozinho). *A régua estava a medir a régua.*
2. a mensagem de erro imprimia as **oito primeiras** linhas com a divergência na **39.ª** — *uma
   mensagem que mostra o princípio de um vector prova que o princípio está bem.*
3. ⛔⛔ a fixture do abrandador era **VAZIA**, e foi uma **prova de mutação** que o disse: com o
   gatilho sobre uma grelha ESTÁTICA o abrandador nunca chega a abrandar, e a mutação que zera o
   `dt` no uniform **sobreviveu**. O campo passou a ser um oscilador a `8 Hz` e o controlo é
   **comparativo** (`debounce = 0` dispara MAIS: `42` contra `40`). Com isso, a mutação mata os dois.

### §8.5 — ⛔⛔⛔ Um `var<storage, read>` que o corpo não lê é APAGADO do layout, e só rebenta no 2.º tique

O `pulse.threshold` declarou `thr_cool` como `ReadWrite` enquanto o corpo só o escrevia. O layout do
bind group é **derivado do shader**, e a naga **apaga** um buffer que nada referencia — o
sequenciador, esse, monta as entradas a partir da DECLARAÇÃO. Resultado: `7` entradas contra `6` no
layout, e a wgpu recusa em `create_bind_group`.

⚠️⚠️ **No primeiro tique isto PASSA**: a coluna de estado ainda não existe, nenhum buffer é ligado.
*Um erro de declaração que só acorda quando a coluna nasce é o pior sítio para o pôr.*

⭐ A lição **já estava escrita neste repo**, no `reduce_stage.rs` (*«an unread binding is absent from
the reflected layout»*) — e não havia nada que a tornasse executável. Agora há:
`todo_read_declarado_e_lido`, dentro do `every_registered_kernel_validates_across_the_whole_presence_space`,
que corre sobre **todo** kernel do registry × **toda** máscara de presença. Prova de mutação: repor
o `ReadWrite` acusa `pulse.threshold mask 100000`.

⚠️ **E ele apanhou de imediato a MESMA forma no `motion.spring`**, pré-existente — a companheira de
`gather` (`id` na porta de estado) não tem leitor quando o gather está desligado. Essa fica
**isenta com PROVA, não com indulgência**: a presença que a exigiria (estado com `id`, base sem) não
é produzível — a porta de estado é alimentada por um `pre` da própria saída, e a saída herda as
colunas da porta 0. *A varredura é exaustiva sobre `2^n` máscaras, e nem toda máscara é um estado
que a máquina alcança.*

### §8.6 — ⛔⛔ O QUE ISTO AINDA NÃO COMPRA, medido

Com os nove kernels, as **três** cadeias medidas no §8.1 têm exactamente a mesma rota de antes. A
faixa do pulso não acaba no produtor — ela acaba no **consumidor**, e os três estão fechados:

| consumidor | estado | o bloqueador, nomeado |
|---|---|---|
| `sim.spawn` | tem kernel, **recusa a porta** | o nascimento por pulso nasce **na linha que disparou** ⇒ a contagem é função do CONTEÚDO da entrada, e o `CountLawCtx` proíbe olhar para ele por escrito (seria um *readback*). **Wave de substrato.** |
| `motion.strobe` | **sem kernel** | é nó do **ciclo 7** (Aparência/Fx) pela fila do doc 103 |
| `motion.step` | **sem kernel** | idem, sem ciclo atribuído |
| a rota do **param dirigido** | CPU **por desenho** | a W1a resolve o condutor na CPU e entrega o NÚMERO (§6) |

⚠️⚠️ **E a recusa do `sim.spawn` tinha a justificação ESCRITA que esta wave dissolveu:** *«none of
the six `pulse.*` nodes has a GPU kernel … so the chain feeding this port is already a device
boundary»* — eram **nove** e não seis, e hoje todos têm. O comentário foi corrigido no próprio
ficheiro (§0.0: *quem move o número que tornava algo inalcançável tem de reconferir a nota*), e o
gate `the_chain_that_opened_the_wave_is_blocked_by_the_spawn_and_no_longer_by_the_metronome`
**reprova no dia em que alguém curar o spawn**, obrigando a reescrever a nota.

⇒ **os nove kernels são o PRÉ-REQUISITO, não o ganho** — e é assim que a W2 fica escrita: nenhuma
cadeia do produto mudou de rota hoje, e os três bloqueadores que sobram têm mecanismo, endereço e
dono.
