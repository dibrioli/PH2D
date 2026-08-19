# Handoff de integração — `line/physics` (2026-08-09)

**Status:** FECHADO 2026-08-09 · no `main` em `ae29c9870` (o commit que trouxe este arquivo).

> **A linha NÃO integra nem faz ship** (CLAUDE.md §0.7). Este documento é o que o
> integrador precisa para não colidir nem regredir. DIRETRIZ §1.5.9.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/physics` |
| HEAD | `f0b8c699af4593664bbd2dfdc32b30ab16788532` |
| merge-base com `main` | `17a0f6d6d2999de4111ea4e9f3c61b08d6f982cd` |
| commits | **21** |
| diff | 68 arquivos, +9.849 / −414 |

⚠️ **Todos os 21 são pós-integração de 2026-08-08** (a jornada `W-KinMove`, que já
está no `main`). Nada aqui foi entregue antes.

**O assunto é UM:** o **modo cinemático** do player de plataforma — o que faltava
nele depois de o modo nascer. Sete waves e cinco medições, e a última é a água.

---

## 2. Foundational / compartilhado tocado, e por quê

Fora de `crates/ph2d-phys*`, `crates/ph2d-platformer` e `docs/Physics/`:

| arquivo | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/inspector_player.rs` | 4 ids novos (§3) | **sim** |
| `crates/ph2d-editor-core/src/screens/hero/inspector_model_player.rs` | campos do snapshot da §14 | **sim** |
| `crates/ph2d-editor-core/src/screens/hero/inspector_model_physics.rs` | a massa deixa de ser oferecida por KIND | ⚠️ **não** — muda um predicado |
| `crates/ph2d-panel-inspector/src/{sections/player.rs, event_player.rs}` | as rows do modo + do empurrão | **sim** |
| `crates/ph2d-panel-inspector/src/{sections/physics_body.rs, sections/physics_rows.rs, event_physics.rs, populate_physics.rs, sync_physics.rs}` | a `W-KinWeight` | ⚠️ **não** — ver abaixo |
| `crates/ph2d-panel-inspector/tests/seam_{player,physics}.rs` | seams | **sim** |
| `shells/desktop/src/main.rs` | 3 `mod` de cena de smoke | **sim** |
| `shells/desktop/src/physics_smoke.rs` | 3 braços do roteador (`"102"`, `"103"`, `"104"`) | **sim** |
| `shells/desktop/src/project.rs` + `project_schema_tests.rs` | **`PROJECT_SCHEMA` 69 → 70** (§3) | ⚠️ **não** |
| `shells/desktop/src/render_loop/inspector_{player,physics}*.rs` | fiação da §14 | **sim** |
| `CLAUDE.md` | a §5 desta linha | **sim** |

⚠️ **A única mudança de PREDICADO compartilhado é a `W-KinWeight`** (`09abd85e0`): a
row de massa era oferecida por **KIND** (`Dynamic`) e passou a ser oferecida por
**quem a LÊ** — um corpo cinemático não tem massa para o solver, mas a lei do
player a lê. Se outra linha tocar `inspector_model_physics.rs` no mesmo predicado,
é ali que o conflito nasce; o resto é apêndice.

---

## 3. Símbolos que podem COLIDIR

### 3.1 `PROJECT_SCHEMA` — **69 → 70**, e o valor é PROVISÓRIO

⚠️ **O número se CONTA contra o `main` do dia, nunca se escolhe**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). Esta linha escreveu
**70**; se outra linha desta janela também bumpar, o valor certo pode não estar em
nenhum dos dois lados.

**Um degrau só:** `PlatformPlayer.reaction_push` — o terceiro escalar da 3ª lei
(quanto de um bloqueio **lateral** volta para o corpo que o causou). Campo de
componente, postcard **posicional** ⇒ um leitor velho leria os campos seguintes
deslocados.

⚠️ **E a colisão pode passar MUDA:** se a outra linha escrever o **mesmo literal**,
o `project.rs` **não conflita** — o git não sabe o que o número significa, e o bump
de uma delas evapora com a suíte verde. Foi exactamente o que quase aconteceu entre
`physics` e `FLIP` em 01/08, e quem denunciou foi o conflito do
`project_schema_tests.rs` ao lado. **Confira os dois arquivos, não um.**

Tripla do pin nesta linha: **`(70, 13, 14)`**.

### 3.2 Ids novos — **todos por HASH DE STRING**, nenhum numérico

```
INSP_PLAYER_MODE_IDS: [NodeId; 3]  = hash_node_id("insp_player_mode_{pure,kinematic,dynamic}")
INSP_PLAYER_REACT_PUSH: NodeId     = hash_node_id("insp_player_react_push")
```

⚠️ **Nenhum id numérico foi cunhado** — nem gizmo (o último segue **973**, próximo
livre **974**), nem scrollbar. O `node_id_collisions` cobre os de hash.

### 3.3 Cenas de smoke — **`=102`, `=103`, `=104`**

⚠️ **O roteador é um `match` de strings cujo `_` cai na cena 1**: um nível
duplicado não avisa, o primeiro braço vence e o outro fica **inalcançável em
silêncio** (o precedente é a `line/Vector`, que perdeu a cena dos tokens assim).
**Próxima livre: `105`.** O `=84` não existe de propósito.

### 3.4 O que NÃO se moveu

| | |
|---|---|
| registro `ph2d-physics-ecs` | **fica 29** (nenhum componente novo) |
| registro `ph2d-ecs` e os **dois** espelhos | **intocados** |
| `VEC_SCENE` / `FLIP_SCHEMA` | intocados |
| ADRs | **nenhum** ⇒ a linha fica **fora de toda disputa de número** |
| `Cargo.toml` / `Cargo.lock` | **zero** — nenhuma crate nova, nenhuma dep nova |

⚠️ **A §9 do plano 07 está STALE em dois pontos** e foi escrita antes desta jornada:
ela diz *"registro 28 → 29"* (isso foi a `W-KinMove`, já no `main`) e *"`PROJECT_SCHEMA`
NÃO bumpa"* (bumpou, ver §3.1).

---

## 4. Contratos congelados encostados

**Nenhum.** Conferido por `git diff --stat main..HEAD` em `crates/ph2d-nodegraph` e
`crates/ph2d-core/src/tool.rs`: **vazio**. `NodeOp`/`OpResolver`/`NodeManifest` e
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` intactos.

---

## 5. O que só o `ship.sh` pega

O gate de integração **não** roda estes ([[project_integration_prefork_lines_ship_drift]]):

- **`cargo machete`** — a linha não acrescentou dependência nenhuma, então não há
  risco novo; mas ela também não o rodou.
- **`cargo deny` / `cargo audit`** — idem, sem deps novas.
- **`typos`** — os docs desta linha são longos e em pt-BR.
- **clippy `--all-targets --all-features`** — a linha rodou clippy em
  `ph2d-physics`, `ph2d-physics-ecs`, `ph2d-platformer` e `ph2d-host-desktop`, **sem
  `--all-features`**.
- **`file_loc_caps` da shell e o `architecture_workspace_file_loc_cap`** — rodados
  por contagem manual nos arquivos tocados (todos sob o teto; o maior é
  `bridge/player.rs` em **664/700**), **não** pelo gate.

⚠️ **E o precedente desta linha:** os gates de `shells/desktop/tests/` **só correm
na varredura impactada**, e um fechamento por `cargo test -p` por crate não os
alcança. Foi assim que a `line/Vector` e a `line/motion-value` shiparam
vermelho-latente. **A árvore combinada é quem decide.**

---

## 6. Ordem, dependências e o que smoke-testar

### 6.1 Ordem

Os 21 commits são **sequenciais e auto-contidos**; não há par cuja ordem seja
load-bearing. Uma ressalva:

⚠️ **`1c6dc41bc` (a sonda que mede a água quebrada) e `7c345729c` (a cura) têm de
ficar nessa ordem** — o primeiro grava no doc a afirmação *"metade da água já
atravessa"*, que o segundo **corrige por ser falsa**. Reordenar deixaria a correção
antes do erro que ela corrige.

<details>
<summary>Os 21, em ordem</summary>

- `acd439a4f` fix(player): a absorção perguntava ao INTEGRADOR, e a resposta da lei sobre chão é o footing
- `4386c3b74` meas(player): o resíduo da W11 está MORTO no default — 0,0000 em 120 s e até 44 graus
- `e71c38e78` meas(player): a K6 já passa 100% do peso sob Snap — a W-KinWeight não é o que o plano diz
- `09abd85e0` W-KinWeight: a massa deixa de ser oferecida por KIND e passa a ser por quem a LÊ
- `54e4c6bb5` W-KinPush: o personagem cinemático empurra o que está ao lado dele
- `942985da8` W-KinPure: o terceiro modo — o mundo físico vira cenário
- `fb31706a6` W-KinCarry: a plataforma móvel era contada DUAS vezes no modo cinemático
- `17cb4017c` W-FloatFloor: o piso da perna é geométrico — medido, gateado, e NÃO há defeito
- `6dd4c2b1e` A lei da deriva de rampa NÃO tem termo em k — o eixo que faltava varrer
- `4561282da` O que um personagem CUSTA: +37% no cinemático, e o orçamento não é o teto
- `1937abfd2` A folga do pé: o item 5 respondido, e um defeito MAIOR achado e nomeado
- `da9c371e1` O item 4: as duas metades são UM número, e uma delas nunca dispara
- `086961715` O empurrão lateral sai do painel onde ninguém o lê, e o rodopio entra na fila
- `e93bd28c7` A §8.1 fechou: a absorção ganhou um PISO, e o snap voltou a viver
- `0c1ef02f6` O item 2: a deriva cinemática não tem termo em θ — e a rampa íngreme não escorrega
- `7ec74eb10` A §8.3: a absorção pede as DUAS respostas, e o Max Slope volta a significar o que diz
- `4d76b9954` A §8.2: o empurrão lateral entra no CENTRO — e o (2) foi medido e reprovado
- `d9cd37177` A tabela do §7 fecha: três linhas que diziam "a fixture da W2a" já tinham sonda
- `1c6dc41bc` A água não existe para o modo cinemático — medido, e METADE dela já atravessa
- `7c345729c` A água passa a existir para o modo cinemático — e a causa era o PAR, não a massa
- `f0b8c699a` A cena da água e o modo cinemático (`PH2D_PHYSICS_SMOKE=104`)

</details>

### 6.2 Determinismo

**`physics_ecs_c9` MOVEU, e é esperado:** a linha acrescentou lanes de player ao
`physics_ecs_c9/player.rs` (+248 linhas).

| | hash | corpos |
|---|---|---|
| `main` (registrado na integração de 08/08) | `dd5230d7…` | 108 |
| esta linha, **medido** | **`fb27f676170bd3abf45b539d3ccc9153c976c7fee02831460cbb18f96ac67365`** | **117** |

**debug ≡ release** nos dois. ⚠️ O número da **árvore combinada** é do integrador —
o `main` acima é o que a §5 do CLAUDE.md registra, não uma medição desta sessão.

⚠️ **E a última wave é byte-neutra no c9**: os pares novos do sensor
(`ActiveCollisionTypes::all()`) e o teto removido da razão de empuxo **não movem
pose nenhuma** — medido, `fb27f676…` antes e depois —, porque o `effector::apply`
recusa um corpo não-dinâmico antes de tocar nele.

### 6.3 O que smoke-testar

**Tudo abaixo foi APROVADO pelo Enio nesta jornada.** Um re-smoke pós-integração
confere que a árvore combinada não regrediu:

| cena | o que julgar |
|---|---|
| **`=101`** | o modo cinemático: caminhada, rampa, degrau. ⚠️ Os dois modos **repousam a alturas diferentes de propósito** (1,400 × 1,057 — um paira, o outro pousa) |
| **`=102`** | o **empurrão** lateral: o caixote é empurrado e **não rodopia** (75 rad → 0,00005) |
| **`=103`** | o **modo puro**: o mundo físico vira cenário |
| **`=104`** | a **água** (novo): três cápsulas idênticas, e o **azul tem de acompanhar o âmbar** |
| **`=100`** | o dinâmico na água — o **controle** de que a wave da água não o regrediu |

**Nada nesta linha está pendente de smoke.**

### 6.4 Mudanças de COMPORTAMENTO, nomeadas

1. **Um SENSOR passa a ver corpos cinemáticos** (`ActiveCollisionTypes::all()` só em
   sensor). Isto alcança **triggers** e **zonas**, não só a água: um gatilho que um
   player Snap atravessa passa a **disparar**. É a correção, e é o que *"um sensor
   nota coisas"* significa — mas é comportamento novo em toda cena com sensor.
2. **A razão empuxo/peso deixou de ser capada em `1`.** O consumidor antigo pergunta
   `> 0` e não muda; quem ler a magnitude vê a razão real. Um gate mudou de nome e
   de afirmação: `the_scale_tops_out_at_one_…` → `the_scale_is_the_density_ratio_…`.
3. **A absorção de gravidade do modo Snap pede as DUAS respostas** (§8.3): rampas
   caminháveis ficam **byte-idênticas**; 60° e 80° passam a **escorregar**, que é o
   `max_slope` a voltar a significar o que diz.
4. **A massa é oferecida por quem a LÊ, não por KIND** (§2).

### 6.5 Aberto, com o preço ao lado

- **`form_drag` não alcança a lei cinemática** — é um kernel por-aresta sobre o
  polígono do collider, não um escalar do meio.
- **A FORÇA de uma zona não leva um personagem cinemático.** Fica de fora **por
  desenho**: ela precisa do frame da zona, do espelho e do falloff, e re-derivá-los
  numa consulta seria uma **segunda resposta** para *"que empurrão esta zona dá
  neste ponto?"*. A fronteira é a que o **W-AreaFalloff** já desenhou (o falloff
  pesa os EMPURRÕES e deixa o MEIO em paz).
- **Nadar** — controlar a subida dentro d'água — é **produto**, não correção.
- **A paridade de arrasto com o dinâmico é APROXIMADA:** o solver amortece por
  SUB-PASSO e a lei uma vez por TIQUE (`(1+d·h)⁻⁴` contra `(1+d·4h)⁻¹`), a mesma
  classe que a W-AreaDrag mediu em 1,25%. Um corpo cinemático não tem sub-passo.
- **O player bobeia ~1,44 m numa poça** — nos **dois** modos (o dinâmico faz 1,4357
  contra 1,4394 do cinemático). É **anterior** a esta jornada e não é dela.

---

## 7. Verificação feita nesta linha

| | |
|---|---|
| `ph2d-physics` + `ph2d-physics-ecs` + `ph2d-platformer` | **970 / 970** |
| `ph2d-host-desktop` | **2.730 / 2.730** |
| `cargo fmt --all -- --check` | limpo |
| clippy (as 4 crates, `--all-targets`) | limpo |
| `physics_ecs_c9` | `fb27f676…`, 117 corpos, **debug ≡ release** |

**Resumo:** *Linha `physics` pronta (HEAD `f0b8c699a`, 21 commits). `PROJECT_SCHEMA`
69→70 (PROVISÓRIO, um degrau: `PlatformPlayer.reaction_push`) · 4 ids novos por hash
de string · cenas `=102`/`=103`/`=104` · registro, `ph2d-ecs`, contrato congelado,
ADRs e `Cargo.toml` **todos intocados** · c9 move (lanes novas) · nada pendente de
smoke. **Aguardo ordem de integração.***
