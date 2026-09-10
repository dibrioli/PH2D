# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-09-10

> **Os ciclos 3, 4 e 5 da dinâmica dos ciclos** ([doc 103](../103_dinamica_dos_ciclos.md)), os três
> com o smoke do dono aprovado, mais uma cena avulsa que o report dele pediu.
> ⚠️ **A linha FECHA e PARA** (`CLAUDE.md` §0.7). Integrar e shipar são ordem explícita do Enio,
> por um agente integrador dedicado.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/motion-value` |
| HEAD | `a200ee724` |
| merge-base com `main` | `39d48cd76` |
| commits | **56** |
| ficheiros | **166** |
| crates novas | **nenhuma** |
| ADR novo | **nenhum** ⇒ fora de toda disputa de número |

---

## §2 — O que a linha ENTREGA (uma frase por bloco)

**Ciclo 3 — TRANSFORMES & DEFORMADORES** (commits 1–36). O grupo inteiro passa a girar em torno de
**uma porta** (`ph2d_nodegraph::pivot`) em vez de seis vocabulários; o centroide, o espelho, o
cisalhamento, a fronteira curva e a direcção da dobra chegam ao **dispositivo**; e o
`motion.spline_wrap` deixa de nascer mudo. Tutorial: `tutoriais/03_transformes.pdf`, cena `=111`.

**Ciclo 4 — FOCO (os campos)** (37–45). O `motion.falloff` ganha **alça de canvas**, os cartões dos
campos espaciais passam a falar a mesma língua (`Placement`/`Falloff`), dois campos que ficavam
inertes em silêncio passam a avisar. Tutorial: `tutoriais/04_campos.pdf`, cena `=112`.

**Ciclo 5 — SIMULAÇÃO** (46–54). O `sim.collide` ganha alça (e é o primeiro nó cuja alça **muda com
um param**), o `Mode` vago vira **`Acts As`**, e três paredes de sliders ganham secções. Tutorial:
`tutoriais/05_simulacao.pdf`, cena `=113`.

**Cena `=114`** (55) — *peças que não se atravessam*: o `motion.collide` **dentro** de uma simulação
a correr, que nenhuma das duas cenas que o nó já tinha mostra.

**Fecho** (50, 52, 53, 54, 56) — os vermelhos que só o comando do `ship.sh` tinha; ver §6.

---

## §3 — ⚠️ SUPERFÍCIE PARTILHADA (saída do `collision-surface.sh`, não de memória)

```
SUPERFÍCIE DE COLISÃO — line/motion-value contra main
  merge-base 39d48cd76   ·   55 commit(s)   ·   166 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        123   (base: 123)
      └ tripla do gate               (123, 13, 22)   (base: (123, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              —   ·  ph2d-render 80 (base 80)  ·  ph2d-script 80 (base 80)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — último no disco 0169 · esta linha não cria ADR
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️ **PRAZO DE VALIDADE:** esta tabela mede contra o `main` de **2026-09-10**. O integrador
**re-roda** o script em cada worktree antes de fundir; a divergência entre as duas leituras é ela
própria um achado.

⭐ **Zero schemas movidos, zero registos, contratos intocados, nenhuma dependência externa nova.**
A única linha do `Cargo.lock` é uma **aresta interna** (`ph2d-node-motion-bezier-warp` como
dependência do agregador), não um pacote.

### §3.1 — Símbolos PÚBLICOS novos em crates partilhadas (o que o integrador grepa)

| símbolo | crate | forma |
|---|---|---|
| `pivot::{PivotMode, PARAM, LABELS, CENTROID_CX/CY, CENTROID_REDUCES, centroid_of}` | `ph2d-nodegraph` | **módulo novo**, aditivo |
| `KernelResolver::wgsl_shared` | `ph2d-nodegraph` | método de trait com **default `""`** ⇒ append-only |
| `codegen::ExtraBuffers<'a>` (+ `NONE`) | `ph2d-gpu-cook` | **struct novo**; `kernel_module`/`storage_buffers` mudaram de assinatura (23 sítios, todos nesta linha) |
| `codegen::map_module` | `ph2d-gpu-cook` | pub novo |
| `ParamPredicate`, `RequiredTextParam` | `ph2d-node-registry` | tipo + struct novos |
| `register_required_inputs` / `required_inputs` | `ph2d-node-registry` | side-table aditiva |
| `register_required_text_params` / `required_text_params` | `ph2d-node-registry` | side-table aditiva |
| `register_wgsl_shared` | `ph2d-node-registry` | side-table aditiva |
| `MODE_LABEL` | `ph2d-node-force-wind` · `-vortex` | const novo (o rótulo `Acts As`) |

⚠️ **O único de assinatura QUEBRADA é o `ExtraBuffers`** — `kernel_module` foi de 8 para 6
parâmetros. Se outra linha acrescentou uma chamada, ela **não compila** e a cura é mecânica
(agrupar `grid`/`reduces`/`luts` no struct). ⛔ Não é cosmética: eram 8 sobre um tecto de 7, e a
cura de um tecto é corte, nunca isenção.

### §3.2 — Números de cena (colidem por VALOR, não por símbolo)

`MAX_DEMO_LEVEL` **110 → 114**, com as cenas **`=111`** (ciclo 3), **`=112`** (ciclo 4),
**`=113`** (ciclo 5) e **`=114`** (o `Collide` na simulação). ⚠️ O gate
`no_two_smoke_scenes_claim_the_same_level` mede o **piso**, não o tecto: duas linhas que escrevam o
MESMO número fundem **mudas**. Se outra linha do Motion abriu cenas, **conte-as no roteador**.

### §3.3 — `.typos.toml` (ficheiro partilhado, na raiz)

Duas entradas: `"^transformes?$"` → `"(?i)^transformes?$"` e a palavra `transformes`. Ver §6.4.

---

## §4 — Contratos congelados (§6 do `CLAUDE.md`)

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool` intocados — confirmado
pelo `collision-surface.sh`. Todo canal novo é **side-metadata no registry**, que é a lei do módulo.

---

## §5 — Ordem, dependências e o que SMOKAR

Os 56 commits são **lineares e cronológicos**; não há ordem alternativa a preservar. Três blocos
dependem em cadeia (a porta do pivô → os adoptantes → as cenas), e o fecho (§6) vem por cima de tudo.

| smoke | estado |
|---|---|
| `tutoriais/03_transformes.pdf` + cena `=111` | ✅ **aprovado pelo dono** |
| `tutoriais/04_campos.pdf` + cena `=112` | ✅ **aprovado pelo dono** |
| `tutoriais/05_simulacao.pdf` + cena `=113` | ✅ **aprovado pelo dono** («SMoke OK») |
| cena **`=114`** (`Collide` na simulação) | ⏳ **NÃO SMOKADA** — nasceu depois da aprovação |

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_GPU_COOK_DEMO=114 cargo run -p ph2d-host-desktop --release
```

---

## §6 — ⛔⛔ O que o portão de fecho apanhou (leia isto antes de fechar outra linha)

**A causa é uma só, e a memória já a tinha escrita:** os ciclos 3, 4 e metade do 5 fecharam com
`cargo test --bins` e `cargo clippy -p <a crate em que eu estava>`. Ao correr enfim **a linha do
`ship.sh`**, apareceram vermelhos acumulados de dois ciclos — todos em código desta linha.

### §6.1 — Sete reds de clippy, e **dois não eram cosmética**

- `warp_gizmo_tests.rs`: `4,5 < bar && 3,5/√2 < bar` colapsava **dois controlos de grandezas
  diferentes** num só. A metade direita é logicamente implicada pela esquerda ⇒ o clippy dizia que
  não tinha efeito **e tinha razão**: o dia em que o losango deixasse de reprovar, o gate não se
  movia. *Um gate meio morto, nomeado por um lint.*
- `codegen::kernel_module` a **8** parâmetros sobre um tecto de 7 ⇒ `ExtraBuffers` (§3.1).

Os outros cinco: `ParamPredicate` (tipo complexo), duas closures redundantes, um `if` colapsável,
um empréstimo desnecessário, um `map` da identidade, uma linha em branco entre um doc e o item.

### §6.2 — Dois gates de ÁRVORE, que `--bins` nunca alcança

- **Tecto de LOC:** `ph2d-node-field-box/src/lib.rs` a **722 sobre 700**, vermelho desde a wave do
  ciclo 4. Cortado em `params_ui.rs` (545 + 192), o molde do `force.wind`.
- **Tofu:** três dos sete `→` acusados viviam em **legendas de canvas** (`Caption::new`), pintadas
  na fonte da UI — e a `Inter` não cobre o bloco das setas. ⛔ **As cenas `=111` e `=112` mostravam
  ao dono uma CAIXA VAZIA na legenda desde que existem.** Não era lint: era defeito de produto.

### §6.3 — O `fmt` da árvore

`cargo fmt --all` normalizou mais quatro ficheiros desta linha (kaleidoscope · mirror · twist ·
transform) que o `fmt --check` do ship teria acusado.

### §6.4 — O `typos`, com **três causas distintas**

O circunflexo ASCII (`tre^s`) era invenção minha, 4 sítios ⇒ ASCII simples. As **entidades HTML**
do meu tutorial fabricavam falsos positivos (`s&iacute;tio` → `tio`) e o irmão `04_campos.html` não
usa nenhuma ⇒ 248 entidades → 2. E o `transformes` **já estava isento com o alcance errado**
(ancorado em minúsculas, contra um título em maiúsculas) ⇒ `(?i)` **mais** uma entrada por PALAVRA,
porque a isenção por identificador não alcança um **nome de ficheiro**.

⚠️ **Há 5 ocorrências do circunflexo ASCII já no `main`**, de outras linhas
(`ph2d-editor-core`, `ph2d-panel-widget-lab`, `shells/desktop/tests/every_panel...`). **Não lhes
toquei** — são delas.

---

## §7 — Portão de fecho (batched, 1×)

| gate | resultado |
|---|---|
| `cargo fmt --all --check` | ✅ |
| `bash scripts/doc-index.sh --check` | ✅ 18 índices em dia |
| `typos` (project-wide) | ✅ |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | ✅ |
| `cargo test -p ph2d-host-desktop` (inclui `tests/`) | ✅ **5544 · 0 · 345** |
| `nextest` workspace | ver §7.1 |

### §7.1 — ⚠️ A ÚNICA reprovada é uma FLAKE DE CARGA já catalogada

`ph2d-node-motion-soft-body :: the_shape_match_is_linear_in_the_mesh` — **membro confirmado** da
família do `CLAUDE.md` §5.0 (registada em 2026-09-01). As três assinaturas:

1. **Zero linhas do diff desta linha naquela crate** (`git diff --name-only main..HEAD -- crates/ph2d-node-motion-soft-body` ⇒ `0`).
2. Reprovou numa corrida a **`load 69`** (o clippy da workspace ainda a esvaziar).
3. Confirmação isolada na máquina calma: ver a corrida colada abaixo.

⚠️ **E a corrida cancelou no 1.º ✗, escondendo 6942 testes** — a lei do §5.0. A corrida de
confirmação leva `--no-fail-fast`.

```text
== a acusada, SOZINHA, 5x, com a carga ao lado ==
  corrida 1 · load 15.08 · test result: ok. 1 passed; 0 failed
  corrida 2 · load 15.08 · test result: ok. 1 passed; 0 failed
  corrida 3 · load 13.95 · test result: ok. 1 passed; 0 failed
  corrida 4 · load 13.95 · test result: ok. 1 passed; 0 failed
  corrida 5 · load 13.95 · test result: ok. 1 passed; 0 failed

== a suíte INTEIRA, --no-fail-fast ==
     Summary [ 579.807s] 22248 tests run: 22248 passed (5 slow), 2205 skipped
```

⚠️ **A máquina NUNCA desceu abaixo de `load ~5`** — o esperador desistiu ao fim de 30 minutos a
`16,22`, e as cinco corridas de confirmação correram a `13,9`–`15,1`. ⛔ Colar «5 de 5 verde» sem
esta linha seria exactamente o defeito que o §5.0 nomeia (*a régua que desmente a flake ser a
própria flake*) — só que **ao contrário**: aqui o risco não existe, porque um VERDE sob carga alta é
inequívoco (a carga fabrica reprovações, não aprovações).

⭐ **E o que resolve não são as cinco corridas: é a suíte inteira.** `22 248 de 22 248`, com a
acusada dentro. A reprovação da corrida cancelada foi transiente.

⚠️ **Causa da máquina nunca acalmar (não é desta linha):** dois órfãos da `line/Vector`
(`ph2d_poly2d`, reparentados ao init, 33 e 37 min a ~270 % de CPU cada) e um da
`line/3DModeling` (`the_census_of_every_primitive`). É o mesmo bloqueio que impede a medição do §9.1.

---

## §8 — O que só o `ship.sh` apanha (o gate de integração não corre)

- **`cargo machete`** — a linha não acrescentou dependência externa nenhuma (`Cargo.lock` sem
  `+name`), mas o `ph2d-gpu-cook/Cargo.toml` foi tocado.
- **`cargo deny` / `audit`** — sem deps novas ⇒ risco baixo, mas não corrido aqui.
- **`typos`**, **`fmt`**, **clippy `-D warnings`** — corridos (§7), e a §6.4 explica porquê isso
  não era garantido.

---

## §9 — O que fica ABERTO (para o §5 do `CLAUDE.md`, não para esta linha)

1. ⏳ **A MEDIÇÃO do ciclo 5 (passo 5) não tem relógio.** A residência está feita e vale sob
   qualquer carga — **10 dos 12 nós do grupo são reivindicados pelo dispositivo** a 102 400
   objectos —, mas o relógio **não foi medido**: a corrida saiu a `load 13,06` contra o tecto de
   `~5` do §5.0, depois de 25 minutos à espera de calma. ⛔ A tabela que a corrida imprimiu **não**
   entra em doc nenhum. Causa: dois órfãos da `line/Vector` (`ph2d_poly2d`, reparentados ao init,
   33 e 37 min a ~270 % de CPU) e um da `line/3DModeling`.
2. ⏳ **A cena `=114` não foi smokada** (§5).
3. ⭐⭐ **DECISÃO DO DONO, 2026-09-10 — o colisor vai para a SHAPE.** Perguntado se não era
   contra-intuitivo pôr o `Collide` na linha da simulação sem referência à forma, ele decidiu:
   *«Vou preferir colocar na shape.»* ⚠️ **Eu levantei duas objecções e ele reafirmou a direcção**,
   então ela é a ordem. As objecções ficam **registadas, não vencidas**:
   - a fonte nem sempre é uma forma (grade, espalhamento, emissor, texto, L-System, tabela) ⇒ um
     botão no `source.shape` responde por **um** de ~20 produtores;
   - colidir é uma operação **entre N coisas**, logo a *colisão* continua a ser um nó mesmo que a
     *declaração* do colisor mude de dono.
   ⭐ **E a medição que sustenta a decisão dele:** o `motion.collide` transforma tudo num **círculo**
   de raio `radius × max(|size.x|,|size.y|)` — uma vareta 4:1 reserva um círculo do **lado longo**;
   e o nó irmão `sim.collide` **já** pergunta de onde vem o raio (`Radius From: Point · Fixed ·
   Sprite Size`), com o doc a chamar-lhe *«a ÚNICA porta»*. O `motion.collide` **não pergunta —
   assume**. ⇒ *dois nós do mesmo grupo, a mesma pergunta, e só um deixa o artista respondê-la.*
4. ⏳ **Colisor com o CONTORNO da forma não existe em lado nenhum** — nem no Motion (plano · disco ·
   taça · caixa) nem na física de corpos rígidos (bola · caixa · cápsula, com *«Triangle/Polygon
   land later»* escrito no código). O mecanismo de levar geometria a um nó existe e é usado duas
   vezes (`field.shape`, o `obstacle` do `motion.boids`) — colidir contra polígono é trabalho, não
   fiação.
5. ⛔ **W1b do ciclo 5, com o preço medido:** `force.vortex` e `force.attractor` põem-se no mundo e
   **não têm param de ângulo** ⇒ uma alça deles teria uma argola de rodar inerte. A cura é a
   `GizmoView` saber **suprimir** a argola: **28 sítios de construção**, em crates de outras linhas.
   O `rotation` da spec já é `Option`, então o dia em que ela souber, entram com um braço cada.
6. ⏳ **`project-memory/MEMORY.md` está acima do tecto** (29,4 KB contra 24,4) e é truncado ao
   arranque de cada sessão. Toda linha escreve nele ⇒ é decisão de coordenação, não desta linha.

---

## §10 — Limpeza e o binário do smoke

### `incremental/` reclamado (item 7 da §1.5.9)

```
antes:  30G  target/debug/incremental
depois: (nada)
```

⭐ **30 GB devolvidos** por esta worktree. Risco zero (o cargo recria), sem ship.

### O binário do smoke, COMPILADO (item 9 da §1.5.9)

```
1ª corrida: Finished `release` profile [optimized] target(s) in 0.40s
2ª corrida: Finished `release` profile [optimized] target(s) in 0.18s
linhas "Compiling" na 2ª: 0
target/release/ph2d-host-desktop — 67 629 328 bytes
```

⚠️ **E a prova NÃO é o `Finished` nem o mtime** — é o binário conter uma **string do código de
hoje** ([memória](../../../project-memory/feedback_a_probe_that_waits_on_pgrep_catches_the_other_worktrees_compiler.md)):

| string procurada | ocorrências |
|---|---:|
| `PECAS QUE NAO SE ATRAVESSAM` (a cena `=114`) | 1 |
| `DEIXAR A FISICA DECIDIR` (a cena `=113`) | 1 |
| `Acts As` (o rótulo da W2) | 4 |
| `arraste ESTE bloco` (a legenda da `=114`) | 1 |

⇒ o dono corre qualquer smoke desta linha **sem esperar build**.

### O comando, inteiro e copiável

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env PH2D_GPU_COOK_DEMO=114 cargo run -p ph2d-host-desktop --release
```

---

## §11 — A linha PAROU

Working tree limpo, `56` commits, nada por commitar. ⛔ **Esta linha não integra nem pusha**
(`CLAUDE.md` §0.7) — a integração é ordem explícita do Enio, por um agente integrador dedicado
munido deste documento.

