# 63 — `motion.delay` (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-14). O **último** nó da fila de
> fan-out do plano. Contrato congelado **intocado**. **89 crates-nó.**

## 1. A primeira pergunta foi se ele ainda tem emprego

O plano nomeou `motion-delay` **antes** de existirem `time_remap`, `trail` e `slit_scan`. Então a
pergunta honesta não era *"como implemento?"*, era *"isto ainda serve pra quê?"*. Serve — pra
**exatamente uma** coisa, e não é a que está no nome dele:

| você quer | você já tem |
|---|---|
| a mesma sub-árvore **Pure**, cozida em `t − d` | **`motion.time_remap`** — exato, sem estado, scrub-perfeito, de graça |
| gerações passadas como **cópias** que somem | **`motion.trail`** |
| uma **rampa** de atrasos pelo conjunto (o elemento *i* vê `t − i·lag`) | **`motion.slit_scan`** |
| uma perseguição **com overshoot** e assentamento | **`motion.spring`** |
| **atraso SEM overshoot** — um suavizador | **nada.** Este nó. |

**Uma mola não pode ser um suavizador:** ela passa do alvo por construção, e uma entrada trêmula a
faz *tocar sino*. Um polo simples (one-pole) só consegue **aproximar** — e é por isso que todo
pacote de composição te dá os dois.

E o `time_remap` **não consegue atrasar uma simulação**: uma sim não é função de `t`, então você não
pode re-cozinhá-la um segundo atrás. O único jeito de ver onde ela **estava** é ter **guardado**.
Isso é o ring.

## 2. A referência

**Cinema 4D, Delay Effector** — que, apesar do nome, não é uma linha de atraso: ele senta **depois**
do que move o seu conjunto e **lag** o resultado, e o **modo default dele é Blend** (uma ease
exponencial rumo ao valor vivo). Os modos dele são Average, Blend e Spring; os nossos são **Delay**,
**Average** e **Blend** — porque mola nós já temos, e a nossa é melhor.

| modo | o que é | pra quê |
|---|---|---|
| **Delay** (0) | a posição de `ticks` atrás (fracionário: 3,4 ticks interpola entre os slots 3 e 4) | *"o B faz o que o A fez 12 quadros atrás"* |
| **Average** (1) | a média das últimas `ticks` posições — um boxcar passa-baixa | **mata** tremor (um ±1 alternado vira ~0), ao custo de meia janela de atraso |
| **Blend** (2, default) | o polo simples: `out += (live − out) / ticks` | **atrasa, suaviza e NUNCA passa do alvo.** É pra isto que o nó existe |

**`ticks = 0` é no-op byte-idêntico** em todos os modos — o ponto neutro: largar o nó numa cadeia
não muda nada até você pedir.

## 3. A decisão que faz o nó FUNCIONAR (e que o `slit_scan` não tomou)

**O histórico segue o ELEMENTO, não a LINHA.**

O ring do `slit_scan` casa a linha *i* do estado com a linha *i* do vivo, e **re-semeia a linha
inteira quando a contagem muda**. Isso serve pra uma grade. E é **inútil dentro de uma zona de
simulação**: um sistema de partículas nasce e morre em quase todo tick, então a contagem muda o tempo
todo, o histórico re-semeia o tempo todo, e **o nó vira um no-op silencioso** — verde, ligado, e sem
fazer nada.

Então este casa por **`id`** quando o stream tem um (todo stream de zona tem — o `sim.spawn` o
cunha). Um recém-nascido **não tem passado**, então a linha dele semeia **plana** onde ele está: ele
começa sem atraso, em vez de herdar o histórico de um estranho. Sem `id` (uma grade, uma
distribuição — um conjunto cujas linhas **são** a identidade dele) ele cai no casamento por linha.
**Os dois mundos, um nó.**

## 4. ⚠️ CORREÇÃO — **a neve NÃO treme**, e a demo original era falsa

A primeira versão desta nota dizia que o nó *"tirava o tremor da neve"*. **O Enio duvidou. Ele estava
certo.** Medido:

| | queda | desvio da aceleração (o "tremor") | excursão lateral |
|---|---|---|---|
| a neve, com o `gust` do demo | 85 ticks | **0,00024 = 0,1% de um floco** | **ZERO** |
| a neve, sem gust | 75 ticks | 0,00000 (parábola perfeita) | zero |

**O `gust` do `force.wind` modula a MAGNITUDE de uma força que aponta reto pra baixo.** O floco cai em
**linha reta** — só que mais rápido ou mais devagar. Não há oscilação, não há flutter, não há deriva.
O efeito real do gust é que uns flocos caem ~13% mais devagar que outros: variação de **ritmo**, não
tremor.

E a **queda de 47% na 3ª diferença** que eu tinha medido e celebrado? Era a ease amaciando o
**SPLASH** (a batida no leito) — **não** tremor de gust. **O número era certo; a história que contei
em cima dele era errada.** É a mesma família do erro do doc 61 §2, uma jornada atrás
([[feedback_stale_comment_and_dead_code_lie]] 3º caso), e ele tem nome próprio agora:
[[feedback_a_correct_number_can_carry_a_false_story]].

**O nó saiu do documento de boot.** Ele estava atrasando o desenho (até 89% de um floco) e amaciando
o splash — nenhuma das duas é a feature, e uma delas era um dano (o splash é o que prova o
`sim.collide`).

## 5. A demo de verdade: `PH2D_MOTION_DELAY_SMOKE=1`

Um suavizador precisa de algo **trêmulo**, e a simulação produz movimento **suave**. Então o nó ganhou
a cena em que ele **é** o que ele diz ser — duas fileiras, o **mesmo** `motion.wiggle`, e a única
diferença é o nó:

```text
grid → move(cima)  → wiggle(f=8) ──────────────────→ scale → output   ← TREME
grid → move(baixo) → wiggle(f=8) → delay(Blend, 6) → scale → output   ← SEDOSO
```

**Medido, os dois:** o `motion.wiggle` a f=8 sacode **0,095 de mundo por tick — 53% da largura do
próprio objeto, a cada quadro**. Com a ease: **0,036 (20%)**. O tremor cai **61%** e a **excursão
sobrevive** (1,00 → 0,92).

É essa a promessa, e é essa a forma do gate (`the_ease_kills_the_twitch_and_keeps_the_motion`), que
**falha nos dois sentidos**: se o nó deixar de suavizar, **e** se ele suavizar até achatar o gesto —
*um suavizador que mata o movimento não é um suavizador, é um mute*. E ele começa afirmando que **o
fixture treme** (`raw_twitch > 0.07`): sem isso eu estaria medindo a suavização de nada, que é
exatamente o erro que este gate corrige.

## 6. Superfície

- **Drop-crate nova:** `ph2d-node-motion-delay` (só `ph2d-nodegraph` + `ph2d-node-registry`).
- **`ph2d-node-registry-init` regenerado — 89 crates-nó** (era 88). Conflito esperado no rebase:
  **regenere, nunca resolva à mão.**
- **Shell:** `motion_delay_smoke.rs` (a cena A/B, atrás de `PH2D_MOTION_DELAY_SMOKE=1`) + o gate
  `the_ease_kills_the_twitch_and_keeps_the_motion`. **O boot NÃO tem o nó** (§4).
- **Contrato congelado:** intocado.
- **3 mutações mortas:** o Blend virando mola (passa do alvo) · o histórico voltando a casar por
  linha · o `ticks=0` deixando de ser no-op.
- **Aberto:** o modo **Spring** do C4D não existe aqui **de propósito** (é o `motion.spring`, e é
  melhor); o atraso é de **posição** (ecoar linhas inteiras, cor e tudo, é o `motion.trail`).
