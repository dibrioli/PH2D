# HANDOFF DE INTEGRAÇÃO — `line/physics` · a água, medida (2026-08-10)

**Status:** FECHADO 2026-08-10 · no `main` em `aafba0513` (o commit que trouxe este arquivo).

> **Jornada curta e de MEDIÇÃO, não de feature.** Ela fecha os dois itens
> mensuráveis da frente A da reabertura, e o resultado principal é um
> **negativo**: o defeito que a nota registava **não existe**. O entregável são
> as sondas que o provam, os gates que impedem a re-derivação do item falso, e
> as quatro notas que passaram a ser verdadeiras.

---

## §1 — O essencial

| fato | valor |
|---|---|
| branch | `line/physics` |
| HEAD | `0c931a20f` |
| base (`merge-base main HEAD`) | `76788440a` |
| commits | **3** (mais o `cb3854b6d` do doc de reabertura, já na branch) |
| `PROJECT_SCHEMA` | **70, INTOCADO** (`project.rs` não é tocado) |
| `physics_ecs_c9` | **`fb27f676…`, 117 corpos, debug ≡ release** — **byte-idêntico ao `main`** |
| registro `ph2d-physics-ecs` | **29, intocado** (nenhum componente novo) |
| registro `ph2d-ecs` + os **dois** espelhos | **intocados** |
| gizmo ids | **nenhum novo** (o último segue **973**, próximo livre **974**) |
| ids / consts / variants novos | **NENHUM** |
| contratos congelados | **nenhum encostado** |
| ADR | **nenhum** ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **ZERO** — nenhuma crate nova, nenhuma dep nova |
| cenas de smoke | **nenhuma nova** (a maior segue `104`, próxima livre `105`) |

**Superfície de colisão: praticamente vazia.** O diff toca `src/` num único
arquivo (`ph2d-platformer/src/kinematic.rs`) e **só em comentário** — conferido
por `git diff main -- … | grep -v '^[+-]\s*//'`, que sai vazio.

---

## §2 — O que a jornada fez

### (A) O `1,44 m` de bobeio na água **não era um defeito**

A W-KinFluid deixou nomeado como pendência que *"o player bobeia ~1,44 m nos
dois modos e a cápsula solta faz 0,81"*. A sonda nova
(`tests/measure_the_bobbing.rs`) atribuiu o excesso por **ablação da ENTRADA**
— knobs do `PlatformPlayer`, nunca instrumentação:

| ablação (mesma poça, regime = 2.ª metade de 6 s) | amplitude | vs controle |
|---|---|---|
| cápsula solta (CONTROLE) | `0,8097` | — |
| player default, largado de `+1,5` (no ar) | `1,4408` | `1,78×` |
| **os quatro multiplicadores de gravidade a `1`** | `0,8097` | **`1,00×`** (média também: `0,2330`) |
| sem perna · sem amortecimento · sem raio de chão · perna inteira fora | `1,4408` | `1,78×` — **inertes ao dígito** |
| **largado de `−0,5` (já submerso)** | `0,8326` | **`1,00×`** |
| largado de `−1,5` (submerso fundo) | `3,3214` | `0,99×` |

**A trava do fluido FUNCIONA.** O excesso inteiro é a modelagem do arco a agir
**no AR**, antes do primeiro contacto — que é onde ela é autorada para agir. O
personagem cruza a superfície a **`1,299×`** a velocidade do controle porque
`fall_gravity = 2.0`.

⚠️ **A minha previsão de `√2 = 1,414×` para essa razão estava ERRADA**, e a
medição corrigiu-a: falta na conta o **`peak_gravity = 0.5`**, que deixa o
COMEÇO da queda mais leve que o mundo. `1,299² = 1,687×` de energia fecha com os
`1,78×` de amplitude, com a folga a vir da saturação do empuxo (submerso ele é
constante, não linear no deslocamento).

⚠️ **E o que decide que não há defeito é a SEQUÊNCIA, não uma janela** — as duas
podem medir `1,44` no mesmo instante. Amplitude por janela de 3 s, em 30 s:

* controle `1,927 · 0,810 · 0,329 · 0,139 · 0,059 · 0,021 · 0,009 · 0,004 · 0,002 · 0,001`
* player&nbsp;&nbsp;`2,172 · 1,441 · 0,594 · 0,221 · 0,093 · 0,039 · 0,017 · 0,006 · 0,003 · 0,001`

Monotónicas e **convergentes**: transiente, e o meio come-o.

### (B) A paridade de arrasto entre modos **vale `1,149%`, medido**

O plano 07 e o comentário do `kinematic.rs` precificavam a divergência por
**analogia** — *"a mesma classe que a W-AreaDrag mediu em 1,25%"*. Uma analogia
com outra medição não é a medição desta. Medido em arrasto **puro** (sem empuxo:
com ele a oscilação é uma ordem de grandeza maior e afogaria o sinal):

| t | 1 s | 2 s | 3 s | 4 s |
|---|---|---|---|---|
| divergência relativa | **1,149%** | 0,257% | 0,056% | 0,018% |

A analogia estava certa em ordem de grandeza. **A forma é o que ela não dizia:**
a divergência **decai**, porque a velocidade terminal é `g/d` nos DOIS por
álgebra e ela vive só no transiente. ⚠️ **Corolário que decide o oráculo:** um
gate na velocidade terminal seria **verde por construção** e cego à divergência
inteira — o gate afirma o **PICO**.

---

## §3 — Gates e mutações

**3 gates novos**, todos em `crates/ph2d-physics-ecs/tests/player_in_water.rs`:

* `the_water_lock_contains_the_arc_shaping` — largado **submerso** (a única
  largada em que a trava arma no tique 1, logo a única que isola *a trava
  contém* de *a entrada foi mais rápida*).
* `the_bobbing_decays_it_does_not_pump` — a **sequência** de amplitudes.
* `the_drag_parity_between_modes_stays_within_its_measured_price` — o pico.

**3 mutações, todas sangram** (verde → RED → verde, com o `cp` de reversão):

| mutação | efeito |
|---|---|
| a trava não cala (`extra = scale − 1`) | **857 m** de amplitude aos 30 s — sai de quadro |
| a fração instantânea em vez da trava | **15,3 m**, a crescer |
| a lei cinemática ignora o arrasto do meio | sangra 3 gates |

⚠️ **E o achado sobre os gates que já existiam:** os **três** que já viviam
naquele arquivo ficam **VERDES** nas duas primeiras mutações — o de paridade
entre modos **por construção**, porque a trava é comum aos dois modos e uma
razão entre dois doentes não a vê. Era esse o buraco que esta jornada fecha.

**Auditoria (DIRETIVA §3), 2 lentes:**

```
LENTE:  correção — a conclusão "não há defeito" está certa?
CLAIM:  o excesso de bobeio é transiente de entrada, contido pela trava.
TRAÇO:  measure_the_bobbing.rs (ablação por knobs) → jump.rs:392-395 (o arm da
        trava: chão desarma, fluido arma) → jump.rs:613 (extra = 0 se waterborne)
        → measure_where_the_extra_energy_enters (submerso ⇒ 1,00× o controle).
ASSERÇÃO-VERMELHA: the_water_lock_contains_the_arc_shaping (mutação: 6,65 vs 0,83)
        + the_bobbing_decays_it_does_not_pump (mutação: 857 m).
NÃO-CHECADO-PELA-COMPILAÇÃO: o caso de VADEAR (footing + molhado no mesmo tique)
        não é coberto por gate novo — mas a lei o cobre por early-out, e o
        comentário do `jump.rs` já o nomeia. O número desta wave é de poça FUNDA.
LOC LIDAS: ~600 (jump.rs, lib.rs/Buoyed, kinematic.rs, cast.rs, bridge/player.rs)
```

```
LENTE:  disciplina de oráculo — os gates podem falhar pelo motivo que alegam?
CLAIM:  cada um dos 3 tem uma mutação que o sangra, e passa sobre o produto.
TRAÇO:  6/6 verdes no produto; as 3 mutações acima com os números ao lado.
ASSERÇÃO-VERMELHA: as próprias mutações (é o que esta lente audita).
NÃO-CHECADO-PELA-COMPILAÇÃO: o gate de decaimento PULA a 1.ª janela
        (`.skip(1)`), porque o transiente de entrada a faz menor que a 2.ª —
        exigir monotonia ali afirmaria algo que a física não promete. A
        consequência honesta: uma "bomba" que agisse SÓ nos 3 primeiros
        segundos e parasse não seria vista — e uma bomba que para não é uma.
LOC LIDAS: ~380 (os dois arquivos de teste, inteiros)
```

---

## §4 — Gate de fechamento (rodado, não auto-relatado)

| gate | resultado |
|---|---|
| `cargo test -p ph2d-physics-ecs -p ph2d-platformer --release` | **verde** |
| o mesmo em **DEBUG** | **146 suítes verdes, 0 falhas** |
| `cargo clippy -p … --all-targets` | **limpo** (um `print_literal` corrigido) |
| `rustfmt --check` nos 4 arquivos | **limpo** |
| `physics_ecs_c9` release **e** debug | `fb27f676…`, 117 — **idêntico ao `main`** |
| `ph2d-editor-core`: `arch_safe_clamp_only` | **verde** |
| `ph2d-editor-core`: `architecture_workspace_file_loc_cap` | **verde** |
| `ph2d-host-desktop`: `file_loc_caps` | **verde** |

Os três últimos são o **gotcha #2** da reabertura (gates que a varredura
impactada não alcança) — rodados explicitamente.

---

## §5 — ⚠️ O QUE O INTEGRADOR TEM DE CORRIGIR NO `CLAUDE.md`

A §5 do `CLAUDE.md`, na entrada de **2026-08-10** desta linha, termina com:

> *"e o player **bobeia ~1,44 m numa poça nos DOIS modos** (1,4357 × 1,4394), que
> é **anterior** a esta jornada"*

listado entre os **abertos**. **Essa frase passou a ser falsa** — o item foi
medido e dissolvido. Ela **não foi editada aqui de propósito**: o `CLAUDE.md` é
arquivo compartilhado por todas as linhas e a §5 é escrita na integração.

**Substituição sugerida** (o item sai dos abertos):

> ⚠️ **E o *"bobeio de ~1,44 m na poça"* que esta entrada listava como aberto
> FECHOU POR MEDIÇÃO no mesmo dia — não era defeito:** com os quatro
> multiplicadores de gravidade a `1` a amplitude é **`0,8097` = o controle ao
> quarto decimal**, e largado **já submerso** o player é `1,00×` o controle ⇒ a
> trava do fluido **contém**, e o excesso inteiro é a modelagem do arco a agir
> **no AR** antes do primeiro contacto, que é onde ela é autorada para agir (o
> personagem cruza a superfície a **`1,299×`** a velocidade do controle porque
> `fall_gravity = 2.0`; ⚠️ a previsão de `√2` estava errada — falta o
> `peak_gravity = 0.5`, que alivia o começo da queda). ⚠️ **E o que decide não é
> uma janela, é a SEQUÊNCIA:** em 30 s as amplitudes por janela de 3 s vão de
> `2,172` a `0,001` e **convergem com o controle** ⇒ transiente, não bomba. Mais
> a paridade de arrasto, que era precificada por **analogia** (*"a classe dos
> 1,25% da W-AreaDrag"*) e agora tem o número **desta** paridade: **`1,149%` no
> pico, a decair** — e um gate na velocidade terminal seria **verde por
> construção**, porque ela é `g/d` nos dois por álgebra. 3 gates, 3 mutações,
> todas sangram; ⚠️ **os três gates que já viviam ali ficavam VERDES** nas duas
> primeiras (857 m · 15,3 m), o de paridade entre modos **por construção**.

---

## §6 — O que smoke-testar

⚠️ **Nada de novo, e isto não é uma omissão.** A jornada **não muda um byte de
comportamento**: o único toque em `src/` é comentário, e o `physics_ecs_c9` sai
byte-idêntico ao `main` em debug e release. O que se pede é o **CONTROLE**:

* **`PH2D_PHYSICS_SMOKE=104`** — a água e o modo cinemático — tem de ficar
  **exactamente como o Enio aprovou em 09/08**.
* **`PH2D_PHYSICS_SMOKE=100`** — o dinâmico na água — o mesmo.

Se qualquer um dos dois mudar, a premissa desta jornada (*ninguém mexeu na lei*)
está errada e o resto não vale.

---

## §7 — O que fica ABERTO, com o preço ao lado

* **Decisão de PRODUTO, não dívida:** um personagem que cai com
  `fall_gravity = 2.0` entra na água com o momento que essa queda lhe deu. Querer
  que ele entre como uma pedra é mexer no knob que governa o platformer inteiro
  — não é um conserto local, e os números acima são o que se estaria a trocar.
* A frente A da reabertura fica **sem itens mensuráveis**: o bobeio dissolveu, a
  paridade de arrasto tem número e gate, e `form_drag` / a FORÇA de zona têm
  motivo escrito (⛔ §3 da reabertura).
* O **vadear** (footing + molhado no mesmo tique) não ganhou gate novo — a lei o
  cobre por early-out e o `jump.rs` já o nomeia, mas nenhum número desta jornada
  fala dele: toda medição aqui é de poça **FUNDA**.
