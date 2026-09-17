# `CounterWatch` — o contador que REAGE, e o `Health` que cai por composição

> **Estado:** plano. A wave nasce da MEDIÇÃO abaixo, não da tabela do TOP-20 — e a medição mudou o
> que ela é: o item que o levantamento chama de `Health` (suplente #24) **não é um componente**, é
> a composição que existe assim que um contador puder falar.

## §1 — ⭐⭐⭐ A medição que abriu a wave (2026-09-17)

O TOP-20 fechou (o #17 é do projeto Tiling, ordem do dono). A §5.0 do `CLAUDE.md` manda medir a
composição **antes** de construir — foi ela que dissolveu o `#3 SensorZone` e o `SpawnPoint` do #11.
Corrida sobre a lista inteira, ela devolveu **um** buraco, e ele é exacto:

| elo | existe? | onde |
|---|---|---|
| um número autorável na cena | ✅ `Counter { name, start }` | veio com o HUD (#20) |
| alguém que o MOVA | ✅ `SignalVerb::AddToCounter` | idem |
| alguém que o MOSTRE | ✅ `UiLabel` + `LabelSource::Counter(nome)` | idem |
| alguém que **REAJA** a ele | ⛔ **NINGUÉM** | — |
| a válvula de escape (#16) chega lá? | ⛔ **não** | a API do Luau tem `emit`/`spawn`/`despawn`/`get`/`set`/`state_*`/`find_by_name`/`input` — e **nenhuma porta para contadores** |

⇒ **o artista consegue contar e VER a conta, e não consegue fazer acontecer nada a um número.**
Três vidas chegam a zero e o jogo não sabe. Dez moedas não abrem porta nenhuma.

⚠️ **E a leitura por `grep` não bastou** — foi a lição do #3. O veredito saiu de censar os verbos
(`SignalVerb`, 8 variantes), os leitores do `Counter` (o rótulo, o rebobinar, o registo) e a
**superfície da API** do script, que é onde uma capacidade se afirma ou não existe.

## §2 — O estado da arte, e o que ele já tentou

| referência | como se exprime | o que se aprende |
|---|---|---|
| **Construct 3** | condição *«Compare variable»*, mais a condição separada ***«Trigger once while true»*** | ⭐ a separação é a lei: uma comparação é um **NÍVEL** e um gatilho é uma **ARESTA**, e o produto obriga o artista a dizer qual quer |
| **Unity Visual Scripting** | *On Variable Change* + comparação | a mudança é o evento; o valor é o argumento |
| **Godot** | `if health <= 0: emit_signal("died")` | só por código — não há autoria sem script |
| **GDevelop** | condições sobre variáveis, avaliadas por quadro | nível puro ⇒ o autor paga o *«disparou 60×»* |
| **Unreal Blueprints** | comparação + *Do Once* | a mesma separação do Construct, com outro nome |

⭐⭐⭐ **A lição que decide o desenho: um nível avaliado por quadro DISPARA POR QUADRO.** Com as
vidas a zero e um nível puro, o verbo *Hide* correria 60 vezes por segundo e a tabela de acções
encheria o orçamento de profundidade que o #15 mediu. ⇒ **a nossa lei é a ARESTA**, e o
*«só uma vez»* do Construct/Unreal fica como um **segundo** knob por cima dela.

⚠️ E esta casa já pagou esta distinção, na wave anterior: o #15 portou a lei de um oráculo cuja
entrada é um **nível** e, escrita à letra, uma porta ia a `Fechada` **e logo a `A abrir`** com um
toque. *A lei que ficou foi o sinal ser um EVENTO.* Aqui ela nasce assim.

## §3 — O desenho

```rust
pub enum Compare { AtMost, AtLeast, Exactly }   // <= · >= · ==

pub struct CounterWatchRow {
    pub counter: String,   // QUAL contador — pelo NOME (a lei da casa; o índice renumera)
    pub compare: Compare,
    pub value: i64,
    pub signal: String,    // o que emitir na travessia
    pub once: bool,        // só a primeira vez
}
pub struct CounterWatch(pub Vec<CounterWatchRow>);        // REGISTADO (config)
pub struct CounterWatchRuntime(pub Vec<WatchSlot>);       // ⛔ NÃO registado (estado vivo)
pub struct WatchSlot { pub held: bool, pub fired: bool }
```

### §3.1 — ⭐⭐⭐ A PORTA ÚNICA: *«quanto vale o contador X?»*

O `UiLabel` já responde a isto, e responde **SOMANDO** todos os `Counter` com aquele nome — a ordem
de iteração entre arquétipos não é prometida, logo *«o primeiro»* não significa nada. A vigia **tem
de ler a mesma soma**.

⇒ a soma sai para [`ph2d_ecs::counter::soma`], com **dois leitores**: o rótulo e a vigia.
⛔ *Escrita duas vezes, o placar mostraria um número e a regra reagiria a outro* — é a mesma forma
do `a_vista_deixa_correr` da wave anterior, e há censo a afirmá-la.

⚠️⚠️ **E ela devolve `Option`, nunca `0`** — e isso é herdado, não inventado: o doc do `valor` já
escreve que um zero inventado *«se leria como o jogo está a funcionar e a pontuação é zero»*.
⇒ **uma vigia sobre um contador que não existe NUNCA dispara.** Sem isto, escrever `vidas` com um
`d` a mais faria a vigia `AtMost 0` disparar *«morreste»* no arranque — e o artista não teria como
saber porquê.

### §3.2 — A ARESTA, e onde ela mora

Por tique: `tem = compara(soma, valor)`; emite **sse** `tem && !slot.held`. O `held` vive no
runtime **não registado** — a lei que a física escreveu e que o `Timer`, o `Counter`, o cérebro e a
fábrica já honram: *config no componente, estado vivo ao lado, senão cada quadro vira um passo de
`Ctrl+Z`*.

**Nascer é `held = false`**, e isso é uma decisão com consequência visível: uma condição que já
está satisfeita no tique 0 dispara **uma vez** no tique 0. É o que o Construct faz, e é o que
impede o caso *«a porta nasce aberta e ninguém o diz»*. ⇒ `rewind_runtime_state` repõe-o — a porta
que o #15 construiu, e a **sétima** espécie a passar por ela.

### §3.3 — QUANDO, no quadro

Com **os motores** (cérebros · scripts · partículas), **antes** de a tabela de acções ler — a janela
que o `fase_signal_outbox` já documenta: *«é isso que faz uma porta abrir no MESMO quadro em que o
botão é tocado»*.

⚠️ **A latência é a mesma nas duas margens e isso foi medido no papel antes de escolher:** falando
antes das acções, a vigia vê o valor que o quadro anterior deixou e a reacção é **no mesmo quadro**;
falando depois, vê o valor de agora e a reacção é **no seguinte**. `1` quadro nas duas. ⇒ escolhe-se
a que fica **junto dos irmãos**, porque é essa que o censo de ordem do quadro já cobre.

### §3.4 — ⭐⭐ O `Health` não se constrói: ele CAI

| o que o artista quer | como se autora, depois desta wave |
|---|---|
| três vidas | `Counter { name: "vidas", start: 3 }` |
| levar um golpe | `SignalOnHit("golpe")` + `SignalActions[golpe → AddToCounter(-1)]` |
| **morrer** | `CounterWatch[vidas AtMost 0 → "morri"]` |
| o que a morte faz | `SignalActions[morri → Hide · PlaySound · …]` · ou um `StateMachine` |
| o placar | `UiLabel { source: Counter("vidas") }` — **já existe** |

⇒ o suplente **#24 `Health`** do levantamento é **composição**, e construí-lo como componente seria
uma segunda resposta a *«quanto vale este número?»*. ⭐ *É a §5.0 a pagar-se outra vez: o que se
perde ao não reconferir não é tempo, é construir o que já existe.*

## §4 — Contrato congelado e schema

| superfície | toca? | prova |
|---|---|---|
| Nodes (`NodeOp`/`OpResolver`/`NodeManifest`) | ⛔ não | a wave não abre `ph2d-nodegraph` |
| Tools (`Tool=12`/`RasterEditTool`/`CanvasPaintTool`) | ⛔ não | não há ferramenta de canvas |
| Vector doc | ⛔ não | — |
| `PROJECT_SCHEMA` | **+1** | componente registado novo. ⛔ **DELTA contra o `main` em que aterra**, nunca o literal |
| registo `ph2d-ecs` + os **dois** espelhos | **+1** cada | `ph2d::ecs::CounterWatch` |
| `LIVE_SECTIONS` / `any_live_section` | **+1** | a secção *Counter Watch* |
| `SignalOrigin` | **+1** variante, **append-only** | `CounterWatch { counter, entity }` |
| ADR | **0** | nenhuma decisão de arquitectura nova — a forma é a do `Timer` |

⚠️ O `CounterWatchRuntime` **não se regista**, e a ausência é a lei (§3.2).

## §5 — A UI, nas quatro condições independentes

1. **o componente EXISTE** — `ph2d-component-desc` com família `Logic`, e `Counter` **requerido**?
   ⛔ **não**: a vigia pode viver num objecto *«Regras»* sem contador nenhum — o alvo é o NOME.
2. **é pintado e registado** — a secção, os ids das linhas, e o `populate`. ⛔ *é aqui que sete waves
   desta crate morreram* (pintado, hit-registado e **morto sob o dedo**).
3. **o clique chega ao barramento** — `InspectorCounterWatchEdit`, append-only no `EditorAction`.
4. **a sequência leva a algum lado** — a fase aplica, o mundo muda, e o gate de costura carrega com
   `Down`+`Up` REAIS.

## §6 — Os gates, red-first

**Lei pura** (`ph2d-ecs`): a aresta dispara uma vez · o nível repetido **não** dispara · re-travessia
dispara outra vez · `once` cala a segunda · contador **inexistente** nunca dispara · as três
comparações · a soma de dois contadores homónimos · nascer com a condição satisfeita dispara no
tique 0 · rebobinar re-arma.
**Porta**: o rótulo e a vigia leem a **mesma** soma (censo de dois leitores).
**Ponte**: o sinal chega à tabela de acções no mesmo quadro · e não dispara com o relógio parado.
**Costura**: clique real nas linhas da secção.
**Tecto**: `LIVE_SECTIONS` e a arity do `any_live_section`.

⚠️ **A fixtura tem de conter o fenómeno:** o gate da aresta precisa de um contador que **atravesse**
o limiar (não que já esteja lá), e o do `once` precisa de **duas** travessias — senão as duas leis
são indistinguíveis.

## §7 — A cena de smoke, com o CONTROLO ao lado

`PH2D_COUNTERWATCH_SMOKE=1` — **três vidas**: um obstáculo tira uma vida de cada vez que toca no
herói; o placar no HUD desce `3 · 2 · 1 · 0`; a zero, a vigia diz *«morri»* e o herói desaparece.
**Ao lado, o CONTROLO**: o mesmo herói com o mesmo contador e **sem a vigia** — o placar dele chega
a `0` e **nada acontece**, que é exactamente o buraco que a §1 mediu.

⚠️ A cena **confere-se a si mesma** sobre um intervalo (a lição da wave anterior: a foto mostra um
instante, e *«desapareceu ao chegar a zero»* é uma propriedade de um INTERVALO).

---

## §8 — ✅ FECHADO (2026-09-17) — e o que a construção mudou no plano

| §  | o plano dizia | o que ficou |
|---|---|---|
| §3 | `Compare` com 3 variantes | igual — `#[repr(u8)]`, append-only |
| §3.1 | a soma sai para uma porta com **dois** leitores | igual, e ela passou a `&World` (`try_query`) para o instantâneo a poder ler |
| §3.2 | a aresta, `held` no runtime não registado | igual; a **sétima** espécie da porta do rebobinar |
| §3.3 | falar com os motores | igual — e a latência foi confirmada idêntica nas duas margens |
| §3.4 | o `Health` cai por composição | ✅ e a cena prova-o |
| §5 | quatro condições da UI | ✅ as quatro, com **5 gates de costura** de clique real |
| §7 | a cena com placar de texto | ⛔ **mudou** — ver §8.3 |

**Contadores, como DELTA:** `PROJECT_SCHEMA` **+1** · registo do `ph2d-ecs` **+1** · os **dois
espelhos +1** cada · `LIVE_SECTIONS` **+1** (30 → 31) · `any_live_section` **+1** (24 → 25) ·
`SignalOrigin` **+1** (append-only) · `EditorAction` **+1** (append-only). ⛔ Contrato congelado
**0**, ADR **0**.

**Gates:** 8 (lei) + 1 (rebobinar) + 4 (ponte) + 5 (costura, clique REAL) + 2 (tecto/ordem, na
shell) + 6 (cena) + 2 (a corrente inteira). **Mutação: 25 de 25** —
[`mutacao_counter_watch.sh`](ferramentas/mutacao_counter_watch.sh).

### §8.1 — ⛔⛔⛔ O `SignalOrigin` é `Copy`, e isso ESCOLHEU o que a origem carrega

A 1.ª redacção punha lá o **nome do contador** (`Arc<str>`) e tirava o `Copy` a **todas** as
origens. ⇒ ela carrega o **índice da regra** (`u16`), que é estritamente mais forte: dele tira-se o
contador, a comparação e o limiar; do nome do contador não se tira qual das regras falou.

### §8.2 — ⛔⛔ Três defeitos que só a CORRENTE INTEIRA apanhou

O gate `counter_watch_chain_tests` percorre `relógio → AddToCounter → contador → vigia → sinal →
Hide` sem janela nenhuma. Ele reprovou **três** vezes, e nenhuma das causas era a lei:

1. **A IDENTIDADE.** O `resolve_signal_actions` consulta `(Entity, &SignalActions, &StableId)` ⇒
   **um mundo sem identidade não reage a sinal nenhum e não o diz**: os relógios falavam, a
   resolução devolvia zero, o contador ficava parado. É a família do defeito que as Tags pagaram
   em 14/09. *O arnês tem de reproduzir o passo da shell, senão mede outro programa.*
2. **A `Visibility`.** O verbo `Hide` lê a visibilidade de ANTES para a declarar ao ledger, logo um
   alvo **sem** o componente é **inerte** — e o relatório conta-o como `inert` **sem gritar**. ⇒ a
   cena passou a spawnar `Visibility::visible()` explicitamente, como a irmã do #5 já fazia.
3. ⭐⭐⭐ **UM SINAL É GLOBAL POR NOME.** Com o herói e o controlo à escuta de `"morri"`, o
   controlo **desaparecia junto com o herói** — uma tabela reage a um nome venha ele de quem vier.
   ⇒ o controlo espera por `"morri_controlo"`, que **ninguém diz nesta cena**, porque quem o diria
   era a vigia que ele não tem. *É esta a forma certa de um controlo: falta-lhe quem DIGA, não quem
   faça.*

### §8.3 — ⛔⛔ E a FOTO mudou a cena duas vezes

1. **Os placares estavam fora do ecrã.** Eu pendurara-os numa raiz de HUD com coordenadas de
   **referência** (`−0,28 × 1280`) e, sem câmera de jogo, o canvas fica na identidade ⇒ o rótulo
   aterrava a `−358` no mundo, a trinta e seis ecrãs de distância — com os cinco gates **verdes**,
   porque eles perguntam o que a cena **MONTA** e nenhum perguntava **ONDE**.
2. **E no sítio certo eles desenhavam-se como ANÉIS VAZIOS.** Um `UiLabel` **troca o que um texto
   MOSTRA; ele não cria o texto** — sem um `VecShape::Text` autorado por baixo não há glifos.
   ⇒ o placar **saiu** e entraram **três LUZES de vida por lado**, apagadas por **três regras no
   mesmo contador** (`≤2`, `≤1`, `≤0`).

⭐⭐⭐ **E a troca deixou a cena MELHOR do que o plano pedia:** um número a descer não mostra que uma
vigia dispara **a um limiar**; três luzes a apagarem-se uma a uma mostram-no sem uma palavra. O
controlo fica com as **três acesas** enquanto o contador dele desce igual.

### §8.4 — ⏳ O que fica ABERTO

* a secção fica no **FIM** do painel — o mesmo item que o #18 e o #19 deixaram, e agora com números:
  a coluna do Inspector mostra **836 px** e cada secção custa **~140–230 px**, logo **quatro** já a
  enchem. ⚠️ A dobra existe e é **global por secção**, mas vive só em memória (`BTreeMap<NodeId,
  bool>` no store) ⇒ **morre ao fechar o app**. *Que ordem, e o que nasce fechado, é decisão do dono.*
* a vigia não sabe dizer *«quando o contador MUDAR»* (só limiares) — nenhum consumidor o pediu;
* dois contadores homónimos **somam**, e a secção não o diz (o painel mostra a soma em *Now*).
