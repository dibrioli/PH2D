# ARQUIVO — parte_1.md (história, 341 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_1.md`](parte_1.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-543` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## §9 — W3a: o VOCABULÁRIO e o ALCANCE — dois instrumentos novos, três curas e uma recusa

> Ordem do dono, 2026-09-14: *«continuar o grupo.»*

### §9.1 — ⛔⛔ A sonda do §4 agrupa pela CHAVE, e por isso é cega a metade do problema

Três nós perguntam *«como é que eu interpolo?»* e a
[`the_value_vocabulary`](../../../../crates/ph2d-app-motion/src/motion_valor_probe.rs) só via dois:

| nó | chave | rótulo | opções |
|---|---|---|---|
| `value.pattern` | `interp` | **Interp** | Step · Linear |
| `value.table` | `interp` | **Interpolation** | Step · Linear |
| `value.map_range` | **`interpolation`** | **Interpolation** | Linear · **Stepped** · Smooth · Smoother |

⇒ **uma chave é o que o código guarda; um rótulo é o que o artista lê**, e uma tabela indexada por
chave nunca põe o `map_range` ao lado dos outros dois. A sonda nova
(`the_vocabulary_the_artist_reads`) imprime **três** listas, porque são três defeitos:
*um rótulo/várias chaves* · *uma chave/vários rótulos* · **as PALAVRAS de um enum**.

### §9.2 — As três curas (rótulos e palavras — zero risco para documento gravado)

| o que o artista via | cura |
|---|---|
| `Interp` (pattern) contra `Interpolation` (table, map_range) | pattern passa a dizer **Interpolation** |
| `Stepped` (map_range) contra `Step` (pattern, table) — a MESMA coisa | map_range passa a dizer **Step** |
| `Stagger` (lfo, time) contra `Phase Stagger` (beat) | os três dizem **Phase Stagger** |

⚠️ **Só o ÍNDICE é guardado**, então trocar a palavra de um enum não toca em nenhum documento.

### §9.3 — ⛔ A acusação do §4 contra o `value.attribute` era FALSA, e a isenção é derivada

O §4 marcava *«o `mode` do `value.attribute` não tem hint nenhum — a única entrada da tabela sem
rótulo»*. Ele **não tem, de propósito**: o picker `Read` é um `ParamWidget::Channels { mode_param:
"mode" }` e escreve os dois, e uma row própria para o `mode` seria a segunda superfície a decidir a
mesma coisa. ⇒ a sonda passou a **derivar** a isenção do widget que dobra o vizinho (o `Channels`
e o `Color`), nunca de uma lista de nomes. *Um censo que acusa o legítimo é apagado na primeira
semana.*

### §9.4 — ⭐⭐⭐ O instrumento que faltava: o CENSO DO ALCANCE

O retrato do ciclo diz *quantos params o cartão pinta agora* — e um `1 de 4` lê-se igual nas duas
leituras opostas: *o nó está bem gateado* ou *três controlos estão enterrados onde a mão não chega*.
⛔ **Nenhuma sonda deste repo fazia a segunda pergunta.** A caça aos knobs mortos de 30/08 segue o
valor até ao efeito (*o painel escreve onde · quem lê · o leitor decide?*); esta vem **antes** dela:
*o artista consegue sequer CHEGAR ao controlo?*

[`motion_param_reach`](../../../../crates/ph2d-app-motion/src/motion_param_reach.rs) resolve um **ponto
fixo** sobre o grafo dos gates — para cada param escondido, existe alguma atribuição dos params que
o gateiam, dentro da faixa que o hint deles declara e só se ESSES forem eles próprios alcançáveis,
que o revele? A resposta sai do **mesmo predicado que o cartão usa** (`Visibility::shows`), nunca de
uma segunda cópia da lei.

**Resultado sobre o catálogo INTEIRO: `0` params enterrados**, e a resposta ao §2 achado 4 é
*«correctamente gateado»* (`value.instance_field` e `value.math` escondem o que pertence a outro
modo).

⚠️⚠️ **E ela mentiu DUAS vezes antes de dizer a verdade — as duas apanhadas por verificação, não por
sorte:**

1. **Acusou onze params do `source.shape`.** Ela lia `hint.max` para a faixa, e o hint do `Shape`
   declara `min: 0, max: 0` deixando a faixa ao widget (`Enum { labels }`) — o censo via um selector
   de **uma** opção e dava por enterrado tudo o que pende das outras. *A faixa de um `Enum` são as
   LABELS.*
2. **Acusou o `source.shape::collider_radius`.** A busca era **gulosa**: escolhia um valor para cada
   `when` de cada vez, e o `shows` é uma **conjunção** — com os outros `when` no default, nenhum
   valor do primeiro satisfaz. Aquele param precisa de `Collide` ligado **E** de
   `Collider Shape = Circle` ao mesmo tempo. *Uma condição com duas metades não se verifica uma
   metade de cada vez.*

⚠️ **O piso de população é metade do gate**: ele lê `0 acusados` quando está são, que é exactamente
o que lê quando está partido. `nos ≥ 100` · `params ≥ 400` · `com_gate ≥ 20`. **Prova de mutação:**
apontar o `ParamGate` do `collider_radius` a um índice que o selector de duas opções não alcança
acusa-o de imediato.

### §9.5 — ⛔⛔ A RECUSA: não há catraca de vocabulário, e o número diz porquê

A tentação seguinte era uma catraca sobre *«um rótulo, várias chaves»*. Corrida sobre o catálogo
inteiro ela acusa **~40 grupos**, e quase todos são legítimos — o `Count` de nove geradores e o
`Count` de um contador são perguntas diferentes que a mesma palavra serve bem; o
`motion.bezier_warp` tem **quatro** *In X* e está certo. ⇒ uma catraca de 40 entradas **não é um
gate, é uma licença** (`CLAUDE.md` §5.0), e decidir se dois nós fazem a *mesma pergunta* é
semântica, não sintaxe. *A sonda fica; o gate não nasce.*

⭐ **O que É gateável, e nasceu:** dentro de **uma secção de um nó**, dois controlos não podem ter o
mesmo nome — ali estão lado a lado sob o mesmo título e o artista não tem como os distinguir.
`two_controls_in_one_section_never_share_a_name`, com piso de população (`≥ 400` params rotulados)
e prova de mutação (dois *Attack* na secção *Envelope* do `pulse.adsr` acusam).

### §9.6 — Os três cartões VAZIOS: decididos, não assumidos

`value.cursor` · `pulse.level` · `pulse.sample_hold` pintam **zero** controlos. O §2 mandava
**decidir**. Medido: os três são nós cuja resposta inteira é o FIO, e o cartão **nomeia os fios**
desde 2026-08-27 (`paint_port_label`, report do dono) — entradas sempre, saídas quando há mais de
uma, e o `value.cursor` é justamente um dos **três** nós do catálogo com duas saídas.

⇒ **um cartão vazio é legítimo quando a interface inteira do nó são as portas, e elas têm nome.**
⚠️ A metade que segura isto vive no painel (`the_card_painter_actually_calls_it`); se um dia os
rótulos de porta ficarem condicionais, estes três passam a cartões em branco.

### §9.7 — O que fica para a W3b

⏳ As **três chaves partilhadas por perguntas diferentes** (`clamp` em `value.map_range`/`value.mix`,
`step` em `value.quantize`/`pulse.counter`, `value` em `value.number`/`value.table`). O precedente
da casa é o `substeps` — *pergunta diferente ⇒ chave diferente* —, **mas ali a chave era LOAD-BEARING**
(o `SUBSTEPS_PARAM` é lido por um consumidor partilhado, a bomba, e duas leis compunham-se em
silêncio). Aqui **nenhum consumidor partilhado lê estas três por nome**: cada `eval` lê a sua, e o
par `(nó, nome)` é inequívoco. ⇒ renomear custaria o valor autorado de todo documento gravado
(um override desconhecido cai no default, em silêncio) e compraria zero. **Fica declarado, não
curado** — e o dia em que uma destas chaves ganhar um consumidor partilhado, ela vira um `substeps`
e a cura passa a ser obrigatória.

---

## §10 — W4: a CONFERÊNCIA do grupo — e o furo que ela tinha

### §10.1 — ⛔⛔ O placar lia `0 aberto` sobre uma população de 29 de 35

A folha **12** abre com *«PULSE / EVENTOS (6 nós)»* e a família tem **9**; a **15** com
*«VALUE (23 nós)»* e a família tem **26**. ⇒ **o placar é derivado das FOLHAS, e uma folha não tem
célula nenhuma para um nó que ela não conhece** — logo os seis lêem `0 aberto` **por ausência**, não
por estarem conferidos.

⚠️⚠️ **E a régua certa não é «o nome aparece na folha»: é a LINHA DA TABELA.** Os três `pulse.*`
novos **nasceram desta folha** — são a cura de três células de família — e por isso são citados nela
como a resposta, nunca como o **sujeito** de uma linha. *Um nó citado como a CURA de uma célula não
foi conferido como nó.*

**Censo sobre o catálogo inteiro** (`probe_unconferred_nodes`, 17 folhas / 134 nós):

| régua | acusados |
|---|---|
| *«o nome aparece algures na folha»* | **6** |
| **«tem uma LINHA cujo sujeito é ele»** | **15** |

### §10.2 — Os seis do grupo, conferidos

Escritos como **§12** das folhas [12](../../89_conferencia/12_pulse.md) e
[15](../../89_conferencia/15_value.md), com a referência citada em cada célula. Resultado: **0 P0 · 0 P1 ·
0 P2**, `3` ✅ e `6` ⛔.

| nó | o que a referência tem e nós não | veredito |
|---|---|---|
| `pulse.adsr` | **`Peak`** (TD Trigger CHOP) | ⛔ — `→ value.math(Multiply)`, um nó |
| `pulse.adsr` | **`retrigger delay`** como TEMPO | ⛔ — é o `debounce` do `pulse.threshold`, a montante ⚠️ *e o `pulse.compare` não o tem: quem gatilha por CAMPO não alcança a janela* |
| `pulse.adsr` | — | ⭐ `SUPERAR:` o **`hold`** é nosso — um gatilho não tem *note off*, logo o patamar tem de ter duração autorada |
| `pulse.level` | os modos do TD **Logic CHOP** | ⛔ — *toggle* e *latch* são `pulse.counter(count_max = 2)`, escrito no header |
| `pulse.signal` | um sinal com **PAYLOAD** | ⛔ — **fronteira declarada**; a contagem de linhas que dispararam já viaja no `SignalOrigin` |
| `value.cursor` | o `followMouse` do Cavalry **dentro do Falloff** | ⭐ `SUPERAR:` como produtor ele alcança **todo param de todo nó**; um toggle por nó só saberia mover um CENTRO |
| `value.number` | uma constante de **TEXTO** | ⛔ — **não existe canal de texto** num fio (`Dim` tem `Scalar`/`Vec2`/…), medido |
| `value.table` | **N fluxos** de um ficheiro (o `.mgjson` da Adobe) | ⛔ — um `value.table` por coluna; a cadeia é linear nos fluxos |

⛔⛔ **E uma célula ficou FORA da tabela de propósito:** o `outside` do `value.table` tem `Hold` e
`Loop`, e a memória diz que um leitor de tabela costuma oferecer mais extensões — **não há
referência CITADA neste repo** para isso. O placar **recusou** a linha sem veredito e tem razão:
*uma linha de conferência sem referência citada é um palpite com cara de medição*, e uma tabela que
a aceitasse passava a contar palpites como dívida medida.

### §10.3 — A catraca que impede isto de voltar a acontecer em silêncio

`every_node_has_a_conference_row_or_is_named_in_the_debt` — **as duas metades**: reprova quando um
nó nasce fora das folhas, **e** quando uma entrada da dívida já não descreve nada. Mais o **piso de
população** (as folhas são lidas em RUNTIME, sem `include_str!` com glob: um caminho partido daria
zero folhas e a lista inteira).

⭐ **Ela já desceu: `15 → 9`**, e quem exigiu apagar os seis foi a própria metade de obsolescência,
em voz alta e a nomeá-los. Os `9` que ficam são de outros ciclos, cada um com a folha a que
pertence (`audio.bands` · `motion.bezier_warp` · `motion.proximity` · `motion.randomize` ·
`motion.sub_uv` · `motion.velocity` · `source.lsystem` · `source.table` · `source.text`).

### §10.4 — A suíte de GPU inteira, depois do `dt` uniform

`210 passed / 2 failed` em `766 s`. **Nenhum dos dois é desta wave:**

| vermelho | medida | veredito |
|---|---|---|
| `value_slope_kernel_matches_the_cpu_on_the_device` | `1.05023384e-4` contra uma barra de `1e-4` — **bit-idêntico** em 2 de 2 corridas e **o mesmo número** já registado no §6 antes desta wave | **pré-existente, determinístico** |
| `crossing_the_reach_boundary_does_not_step_the_cost` | na suíte: `127,0` → `3 264,6 ms`, razão **`25,7×`**. Sozinho, `3 de 3`: `7,0` → `7,0 ms`, razão **`0,998`** | ⚠️ **FLAKE DE CARGA** — membro novo da família |

⚠️⚠️ **A assinatura deste é a mais limpa que esta linha viu:** não é a razão que se move, é o
**relógio absoluto**, `466×` (`7 ms` → `3 265 ms`). Sob o fan-out da suíte a placa está contendida, e
um gate que divide dois relógios de GPU herda isso inteiro. ⇒ **proposta à integração**: acrescentar
`crossing_the_reach_boundary_does_not_step_the_cost`
([`gpu_collide.rs`](../../../../crates/ph2d-gpu-cook/tests/it/gpu_collide.rs)) à família de flakes de
carga do `CLAUDE.md` §5.0 — *a linha pede, o integrador escreve*.

---

## §11 — W5: a MEDIÇÃO — `32 de 35` no dispositivo, e o preço dos três que ficam

### §11.1 — A residência (independente da carga, logo mede-se sempre)

| família | no dispositivo |
|---|---|
| `value.*` | **23 de 26** |
| `pulse.*` | **9 de 9** (eram `0 de 9` antes da W2) |
| **grupo** | **32 de 35** |

Fora: `value.number` · `value.cursor` · `value.table`.

### §11.2 — ⛔⛔ Um `NAO` na coluna do retrato NÃO é um preço

A rota do param dirigido é CPU **por desenho** — a W1a coze o condutor e entrega o NÚMERO —, logo um
`value.*` que só dirige params nunca paga por não ter kernel. ⇒ a sonda
(`probe_what_the_three_off_device_nodes_cost`) põe cada um numa cadeia que chega mesmo ao **stream de
objectos** (`grid → motion.drive → output`, `320 × 320`) e pergunta ao planeador onde é a costura
**e quantos elementos ela carrega**:

| cadeia | stages | costura | elementos que SOBEM |
|---|---|---|---|
| `drive(v = value.lfo)` — o controlo | **4** | — | — (nada sobe) |
| `drive(v = value.number)` | 3 | `value.number:0` | **1** |
| `drive(v = value.table)` | 3 | `value.table:0` | **102 400** |
| `drive(v = value.cursor)` | 3 | `value.cursor:0` | **102 400** |

⭐⭐⭐ **A costura é UM NÓ, não a cadeia** — o resto (`grid`, `drive`, `output`) fica no dispositivo
nos quatro casos. *É a diferença entre a W1a e o que havia antes dela, medida: em Agosto qualquer um
destes derrubava a cadeia INTEIRA.*

### §11.3 — ⭐ O `value.number` fica fora, e isso é grátis

Ele emite **UM** elemento (a regra `1→N` difunde-o a jusante), logo a costura dele é **um `f32` por
quadro**. ⚠️ **O achado do §3 nomeava-o** (*«e a primeira delas era uma CONSTANTE»*) e era sobre o
**PLANEADOR** a recusar a cadeia inteira — o que a W1a curou. *Escrever-lhe um kernel agora compraria
um float por quadro, e o §0.0 manda medir antes de limitar — e também antes de construir.*

### §11.4 — ⛔⛔ E os outros dois sobem **102 400 cópias do MESMO número**

O `value.cursor` e o `value.table` declaram, os dois, *«a cardinalidade segue a geometria»* (o padrão
do `value.lfo`/`value.time`): ligada a porta `in`, eles emitem **um campo de comprimento `N` com o
mesmo número em todas as linhas**. Para o `value.lfo` isso é obrigatório — com a porta `time` ligada
ele dá **um relógio por elemento** —, mas o cursor é global por definição e a tabela devolve **um
instante**.

⇒ **a cura não é um kernel: é o comprimento.** Se os dois emitissem `1`, a costura passava de
`102 400` para `1` — a mesma conta do `value.number`, e a regra `1→N` da casa já existe dos dois
lados (`ColumnAccess::ReadBroadcast` no dispositivo; o `if v.len() == 1 { resize(n, v[0]) }` na CPU).

⚠️ **Não está feito, e o motivo é medido e não preguiça:** *nem todo consumidor honra a regra*. Um
`Read` simples no dispositivo julga um campo de comprimento `1` **ausente** e devolve a identidade —
foi por isso que o `ReadBroadcast` nasceu, e o `pulse.compare` precisou dele esta semana. ⇒ mudar o
comprimento de saída exige **varrer os consumidores de VALOR um a um** e provar que cada um
difunde. **Wave própria, com este número (`102 400 → 1`) como justificação e como barra.**

### §11.5 — O RELÓGIO fica por medir, e a razão é a mesma do ciclo 5

`load 12–18` durante toda a janela (§5.0 pede `≤ 5`). ⚠️ E um A/B device-contra-CPU deste grupo pede
um `GpuContext` no arnês, que já está nomeado como wave própria no §6. *O que se mede sem máquina
calma é a máquina, não o código.*

---

## §12 — W6: a cena `=117` e o TUTORIAL — o smoke do dono

### §12.1 — A cena: quatro quadrantes, e em cada par muda UM NÓ

```text
  EM CIMA   o mesmo pano, o mesmo LFO     só o LFO        →  respira LISO
                                          + Quantize      →  respira aos DEGRAUS
  EM BAIXO  o mesmo pano, o mesmo Beat    só o Beat       →  pisca a CADA batida
                                          + Counter       →  pisca a cada QUATRO
```

⚠️⚠️ **Em cada par muda um NÓ, e não um número** — e há gate a afirmá-lo
(`each_pair_differs_by_exactly_one_node`, que conta os tipos). *Se a diferença fosse um slider, a
cena ensinaria que o grupo é uma gaveta de controlos; ela é sobre o que um nó a mais faz ao número
e ao instante que já lá estavam.*

⚠️ **O pano é `6 × 6` e não `320 × 320`.** A `=116` ensina um CUSTO e por isso tem 102 400 peças;
esta ensina uma LEI, e uma lei lê-se melhor em trinta e seis.

⚠️ **A metade de baixo corre na CPU, e é declarado** (§8.6): o `motion.strobe` ainda não tem
kernel. *Uma cena de ciclo mostra o que o grupo FAZ; a rota tem a `=116`.*

**Cinco gates de comportamento**, e as duas medidas que os sustentam:

| o que o anúncio promete | a régua | medido |
|---|---|---|
| em cima, um respira liso e o outro aos degraus | **valores distintos** que o tamanho toma numa volta | `49` contra **`4`** |
| em baixo, ritmos diferentes | quantas vezes o tamanho **SOBE de repente** | `4` contra **`1`** |

⛔ A régua de cima **não é «os dois mexem»**: uma grelha é um contradomínio mais pequeno, e comparar
dois instantes diria que os dois respiram sem dizer qual salta. A de baixo conta a **subida** e não
o valor — o clarão desvanece, e um limiar sobre o tamanho contaria o mesmo flash muitas vezes.

### §12.2 — ⭐⭐⭐ O gate que nasceu disto apanhou TRÊS passos impossíveis, dois deles meus

`every_card_an_announcement_tells_you_to_click_exists_in_that_scene`: **todo cartão que um anúncio
manda clicar existe na cena que ele anuncia**, derivado do próprio texto (o bloco de cada cena é
delimitado pelo `[cena N]` que ela imprime) e cruzado com os títulos que a cena de facto pinta.

| cena | o passo dizia | o cartão chama-se |
|---|---|---|
| `=116` | `Value LFO` | **`LFO`** |
| `=117` | `Value LFO` | **`LFO`** |
| `=112` | `Field Remap` | **`Remap`** |

⚠️⚠️ **A `=112` é do ciclo 4 e o dono JÁ a smokou e aprovou** — o passo impossível atravessou uma
aprovação. *O nome de um nó no código (`value.lfo`, `field.remap`) e o nome do cartão na tela não
são a mesma palavra, e quem escreve o passo tem o primeiro na cabeça.*

⚠️ **E ele tem a metade NEGATIVA:** a `=114` ensina que o colisor é da FORMA dizendo que **não há**
cartão `Collide` na linha da simulação — uma afirmação tão forte como a outra, e que no dia em que
houver um passa a ensinar o contrário do que se vê. ⛔ A 1.ª redacção do gate reprovou sobre essas
duas frases, que estão **certas**: *uma régua que confunde «clique aqui» com «isto não existe»
acusa o texto correcto.* A extracção passou a ser pela **forma imperativa**, varrida do próprio
ficheiro.

### §12.3 — As figuras: uma FORMA DE ONDA, e é a primeira deste módulo

As quatro saem da cena `=117` do produto — cada ponto é o `size` que o motor deu a uma peça num
instante. ⚠️ **O eixo horizontal é o TEMPO**, o que nenhum tutorial deste módulo tinha desenhado
ainda, e a razão é o assunto: *um número que manda em tudo só se vê ao longo do tempo, e uma
fotografia de um instante mostraria quatro panos iguais.*

⭐ **Na segunda de cada par, o fantasma por baixo é a PRIMEIRA** — é assim que a figura mostra que
os dois lados são o mesmo número e o mesmo ritmo, com um nó pelo meio.

⛔ **A escrita vem depois das asserções** (a lei que o ciclo 4 pagou), e há gate a exigir que as
quatro existam no disco e sejam **diferentes entre si**.

### §12.4 — O tutorial

[`06_valor_e_pulso.pdf`](../../tutoriais/06_valor_e_pulso.pdf) — **5 páginas**, fonte em
[`src/06_valor_e_pulso.html`](../../tutoriais/src/06_valor_e_pulso.html). Sete capítulos: abrir · o número
· o número moldado · o instante · o instante contado · **o mapa do grupo** (uma tabela *«quando você
quer… procure por…»*, porque trinta e cinco cartões não se decoram) · e o que isto custa (a `=116`).

**Gates:** `every_row_the_value_tutorial_names_is_on_the_card` (cada linha que o texto manda
arrastar está no cartão, **na cena**, e o título está de facto no texto) · o comando é
`--profile smoke` e **nunca** `--release` · as duas cenas que ele abre existem ·
`the_four_figures_the_tutorial_shows_exist_and_differ`.

### §12.5 — O smoke, para o dono

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=117 cargo run -p ph2d-host-desktop --profile smoke
```

---

