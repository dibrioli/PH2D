# HANDOFF — `line/physics` → `main` (2026-08-05) · **W11b, o ajuste diferido**

**A linha está FECHADA e PARADA.** 2 commits, 12 arquivos.
Nada integrado, nada pushado.

> Este handoff é pequeno de propósito: a jornada de 04/08 já integrou, e o que
> segue é **o ajuste que o Enio diferiu** (*"no setup como está o player sobe
> sozinho bem devagar; mas faremos os ajustes amanhã"*) mais um degrau de escada
> que faltava.

---

## §1 — O que entra

| commit | o quê |
|---|---|
| `2b4f0df0d` | **o degrau v54 que faltava na escada do `PROJECT_SCHEMA`** |
| `865aa410f` | **W11b** — o que cancela a gravidade passa a ser integrado como ela |

---

## §2 — Os números que se contam: **NENHUM se move**

| número | veredito |
|---|---|
| `PROJECT_SCHEMA` | **INTOCADO** (55) — só um doc-comment foi acrescentado |
| registro `ph2d-physics-ecs` | **INTOCADO** (28) — nenhum componente novo |
| registro `ph2d-ecs` (as 3 casas) | **INTOCADO** |
| gizmo ids | **nenhum novo** — o próximo livre segue **974** |
| ADR (máx) | **nenhum ADR** ⇒ fora de toda disputa de número |
| contrato congelado | intacto |
| `Cargo.toml` | **zero** — nenhuma dep, nenhuma crate |
| **`physics_ecs_c9`** | ⚠️ **`b3dbe792…` → `2278035e…`**, 108 corpos, debug ≡ release |

⚠️ **O `c9` TEM de se mover, e isso é a afirmação da wave:** a altura de repouso
do player mudou em toda cena que tenha um. Ele **não é pinado em literal** (o CI
compara os 3 OSes entre si), então o que o integrador confere é *debug ≡ release*
e que os três OSes concordem.

---

## §3 — O commit pequeno: o buraco na escada

A escada de doc-comments do `project.rs` ia v52 → v53 → **v55**, sem entrada para
o v54 (o `PhysicsJoint.custom` do W-JointCustom). O gate ao lado
(`project_schema_tests.rs`) documenta o degrau por inteiro, então o fato nunca se
perdeu — o que ficou incompleto foi **o documento que o próximo a bumpar lê para
contar**.

⚠️ **A origem torna a lição mais estreita:** o commit `75dddfab3` de ontem
chama-se *"a escada do schema para de mentir"* e consertou a escada da **§5 do
`CLAUDE.md`**, que é o ESPELHO. A do `project.rs` é a FONTE, e ficou com o
buraco. *Corrigir o espelho e deixar a fonte é como o próximo bump nasce
mal-numerado* — foi esta mesma linha que escreveu isso em 02/08.

---

## §4 — A W11b, em uma frase e uma tabela

**O que cancela a gravidade passa a ser integrado como ela.** Antes, o motor
inteiro do player era pago como **um impulso no topo do tique**, enquanto o
`rapier` integra a gravidade **ao longo** dele, sub-passo a sub-passo — e
`drag`, `effector`, `blast` e as polias já eram todos pagos dentro daquele laço,
com o comentário que diz por quê. O motor do player era o único de fora.

| `spring_damping` | deriva (30°, 10 s) | erro de repouso | peso transmitido |
|---|---|---|---|
| 0,25 | 0,2476 → **0,0498** | 2,87 → **0,57 mm** | 88% → **98%** |
| **0,50** (o que shipa) | 0,1644 → **0,0331** | 5,75 → **1,15 mm** | 77% → **95%** |
| 0,75 | 0,0819 → **0,0165** | 8,62 → **1,72 mm** | 65% → **93%** |
| 1,00 (o teto) | **0,0000** | 11,50 → **2,30 mm** | 53% → **91%** |

**Cinco vezes menos deriva em TODO valor do knob**, e o peso de volta.
⚠️ **E o quique do pouso não se moveu** (196 → 199 mm a 0,25; 20 → 24 a 0,50) —
a correção é de **integração**, não de lei, e é essa coluna que o prova: o que o
Enio aprovou no smoke da W6/W9 continua igual.

O mecanismo, as duas hipóteses que morreram medindo e o que sobra estão no
[`BUGS_physics.md`](BUGS_physics.md) **§7b**; o resumo de uma linha está na W11b
do [`00_plano_waves.md`](00_plano_waves.md).

---

## §5 — ⚠️ O que o Enio tem de decidir no smoke (e a medição não decide)

**O default fica em `spring_damping = 0,50`, e isso é uma decisão de FEEL.**

O `1,00` sempre deu deriva **exactamente zero**; o que o impedia de ser o default
era custar **metade do peso** do personagem. Hoje custa **9%**. Mas ele também
zera o **quique do pouso**, e um pouso sem quique é outra sensação — por isso o
número não foi mexido sem um olho em cima.

⇒ **No smoke:** se os 3,3 cm em 10 s ainda incomodarem, o **Spring Damping no
painel** é o knob, e ele agora oferece a troca de verdade em vez de uma
armadilha.

---

## §6 — Ordem

1. `git rebase main` (ou merge). **Não deve haver conflito**: os 7 arquivos de
   código são todos do módulo do player, e o único arquivo compartilhado é o
   `project.rs` — onde a mudança é **só um doc-comment inserido**, sem tocar o
   literal do `PROJECT_SCHEMA`.
2. Rodar o gate da árvore combinada **em DEBUG E RELEASE** (esta linha tem
   precedente registado de vermelho só-em-debug).
3. Recomputar o `physics_ecs_c9` **depois** do rebase e conferir debug ≡ release.
   ⚠️ Ele **deve** dar `2278035e…`; se der `b3dbe792…`, a wave não fundiu.

---

## §7 — Estado de smoke: **APROVADO**

> **Enio, 2026-08-05:** *"Quase perfeito na rampa! Sobe muitíssimo devagar, quase
> imperceptível. **Jangadas Smoke OK.**"*

As duas metades confirmadas no produto: a deriva quase fechou e **o peso voltou**
— a jangada era exactamente a cena que media a segunda coluna da tabela do §4, e
ela passou.

O gesto que as julga, para quem repetir:

```
env PH2D_PHYSICS_SMOKE=81 cargo run -p ph2d-host-desktop --release   # a rampa de 30°
env PH2D_PHYSICS_SMOKE=88 cargo run -p ph2d-host-desktop --release   # o par 40°/50°
env PH2D_PHYSICS_SMOKE=85 cargo run -p ph2d-host-desktop --release   # a jangada (o PESO)
```

⚠️ **O gesto é NÃO FAZER NADA:** marque **Physics** no transporte, dê Play, leve o
personagem à rampa e **solte as teclas**.

⚠️ **E a cena `=85` é a outra metade**, a que a tabela do §4 diz ter melhorado
sem ninguém a pedir: a jangada tem de afundar **mais** que antes, porque o
personagem voltou a pesar 95% em vez de 77%.

---

## §8 — Aberto, com o preço ao lado

- **O resíduo de `0,0331 m`/10 s no default** — exactamente linear em `(1 − d)` e
  zero no teto. É o mesmo mecanismo um degrau abaixo: o termo da **MOLA**
  continua agrupado no topo do tique, **de propósito**. ⛔ Fatiá-lo foi
  construído e medido: corta a deriva 4× e faz um `spring_damping = 0` **LARGAR o
  personagem** (erro de repouso 0,03 → **1430 mm**), com um piso de amortecimento
  que **anda com a rigidez** (0,01 a `k = 100`, 0,25 a `k = 3200`) — um knob cujo
  mínimo é função de outro knob. Fechá-lo pede uma mola **semi-implícita**
  (re-amostrar o raio por sub-passo), que é outra wave e cobra um raio por
  sub-passo.
- **O "pulinho"** do relato de 04/08 segue **sem repro** — a sonda `measure_idle`
  já traz o instrumento se ele voltar.
- **`float_height` abaixo do mínimo geométrico** faz a cápsula ENCOSTAR e o
  solver de contato assumir — pré-existente, documentado no `min_float_height`.
- O resto do aberto por wave vive nos handoffs que o MESTRE de 04/08 lista.
