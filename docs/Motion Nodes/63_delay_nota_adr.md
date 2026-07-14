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

## 4. A demo: a neve para de tremer

O `force.wind` dá um **gust** a cada floco, na fileira de ruído dele — cada um oscila sozinho. Um
polo simples na posição **DESENHADA** tira o tremor sem o ringing que uma mola somaria.

Ele fica **FORA da zona**, na cadeia de render: ele atrasa o **RESULTADO**, como o Delay Effector do
C4D. Realimentar uma posição suavizada **dentro** da simulação seria integrar a partir de um lugar
onde o floco não está.

Pequeno de propósito: 3 ticks (50 ms). Suaviza a oscilação e ainda deixa o splash ler como splash.
Suba o `Ticks` pra 20 e ponha o `Mode` em 0 (Delay) e a nevasca inteira chega **um terço de segundo
atrasada** — o mesmo nó, a outra pergunta.

## 5. A armadilha do gate (e ela é boa)

O gate e2e da neve **ficou vermelho** quando a ease entrou — e **estava certo**: ele media *o que é
desenhado* e chamava aquilo de *o mundo*. Uma afirmação sobre **física** ("o floco **bate** no
leito") tem que ser medida na **física** — a saída da **zona** —, não no desenho, porque o polo
simples **arredonda o fundo do mergulho**. Gate reescrito: física na zona, população/idade/cor no
sink.

E o gate NOVO (a ease está fazendo alguma coisa) quase nasceu inútil:

> **A métrica óbvia — a 2ª diferença (o *jerk*) — é dominada pela GRAVIDADE.** Uma parábola de
> aceleração constante tem 2ª diferença **constante**, e a ease não remove (nem deve remover) isso.
> Medindo assim, o tremor fica **enterrado** debaixo da queda: caiu **3%**, e eu tinha afirmado 10% —
> o gate ficou vermelho e me corrigiu.
>
> **A 3ª diferença de uma parábola é ZERO.** Então a 3ª diferença **É** o tremor, com a queda
> subtraída. Medida assim, ela **cai pela metade** (0,0745 → 0,0394).

É [[feedback_oracle_must_model_appearance_not_implementation]] pelo avesso: o oráculo tem que modelar
**a grandeza que a feature muda**, e não a primeira que vem à cabeça.

## 6. Superfície

- **Drop-crate nova:** `ph2d-node-motion-delay` (só `ph2d-nodegraph` + `ph2d-node-registry`).
- **`ph2d-node-registry-init` regenerado — 89 crates-nó** (era 88). Conflito esperado no rebase:
  **regenere, nunca resolva à mão.**
- **Shell:** a demo (`motion_demo_strobe.rs`, +1 card "Ease The Wobble") + o gate reescrito
  (`motion_state_tests.rs`).
- **Contrato congelado:** intocado.
- **3 mutações mortas:** o Blend virando mola (passa do alvo) · o histórico voltando a casar por
  linha · o `ticks=0` deixando de ser no-op.
- **Aberto:** o modo **Spring** do C4D não existe aqui **de propósito** (é o `motion.spring`, e é
  melhor); o atraso é de **posição** (ecoar linhas inteiras, cor e tudo, é o `motion.trail`).
