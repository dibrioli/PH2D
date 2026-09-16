# TOP-20 #15 — `StateMachine`: o plano

> *«enquanto o script não tem UI, é o único cérebro autorável; estado-como-componente»* —
> [levantamento §7](00_levantamento_componentes.md).
>
> Fila: os itens **1–14** estão fechados (uns feitos, dois por **recusa medida**). Este é o
> seguinte, e o levantamento põe-no no bloco **13–15 — gameplay sem código**.

---

## §0 — O que o artista consegue FAZER quando isto fechar

Escrever o cérebro de um objecto **sem uma linha de script**: *«esta porta está `Fechada`; quando
ouvir `botao` vai para `A abrir`; quando ouvir `fim` vai para `Aberta`»* — e cada estado
**anuncia-se**, de modo que a tabela de acções do #5 faça o resto (mostrar, esconder, tocar som,
arrancar um relógio).

---

## §1 — ⭐⭐⭐ A PERGUNTA DE ANTES: a composição já o exprime? (§5.0)

Sonda: [`mede_o_que_a_composicao_ja_da_ao_cerebro`](../../crates/ph2d-ecs/tests/it/mede_o_que_a_composicao_ja_da_ao_cerebro.rs)
(`--ignored`). **Corrida, e a resposta é NÃO em três sítios independentes:**

| pergunta | medido |
|---|---|
| O MESMO sinal pode fazer coisas diferentes? | ⛔ **Não.** Duas linhas contraditórias (`Show` + `Hide` na mesma parede) disparam **as duas**. Nada escolhe uma — *e escolher uma é o que um ESTADO é* |
| Uma acção pode ser condicional? | ⛔ **Não.** A `SignalAction` tem **5** campos (`on · target · verb · arg · target_by`) e **zero** são uma guarda |
| Uma acção pode EMITIR um sinal? | ⛔ **Não.** Dos **7** verbos com sink, **nenhum** emite — e o doc do módulo **declara a ausência por escrito**, com o motivo (*«a classe inteira dos laços»*) e com a cura prescrita: *«quando existir, ela entra com um ORÇAMENTO DE PROFUNDIDADE, não com um `if`»* |

⭐⭐ **E a casa TEM uma máquina de estados — `ph2d-ui-state::Machine` — cujo próprio doc nomeia este
buraco:** *«o que isto deliberadamente não é: um grafo de estados com transições autoradas (a state
machine do Rive) — aquilo tem condições, entradas nomeadas e um editor próprio»*. Mais: os estados
dela são **papéis fixos** (Default/Hover/Pressed/Disabled), o gatilho é **derivado do rato**, e ela
**não é componente registado** ⇒ não viaja no ficheiro nem aparece no Inspector de um objecto de cena.

⇒ **O que falta não são acções. É a MEMÓRIA de em que estado se está, e o sítio onde se escreve
«SE estou em X E acontecer Y, vá para Z».**

---

## §2 — O ORÁCULO (§0.9), e as SETE leis que ele deu

**Triagem de licença primeiro:** **Godot 4.7.2**, `/usr/bin/godot`, **MIT** — porta ABERTA, sem
parede. Porta de consola medida: `godot --headless --script <s.gd> --quit`.

Corrido sobre entradas **nossas** por
[`godot_statemachine_probe.gd`](ferramentas/godot_statemachine_probe.gd) — o
`AnimationNodeStateMachine`, que é a única máquina de estados com transições autoradas que esta
máquina tem com licença permissiva.

| # | pergunta | o que o oráculo FAZ |
|---|---|---|
| 1 | duas transições satisfeitas, prioridades diferentes | **o número MENOR ganha** (`1` bate `5`) |
| 2 | …com a MESMA prioridade | **ganha a ÚLTIMA declarada** |
| 3 | uma condição num `advance_mode = ENABLED` | ⛔ **INERTE** — só o `AUTO` a lê |
| 4 | …no `AUTO`, condição verdadeira | transita **no MESMO avanço** |
| 5 | cadeia acíclica `A→B→C→D→E`, toda auto | **atravessa-se INTEIRA num avanço** (1 avanço ⇒ `E`) |
| 6 | ⭐⭐⭐ **ciclo `A↔B`, as duas auto** | **NÃO pendura** (14 µs) e avança **exactamente um estado por avanço** |
| 7 | auto-transição `A→A` | ⛔ **RECUSADA** pelo alvo (`has_transition` = `false`) |

⭐⭐⭐ **A linha 6 é a mais valiosa, e responde à pergunta que o `SignalActions` desta casa deixou
escrita:** o orçamento de profundidade **não é um contador** — é o **CONJUNTO DE VISITADOS deste
avanço**. Atravessa-se enquanto houver transição satisfeita e **pára-se ao voltar a um estado já
visitado neste tique**. É isso que faz a cadeia de 5 caber num avanço **e** o ciclo nunca pendurar,
sem nenhum número escolhido por ninguém.

⚠️ **E a sonda mentiu DUAS vezes antes de dizer a verdade** — as duas vezes por fixtura, não por
leitura: (a) sem arranque explícito a máquina fica no `Start` implícito e as **cinco** perguntas
liam `Start`, o que se teria lido como *«o Godot não transita»*; (b) a condição foi escrita no
modo `ENABLED`, onde é **inerte**, e a leitura *«não transita»* era um knob morto do alvo, não uma
lei dele. *Uma fixtura que o oráculo recusa nunca é comparada.*

---

## §3 — O desenho, com a porta ÚNICA de cada pergunta

### §3.1 — A entrada é o SINAL; a saída é o SINAL

- **O que faz uma transição disparar?** ⇒ **um nome de sinal.** O `ph2d-runtime` já tem **seis**
  produtores autorados (timeline · contacto · sensor · animação · relógio · nascimento/morte), e o
  #9 deu-lhes as tags. ⛔ **Não há expressões** — a autoria de expressões desta casa foi
  **retirada** por decisão (Timeline doc 14), e reintroduzi-la por uma porta lateral seria
  reconstruir trabalho recusado.
- **O que um estado FAZ?** ⇒ **emite um sinal ao ENTRAR e ao SAIR**, e o `SignalActions` (#5) faz o
  resto. ⛔ **Uma segunda lista de verbos seria uma segunda resposta a «o que pode acontecer neste
  app»** — e a lista do #5 já tem sink real para sete coisas.
  ⭐ É isto que **dissolve a recusa** do #5 *exactamente como ela prescreveu*: o consumidor que
  pedia o verbo que emite é este, e ele entra com o orçamento medido no §2.6.

### §3.2 — A arbitragem: a ORDEM DA TABELA, e é uma divergência DECLARADA

O oráculo tem um número (`priority`, menor ganha) porque um **grafo** não tem ordem visível. A nossa
autoria é uma **tabela ordenada que o artista vê e reordena**, como a do `SignalActions`.

⇒ **Ganha a PRIMEIRA linha satisfeita.** ⛔ Sem campo de prioridade: um segundo eixo de precedência
sobre uma lista que já tem ordem dá duas respostas à mesma pergunta, e a que o artista vê é a que
envelhece. ⚠️ **Divergência declarada e gateada** — e note-se que o desempate do alvo (*a última
declarada*) é o **oposto** do que a ordem de leitura sugere.

### §3.3 — A travessia: o conjunto de VISITADOS, portado do oráculo

`advance(machine, rt, sinais)`: enquanto houver transição satisfeita a partir do estado corrente,
entra nela — **e pára quando o destino já foi visitado neste avanço**. ⇒ uma cadeia resolve-se num
tique; um ciclo anda um passo por tique e **nunca pendura**. ⛔ **Sem `MAX_DEPTH`**: o recurso não é
um número, é o grafo.

### §3.4 — A MEMÓRIA: `StateMachine` (CONFIG, registado) + `StateMachineRuntime` (vivo, NÃO registado)

O molde é o do `Timer` (#2), letra por letra, **incluindo a cerca pelo TIPO**: o runtime **não
deriva `Serialize`**, logo registá-lo é **erro de compilação** e não um gate. ⛔ E o ledger do
`preview_drive` é **recusa medida** pelo próprio `Timer` (o `Driven` é `Copy` e o maior variante
manda no tamanho de toda entrada do ledger).

### §3.5 — ⛔⛔ O QUE ISTO OBRIGA A CURAR ANTES (W0): rebobinar não repõe o vivo

Medido pela **superfície da API**, não por grep: o `ph2d-ecs` exporta `advance` · `reconcile` ·
`start` · `stop` para o timer e **nenhuma porta de reposição** — *ninguém pode repor o relógio ao
rebobinar, mesmo que queira*. A única varredura de rebobinar que existe na shell é a
`sweep_spawned` das cópias da fábrica.

⇒ **um `Timer` que correu continua corrido depois de um Reset**, e o `LifetimeRuntime` o mesmo.
É a **mesma família** do report do dono de 2026-09-15 sobre o rewind dos projécteis, um nível acima
— e o `StateMachineRuntime` herdá-la-ia por construção.

⇒ **W0 cura-a com UMA porta para a família inteira**, com censo a impedir que um quarto runtime seja
esquecido. *A cura de um NOME repete-se; a de uma PORTA não.*

### §3.6 — O que NÃO entra, e porquê

| fora | motivo |
|---|---|
| Expressões / condições numéricas | a autoria de expressões foi **retirada** por decisão (Timeline doc 14) |
| Campo de prioridade | §3.2 — a ordem da tabela já é a precedência |
| Auto-transição `A→A` | **recusada pelo oráculo** (§2.7) e sem sentido: entrar onde já se está |
| Estados aninhados / sub-máquinas | um grafo dentro de um grafo pede um editor próprio; a tabela do Inspector não o exprime |
| Um verbo `EmitSignal` no `SignalActions` | ⛔ seria a classe dos laços **sem** o conjunto de visitados que só esta máquina tem |

---

## §4 — Onde encosta em contrato congelado (§6) ou schema

- **Contrato congelado: NÃO.** Nada em `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/
  `CanvasPaintTool`/`PanelEvent`, nem na superfície do `ph2d-vector-doc`.
- **`PROJECT_SCHEMA`: +1** (delta, nunca o literal) — um componente registado novo.
- **Registo do `ph2d-ecs`: +1**, e os **dois espelhos** (`ph2d-render` · `ph2d-script`) **+1 cada**,
  porque eles contam `ecs + …`. ⚠️ Conferir **no gate**, que imprime o `left:`.
- **`LIVE_SECTIONS`: +1** (a secção do Inspector).

---

## §5 — As quatro condições de UI (independentes)

1. **existe** — `StateMachine` no catálogo do `ph2d-component-desc`, família `Logic`.
2. **é pintado e registado** — secção do Inspector com a lista de estados e a tabela de transições.
3. **o clique chega ao barramento** — `StateMachineFieldEdit` pelo `EditorAction`.
4. **a sequência leva a algum lugar** — a cena de smoke: uma porta que abre ao sinal de um botão.

---

## §6 — As waves

| W | o quê | pronto quando |
|---|---|---|
| **W0** | a **reposição do vivo ao rebobinar**, uma porta para a família `Logic` | um `Timer` corrido volta ao zero depois de um Reset, com censo |
| **W1** | a **lei pura** (`ph2d-statemachine`), com o corpus do oráculo | as 7 leis do §2 gateadas |
| **W2** | o **componente** + a ponte no passo fixo + os sinais | a porta abre ao sinal, headless |
| **W3** | a **secção do Inspector** | as 4 condições do §5 |
| **W4** | a **cena de smoke** + provas de mutação + handoff | o dono corre e vê |

---

## §7 — ⚠️ As premissas DESTE plano que a implementação derrubou

*(escrito durante a construção, não depois — a §6 acima fica como foi planeada, de propósito)*

1. ⛔ **«W1: a lei pura numa crate-folha `ph2d-statemachine`»** — **REFUTADO.** A família `Logic`
   inteira (`Timer` #2 · `Lifetime` #12 · `SignalActions` #5) tem a lei **dentro do `ph2d-ecs`**, ao
   lado do componente. As folhas puras desta casa (`ph2d-topdown`, `ph2d-projectile`) existem porque
   são **matemática** partilhada com a ponte de física; esta é uma tabela de nomes. ⇒ uma crate nova
   seria o **primeiro** membro da família fora da casa dela, por zero ganho medido.
   ⇒ vive em [`ph2d_ecs::state_machine`](../../crates/ph2d-ecs/src/state_machine.rs).

2. ⛔⛔ **«§3.3 — a travessia pára no CONJUNTO DE VISITADOS, portado do oráculo»** — **REFUTADO por
   um gate vermelho sobre o meu próprio desenho**, e é o achado mais importante da wave.
   A regra do oráculo é boa **para a máquina dele**, cuja entrada é um **NÍVEL** (um
   `advance_condition` é um booleano que **fica** verdadeiro). A nossa entrada é um **EVENTO**.
   Portada à letra, ela dava uma porta que, com **um** toque, ia de `Aberta` a `Fechada` **e logo a
   `A abrir`** — porque a mesma seta de `botao` era ouvida outra vez pelo estado de chegada.
   ⭐ A lei que fica é **o sinal ser GASTO por quem o ouve**, e com ela:
   - a lei observável do oráculo fica **intacta** (uma cadeia resolve-se num tique quando os sinais
     lá estão; um ciclo nunca pendura);
   - o **orçamento de profundidade** que o `SignalActions` prescreveu passa a ser um recurso REAL —
     *quantos sinais soaram neste tique* — e **não** um `MAX_DEPTH` escolhido por alguém.
   ⇒ *o oráculo ensinou a PERGUNTA e o formato da resposta; a resposta é de quem conhece a natureza
   da própria entrada.*

---

## §8 — O que FICOU, e o que fica ABERTO

### §8.1 — As waves, como foram

| W | o quê | onde |
|---|---|---|
| **W0** | **Rebobinar é RENASCER** — uma porta para o estado vivo da família `Logic` | [`ph2d_ecs::rewind_runtime`](../../crates/ph2d-ecs/src/rewind_runtime.rs) |
| **W1** | a **lei** (estados · setas · arbitragem · consumo) | [`ph2d_ecs::state_machine`](../../crates/ph2d-ecs/src/state_machine.rs) |
| **W2** | o **componente registado** + a ponte no dreno de sinais | `render_loop::state_machine_tick` |
| **W3** | a **secção do Inspector** (duas listas, um editor cada) | `sections::statemachine` |
| **W4** | a **cena de smoke** com o CONTROLO ao lado | `PH2D_STATEMACHINE_SMOKE=1` |

### §8.2 — Aberto, nomeado

1. ⏳ **Estados ANINHADOS / sub-máquinas** — um grafo dentro de um grafo pede um editor próprio, e
   a tabela do Inspector não o exprime. ⛔ Recusado por desenho, não por falta de tempo.
2. ⏳ **Um verbo `EmitSignal` no `SignalActions`** continua fora. ⭐ A recusa dele dissolveu **para
   esta máquina** (que traz o orçamento), e **não** para a tabela: ela não tem conjunto de estados
   nem consumo, logo um verbo que emitisse ali reabriria a classe dos laços sem rede.
3. ⏳ **O custo a N máquinas não foi varrido.** O laço é `O(máquinas × setas)` por dreno, com
   `STATES_MAX = 16` e `TRANSITIONS_MAX = 32`; a cena do smoke tem uma.
4. ⏳ **A arbitragem é a ordem da tabela e a tabela não se REORDENA no painel** — hoje o artista
   apaga e reescreve. Um arrasto de linha é o `PanelRowFamily` que a wave das tags trouxe, e ele
   **existe**; ligá-lo aqui é uma linha de UI, não um mecanismo.
5. ⛔⛔ **NENHUMA das 26 secções opcionais do Inspector tem gate a provar que ela chega a PIXEL**
   — aberto pelo report do dono de 2026-09-15 (*«não apareceu no painel a seção state machine»*),
   que acabou por ser um defeito da CENA e não da pintura
   ([handoff §9-bis](handoffs/HANDOFF_INTEGRACAO_line_components_STATEMACHINE_2026-09-15.md)). A
   **presença** é gateada nos dois sítios que a decidem (o `build_*_info` e o censo
   `architecture_every_live_section_is_in_the_table`) e a **pintura** não é, porque não existe arnês
   de pintura na `ph2d-panel-inspector`. ⚠️ É a mesma forma que a lei
   *«uma faixa RESERVADA não é uma faixa PINTADA»* já mediu noutro painel — construí-lo é wave
   própria, e esta rodada **não** o construiu.
