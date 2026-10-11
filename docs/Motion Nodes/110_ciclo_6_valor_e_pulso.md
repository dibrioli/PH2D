# 110 — CICLO 6: VALOR & PULSO, o cérebro

> **Índice** — partido em 2026-10-11 (`doc-split`, sha256). Vivo: o veredito (§13) e o fecho dos abertos
> (§14). O resto, verbatim, em `archive/110_ciclo_6_valor_e_pulso/` (cada pedaço ≤ 40 KB).

| § / Bug | Onde |
|---|---|
| §1–§5, §6 (W1a), §7, §8 (W2, §8.1–§8.6) | [`parte_1`](archive/110_ciclo_6_valor_e_pulso/parte_1.md) |
| §9 (W3a, §9.1–§9.7), §10 (W4), §11 (W5), §12 (W6, §12.1–§12.5) | [`parte_2`](archive/110_ciclo_6_valor_e_pulso/parte_2.md) |
| §13, §14 (§14.0–§14.2) | **neste doc** (vivo) |

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, e o **tutorial é o smoke**.
> **Premissa do tutorial:** *«Um número que manda em tudo»*.
>
> ⚠️ Este doc é o do CICLO. O que ele mede vive nas sondas de
> [`motion_valor_probe.rs`](../../crates/ph2d-app-motion/src/motion_valor_probe.rs); o mecanismo de
> cada wave vai para o handoff dela.

---

## §13 — ✅ O VEREDITO DO DONO (2026-09-15): o ciclo 6 FECHA

Ele correu a cena `=117`, leu o [tutorial](tutoriais/06_valor_e_pulso.pdf) e aprovou os dois —
*«tuto ok»* e *«smoke OK»*. Pelo [doc 103 §1](103_dinamica_dos_ciclos.md) passo 7, é isto que fecha
um ciclo; a tabela da §5 daquele doc está marcada.

⚠️ **O que FECHA é o ciclo, não a lista de abertos** — os quatro itens nomeados na nota de estado do
doc 103 (W1b · W3b · o relógio da W5 · as 102 400 cópias) continuam vivos, cada um com o mecanismo
escrito acima. *Um ciclo aprovado com abertos declarados e um ciclo sem abertos leem-se igual numa
tabela de ✅, e só um deles é verdade.*

⭐ **E uma pergunta do dono, respondida e registada por valer para quem desenhar o ciclo 7:** *«além
de Strobe, o `Beat` pode ser usado para mais o quê?»* — os consumidores de um pulso são **oito**, e
saem do catálogo (`PortSpec { ty: PULSE }` do lado da ENTRADA, ⛔ nunca de uma lista escrita à mão):

| cartão | o que a batida faz lá |
|---|---|
| `motion.strobe` | o flash que decai (o que ele já conhecia) |
| `motion.step` | um degrau **persistente** por batida — o `strobe` volta, este acumula |
| `sim.spawn` | **nascimento** em rajada dentro de uma zona |
| `pulse.counter` | a batida vira um **inteiro**, que por fio manda em qualquer param |
| `pulse.level` | a batida vira `0`/`1` (a metade *nível* do vocabulário) |
| `pulse.sample_hold` | a batida **amostra** um valor contínuo e segura-o até à próxima |
| `pulse.adsr` | a batida vira uma **curva** (o envelope no domínio de VALOR) |
| `pulse.signal` | a batida **grita um nome** para fora do grafo |

⚠️ **O `sim.lifetime` NÃO está na lista, e é o engano fácil:** o `pulse` dele é uma **saída** (*«esta
linha morreu»*), não uma entrada — os dois lêem-se iguais num `grep` por `"pulse"`.

---

## §14 — O FECHO DOS ABERTOS (2026-10-07)

> Ordem do dono, 07/10: *«resolver num único ciclo o Ciclo 6 completo»*. O ciclo está ✅ desde 15/09
> (§13); o que fecha aqui é a LISTA de abertos — sete, contando os quatro da nota do doc 103 §5 e os
> três que este doc deixou escritos (o condutor na placa, o vermelho a um ULP, o consumidor do pulso).
> ⚠️ **Escrito ANTES de construir**, com o estado medido no código de hoje (três semanas e várias ondas
> passaram por estas crates). Cada item diz a barra e o critério de abandono; o resultado vem nas
> subsecções seguintes, que o substituem por número.

### §14.0 — O estado MEDIDO em 07/10, antes de uma linha

| # | aberto | estado no código de 07/10 | número |
|---|---|---|---|
| 1 | W1(b) — porta ≠ 0 | **aberto**: a recusa vive em [`plan.rs`](../../crates/ph2d-gpu-cook/src/plan.rs) (`e.from.1 != 0`), `StreamOp::Compact` não tem complemento, `GpuSource::Stage` não tem porta | população: `sim.lifetime` (`died`=1, `pulse`=2) e `pulse.counter` (`carry`=1, que o kernel NÃO calcula) |
| 1′ | ⛔ a RÉGUA do item 1 | o `motion_route_census` planeia com o mapa de fios **VAZIO** — não é a porta do produto: a `=116` e a `=117` contam-se «CPU» e no app correm na placa | 58 inteiras · 57 híbridas · 11 «fronteira sem estágio» · 1 escopo · 1 pede CPU (128 cenas) |
| 2 | o condutor na placa | **zero leituras de volta hoje** — o condutor é COZIDO na CPU (`valores_dirigidos`) e o número entra pelo uniform. A «volta pela CPU» é uma cozedura, não um *readback* | e existe condutor CARO no catálogo: `value.reduce`/`percentile`/`median`/`normalize` leem o fluxo inteiro |
| 3 | W3b — três chaves | **zero consumidores partilhados** (varrido: bomba, cozedor, avaliação, kit, painel, timeline, shell); rótulos chaveados por `(tipo, param)`; e o `value` do `value.table` é um param de **TEXTO** (o nome da coluna) — nem é o mesmo mapa do `value` numérico do `value.number` | 0 |
| 4 | as cópias do mesmo número | **aberto, e o número MUDOU**: a grelha tem lado máximo `181` desde `d60a8398c` (o tecto de 32 768 por nó, ordem do dono) | **32 761** cópias (era 102 400) |
| 5 | o relógio do ciclo | **sem instrumento** — a sonda do preço foi apagada (§6) | — |
| 6 | o vermelho a um ULP | **o mesmo número, ao bit**, de 11/08 e de 15/09; a barra é a do commit do próprio kernel (`85def0fa2`, 26/07), nunca recalibrada | `1,05023384e-4` contra `1e-4` |
| 7 | o consumidor do pulso | `sim.spawn` **ainda recusa** a porta `pulse` (`RefuseIfPresent`, [`kernel.rs`](../../crates/ph2d-node-sim-spawn/src/kernel.rs)); o `motion.strobe` JÁ tem kernel (ciclo 7), o `motion.step` não | o gate `the_chain_that_opened_the_wave_is_blocked_by_the_spawn_and_no_longer_by_the_metronome` afirma a recusa |

### §14.1 — O desenho e o critério de abandono de cada um

**(1) O complemento de um `Compact`.** `StreamOp::Compact` ganha `complement: &[ComplementPort]` —
cada porta diz o que leva: `Rows` (as linhas que o predicado tirou, TODAS as colunas, como o `died`)
ou `Event(col)` (uma coluna de `1.0`, como o `pulse`). Na placa é a **mesma** varredura: o passe novo
escreve `crow[i − scan[i]] = i` onde a bandeira é `0`, e a contagem é `n − total` da MESMA leitura de
8 bytes — zero leituras a mais. No plano: `GpuSource::StagePort(nó, porta)` (variante NOVA; o
`Stage(nó)` continua a ser a porta 0, logo nenhum `match` que existe muda de sentido), a recusa
`e.from.1 != 0` fica só para a porta que o nó NÃO declara (o `carry` do `pulse.counter` continua a
recuar), e a forma da saída passa a ser por porta. **Barra:** `spawn → lifetime → died/pulse → …`
`fully_gpu`, paridade CPU × placa ao bit nas duas portas, e o censo ANTES/DEPOIS pela porta do
produto (1′ curado primeiro). **Abandono:** se pedir uma segunda leitura de volta, ou mais que um
passe além do *gather* por coluna.

**(2) O condutor na placa.** O número só pode ficar na placa quando ele **só entra no uniform**: um
param que uma lei do hospedeiro lê (a contagem, a variante, o `applicable`, as varreduras) precisa
dele na CPU. Desenho: o plano passa a caminhar também as fontes dos params; um condutor elegível vira
estágio ANTES do consumidor, e o consumidor guarda `(param, condutor)`; o sequenciador copia 4 bytes
(`v[0]` do condutor → o campo do param no uniform do consumidor) quando a contagem do condutor é
`≥ 1` — `0` deixa o override/default que o uniform já tem, que é a lei do `None` (§6). ⚠️ Um param
que o hospedeiro lê sem o plano saber tem de RECUAR, nunca dar o número velho: a porta
`resolve_param` é única, e ela recusa o quadro se lhe pedirem um nome dirigido pela placa. **Série
declarada:** o critério deste item é o PREÇO do condutor caro na CPU, que só o instrumento do (5)
mede — logo o (5) vem antes. **Abandono:** se um condutor caro (`value.reduce` sobre a grelha a
mexer-se, no tecto) custar na CPU menos de `0,1 ms` por quadro, a cura não compra nada e fica recusada
com esse número.

**(3) As três chaves: FICAM, com catraca.** Medido: zero consumidores partilhados; a regra da memória
[chave e texto](../../project-memory/feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting.md)
manda perguntar *«alguém faz o caminho INVERSO a partir desta string?»* — e faz: o documento gravado
lê o nome de volta. O nome é DADO (formato de ficheiro); a palavra que o artista lê já é outra coisa
(`node.<tipo>.param.<p>`, e já difere: *Increment* × *Step*). ⇒ renomear + migrar pagaria um degrau
do `PROJECT_SCHEMA` (número que soma entre linhas: colide com toda linha aberta), uma migração por
tipo de nó e a reescrita das cenas — e compraria **zero**, porque nenhum leitor confunde os dois.
**A catraca:** fora das crates donas, dos testes e das cenas, o número de leituras de param POR NOME
destas três chaves é `0`; o primeiro reprova e manda ler esta secção (o dia em que vira um
`substeps`, a cura passa a ser obrigatória e com migração).

**(4) O comprimento `1`.** Primeiro o CENSO dos consumidores de VALOR, derivado do registo (toda
porta de entrada `Instances·Scalar`), cada um provado a dar o MESMO resultado com o campo de
comprimento `1` e com `N` cópias — na CPU e, se tem kernel, na placa. Só então o `value.cursor` e o
`value.table` emitem `1`. ⚠️ **E a porta `in` deles fica MORTA** (ela só servia para dar o
comprimento) — um encaixe que não faz nada no cartão é um controlo que mente; o destino dela decide-se
pelo que acontece a um documento gravado com um fio nela. **Barra:** a costura `32 761 → 1`, a `=117`
e o tutorial 06 ao bit. **Abandono:** um consumidor que não difunda e cuja cura não caiba nesta onda —
fica nomeado e o comprimento NÃO muda.

**(5) O relógio.** Uma sonda intercalada no MESMO processo (o método do
[MEDIR_VELOCIDADE](../DevOps/MEDIR_VELOCIDADE.md)), pela porta do produto (a ponte), sobre a cena do
fio no tecto: variantes = a rota da placa com o fio, a rota da CPU (o interruptor do §6 como flag em
execução, não variável de ambiente), e o condutor caro. **O controlo:** cada variante imprime a rota
que a ponte registou — a metade «placa» tem de ler `FullyGpu`, senão a linha não conta.

**(6) A barra a um ULP.** Medir por estágio (`noise.v`, `slope.v`, `drive.P`) e em ULPs da grandeza
de cada coluna; a barra decide-se pelo VALE entre o ruído medido (o pior desvio legítimo) e o defeito
mais pequeno que ela tem de apanhar (uma mutação real do kernel) — nunca subindo o número até passar.

**(7) O nascimento por pulso na placa.** É um `Compact` disfarçado: as linhas do modelo que
dispararam saem da mesma varredura (bandeira `pulse > 0,5` → *scan* → as linhas densas), a contagem
`F` vem pela MESMA leitura de 8 bytes do `Compact` (precedente com preço medido), e daí em diante tudo
é do hospedeiro como na taxa: `C = min(F·burst, 256)` candidatos, os ids são função do tique, e o
sorteio da probabilidade é a MESMA busca de posto que o kernel já faz para a taxa. O empurrão
(`burst_speed`) usa o gêmeo WGSL do `cos_sin_cycles` que o `motion.emitter` já tem. **Barra:**
`grid → pulse.beat → sim.spawn` `fully_gpu`, paridade ao bit (ids, linhas, colunas herdadas, `vel`),
e o gate que afirmava a recusa vira o gate da cura (com a nota reescrita, §0.0). **Abandono:** se o
`vel` não sair ao bit (o trig), o nó fica na CPU com o desvio medido escrito aqui.

### §14.2 — O que se mediu e o que se fez (07/10)

> ⚠️ **Duas correcções ao §14.1, ditas antes do resultado:** o (4) mudou de desenho por medição (o
> comprimento É significado em 5 consumidores — ver abaixo), e o critério do `vel` no (7) passou
> de *«ao bit»* para *«o ε do seno parabólico da casa»* (a placa pode fundir `a·b + c`, e o
> `motion.emitter` vive com o mesmo ε desde sempre). Os dois estão escritos aqui, não escondidos.

**A régua primeiro (1′).** O censo de rota passou a planear pela PORTA DO PRODUTO
(`plano_do_produto`, a mesma função que o `cook_gpu` chama — uma porta, dois chamadores). Só
isso moveu três cenas (`=116` e `=65` sobem a inteiras; `=117` sobe a híbrida). E ganhou a secção
*«portas ≠ 0 na ancestralidade»* — a população real eram **4 cenas, e nenhuma era só do
`sim.lifetime`**: `=25`, `=26` e a **`=117` do próprio ciclo** estavam presas pelo `carry` do
`pulse.counter`, e a `=27` pelo `sim.lifetime → sim.spawn`.

| # | resultado | número | gates |
|---|---|---|---|
| 1 | ✅ **duas espécies de porta ≠ 0**: o COMPLEMENTO de um `Compact` (`died`/`pulse` do `sim.lifetime`: o passe `rows[i − scan[i]] = i` sobre a MESMA varredura, contagem `n − total` da MESMA leitura de 8 bytes) e a PORTA PROJECTADA (`ProjectedPort`: o kernel do `pulse.counter` escreve `cp_carry`, o sequenciador tira-o da porta 0 e entrega-o na 1 como `pulse`). Um `pre` de porta ≠ 0 recua sempre (o estado só guarda a porta 0) | censo pela porta do produto: **60 → 65** cenas inteiras na placa (`=24 =25 =26 =27 =117`) — ⚠️ **64** depois de a `=117` ganhar ESTRELAS (ela passa a híbrida: a forma vem da CPU, um elemento fixo, e o resto corre na placa); presas por porta ≠ 0: **4 → 0**; os mortos saem **byte a byte** (96 tiques); o `carry` ao bit nos três modos, com reset | `the_dead_port_is_claimed_by_the_device` · `a_delayed_edge_from_a_complement_port_still_recedes` · `the_dead_leave_on_the_device_as_they_were` · `a_counter_with_its_carry_wired_stays_on_the_device` · `parity_pulse_counter_carry` |
| 2 | ✅ **o condutor na placa**, por uma porta NOVA (`plan_with_device_drivers`; a `plan()` com mapa vazio continua a lei antiga ao bit): o condutor é estágio antes do consumidor, e o sequenciador copia 4 bytes (`v[0]`) para o campo do uniform depois do `write_buffer`. Só para um consumidor de MAPA PURO (nenhuma lei do hospedeiro lê o param — critério estrutural, o `motion.move` com variantes fica de fora). **Zero leituras de volta; a CPU não coze o condutor** | condutor caro (`reduce(Mean)` do comprimento de `P` de uma grelha viva no tecto), mesmo processo: CPU-lado **0,199 → 0,125 ms** (−37 %); parede com a espera da placa **0,287 → 0,265** (−8 %: a média passa a correr na placa) | `the_device_driver_is_staged_before_its_consumer` · `a_param_a_host_law_reads_is_never_device_driven` · `the_device_driver_reaches_the_uniform_without_the_cpu` · `an_expensive_driver_reaches_the_uniform_without_the_cpu` · `an_empty_device_driver_leaves_the_override_like_the_cpu` · `the_wire_s_driver_never_takes_the_cpu_round_trip` |
| 3 | ✅ **as chaves FICAM, com catraca** (decisão do §14.1 (3)) | 3 840 ficheiros varridos, **0** leituras partilhadas; mutação a sangrar | `nenhuma_das_tres_chaves_e_lida_por_nome_fora_do_dono` (+ o controlo do precedente `substeps` e o do detector) |
| 4 | ⛔ **o comprimento NÃO muda — recusado por duas medições.** (a) O CENSO dos 72 consumidores de valor (derivado do registo): **5 dão outra resposta a `1` que a `N` cópias, e não por defeito — o comprimento É o significado** (`pulse.compare`/`pulse.on_change` disparam um evento POR PEÇA; `value.reduce` soma/conta linhas, na CPU e na placa). Logo a porta `in` do cursor/tabela diz *«um por peça»* e não é morta. (b) A cura que sobrava era o TRANSPORTE (1 número a atravessar, expandido na placa) — e a costura de `32 761` cópias custa **0,002 ms** por quadro (0,073 × 0,071) | — | `every_value_port_reads_a_length_one_field_as_its_n_copies` (a lista dos 5 é exacta: um novo reprova, uma cura também) |
| 4′ | ✅ o censo achou DOIS defeitos de verdade, curados: o `value.wrap` tomava o comprimento só da entrada e perdia a faixa por peça com uma entrada de `1` (a regra 1→N da casa); o `motion.distribute_radial` lia o `spin` com `Read` sob uma lei de contagem (comprimento `1` julgado AUSENTE na placa) → `ReadBroadcast` | — | `the_range_ports_make_the_fold_per_instance` · o censo |
| 5 | ✅ **o relógio, pela porta do produto** (`sonda_o_relogio_do_ciclo_6`: variantes no mesmo processo, blocos rodados, o mínimo; o controlo imprime a rota e quem coze cada condutor). A sequência do quadro saiu do `dispatch` para `coze_o_quadro` — a ponte e a sonda chamam a MESMA | o pano do fio no tecto (`32 761`): **0,051 ms** na placa · **0,053** sem o fio (dirigir um param custa ~0) · **0,134** com o fio a derrubar para a CPU (**2,6×**, o caminho de antes da W1a) | — (sonda) |
| 6 | ✅ **a barra estava a medir o `value.noise`, não o `value.slope`**: em espaço de ÍNDICE a coordenada da rede cresce com `i` (~128–512 nas últimas linhas) e o ruído diverge até `6,9e-5` lá; o slope só o multiplica (`2,3 × max|dnoise| = 1,0501e-4`). O fixture passou a espaço de MUNDO e a barra `1e-4` ficou no VALE: piso **6,2e-6**, menor mutação do kernel **5,0e-4** (16× acima do ruído, 5× abaixo do defeito). E o comparador engolia NaN (`f32::max` devolve o acumulado): um defeito só-NaN passava | os vizinhos com a mesma barra vivem 6,5× a 200× abaixo dela | `value_slope_kernel_matches_the_cpu_on_the_device` · `o_desvio_nao_engole_um_nan` · `probe_value_slope_parity_in_ulps` |
| 7 | ✅ **o nascimento por pulso na placa** (`FiredBirth`): a porta lateral ganha a linha original (`iota`), é COMPACTADA pelo predicado (a mesma varredura do `Compact`), e um SEGUNDO kernel coze os nascidos do pulso na janela da lei DELE — que vê só a LARGURA `F` das linhas que dispararam; concatenado depois da taxa. Com o pulso ligado a taxa usa a variante de período pela metade (a escolha é estrutural, decidida pela forma do plano) | metrónomo: **531** nascidos do pulso ao bit; rajada+sorteio+empurrão: **999**, `vel` a `6e-8`; **fogos** (`died → template`, `pulse → pulse`): **12 891**, `vel` a `1,5e-6` | `the_chain_that_opened_the_wave_is_now_claimed_by_the_device` · `the_pulse_births_the_same_ids_at_the_same_rows` · `the_pulse_burst_draws_and_kicks_the_same_survivors` · `the_fireworks_are_born_on_the_device_as_on_the_cpu` · `a_pulse_longer_than_the_template_fires_only_its_rows` · `the_two_rate_spans_are_the_eval_s_spans` |

**Cinco achados que o caminho trouxe, e o que se fez com eles:**

- ⛔⛔⛔ **O smoke foi mandado SEM FORMAS, pela terceira vez** (report do dono, 07/10: *«vc não
  colocou shapes nos grafos. vejo apenas gizmos. eles não piscam»*). A `=117` e a `=27` eram
  correntes só de posições: a saída desenha só as cruzes, e o clarão (que incha e colore) agia sobre
  peças que não estão no ecrã — o gate novo mediu **zero** acesas nas DUAS rotas antes da cura.
  ⇒ a `=117` carimba uma estrela (`source.shape` + `motion.duplicator`) ANTES de o número e o
  instante agirem (⛔ a `=27` NÃO: carimbar a saída de uma zona de simulação não aparece no ecrã —
  [Bug #14](BUGS_motion_nodes.md), aberto, com a bissecção; ela ficou como estava); gate pela porta do produto com placa:
  `the_bottom_cloths_flash_on_the_device_like_on_the_cpu` (as estrelas de baixo acendem nas mesmas
  batidas nas duas rotas — a 1 quadro: a batida de `0,6 s` cai exactamente num quadro, o empate
  declarado do `pulse.beat`). Os gates que usavam a `=117` como «cena só de posições» passaram à `=6`.

- ⛔⛔⛔ **Um documento INTEIRO na placa com um `pulse.signal` gritava errado** (o gate batched o
  apanhou: a `=26` gritava a cada tique em vez de a cada volta do contador). O `pulse.signal`
  tem kernel desde a W2 deste ciclo, logo «um grafo com sinal nunca é 100 % placa» deixou de ser
  verdade em 14/09 sem ninguém o dizer; a `=26` escapava POR ACASO (o `carry` derrubava o contador
  — a cura do (1) tirou-lhe o acaso). Na rota inteira-na-placa a ponte só «cozia as tomadas sem
  marchar»: o `pre` do cone delas nunca avançava. ⇒ duas metades: a bomba avança o `pre` do cone
  das tomadas também quando não há fronteira (`has_work() || taps`, nos laços de avanço e de
  scrub), e a ponte, com tomadas de SINAL, MARCHA o cone delas em cada tique devido (a marcha da
  rota híbrida, com zero fronteiras). Gate pela ponte real com placa:
  `a_ponte_na_placa_grita_o_que_a_cpu_grita` (as duas metades sangram sem a cura).

- ⛔⛔ **A placa HERDAVA o `age` do modelo ao nascer, e a CPU não** (o `newborns` descarta `id` e
  `age`). Latente desde a ADR-0136, porque nenhum modelo de spawn trazia `age`; os FOGOS trazem
  (os mortos são o modelo) e os filhos nasceriam velhos na placa. Curado no contrato
  (`SourceRows::not_inherited`), e é o CONJUNTO de colunas do gate dos fogos que o apanha.
- ⚠️ **O fio da navalha do `pulse.beat` é provocado pelos parâmetros decimais da fixtura**: em
  aritmética racional EXACTA, `0,0537` e `0,0031` sobre a grelha de 1/60 s dão empates
  batida-tique nos tiques 36 e 72 (linhas 3 e 6) — e aí `f32` e `f64` arredondam para lados
  opostos (a divergência que o próprio nó declara). A paridade do spawn salta esses tiques
  (contados: 4 em 91) porque o spawn não tem estado; ⚠️ o gate de paridade do `pulse.beat` só
  corre 6 tiques e nunca chega a eles.
- ⚠️ **Preparar os passes de `value.attribute` + `value.reduce` na placa custa ~0,09 ms de CPU
  por quadro** (0,125 contra 0,033 do condutor constante) — é ele, e não a placa, que come o
  ganho do (2). Medido e NOMEADO aqui; não é desta lista.

**E as frases que deixaram de ser verdade, corrigidas na origem:** a legenda da `=116` e o anúncio
dela diziam `102 400 peças` (a cena produz `32 761` desde o tecto do dono) e prometiam que, com o
interruptor desligado, *«o pano fica pesado, a respiração engasga»* — medido, `0,13` contra `0,05`
ms, nada que se veja a 60 fps. Os dois lêem agora a contagem do tecto, e o tutorial 06 (§7) foi
refeito com o custo medido.
