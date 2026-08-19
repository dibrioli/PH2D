# HANDOFF — `line/physics` → `main` (2026-08-05) · **W11b, o ajuste diferido**

**Status:** FECHADO 2026-08-05 · no `main` em `857772eec` (o commit que trouxe este arquivo).

> ⚠️ **SUPERSEDIDO por [`HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-05.md`](HANDOFF_INTEGRACAO_line_physics_MESTRE_2026-08-05.md)**
> — a jornada continuou com a **W12** (descer da plataforma) e a **W13** (as
> paredes), e o título deste arquivo deixou de descrever o que a linha entrega.
> O MESTRE é quem tem a lista de commits, os números que se contam e a ordem de
> integração; **o detalhe de mecanismo da W11b/W11c continua aqui** e não foi
> copiado para lá.

**A W11b/W11c estão FECHADAS e SMOKADAS** (Enio, 2026-08-05: *"Smoke OK"*).
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
| `43852a422` | este handoff |
| `f3cfb9b96` | o veredito do 1º smoke + as duas tentativas medidas e mortas |
| `d591e01b5` | **W11c** — o default sobe ao TETO (o 2º smoke; §5) |

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
| **`physics_ecs_c9`** | ⚠️ **`b3dbe792…` → `74d4ea5d…`**, 108 corpos, debug ≡ release |

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
| 0,50 | 0,1644 → **0,0331** | 5,75 → **1,15 mm** | 77% → **95%** |
| 0,75 | 0,0819 → **0,0165** | 8,62 → **1,72 mm** | 65% → **93%** |
| **1,00** (o teto, e **o que shipa** desde a W11c) | **0,0000** | 11,50 → **2,30 mm** | 53% → **91%** |

**Cinco vezes menos deriva em TODO valor do knob**, e o peso de volta.
⚠️ **E o quique do pouso não se moveu** (196 → 199 mm a 0,25; 20 → 24 a 0,50) —
a correção é de **integração**, não de lei, e é essa coluna que o prova: o que o
Enio aprovou no smoke da W6/W9 continua igual.

O mecanismo, as duas hipóteses que morreram medindo e o que sobra estão no
[`BUGS_physics.md`](../BUGS_physics.md) **§7b**; o resumo de uma linha está na W11b
do [`00_plano_waves.md`](../00_plano_waves.md).

---

## §5 — W11c: a decisão de FEEL foi TOMADA, e o default é o teto

Este handoff dizia, na versão anterior: *"o default fica em `0,50`, e quem decide
é o smoke"*. **O smoke decidiu** — o Enio reportou a subida uma segunda vez
(*"continua subindo um pouco mais rápido que antes, ainda bem discreto"*) e
**nunca reportou o quique**. Entre um personagem que anda sozinho e um pouso sem
24 mm de quique, o defeito é o primeiro.

**`RideConfig::STARTING_POINT.spring_damping` = `MAX_DAMPING` (1,00).**

⚠️ **E a varredura que a frase dele motivou MATOU uma suspeita minha** — eu
achava que o `0,0000` do teto fosse um cruzamento calibrado a 30°, porque os dois
termos do resíduo escalam diferente com o ângulo. Medido, ele é **zero exacto de
20° a 45°**, e as outras colunas colapsam numa lei que reproduz os 24 valores ao
quarto decimal:

```text
  deriva(10 s) = 0,153 · sen θ · (1 − d)   metros
```

Daí também a leitura do *"mais rápido"*: a deriva **cresce com a rampa**, e a 40°
(a cena `=88`) ela é **1,29×** a de 30°. Não havia regressão a procurar.

⚠️ **O quique de verdade vive no FUNDO do knob:** 199 mm em `0,25`, 24 mm em
`0,50`, e **já zero em `0,75`** — quem o quiser de volta baixa o **Spring
Damping** no painel, que agora oferece a troca de verdade em vez de uma
armadilha. Mecanismo, tabela e a inversão da conta: [`BUGS_physics.md`](../BUGS_physics.md) **§7c**.

---

## §6 — Ordem

1. `git rebase main` (ou merge). **Não deve haver conflito**: os 9 arquivos de
   código são todos do módulo do player, e o único arquivo compartilhado é o
   `project.rs` — onde a mudança é **só um doc-comment inserido**, sem tocar o
   literal do `PROJECT_SCHEMA`.
2. Rodar o gate da árvore combinada **em DEBUG E RELEASE** (esta linha tem
   precedente registado de vermelho só-em-debug).
3. Recomputar o `physics_ecs_c9` **depois** do rebase e conferir debug ≡ release.
   ⚠️ Ele **deve** dar `74d4ea5d…`; se der `b3dbe792…`, a wave não fundiu, e se
   der `2278035e…` a **W11c** (o default no teto) se perdeu no caminho.

---

## §7 — Estado de smoke: **APROVADO (W11b) · PENDENTE (W11c)**

> **Enio, 2026-08-05:** *"Quase perfeito na rampa! Sobe muitíssimo devagar, quase
> imperceptível. **Jangadas Smoke OK.**"* — e, no smoke seguinte, *"continua
> subindo um pouco mais rápido que antes, ainda bem discreto"*.

As duas metades da **W11b** confirmadas no produto: a deriva quase fechou e **o
peso voltou** — a jangada era exactamente a cena que media a segunda coluna da
tabela do §4, e ela passou.

⚠️ **A W11c (o default no teto) ainda NÃO foi smokada** — ela é a resposta ao
segundo relato, e o que ela promete é a deriva em **zero exacto** na `=81` e na
`=88`, ao preço de o pouso perder 24 mm de quique. **É essa troca que o smoke
julga.**

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
personagem voltou a pesar **91%** em vez de 53% — ⚠️ e o número é o do TETO, não
os 95% do `0,50` que a W11b mediu: a W11c pagou 4 pontos de peso pela deriva
zero, e a jangada é onde esses 4 pontos são visíveis.

---

## §8 — Aberto, com o preço ao lado

- ~~**O resíduo de `0,0331 m`/10 s no default**~~ — **FECHADO pela W11c: o default
  é o teto, e ali a deriva é zero exacto em toda inclinação de 20° a 45°.** O que
  segue aberto é o resíduo **abaixo** do teto, para quem baixar o knob à procura
  do quique: ele é exactamente linear em `(1 − d)` e vale
  `0,153 · sen θ · (1 − d)` m por 10 s. É o mesmo mecanismo um degrau abaixo: o
  termo da **MOLA**
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
