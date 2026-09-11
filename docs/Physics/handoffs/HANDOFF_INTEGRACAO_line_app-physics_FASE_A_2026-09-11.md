# HANDOFF DE INTEGRAÇÃO — `line/app-physics` (W2/L2, **Fase A**, 2026-09-11)

**Status:** Fase A FECHADA · a linha **não integrou e não pushou** · aguarda ordem do Enio.
**Fase B (o corte pelo HOWTO da L0) NÃO começou** — ela depende de a `line/app-host` integrar.

> Para o **agente integrador**, DIRETRIZ §1.5.9. Tarefa: [`BRIEFINGS_W2`](../../IntegracaoMultiAgente/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md) §5-A.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/app-physics` |
| HEAD | `c5a7e4f5c3be014ce648fd613402c9254eb7a80a` |
| merge-base | `8fa4f115bbdedb7635581af8c528a8aedd5c8d74` |
| commits | 7 |
| diff | 214 ficheiros, +9 606 / −9 195 |

⚠️ **O diff parece pequeno para 22 k linhas movidas porque o git detecta as
renomeações** — 108 dos 214 ficheiros são `R` (rename), com 0 linhas de mudança.

---

## §2 — O que a Fase A entregou, em números

### Os quatro itens do §5-A

| item | pedido | entregue |
|---|---|---|
| **A1** poda das cenas | apagar as não citadas | **0 apagadas** — nenhuma existe (§4) |
| **A2** desfazer o acoplamento | `impl App` → função livre; campos → um estado | `impl App` **88 → 13 blocos**; 7 campos → **1** |
| **A3** agrupar | `physics_*.rs` → `src/physics/` | 37 ficheiros, `main.rs` −19 `mod` |
| **A4** a crate nasce | o que só depende do módulo + ECS | **`ph2d-app-physics`, 112 ficheiros, 22 546 LOC** |

### A família, antes e depois (mesma definição de família nos dois lados)

> Definição usada: `shells/desktop/src/{physics_*,joint_*,body_{fk,grab,pose}*,player_input*}.rs`
> \+ `render_loop/{physics_*,inspector_physics*,inspector_joint*,inspector_player*,measure_player_tape}.rs`.
> ⚠️ **Ela é MAIOR que a do censo do briefing** (que mediu 94 ficheiros / 22 783 LOC): o briefing
> contou só o topo de `src/`, e a família tem 64 ficheiros dentro de `render_loop/`.

| | ANTES | DEPOIS (na shell) |
|---|---:|---:|
| ficheiros da família na shell | 208 | **103** |
| ficheiros de produto | 116 | **49** |
| ficheiros com `impl App` | 86 | **12** |
| blocos `impl App` | 88 | **13** |
| campos de `App` tocados | 21 | **15** |
| LOC da família na shell | 51 663 | **29 507** |

### A shell

| | ANTES | DEPOIS |
|---|---:|---:|
| LOC de `shells/desktop/src` | 493 252 | **470 963** (−22 289, −4,5 %) |
| ficheiros `.rs` | 1 784 | **1 678** |
| declarações de `mod` no `main.rs` | 568 | **498** |

---

## §3 — Foundational / partilhado tocado, e porquê

| ficheiro | o quê | porquê |
|---|---|---|
| `shells/desktop/src/app_state.rs` | −7 campos, +1 | os sete da família viram um `PhysicsState` |
| `shells/desktop/src/main.rs` | −70 `mod`, +2 | 71 módulos saíram; `mod physics;` + `mod physics_state;` |
| `shells/desktop/src/input_dispatch.rs` | 13 linhas | `self.X` → `self.physics.X` |
| `shells/desktop/src/render_loop/mod.rs` | 13 linhas | idem |
| `shells/desktop/src/render_loop/snapshots.rs` | 1 comentário | idem |
| `shells/desktop/src/signal_smoke.rs` | 1 linha | a cena que ele chama mudou de crate |
| `shells/desktop/src/instance_smoke.rs` | 1 linha | `spawn_floor` mudou de crate |
| `shells/desktop/Cargo.toml` | +1 dep | `ph2d-app-physics` |
| `Cargo.lock` | +1 aresta interna | idem |
| **14 docs** | 20 citações de caminho | os ficheiros mudaram de sítio |

⛔ **Nenhum contrato congelado (§6) foi encostado.** Nenhum ADR criado.

---

## §4 — A poda das cenas: ZERO, e o método é o achado

O §5-A.1 manda apagar cada cena que **nenhum doc cita pelo número**. Medido:

- o roteador oferece **117 níveis** (`2..119`, sem o `84`, que não existe de propósito);
- citados por pelo menos um doc vivo: **117**;
- **a apagar: 0**.

⚠️⚠️ **E o censo que o briefing sugere apagaria 28 cenas VIVAS.** O comando do briefing é

```
grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE=[0-9]+' docs CLAUDE.md project-memory .claude
```

mas os docs de física citam cena quase sempre no **atalho** — *«cena `=111`»*, *«smoke `=13`»* —,
e não na forma da env. Medido: **28 dos 117 níveis são citados SÓ pelo atalho**, entre eles a `=114`
(Brake), a `=82` (autoria do player) e a `=86` (a fita). O censo tem de ser a **união** de:

1. `PH2D_PHYSICS_SMOKE=<n>` em qualquer doc vivo;
2. `` `=<n>` `` dentro de `docs/Physics/**`;
3. `` `=<n>` `` em `project-memory/` e `.claude/`.

⇒ *uma leitura só fabrica a lista de dívida* — a mesma lei que o L-System pagou (CLAUDE.md §5.1).
**Passe isto às outras quatro famílias:** a forma do atalho é do repo todo, não da física.

---

## §5 — O desenho: o que uma cena PEDE à shell é DADO

As 101 cenas convertidas são `pub fn(ctx: &mut SceneCtx)`. O `SceneCtx` traz o `World` e as
definições vigentes; o que a cena quer da shell (câmera, painéis, selecção, definições) viaja num
`SceneSetup` que **o roteador aplica depois** de ela correr — ADR-0075 ao pé da letra.

⛔ **Porque NÃO um trait `SceneHost`:** ele obrigaria a crate a declarar a superfície da shell
(câmera, painéis, gizmo) e a shell a implementá-la — o acoplamento mudava de **forma**, não de
tamanho. Um registo de intenções é inspeccionável e testável sem janela.

⚠️ **Isto NÃO é a porta que a L0 está a construir** e não pretende ser: é vocabulário **interno da
família**, todo em tipos que já eram do módulo (`&'static str` para a chave de painel,
`ph2d_render::Camera2d`, `ph2d_physics_ecs::PhysicsSettings`). Se o `ph2d-app-host` trouxer a sua
própria porta, o `SceneSetup` **colapsa nela** — são 5 campos e um `apply` de 20 linhas.

---

## §6 — O que SÓ o substrato (L0) resolve — **a lista que o briefing pediu**

As **17 cenas** e os **12 ficheiros** que ficaram na shell, com o que **exactamente** os prende.
⛔ Não desenhei nenhuma porta para isto (regra B do MODELO).

| o que prende | cenas / ficheiros | o que é preciso |
|---|---|---|
| **a TIMELINE** | `physics_smoke_player` (5 cenas: `=81`,`=80`,`=83`,`=85`,`=82`) · `physics_smoke_joint_anim` (`=78`) | escrever tracks num `TimelineDoc` (`author_platform_track`, `author_joint_anim_tracks`). ⭐ **É módulo IRMÃO, não a shell** — `ph2d-timeline` é uma crate; a decisão é se a crate da família pode depender dela |
| **o PLAYHEAD** | `physics_smoke_rigs` (5 cenas: `=6`,`=7`,`=8`,`=11`,`=37`) | `playhead.set_loop(a, b)` — um par de `f32` |
| **o READOUT** | `physics_smoke_out` (`=113`) | escrever `physics.player_readout_log` (é campo MEU; sai de graça quando o estado da família viajar) |
| **o INSPECTOR da roldana** | `physics_smoke_pulley_{comp,diff,tackle}` · `physics_smoke_part` | os **gates** deles dirigem `render_loop::inspector_joint_wheel` (`add_pulley_wheel`, `wheel_with_edit`, `set_wheel_mount`) e `inspector_physics_tests::apply` |
| **o gesto do PONTEIRO** | `joint_draw` · `joint_anchor_drag{,_stop,_wheel}` · `joint_rig{,_drag}` · `body_{fk,grab,pose}` · `player_input` | `last_pointer`, `modifiers`, `input_actions`, `any_input_this_frame` |
| **a própria `App`** | `physics_smoke` (o roteador) · `physics_smoke_base` (5 cenas base) | o roteador segura a `App`; o `physics_smoke_author` mexe no Inspector |
| **o `render_loop`** | os 64 ficheiros de `render_loop/physics_*` e `inspector_*` | overlay e inspector — são a metade-shell por natureza |

⭐ **A lista mais curta de todas:** dos sete prendedores, **três são baratos** (o playhead é um par de
`f32`; o readout é um campo meu; a timeline é uma crate irmã). Se a L0 resolver só a timeline, mais
**6 cenas** saem sem desenho novo.

---

## §7 — Símbolos novos que podem COLIDIR

```
crate ph2d-app-physics            (pacote novo; glob `crates/*`, ZERO edição no Cargo.toml da raiz)
ph2d_app_physics::SceneCtx        (struct nova)
ph2d_app_physics::SceneSetup      (struct nova)
ph2d_app_physics::common::{spawn_floor, spawn_player, slab}
crate::physics_state::PhysicsState        (struct nova, shells/desktop)
App::physics                      (campo novo; SUBSTITUI 7 campos — ver §3)
App::run_physics_scene            (método novo, privado)
shells/desktop/src/physics/       (pasta nova, com mod.rs)
gate outside_frame_has_no_production_caller   (teste novo)
```

**`collision-surface.sh`, corrido nesta worktree em 2026-09-11:**

```
SUPERFÍCIE DE COLISÃO — line/app-physics contra main
  merge-base 8fa4f115b   ·   7 commit(s)   ·   214 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 1 pacote novo: "ph2d-app-physics"   (aresta INTERNA, não externa)
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⭐⭐ **ZERO contador partilhado se move** — é o que o §6 das notas do integrador exige de uma wave de
extracção: *mover código não muda serialização; se uma linha o subiu, ela fez mais do que a tarefa.*

⚠️ **PRAZO DE VALIDADE:** esta tabela mede contra o `main` de **11/09**. Re-rode
`collision-surface.sh` nesta worktree **imediatamente antes de fundir**, e lê o `PROJECT_SCHEMA` no
ficheiro, nunca na coluna «base» (que é o merge-base).

---

## §8 — A PROVA (§3 do briefing)

**a) Nenhum teste se perde.** `cargo nextest list --workspace --cargo-profile ci-test`, antes e depois,
comparados por `scripts/nextest-list-diff.py`:

```
antes: 22635 testes (22011 chaves) | depois: 22636 (22012)
MOVED (mesma chave, outro pacote/binário): 109
ONLY-A (perdidos): 0          ← a barra
ONLY-B (novos): 1
   + outside_frame_has_no_production_caller  (em ['ph2d-app-physics::it'])
```

**b) Nenhuma cena se perde.** Níveis do roteador: **117 antes, 117 depois, conjunto idêntico**
(diff vazio). Os gates `handle_scenes_start_paused` e
`scenes_that_ask_for_the_ruler_open_the_timeline` continuam a correr — e passaram a varrer **as duas
pastas** (§9.4).

**c) A shell encolheu.** `shells/desktop/src`: **493 252 → 470 963 LOC** (−22 289, −4,5 %);
**1 784 → 1 678** ficheiros; `main.rs` **568 → 498** `mod`. *(Números exactos e independentes de carga.)*

**A unidade a frio, num `target/` novo** (`cargo check -p ph2d-host-desktop --tests --timings -j 32`):

| unidade | corrida A | corrida B |
|---|---:|---:|
| **`ph2d-host-desktop bin (check-test)`** | **15,88 s** | **15,87 s** |
| `it test (check-test)` | 1,37 s | 1,36 s |
| workspace inteiro (parede) | 46,57 s | 46,96 s |
| **`load` no arranque → fim** | `23,5 → 16,5` | **`5,1 → 7,6`** |

⭐⭐ **As duas corridas concordam a `0,01 s` com a carga a variar `4×`** — e isso é um facto sobre
**esta** unidade, não sorte: o front-end da shell é **monotarefa** (auditoria §3.2), então ele não
disputa núcleos com mais nada e a régua do `load ~5` do §5.0, que governa gates de RAZÃO, não morde
aqui. *Um relógio de parede de um build paralelo flaka sob carga; o de uma unidade de um thread só
não.*

⛔ **O que NÃO tenho é o «antes» no mesmo par**, e os `16,8 s` da auditoria §4-C2 **não** o
substituem: aquilo foi medido sobre a árvore de **antes das seis linhas da W2**, noutro dia.
Comparar `15,87` com `16,8` mistura duas variáveis. ⇒ *a comparação honesta desta linha é a de
**LOC**, que é exacta; para o relógio, o integrador tira as duas pontas com um
`git checkout <merge-base>` num target novo — 47 s cada.*

**d) O gate de fecho.**

| gate | resultado |
|---|---|
| `scripts/nextest-impacted.sh` | **5 679 testes, 5 679 passam** (load 64,3) |
| `cargo clippy -p … --all-targets` | **0** |
| `cargo fmt --all -- --check` | limpo |
| `scripts/doc-index.sh --check` | 19 índices em dia |
| tetos de LOC | verdes — **por corte, zero isenção nova** (§9.5) |
| suíte da crate nova | **127/127** |

**e) O smoke do Enio** — ver §11. ⏳ **Não foi corrido por mim** (é dele, §0.7); o binário fica
compilado.

---

## §9 — ARMADILHAS MEDIDAS — o que uma leitura rápida do diff entende ao contrário

### 9.1 ⛔⛔ Apagar uma linha `mod X;` deixa o `#[cfg(test)]` dela ÓRFÃO — e ele RE-LIGA-SE ao `mod` seguinte

A primeira remoção das 71 declarações deixou para trás um `#[cfg(test)]` cujo `mod` tinha ido embora.
Ele colou-se ao vizinho: **`mod physics_smoke_joint_anim;` ficou `#[cfg(test)]`**, a cena `=78`
desapareceu do build de produto, e o único sintoma foi *«no method named
`physics_smoke_joint_anim`»* a **200 linhas de distância**, num ficheiro que ninguém tinha tocado.
⇒ **a remoção tem de levar o bloco de atributos/doc que a precede.** Foram 3 linhas órfãs.
*Vale para as outras quatro famílias, todas elas a apagar `mod` do mesmo `main.rs`.*

### 9.2 ⛔ `include_str!` com caminho relativo — a armadilha que o briefing mandou procurar, e ela mordeu DUAS vezes

`the_player_smokes_name_the_key_that_jumps.rs` tem **três** `include_str!` para cenas de player: dois
alvos foram para a crate e um para a pasta. É falha de **compilação** (o modo bom) — mas só no alvo
`test "it"`, que um `cargo check -p` da lib **não alcança**.

### 9.3 ⛔⛔ Doze leituras de caminho em RUNTIME, que compilam e só falham ao correr

`fs::read_to_string("src/physics_smoke.rs")` e irmãs, em 6 gates. E **uma delas eu reescrevi
ERRADO**: prefixei `physics/` a uma cena que tinha ido para a **crate**
(`a_smoke_scene_ships_the_default_tuning::SCENE`). Quem o disse foi o **controlo positivo do próprio
gate** (*«o scanner lê a cena que diz ler»*), não a minha varredura — que reportou sucesso.

### 9.4 ⛔⛔ Gates que VARREM `src/` por nome de família ficam CEGOS a metade, sem reprovar

`handle_scenes_start_paused` e `scenes_that_ask_for_the_ruler_open_the_timeline` liam
`read_dir("src")` e colhiam `physics_smoke*.rs`. Com a família em **duas** pastas, um varredor de uma
só devolve um conjunto **menor** — e um conjunto menor **passa**. Os dois varrem as duas agora.

⚠️ **E o parser de braços do roteador tinha o mesmo modo de falha, uma camada acima:** ele lia
`self.NOME()` e, na forma nova (`self.run_physics_scene(crate::…::NOME)`), colhia **`run_physics_scene`**
— o nome do INVÓLUCRO — para as 101 cenas. O conjunto ficava vazio, e *um conjunto vazio lê-se como
«está tudo bem»*. Só o **controlo positivo** (`the_needles_can_match_something`) o acordou.

### 9.5 ⛔ O teto de LOC do roteador estourou por CRESCIMENTO DE CHAMADA, não por feature

`physics_smoke.rs` foi a **762 LOC** porque cada braço passou de `self.cena()` para
`self.run_physics_scene(crate::…::cena)` e o `rustfmt` partiu metade deles em 3 linhas. A cura é
**corte por responsabilidade** (o roteador · as cenas base), nunca uma entrada no `FILE_OVERAGE_OK`.

### 9.6 ⛔⛔ Um gate TEXTUAL que compara uma LINHA INTEIRA afirma sobre a FORMATAÇÃO

O `PhysicsState` fez o nome crescer 4 caracteres; o `rustfmt` partiu
`self.interaction.joint.drag_reach(self.modifiers.alt_key())` em **seis** linhas; e o gate reprovou
sobre wiring que não mudou uma vírgula. Ele achata o espaço em branco agora — **e a asserção
NEGATIVA também**, porque uma quebra de linha entre o `!` e o `self` fá-la-ia evaporar, e *o modo de
falha de um gate negativo é ficar VERDE*.
⚠️ **Ao todo foram 10 gates textuais** partidos pelo rename: 6 num commit, 4 apanhados só pelo
`nextest-impacted` (a corrida filtrada não os via).

### 9.7 ⛔⛔ O `#[cfg(test)]` do `outside_frame` NÃO atravessa a fronteira da crate

Ele dizia, por escrito, porque existia: *«um `pub(crate)` sem chamador de produto é uma SEGUNDA
RESPOSTA esperando alguém»*. Mas `cfg(test)` significa *«quando se compilam os testes DESTA crate»*, e
os dois chamadores dele ficaram na shell. Mantê-lo tornava a função invisível a quem a chama.
⇒ o `cfg` sai e a propriedade passa a ser afirmada por um **instrumento**: o gate
`outside_frame_has_no_production_caller`, **com prova de mutação** (um chamador de produto que
compila ⇒ RED, nomeando `ficheiro:linha`; restaurado ⇒ GREEN).

### 9.8 ⛔ Um `cargo clippy` corrido ANTES do último corte não descreve a árvore que se entrega

Eu corri o clippy, curei o único achado, e **só depois** parti o roteador em dois pela cura do teto
de LOC — o que deixou **cinco `use` órfãos** no ficheiro que perdeu as cinco cenas. O clippy que eu
já tinha «verde» era de uma árvore que já não existia; quem os apanhou foi a **build fria do
`--timings`**, corrida para outra coisa. ⇒ *o gate de fecho corre-se sobre o ÚLTIMO commit, não
sobre o penúltimo* — e um `cargo check` incremental quente não reproduz um `warning` de import
órfão que o cold build mostra.

### 9.9 ⛔⛔ Reapontar citações de doc pelo NOME do ficheiro corrompe 76 delas em silêncio

A 1.ª redacção mapeava `mod.rs → shells/desktop/src/physics/mod.rs`, e `mod.rs` existe em dezenas de
módulos: **`render_loop/mod.rs` virou `physics/mod.rs` em 76 sítios**, e o script imprimiu *«80 docs
reapontados»* com ar de sucesso. A chave tem de ser o **caminho original completo**. Quem o apanhou
foi ler o diff.

---

## §10 — Premissas que a medição derrubou

1. **«A física é a mais acoplada: 22 `impl App`»** (briefing §2). São **88 blocos em 86 ficheiros** —
   o censo do briefing contou só `^impl App` no topo de `src/`, e 74 dos 86 escrevem `impl crate::App`.
   ⭐ E o acoplamento era **raso**: 75 dos 86 tocavam **só** `self.gfx`.
2. **«126 membros de `self.`»**. São **140 nomes distintos**, mas **119 são MÉTODOS** (o roteador a
   chamar as cenas). Campos de `App` a sério: **21**, dos quais **16** existem mesmo em `App` e
   **7** eram só da família. *Contar `self.x` sem separar campo de método mede o tamanho do `match`.*
3. **«O corte é mover ficheiros»**. Os 75 ficheiros puros precisavam de **três** coisas além do
   `World` — câmera (21), painéis (30), definições (2) — e foi isso que obrigou ao `SceneSetup`.
4. **«Uma cena não citada apaga-se»**. Nenhuma existe nesta família, e o censo ingénuo teria
   apagado 28 vivas (§4).
5. **Eu escrevi que o `physics_smoke_rig` podia ir para a crate.** Não podia: o teste dele usa
   `crate::joint_rig`, que fica na shell. Voltou — e a **regra** que daí saiu é a que governou o
   resto: *uma cena fica se algo shell-held depender da superfície de TESTE dela.*

---

## §11 — O que smoke-testar (o comando, inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-physics && env PH2D_PHYSICS_SMOKE=22 cargo run -p ph2d-host-desktop --profile smoke
```

**A extracção não muda produto** — a barra é *comportamento idêntico*. As cenas que mais exercitam o
caminho novo, uma de cada tipo de pedido:

| cena | o que ela prova do caminho novo |
|---|---|
| `=22` | o mais simples: só povoa o mundo (as 44 assim) |
| `=67` | **selecção** — a cena escolhe o torso; o gizmo tem de abrir nele |
| `=108` | **painel** — tem de abrir o painel de física sozinha |
| `=15` | **câmera** enquadrada + ⚠️ as paredes CINEMÁTICAS (§5 do CLAUDE.md — **não a «arrume»**) |
| `=106` | **definições de mundo** (`set_settings`) |
| `=78` · `=113` | as que **ficaram** como método (timeline · readout) — têm de continuar iguais |

⚠️ **Rode também SEM a env var** — o app tem de abrir exactamente como antes.

**Smoke compilado (DIRETRIZ §1.5.9 item 9)** — 2.ª corrida, byte a byte o mesmo comando:

```
$ cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.18s
```

**zero linhas `Compiling`**; binário em `target/smoke/ph2d-host-desktop` (78 MB, 17:54).
⚠️ Ele está quente **nesta worktree** — que é a do `cd` do comando acima, e não a do primário.

---

## §12 — O que só o `ship.sh` apanha

- `cargo machete` sobre a **dependência nova** (`ph2d-app-physics` no `Cargo.toml` da shell) — ela
  **é** usada, mas é a primeira corrida dela;
- `cargo deny` / `audit` sobre o `Cargo.lock` com o pacote novo (aresta interna, risco ~0);
- `typos` sobre os doc-comments novos, que são **densos e em português**;
- a matriz 3-OS e o `physics_ecs_c9` — ⚠️ **esta wave não toca solver nem serialização**, então o
  risco real é zero, mas o hash só é comparado entre OS no CI.

---

## §13 — Ordem entre os commits

Linear e cada um verde por si:

1. `fc96cf555` os 7 campos → `PhysicsState`
2. `546c4239c` 101 cenas → função livre (+ a crate nasce)
3. `d30ae7d7e` o corte: 108 ficheiros saem
4. `320c0544f` os 6 gates textuais
5. `6f169a2d8` o que resta agrupa-se em `src/physics/`
6. `7151741bc` as 20 citações de doc
7. `c5a7e4f5c` o portão de fecho (teto de LOC por corte + 4 gates)

⚠️ **Nenhum deles é opcional** — o 2 sem o 3 deixa a crate sem cenas; o 4, 6 e 7 são os vermelhos que
os anteriores criaram.

---

## §14 — A FASE B, quando a L0 integrar

1. `git rebase main`
2. Ler o `HOWTO_partir_uma_familia_da_shell.md` **inteiro**
3. O corte: `shells/desktop/src/physics/` → a crate, pelo trait de `ph2d-app-host`; registar a
   família em `ph2d-app-registry-init`
4. ⛔ **Se o HOWTO não cobrir a lista do §6 — PARAR e reportar.** A extensão do substrato é decisão
   do integrador.

⏳ **Aberto e nomeado, além do §6:**
- os **64 ficheiros de `render_loop/`** da família não foram tocados (overlay + inspector);
- os nomes na crate mantêm o prefixo `physics_smoke_` (`ph2d_app_physics::physics_smoke_damping`),
  redundante mas **de propósito**: renomear multiplicaria a reescrita de citações e de `#[path]`.
  Limpeza de Fase B;
- a unidade `bin (check-test)` a frio, por medir (§8c).
