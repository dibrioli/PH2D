# W5 — diferenciais: «A e B colados» (objectos, nos dois sentidos)

> Plano: [`01_plano.md`](01_plano.md) §7 (linha W5) e o kill-criterion. Anterior: [`04_W4_cad.md`](04_W4_cad.md) §3.8.
> Escrito ANTES do código (handoff W5, item 1). Os outros três itens da W5 (física parâmetro→parâmetro, *behaviours*
> de um clique, layout de HUD) ficam para depois desta peça.

## 0. Decisões do dono (26/10)

1. A W5 começa por **«A e B colados»**.
2. Colam-se **dois OBJECTOS inteiros** (a mão e a espada), não pontos de formas — os pontos de formas já colam com as
   regras da W4 (`Lei::Coincidente` entre paths da mesma `Folha`).
3. Dois tipos: **Rígido** (solda: andam e giram juntos) e **Pino** (presos num ponto, cada um gira livre — braços,
   pernas, mecanismos, laços como o quadrilátero articulado).

## 1. O que existe e porque não chega

- A pilha W1–W3 é de **um sentido** ([ADR-0192](../architecture/decisions/0192-a-pilha-de-constraints-e-um-componente-por-entidade-avaliada-da-pose-autorada.md)
  §2): o dono lê a pose autorada do alvo e nunca escreve nele; a pilha escreve `PreviewDrive`
  (`Driver::ConstraintPose`), nunca o documento. Child Of em A→B e B→A é um **ciclo**
  (`ph2d-constraints-ecs/src/ordem.rs:60-82`, `Estado::Ciclo`): os dois param.
- O Blender proíbe o ciclo (research notes `blender.md:153-156`); nenhum app de animação 2D que conhecemos junta
  «arrasta qualquer um dos dois» com um mecanismo de pinos resolvido. É o diferencial.
- O solver da W4 (`ph2d-vec-constraints`, `kcl-ezpz =0.2.18`) já resolve pontos de vários paths com prioridade
  (o ponto agarrado é `Fixed` de prioridade máxima) e cabe no kill da W4 (202 pontos: pior passo 0,68–0,81 ms).

## 2. O desenho

### 2.1 O dado — uma relação, não um item da pilha

`Cola { a: StableId, b: StableId, tipo: Rigida | Pino { em_a: [f64; 2], em_b: [f64; 2] } }` — uma lista de colas da
**cena** (componente do objecto raiz Cena, como as regras de CAD são da `VecScene`), referência durável por
`StableId` (lei do §5.1 Editor). Rígida guarda a pose relativa de B em A no momento de colar (`b.relative_to(a)`,
a mesma conta do Child Of `keep_current`, `ph2d-constraints/src/kind.rs:204` — chamada, nunca copiada).
O Pino guarda o ponto em coordenadas locais de cada um (onde o artista clicou), e nada sobre rotação.

⇒ **ADR novo** (o número conta-se na integração): «a cola é uma relação simétrica resolvida por GRUPO antes da pilha;
a pilha continua de um sentido». Sobe o `PROJECT_SCHEMA` (+1, recontado na integração).

### 2.2 Os dois momentos

**A. O gesto (editar).** O artista arrasta (ou roda) um objecto colado: ele é a **mão** (fixo, prioridade máxima) e
o resto do **componente conexo** do grafo de colas segue. A escrita é no **documento** (o `Transform` autorado de
cada parceiro), **no mesmo passo de undo** do gesto, e o Auto Key grava todos (lei do dono «grava tudo»). A lição da
W4.6 vale aqui: a população do Auto Key é seleção + mãos + `AlvosDasFamilias` — o grupo publica os parceiros em
`AlvosDasFamilias` **antes** do gesto (o diff precisa do quadro anterior).

**B. O quadro (tocar/scrub).** Num quadro, os **condutores** de um grupo são os membros cuja pose vem de outra
coisa que o seu `Transform` parado: curva na timeline, a pilha, a física. Sem condutor: nada a fazer (o documento
já é coerente, o gesto escreveu os dois). Com um condutor: o resto segue. Com dois condutores que discordam:
`Estado::Conflito` no cartão (a cor do CAD: livre · definido · conflito) e cada um fica na sua pose.
**Determinismo:** o palpite do solver é a pose AUTORADA do quadro, nunca a do quadro anterior ⇒ tocar × scrub = o
mesmo bit (o gate da cena 7 é o modelo).

### 2.3 Como se resolve — fecho analítico primeiro, solver só onde é preciso

| Grupo | Resolução |
|---|---|
| Árvore de colas rígidas | Fecho: BFS a partir do condutor, `B = A · rel`. Zero solver. |
| Árvore com pinos | Fecho: o seguidor mantém a SUA rotação autorada e translada até o pino coincidir. |
| **Laço** (ciclo no grafo) ou ≥ 2 condutores | `kcl-ezpz`: cada corpo = 3 pontos (origem, `+x̂`, `+ŷ`) com as 3 distâncias fixas (a escala autorada viaja); pino = ponto extra de cada corpo (3 distâncias às do corpo) + `PointsCoincident`; rígido = os pontos do outro corpo presos aos do primeiro. Condutores `Fixed`, prioridade máxima. |

O solver recebe o palpite autorado; o resultado sai como `Transform` (origem + ângulo do `+x̂`). A porta do
`ph2d-vec-constraints` que traduz `Lei` → `K::*` não muda: a cola tem a sua tradução, na crate nova.

### 2.4 Onde mora

- Crate nova **`ph2d-constraints-cola`** (drop-crate): o dado, o grafo (componentes conexos, ciclos), o fecho, a
  tradução para ezpz. Sem ECS.
- `ph2d-app-constraints`: o passo por quadro dentro de `quadro()` (`src/lib.rs`, depois de `caminhos`, **antes** do
  `evaluate` da pilha — a pilha lê o que a cola deixou) e o gesto. **Zero linhas novas na shell** (tecto com folga 1).
- A porta do gesto do gizmo de objecto: **por achar** (o explorador não encontrou a linha que grava o `Transform` em
  `input_dispatch/gizmo_drag.rs:45`) — passo 1 do código é lê-la; se não houver uma porta única, a cura é criá-la
  em crate e a shell chamá-la (troca de linha, nunca linha nova).

### 2.5 A UI (as 4 condições do §4 do plano)

- **Add Constraint → «Glue · Rigid» e «Glue · Pin»**, alvo por clique (o mesmo picker da W1). A cola aparece no
  cartão dos DOIS objectos (é simétrica), com o mesmo estado (definido · conflito) e um botão «Unglue».
- **Pino:** um gizmo no canvas no ponto (arrastável: move o pino nos dois locais de uma vez).
- Selos no canvas só com o cartão aberto (lei W4.6: selos com a tool, cotas sempre).
- i18n nas duas línguas; tokens; nada de hex/literal.

## 3. Oráculo — antes do código (§0.9)

**FreeCAD 1.1.4 Assembly** (instalado; `freecadcmd` responde). Juntas `Fixed` (= Rígido) e `Revolute` (= Pino),
`assembly.solve()`. Triagem de licença por artefacto (passo 1): o pacote diz LGPL-2.0-only, os `.py` do Assembly
LGPL-2.1+, e `libOndselSolver.so` **não traz LICENSE** no pacote — conferir o artefacto upstream antes de correr.
Corre-se por script sobre entradas NOSSAS, nunca se lê o fonte. Script `ferramentas/oraculo_freecad_assembly.py`,
fixture `fixtures/freecad_1_1_4_assembly_cola.json` com cabeçalho. Casos:

1. Cadeia de 3 corpos com 2 pinos, o 1.º arrastado (rodado 30°): poses dos outros.
2. Rígido: B colado a A, A movido e rodado: pose de B.
3. **Quadrilátero articulado** (4 corpos, 4 pinos, chão fixo): manivela a 0°, 30°, 60°, 90° → ângulos do balancim.
4. O mesmo arrastando o balancim (o sentido contrário: a manivela segue).
5. Laço impossível (comprimentos que não fecham): o que o FreeCAD diz (o nosso: `Conflito`).

Comparação **por passo**, tolerância declarada no gate (o FreeCAD é 3D/float; a nossa conta é 2D/f64).

### 3.1 Triagem e corrida (27/10)

- **Licença por artefacto:** `freecad 1.1.4-3` = LGPL-2.0-only (`pacman -Qi`); `libOndselSolver.so.1.0.1` vem
  dentro dele (`/usr/lib/freecad/lib/`), sem ficheiro LICENSE — os cabeçalhos instalados dizem «Copyright (c) 2023
  Ondsel, Inc. … See LICENSE file» (o upstream é LGPL-2.1). Copyleft ⇒ **só se corre**, nunca se lê; nenhuma
  porta permissiva a reaproveitar.
- **Idioma medido do 1.1:** a referência da junta enraíza-se no COMPONENTE (`[corpo, ["VertexN","VertexN"]]`; com a
  raiz no Assembly o `getMovingPart` devolve o próprio Assembly e a junta não liga); criar a junta pré-alinha os
  corpos ⇒ a pose autorada repõe-se DEPOIS, e só então o `GroundedJoint` do condutor.
- ⛔ **`solve()` devolve 0 num laço que NÃO fecha** (caso 5: resíduo 1,5) e também com o balancim fora do curso
  (98,6° com curso 101,4°–141,4°, primeira corrida). Quem decide é o **resíduo** gravado por caso, nunca o código.
- **Caso 1 (cadeia sub-determinada):** o FreeCAD distribui a rotação (p2 −23,4°, p3 0°); a nossa lei (§2.3) é a do
  animador — cada seguidor mantém a SUA rotação autorada. Diferença de desenho declarada; o gate do oráculo cobre os
  casos 2–5, que são determinados.
- Saída: [`fixtures/freecad_1_1_4_assembly_cola.json`](fixtures/freecad_1_1_4_assembly_cola.json) (entradas NOSSAS
  dentro da fixture, o gate lê-as dali).

## 4. Gates red-first

| Gate | Prova | Mutação que sangra |
|---|---|---|
| `dragging_either_glued_object_moves_the_other` | arrasto de A move B; de B move A; um passo de undo | escrever só o arrastado |
| `a_rigid_glue_keeps_the_relative_pose` | `B = A · rel` bit a bit (fecho) | trocar a ordem do produto |
| `a_pin_keeps_the_points_together_and_each_its_rotation` | pino coincidente ≤ 1e-9, rotação do seguidor intacta | rodar o seguidor com o condutor |
| `the_four_bar_matches_freecad_step_by_step` | os 4 passos do caso 3 e o caso 4 contra a fixture | trocar a prioridade do condutor |
| `two_drivers_that_disagree_are_a_conflict_not_a_fight` | `Conflito`, ninguém se mexe | aceitar o 1.º condutor |
| `glued_play_and_scrub_are_the_same_bit` | tocar × scrub, 49 quadros | palpite = quadro anterior |
| `autokey_records_every_glued_partner` | uma chave em cada parceiro no quadro do gesto | não publicar `AlvosDasFamilias` |
| `a_glue_survives_save_and_open` + ficheiro v195 abre sem colas | o degrau novo | — |
| `the_glue_card_shows_on_both_objects` | costura do cartão, pelo clique real | — |
| **kill** `glue_frame_under_half_a_millisecond_per_thousand` | 1 000 objectos colados (cadeia + laços), `--profile smoke`, o MÍNIMO de blocos intercalados | — |

### 4.1 Mutações (27/10)

`ph2d-constraints-cola` (`tests/cola.rs`): produto trocado, seguidor rodado com o condutor e «aceitar o 1.º
condutor» sangram. ⚠ Duas são **equivalentes** e ficam escritas: (a) baixar a prioridade do `Fixed` do condutor não
muda nada num sistema consistente (nada compete com ele) — a variante «sem `Fixed` no condutor» sangra o quadrilátero;
(b) o `is_satisfied()` em `ez.rs` é 2.ª barreira: o laço impossível já sai por `Err` do solver — a variante «`Err`
vira a pose do palpite» sangra. Mundo (`ph2d-constraints-ecs/tests/it/cola.rs`): «escrever só o arrastado», «palpite
= quadro anterior» (scrub 0,729 × tocar 0,948 rad no quadro 5), «não publicar `AlvosDasFamilias`» e «o cadeado não é
chão» — 4/4 sangram. Desenho do mundo: [ADR-0193](../architecture/decisions/0193-a-cola-e-uma-relacao-simetrica-resolvida-por-grupo-antes-da-pilha.md).

### 4.2 Kill medido (27/10) — ✅ FECHADO na mesma data: cola 0,44 ms, pilha 0,29 ms (§4.3); abaixo, a 1.ª medida

Bancada `ph2d-constraints-ecs/tests/it/kill.rs` (`#[ignore]`; `--profile smoke`, que herda `release`; 7 blocos × 40
quadros, cola e pilha intercaladas, o MÍNIMO): **cola 5,30 ms / 1 000 objectos** (100 quadriláteros na ezpz + 60
cadeias de 10 pinos no fecho) · **pilha 6,18 ms / 1 000 itens** (500 donos × Copy Location + Copy Rotation em
cadeia). `loadavg` 4,72 → 6,11 (acima de ~5 o relógio vale pouco, mas a distância é de uma ordem). Tecto NÃO mexido.
Próximo passo: perfilar antes de curar (suspeitos por ler, não medidos: `iter_entities` + `indice()` a cada quadro, o
`parent_world_transform` por membro, uma `kcl_ezpz::solve` por quadrilátero; na pilha, `ordem()` e clones por
quadro). A cola não depende disto para o smoke (5 objectos).

O kill-criterion da **pilha** (W1, 0,5 ms / 1 000 itens) **nunca foi medido** (o explorador não achou bancada):
mede-se na mesma bancada, na mesma rodada (§0.10).

### 4.3 Perfil e curas (27/10)

`perf record` sobre a bancada (`--profile smoke`, `CARGO_PROFILE_SMOKE_STRIP=none`) e depois fases com `Instant` (cópia
temporária, nunca comitada). Antes: `ordem()` (o laço de Kahn reavaliava todos os nós a cada camada: O(n²) na cadeia
de 500) era ~33 % das amostras; `PreviewDrive::drives_other_than` varria o ledger inteiro por membro (O(n²), ~12 %);
faer/ezpz ~25 %. Mínimo de 7×40, `loadavg` ao lado (1 min):

| cura | cola | pilha | load |
|---|---|---|---|
| antes | 5,47 ms | 6,29 ms | 16,5 |
| `ordem` por Kahn com contagem (a mesma ordem por camadas) + ledger por `range` da entidade | 3,69 | 0,53 | 21 |
| régua e compasso antes da ezpz (`construcao.rs`: díade e corpo por dois pinos) | 1,66 | 0,53 | 26 |
| a estrutura dos grupos em cache (`Cache`/`resolve_com`; chave = colas + condutor/cede de cada corpo) | 0,88 | 0,58 | 31 |
| `Vec` pela ordem do `StableId`, arquétipos só com o componente, índice uma vez | 0,52 | 0,50 | 14 |
| `ordem` em índices densos + alvo do item uma vez por pilha + `sincosf`/`tanf` saltados no zero (ao bit) | 0,48 | 0,28 | 4,4 |
| `driven` só percorre a camada quando outra mão escreveu; rascunhos reutilizados; `Estados` montados de uma vez | **0,42** | **0,28** | 4,6 |
| final (3 corridas) | 0,436–0,441 | 0,287–0,291 | 12,3 |

Lei: a construção só responde quando coloca TODOS os corpos e fecha TODOS os pinos; senão é a ezpz (laço que não
fecha continua conflito dela). Ramo da díade = o mais perto do palpite autorado. Gate
`the_construction_gives_what_ezpz_gives_around_the_whole_turn`: 292 casos (volta inteira a 7,5°, escala não
uniforme, espelho, balancim condutor), pior desvio 8,9e-9 (o critério de convergência da ezpz). Ele achou um defeito
da ezpz: com `sx < 0` a rotação saía 180° ao lado (`u − o` aponta para trás) — corrigido em `ez.rs`. A cache só
depende das entradas e o caminho sem ela é o mesmo `prepara`+`executa`: quente e frio dão o mesmo bit.
Resta (fases médias da cola, load 3,5): membros ~136 µs, resolve ~108, saída ~90, fim ~59 — custo por membro em
`BTreeMap` (índice, ledger, memória) e alocação; nenhum dominante.

## 5. Cena de smoke 8 — «Braço articulado»

Formas (lei do Motion: formas e simulação): um chão, uma manivela, uma biela e um balancim com 4 pinos, e uma
«mão» colada rígida na ponta do balancim. O dono arrasta a manivela (o balancim segue), depois o balancim (a
manivela segue), depois a mão (tudo segue); com Auto Key, grava 3 quadros e toca. Fotografada antes de mandar.

## 6. Ordem do código (uma janela)

1. Ler a porta do gizmo de objecto (§2.4) e o undo do gesto.
2. Oráculo (triagem de licença → script → fixture).
3. Crate `ph2d-constraints-cola` (dado, grafo, fecho, ezpz) + gates de fecho e do oráculo, red-first.
4. O quadro (`quadro()`), o gesto, `AlvosDasFamilias`, gravar/abrir (+1 schema), ADR.
5. UI (Add Constraint, cartão nos dois, gizmo do pino), cena 8, kill + kill da pilha numa rodada.
