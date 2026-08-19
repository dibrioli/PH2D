# Handoff de integração — `line/Vector` · W7m, **a MOLA**

**Status:** FECHADO 2026-08-08 · no `main` em `394bf9b80` (o commit que trouxe este arquivo).

> **Data:** 2026-08-08 · **Branch:** `line/Vector` · **Wave:** W7m (ordem do Enio)
> **Estado:** fechada, gates verdes, **PENDENTE DE SMOKE** e de ordem de integração.

---

## 1. A ordem, e o que ela obrigava

> *"Coloque a mola como uma opção a mais (um checkbox) e não prejudique nada do sistema de easing."*

As duas metades são independentes e as duas foram construídas:

- **opção**, não substituição — um hospedeiro sem mola é **byte-idêntico** ao que já shipava, e a
  `Option` ausente nem entra no caminho;
- **sem prejuízo** — toda curva contida em `[0, 1]` sai idêntica. ⚠️ **As DUAS que mudam são
  exatamente as duas cujo nome promete o que elas não entregavam** (§4).

---

## 2. O que uma mola compra, com o número ao lado

**Não é a FORMA.** `Elastic Out` mede pico **1,373** / assenta em **0,631** / **4** travessias
contra **1,309 / 0,600 / 3** de um oscilador real — a mesma animação.

O que ela compra é **continuidade de VELOCIDADE sob interrupção**, e há dois regimes **a um clique**
onde a curva morde:

| regime | a volta arranca a | o que o olho vê |
|---|---|---|
| `Cubic Out` (o default) | 1,34× | nada — indistinguível |
| **`Cubic InOut`** | **0,00×** | a cena **para e recomeça** |
| **`Elastic Out`** | **7,02×** | estalo |

⚠️ O seletor de curva existe desde a W7c, então os dois regimes deixaram de ser hipotéticos —
foi isso que tirou a mola de *"dispensável"* e a pôs em *"decisão de produto"*.

---

## 3. As linhas TROCAM, não somam

*Rigidez* e *amortecimento* respondem a mesma pergunta que *duração* e *curva*. Oferecer as quatro
seria pedir ao artista que mantivesse **dois modelos de acordo**, com a cena a obedecer a um deles
sem dizer qual.

- checkbox **Spring** marcado ⇒ **Stiffness** + **Damping**, e **NÃO** Duration/Curve;
- desmarcado ⇒ o par de sempre, **com o que ele já tinha afinado** (desligar não apaga).

⚠️ O gate central é de **PRESENÇA E AUSÊNCIA** — sem a metade da ausência, um painel que pintasse
as quatro linhas passaria.

⚠️ E o **checkbox sobrevive ao próprio modo** (gate próprio): sem isso a mola seria um caminho sem
volta.

---

## 4. ⚠️ A MUDANÇA DE COMPORTAMENTO — o clamp era GLOBAL e virou POR CANAL

**Esta é a única coisa desta wave que mexe no que já shipava, e é a que o smoke tem de julgar.**

A mola **não era entregável** com o clamp global: o primeiro quadro de uma reversão media
**0,000000** de deslocamento — o objeto **congelava** em vez de carregar o momento.

A linha nova é ***passar do alvo significa alguma coisa neste canal?***

| canal | `t` | porquê |
|---|---|---|
| posição, rotação | **cru** | passar do alvo **é** o movimento |
| escala | clampado | escala que vai a zero **espelha** o objeto |
| opacidade | clampado | alfa negativo não é uma tinta |
| tinta, largura | clampado | idem |
| geometria | clampado | um morph casado por Hungarian não tem significado além do destino |

**O efeito colateral é uma correção:** `Back Out` (pico **1,100**) e `Elastic Out` (**1,3731**) eram
postos em 1,000 — o artista escolhia *Elastic* e via um botão que apenas **chegava**. Metade do
seletor da W7c era controle morto.

⚠️ **Se o Enio preferir o comportamento antigo**, reverter esta metade é possível e o preço está
medido: a mola deixa de carregar momento, ou seja **deixa de comprar o que a distingue de uma
curva**.

---

## 5. ⚠️ E o doc contradizia o código DUAS linhas abaixo

Escrevendo a cena de smoke: o doc do `Transition::at` nomeia a opacidade (*"não é overshoot, é
lixo"*) e os ramos `Leaving`/`Entering` usavam o `t` **CRU** justamente nela.

Medido: **alfa −0,3999** com `Back Out`; e uma mola a carregar momento (`t < 0`) dava alfa negativo
a quem ENTRA.

⚠️ **Era LATENTE, e a honestidade importa mais que o susto:** o `install` da shell **não escreve**
`pose.opacity` hoje, então nada chegava à tela. Por isso o gate afirma **a LEI**, não um sintoma.

⚠️ **O gate irmão não podia pegá-lo:** um objeto presente nos dois lados nunca vira
`Leaving`/`Entering` — uma fixture com o mesmo id dos dois lados é verde por construção ali
[[reference_topic_fixture_discipline]].

---

## 6. A tabela de colisão

| Eixo | Valor | Nota |
|---|---|---|
| `PROJECT_SCHEMA` | **62** na linha · `main` diz **55** | ⚠️ **PROVISÓRIO** — se CONTA contra o `main` do dia, nunca se escolhe [[feedback_numbers_that_sum_across_lines_count_dont_pick]]. Esta wave **não o move** (a mola viaja no `HostStates`, dentro do blob que já era serializado) |
| `VEC_SCENE_SCHEMA_VERSION` | **14**, intocado por esta wave | |
| `FLIP_SCHEMA_VERSION` | intocado | |
| Registro do `ph2d-ecs` | **intocado** | |
| Contrato congelado | **4/4 verde** | rodado, não auto-relatado |
| ADR | **nenhum** na linha inteira | ⇒ fora de toda disputa de número |
| Dep externa nova | **nenhuma** | as crates novas do `Cargo.lock` são todas INTERNAS |
| Cena de smoke | **`=65`** | próximo livre: **66** |

### 6.1 O ponto de merge sensível

**`HostStates` ganhou o campo `spring: Option<Spring>`** — e ele é serializado. Uma linha que
apenda outro campo ao mesmo struct funde limpo e **as duas ordens ficam incompatíveis em
silêncio**, porque o postcard é posicional. Se houver outra linha a tocar `ph2d-ui-state`, confira
a ORDEM dos campos, não só o merge.

---

## 7. Gates e mutações

**Na `ph2d-ui-state`** (41 verdes no total):

| gate | o que ele afirma |
|---|---|
| `the_default_spring_settles_and_it_settles_fast` | ela CHEGA, e em tempo de UI (sem isto ela anima para sempre e `arrive` nunca corre) |
| `damping_decides_whether_it_overshoots` | o `ζ` não é um número inerte (o `x` final é 1,0 nos dois casos — só o PICO os separa) |
| `stiffness_decides_how_fast_it_gets_there` | idem para o `ω` |
| ⭐ `a_resumed_spring_carries_the_velocity_a_curve_would_have_dropped` | **a wave inteira num gate** |
| `the_trajectory_does_not_depend_on_the_frame_rate` | o passo do integrador é FIXO, não o `dt` do quadro |
| ⭐ `the_per_channel_law_holds_for_the_one_who_leaves_and_the_one_who_enters` | §5 |
| `reversing_a_spring_mid_flight_carries_its_momentum` | o mesmo pela `Machine` (a PROJEÇÃO no eixo novo) |

**No painel:** presença-e-ausência das duas famílias · o checkbox chega ao bus por **Down+Up REAL**
(o `Click` sintético **pula a checagem de focabilidade** e deixaria uma caixa pintada e morta sob o
mouse) · os dois knobs chegam como `SetValue`.

**Na cena `=65`:** 8 gates, dos quais dois valem por si —
`the_three_verdicts_the_scene_prints_are_true` (os vereditos do `announce` medidos no CI: senão a
cena só falha na mão do Enio, num sábado, com a suíte verde) e
`reverting_mid_flight_carries_momentum_and_the_curve_does_not`.

| # | Mutação | Sangra |
|---|---|---|
| M18 | o clamp volta a ser GLOBAL | **4** — 2 na crate, 2 na cena |
| M19 | a faixa Spring corre por CURVA | 2 |
| M20 | a mola retoma PARADA (`at_rest` em vez de `resuming`) | 2 — **uma em cada camada** (o integrador, e a `Machine` pela porta do produto) |

⚠️ **A régua dos sliders é UM número, e o painel NÃO depende da `ph2d-ui-state`** (por desenho: as
consts do painel são a face de UI, as da crate são a lei) ⇒ o gate de paridade mora onde os dois
lados são visíveis ao mesmo tempo — `the_spring_rulers_are_one_number`, em
`shells/desktop/src/vec_ui_state_edit_tests.rs`. Uma cópia em qualquer um dos dois seria a segunda
resposta a *"até onde este slider vai?"*.

---

## 8. O smoke

```
env PH2D_BUILD_SMOKE=65 cargo run -p ph2d-host-desktop --release
```

⚠️ **A cena imprime o pico de cada faixa antes do roteiro.** Se aparecer `!! PARE`, pare: ou a mola
não passa da marca, ou o Back não passa, ou o **controle** passou — e o terceiro é o que responde à
metade *"não prejudique nada do easing"*.

O passo que decide é o **3**: entre na faixa **Spring** e **saia antes de ela chegar**. Ela continua
um instante para onde ia. Depois o mesmo na faixa **Curve** — ela **para** e arranca do zero.

O passo **4** é o que precisa do seu veredito: a faixa **Back** agora passa da marca. Era isso que o
nome da curva sempre prometeu, e é a única coisa desta wave que muda o que já shipava.

---

## 9. Aberto, nomeado

- **O `Disabled` continua sem gatilho** — é fato do DOCUMENTO, não do rato (herdado da W7r).
- **A mola é por-HOSPEDEIRO**, como a duração e a curva: *"ir"* e *"voltar"* usam os mesmos
  números. Quem quiser assimetria tem a timeline — a mesma decisão que o `duration_s` já carrega.
- **W8a** segue bloqueado por **ausência**: `ph2d-runtime` não existe.
