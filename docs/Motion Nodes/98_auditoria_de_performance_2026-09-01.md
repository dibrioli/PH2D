# 98 — Auditoria de performance do módulo Motion, 2026-09-01

> **A pergunta:** *somos uma game engine e queremos animar milhões de objetos.* Onde está o
> teto, de que recurso ele é, e o que o define hoje?
>
> **A resposta em uma linha:** o **device já faz 4,19 M objetos em 3,85 ms** (23% de um quadro
> de 60 fps) — mas **69,7% das cenas que o produto expõe nunca chegam lá**, e a causa de 67%
> delas é **uma escada que não nomeia recurso nenhum**.

⚠️ **Toda medição desta página traz a carga da máquina ao lado.** Nenhuma leitura de relógio
desta workstation vale acima de `load ~5` (CLAUDE.md §5.0), e esta jornada correu com outra
linha a ocupar um núcleo inteiro. Onde a carga mudou entre duas colunas, está dito.

**Máquina:** RTX 5060 Ti · Ryzen 9 9950X (16 núcleos / 32 fios) · Linux 6.18.

---

## §1 — O TETO, medido

`emitter_sim_ceiling_probe` (`ph2d-gpu-cook`, `#[ignore]`), a MESMA sim tique a tique pelos dois
caminhos. `load 5,48 → 5,03`:

| janela | GPU ms/tique | CPU ms/tique | CPU/GPU |
|---:|---:|---:|---:|
| 4 096 | 0,102 | 0,254 | 2,5× |
| 65 536 | 0,149 | 3,061 | 20,6× |
| 262 144 | 0,314 | 11,159 | 35,6× |
| 1 048 576 | 1,050 | 44,318 | 42,2× |
| **4 194 304** | **3,847** | **195,935** | **50,9×** |

⭐ **O device entrega o pedido.** 4,19 M partículas em 3,85 ms é **23% de um quadro**; a
escala é linear (1,05 ms a 1 M), logo o orçamento de 16,7 ms comportaria **~16 M**. O tecto
real do `motion.emitter` é `MAX_ALIVE = 4 · 1024²`, e ele é **legítimo**: nomeia o recurso
(≈ 370 MB de residência de GPU para as oito colunas em ping-pong) e traz a tabela.

⛔ **A CPU cruza os 16,7 ms entre 262 144 e 1 048 576** — o tecto de tempo-real do caminho de
referência é **~400 k elementos**. Isso está **certo por desenho** (§0.0: *o caminho de
referência só precisa de computar a mesma resposta; quem manda no tecto é o dispositivo*).

⇒ **Toda escada que atira um grafo para a CPU custa ~10× a contagem de objectos que o artista
pode ter.** É por isso que o resto desta auditoria é sobre as escadas, e não sobre kernels.

---

## §2 — ⛔⛔⛔ O ACHADO: 69,7% do produto nunca vê o device

`motion_route_census` (`ph2d-host-desktop`, `#[ignore]`) planeia **as 109 cenas de demonstração
do produto** e pergunta a rota a `gpu_route`, que é a função pura onde a política vive:

| rota | cenas | % |
|---|---:|---:|
| 1. device inteiro | 26 | 23,9% |
| 2. híbrido (prefixo na CPU) | 7 | 6,4% |
| **3. CPU: mais de UM sink** | **73** | **67,0%** |
| 5. CPU: fronteira sem estágio que despache | 3 | 2,8% |
| **device (inteiro ou híbrido)** | **33** | **30,3%** |
| **CPU serial** | **76** | **69,7%** |

⚠️ **Leia os números das cenas, não só a percentagem:** as cenas do device são `1..35`. As
cenas **36 a 109 são TODAS CPU** — a conferência inteira, as folhas de nós, o L-System, o
`bezier_warp`, o `soft_body`, o emissor. *Tudo o que este módulo construiu desde que a escada
existe nasceu do lado lento dela.*

### §2.1 — A escada não nomeia recurso nenhum

O doc da própria função:

> *GPU is opt-in and only for a **single** sink with **no time scopes** — multi-sink and
> `motion.time_remap` recuse to the CPU whole (**F1.1's scope; F2+ territory**).*

⛔ **«F2+ territory» é escopo de wave, não um limite.** §0.0 exige que um limite diga **de que
recurso ele é** e traga a medição; este diz de que *wave* ele é. É a forma pura do caso que o
§0.0 regista: *o caminho mais lento definiu o tecto do mais rápido, no módulo cuja razão de
existir é o mais rápido* — aqui em **67%** das cenas, por **50,9×**.

### §2.2 — E o preço de a levantar está MEDIDO

Para cada cena multi-sink, planeámos **cada sink sozinho**:

| das 73 cenas multi-sink | |
|---|---:|
| **TODOS os sinks já seriam device** | **23 (31,5%)** |
| alguns dos sinks já seriam device | 27 (37,0%) |
| nenhum | 23 (31,5%) |

⭐⭐ **23 cenas estão a UM passo de composição do caminho rápido** — o trabalho de GPU já é
reclamado; o que falta é **dois planos a acrescentar no mesmo buffer de instâncias**, que é
exactamente o que o pump da CPU já faz (`lower_to_instances_onto` *acrescenta*, e o pump limpa
uma vez). Outras 27 ganhariam em parte.

⇒ Isto **não é uma recomendação de arquitectura**: é o preço, medido, de uma decisão que hoje
está escrita como uma nota de escopo. A decisão é do Enio.

### §2.3 — E NADA na tela diz que aconteceu

`GpuOutcome::FellThrough` é consumido pela ponte para correr o pump; **não há toast, não há log,
não há leitura no painel**. O `motion_bridge_readout` lê `gpu_live` apenas para decidir **de
onde amostrar** os digests. Um grafo a 4 M no device e o mesmo grafo num núcleo só **têm a mesma
aparência na UI** — um deles engasga, e o artista não tem como saber porquê.

⚠️ Esta é a metade do achado que **não custa uma wave**: uma linha na leitura do painel a dizer
a rota tornaria os outros dois achados auto-diagnosticáveis.

---

## §3 — O lowering: MEDIDO e CURADO nesta jornada

`lowering_cost.rs` (`ph2d-eval-motion`, `#[ignore]`), mediana de 5 no mesmo processo, metade das
linhas com geometria (o caso misto que uma planta com folhas vectoriais sobre galhos de sprite
de facto produz):

| n | sprite puro | misto: sprite | misto: vector (antes) | misto: vector (**depois**) |
|---:|---:|---:|---:|---:|
| 1 048 576 | 6,96 ms | 12,91 | 13,98 | **7,89** |
| 4 194 304 | 27,83 ms | 41,27 | 56,76 | **23,01** |

*(a coluna «antes» a `load 6,4`; a «depois» a `load 9,0` — o controlo `sprite puro`, cujo código
não mudou, moveu-se `+6%` entre as duas, e a cura mede `2,47×` por cima disso.)*

**A causa:** `row_medium(stream, i)` resolvia `stream.get("geometry_id")` e
`stream.get(VECTOR_PASS_COLUMN)` **a cada chamada**, e os dois lowerings chamavam-na **por
elemento** — dentro de laços que já haviam içado sete outras colunas. O lowering vectorial
repetia ainda um terceiro lookup para o valor cru. Cada um é uma descida de
`BTreeMap<String, _>` com comparação de string, e a resposta **não pode mudar** dentro do laço:
a corrente é imutável.

**O custo isolado** (`per_element_lookup_cost_probe`, serial, 4 194 304 linhas):

| | ms | ns/linha |
|---|---:|---:|
| por elemento (2 lookups) | **49,033** | 11,7 |
| içado (0 lookups) | **1,026** | 0,2 |
| **poupança** | **48,007** | **47,8×** |

⭐ **49 ms — quase três quadros de 60 fps — eram as duas mesmas perguntas repetidas 4,19 M vezes
cada.**

**A cura** (`MediaColumns`, `crates/ph2d-eval-motion/src/lower.rs`): as duas colunas resolvem-se
uma vez; `row_medium` público **delega** na mesma lei (uma porta, dois leitores — senão a ordem
das perguntas ficaria escrita duas vezes e os dois lowerings poderiam discordar sobre a média de
uma linha, que faria a mesma linha desenhar-se duas vezes ou nenhuma). E o lowering vectorial,
que era o **único dos dois a correr num núcleo só**, passou a `par_extend` com `filter_map`
— que preserva a ordem ⇒ saída **byte-idêntica**, com gate a prová-lo dos dois lados do
`PAR_THRESHOLD` (`the_parallel_vector_lowering_is_bit_identical_to_the_serial_one`).

### §3.1 — ⚠️ Uma inferência minha foi REFUTADA pela medição

A 1.ª leitura da tabela dizia *«~73% do custo do caminho misto são os lookups»*, por aritmética
sobre as colunas. **Falso.** Depois do içamento o caminho `misto: sprite` **não se moveu**
(normalizado pelo controlo: `1,48×` antes e depois). O motivo é que **aquele caminho já era
paralelo**: 49 ms espalhados por 32 fios são ~1,5 ms, dentro do ruído de uma medição de 43 ms.
Os lookups dominam um laço **serial**, e é lá que a cura paga.

⇒ *Uma poupança real pode ser invisível na régua errada; e a régua errada aqui era a que
media os dois efeitos somados.* O número que decidiu foi o micro-probe que mede **só** o lookup.

---

## §4 — O catálogo: quem tem caminho de device, e quem paraleliza

| | de 134 crates de nó |
|---|---:|
| declaram `GpuKernel` | **74** |
| **não declaram nenhum** | **60** |
| dos 60, os que **iteram por elemento** | **42** |
| desses 42, os que chamam `par_build` | **1** |
| **desses 42, SERIAIS** | **41** |

Os 41 seriais e por-elemento: `audio-bands` `fx-drop-shadow` `fx-rgb-split` `motion-clone`
`motion-delay` `motion-distribute-poisson` `motion-duplicator` `motion-expression`
`motion-make-point` `motion-mirror` `motion-mixer` `motion-morph` `motion-randomize`
`motion-shape` `motion-slit-scan` `motion-sort` `motion-spline-wrap` `motion-step`
`motion-strobe` `motion-trail` `motion-velocity` `motion-wave` `rig-fabrik` `rig-ik-2bone`
`rig-rubber-hose` `rig-skeleton` `rig-skin-deformer` `source-object` `source-table`
`util-reroute` + os `pulse-*` e `value-*`.

⚠️ **Corrija a leitura antes de acusar:** os `pulse-*` (9) e `value-*` (3) são nós de VALOR —
custo `O(1)`, contagem 1. Não há acusação neles. O que sobra são **~29 nós por-elemento sem
device e sem paralelismo**, e entre eles estão os que um jogo usaria a sério: `motion.trail`,
`motion.duplicator`, `motion.clone`, `motion.mirror`, `rig-skin-deformer`, `source.object`.

⚠️ **E o substrato já existe:** `ph2d_nodegraph::attr::par_build` (`PAR_THRESHOLD = 8192`) é a
costura auditada, **byte-idêntica ao serial** por construção (`collect` indexado do rayon), e
**22 crates** já a usam. Retrofitar um nó é uma linha, quando o nó é um mapa puro. *A questão
não é se dá; é que ninguém voltou.*

### §4.0 — ⭐ Conferido e medido em 04/11: 2 curados, o resto recusado com número

> A tabela acima é de 01/09 e **fica como estava**; o que vale hoje é esta secção. Sondas:
> `sonda_par_build` (por nó, `crates/ph2d-node-registry-init/tests/it/sonda_par_build.rs`) e
> `sonda_par_build_no_quadro` (o quadro do produto com a placa à espera dentro,
> `crates/ph2d-app-motion/src/motion_par_build_quadro_probe.rs`). Variantes: pool de 1 fio × pool
> da fatia (`16` fios sob o `ph2d-run.sh`), no mesmo processo, intercaladas, o mínimo com a mediana
> ao lado. ⚠️ Máquina carregada por outras linhas: `load` na tabela; a régua é a razão dentro de cada
> rodada, nunca o absoluto entre rodadas. ⛔ **A pool de `32` fios do produto não foi medida.**

**O que mudou desde 01/09 e muda a pergunta:**

1. **Registo hoje: `96` tipos com kernel, `44` sem** (lido do `NodeRegistry`; a tabela de cima
   contava crates por `grep`). `motion.trail`, `.clone`, `.mirror`, `.duplicator`, `.slit_scan`,
   `.strobe`, `fx.drop_shadow`, `fx.rgb_split` **ganharam kernel**: a placa dispensa-os.
2. **O tecto por nó (`MAX_INSTANCIAS_POR_NO = 32 768`, ordem do dono em 22/09) cura o ponto 1 do
   §4.2c:** o duplicator já não emite `16 777 216`, e o `motion.clone` também pára ali. ⇒ **`N` real
   é `≤ 32 768`**; só `mirror` (`×2`) e `trail` (`×3`) entregam mais. A 1.ª rodada media `262 144`
   e a cena de quadro «x9» dava os mesmos `32 761`: *essas linhas não existem no produto*.
3. **A GPU por omissão NÃO dispensa um nó sem kernel na ponta.** A rota medida é
   `CPU: fronteira sem estagio de GPU que despache`: **a cena inteira coze na CPU**, e o quadro
   é quase só a `marcha`. Só por esta rota o custo destes nós pesa.

**O quadro, grelha viva no tecto (`181² = 32 761`):**

| cena (ms por quadro: 1 fio → pool, med. entre parênteses) | load `18,9→13,5` | load `10,5→7,1` | load `6,5→4,7` | código final, load `19,7` |
|---|---:|---:|---:|---:|
| referência: toda na placa | `0,271→0,167` | `0,145→0,102` | `0,110→0,104` | `0,104→0,082` |
| ⭐ pele, 16 ossos | `11,93→1,83` (`16,7→9,1`) | `10,99→1,76` (`11,4→6,5`) | `9,06→1,07` (`11,2→3,2`) | `4,40→0,59` (`8,7→0,8`) |
| ⭐ 2 expressões + make_point | `8,73→1,82` (`10,6→10,0`) | `4,74→2,01` (`9,5→3,1`) | `6,89→1,25` (`9,6→3,1`) | `2,97→0,63` (`5,9→0,8`) |
| ⭐ cena `=34` no tecto (4 expressões) | `11,68→9,46` (`13,2→20,3`) | `12,05→4,55` (`16,5→9,9`) | `10,51→2,72` (`12,5→9,5`) | `4,71→1,46` (`10,2→2,0`) |
| cena `=34` de fábrica (`64²`) | `1,03→0,91` | `0,66→0,87` | `0,82→0,86` | `0,56→0,57` (`1,07→1,06`) |

⚠️ **Nenhuma cena de fábrica acelera:** a `=34` tem `64² = 4 096` linhas e a pele da cena `=120` tem
`21`, abaixo do `PAR_THRESHOLD`. O ganho é de quem sobe a grelha.

**Por nó, código final, load `19,6→19,5`, `1 fio → pool`, ms por tique:**

| `N` | `rig.skin_deformer` (16 ossos) | `motion.expression` |
|---:|---:|---:|
| `1 024` (controlo: nada paraleliza) | `0,221→0,216` | `0,119→0,115` |
| `8 192` | `1,622→0,207` (`7,8×`) | `0,931→0,118` (`7,9×`) |
| `32 768` | `6,708→0,703` (`9,5×`) | `3,888→0,338` (`11,5×`) |

**Prova do mesmo bit:** as `54` células da sonda (`18` nós × `3` `N`) dão a mesma impressão FNV do
serial de ANTES da mudança, e o gate `par_build_bit_identico` (`crates/ph2d-node-registry-init`)
compara pool de `1` × `8` a `N = 32 768`. Mutação vista vermelha nos dois nós: um contador partilhado
no lugar do índice (`left ≠ right`). O `motion.expression` só é mapa porque `ph2d_expr::eval` lê o
elemento, o `t` e os params; o `rig.skin_deformer` só lê os ossos.

**Os recusados.** Critério: ganho de quadro `≥ 1 ms` a `32 761` no mínimo E mediana a melhorar
(MEDIR_VELOCIDADE passo 4). Os três primeiros foram **aplicados, medidos e revertidos**:

| nó | mapa puro? | `1 fio` a `32 768` | com `par_build`, no quadro | porquê |
|---|---|---:|---|---|
| `motion.randomize` | sim | `0,98` (posição), `1,56` (matiz) | `1,87→1,24` (med `2,36→2,52`), `2,47→1,48` (med `6,6→9,5`) | o controlo SEM `par_build` já dá `1,46→1,16`: quem ganha é a grelha e o oscilador a montante. ⚠️ O próximo da fila: por nó dava `2,8×`–`3,5×` |
| `motion.step` | sim | `0,98` | `1,20→0,57` (med `1,61→1,27`), `1,49→1,03` (med `1,67→1,96`) | `< 1 ms`, e o controlo sem `par_build` dá `1,5×`: 4 colunas de um `Vec` de tuplos |
| `motion.morph` | sim | `0,71` | `1,10→1,16` (med `1,35→2,17`), `1,08→0,77` (med `1,24→1,90`) | a mediana piora |
| `motion.spline_wrap` | sim | `0,86` | não tentado | `< 1 ms` |
| `motion.distribute_curve` | sim | `0,57` | não tentado | `< 1 ms` |
| `motion.mixer` · `make_point` · `velocity` · `wave` | sim | `0,28` · `0,26` · `0,11` · `0,18` | não tentado | `≤ 1,7 %` de um quadro |
| `motion.delay` | não: anel, `flat`/`unflat` | `1,81` | não tentado | cópia de memória, não cálculo |
| `motion.sort` | não: a ordenação manda | `1,46` | não tentado | só as chaves seriam mapa |
| `motion.scatter` | não: gerador `Pure` | `27,3` | não tentado | **memo**: a sonda muda a semente para o forçar; numa cena coze 1× |
| `motion.trail` | não: anel por cópia | `7,67` (`98 304` linhas) | não tentado | tem kernel; na CPU só por trás de um nó sem kernel |
| `motion.clone` · `mirror` | sim | `0,09` · `0,12` | não tentado | têm kernel e `≤ 0,7 %` |
| `motion.duplicator` | sim | não medido | já tem `par_build` | tem kernel |
| `source.object` · `util.reroute` · `source.table` · `source.shape` | sem laço caro por quadro (`Arc` ou memo) | não medido | — | lidos no código |
| `rig.fabrik` · `ik_2bone` · `rubber_hose` · `skeleton` | não: cadeia de juntas | não medido | — | `N` = juntas, dezenas |
| `motion.distribute_poisson` | não: rejeição em ordem | não medido | — | gerador em memo |

⏳ **Fica por conferir do §4.2c:** o ponto 2 (a truncagem muda) não foi visto nesta passada.

### §4.1 — Os `O(N²)` da CPU, e por que estão certos

`motion.boids` (`for i in 0..n { for j in 0..n }`), `motion.collide` e `motion.proximity`
(`for j in (i+1)..n`) são força bruta na CPU. **Os três registam `GridSpec` no device** — a
grelha de vizinhança existe, do lado que manda. Pela lei do §0.0 isto está **certo**: a CPU é a
referência e pode demorar o que um teste precisar.

⛔ **Deixa de estar certo no instante em que o grafo é atirado para a CPU pela escada do §2** —
e uma cena de flocking com dois sinks é exactamente o que um jogo autora.

---

## §4.2 — ⛔⛔⛔ O report do Enio, no mesmo dia: `motion.duplicator` depois do emissor

> *«o simples facto de tentar colocar um duplicator logo após o Emitter já quebra a cena … trava
> … automaticamente o fio do emitter entra no input errado do duplicator (o da shape ou objeto)»*

**São TRÊS defeitos e um deles é meu.** A reprodução é `the_enio_duplicator_after_emitter`
(`shells/desktop`, `#[ignore]`), que monta o gesto REAL — o mesmo `splice_node` que o menu chama.

### (a) O fio entrava na porta errada, e o TIPO não podia acusar

`splice_into_wire` ligava **sempre à porta 0**. Para 133 dos 134 tipos isso está certo. Para o
`motion.duplicator` a porta 0 é `shape` e a 1 é `points` — **as duas `INST_VEC2`**, logo o
`validate` do trial aceita ambas e o erro é **silencioso**.

⇒ side-metadata no **REGISTRY**, nunca no contrato (§6: o `NodeManifest` está congelado):
`NodeRegistry::register_primary_input`. Ausente ⇒ `0`, o literal que estava cravado ⇒ os outros
tipos ficam byte-idênticos. Gate `a_spliced_wire_enters_the_port_the_type_declares`, com as duas
metades e as duas mutações mortas.

⚠️ **CENSO:** **32** tipos têm a porta 0 a partilhar o tipo com outra entrada — a população onde
nem o tipo nem o `validate` desambiguam. Na quase totalidade a porta 0 chama-se `in`/`in0`/`a` e
**está certa**; o irmão claro do duplicator é o **`value.switch`**, cuja porta 0 é `select` (um
fio inserido ali vira o CONTROLO, não os dados). ⏳ Não declarado — falta a medição do gesto.
⚠️ A 1.ª régua deste censo dizia **74** porque a regex apanhava as portas de SAÍDA.

### (b) A cena ficar VAZIA é aceitável — porque o app diz porquê

Medido: com o fio nos `points`, a saída é **0 linhas**. Isso REFUTOU a minha 1.ª leitura de que a
mudança era obviamente melhor. O que a torna defensável é que o `Deficit::MissingInput` **já
existia** e o duplicator **já declara** `shape`/`points` como obrigatórias ⇒ o selo ⚠ aparece
(`MissingInput("shape")`, verificado na sonda) com cura clicável. E fica a **um** fio do que o
artista quer, contra três pela porta antiga.

| | linhas emitidas |
|---|---:|
| só o emissor | 21 |
| depois do splice (fio nos `points`) | **0** + selo ⚠ `MissingInput("shape")` |
| emissor nas DUAS portas | 441 |

### (c) O CONGELAMENTO: um orçamento que a máquina não consegue pagar

`points_within_budget` honra `RECOMMENDED_MAX_ELEMENTS = 1 << 24` **saturando o produto no
tecto** — qualquer fonte de tamanho médio dá 16,7 M elementos. E `motion.duplicator` **não
declara `GpuKernel`** e era **serial** (um dos 41 do §4).

| formas | pontos pedidos | aceites | total | antes | **depois** | % de um quadro |
|---:|---:|---:|---:|---:|---:|---:|
| 512 | 512 | 512 | 262 144 | 1,9 ms | 1,9 | 11% |
| 4 096 | 4 096 | 4 096 | **16 777 216** | 133,6 ms | **44,9** | **270%** |
| 78 124 | 78 124 | **214** | 16 718 536 | 132,5 ms | **45,5** | 273% |
| 262 144 | 262 144 | **64** | 16 777 216 | 129,6 ms | **47,5** | 285% |

⭐ `par_build` nos cinco laços (`pairs_for`, `pos`, `rot`, `Index`, `spread`) dá **3,0×** — ⚠️ e
**não 16×**, porque a lista de pares é `Vec<(usize,usize)>` de 16,7 M = **268 MB** percorrida
quatro vezes: o nó é **limitado por largura de banda**, não por cálculo.

⛔ **NÃO CURADO, e são dois** (04/11: o 1 está curado pelo tecto por nó, §4.0):
1. **O tecto continua a custar 2,7 quadros.** `1 << 24` nomeia *«a multiplicação não pode
   estourar a alocação»* — um recurso REAL, mas **não o orçamento do quadro**, e nada guarda esse.
   ⛔ Baixar o número é decisão de produto: ele já trunca, e ver o ponto 2.
2. **A truncagem é MUDA.** 78 124 pontos pedidos ⇒ **214** aceites (`0,27 %`), sem nada na tela.
   *Um knob que entrega 0,27% do que lhe pedem e não o diz é um knob que mente* — e a casa tem o
   vocabulário para o dizer (o `Deficit` com selo ⚠), que este caso não usa.

---

## §4.3 — ⛔⛔⛔ O 2.º report: *«entrou em points corretamente mas a simulação morreu»*

Com a porta curada (§4.2a) o fio entra onde deve — e a simulação **para**. Medido
(`a_simulation_survives_a_duplicator_in_the_middle`, a cadeia da cena `=5` com o duplicator
enfiado no `rest`):

| colunas que chegam ao integrador | |
|---|---|
| **sem** duplicator | `Count · Index · P · age · id · life · size · vel` |
| **com** duplicator (antes) | **`P`** |
| **com** duplicator (depois) | `Count · Index · P · age · id · life · vel` |

**Sete das oito desapareciam.** Sem `vel` não há o que integrar; sem `id` o integrador não
reconhece a partícula do tique anterior — e o doc-comment do `motion.integrate` **já avisava**
que um emparelhamento posicional *«daria a cada partícula a velocidade de uma estranha»*.
Queda máxima após 40 tiques: `0,782` sem o nó, **`0,000`** com ele.

**A causa:** `Transfer::ShapeWins` (o default) saía **antes do laço** de transferência. Mas
ler os quatro doc-comments do enum lado a lado mostra a assimetria — três dizem *«uma coluna
só-do-ponto chega»* e o default deitava-a fora:

> ⭐ **Um modo que resolve CONFLITO não decide sobre uma coluna que ninguém disputa.**

⇒ o modo inerte mantém o que o nome promete (a forma vence a coluna **disputada**) e deixa de
descartar as **não disputadas**. Queda com o duplicator: **`0,782`**, idêntica à do controlo.

⚠️ **O que muda em arte já gravada:** um documento cujos PONTOS trazem uma coluna exclusiva
(o caso que o próprio módulo mediu em 2026-08-26 — uma rampa de cor autorada nos pontos, cujo
`tint` sumia) passa a mostrá-la. *A perda «mantinha a arte de pé — de pé e muda.»* A pergunta
que este módulo deixou ao dono — **de quem é o valor quando os dois lados a têm** — fica
intacta, com o mesmo default.

⚠️ **E um GATE codificava o defeito como contrato** (`the_default_mode_is_the_world_that_always_shipped`,
com *«a coluna só-do-ponto NÃO chega no modo de sempre»* escrito num `assert`). Ele foi
reescrito, não isentado: a metade verdadeira (a forma vence a disputada) fica.

⛔⛔ **E a lição de régua:** havia **quatro** gates sobre o `transfer`, todos verdes, com a
cadeia inteira morta. Eles perguntavam *«o modo faz o que diz?»*; nenhum perguntava *«a
simulação ainda anda?»*. O gate novo mede o **sintoma do artista** e a barra é o próprio
caminho sem o nó. ⚠️ E a 1.ª fixtura dele usava `motion.make_point`, que sem portas ligadas
emite **zero** — com `ns == 0` o duplicator devolve a forma tal e qual, e a fixtura **mediu a
cura como inútil**.

---

## §4.4 — O 3.º e o 4.º report, e a VARREDURA que o Enio pediu

### (a) *«size random para de funcionar»* — o knob EXISTE

Medido (`which_emitter_properties_survive`): das oito colunas do emissor, **só o `size`** não
chega — e volta **inteiro** (`0,107 .. 1,887`, a faixa exacta) com **`Point Scale = 1`**, que é
um controlo que já está no painel.

⛔ **Não é um defeito, é o DEFAULT** (`point_scale = 0`) — e a cura óbvia foi **construída,
medida e REVERTIDA**: fazer a escala do ponto passar quando a forma não tem `size` apaga o peso
interpolado do knob, e **dois gates existentes apanharam-no** (`a_point_scale_of_zero_leaves_the_stamp_exactly_as_it_shipped`
e `the_points_scale_reaches_the_stamp_and_the_weight_interpolates`). *Eles não codificavam o
defeito — guardavam uma feature medida.* ⇒ a pergunta que fica é se o default deve ser `1`
agora que **tudo o resto** chega por omissão (§4.3), e essa é do dono.

### (b) *«Shape em Emitter é redundante»* — é o RÓTULO, não o param

O `Shape` do `motion.emitter` é o **`shape_mode`** — *onde a partícula NASCE* (ponto / caixa),
no grupo **Origin**. Nada tem a ver com a geometria que o duplicator carimba. ⏳ O que colide é
o rótulo, e o doc-comment do próprio param já dizia *«`shape_mode`, not `shape`: the artist
reads the LABEL»* — e depois escolheu a palavra que colide.

### (c) *«lfo nem oscillator atuam sobre Angle de Rotate»* — DOIS defeitos diferentes

Medido (`what_can_drive_the_rotate_angle`), o `rot` da corrente ao longo de 8 instantes:

| condutor | tal como sai da caixa | com amplitude 90 |
|---|---|---|
| `value.lfo` | **`−0,95 .. 0,95`** | `−85,6 .. 85,6` |
| `motion.oscillator` | — | **⛔ `rot` fica em `0`** |

1. ⏳ **O LFO FUNCIONA** e parece partido: `amplitude` nasce em **`1`**, e a conduzir um
   ângulo em GRAUS isso é ±1°. A row veste a faixa do destino (`−180..180`, o
   `motion_bridge_params_wire` já o faz), logo é legível — mas a primeira impressão é *«não
   faz nada»*. ⇒ decisão do dono: um condutor deve **semear** a magnitude a partir da faixa
   que passa a vestir?
2. ✅ **O oscillator ligava e não fazia NADA.** `Graph::drive_param` verifica **só** que os
   nós existem e que não se fecha um ciclo — não tem registry, logo **não pode** ver que o
   `motion.oscillator` emite `Instances/Vec2` (uma corrente por-elemento) e que um param lê a
   coluna `v`, que só uma saída **escalar** produz. *Um fio VISÍVEL que não age é pior que um
   knob morto.* ⇒ a 2.ª recusa entra na porta do GESTO (`subgraph::drive`), com a **mesma**
   régua que o menu usa. Gate `only_a_value_output_may_drive_a_param`, mutação morta.

### (d) A VARREDURA — *«conferir estes bugs nos outros nós»*

`which_nodes_drop_a_streams_columns`: um `motion.emitter` numa entrada, uma grelha na outra, e
conta-se quantas das **quatro** colunas exclusivas (`id · vel · age · life`) sobrevivem.

| tipo | emissor na porta 0 | emissor na porta 1 |
|---|---|---|
| `motion.duplicator` | ok | ok *(curado nesta jornada)* |
| `motion.combine` · `motion.morph` | ok | ok |
| **`motion.mixer`** | **⛔ perde as 4** | **⛔ perde as 4** |
| **`motion.lattice`** | **⛔ perde as 4** | ⛔ |
| `motion.clone` · `spline_wrap` · `rig.skin_deformer` · `field.shape` | ok | ⛔ |
| `motion.integrate` | ok | só `vel` |

⚠️ **A leitura é o achado:** perder na **porta 1** é quase sempre **CERTO** — ali entra uma
jaula, um caminho, um molde, as forças. O que acusa é perder na **porta 0**, o fluxo que
atravessa: ⏳ **`motion.mixer`** e **`motion.lattice`** ficam nomeados, por medir caso a caso.

⚠️ E a 1.ª versão desta sonda punha uma **grelha dos dois lados** — que não autora coluna
exclusiva nenhuma — e a tabela inteira saía igual: *a fixtura não continha o fenómeno*.

---

## §5 — Os tetos: auditados contra o §0.0, e estão bons

| teto | valor | nomeia recurso? |
|---|---:|---|
| `motion.emitter::MAX_ALIVE` | 4 194 304 | ✅ residência de GPU, ≈370 MB, com tabela |
| `motion.voronoi::MAX_RES` | 1 625 | ✅ representação: `Σgx ≤ res³` em `u32`, `1625³ < 2³²` |
| `motion.voronoi::MAX_POINTS` | 165 000 | ✅ a lei de amostragem, + tabela medida na RTX |
| `motion.trail::MAX_INSTANCES` | 262 144 | ✅ relógio (⅔ de um quadro) + memória, e gate a partilhá-lo com os dois `fx.*` |
| `eval-motion::CPU_RING_BYTES` | 128 MB | ✅ orçamento em bytes, com a política de desbaste |

⭐ **Este módulo levou o §0.0 a sério nos tetos.** O buraco não está nos números que alguém
escreveu — está na escada que **ninguém escreveu como número**.

---

## §6 — O que fica ABERTO, por ordem do preço medido

1. ⛔⛔⛔ **O multi-sink no device** — 67% das cenas, `50,9×`, e **23 delas a um passo de
   composição**. Wave com desenho próprio (dois planos, um buffer; que significa a ORDEM dos
   sinks no device; o memo partilha a montante?). **Decisão do Enio.**
2. ⚠️ **A rota tem de ser VISÍVEL** — barata, e torna as outras duas auto-diagnosticáveis.
3. ✅ **Os ~29 nós por-elemento seriais** — fechado em 04/11 (§4.0): `rig.skin_deformer` e
   `motion.expression` curados; o resto recusado com a tabela. Reabra só com medição nova.
4. ⏳ **Os nós sem kernel** (`44` tipos em 04/11, §4.0) — cada um que ganhe um kernel encolhe a
   fronteira híbrida.
5. ⏳ **As 3 cenas da escada nº 5** (`=24`, `=27`, `=34`): fronteira cujo único estágio de GPU é
   o `output` passa-through. Diagnosticar caso a caso.

---

## ⛔ Recusas MEDIDAS desta auditoria

| recusa | mecanismo |
|---|---|
| *«os lookups são ~73% do caminho misto»* | **Refutado**: aquele caminho já era paralelo; normalizado pelo controlo não se moveu (§3.1) |
| *«os tetos do módulo são palpites»* | **Refutado**: os cinco auditados nomeiam o recurso e trazem tabela (§5) |
| *«a CPU ser `O(N²)` em boids/collide/proximity é um defeito»* | **Não é**, enquanto o grafo for para o device — os três registam `GridSpec` (§4.1) |
| *«1 de 134 crates de nó usa rayon»* | **Régua errada**: os nós não dependem de `rayon`, chamam `par_build` do substrato — são **22** (§4) |
| *«pôr o fio do duplicator nos `points` é obviamente melhor»* | **Refutado e depois re-justificado**: a saída fica em **0 linhas**. Só se sustenta porque o selo ⚠ `MissingInput("shape")` já existia e aparece (§4.2b) |
| *«32 tipos têm a porta 0 errada»* | **Não**: em quase todos a porta 0 é `in`/`in0`/`a` e está certa. O irmão claro é o `value.switch` (§4.2a). E a 1.ª contagem, **74**, media as portas de SAÍDA |
| *«paralelizar o duplicator resolve o congelamento»* | **Não**: `3,0×`, e sobram `2,7` quadros. Ele é limitado por LARGURA DE BANDA (268 MB de pares), não por cálculo (§4.2c) |
| *«`par_build` em todo mapa puro: é uma linha e é byte-idêntico»* (04/11) | **Não**: `motion.randomize`, `.step` e `.morph` foram aplicados e revertidos. Por nó dão até `3,5×`, mas no quadro o ganho não passa do controlo sem `par_build` (§4.0) |
| *«a GPU por omissão já dispensa estes nós»* (04/11) | **Não**: um nó sem kernel na ponta põe a cena inteira na CPU (§4.0) |
| *«medir a `262 144` linhas»* (04/11) | **Régua errada**: nenhum gerador passa de `32 768` desde 22/09, nem o duplicator (§4.0) |
