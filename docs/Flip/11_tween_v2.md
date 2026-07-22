# Flip §11 — Tween v2: a correspondência espacial e a espiral logarítmica

> **O que esta wave entrega:** o inbetween deixa de parear por ÍNDICE e de interpolar
> POSIÇÃO. Quem vira quem sai da geometria; o traço percorre o **arco** entre as duas poses
> em vez da corda. Spec de origem: [`04_alem_do_blender.md` §2](04_alem_do_blender.md).
> **Estado: construída e gateada (2026-07-22), PENDENTE DE SMOKE** —
> `PH2D_FLIP_TWEEN_SMOKE=1`.

## §1 — O que estava errado, e o quanto

O W3 portou o `interpolate.cc` do Grease Pencil **literalmente**, e o próprio doc do GP
admite os dois limites:

| | o GP (e o nosso v1) | o preço |
|---|---|---|
| **correspondência** | curva *i* ↔ curva *i* (`interpolate.cc:244-315`) — puramente ordinal | redesenhar o quadro B começando por outra parte do corpo faz o braço interpolar contra a perna |
| **interpolação** | lerp de coordenadas | o traço corta pela CORDA: um braço que gira 120° **encolhe a 8,7%** no meio do caminho (medido, `the_control_the_plain_lerp_does_collapse_that_limb`) |

## §2 — O desenho

Três módulos irmãos na `ph2d-flip`, e **nenhum knob novo** — as duas metades são
*subsunção*, não substituição:

| módulo | responsabilidade |
|---|---|
| `tween_match.rs` | features → custo → atribuição ótima → recusa. Publica `TweenPlan` |
| `tween_spiral.rs` | a similaridade A→B (Umeyama) + o ponto fixo + o resíduo. Publica `StrokeMotion` |
| `tween.rs` | costura: monta o plano UMA vez, interpola, e leva os órfãos |

**Por que não há flag "usar o v2":**

- a **ordem de desenho continua contando**, como um TERMO do custo (peso 0.15) — quando
  tudo mais empata, o par ordinal ganha, e o v2 devolve o que o v1 devolvia;
- a translação pura cai em `StrokeMotion::Translate` e o ponto sai **byte-idêntico** ao
  lerp do v1 (gate `a_pure_translation_is_bit_identical_to_the_old_lerp`, 9 fatores,
  overshoot incluso).

### O custo do par

```text
custo(i,j) = ∞  se aberta/fechada incompatíveis
           | 0.40·|Δcentróide|/D + 0.25·|ΔL|/max(L) + 0.20·(1−|û·v̂|) + 0.15·|Δordem|/span
```

Três desvios deliberados da fórmula do `04 §2`:

1. **Features INTEGRADAS ao longo da polilinha**, nunca médias de vértice. Dois traços com
   a mesma forma e densidades diferentes têm de dar as mesmas features — a lição que o
   `ph2d-vec-blend` pagou (*"picar uma aresta reta em 20 pedaços mudava a correspondência"*).
2. **`Δeixo = 1 − |û·v̂|`**, não `Δângulo/(π/2)`: transcendental-free (são O(n²) pares) e
   **quadrático perto de zero** — a rotação pequena que um inbetween de fato tem quase não
   custa.
3. **Termo indisponível é OMITIDO da média ponderada, nunca contado como zero.** Uma forma
   isotrópica (círculo, ponto) não tem eixo, e contá-lo zero premiaria quem não trouxe
   informação.

⚠️ **`D` é o bbox dos PONTOS, não o dos centróides.** Com centróides, um desenho de um traço
só teria a diagonal igual ao próprio deslocamento, o termo saturaria em 1.0 para QUALQUER
movimento, e o caso mais simples que existe — um traço que anda — seria recusado.

### A espiral

```text
S(a) = σ·R(θ)·a + c          a similaridade A→B (Umeyama, forma fechada)
F    = (I − σR)⁻¹·c          o ponto FIXO
P(t) = F + σᵗ·R(θt)·(a − F)  + t·resíduo
```

- **A representação apaga o caso especial:** translação pura não tem ponto fixo (`I − σR`
  singular) ⇒ vira a variante `Translate(Vec2)`, e a fórmula do chamador vale para as duas.
- **A conta é em `f64`, a saída em `f32`:** `F ≈ c/det` fica longe quando o movimento é
  quase uma translação, e `(p − F) … + F` é cancelamento catastrófico.
- Orçamento HR-5: **um `atan2` por traço** no ajuste; **um `sincos` + um `powf` por traço e
  por inbetween** na avaliação. Os pontos só somam e multiplicam.

## §3 — Os números MEDIDOS (as réguas)

```bash
cargo test -p ph2d-flip --release the_cost_ruler    -- --ignored --nocapture
cargo test -p ph2d-flip --release the_outlier_ruler -- --ignored --nocapture
cargo test -p ph2d-flip --release the_spiral_ruler  -- --ignored --nocapture
```

### `PAIR_REJECT_COST = 0.38` — e a medição desmentiu a expectativa

| pares LEGÍTIMOS | custo | pares "espúrios" | custo |
|---|---|---|---|
| anda 20 | 0.0370 | braço × **cotoco** | **0.2774** |
| gira 45 | 0.0964 | braço × perna | 0.4261 |
| gira 90 | 0.2653 | braço × canto oposto | 0.5020 |
| gira 90 + encolhe 30% | **0.3352** | | |

**As colunas SE CRUZAM** — nenhum limiar separa `0.3352` de `0.2774`, porque o "cotoco"
**não é espúrio**: um braço que encolhe muito É esse par. O que a tabela separa é a zona
AMBÍGUA (0.27–0.34) do claramente-alheio (≥ 0.426), e `0.38` é o meio desse vão.

**A política que a escolha implementa: na dúvida, PAREAR.** Um par estranho é um inbetween
torto que o artista vê e corrige; um órfão é um traço que SOME no meio da animação.

### `OUTLIER_FACTOR = 2.0` — a recusa tem DUAS perguntas

Um limiar absoluto sozinho estava errado, e de dois jeitos:

- um quadrado **sozinho** que viaja 5× o próprio tamanho era orfanado;
- pior: numa **panorâmica** (a cena inteira se deslocando) *todos* os custos sobem juntos e
  o desenho INTEIRO seria orfanado de uma vez.

Então: recusa `⟺ cost > PAIR_REJECT_COST **e** cost > 2.0 × mediana`. Com **um** par só ele
É a própria mediana ⇒ pareia, **sem `if n == 1` em lugar nenhum**.

| regime | razão custo/mediana |
|---|---|
| panorâmica (`dx` 40 → 2000) | **1.000 exato** (o deslocamento é comum a todos) |
| um traço some e outro nasce | **246,6×** |

⚠️ E a escada do custo absoluto da panorâmica — `dx=40 → 0,119 · 140 → 0,255 · 400 → 0,341 ·
2000 → 0,388` — é o que mostra que o gate original (`dx=140`) **não continha o fenômeno**.

### `DET_MIN = 1e-6` — onde as duas fontes de erro se cruzam

| θ | det | erro da espiral `f32` | arco − corda |
|---|---|---|---|
| 1e-1 | 1.0e-2 | 4.43e-5 | 6.25e-1 |
| 1e-2 | 1.0e-4 | 3.78e-5 | 6.25e-3 |
| **1e-3** | **1.0e-6** | **7.29e-5** | **6.25e-5** ← elas se encontram |

Abaixo desse ponto a corda é pelo menos tão exata quanto a espiral, e é de graça.

### `AXIS_MIN_ANISOTROPY = 0.05`

círculo 48-gon **0.0000** · elipse 1.05:1 **0.0366** · 1.1:1 **0.0715** · 1.3:1 **0.1951** ·
2:1 **0.4872** · reta com tremor de mão **0.9992**.

### O custo do plano (`the_plan_cost_ruler`)

```text
  traços     10      50     100     200     400     800
  plano    0,004   0,021   0,060   0,223   0,820   3,226  ms
```

*"n é pequeno"* era uma afirmação sobre um número que ninguém tinha olhado. Medido: nesta
faixa quem domina é a matriz de custo (`O(n·m)`), não o solver `O(n³)` — e o plano é
construído **UMA vez por intervalo**, não por quadro nem por inbetween.

## §4 — Os três defeitos que a wave EXPÔS (todos pré-existentes)

| defeito | quem o escondia | o fix |
|---|---|---|
| **o auto-flip invertia ANEL** — num traço fechado as "pontas" são vizinhas (a costura), então a corda entre elas não descreve percurso | o lerp já saía torto de qualquer jeito | para anel a pergunta é o **sentido do percurso** (área com sinal) |
| **o auto-flip lia "girou muito" como "desenhado ao contrário"** — `da·db < 0` vale para QUALQUER giro além de 90° | idem | as três heurísticas eram proxies de **uma** pergunta: *qual dos dois jeitos de parear as pontas percorre menos caminho?* |
| **distância ao QUADRADO** decidia o oposto da distância real no braço de 120° | o quadrado só era usado como desempate de cordas quase paralelas, onde os dois concordam | somar distâncias REAIS |

Medições: `dot = −1,62` (o GP diz inverter) × direto `3,118` < cruzado `3,600` (a distância
diz que não) · por quadrado, cruzado `6,48` < direto `9,72` (inverte, errado).

## §5 — A UI (a dívida T3.7, fechada)

O motor sempre soube os dois; a barra é que não oferecia — o plano chamava isso de
*"carry-over de UI, não de motor"*.

- **chip `Ease`** (Linear / Ease In / Out / In-Out) nos MESMOS rótulos do menu de curvas da
  timeline. A **família é fixa em `Quad`** de propósito: o picker completo já existe na
  timeline, e onze famílias num chip de toolbar é a UI cara no lugar errado.
- **toggle `Fade`**: traço que existe em só uma das chaves entra/sai esmaecendo **e viaja
  com o vizinho** — antes o fade só existia para o lado de B, e por índice.

`FlipStrip::tween_options()` é a **porta única** entre a barra e o motor.

## §6 — O smoke

```bash
cd /home/enio/Documentos/Projetos/PH2D && \
  env PH2D_FLIP_TWEEN_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

A cena imprime `[tween-smoke] cena montada: 2 chaves (0 e 8) …` — **se essa linha não
aparecer, pare**. Aperte **Add** e folheie 0 → 2 → 4 → 6 → 8:

| conferir | o que estar errado significa |
|---|---|
| o BRAÇO não encolhe (percorre o arco) | a espiral não está rodando |
| o OMBRO fica parado | o ponto fixo saiu do lugar |
| nada atravessa a figura | a correspondência não está rodando (B foi desenhado na ordem trocada) |
| com **Fade**, o CHAPÉU some viajando com a cabeça | a advecção do órfão não está rodando |
| o chip **Ease** muda onde os inbetweens se acumulam | a barra não chega ao motor |

## §7 — Aberto (nomeado, não escondido)

1. **A FASE da costura em traço fechado.** Dois anéis de mesmo sentido cujo ponto 0 está em
   lugares diferentes do contorno são interpolados com a costura desalinhada, e a forma do
   meio fica torcida. A resposta é a correlação circular que o `ph2d-vec-blend` **já
   construiu** (`phase_only`) — wave própria, não um `if` a mais no auto-flip.
2. **O overlay de PARES + o re-par manual** (a lição CACANi: *nenhum matcher automático é
   confiável; a UI de correção é parte do algoritmo*). O `TweenPlan` já publica
   `pair_of_a`/`pair_of_b`/`cost_of_a` **para isto** — falta desenhar as linhas que ligam
   os traços casados entre as duas chaves e aceitar um par PINADO como restrição dura na
   atribuição (`assign` recebe a matriz; um pino é custo 0 na célula e `BLOCKED` no resto
   da linha e da coluna). **Ele não é UI morta hoje**: a wave não adiciona nada que dependa
   dele; ele é o próximo passo de qualidade.
3. **Rotação grande ainda torce** (o lerp do resíduo não-rígido). O estado da arte é
   Sederberg 1992 / Alexa 2000 — e a correspondência, que esta wave entrega, era o
   pré-requisito dos dois.
4. **A meia-volta EXATA é ambígua** e nenhuma ferramenta resolve: girar 180° para os dois
   lados dá a MESMA pose. O gate pina o que importa (mesmo ali o traço **não colapsa**);
   escolher o lado é trabalho do artista, e é a razão de o BetweenIT ter correção manual.
