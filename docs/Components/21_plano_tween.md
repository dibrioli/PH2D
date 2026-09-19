# Suplente #22 — `Tween` + os presets Fade/Flash: uma propriedade vai de A a B

> **O item, na ordem do dono** ([levantamento §7](00_levantamento_componentes.md), suplentes 21–25):
> *«`Tween` + presets Fade/Flash»*. A linha do catálogo (§4, 224/225) escreve o desenho antes de
> mim: *«anima qualquer propriedade registada, disparado por Signal»* e — a lei que governa a wave
> inteira — **«presets de 1 clique (lei 8: açúcar sobre o motor, nunca 2º motor)»**.
>
> ⚠️ **Este doc começa pela MEDIÇÃO.** Na wave anterior desta mesma lista (o **#21**) ela reescreveu
> a entrega — o pintor do raio já existia, e a wave passou a ser *publicar, e não pintar*; e o
> **#3** estava fechado por composição. A sonda é
> [`mede_o_que_a_composicao_ja_da_ao_tween`](../../crates/ph2d-timeline/tests/it/mede_o_que_a_composicao_ja_da_ao_tween.rs).

## §1 — O que a composição JÁ dá (medido em 2026-09-19)

⚠️ **A sonda mora no `ph2d-timeline`, e não na crate do componente, porque essa é a ÚNICA que vê os
dois lados**: o `ph2d-ecs` (o `Timer`, o `SequencePlayer`, os verbos), o `ph2d-anim` (o motor de
curvas) e — atrás da feature `render` — o `ph2d-render` (o `Sprite`, onde a opacidade e a cor de
facto moram). *Uma sonda que só visse um lado mediria o espantalho.*

| a pergunta | a resposta MEDIDA | o que ela decide |
|---|---|---|
| **um sinal sabe dizer *«ao longo de N segundos»*?** | **não**: `SignalVerb::ALL` tem **9** verbos, **3** leem o `arg` e os três leem-no como *nome de relógio* ou *delta de contador* | `Show`/`Hide` são um **degrau**, nunca uma rampa |
| **existe um `0 → 1` contínuo por objecto numa corrida?** | **sim** — `Timer::progress`, **derivado**, arranca por sinal e renasce no rebobinar (medido: `0,000 · 0,067 · 0,133 … 0,267` em 16 tiques) | ⛔ e o único canal para fora dele é `TimerOutcome { fires, finished }` — **dois inteiros, zero fracções** |
| **que propriedades a timeline anima?** | **13** canais: pose (5), `Opacity`, `TimeRemap`, `Morph`, `Position`, 4 de junta | ⭐ `Opacity` está lá ⇒ o **FADE** é exprimível |
| **…e a COR?** | **ZERO** canais de cor (`tint`/`color`/`fill`: lista vazia) | ⛔ o **FLASH** (`tint_fill` + `self_tint`) é **INEXPRIMÍVEL** por qualquer caminho de hoje |
| **um `SequencePlayer` + um container autorado JÁ fazem um fade?** | ⭐⭐⭐ **SIM**, e exactamente: `t = 0 → α 1,0000` · `0,125 → 0,7500` · `0,250 → 0,5000` · `0,375 → 0,2500` · `0,500 → 0,0000` | o concorrente é **real** e tinha de ser corrido, não suposto |
| **…e ele alcança um objecto NASCIDO na corrida?** | ⛔ **NÃO**: `1` ligação no documento, e ela **nomeia uma entidade**. O autorado mede `0,5000` e a cópia mede **`1,0000`** | é a linha que decide que isto é um **COMPONENTE** e não uma ligação |
| **o motor de curvas existe?** | **sim**: `11` famílias × `3` modos = **33**, `21` delas **sem transcendentais** (`is_deterministic`) | ⇒ *açúcar sobre o motor*: zero matemática nova |
| **quem o vê?** | `ph2d-ecs` **não**; `ph2d-app-components` **não** | a aresta é uma **decisão** desta wave, não um acaso |

⇒ **O buraco tem três nomes, e só o primeiro é sobre conforto:**

1. **o preço de autoria** — hoje um fade custa autorar um container com nome, keyar duas vezes,
   anexar um `Timer` **e** um `SequencePlayer`, e arrancá-lo; um *tween* é um componente e um
   dropdown;
2. ⭐⭐⭐ **a CÓPIA** — a linha acabou de shipar a **fábrica** (#11), os **projécteis** (#14) e as
   **partículas** (#18), que produzem objectos que **nascem durante a corrida**, e uma cutscene não
   lhes chega *por construção*: uma ligação é autorada e nomeia **um** objecto. *Vinte inimigos
   iguais precisariam de vinte ligações.*
3. ⛔ **a COR** — não há canal nenhum, em lado nenhum. O `Flash` que o levantamento pede não é
   *difícil*: é **inalcançável**.

### §1.1 — ⛔⛔ E a primeira redacção da SONDA mentiu, a favor do desenho que eu já tinha

O bloco (F) perguntava `manifesto.contains("ph2d-anim")` e leu **`true`** para o `ph2d-ecs`. A
ocorrência é um **COMENTÁRIO** a descrever de que é feita a `ph2d-morph-machine` (*«É uma FOLHA
(serde + ph2d-spring + ph2d-anim…)»*) — não uma dependência.

*Um censo textual que não separa prosa de código mede a prosa* — a família que este repo tem
escrita, paga aqui **dentro da sonda cujo trabalho é não deixar uma premissa passar por medida**. A
régua passou a ler só a secção `[dependencies]`, sem comentários, exigindo a FORMA de uma
dependência — **com controlo positivo** (`ph2d-core`, que as duas declaram), senão uma leitura
partida devolveria `false` a tudo e a sonda leria-se como confirmada.

### §1.2 — ⚠️⚠️ A ambiguidade que a medição desenterrou, e que é de OUTRO componente

O `advance` do `Timer` declara por escrito: *«Um one-shot que chega ao fim **pára e zera**»* —
`running = false`, `elapsed_us = 0`, logo **`progress() == 0`**.

⇒ um *one-shot* **terminado** e um que **nunca começou** são o **mesmo estado, bit a bit**. É
exactamente a ambiguidade que a auditoria da §11 Animation nomeou em 23/08 (*«pausado» e
«terminado» leem-se igual no `playing == false`, e não são a mesma coisa*) e que o #14 desta linha
pagou outra vez (o Inspector lia um EVENTO para pintar um ESTADO ⇒ `projectiles_finished()` ao lado
de `projectile_done()`).

⚠️ **Para o tween isto não é um detalhe: é a diferença entre um fade-out que fica e um que faz o
objecto REAPARECER** no quadro em que acaba. ⇒ **W0**.

## §2 — O desenho, com a PORTA única de cada pergunta

| a pergunta | a porta ÚNICA |
|---|---|
| *que propriedade, de onde para onde, com que curva?* | `Tween { channel, from, to, easing, on_finish }` — **CONFIG**, como o `SequencePlayer` |
| *quanto tempo?* | ⛔ **o `Timer`, e nada mais.** A medição do #19 já o tinha decidido: *«um `SequenceRuntime` seria um segundo relógio»*. Aqui vale letra por letra |
| *quem o arranca?* | ⛔ **zero verbos novos** — o `StartTimer` que existe desde a W3 do #2 |
| *qual relógio, se houver vários?* | o **índice**: o tween `i` corre no timer `i`, a mesma lei que liga `Timers` ↔ `TimerRuntime` (*«o índice casa, e é isso que os liga — não um nome»*) |
| *o que ele escreve?* | um canal com **aridade** declarada (1 número, ou 4 para a cor) — ⛔ nunca duas listas |
| *e no fim?* | `on_finish`: **`Hold`** (fica onde chegou) ou **`Rewind`** (devolve o autorado) — ⭐ *as duas são precisas, e é por isso que é um campo e não um palpite*: o **fade-out** quer `Hold`, o **flash** quer `Rewind` |
| *parado, o que ele escreve?* | **nada** — e é isso que faz o objecto voltar à cena sem uma linha a repô-lo (a lei do `em_corrida`) |
| *o que o `Ctrl+Z` vê?* | **nada da corrida** — o ledger do [`preview_drive`](../../crates/ph2d-preview-drive/src/lib.rs), com `Driver` próprio |
| *e o rebobinar?* | `rewind_runtime`, a porta que a W0 do #15 construiu |

### §2.1 — ⭐ Onde a lei mora, e porque a aresta já é legal

A lei vive numa **crate-folha nova `ph2d-tween`** (`serde` + `ph2d-anim`), e o componente no
`ph2d-ecs`. ⚠️ **Não é desenho novo: é o molde que a casa já ship** — a `ph2d-morph-machine` é uma
folha com exactamente essas duas dependências, e o `ph2d-ecs` depende dela para guardar o
`VecMorphMachine`. Folha → folha não sobe camada nenhuma
([`architecture_no_dependency_climbs_a_layer`](../../crates/ph2d-editor-core/tests/it/architecture_no_dependency_climbs_a_layer.rs),
catraca **vazia**, e continua vazia).

⛔ **E o `Easing` guarda-se como `Easing`**, nunca como um par de `u8`: uma segunda representação do
mesmo facto é a forma exacta do defeito que esta casa nomeia em cinco sítios. Ele é `Serialize`.

### §2.2 — Os canais, e de onde cada sink veio

| canal | escreve em | existia? |
|---|---|---|
| `Opacity` | `Sprite::tint[3]` | ✅ sink e entrada de ledger (`Driver::SpriteAlpha`) |
| `Tint` | `Sprite::self_tint` (RGBA) | ✅ o campo (ADR-0071, *o `self_modulate` do Godot*) · ⛔ **ninguém o conduz** |
| `PositionX` · `PositionY` | `Transform::translation` | ✅ sink · ⚠️ segunda mão sobre a pose |
| `ScaleX` · `ScaleY` | `Transform::scale` | ✅ |
| `Rotation` | `Transform::rotation` | ✅ |

⚠️ **A lista espelha a do `PropKind` de propósito, menos o que não faz sentido aqui** (`TimeRemap`,
`Morph`, as juntas) **e mais o que a timeline não tem** (a cor). ⛔ *Duas listas de «o que se pode
animar» divergem*, e a fronteira entre elas é declarada: a timeline anima o que uma LANE endereça,
o tween anima o que um OBJECTO carrega.

### §2.3 — ⚠️ O risco nomeado: a segunda mão sobre a pose

Um tween de posição e o solver escrevem o mesmo `Transform`. A casa já respondeu **duas vezes** —
`Driver::ScriptPose` e `Driver::PrefabStage`, as duas com a mesma frase: *a chave do ledger é
`(entidade, driver)`, e um objecto que seja também um corpo teria duas mãos na mesma entrada.* ⇒
**`Driver::TweenPose`** e **`Driver::TweenLook`**, apendados.

## §3 — Contratos congelados e schema

* ⛔ **Nenhum contrato do §6 é tocado.** ⚠️ O `AnimValue`/`LinearInterp` **é** superfície congelada
  (`ph2d-vector-traits`) — esta wave **lê-a** e não lhe acrescenta nada; o gate
  `architecture_vector_contract_surface` varre aquelas duas crates, e nenhuma é editada.
* `PROJECT_SCHEMA` **+1** (o `Tweens` é registado). O `TweenRuntime` **não** entra — e, como no
  `TimerRuntime`, a **ausência de `Serialize` é load-bearing**: registá-lo não dá um gate vermelho,
  dá **erro de compilação**.
* O registo do `ph2d-ecs` **+1**, e os **dois espelhos** (`ph2d-render` · `ph2d-script`) **+1** cada.
* `DOC_VERSION` da timeline: **0**. `SignalVerb`: **0** — ⭐ a wave não acrescenta um verbo.
* ⚠️ **Contar tudo como DELTA contra o `main`**, nunca o literal.

## §4 — O que a UI precisa (as QUATRO condições, independentes)

1. **existe**: `Tweens` no catálogo do `ph2d-component-desc`, com rótulo e `requires` os `Timers` —
   *sem relógio ele não é meia feature, é uma feature INERTE*;
2. **pintado e registado**: secção no Inspector (canal · de · para · curva · no fim), e os ids no
   `populate` — ⛔ é aqui que **oito** waves desta crate já morreram (*pintado e morto sob o dedo*);
3. **o clique chega ao barramento**: gate de costura com clique REAL;
4. **a semente** (`sync_tween`): ⛔ a lição que a **FOTO** da W6 do #21 cobrou há um dia — sem ela o
   painel mostra os valores de **fábrica** sobre um objecto autorado, e *quatro números plausíveis
   são a pior forma deste defeito*.

## §5 — As waves

| # | o quê | porquê agora |
|---|---|---|
| **W0** | o relógio aprende a dizer **«ACABEI»** (`TimerState`) | §1.2 — sem isto o fade-out faz o objecto reaparecer |
| **W1** | a lei: crate-folha `ph2d-tween` (canais, aridade, o valor, `on_finish`) | o motor é o `ph2d-anim`; aqui só a costura |
| **W2** | o componente `Tweens` + registo + schema + renascer | |
| **W3** | a ponte: a fase do quadro e o ledger | a ordem é o que faz o ledger compor (precedente: `fase_sequences`) |
| **W4** | a secção do Inspector + ids + `requires` + **a semente** | as quatro condições do §4 |
| **W5** | os **PRESETS** de 1 clique: *Fade In* · *Fade Out* · *Flash* | ⭐ açúcar que **escreve os campos** (o tween **e** a duração do timer), nunca um segundo motor |
| **W6** | a cena `PH2D_TWEEN_SMOKE=1`, a FOTO e as provas de mutação | a cena tem de mostrar a **cópia** a desvanecer — que é a coluna que a composição não tem |

## §6 — As decisões que a construção tem de MEDIR, não supor

1. ⏳ **A cor interpola em que espaço?** O `AnimValue::Color` é **OKLCH com arco curto de matiz**, e
   o levantamento chama a combinação de diferenciador. O `Sprite::self_tint` é RGBA do shader. ⇒ a
   W1 **mede** a rampa branco→vermelho nos dois espaços e o custo da conversão por quadro, e escolhe
   com a tabela ao lado.
2. ⏳ **Curvas transcendentais no passo fixo.** `21` das `33` são polinomiais
   (`is_deterministic == true`). Um tween que escreva `Transform` entra no `physics_ecs_c9`, que
   compara **bit a bit** entre três sistemas operativos. ⇒ a W1 decide se o canal de POSE aceita as
   `12` transcendentais, ou se elas ficam para os canais de aparência — **com o gate a dizê-lo**.
3. ⏳ **O tween `0` partilha o relógio com o `SequencePlayer`**, que também toma o timer `0`. Ou isso
   é *«a cutscene e o tween correm juntos»* (uma feature), ou é uma colisão. ⇒ nomeado, gateado, e a
   leitura escrita no ficheiro.
