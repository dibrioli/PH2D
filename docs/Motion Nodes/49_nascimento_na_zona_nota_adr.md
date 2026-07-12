# 49 — **Nascimento** na zona (`sim.spawn`) — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **O4** (parte 2)
**Status:** implementado, testado (2 mutantes provados — um deles um bug REAL de f64), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** `EvalCtx::dt()` (**aditivo**)

---

## 1. O buraco que a zona deixou (e que o doc 48 prometeu sem poder cumprir)

A zona deu **vida** (`sim.step` sobre estado vivo) e **morte** (`motion.cull`, que dentro dela deixa de ser filtro e
vira kill). **Não deu nascimento** — e simulação que não nasce é população que só encolhe: a chuva do doc 48 afinava
até acabar.

O doc 48 afirmou que *"`motion.combine` vira NASCIMENTO"*. **Era falso na prática**: o `motion.emitter` é
**stateless** (o conjunto vivo é função pura do playhead), então fundi-lo no estado a cada tick funde **as mesmas
partículas de novo**, para sempre. Faltava um nó que respondesse uma pergunta bem menor: **quem nasceu NESTE tick?**

## 2. Um nó, uma pergunta — e o `combine` cumpre o que foi prometido

`sim.spawn` emite **só os recém-nascidos deste tick** (quase sempre nenhum, às vezes um, às vezes três). Ele **não
funde nada**: quem funde é o **`motion.combine`**. Assim cada nó faz uma coisa, e o nascimento fica **componível**:

```text
  zone ⊙─→ combine(in0) ─→ force.wind ─→ sim.step ─→ falloff ─→ cull ─→ state
  grid ──→ sim.spawn ────→ combine(in1)
```

O recém-nascido **herda TODA a coluna da linha do template** de onde nasceu (`P`, `vel`, `size`, `tint`…) — então
você **mira, colore e dimensiona o nascimento com os nós de sempre**, a montante do spawn. Um nó de nascimento que
inventasse vocabulário próprio seria uma segunda cópia, pior, da biblioteca.

## 3. Identidade = ordinal de nascimento (por isso o scrub reproduz)

O k-ésimo elemento que nasce recebe `id = k`, e `k` vem do **RELÓGIO** (`floor(rate·t)`), não de um contador que o
nó guarda. É função pura do playhead: rebobine, re-cozinhe, e **as mesmas partículas voltam** com os mesmos ids,
os mesmos slots e o mesmo jitter (`hash(seed, id, lane)` — a aleatoriedade stateless do emitter, Jarzynski & Olano
2020; HR-5).

Um contador numa coluna de estado seria o óbvio — e faria o id depender da **história** do cook, não do relógio:
**um scrub renumeraria o mundo**.

## 4. Os dois mutantes

### 4.1 Taxa fracionária arredondada por tick = **nada nasce, nunca**

7 nascimentos/s a 60 fps são **0,116 por tick**. `round(rate·dt)` por tick arredonda isso para **zero em todo
tick** — e o emissor nunca emite. Os nascimentos vêm da **diferença de dois pisos** (`births_upto(t) −
births_upto(t−dt)`), então o resto **acumula**: em um segundo saíram exatamente 7. Guarda:
`a_fractional_rate_is_exact_over_time_and_does_not_round_itself_away`.

### 4.2 O ULP que fazia a **mesma partícula nascer duas vezes** (bug real, achado pelo teste)

O tick anterior é reconstruído como `t − dt`, e **em f64 isso é o playhead anterior a menos de 1 ulp, não
exatamente**. Sem folga, o `floor` cai **em cima** da fronteira de nascimento e **recua**: o tick recalcula
"nascidos antes de mim" com **um a menos** do que o tick passado emitiu — e **o mesmo id nasce de novo**. O teste
contou **97 nascimentos onde 90 eram devidos**. Fix: `BIRTH_EPS = 1e-6` **nascimentos** de folga (muito acima do
ruído de f64, muito abaixo de uma partícula), aplicado no mesmo lugar pelos dois lados do tick.

## 5. `EvalCtx::dt()` — o relógio que o motor sempre soube e nunca disse

O `sim.spawn` **não tem estado**, logo não tem coluna de relógio própria para subtrair. O motor **sempre soube** o
delta (ele fecha cada tick com um playhead), e nunca o expôs — então os nós que precisavam **o inventaram**: o
`motion.integrate` carrega uma coluna `sim_t` no próprio estado. Isso **fica** (relógio por-elemento é exatamente
certo para elementos nascidos em tempos diferentes), mas um nó **sem estado** não tinha como perguntar.

`EvalCtx::dt()` = passo do relógio **RAIZ**; `0.0` no primeiro tick e **dentro de um time scope** (a lane é
reescrita, então delta entre ticks não existe lá — e nó que precisa de `dt` para guardar estado é sequencial, o que
um scope já recusa). Vai no `CookCheckpoint`, então **um tick replayado toma exatamente o `dt` que tomou da
primeira vez**.

## 6. Demo (doc de boot): a chuva virou **NEVE**

**O `init` da zona ficou DESLIGADO de propósito**: a população começa em **nada** e é **inteiramente nascida** —
então todo floco tem identidade única desde o nascimento, e o triângulo inteiro está na tela: **nasce** (spawn +
combine) · **vive** (wind + step: acelera, porque a velocidade mora no estado) · **morre** (falloff + cull: o que
sai do círculo não volta). **Nascimento e morte se equilibram: a neve atinge REGIME e fica.**

Guarda de produto: `the_snow_is_born_accelerates_and_settles_into_a_steady_state` — e ela mede a aceleração num
**floco identificado (`id = 0`)**, não numa média: a população muda por baixo de qualquer média (a armadilha de
sobrevivência em que este mesmo teste caiu no doc 48, quando media a gota **mais baixa** — que é justamente a que o
disco está prestes a matar, então o mínimo **subia**).

## 7. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| foundational | **`EvalCtx::dt()`** + `Cook.prev_playhead` (aditivo; entra no `CookCheckpoint`) |
| crate nova | **`ph2d-node-sim-spawn`** (`sim.spawn`) → **83 crates-nó** |
| shell | a 5ª cena do doc de boot virou a NEVE (`init` desligado; +`sim.spawn`, +`motion.combine`) |

## 8. A lição

**Eu escrevi no doc 48 que "combine vira nascimento" e não era verdade** — era verdade sobre *fundir streams*, e
falsa sobre *nascer*, porque não havia quem calculasse os recém-nascidos. Uma afirmação de doc que ninguém pode
executar é uma afirmação que envelhece como mentira. Ou vira teste, ou vira nó, ou sai do doc.
