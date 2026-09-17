# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-09-16

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.9). A linha **fechou e parou**: não integrou,
> não fez ship, não tocou no `main` ([`CLAUDE.md §0.7`](../../../CLAUDE.md)).
>
> ⚠️ **Leia o §6 antes de fechar outra linha** — o portão desta apanhou um teto de LOC que só o
> mapa de colisão vê, e um vermelho de GPU que **o próprio teste já documentava**.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/motion-value` |
| HEAD | `b9708b09f` (+ os commits deste fecho) |
| merge-base com `main` | `1d43da737` |
| commits | **72** |
| ficheiros tocados | **282** |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |

---

## §2 — O que a linha ENTREGA (uma frase por bloco)

**1. ⭐⭐⭐ O COLISOR VAI PARA A FORMA** (doc 109 — ordem do dono, 13/09: *«vou preferir colocar na
shape»*). A peça **declara** o colisor (coluna `collider`, secção *Collision* no cartão), o `sim.step`
resolve o contacto por uma crate-folha nova (**`ph2d-contact`**), o `sim.collide` pousa pelo colisor
declarado (`Auto`), o gizmo desenha o contorno em todas as peças **por cima da arte**, e a peça ganha
**material** (atrito e restituição — é o atrito que RODA um círculo). Cenas **`=114`** e **`=115`**.

**2. ⛔ A PILHA: quatro medições que mudaram o desenho, e uma cura REPROVADA pelo dono.** O motor de
contacto com memória ficou **encomendado** (spec), a família *«afinar o encosto»* morreu na medição, e
a minha cura do tremor foi **revertida por veredito dele** — o termo que eu tirara segurava outra coisa.
⚠️ Um defeito real foi curado no meio disso: *o binário da normal viajava na MOEDA ERRADA* e a pilha
zumbia (doc 109 §8).

**3. ✅ CICLO 6 — VALOR & PULSO** (doc 110, cena `=116`/`=117`, [tutorial 06](../tutoriais/06_valor_e_pulso.pdf)).
⭐⭐ **Um param DIRIGIDO deixou de derrubar o nó para a CPU** (`gpu-cook` W1a) e a família `pulse.*`
fecha **9 de 9** no dispositivo, com o `dt` a virar *uniform*. O placar lia `0 aberto` sobre 29 de 35
nós porque a régua era a errada (a certa é a LINHA do cartão).

**4. ✅ CICLO 7 — APARÊNCIA** (doc 112, cena `=118`, [tutorial 07](../tutoriais/07_a_cor_e_o_rasto.pdf)).
Os **dez** nós ficam na placa; a medição achou e curou um nó em série escrito na própria wave
(`motion.slit_scan`, **`16,33 → 3,49 ms`** a um milhão). O `Delay By` (Order|Field) substituiu a
receita refutada de *«ponha um sort antes»*.

**5. ✅ CICLO 8 — FONTES & DADOS** (doc 113, cena `=119`, [tutorial 08](../tutoriais/08_de_onde_vem_as_coisas.pdf)).
⭐⭐⭐ **A costura de uma fonte deixou de ser taxa por quadro**: `9,62 → 1,62 ms` a um milhão de linhas,
por **uma régua só** (`Stream::shares_storage_with` — identidade de armazenamento, nunca igualdade) com
**dois consumidores** (`Cook::set_external` não rehasha o mesmo stream; `GpuCook` não reenvia a costura
que não mudou). ⭐⭐ **A VISTA entra no grafo** (`source.camera`, crate nova) e destapou uma lei no
`motion.drive`: *um fio sem valor não escreve* — antes ele escrevia `Size = 0` e a arte **sumia**.

---

## §3 — ⚠️ SUPERFÍCIE PARTILHADA (saída do `collision-surface.sh`, não de memória)

```text
SUPERFÍCIE DE COLISÃO — line/motion-value contra main
  merge-base 1d43da737   ·   70 commit(s)   ·   281 arquivo(s)
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      22   (base: 22)
▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               85   (base: 85)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)
▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 2 pacote(s) '+name' novo(s):  "ph2d-contact"  "ph2d-node-source-camera"
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC nos arquivos que a linha tocou
    ✗   711 / 700   crates/ph2d-gpu-cook/src/lib.rs      ← CURADO no fecho, ver §6
```

⭐⭐ **ZERO contadores partilhados se movem.** Nenhum schema, nenhum dos três registos, nenhum
contrato congelado, nenhum ADR. ⇒ o risco de mesmo-símbolo desta linha é **só** o das duas crates
novas (nomes únicos) e do que ela editou fora da família:

| fora de `ph2d-app-motion` | o quê |
|---|---|
| `crates/ph2d-nodegraph/` | `attr.rs` (`shares_storage_with`) · `cook.rs` (`set_external` + contadores) · `external.rs` (**`CAMERA = "$camera"`**, chave reservada nova) · `gpu.rs` · `stream_op_meta.rs` |
| `crates/ph2d-gpu-cook/` | `sent_boundaries` no `estado.rs`, o laço em `lib.rs` → **`costura.rs` (ficheiro novo)**, `accessors.rs` |
| `crates/ph2d-node-registry/` | `gpu_channels.rs`, `lib.rs` |
| `shells/desktop/` | **9 ficheiros** — o `fase_motion_bridge.rs` (a chamada passou a `publish_editor_inputs`), gizmos do warp/colisor e 2 gates |
| nós | `motion-shape` · `motion-drive` · `motion-trail` · `motion-slit-scan` · `motion-strobe` · `sim-collide` · `sim-step` · `sim-spawn` · `pulse-beat` · `fx-rgb-split` · `fx-drop-shadow` · `value-*` |
| crates NOVAS | **`ph2d-contact`** · **`ph2d-node-source-camera`** |

⚠️ **A chave `"$camera"`** é o único literal novo no espaço reservado; ela entra pela porta
`is_reserved` que já existia e tem gate nos dois sentidos.

⚠️⚠️ **PRAZO DE VALIDADE:** esta tabela mede a linha contra o `main` de **2026-09-16**. Se outra linha
fundir antes, leia o valor do `main` **no ficheiro** e conte o DELTA (DIRETRIZ §1.5.3) — esta tabela diz
só *o que a linha achava que estava a tocar*.

---

## §4 — Contratos congelados (§6 do `CLAUDE.md`)

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool` intocados (confirmado pelo
mapa acima). Todo canal novo desta linha é **side-metadata no registry** ou uma coluna de `Stream` —
que é exactamente o que o congelamento permite.

---

## §5 — Ordem, dependências e o que SMOKAR

- **Ordem:** os 72 commits são sequenciais e não têm dependência cruzada com outra linha. As duas
  crates novas são folhas (`ph2d-contact` é usada pelo `sim.step`; `ph2d-node-source-camera` pelo
  registry gerado — ⚠️ **`cargo run -p ph2d-node-sync` já foi corrido**, o registo está no commit).
- **Smoke aprovado pelo dono** (os cinco ciclos): `=114`/`=115` (colisor) · `=116`/`=117` (valor e
  pulso) · `=118` (aparência) · **`=119` (fontes, aprovado hoje)**.
- ⏳ **O que NÃO foi smokado:** nada de novo — mas o **`=114` mudou depois da aprovação do ciclo 5**
  (o colisor foi para a shape) e o dono aprovou a versão nova em 13–16/09.
- ⚠️ **Depois da fusão é o `main` que ele smoka**, e o binário quente está na worktree DESTA linha
  (§10) — não na árvore integrada.

---

## §6 — ⛔⛔ O que o portão de fecho apanhou

**1. ⛔ Um TETO DE LOC que só o mapa de colisão vê.** `crates/ph2d-gpu-cook/src/lib.rs` fechou em
**711 / 700**: a cura da W1 engordou o `cook`. ⛔ A lei é **MOVER, nunca subir o número** ⇒ a
travessia CPU→GPU saiu para **`src/costura.rs`** (`711 → 677`), e o corte é melhor do que o tecto
exigia: *subir, reutilizar e largar uma costura* eram três metades da mesma pergunta espalhadas no
meio de um `cook` de ~450 linhas. O gate `an_unchanged_boundary_is_uploaded_once_and_draws_the_same_bits`
passa sobre o módulo partido. ⚠️ **Nenhuma suíte o teria apanhado** — foi o `collision-surface.sh`.

**2. ⛔⛔ A suíte `--ignored` de GPU: 8 vermelhos, e os 8 têm veredito.** Corrida inteira (225 testes,
903 s): **217 passaram, 8 falharam**. ⚠️ **A corrida saiu por `| tail -12`, que me deu os NOMES e não
os motivos** — *um `tail` é uma janela, não um veredito*. Re-corridos um a um:

| teste | veredito |
|---|---|
| `how_far_does_the_flock_scale` · `crossing_the_reach_boundary_does_not_step_the_cost` · `fx_row_ceiling_probe` · `bounded_readback_cost_probe` · `emitter_sim_ceiling_probe` · `readback_tap_cost_probe` | ✅ **os seis passam juntos** — e passaram com `load 48` à partida, o que é **conclusivo** (um verde sob carga não é sorte) |
| `two_seam_hybrid_timing` | ✅ passa sozinho — é sonda de PERF (`#[ignore = "perf probe"]`), família de flake de carga |
| `value_slope_kernel_matches_the_cpu_on_the_device` | ⛔ **VERMELHO REAL, pré-existente e JÁ DOCUMENTADO NO PRÓPRIO TESTE**: mede `1,05023384e-4` contra a barra de `1e-4` (5 % acima), atribuído por ablação em **2026-08-11**, e a barra é sensível a driver/FMA |

⭐ **Confirmei a atribuição com uma ablação NOVA**, porque o suspeito de hoje era outro: desliguei a
reutilização da costura (a cura da W1) e o teste devolveu **exactamente o mesmo número**,
`1,05023384e-4`, ao bit ⇒ a minha cura não é a causa. *Uma atribuição velha vale mais quando o
suspeito novo é ablado contra ela.*

**3. ⛔ `cargo fmt` e dois avisos meus** (um `use` morto e um `mut` desnecessário nos meus ficheiros de
teste) — curados; o portão corre com `CARGO_BUILD_WARNINGS=deny`.

---

## §7 — Portão de fecho (batched, 1× sobre o diff acumulado)

| portão | resultado |
|---|---|
| `scripts/nextest-impacted.sh` (`BASE=1d43da737`) | ✅ **16 667 testes, 16 667 passaram**, 8 622 saltados |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` (com `CARGO_BUILD_WARNINGS=deny`) | ✅ verde depois de curar **dois vermelhos MEUS** (§6.3) |
| `cargo fmt --all --check` | ✅ (depois de um `fmt` nos ficheiros do ciclo 8) |
| `cargo machete` | ✅ *«didn't find any unused dependencies»* |
| `scripts/check-standalone-optional.sh` | ✅ 3 crates com dependência interna opcional compilam sozinhas |
| `scripts/check-workflow-packages.sh` | ✅ 32 nomes citados por workflow, todos existem |
| suíte `--ignored` de GPU (`ph2d-gpu-cook`, 225 testes) | ⚠️ **217 + 7 = 224 verdes, 1 vermelho pré-existente e documentado** — ver §6.2 |

⚠️ **O `ship.sh` inteiro NÃO foi corrido** (é do integrador/ship, DIRETRIZ §8). O que ele tem a mais
está no §8.

---

## §8 — O que só o `ship.sh` apanha (o gate de integração não corre)

- **`cargo deny` / `cargo audit` (RUSTSEC):** não corridos aqui. ⚠️ A linha **não acrescenta nenhuma
  dependência EXTERNA** — as duas entradas novas do `Cargo.lock` são crates **internas**
  (`ph2d-contact`, `ph2d-node-source-camera`), logo a superfície de RUSTSEC é a mesma do `main`.
- **`typos`:** não corrido. ⚠️ Esta linha escreve muita prosa em português com acentos e dois
  tutoriais em HTML — é o candidato mais provável a ruído aqui.
- **A matriz 3-OS e o `physics_ecs_c9`:** só o CI. ⚠️ A linha **não toca determinismo** (nenhum
  `HashMap` novo, nenhuma ordem de iteração mudada), mas o `sim.step` ganhou o resolvedor de
  contacto — *se algum hash de replay mudar, é aí que se olha primeiro*.

---

## §10 — Limpeza e o binário do smoke

- `rm -rf target/*/incremental` na worktree: **feito no fecho** (DIRETRIZ §1.5.9 item 7).
- **Binário do smoke quente** (item 9), compilado com a MESMA linha que o dono corre:
  `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop --profile smoke`
  ⚠️ **Ele está quente NESTA worktree.** Depois da fusão o dono smoka o `main`, e ali o build é novo.

### ⚠️ A UMA LINHA para o `CLAUDE.md §5` (o integrador escreve; a linha não toca no ficheiro)

> Sugestão de texto para a linha **Aberto** do módulo Motion Nodes:
>
> ✅ **Os ciclos 6, 7 e 8 FECHARAM** (valor & pulso `=117` · aparência `=118` · **fontes & dados
> `=119`**), os três com o smoke do dono aprovado e tutorial em PDF. ⭐⭐⭐ **A costura de uma FONTE
> deixou de ser taxa por quadro** — `9,62 → 1,62 ms` a um milhão de linhas, por uma régua só
> (*identidade de armazenamento*) com dois consumidores (o hash do external e o envio da costura),
> e ela vale para as CINCO membranas de uma vez. ⭐⭐ A **VISTA entra no grafo** (`source.camera`) e
> destapou a lei *«um fio sem valor não escreve»* no `motion.drive` (antes ele escrevia `Size = 0` e
> a arte SUMIA). ⭐ O **colisor foi para a SHAPE** (ordem do dono, doc 109) com material e gizmo.
> ⏳ Abertos: a tabela do relógio do grupo das fontes (doc 113 §7 — falta máquina calma), o
> `value.math` a ler operando ausente como `0` (wave de substrato) e as cinco `source.*` fora do
> dispositivo (canal de `geometry_id`). ⇒ o ciclo aberto passa a ser o **9 (RIG & CORPOS MOLES)**.

---

## §9 — O que fica ABERTO (para o §5 do `CLAUDE.md`, não para esta linha)

1. ⏳ **A tabela do relógio do grupo das fontes** (doc 113 §7) — a sonda `measure_the_source_group`
   está escrita e comitada; os números faltam porque a máquina não desceu de `load 5` nesta jornada
   (a `line/UIUX` correu a suíte dela em paralelo). ⚠️ **Não é dívida de código**: é uma tabela de doc,
   e o smoke não depende dela.
2. ⛔ **O `value.math` lê um operando AUSENTE como `0`** — a cura pede conectividade no `EvalCtx`
   (wave de SUBSTRATO). A composição que o tutorial 8 ensina evita-o por construção.
3. ⛔ **Cinco das sete fontes ficam fora do dispositivo**, e duas **recusam o device para o grafo
   inteiro** (forma viva) — é o canal de `geometry_id` que a placa não tem, wave própria.
4. ⛔ **O vermelho do `value_slope`** (§6.2) continua vermelho: ele é `#[ignore]`, logo **o CI nunca o
   corre**, e calibrar a barra exige o número em **mais de uma máquina** (o próprio teste o diz).
5. ⏳ Os abertos dos ciclos 6 e 7 estão nos docs 110 §13 e 112 §6 — nenhum bloqueia.

---

## §11 — A linha PAROU

Fechou o ciclo 8, escreveu este handoff e **espera ordem explícita do Enio** (§0.7). Não integrou,
não pushou, não tocou no `main`.

⇒ **O ciclo seguinte é o 9 (RIG & CORPOS MOLES)**, doc 103 §5 — e ele abre **depois** da integração.
