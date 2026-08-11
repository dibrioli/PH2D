# 12 — PULSE / EVENTOS (6 nós) — conferência contra a referência

**Data:** 2026-08-09 · **Plano:** [89](../89_plano_conferencia_dos_nos.md) §3 · **Família 12/17**
**Nós:** `pulse.beat` · `pulse.compare` · `pulse.counter` · `pulse.on_change` · `pulse.sample_hold` · `pulse.threshold`

**Método:** params lidos do `MANIFEST` de cada crate (não do doc), faixas/unidades/widgets lidos do
`register_param_ui`/`register_param_hard_max` de cada `register()`, e **toda cadeia de
expressibilidade foi TENTADA contra o catálogo real de 118 nós** — três delas caíram por fatos do
substrato que eu só descobri lendo o código (`driven_value` lê a coluna `v` · `value.slope` é
derivada sobre o ÍNDICE, não sobre o tempo · `validate` nunca exige input conectado).

**Referências:** as do repo — [doc 06](../06_pulse_gatilho_primeira_classe.md) (Rive · Cavalry ·
Max/Pd · Schmitt · TD Trigger CHOP), [doc 08](../08_pulse_counter_reducer_bridge.md) (TD **Count
CHOP** · Max **counter** · Houdini Count · Cavalry Timeline Counter, com URLs primárias),
[doc 09](../09_handoff_pulse_signal_source_and_naming.md) §3 (a matriz MiniCavalry × Max × TD ×
Rive), [doc 14](../14_sample_hold_instance_field_nota_adr.md) §2 (Buchla · Max `sah~` · TD Hold
CHOP), [doc 17](../17_switch_on_change_nota_adr.md) §2 (Max `change`),
[`referencia_pesquisa_niagara_stardust.md`](../referencia_pesquisa_niagara_stardust.md) linhas
21/33/34/108/157 (Event Handler · Trigger Event · GPU events),
[`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) linhas 92/93/100,
[`referencia_pesquisa_blender_gn.md`](../referencia_pesquisa_blender_gn.md) (o CONTRASTE: Geometry
Nodes **não tem tipo de evento nenhum** — o mais próximo é a Simulation Zone, que é estado entre
frames, e o "Modal Event" aparece na lista de *faltantes* dos próprios devs, linha 237).

---

## A tabela

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| **(família)** | — | **um pulso não tem NÍVEL** — o par pulso↔0/1 é unânime: MiniCavalry `threshold` emite *"0/1 **+** pulse"* (doc 09 §3), Cavalry Comparison *"output a value of 1 when true and 0 when false"* (doc 06 §1), TD **Logic CHOP** converte nível↔borda, Max `gate` | ✅ **CONSTRUÍDO (2026-08-09): `pulse.level`** — momentâneo, sem estado, **zero params**. Gates de cadeia no registry REAL: `ph2d-node-registry-init/tests/pulse_level_chains.rs`. ⚠️ **Uma frase desta linha estava ERRADA e foi medida:** *"`pulse.counter` acumula (monotônico, nunca volta a 0)"* vale para o `count_tick` no `pre`, **não** para o que ele emite — com `count_max = 2` o valor exibido é `tick mod 2` = **toggle** (Wrap) e `min(tick,1)` = **latch** (Clamp). Por isso o nó novo não ganhou `mode`: os dois já existem, e o gate `the_toggle_and_the_latch_are_the_counter` mede a tabela. E o **nível de um SINAL** também já existia (`value.step(Hard)`) ⇒ o *"0/1 + pulse"* da referência é o par `value.step` + `pulse.compare` sobre o mesmo valor | **omissão** | ✅ | `pulse.level`: nó novo ⇒ nada existente muda |
| **(família)** | — | **nada é DISPARADO por um pulso** além de 4 consumidores — Niagara **Event Handler** (`Spawned`/`Every Particle`, ref linha 21) e **Trigger Event On Die/Always** → GPU event alimentando o Initialize do sistema filho (linha 33/34); doc 63 §2.3 marca `sim.replicate` como **P0** e a ref linha 108 como *"o buraco grande"* | ✅ **CONSTRUÍDO (2026-08-10): a porta `pulse` do `sim.spawn`** — uma linha do template que FIRA dá à luz `burst` elementos **naquela linha**, naquele tique: o gesto inteiro do Event Handler com uma porta e um número. ⚠️ **O muro era a IDENTIDADE, e ele NÃO se dissolveu — ele mudou de RELÓGIO:** o id de um recém-nascido é o ordinal de nascimento, e *quantos pulsos já dispararam* é HISTÓRIA (um contador é exatamente o que o `sim.spawn` recusa, senão um scrub RENUMERA o mundo). O pulse-born é numerado pelo **ordinal do TIQUE** (`PULSE_ID_BASE + tick·PULSE_IDS_PER_TICK + j`) — mesmo relógio, mesma garantia — e as duas espécies vivem em **metades DISJUNTAS** do espaço de id, porque um rate-born e um pulse-born com o mesmo id seriam pareados como UM elemento por todo nó de estado a jusante (`motion.integrate`/`spring`/`delay` chaveiam por `id`). ⚠️ A metade é carvada **só com um pulso fiado**, então sem ele os ids mantêm o período `ID_WRAP` inteiro — é isso que faz de *"desconectada = o mundo de antes"* verdade também dos **IDS**, e não só da contagem. ⚠️ **O caminho é CPU, e não custa nada que alguém tivesse:** nenhum dos seis `pulse.*` tem kernel, logo a cadeia que alimenta a porta já é fronteira de dispositivo — a recusa é DECLARADA onde o plano a vê (`ColumnAccess::RefuseIfPresent`, ADR-0127 D3), porque um kernel que ignorasse a porta responderia o grafo do artista com **todo nascimento-por-pulso AUSENTE**, em silêncio | **omissão** | ✅ | porta `pulse` opcional ⇒ desconectada = `Empty` = hoje (byte a byte) |
| `pulse.beat` | 2 — `period` 1.0 s (0.05‥8, **sem hard max**) · `offset` 0.0 s (−4‥4) · `Temporal` | **BPM / Time Mode** — TD **Beat CHOP** é tempo-nativo em BPM (doc 09 §3, a linha "fonte de batida"); o doc 63 §3.2 já exige *"Time Mode: Seconds/BPM"* do irmão `motion.oscillator` | **NÃO** (é aritmética mental do artista: `60/bpm`; nenhuma composição converte a UNIDADE de um param) | omissão | **P2** | `mode = Seconds` ⇒ o `period` de hoje |
| `pulse.beat` | idem | **swing / fase POR LINHA** — o próprio doc-header confessa (*"Per-row swing/stagger is a follow-up"*); `value.lfo` e `motion.oscillator` **já têm `phase_stagger`** ⇒ assimetria interna | **SIM** — `value.lfo(wave=Saw, phase_stagger=s) → pulse.compare(rise=0.5)` dá pulso escalonado por linha em **2 nós** (o `value.lfo` tem `Saw` e `phase_stagger`; verificado no `PARAM_HINTS` dele) | omissão | **P2** | `phase_stagger = 0` ⇒ `vec![pulse; n]` uniforme de hoje |
| `pulse.beat` | idem | **janela de atividade** (começa em T, N batidas, para) — Cavalry `Sequence`/`Scheduling Group` (ref linha 95, marcada *"sem sequenciador"*) | **NÃO** (precisa do GATE da linha 1: `beat AND (t ∈ janela)`) | omissão | **P2** | `count = ∞` ⇒ metrônomo eterno de hoje |
| `pulse.beat` | idem | ⚠️ **o teto do `period` é 8 s e NÃO há `ParamHardMax`** — o bridge cai no `hint.max` (`motion_bridge_params.rs:467`, `.unwrap_or(max)`) ⇒ **uma batida mais lenta que 8 s é indigitável**; nenhuma referência tem teto de período. Limite sem medição (§0) | n/a (é o teto, não uma capacidade) | omissão | **P2** | `ParamHardMax` acima do slider ⇒ o curso da mão não muda |
| `pulse.threshold` | 4 — `channel` (X/Y/Rot/Size) · `rise` 0.5 (−10‥5, hard 10) · `fall` 0.3 (−10‥3, hard 10) · `edge` (Rise/Fall/Both) | **retrigger delay / debounce** — TD Trigger+Count CHOP têm `retrigger`; Pd `threshold~` tem *"debounce ms por borda"* (doc 06 §3, tabela). **Deferido de propósito** ali (*"a histerese já mata o chatter"*) — mas histerese mata *chatter de ruído*, não *repique de gesto* | **NÃO.** Precisaria segurar o pulso por N ticks: `motion.delay` (mode Delay·ticks) é `INST_VEC2`, não alcança `PULSE`, e não existe delay no domínio de valor | ✅ **CONSTRUÍDO (2026-08-10): o param `debounce`, em SEGUNDOS** — depois de um pulso, a janela seguinte é silenciosa. ⚠️ **A cerca estava meia-certa e a metade que faltava é a lei:** histerese é guarda de **AMPLITUDE** (engole ruído que balança sobre UM nível) e debounce é guarda de **TEMPO** (engole dois cruzamentos *legítimos* rápidos demais) — nenhuma substitui a outra, e o gate `..._swallows_a_second_crossing_...` traz o CONTROLE ao lado: o mesmo sinal sem janela dispara as duas vezes. ⚠️ **O debounce silencia a SAÍDA, nunca a máquina de estados** — e o gate que afirma isso **nasceu VERDE sob a mutação que existe para matar**: a fixture segurava o sinal LOW durante a janela, e fora da banda o Schmitt re-sincroniza em UM tique, então o latch congelado era corrigido no instante em que o congelamento acabava. **Só dentro da banda a perda é permanente** (`armed_now == prev_armed` ali), e é lá que a fixture passou a estacionar o sinal. ⚠️ **O teto digitável é MEDIDO e o recurso é *precisão de representação*:** a contagem vive em `f32` e é decrementada por `dt`, então acima de certa magnitude `cool - dt == cool` e o nó ficaria mudo **para sempre, em silêncio** — medido, o penhasco fica entre 2^19 e 2^20 a 60 fps e desce uma oitava a cada dobra da taxa; o teto é **131072 s**, o último que ainda drena a 240 fps, com o gate afirmando as DUAS metades (drena · o dobro não) | omissão | ✅ | `debounce = 0` ⇒ dispara todo cruzamento, como hoje — a contagem é identicamente zero e nunca gateia |
| `pulse.threshold` | idem | ⚠️ **REDUNDÂNCIA, não gap** — desde que `value.attribute` virou picker de canais (2026-07-30), `value.attribute(Y) → pulse.compare(rise,fall,edge)` **é** o `pulse.threshold(channel=Y)`: mesmo Schmitt, mesmos 3 params, um adaptador na frente | **SIM, e é o problema** — dois nós respondendo uma pergunta. O doc 16 §1 já distinguia os dois pelo *domínio* (transform × valor), e o picker apagou a distinção | — | ⛔ | (consolidação, não param novo) |
| `pulse.compare` | 3 — `rise` 0.5 (−10‥5, hard 10) · `fall` 0.3 (−10‥3, hard 10) · `edge` (Rise/Fall/Both) | **lógica booleana entre pulsos** (AND/OR/NOT/XOR) — Cavalry **Logic** (ref linha 92) e TD **Logic CHOP** | ✅ **FECHADO com o `pulse.level` (2026-08-09), sem nó novo.** Com nível 0/1, `value.math(Min)` **é** AND e `value.math(Max)` **é** OR (os dois já existiam no enum), e `pulse.compare(rise=0.5)` traz de volta ao domínio de pulso — medido no gate `two_pulses_are_anded_by_the_value_math_that_already_exists` (0,5 s ∧ 0,25 s = quadros 0 e 30; ∨ = 0, 15, 30, 45), com o OR servindo de oráculo do AND | omissão | ✅ (colapsou na P0) | — (foi a P0 que resolveu) |
| `pulse.compare` | idem | **comparar contra outro CAMPO**, não só contra uma constante — Cavalry Comparison compara dois valores; TD Logic CHOP compara canais | **SIM, duas rotas** — `value.math(Subtract, a=sinal, b=referência) → pulse.compare(rise=0)` (1 nó), **ou** dirigir o `rise` por fio (doc 58: `driven_value` lê `v`, e um `value.*` emite `v` ⇒ funciona hoje, sem nó nenhum) | — | **P2** | (já exprimível) |
| `pulse.counter` | 2 — `count_max` 6 (1‥32, **sem hard max**) · `mode` (Wrap/Clamp/Zigzag) | **entrada de RESET** — TD **Count CHOP** tem `Reset` + `Reset Value`; Max `counter` reseta (doc 08 §1, com URLs). **Deferido no doc 08 §3.1** — ver CERCAS: a premissa do deferimento é FALSA hoje | **PARCIAL, num regime estreito** — `value.math(Subtract, a=count, b=pulse.sample_hold(value=count, pulse=reset))` dá *"contagem desde o último reset"*, **mas só com `mode=Clamp` e `count_max` alto**: sob Wrap a subtração de dois valores já modulados é aritmética errada, e `count_max` **não passa de 32 digitando** | omissão | **P1** | porta `reset` opcional ⇒ desconectada = `Empty` = hoje |
| `pulse.counter` | idem | **incremento ≠ 1** — TD Count CHOP conta `±1` **ou `±tempo`** e tem entrada `Increment`; Cavalry **Accumulator** *"acumula valor ao longo do tempo"* (ref linha 93, marcada PARCIAL). ⚠️ E o `motion.step`, o irmão de onde este nó SAIU, **tem `step`** — o redutor perdeu a capacidade do ancestral | **PARCIAL** — escalar por uma constante é `value.gain`/`value.math(Multiply)` a jusante (**SIM**, 1 nó). Acumular um valor **VARIÁVEL** por pulso (o Accumulator) é **NÃO**: exigiria somar `sample_hold(v)` ao acumulado, e a realimentação de valor (`pre`) é porta interna, nunca autorável | ✅ **CONSTRUÍDO (2026-08-10): o param `step`** — inteiro, default **1** ⇒ o mundo de hoje byte a byte, e **pode ser NEGATIVO**, que é a capacidade de verdade (contar PARA TRÁS; o `value.gain` a jusante escala a contagem, não a direção dela). ⚠️ **O `Clamp` ganhou PISO no mesmo commit, e ele é byte-idêntico hoje:** antes do `step` o tique só sabia crescer a partir de zero, então `clamp(0, n-1)` e `min(n-1)` concordam em TODO tique alcançável — o piso existe porque um incremento negativo torna o tique negativo alcançável, e um Clamp sem piso mostraria uma contagem NEGATIVA. A representação apaga o caso especial em vez de o gatear. ⚠️ A metade do **Accumulator** (somar um valor VARIÁVEL por pulso) segue **NÃO** e pela razão que a coluna já dava | omissão | ✅ | `step = 1` ⇒ a escada de hoje |
| `pulse.counter` | idem | **CARRY-OUT** — Max `counter` *"emite um bang no limite"*, e é *"exatamente como se encadeiam contadores"* (doc 08 §1). **Deferido no doc 08 §3.1** (*"exige 2ª porta de saída Event"*) | **NÃO** — tentei `counter(Wrap) → pulse.on_change`: dispara em **toda** mudança de contagem, não só no wrap. Isolar o wrap exige comparar com o tick **ANTERIOR**, e ⚠️ **`value.slope` NÃO é a derivada temporal** — o doc-header dele é explícito: *"the discrete `d(value)/d(index)` … across the instance order"* | ✅ **CONSTRUÍDO (2026-08-10): a 2ª saída `carry`** — e ela é o **PRIMEIRO nó do repo com duas saídas**. O motor sempre as suportou (`Cook::cur_output` indexa a porta de origem) e o painel sempre desenhou `outputs.len()` sockets; o que não existia era um nó a exercitá-las, e o gate do divisor de relógio (`beat → counter(4) → carry → counter(4)`, cozido no registry REAL) é o primeiro grafo do repo a LER uma porta de saída 1. ⚠️ **A lei é o CICLO, e a primeira que escrevi estava ERRADA:** *"o carry dispara se a contagem exibida não andou os `step`"* lê bem, acerta no Wrap, e no **Zigzag dispara em TODA batida da perna descendente** — descer é a trajetória legítima daquele modo, não uma intervenção. A lei que shipou é `div_euclid(tique, período)` mudou, com o período do MODO: **Wrap** `n` · **Zigzag** `2(n−1)` (a volta INTEIRA, não a dobra do meio) · **Clamp** nenhum, porque uma contagem que PARA nunca completa uma volta — e recusar o sinal é o nó não mentindo | omissão | ✅ | 2ª saída `carry` ⇒ desconectada = hoje |
| `pulse.counter` | idem | **contar para BAIXO** — Max `counter` tem up/down/updown (doc 08 §1); `Zigzag` é o updown, "down" não existe | **SIM** — `value.math(Subtract, a=debug.const(N−1), b=count)` inverte a escada (1 nó) | — | **P2** | `direction = Up` ⇒ hoje |
| `pulse.counter` | idem | ⚠️ **`count_max` para em 32 digitando** (hint 1‥32, sem `ParamHardMax` ⇒ fallback `hint.max`); TD/Max contam sem teto. Limite sem medição (§0), e é o mesmo número que trava a cadeia de reset acima | n/a | omissão | **P2** | `ParamHardMax` acima do slider ⇒ curso da mão intacto |
| `pulse.on_change` | 1 — `epsilon` 0.001 (0‥0.01, **hard 1.0**, documentado) | **DIREÇÃO da mudança** (subiu / desceu / qualquer) — Max `edge~` tem **dois outlets** (doc 06 §3); TD Logic tem *Rising Edge* e *Falling Edge* como modos distintos. ⚠️ Assimetria interna: `pulse.compare` e `pulse.threshold` **têm** o param `edge` (Rise/Fall/Both) e este não | **NÃO** — `pulse.compare` dispara num cruzamento de NÍVEL, não na direção de um degrau qualquer; e `value.slope` é sobre o índice (acima) | ✅ **CONSTRUÍDO (2026-08-10): o param `direction`** — e a escolha que importa é que ele fala **a língua da família**: mesmos rótulos (`Rise`/`Fall`/`Both`), mesma numeração (`0·1·2`) e mesma escada do `edge` dos dois irmãos, porque a assimetria que esta linha aponta é sobre o que o ARTISTA lê. Só o **default** difere, e tem de diferir: o neutro aqui é `Both`. Gate no registry REAL (`pulse_edge_vocabulary.rs`), que é a única crate que enxerga os três nós ao mesmo tempo — cada `pulse-*` é folha drop-in e re-declara o próprio enum de propósito. ⚠️ **E o gate do fallback achou um defeito VIVO:** `f32::NAN as i32` **satura em 0** (o cast float→int do Rust), então um param ilegível selecionava a variante 0 — inofensivo no `pulse.threshold`, onde 0 **é** o default, e aqui **estreitava o nó para Rise-only, em silêncio**. A coincidência que cobria o irmão parou de cobrir na primeira vez que o neutro deixou de ser a variante zero | omissão | ✅ | `direction = Both` ⇒ `\|v − prev\| > eps` de hoje, bit a bit (o braço `Both` não acrescenta aritmética nenhuma) |
| `pulse.sample_hold` | **0** | **modo de borda** (TD Hold CHOP *"Off to On"*; doc 14 §2) | — | **NATUREZA, com mecanismo:** o nosso pulso carrega **só "disparou"** (doc 06 §2, decisão contra o `{value,edge,t}` do MiniCavalry) ⇒ um pulso de 1 tick **não tem** borda de descida nem "enquanto alto"; o modo de borda só existe sobre um GATE, e gate é a P0 da linha 1 (se ela nascer, este item nasce com ela) | ⛔ | — |
| `pulse.sample_hold` | **0** | **slew / portamento** entre amostras (o par clássico S&H + lag do modular) | — | **NATUREZA:** a referência também os separa (TD tem Hold CHOP **e** Lag CHOP como nós distintos), e o `motion.lag` já está catalogado como **P0 no doc 63 §2.3** — pertence à família VALUE, não a este nó | ⛔ | — |
| `pulse.sample_hold` | **0** | **magro por NATUREZA — o veredito, com mecanismo:** as duas entradas do sampler são PORTAS (o valor e o gatilho, ambos animáveis), e a única escolha que sobraria — *qual borda* — é inexprimível no tipo (linha acima). Zero params está **certo** | — | natureza | ⛔ | — |
| **(família)** | — | **`pulse.adsr`** — envelope Delay/Attack/Sustain/Release no trigger (TD **Trigger CHOP**: *"starts an audio-style ADSR envelope to all trigger pulses"*, doc 06 §4); o doc 63 §2.4 já o lista como **P1** e continua correto (não há crate `pulse-adsr`) | **PELO HACK, e só** — `motion.strobe` É o envelope, mas escreve **canal de transform**; para usá-lo como VALOR seria `strobe → value.attribute(canal)` = exatamente o "clock hack" que o doc 09 matou | ✅ **CONSTRUÍDO (2026-08-10): a crate `ph2d-node-pulse-adsr`** — e a decisão que desenha o nó é que **um gatilho não tem *note off***. Num sintetizador o envelope é dirigido por um PORTÃO (a tecla desce, a tecla sobe); o nosso pulso é um IMPULSO, e `pulse.level` é **momentâneo e sem estado por decisão** (a P0 desta folha) ⇒ **não existe no catálogo nada que segure um portão aberto**. Logo o envelope é um **one-shot** e o param `hold` é quem o fecha — a única forma que COMPÕE com o que a família produz. Um segundo porto *release* daria o modelo de teclado e seria a **segunda** resposta a *"quando este envelope termina?"*: fica nomeado, não construído. ⚠️ **As rampas são ALGÉBRICAS** (bias de Schlick, HR-5) e em `0.5` reduzem **literalmente** à identidade — linear não é aproximação de linear. **Dois** shapes e não três (o `release_shape` governa as duas QUEDAS), com gate afirmando que cada um dobra exatamente os trechos que promete — sem ele a decisão seria uma frase, porque nos defaults lineares os dois são indistinguíveis. ⚠️ **E o teto NÃO é o do `debounce`, embora a grandeza e o relógio sejam os mesmos:** aquele conta para BAIXO e este para CIMA, e numa potência de dois os vizinhos de baixo estão **duas vezes mais juntos** — copiar o número teria shipado um teto quebrado; medido, **65536 s** | omissão | ✅ | nó novo ⇒ nada existente muda |

---

## `SUPERAR:`

As três derivam do mesmo par que **nenhuma referência tem junto**: *o pulso é um CAMPO* e *o cook é
função pura do playhead com scrub bit-exato*.

1. **O contador que REBOBINA — e ele já funciona, conferido no código.** O estado de toda a família
   viaja no `pre` (= o output do tick anterior), `Cook::checkpoint` salva `prev_outputs`
   (`cook.rs:429`) e `restore` o reinstala limpando o memo (`cook.rs:445`), com o gate
   `checkpoint_then_restore_reproduces_a_past_frame_and_resim_is_bit_exact` (`cook_tests.rs:593`) e o
   irmão que mostra que **o cook rebobinado ingênuo casa com nenhum frame passado** (`:655`) — a
   prova de que o checkpoint é load-bearing. ⇒ **arrastar a régua para trás devolve a contagem
   EXATA.** Niagara acumula estado por-partícula e não rebobina; Max/TD são streams ao vivo **sem
   cursor de tempo** (não há o que rebobinar). O degrau inédito que isso destrava é **`pulse.at` — o
   pulso AUTORADO na régua** (uma lista de tempos, ou os markers da timeline): nas referências um
   padrão de disparos precisa ser *executado* e gravado; aqui ele é *função do playhead*, logo
   reproduzível e editável como keyframes.
2. **O pulso do CONJUNTO, derivado do pulso das partes.** `pulse.on_change`/`compare`/`threshold` já
   disparam **por linha** — o gate `it_watches_each_instance_of_the_field_independently` prova
   (`[0,1]` para duas linhas, só a que mudou). Max/Pd/TD são canais escalares (um bang é um bang) e
   Niagara é por-partícula **sem editor de sinal**. ⇒ **`pulse.reduce`** (*quantas linhas dispararam
   neste tick* → valor) e **`pulse.any`/`pulse.all`** (o pulso do conjunto) são operações que
   **nenhuma referência pode ter**, porque nenhuma tem pulso-como-campo. É a ponte que faz *"quando
   a última partícula morrer, dispare"* virar uma aresta em vez de um script.
3. **O portão ESPACIAL.** A família `field.*` (5 nós componíveis, C4D Fields como referência de
   origem) produz um peso por linha; o pulso é por linha. ⇒ **`pulse.gate` cujo portão é um CAMPO**
   — *"dispare só quem está dentro da caixa"*, *"o beat varre a cena como um radar"* (o
   `field.radial_sweep` já é o radar). C4D tem Fields e **zero eventos**; Niagara tem eventos e zero
   campos componíveis; nós temos os dois e eles nunca se encontraram. ⚠️ E este item **é a mesma P0
   da linha 1 vista de outro ângulo** — construir o nível/gate uma vez paga os dois.

   ✅ **METADE ENTREGUE (2026-08-09):** o portão existe como CADEIA —
   `pulse → pulse.level → value.math(Multiply, condição) → pulse.compare(rise = 0.5)` — e o gate
   `a_pulse_is_gated_by_a_condition_the_value_domain_names` o mede por linha, com um pulso
   **escalonado** (`value.lfo(Saw, phase_stagger) → pulse.compare`, que de passagem confere a
   resposta ao P2 de *swing por linha*).

   ✅ **E FECHADO no mesmo dia — o campo chegou:** a família `field.*` escreve a coluna
   **`falloff`** no stream de instâncias (as cinco, mais o `motion.falloff`), ela era consumida
   por SEIS `motion.*` e **ilegível** no domínio de valor, porque o `READ_CHANNELS` do
   `value.attribute` não a listava. ⚠️ **Uma linha de tabela, não um nó** — e é a diferença que
   decidiu a wave: o canal **Falloff** entrou no picker, e
   `field.box → value.attribute(Falloff) → value.math(Multiply, pulse.level) → pulse.compare`
   **é** *"dispare só quem está dentro da caixa"*, medido em
   `a_pulse_fires_only_where_a_spatial_field_says_it_may` (caixa de borda dura cobrindo metade da
   fileira ⇒ peso `[0, 0, 1, 1]`, e só o par de dentro dispara — em quadros diferentes).
   ⚠️ **E o doc do `READ_CHANNELS` afirmava um TETO que não existia** (*"sete + Custom = 8 = o
   teto do seletor segmentado"*): o teto é `MAX_ENUM_OPTIONS = 48`, derivado, e a fileira quebra
   em quatro colunas crescendo a própria altura. O 8 era um palpite sobre LARGURA vestindo a
   palavra *teto* — corrigido, com o gate `the_channel_picker_fits_the_panels_ceiling` no shell,
   onde a tabela e o teto se encontram.

   **SMOKE: `env PH2D_GPU_COOK_DEMO=23 cargo run -p ph2d-host-desktop --release`** — 262.144
   pontos, um metrônomo de meio segundo, e **só os pontos dentro de um losango piscam**. O resto
   da grade nunca se mexe: o beat chega a todas as linhas e o campo decide quem o escuta.
   ⚠️ **O campo é um RAMO LATERAL de propósito** — o `motion.drive` lê `falloff` como máscara
   de força própria, então pô-lo no caminho de instâncias faria a cena mostrar o quadro certo
   **pelo motivo errado**; MEDIDO: com o portão de pulso deletado *e* o campo no caminho, o gate
   do pisca-pisca fica **VERDE** e só o `the_gate_is_the_pulse_not_the_drives_own_mask` falha.
   A cena custa **3,96 ms/tique** (15,1 ns/ponto, `--release`) — a cadeia é CPU-only por
   natureza, e o censo de cobertura agora **NOMEIA** essa fronteira em vez de não a ver.

---

## `CERCAS:`

Grepadas antes de propor. Duas ainda valem, **duas envelheceram**, uma é lei.

- ✅ **O RESET LANDOU (2026-08-10), e a conferência tinha errado o VEREDITO, não o item.**
  A linha 41 o cataloga como *omissão* citando TD/Max — e **o ancestral já shipava a porta**:
  o `motion.step`, de onde este nó saiu (*"the count math is `motion.step`'s, verbatim"*),
  tem `reset` + `reset_to` com a MESMA lei (nível para o reset, borda para a contagem, o
  reset ganha o tique, desconectada = byte-idêntica). ⇒ era a **mesma classe da linha 42**
  (*"o redutor perdeu a capacidade do ancestral"*), e não uma capacidade que faltasse ao
  catálogo. Construído em `pulse.counter` **e** em `pulse.sample_hold` (o outro estado que
  não se auto-cura — medido: `compare`/`threshold`/`on_change` reescrevem o estado todo
  tique e `beat` deriva do tempo, então **só três nós** da família tinham o buraco).
  Gatilho: o report do Enio de 2026-08-10 (BUGS #1) — mudar um param de campo em runtime
  deixava contagem inalcançável.
- ⚠️ **A cerca do RESET tem premissa FALSA hoje.** O doc 08 §3.1 defere a entrada de reset assim:
  *"exige tolerar porta opcional desconectada (**validate rejeita input faltante**)"*. **Não rejeita:**
  `Graph::validate` (`graph.rs:596`) itera sobre as **arestas** e nunca exige que um input tenha uma;
  e `EvalCtx::input` está documentado como *"empty if unconnected"* (`cook.rs:128`). Dois nós que
  SHIPAM já dependem disso: `value.lfo` (*"`in` opcional lido só pela contagem"*, doc 14 §3) e
  `value.switch` (*"`select` desconectado → 0"*, doc 17 §3). ⇒ **a cerca pode ser removida**; o
  motivo dela deixou de existir, provavelmente quando o domínio de valor nasceu.
- ⚠️ **A cerca do CARRY-OUT tem premissa VIVA mas incompleta.** Doc 08 §3.1: *"exige 2ª porta de
  saída `Event`"* — verdade (o manifesto declara 1 output). O que ela **não** diz é que a alternativa
  por composição não existe (tentada acima), então o custo real é *uma porta* contra *nada*.
- **`retrigger`/debounce deferido** (doc 06 §3): *"a histerese já mata o chatter"*. **Valia
  parcialmente** — histerese mata chatter de **ruído**, não repique de **gesto** (dois cruzamentos
  legítimos rápidos). ✅ **FECHADO em 2026-08-10** (linha 37): o `debounce` é a guarda de TEMPO ao
  lado da guarda de AMPLITUDE que já existia, e a cerca sobrevive como o que ela sempre foi — a
  metade certa de uma frase.
- **O pulso carrega SÓ "disparou"** (doc 06 §2, decisão contra o `{value,edge,t}` do MiniCavalry,
  com o argumento: *"um consumidor que esqueça o `&& edge` conta o nível sustentado como N
  disparos"*). ⚠️ **Isto NÃO proíbe a P0 da linha 1** — um nó `pulse.level` produz um valor
  **separado**, no domínio de valor, e não põe um campo `value` dentro do pulso; a lei *"1.0 = borda,
  o consumidor só lê o tick"* fica intacta.
- **Zero RNG** (doc 06 §5): qualquer aleatoriedade futura da família é `hash(seed, id, …)`, nunca
  `Math.random` — a lição que o MiniCavalry pagou. Vale para um `probability` de beat/gate.
- **`pulse.counter` vs `motion.step`** (doc 08 cabeçalho + doc 09 §4.2): o `motion.step` é o
  *behaviour* (empurra canal), o `pulse.counter` é o *redutor puro*. Não são duplicata — mas ver a
  linha do `increment`: o redutor **perdeu** um param que o behaviour tem.

---

## `O DOC 63 ERROU EM:`

1. **A §3.2 (*"gap por nó existente"*) não tem UMA LINHA para os seis `pulse.*`.** Ela cobre
   `force.*`, `motion.*`, `value.map_range`, `sim.spawn` — e pula a família inteira. Não é
   envelhecimento: é ausência desde 2026-07.
2. ⚠️ **A afirmação *"o `pulse.*` já dirige o `rate`"* é FALSA** — e ela aparece **duas vezes**: no
   doc 63 §2.2 (`Spawn Burst` = *"PARCIAL (pulse.\* pode dirigir rate…)"*, ref linha 95) e no
   **gabarito do próprio plano 89** (§7 P2 e a linha de burst da §10). Verificado:
   `param_source::driven_value` (`param_source.rs:92`) lê `crate::attr::VALUE_COLUMN` = `"v"`, e um
   pulso emite `PULSE_COL` = `"pulse"` (`pulse-beat/src/lib.rs:126`) ⇒ **um pulso não dirige param
   nenhum**. O que dirige é o `pulse.counter` (output `VALUE`, emite `v`) — e um contador é
   **monotônico**, então dirigir `rate` com ele dá uma rampa crescente, não um burst. ⇒ a linha do
   burst do emitter **sobe de P2 para P1**, e a cura barata é a P0 da linha 1 (com um nível 0/1 e um
   `value.gain`, dirigir o `rate` dá um burst de um tick).

   ⛔ **A ÚLTIMA FRASE (a "cura barata") está REFUTADA, e a linha inteira está FECHADA por outro
   caminho** — as duas coisas medidas na wave do emitter (folha 01, 2026-08-09), **antes** desta
   folha existir. **(a)** Dirigir o `rate` **não produz burst nenhum**, com nível ou sem: o conjunto
   vivo do `motion.emitter` é função PURA do playhead, então um pulso de `rate` 40→1000 salta a
   janela de ids de `[40,79]` para `[1000,2000]` — **conjuntos disjuntos** —, e as partículas não
   persistem, não envelhecem e não sobrevivem ao gatilho; pior, a coerência de QUALQUER `rate`
   animado decai linearmente com o playhead (Δrate = 1: 100% sobrevivem em t = 1, **0%** em t = 60).
   **(b)** O burst foi construído **NATIVAMENTE** no nó (`emit_mode` · `burst_count` · `burst_time`
   · `burst_period`), com a lei de contagem por janela contígua. ⇒ o que sobrevive desta linha é só
   a metade de cima: *um pulso não dirige param nenhum* — verdade até 2026-08-09, e o `pulse.level`
   a fechou.
3. **A ref Cavalry linha 92 diz *"Comparison / Logic / If Else — **TEMOS** (pulse.compare,
   value.switch)"* — a metade `Logic` não tem dono.** Preciso: `Comparison` = `pulse.compare` ✅,
   `If Else` = `value.switch` ✅, e **`Logic` existe no domínio de VALOR** (`value.math` Min/Max
   **são** AND/OR sobre 0/1) **e não existe no domínio de PULSO** — pela mesma causa única da P0.
   ⇒ a linha devia ler **PARCIAL**.
4. **A ref Cavalry linha 93 (`Accumulator` = *"PARCIAL (pulse.counter)"*) está certa e nunca disse o
   que falta:** o nosso conta `+1` por pulso; o Accumulator acumula um **valor**. Nomeado agora.
5. **A §2.4 (`pulse.adsr` = P1, CHOP Trigger) estava CORRETA e foi FECHADA em 2026-08-10** — a
   crate existe (`ph2d-node-pulse-adsr`), e o `motion.strobe` segue sendo o envelope sobre
   **transform**, que é por que ele não servia. ⚠️ O que a conferência **não** tinha visto é a
   consequência do desenho: um gatilho não tem *note off*, então o envelope é um **one-shot** com
   `hold`, e não o ADSR dirigido por portão do sintetizador.

---

## A fronteira `pulse.*` ↔ `ph2d-runtime`

**Medido primeiro:** `grep` nos dois sentidos dá **zero** — nenhuma crate `pulse-*` (nem
`ph2d-nodegraph`, nem `ph2d-eval-motion`) menciona `ph2d-runtime`, e a `ph2d-runtime` tem **zero
dependências** por gate estrutural (`the_event_core_is_a_leaf`). Os dois "eventos" respondem
perguntas diferentes e é por isso que não colidiram: um `Signal` é **nomeado, por QUADRO, com bits
de entidade**, consumido pelo host (toast, som, Luau); um `pulse` é **anônimo, por LINHA, por TICK
do cook**, consumido por nós. **Não há decisão registrada em lugar nenhum** — não é fronteira
declarada, é um vão que nenhum dos dois lados menciona; o único registro parcial é o doc 63 §4, que
põe *"física↔grafo (collision events→pulse)"* como cross-line/decisão do Enio, e a ref Cavalry linha
100 (*Collision Events* = **FALTA**). **Veredito: gap real, com as duas direções assimétricas** —
`runtime → grafo` (colisão vira pulso) esbarra em que um sinal é fato do QUADRO e o cook é função do
TICK, então ele teria de viajar como **conteúdo** (uma coluna) e não como evento efêmero, sob pena
de um scrub perder o pulso e o grafo deixar de ser função do playhead; `grafo → runtime` (o beat
dispara um som) é trivial de escrever e **errado por construção hoje** — o cook re-roda no scrub e o
som sairia N vezes ao arrastar a régua —, e a regra que falta **já existe no bridge**
(`motion_bridge::ticks_owed`: play = todo tick para a frente, scrub = uma chamada), o que faz desta a
direção certa para abrir primeiro.

### ✅ CONSTRUÍDO (2026-08-10) — a direção `grafo → runtime`

Nó novo **`pulse.signal`** (crate-folha `ph2d-node-pulse-signal`): um pulso ganha um **NOME** e
atravessa intacto. A crate **não conhece a `ph2d-runtime`** — ela devolve o stream verbatim e conta
as linhas que dispararam; quem publica é o shell, que já é dono da outbox e já drena as outras duas
fontes (ADR-0075: o produtor não chama ninguém).

⚠️ **E a folha dizia *"trivial de escrever"*, o que a construção derrubou DUAS vezes.** Primeiro
porque o pump aceita **um alvo por marcha** (a doc dele: *one march, N hand-offs*), então cozinhar
a tomada numa segunda chamada avançaria o relógio **duas vezes por tique** e re-simularia o prefixo
compartilhado — em silêncio; as tomadas passaram a cozinhar **dentro da mesma marcha**, batendo no
memo do `Cook`, e lista vazia é o mundo anterior byte a byte.

⚠️ **E depois porque o produto tem DUAS marchas, não uma — o smoke do Enio pegou a cena MUDA.** A
primeira forma passava as tomadas como **argumento de `advance_or_scrub_scoped`** (a porta dos
*sinks*), e a cena `=26` planeja **HÍBRIDA** (medido: `boundaries = [5, 4]`, 4 estágios de
despacho) ⇒ o quadro marcha por `advance_or_scrub_to_nodes_scoped`, devolve `Handled` e **retorna
antes** do laço onde a leitura morava: o grafo cozinhava, a tela animava e o terminal ficava
silencioso, com **os seis gates verdes** (todos dirigiam a porta dos sinks). A causa não era a
leitura nem a lei — era **a tomada ser argumento de UMA porta**. Hoje ela é **estado da bomba**
(`Pump::set_taps`), cozida depois do alvo nos **dois** braços de `CookTarget`, e o que o host lê é
um **livro-razão** (`tap_fires`: uma linha por tique PEDIDO — nunca por passo de re-simulação,
senão um wrap de loop gritaria a volta inteira num quadro), lido **uma vez, onde toda rota
converge**. O gate que faltava é de ROTA (`as_duas_rotas_de_cook_gritam_a_mesma_coisa`, com o irmão
que MEDE a premissa de que a cena é híbrida): reinstalando o defeito exato, **só ele sangra**.

⚠️ **A lei do relógio NÃO foi derivada do `ticks_owed`, como esta folha previa** — ele não
distingue *tocar* de *saltar para a frente*, e um seek com o play ligado teria virado exatamente a
metralhadora que o parágrafo acima descreve. A resposta certa já existia no emissor de markers da
timeline (`!jumped && is_advancing_forward`), e ela mudou-se para uma casa própria
(`render_loop::clock_forward`) porque **não é de nenhum dos dois emissores**.

⚠️ **Dois preços NOMEADOS:** o colapso **linha → quadro** (576 pontos que disparam juntos são UM
evento com `rows = 576`, não 576 sons) e o produtor de Motion pousar **um quadro atrás** dos outros
dois — o dispatch dele roda depois de os consumidores lerem, e o duplo-buffer da outbox torna isso
*atrasado, nunca perdido*. Fechar o vão move a leitura dos consumidores para baixo do dispatch, o
que reordena uma sequência gateada: decisão própria.

**Cena:** `PH2D_GPU_COOK_DEMO=26` (com `PH2D_SIGNAL_LOG=1`) — a MESMA cena do compasso com uma
tomada em cada relógio, então o terminal conta a razão que o olho conta na tela. **Smoke aprovado
pelo Enio em 2026-08-10**, depois do fix de rota acima.

**Segue ABERTA a direção `runtime → grafo`** (colisão vira pulso), pelo motivo acima: ela é
decisão do Enio, não dívida de engenharia.

