# HANDOFF DE INTEGRAÇÃO — `line/physics` (MESTRE, 2026-08-02)

**Status:** FECHADO 2026-08-02 · no `main` em `fcc8f145f` (o commit que trouxe este arquivo).

**38 commits, 123 arquivos, +12.200/−975.** Todos os smokes **APROVADOS pelo Enio**
(o último, `=75`, em 2026-08-02: *"Smoke OK"*).

⚠️ **Este documento SUPERSEDE
[`HANDOFF_INTEGRACAO_line_physics_compound_signal_2026-08-01.md`](HANDOFF_INTEGRACAO_line_physics_compound_signal_2026-08-01.md)**,
que cobria só os 24 primeiros commits. Aquele fica como histórico das três waves
do fim daquela jornada; **os números de identidade dele estão DESATUALIZADOS** —
use os do §2 daqui, que foram medidos nesta árvore, depois do rebase.

---

## §1 — O que integrar

- **Branch:** `line/physics`, worktree `Worktrees/line-physics`.
- **Já está rebasada sobre o tip do `main`** (`a9f5977e9`): `git merge-base
  --is-ancestor main HEAD` passa ⇒ **`--ff-only` é possível hoje**. Se o `main`
  andar antes de você, veja o §6.
- **Alcance:** `main..HEAD`, de `804dac9d9` a `bec42e2b9`.

⚠️ **Nada aqui é foundational contencioso:** **zero `Cargo.toml`**, **zero
`Cargo.lock`**, **zero crate nova**, **zero ADR**, **contrato congelado
intocado** (`architecture_tool_contract_surface` rodado: 4/4 verde), e a crate
`ph2d-ecs` **não foi tocada** (`git diff --name-only main...HEAD -- crates/ph2d-ecs/`
volta vazio). O diff inteiro mora nas crates de física, no Inspector e na shell.

---

## §2 — Os números de identidade (MEDIDOS agora, nesta árvore, pós-rebase)

| número | `main` | linha | observação |
|---|---|---|---|
| `PROJECT_SCHEMA` | **48** | **48** | ⚠️ **INTOCADO** — ver §2.1 |
| registro `ph2d-physics-ecs` (`reg.len()`) | 24 | **26** | `SignalOnHit` · `RopeStops` |
| registro `ph2d-ecs` | — | — | **intocado** |
| gizmo ids | ≤ 971 | **972 · 973** | `GIZMO_ROPE_STOP_A`/`_B`; **próximo livre 974** |
| `physics_ecs_c9` | — | **`16ba80e807ebc8097ffe1b6da87fb651ed4914ce34408a46629bccda596f75c8`** | 99 corpos, **debug ≡ release** |
| ADRs novos | — | **nenhum** | tudo sob o ADR-0131 |
| cenas de smoke | ≤ 69 | **70 · 71 · 72 · 73 · 74 · 75** | |

### §2.1 — ⚠️ Esta jornada fica FORA da disputa de `PROJECT_SCHEMA`

Os dois componentes novos são **componentes REGISTRADOS**, e o blob de um
componente é chaveado por `stable_type_id = blake3(NOME)[..8]` — cunhar um id
novo **não move layout nenhum**. Bumpar seria o oposto de conservador: um schema
divergente **recusa o arquivo inteiro**, e jogaria fora todo projeto já salvo
para melhorar uma mensagem de erro na única direção que não funciona de qualquer
jeito.

⚠️ **O número que de fato CONTA entre linhas aqui é a contagem do registro.** Se
outra linha desta janela também registrar componente em `ph2d-physics-ecs`, o
valor certo **não está em nenhum dos dois lados do conflito: ele se conta a
partir do `main` do dia** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

✅ **Boa notícia medida:** essa contagem vive em **UM único lugar**
(`crates/ph2d-physics-ecs/src/lib.rs:141`) — grep confirmado, **sem espelhos**.
Ela **não** tem a armadilha de três cópias que a contagem do `ph2d-ecs` tem
(`ph2d-render` + `ph2d-script`), que já ficou vermelho-latente duas vezes na
`line/Vector`.

⚠️ **E os gizmo ids somam igual.** Se outra linha reivindicou 972/973 na mesma
janela, renumere — os NOMES de arquivo/símbolo diferem, então **o git nunca
conflita** e a colisão passa muda.

---

## §3 — A espinha da jornada: **uma premissa que envelheceu**

A W-Compound (integrada em 01/08) deu a um corpo **várias formas**. A frase *"um
corpo tem exatamente um collider"* estava escrita — em código e em doc-comment —
em quatro lugares que ninguém reconferiu, e cada um virou um defeito de classe
diferente:

| # | onde | o que quebrou | medido |
|---|---|---|---|
| 1 | `§11` do Inspector | a peça era **inedidável** e a única porta oferecida a **apagava** | W-PartFace |
| 2 | canal de triggers (por CORPO) | o **sensor de pé** — o `isGrounded` de Box2D/Unity — nascia morto | tronco assenta 1,6990 sólido × 1,4990 sensor, e `triggered_sensors()` **vazio nos dois** |
| 3 | zonas (`rb.colliders().first()`, **5 sítios**) | a jangada composta **CAPOTA** | controle 0,000° × composta **−90,007°** |
| 4 | contatos (`contact_pairs()` itera colliders) | um corpo composto tocava **duas vezes** | 2 entradas / impulso 0,030677+0,030636 × **1 / 0,061313** |

⚠️ **O caso 3 era invisível por COMPENSAÇÃO:** a zona aplicava o empuxo uma vez
por PAR de colliders, então uma jangada de duas metades iguais levava
`2 × meia-força` = a força certa, **por acidente aritmético**. Consertar só o
empuxo a faz boiar com METADE da submersão — *meia correção é pior que nenhuma*.

⚠️ **O caso 4 é o único em que a frase estava escrita como LEI** (*"dois objetos
se tocando é UM evento; relatar cada quina responde quantas QUINAS"*). Ela valia
para **pontos de contato** e quebrou para **FORMAS**.

---

## §4 — As waves, na ordem em que entram

| wave | o que é | cena |
|---|---|---|
| **W-PartFace** | a peça vira **editável** (a 3ª face do §11); `has_collider` **não é** o complemento de `has_body` | `=70` |
| **W-PartSensor** | ser sensor é propriedade da **FORMA**, nunca do corpo | `=71` |
| **W-CompoundZone** | uma zona vê o corpo composto **inteiro** | `=72` |
| **W-PartMass** | o seed do `Mass: Auto → Manual` conhece as peças (0,600 semeado × **1,200** reais) | — |
| **W-CompoundContact** | um corpo composto toca **uma vez** | — |
| **W-WorldPinGlyph** | a ponta que é o **cenário** ganha figura | — |
| **W-WorldPinLocal** | a alça de **onde no corpo** o pino prende | — |
| **W-Signal** | uma colisão passa a **fazer alguma coisa acontecer** (`SignalOnHit`) | `=73` |
| **W-LeadDrag** | arrastar o corpo da âncora **leva o sistema** — rígido em FK, como uma **corda** em IK | `=74` |
| **W-RopeStop** | **o LIMITADOR** — a ponta da corda para antes da roldana | `=75` |

Detalhe de cada uma, com as medições e as mutações, no tracker
[`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) e no mapa
[`00_plano_waves.md`](../00_plano_waves.md), **os dois já atualizados**.

### §4.1 — A última wave (W-RopeStop) teve DUAS rodadas de smoke, e a 2ª é o que vale ler

Pedido do Enio, com desenho: *"limitadores de modo que os objetos nunca colidam
com as polias … dois por corda, desenhados em cima da corda, movidos com o mouse,
um círculo com um x dentro"* + *"permita selecionar as polias com mouse no
canvas"*.

**1ª rodada — o defeito é pior que *"colide"*.** Medido antes de uma linha: a
folga de tangente chega a **0,0000 aos 12,00 s**, a rota **degenera**, o passe
recusa a corda inteira (*"meia rota é pior que nenhuma"*) e a carga **deixa de ser
segurada** — ela é DEVOLVIDA (1,18 m aos 10 s, 3,40 aos 15), sem erro e sem aviso.

⚠️ **A grandeza não foi escolhida:** `len = √(d² − r²)` zera **exatamente quando a
amarração toca o ARO**. Uma distância ao CENTRO diria meio metro de folga com a
carga já encostada numa roldana de meio metro.

**2ª rodada — *"uma força bizarra que empurra o objeto na direção x das polias"*.**
Medido: a força que segurava a carga estava **23,76° FORA da corda**, e o desvio é
pura geometria — `atan(r/s)`, de **9,5° a 76°**. Uma corda puxa ao longo de si
mesma; qualquer componente perpendicular é força sem matéria que a transmita.

A v1 fazia a **CORDA** falar o radial quando a ponta travava. Hoje quem cede é a
**TRAVA**, e só na metade em que ela pode: **empurra** pela corda, **sente** pelo
radial (`End::k2`, a forma bilinear `gᵀM⁻¹u`; `End::k` vira `k2(dir, dir)`, uma
porta só).

⚠️ **Nenhuma das metades basta, e as duas foram medidas uma contra a outra:**

| lei | deriva lateral (prumo, 3 s) | folga mínima (roda r=2,0, limitador 0,5) |
|---|---|---|
| sente e empurra pela CORDA | **0,0000 m** | **0,0000 m** — não segura nada |
| sente e empurra pelo RADIAL | **1,0445 m** | 0,4948 m |
| **sente radial, empurra corda** | **0,0000 m** | **0,3685 m** |

⛔ **MEDIDO E REJEITADO, não refaça:** folgar o orçamento da corda pela violação
da trava (*"o nó para a corda de correr"*, que a doc da v1 dizia em palavras)
compra **0,0007 m** — ruído.

⚠️ **A barra do gate `the_stop_holds_on_a_big_wheel…` foi RE-MEDIDA (0,45 →
0,30), não herdada.** Ela era calibrada para a lei que empurrava pelo radial — que
é justamente a que punha a carga de lado. **Se você a vir e achar que alguém
afrouxou um gate: não; o número honesto da lei correta é 0,3685, e a doc dele traz
a tabela.**

---

## §5 — Verificação (rodada NESTA árvore, pós-rebase)

| gate | resultado |
|---|---|
| `cargo fmt --all --check` | **exit 0** |
| `cargo test -p ph2d-physics --release` | **65 binários, todos `ok`** |
| `cargo test -p ph2d-physics-ecs --release` | todos `ok`, zero `FAILED` |
| `cargo test -p ph2d-host-desktop --release` | **75 binários, todos `ok`** (inclui `file_loc_caps.rs`) |
| `cargo clippy -p ph2d-physics -p ph2d-physics-ecs --all-targets --release` | **zero warning** |
| `architecture_workspace_file_loc_cap` | ok (2/2) |
| `architecture_tool_contract_surface` | ok (4/4) |
| `physics_ecs_c9` debug × release | **idênticos** |

⚠️ **Rode `-p ph2d-host-desktop` na árvore COMBINADA.** Os gates de
`shells/desktop/tests/` **só correm na varredura impactada**, e um fechamento por
`cargo test -p` por crate não os alcança — a causa estrutural que a `line/Vector`
(23/07) e a `line/motion-value` (01/08) já documentaram, cada uma com um gate
vermelho-latente que só a integração viu.

⚠️ **E rode `physics_ecs_c9` nas duas árvores.** O hash desta linha é
`16ba80e8…`; ele foi movido pela **W-CompoundZone** (`e216e367…` → `16ba80e8…`,
atribuído por ablação no tracker). A **W-RopeStop não o move**: medido idêntico
antes e depois da correção, porque nenhuma corda do c9 carrega limitador.

---

## §6 — O que re-conferir se o `main` andar antes de você integrar

1. **`PROJECT_SCHEMA`** — esta linha **não o toca**, então ela não entra na
   disputa. Mas confira que o valor no `main` combinado continua batendo com o
   `project_schema_tests.rs`; ⚠️ **o `project.rs` pode NÃO conflitar** quando duas
   linhas escrevem o mesmo literal, e o único sinal é o conflito no arquivo de
   teste ao lado (a armadilha que a `line/FLIP` pagou em 01/08).
2. **A contagem do registro de física (26)** — some, não escolha.
3. **Gizmo ids 972/973** — renumere se alguém os reivindicou; o git não conflita.
4. **`every_physics_component_is_authorable`** tem uma lista **enumerada** de
   arquivos de escrita (`WRITERS: [&str; 7]`). Se outra linha dividir um arquivo
   do Inspector, a lista precisa crescer — o gate falha ALTO, que é o desenho.
5. **`ITEMS` do transporte / listas compartilhadas** — esta linha não as tocou,
   mas outras jornadas já colidiram ali; um `git diff` confirma em segundos.

---

## §7 — Aberto (não bloqueia; nada aqui é dívida desta jornada)

- A trava é uma restrição de **posição**, então um balanço violento a ultrapassa
  por um sub-passo: **0,3685 contra os 0,5 pedidos** na roda de raio 2,0 com o
  limitador 4× menor que ela. Mesma classe do esticamento da corda que o
  `PULLEY_BIAS` já nomeia, e o número está **pinado no gate**.
- Métodos de **shape** e o **Slider como elo de corda** seguem como o tracker os
  deixou.
- A corda de IK **não colide com nada** enquanto é arrastada — é pose, não
  simulação (o mesmo que vale para a IK).

---

## §8 — Smokes (todos `--release`, todos APROVADOS pelo Enio)

```
env PH2D_PHYSICS_SMOKE=70 cargo run -p ph2d-host-desktop --release   # A CHAVE E A FENDA
env PH2D_PHYSICS_SMOKE=71 cargo run -p ph2d-host-desktop --release   # O SENSOR DE PÉ
env PH2D_PHYSICS_SMOKE=72 cargo run -p ph2d-host-desktop --release   # A JANGADA COMPOSTA
env PH2D_PHYSICS_SMOKE=73 cargo run -p ph2d-host-desktop --release   # O SINAL
env PH2D_PHYSICS_SMOKE=74 cargo run -p ph2d-host-desktop --release   # A CORDA E A PEÇA
env PH2D_PHYSICS_SMOKE=75 cargo run -p ph2d-host-desktop --release   # O LIMITADOR
```

⚠️ **A cena 75 não manda apertar `B`**, de propósito: o contorno **já nasce
ligado**, e `B` o DESLIGARIA — levando junto as alças que o passo seguinte manda
arrastar. (Foi um defeito meu, pego pelo gate
`a_scene_that_asks_for_a_handle_gesture_does_not_tell_you_to_toggle_the_overlay`.)

---

## §9 — Erros meus nesta jornada, para o integrador não repetir

- ⚠️ **Li verde onde havia vermelho:** uma varredura de suíte truncada em
  `head -40` escondeu um gate que falhava, e eu reportei "suíte verde" ao Enio.
  **Conte os binários (`grep -c "test result: ok"`), não corte a saída.**
- ⚠️ **Um oráculo meu dizia o OPOSTO do produto:** afirmei por intuição que a
  carga travada não podia ser atirada além do raio. Medido, a trava **REDUZ** o
  balanço (3,976 m com ela × 6,345 m sem). O controle o derrubou.
- ⚠️ **Uma doc de gate MENTIA:** a do `the_stop_holds_on_a_big_wheel…` descrevia
  como defeito exatamente o que hoje shipa. Reescrita — *um comentário que
  contradiz o código shipado é pior que comentário nenhum*.
