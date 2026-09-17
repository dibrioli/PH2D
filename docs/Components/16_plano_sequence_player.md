# TOP-20 #19 — `SequencePlayer`: a Timeline vira CUTSCENE de jogo

> **O item, na ordem do dono** ([levantamento §7](00_levantamento_componentes.md)):
> *«o difícil (religar bindings por `WireId`) está SHIPADO — o componente transforma a Timeline em
> cutscenes de jogo»*.
>
> ⚠️ **Este doc começa pela MEDIÇÃO**, porque a entrada do TOP-20 acima dele (`#3 SensorZone`) foi
> medida no mesmo dia e estava **fechada por composição**: a costura que ela pedia tinha sido
> construída pela wave das Tags, e reconstruí-la teria sido trabalho já pago (`CLAUDE.md` §5.0).

## §1 — O que a composição JÁ dá (medido em 2026-09-17)

| pergunta | resposta MEDIDA | onde |
|---|---|---|
| existe um `SequencePlayer`? | **não** — zero ocorrências na árvore | `grep -rn SequencePlayer` |
| a identidade estável de um objecto animado existe? | **sim**, `WireId` (sobrevive ao save, `0` = por resolver) | [`ph2d-timeline/src/binding.rs`](../../crates/ph2d-timeline/src/binding.rs) |
| existe uma «pilha promovida a coisa reutilizável», com relógio PRÓPRIO? | **sim**, `NamedContainer` (ADR-0133) — *«the temporal half … which has a clock of its own»* | [`nest.rs`](../../crates/ph2d-timeline/src/nest.rs) |
| esse relógio já corre em RUNTIME? | **não** — o `container_open` é uma **vista de EDIÇÃO** (`prime_rooted(root, playhead.time())`), e quem anda é o playhead da cena | [`autokey_pass.rs`](../../shells/desktop/src/render_loop/autokey_pass.rs) |
| um sinal consegue disparar tempo? | **só um `Timer`**: `SignalVerb::ALL` tem **8** verbos e nenhum toca uma sequência | [`signal_actions.rs`](../../crates/ph2d-ecs/src/signal_actions.rs) |
| a cutscene consegue ANUNCIAR-SE? | **sim, já** — um marcador carrega um sinal nomeado (ADR-0143) | [`doc_markers.rs`](../../crates/ph2d-timeline/src/doc_markers.rs) |
| a família dos componentes alcança a timeline? | **não** — `ph2d-app-components` não depende de `ph2d-timeline` | `Cargo.toml` |

⇒ **O buraco tem forma exacta e não é o que a tabela do TOP-20 diz.** O religar por `WireId` está
shipado *e* o relógio local existe **como vista de edição**; o que falta é **um relógio de
CORRIDA por objecto** e a porta que um sinal usa para o arrancar.

## §1-bis — ⛔⛔ E a SONDA encolheu a wave DUAS vezes (a §2 abaixo foi escrita antes dela)

*O plano manda correr a sonda antes da primeira linha, e foi ela que pagou o doc inteiro.*

1. ⭐⭐⭐ **O relógio de corrida NÃO se constrói: é o `Timer`.** Ele já tem `duration_us`, `repeat`
   (o loop), `autostart`, `signal` (**anuncia-se a cada disparo**) e um `TimerRuntime` com o
   decorrido — e o `progress()` dele é **derivado**, nunca guardado. Um sinal já o arranca e pára
   (`StartTimer`/`StopTimer`), o `rewind_runtime` já o faz **renascer**, e o `requires` do
   `ph2d-component-desc` já exprime *«este componente não funciona sem aquele»*.
   ⇒ ⛔ **o `SequenceRuntime` da §2 e o verbo `PlaySequence` MORREM**: seriam um segundo relógio e
   uma segunda maneira de dizer *«começa»*.
2. ⭐⭐⭐ **E aplicar um container no relógio dele também já existe:**
   [`apply_container(world, doc, ix, t, skip)`](../../crates/ph2d-timeline/src/apply_views.rs), cujo
   doc diz por escrito *«the interior you watch while editing and the interior an instance plays in
   the scene are ONE answer»* — e a esparsidade vale (um canal que nenhuma lane do container escreve
   fica com a pose da cena).

⇒ **O que SOBRA é uma coisa só:** *este objecto diz que container é o dele, e alguém chama aquele
apply no instante do Timer dele, pelo LEDGER.* Tudo o resto é composição do que já está pago.

⚠️ *Duas premissas do §2 caíram na primeira meia hora de medição — e é por isso que ela vem antes
do código, e não depois.*

## §2 — O desenho, com a PORTA única de cada pergunta

| a pergunta | a porta ÚNICA |
|---|---|
| *que sequência é esta?* | `SequencePlayer { container: String, … }` — o **nome** do `NamedContainer`, nunca um índice (um índice muda quando alguém apaga o container de cima) |
| *ela está a correr, e em que instante?* | `SequenceRuntime { t, playing }` — ⛔ **não registado** (a lei do #11/#20: o que nasce numa corrida não é documento), e **renasce** pela porta `rewind_runtime` |
| *quem a arranca?* | um **verbo** novo na tabela do #5 (`PlaySequence`), APENDADO — a posição é a tag |
| *a quem ela se aplica?* | ao **objecto que carrega o componente** e à subárvore dele — é o religar por `WireId` que já existe |
| *quando acaba, o que acontece?* | o que já existe: um **marcador com sinal** no fim do container; ⛔ nada de um segundo canal |

⚠️ **O risco nomeado, e ele decide a wave 1:** durante uma cutscene há **dois escritores** do mesmo
`Transform` (a timeline da cena e a sequência). A casa já tem a resposta — o **ledger** do
[`preview_drive`](../../crates/ph2d-preview-drive/src/lib.rs), com um `Driver` próprio —, e sem ela
*«a cutscene moveu o herói»* vira um passo de `Ctrl+Z` por quadro.

## §3 — Contratos congelados e schema

* ⛔ **Nenhum contrato do §6** (`Tool=12`, `NodeOp`, superfície do vector-doc) é tocado — prova por
  `grep`: as peças vivem em `ph2d-ecs` (componente), `ph2d-timeline` (leitura do container) e na
  composição da shell.
* `PROJECT_SCHEMA` **+1** (o `SequencePlayer` é registado). O `SequenceRuntime` **não** entra.
* `DOC_VERSION` da timeline: **0** — o container já existe; ler um container por NOME não muda bytes.

## §4 — O que a UI precisa (as QUATRO condições, independentes)

1. **existe**: `SequencePlayer` no catálogo do `ph2d-component-desc`, com rótulo;
2. **pintado e registado**: secção no Inspector (nome do container · autostart · loop), e os ids no
   `populate` — ⛔ é aqui que sete waves desta crate já morreram (*pintado e morto sob o dedo*);
3. **o clique chega ao barramento**: gate de costura com clique REAL;
4. **a sequência leva a algum lugar**: o verbo `PlaySequence` na lista do seletor **e** o id
   correspondente em `INSP_ACTION_VERB` — a wave do HUD provou que esses dois contam-se à parte.

## §5 — Os gates, red-first

* a lei pura do relógio local (arranca · anda · acaba · faz loop) na folha, sem mundo;
* **o ledger**: com a sequência a correr, a captura do undo devolve o valor AUTORADO — com CONTROLO
  (sem o componente, a captura não muda);
* **rebobinar RENASCE**: `rewind_runtime` repõe o `SequenceRuntime` — a 6.ª espécie;
* o verbo novo aparece **e** é alcançável (as duas metades: `SignalVerb::ALL` e o array de ids);
* costura: clicar na linha do Inspector chega ao barramento.

## §6 — A cena de smoke (o que ela tem de MOSTRAR)

Uma porta que, ao ser tocada, **corre uma cutscene**: a câmera vai, um objecto anda, e no fim um
marcador emite o sinal que devolve o controlo. ⚠️ **Com o CONTROLO ao lado** (o mesmo objecto sem o
componente), que é a lei que as cenas do #13/#14/#15 já seguem.

⛔ **Antes da primeira linha de código da W1**, a sonda `mede_o_que_a_composicao_ja_da_a_uma_cutscene`
tem de correr e imprimir os números desta §1 — *uma tabela escrita à mão envelhece; uma que a sonda
imprime, não*.

## §7 — O que a W1 e a W2 entregaram (2026-09-17)

| wave | o que ficou | onde |
|---|---|---|
| **W1** | o componente + a lei do NOME (`resolve`) + registo + degrau `145 → 146` + o descritor que **EXIGE** os `Timers` | [`sequence.rs`](../../crates/ph2d-ecs/src/sequence.rs) · [`catalog/logic.rs`](../../crates/ph2d-component-desc/src/catalog/logic.rs) |
| **W2** | *quais correm e em que instante* (`em_corrida`) + a FASE que as aplica pelo ledger | [`fase_sequences.rs`](../../shells/desktop/src/render_loop/fase_sequences.rs) |

**Gates:** 11 (5 da lei do nome · 6 do *quais correm*) + 4 da fase, **todos** pela porta do produto.
**Mutação: 5 de 5 sangram** ([`mutacao_seq_w2.sh`](ferramentas/mutacao_seq_w2.sh)).

⚠️⚠️ **E DUAS não sangraram à primeira, as duas por defeito MEU:**

1. um filtro com o nome errado casou **zero** testes — e o controlo do arnês apanhou-o, que é
   exactamente para isso que ele existe (*um filtro vazio sai verde e lê-se como «sobreviveu»*);
2. ⭐⭐ **a fixtura não continha o fenómeno:** com **um só** container o alvo é sempre o índice `0`,
   logo a mutação *«toca sempre o container 0»* era **inobservável**. A cura é uma **ISCA** — um
   container vazio antes do verdadeiro — e o gate ficou mais forte do que era.

## §8 — O que FALTA para o dono ver (W3) — ✅ **FECHADO em 2026-09-17**

1. a **secção do Inspector** (o campo `Container`), com os ids no `populate` — ⛔ é aqui que sete
   waves desta crate já morreram (*pintado e morto sob o dedo*);
2. uma **cena de smoke**: uma porta que, ao ser tocada, corre uma cutscene — **com o CONTROLO ao
   lado** (o mesmo objecto sem o componente);
3. e a foto, que é o único oráculo do lado PINTADO.

## §9 — O que a W3 entregou, e o que ela descobriu

> **O documento do INTEGRADOR é o
> [handoff de 17/09](handoffs/HANDOFF_INTEGRACAO_line_components_SEQUENCE_2026-09-17.md)** — os
> contadores como delta, a superfície de colisão, as sete leituras que o diff inverte e o que só a
> árvore COMBINADA pode reprovar.

| | onde |
|---|---|
| a secção **SEQUENCE** (a 30.ª de `LIVE_SECTIONS`) | [`sections/sequence.rs`](../../crates/ph2d-panel-inspector/src/sections/sequence.rs) |
| o vocabulário (instantâneo + edição) | [`sequence_edits.rs`](../../crates/ph2d-editor-core/src/sequence_edits.rs) |
| o instantâneo e o dreno | [`sequence_inspector.rs`](../../crates/ph2d-app-components/src/sequence_inspector.rs) |
| a cena `PH2D_SEQUENCE_SMOKE=1` | [`sequence_smoke.rs`](../../shells/desktop/src/sequence_smoke.rs) |

**Gates:** 5 de costura com **clique REAL** · 13 do instantâneo/dreno · 3 da porta da vista · 1 do
tecto. **Mutação: 12 de 12 sangram** ([`mutacao_seq_w3.sh`](ferramentas/mutacao_seq_w3.sh)).

### §9.1 — ⭐⭐ O controlo é um CHIP, e não um campo de texto

O conjunto das cutscenes é **conhecido** (os containers do documento, que a aba *Containers* já
lista). Um campo de texto obrigaria o artista a escrever um nome que casa **exactamente** — e
`"porta"` contra `"Porta"` é uma cutscene que não corre com todos os campos certos no ecrã. ⇒ o
mesmo idioma do verbo do sinal e do barramento do áudio.

⚠️ **E a edição carrega o NOME, nunca o índice da opção** — apagar um container renumera os de
baixo. O tecto das opções é o `ph2d_timeline::MAX_CONTAINERS`, com o gate na **shell** (a única
crate que vê o painel e a timeline), à maneira do `PROPS_MAX`.

### §9.2 — ⛔⛔⛔ O ACHADO: a aba de OMISSÃO da timeline parava TODAS as cutscenes

`Tab::Keys` é `#[default]` **e** é a que escolher um objecto **PEDE**
(`selection_jumps_to_keys`) — e ali o `keys_mode` é `true`, logo a condição da W2
(`container.is_none() && !solo`) **não deixa a fase correr**. ⇒ a cena mostrava duas portas
paradas com todos os dados certos, e o artista leria *«o componente não funciona»*.

⚠️⚠️ **A FOTO não o viu:** uma imagem mostra um **instante**, e *«mexeu-se»* é uma propriedade de um
**INTERVALO**. Quem o disse foi a **auto-conferência** que a cena faz sobre si mesma
(`MOVEU-SE 0.000`) — a lição do HUD (*«o dedo alcança o botão?»*) levada ao tempo.

Duas curas, e as duas são **portas**:

* [`fase_sequences::a_vista_deixa_correr`](../../shells/desktop/src/render_loop/fase_sequences.rs)
  — UMA porta com **dois leitores**: o `timeline_bridge`, que decide se a fase corre, e o
  instantâneo do Inspector, que **diz ao artista** porque é que a cutscene dele está parada.
  *Escrita duas vezes, o painel prometeria uma coisa e o motor faria outra.*
* `ph2d_panel_timeline::state::request_arrange_tab()` — a porta **simétrica** que faltava ao
  `request_keys_tab`, append-only, e que **vence** um pedido de Keys pendente. ⛔ Sem ela a única
  saída era **esconder** a timeline, que tira a régua do tempo ao artista exactamente quando ele a
  quer ver.

⚠️ **A fronteira que fica, e ela é de PRODUTO:** na aba *Keys* as cutscenes pausam. É a semântica
certa de autoria (ali sola-se a clip que se edita), e o painel **di-lo** em vez de a deixar muda.

### §9.3 — ⛔ O que a FOTO acusou e nenhum gate via

1. `Sprite::atlas(0, …)` em vez do `WHITE_TILE_KEY` ⇒ a cena inteira vermelha;
2. o Inspector **atrás** do painel do Sculpt 3D — a arrumação vive fora do repositório
   (`~/.ph2d/layout.txt`), logo `panel_visibility.insert("inspector", true)` não é defensivo;
3. a aba da §9.2.

### §9.4 — ⛔⛔ E a MUTAÇÃO acusou quatro defeitos meus

Dois deles são a mesma forma — **um corpus que não continha o fenómeno**:

* o gate da resolução comparava nomes **sem espaço**, e a lei (`SequencePlayer::resolve`) **apara os
  dois lados** ⇒ um `==` cru escrito à mão no instantâneo respondia igual e a mutação sobrevivia.
  A fixtura passou a ter `"Porta "`;
* o gate do relógio curto media um empate **EXACTO**, e a folga de um milissegundo só é observável
  quando os dois números diferem por **menos** do que ela (eles vêm de unidades diferentes:
  microssegundos inteiros contra `f64` de segundos). Nasceu um caso com `2,0000` contra `2,0005`.

Os outros dois foram do arnês: uma âncora com a indentação errada, e a fixtura do portão de costura
que **não compilava** depois de o instantâneo ganhar um campo — *o portão estava partido no momento
em que a prova correu*, e foi ela que o disse.

### §9.5 — ⏳ O que fica ABERTO

* a secção fica no **FIM** do painel, logo o dono tem de rolar para lá chegar — o mesmo item que o
  emissor de partículas (#18) deixou aberto, e a cura é a mesma para os dois;
* a cutscene não tem **transporte próprio** no painel (pausar/rebobinar só a dela): hoje isso faz-se
  pelo `Timer`, que tem secção própria;
* um objecto com **vários timers** usa sempre o `0`, e o painel não diz qual é.
