# HANDOFF MESTRE — `line/physics` → `main` (2026-08-04)

**A linha está FECHADA e PARADA.** 31 commits, 182 arquivos, +22.691/−1.481.
Nada integrado, nada pushado.

> Este é o documento do INTEGRADOR. Ele não repete o conteúdo das waves — ele
> traz o que só a integração precisa: **os números que se CONTAM, a superfície de
> colisão MEDIDA, a ordem, e o que o smoke já disse.**

---

## §1 — O que entra

Duas jornadas, na mesma linha, sem integração entre elas:

**(A) A cauda de 03/08** — 6 commits. `W-SignalLeave` (a porta que FECHA) ·
`W-PartAdopt` (`Make Independent Body` apagava a forma autorada) · `W-RopeSays`
(o readout de uma corda que não roteia DIZ isso) · `W-RailRope` (o trilho como
elo de corda) · `W-JointAnim` (um param de joint é uma entrada por TICK, com os
canais de timeline) · `W-JointCustom` (o joint que o artista descreve por EIXO).
Handoff próprio: [`HANDOFF_INTEGRACAO_line_physics_2026-08-03.md`](HANDOFF_INTEGRACAO_line_physics_2026-08-03.md).

**(B) O PLAYER DE PLATAFORMA, o módulo inteiro** — 25 commits, da pesquisa à
W11. Crate nova **`ph2d-platformer`** (lei pura, sem rapier, sem ECS) + a ponte +
a §14 do Inspector. Handoffs:
[`..._player_2026-08-04.md`](HANDOFF_INTEGRACAO_line_physics_player_2026-08-04.md)
(W1..W9) e
[`..._player_w10_2026-08-04.md`](HANDOFF_INTEGRACAO_line_physics_player_w10_2026-08-04.md)
(W10 nos §1-§8, **W11 nos §9-§15**).

---

## §2 — ⚠️ O NÚMERO QUE COLIDIU, e ele tem de ser CONTADO

**`PROJECT_SCHEMA`: a linha escreve `52`. O `main` de hoje diz `53`.**
⇒ **O valor certo é `54`, e ele não está em nenhum dos dois lados do conflito.**

A escada do `main` avançou enquanto esta linha corria:

| versão | dono | o quê |
|---|---|---|
| 50 | vector | `StrokeSpec.align` |
| 51 | vector | a tabela de cor autorada (`tokens`) |
| **52** | **3D** | o documento da escultura (`sculpt`) |
| **53** | **3D** | os canais assados (`baked_forms`) |
| **54** | **physics** | ⬅️ **o `PlatformPlayer` desta linha** |

⚠️ **É a QUARTA vez que este par de linhas paga isto** (30 em 25/07 · 32/33/34 em
27/07 · 47 em 01/08 · agora). A regra é a mesma:
[[feedback_numbers_that_sum_across_lines_count_dont_pick]] — *o valor se CONTA a
partir do `main` do dia, nunca se escolhe.*

**O que editar:** o literal em `shells/desktop/src/project.rs`, a entrada da
escada (o doc-comment `/// v52 (physics…)` vira `/// v54`, e ele tem de vir
DEPOIS das entradas v52/v53 do 3D) e o pin em
`shells/desktop/src/project_schema_tests.rs`.

---

## §3 — A superfície de colisão, MEDIDA (não estimada)

`git merge-tree --write-tree main HEAD`, dry-run não-destrutivo:

**Doze arquivos são tocados pelos dois lados. DEZ fundem sozinhos:**
`Cargo.lock` · `.typos.toml` · `crates/ph2d-editor-core/src/lib.rs` ·
`crates/ph2d-editor-core/tests/node_id_collisions.rs` ·
`crates/ph2d-i18n/src/lib.rs` · `shells/desktop/Cargo.toml` ·
`shells/desktop/src/app_state.rs` · `shells/desktop/src/input_dispatch/keyboard.rs` ·
`shells/desktop/src/main.rs` · `shells/desktop/src/render_loop/mod.rs`.

**DOIS conflitam, e são exatamente os do §2:**

```
CONFLICT (content): shells/desktop/src/project.rs
CONFLICT (content): shells/desktop/src/project_schema_tests.rs
```

⚠️ **Isso é uma boa notícia e vale dizer por quê:** na integração de 01/08 este
mesmo número quase passou **MUDO**, porque os dois lados escreveram o **mesmo
literal** e o git não tem opinião sobre o que um número SIGNIFICA. Aqui os
literais diferem (52 × 53), então o conflito é real e o git o mostra. **Resolva
pelos ESTÁGIOS do índice**, nunca pelos marcadores
([[feedback_resolve_conflicts_from_index_stages_not_markers]]).

### ⚠️ A falha EMERGENTE que só a árvore combinada produz — medida, e NÃO recorre

Em 27/07 `keyboard.rs` cruzou o cap de 600 porque duas linhas somaram sobre ele e
**nenhuma cruzava sozinha**. Medido hoje **na árvore fundida** do dry-run:

| arquivo | `main` | linha | **fundido** | cap |
|---|---|---|---|---|
| `input_dispatch/keyboard.rs` | 578 | 586 | **588** | 600 ✅ |

12 linhas de folga. Os outros grandes (`app_state.rs`, `main.rs`,
`render_loop/mod.rs`) carregam marcador `// ph2d-loc-cap:` e o gate é um **cap com
escape inline**, não uma lista de números exatos — crescer não os quebra.

---

## §4 — Os outros números que se contam

| número | `main` hoje | a linha | veredito |
|---|---|---|---|
| `PROJECT_SCHEMA` | **53** | 52 | ⚠️ **CONTAR para 54** (§2) |
| registro `ph2d-physics-ecs` | 26 | **28** | ✅ só esta linha o toca (`PlatformPlayer` + `SignalOnLeave`) |
| registro `ph2d-ecs` (as 3 casas) | — | **intocado** | ✅ nenhum componente de ECS novo |
| gizmo ids (máx) | 973 | 973 | ✅ **nenhum novo**; próximo livre segue **974** |
| ADR (máx) | 153 | 153 | ✅ **nenhum ADR novo** ⇒ fora da disputa de número |
| contrato congelado | — | intacto | ✅ conferido por gate, não por auto-relato |
| `physics_ecs_c9` | — | **`b3dbe792…`, 108 corpos** | debug ≡ release |
| deps novas | — | **nenhuma** | ✅ |
| `Cargo.toml` central | — | **crate nova** `ph2d-platformer` | membro por glob |

⚠️ **O `c9` da W11 é `b3dbe792…`, o MESMO da W10** — e isso é uma afirmação, não
um acidente: a lane do player é em chão plano, onde a normal É o `up` ao bit, e o
hash é o oráculo mais forte disponível para a byte-identidade que a W11 alega.

---

## §5 — Ordem

1. `git rebase main` (ou merge) — resolver **os dois** conflitos do §3.
2. **Contar** o `PROJECT_SCHEMA` para **54** nos três sítios do §2.
3. Rodar o gate da árvore combinada. ⚠️ **Em DEBUG E RELEASE**: esta linha tem
   precedente registrado de vermelho só-em-debug, e a suíte da `ph2d-platformer`
   roda em ~0 s.
4. Recomputar o `physics_ecs_c9` **depois** do rebase e conferir debug ≡ release.
   Ele **não deve mudar** — nada em `main` toca o solver desta linha —, e se
   mudar isso é o achado, não ruído.
5. `keyboard.rs` ≤ 600 (medido em 588, §3).

---

## §6 — ⚠️ O ESTADO DE SMOKE, item a item

**Aprovadas pelo Enio:** a cauda de 03/08 (`=76`..`=79`) e o player até a **W10**
(`=80`..`=90`) — *"Smoke OK"*.

**A W11 tem um veredito PARCIAL e ele é do Enio, hoje:**

> *"No setup como está o player sobe sozinho bem devagar. Mas faremos os ajustes
> amanhã."*

Isto **confirma a medição da wave e não a contradiz**: o default que shipa
(`spring_damping = 0,5`) deixa um resíduo de **0,164 m por 10 s numa rampa de
30°** — metade do que era (0,3295), e a wave o documenta como resíduo NOMEADO,
com um gate que o pina **dos dois lados**.

⚠️ **O ajuste está DIFERIDO por ordem dele**, não esquecido. A decisão e as quatro
colunas medidas estão no §12 do handoff da W10/W11; a cura que compra as duas
colunas (a perna **substitui** a gravidade em vez de a cancelar) está no §14 do
mesmo doc e no [`BUGS_physics.md`](BUGS_physics.md) §7, com a medição inteira já
feita.

⇒ **Integrar isto é integrar uma melhoria medida e um resíduo declarado**, não uma
feature meio-feita. Nada aqui espera o ajuste de amanhã para compilar, passar ou
fazer sentido.

---

## §7 — Cenas de smoke que a linha acrescenta

`=76` a porta que fecha · `=77` o trilho-corda · `=78` os params de joint na
timeline · `=79` o joint por eixo · **`=80`..`=90` o player** (a mola · andar · a
autoria · o pulo · a reação · a fita · o perdão · as rampas 40°/50° · a chaminé ·
o vagão).

⚠️ **`=84` não existe**, e é de propósito — o gate `no_two_smoke_scenes_claim_the_same_level`
existe justamente porque um roteador de `if level == N` deixa a segunda cena
**inalcançável em silêncio**.

Para o resíduo da W11 o gesto é **não fazer nada**: `PH2D_PHYSICS_SMOKE=81`, leve
o personagem à rampa e **solte as teclas**.

---

## §8 — Aberto, com o preço ao lado

- **O ajuste da W11** (§6) — diferido pelo Enio para amanhã, com a medição pronta.
- **O "pulinho"** do mesmo relato **não reproduziu** em cinco configurações de
  repouso; a sonda `measure_idle` já traz o instrumento se ele voltar.
- **`float_height` abaixo do mínimo geométrico** faz a cápsula ENCOSTAR e o solver
  de contato assumir — pré-existente, documentado no `min_float_height`.
- O resto do aberto por wave vive nos handoffs que o §1 lista.
