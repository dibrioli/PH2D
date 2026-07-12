# 41 — Rig: `rig.ik_2bone` (lei dos cossenos) + `rig.fabrik` — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **M4 — Rig** (solvers de alcance)
**Status:** implementado, testado (5 mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum

---

## 1. Dois solvers, duas máquinas diferentes pra mesma pergunta

| | `rig.ik_2bone` | `rig.fabrik` |
|---|---|---|
| Cadeia | **exatamente 3 juntas** (raiz · cotovelo · mão) | **qualquer comprimento** (tentáculo, cauda, espinha) |
| Método | **fechado** (lei dos cossenos) | **iterativo** (Aristidou & Lasenby, 2011) |
| Referência | Maya `ikRPsolver` · Unity `TwoBoneIKConstraint` · Unreal `Two Bone IK` | o padrão da indústria pra N ossos |
| Custo | 1 passe, exato | ~10 passes, converge a 1e-4 |

**Por que FABRIK e não CCD/Jacobiano:** os outros trabalham em espaço de **ângulos** e pagam por isso — o CCD
curva a ponta primeiro e entrega um **anzol**; os Jacobianos invertem matriz por iteração e desmontam nas
singularidades. FABRIK trabalha em espaço de **posições** e não faz álgebra linear nenhuma: percorre a cadeia
duas vezes por iteração, largando cada junta na linha até o vizinho, na distância certa. Os dois passes
preservam os comprimentos exatamente; sobra convergir a âncora da raiz e o alvo da ponta.

## 2. HR-5: a lei dos cossenos **sem `acos`** — e sem aproximar o solver

O livro-texto escreve `a = acos((l1² + d² − l2²) / (2·l1·d))`. `acos` é proibido (HR-5). Mas **o ângulo nunca
é o que se quer** — só a **direção** do primeiro osso. Então fique em vetores:

```
cos a = (l1² + d² − l2²) / (2·l1·d)      clampado a [−1, 1]
sin a = √(1 − cos²a)                      ← só um sqrt, que o HR-5 permite
osso1 = rotate(unit(alvo − raiz), ±a)     ← rotação 2×2 por (cos a, sin a)
```

Girar um vetor unitário por um ângulo cujo cosseno e seno já estão na mão é multiply-add puro. **O solve é
EXATO** — nenhuma aproximação polinomial de `acos` em lugar nenhum. (FABRIK é transcendental-free por
natureza: só `sqrt` nas normalizações.)

O `clamp` não é paranoia: na extensão máxima o quociente é exatamente 1 em aritmética real e `1 + 1e-7` nesta,
e o `sqrt` de um negativo devolveria um **membro NaN**. *Mutante provado.*

## 3. A decisão que sustenta os dois: **um solver escreve uma POSE, nunca posições**

Um solver de IK produz naturalmente **posições** (onde o cotovelo foi parar). Escrevê-las direto em `P`
funcionaria **exatamente uma vez** — e quebraria todo o resto: a verdade de um esqueleto são os **ângulos**
(doc 40), então uma cadeia cujo `P` discorda do `rot` está **rasgada**. O próximo `rig.fk`, o próximo solver,
o skinning — todos leem os ângulos e estalariam o membro de volta.

Então o solver **propõe posições, converte pra ângulos LOCAIS (`pose.rs`) e deixa o `fk::resolve` desenhar**.
Três coisas caem de graça:

- **os ossos ficam exatamente rígidos** (o FK os constrói do `len`, não da aritmética do solver);
- **as juntas abaixo do trecho resolvido seguem junto** (uma mão puxada pelo IK carrega os dedos);
- **os solvers compõem** — IK, depois outro IK, depois uma onda, em qualquer ordem.

O preço é uma ida-e-volta pelo `atan2`/`cos-sin` aproximados: o efetuador cai a ~0,1 % do comprimento da
cadeia do alvo — bem abaixo de um pixel. **A exatidão da POSE vale mais que a exatidão de um ponto.**

> **É a lição desta fatia.** Quando mutei o código pro atalho tentador (`P` direto), **16 dos 17 testes
> continuaram VERDES** — inclusive *"a mão alcança o alvo"* e *"os ossos não esticam"*. Os pontos caem no
> lugar certo; o esqueleto é que fica mentindo. Só a guarda `a_downstream_fk_finds_nothing_to_fix` pega.

## 4. Correção que a fatia trouxe: os ossos agora **realmente** não esticam

O teste de cadeia acusou osso de `1.5013` em vez de `1.5`. Não era o solver: o par `(cos, sin)` **parabólico**
(HR-5) tem norma ~0,1 % fora de 1, então **todo** osso saía ~0,1 % comprido ou curto **dependendo do ângulo** —
o alcance de um membro mudava silenciosamente conforme ele se movia. Minha invariante do doc 40 ("os ossos
nunca esticam") era só **quase** verdadeira.

**Fix:** o `fk::resolve` **normaliza a direção** antes de andar por ela (um `sqrt`, permitido). Agora é
literalmente verdadeira, e a tolerância do teste apertou **10×** (1e-3 → 1e-4). O resíduo de ~0,05° de ângulo é
invisível; um osso que estica não é.

## 5. A demo — o alvo é VISÍVEL

```text
 alvo:      grid(1) ─> move(2.4, 0) ─> orbit(90°/s) ─┬──────────────┐
 ESQUERDA:  skeleton(3)  ─> ik_2bone <───────────────┤              │
 DIREITA:   skeleton(10) ─> fabrik   <───────────────┘              │
 pontos-alvo:                    scale(gordo) ─> move(∓7) ─> output ┘
```

Os dois membros perseguem **o mesmo ponto orbitando**, e o alvo é **desenhado** ao lado de cada um — o que eles
perseguem não é questão de fé. **Quatro sinks, sem nó de merge**: todo `motion.output` do documento lowera no
mesmo buffer. (O `motion.combine` também funcionaria — mas ele **zera as colunas que um input não tem**, então
fundir um alvo tingido num esqueleto sem `tint` pintaria o membro inteiro de **preto transparente**. Sinks
compõem sem esse pedágio.)

## 6. As guardas — 5 mutantes provados VERMELHOS

| # | Mutante | Guarda |
|---|---|---|
| 1 | **`P` escrito direto** (o atalho) | `a_downstream_fk_finds_nothing_to_fix` — **e MAIS NADA** (16/17 seguiam verdes) |
| 2 | lei dos cossenos **sem clamp** | `an_unreachable_goal_extends_the_limb_instead_of_stretching_it` (membro NaN) |
| 3 | FABRIK **sem re-ancorar a raiz** no passe forward | os 3 testes do FABRIK (a cadeia vai embora em vez de convergir) |
| 4 | **fio do alvo não ligado** na demo | `both_limbs_land_their_tip_on_the_orbiting_goal` → *"the hand missed by 3.84"* |
| 5 | (correção §4) FK sem normalizar a direção | `every_bone_keeps_its_length_whatever_the_pose` a 1e-4 |

**O oráculo do teste de cadeia é o OUTRO SINK.** "A mão está no alvo" é literalmente "estes dois pontos
desenhados coincidem" — nada de recomputar a matemática do solver pra conferir a matemática do solver
([[feedback_oracle_must_model_appearance_not_implementation]]).

## 6-bis. O smoke do Enio pegou DUAS coisas (2026-07-12)

> *"o cotovelo dobra — o cotovelo fica sempre no mesmo ângulo. é isso mesmo?"*

**(a) A demo era uma mentira geométrica.** O alvo orbitava em raio **constante** em volta da **própria raiz do
braço** → o triângulo (raiz, cotovelo, mão) tinha os **três lados fixos** (1.5, 1.5, 2.4) — e um triângulo de
lados fixos tem **ângulos fixos**. O braço girava **rígido**. O cotovelo estava dobrado e nunca *dobrava*.

E a minha guarda afirmava exatamente a coisa fraca: que o cotovelo ficava **fora** da linha raiz→mão (verdade,
e constante). *Provar que está dobrado não é provar que dobra.* Agora o alvo **respira** (oscillator no X,
upstream do orbit, onde +X **é** a direção radial → o alcance varre 2.0 ± 0.9) e a guarda mede o **RANGE** da
flexão, mais uma asserção de que a distância do alvo realmente varia (senão a primeira é vácua).

**(b) E isso destapou um BUG REAL no `rig.fabrik`** — a **degenerescência colinear**, que só um teste de
corrente inteira encontra. Com raiz, juntas e alvo **na mesma reta**, o FABRIK **não tem gradiente**: o passe
backward arrasta a cadeia por aquela reta e o forward a empurra de volta — uma cadeia reta **nunca dobra** pra
alcançar um alvo mais perto que sua extensão total. Ela **empaca esticada**, errando por **exatamente a folga**
(o miss de `0.33` = `3.06 − 2.73`).

**Não é exótico, é o DEFAULT aqui:** o nó é `Pure` (sem `pre`, por design), então ele parte da **pose de
repouso** do `rig.skeleton` todo frame — e ela é **exatamente reta**. Qualquer alvo alinhado com o membro cai
na degenerescência. (O paper nunca esbarra: as cadeias dele carregam a pose do frame **anterior**, que nunca é
exatamente reta.) **Fix:** `break_collinearity` — arqueia as juntas internas um fio de cabelo fora da reta,
determinístico, e **só** quando a cadeia é de fato colinear **e** o alvo é de fato mais perto que a extensão
total (alvo em extensão máxima quer a resposta reta, e a recebe). Guarda:
`a_straight_chain_still_folds_to_a_goal_on_its_own_axis`.

**Trade-off documentado:** um solver stateless não tem coerência temporal — ao cruzar a reta, o lado da dobra
pode inverter. Um `rig.fabrik` sequencial (com `pre`, semeado pela pose anterior) resolveria isso; fica pra
quando alguém precisar.

## 7. Superfície nova (pro integrador)

| Item | Valor |
|---|---|
| Crates novas | `ph2d-node-rig-ik-2bone` · `ph2d-node-rig-fabrik` |
| Node ids | `rig.ik_2bone` (params `root`, `flip`) · `rig.fabrik` (param `iterations`) — ambos Transform |
| Porta nova | `target` (2º input, `INST_VEC2`) — o **primeiro elemento** é o alvo; **desconectado = no-op**, jamais "a origem" |
| Leaf alterado | `fk.rs` (normalização) — **os 4 crates `rig.*` carregam a MESMA cópia** |
| Codegen | registry-init regenerado — **77** crates-nó |

## 8. Aberto (M4 Rig)

`rig.rubber_hose` (limbo de borracha cartunesco, ref. Battle Axe RubberHose do AE) · `rig.skin_deformer`
(LBS — vai ler o `wrot` que o FK já publica) · e, se der vontade, `pole` como porta em vez de bit (`flip`).
