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
