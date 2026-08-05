# HANDOFF DE INTEGRAÇÃO — `line/Vector`, os ESTADOS DE UI (W7)

**Data:** 2026-08-05 · **Branch:** `line/Vector` · **Tip:** `c74c2e434` · **17 commits**
**Estado:** FECHADA. Todos os smokes **aprovados pelo Enio**. Aguardando ordem explícita de integração.

> ⚠️ Esta linha **não integrou e não pushou**. O `main` de referência é `a4018d203` (05/08 07:05),
> e a linha está rebaseada sobre ele.

---

## 1. O que a wave entrega

**O SMART ANIMATE**: um objeto de UI ganha até quatro **poses autoradas** (*Default / Hover /
Pressed / Disabled*) e o motor descobre o caminho entre elas. O artista põe a cena como quer,
aperta **Rec**, e o **Show** faz a cena **andar** até lá — em vez de trocar.

Três coisas decidem o desenho inteiro, e é por elas que vale começar a ler:

**(a) O estado tem um PAPEL, não um nome.** Com um nome livre alguém teria de autorar uma
**segunda tabela** — *"quando o rato entra, vá para o estado chamado assim"* — e mantê-la em dia
com a lista. Com o papel, o gatilho é **derivado**. E a lista é **opcional**: um papel que ninguém
gravou recua para o `Default`, então autorar só o Hover não deixa o botão preso ao ser apertado.

**(b) O casamento é por `VecPathId`, nunca por nome nem por posição.** Um nome muda quando o
artista renomeia; uma posição muda quando ele reordena — e as duas coisas são gestos que ninguém
espera que quebrem uma animação.

**(c) A correspondência é do PAR, nunca do `t`.** `ph2d_vec_blend::Plan::new` custa **0,64 ms
mesmo quando as duas formas são idênticas** (a busca de fase 256×256 roda de qualquer maneira)
contra **0,0001 ms** de um passo — **13 079×**. Daí a forma da API (`new`/`at`), e daí a regra de
ouro do custo: **um par cuja forma é idêntica não constrói `Plan` nenhum** (vinte objetos numa
troca só-de-cor pagariam **12,79 ms**, 77% de um quadro).

### As duas rodadas de report do Enio (05/08), e o que elas mudaram

**1. *"Nem sempre quando se aperta o Show de um estado, a animação ocorre."*** — **duas camadas**,
cada uma suficiente sozinha, cada uma com gate próprio:

- `go_to` comparava o **RÓTULO** (`target == current`), um *proxy* para *"a cena já mostra este
  estado"*. O proxy **expira no instante em que um voo é abortado**: `current` continua a nomear o
  estado de onde se saiu enquanto a pose viva está a meio caminho do outro. A pergunta passou a ser
  sobre a **POSE** — e o atalho **assume o papel**, senão dois estados de pose igual deixariam o
  readout a acender o nome errado.
- `retarget` abortava **incondicionalmente**, e a ponte re-alinha a cada pedido ⇒ **cada clique em
  Show destruía a transição em curso** antes de examinar se algo tinha mudado. Abortar é a resposta
  a uma tabela **nova**.

**2. *"A animação não funciona para mudanças nos nós da shape, nem para as tools Fillet, Chamfer,
Width e Cut."*** — ⚠️ **a causa é a que nenhum gate vê sozinho:** `ObjectPose::geometry` **existia**,
a `Transition` sabia construir o `Plan`, o `install` sabia escrever os verts — e **ninguém
preenchia o campo**. *Uma capacidade sem PORTA passa em todos os gates*, porque eles leem quem
**consome** e o defeito estava em quem **escreve**
[[feedback_a_capability_without_a_door_passes_every_gate]].

| ferramenta | onde a mudança mora | como entra |
|---|---|---|
| modo **Node** | `VecPath.verts` | a captura grava a forma |
| **Fillet / Chamfer** | `VecVertex.corner_radius` (dentro do vértice) | idem — e o casamento é na geometria **COZIDA**, que é o que faz a quina *arredondar ao longo do caminho* |
| pilha de **LPE** | `VecPath.effects` | idem, de graça |
| **Width Tool** | componente ECS `VecStrokeProfile` | campo **próprio** na pose (o único canal de forma que não mora no `VecPath`) |
| **Cut** | destrói o `VecPathId` | **fronteira nomeada** — ver §6 |

---

## 2. Os commits

```
c74c2e434 fix(vector): o `Cargo.lock` da aresta que faltou, e o ultimo "Repor"
6f144bd03 docs(vector): "repor" -> "restaurar", a decisao que o repo ja tinha tomado
168dc712e docs(vector): o handoff de integracao da W7 -- os estados de UI
4b21dbc82 refactor(vector): o perfil chega ao mundo pela porta do Width Tool (W7)
176af5e00 feat(vector): a cena de smoke ganha o SHAPE, e o Cut fica nomeado (W7)
45708fb54 feat(vector): a LARGURA VIVA entra no estado -- o Width Tool anima (W7)
6c1e28e58 feat(vector): a FORMA entra no estado -- no, Fillet, Chamfer e a pilha MORFAM (W7)
e322b2b6a fix(vector): o Show que nao fazia nada -- o guard perguntava pelo ROTULO (W7)
c913586ab feat(vector): a cena de smoke da W7, e a MOLA fecha por MEDICAO
9ae4d7d93 feat(vector): a AUTORIA dos estados de UI -- gravar, mostrar e esquecer uma pose (W7)
53d9c6a2b feat(vector): um estado de UI tem um PAPEL, e o gatilho passa a ser DERIVADO (W7)
521c92c0b feat(vector): os ESTADOS de UI viajam no documento (W7) + a tinta vira campo da POSE
53bce0414 feat(vector): a MAQUINA DE ESTADOS da W7 -- go_to + advance(dt) + pose()
f0fc1f198 feat(vector): o SMART ANIMATE -- o nucleo da W7 (crate `ph2d-ui-state`)
f8f8970c7 fix(vector): a pele PREENCHE a moldura -- uma frase, doze tipos (BUGS #26, 2a rodada)
9dd7405af fix(vector): a moldura de uma pele e uma CAIXA, e a pele a PREENCHE (BUGS #26)
864229887 fix(vector): um bloco de TECLA pergunta se as teclas estao VIVAS (BUGS #25)
```

Os três primeiros (de baixo) são a cauda da W6.2 — os bugs #25/#26 do painel de peles, já
smokados. O resto é a W7.

**67 arquivos, +6000 / −82.**

---

## 3. A tabela de colisão — o que o integrador tem de CONTAR

| item | linha | `main` (05/08) | nota |
|---|---|---|---|
| **`PROJECT_SCHEMA`** | **56** | **55** | ⚠️ **PROVISÓRIO — CONTE contra o `main` do dia.** A tabela de estados viaja no `ProjectState` |
| `VEC_SCENE_SCHEMA_VERSION` | 14 | 14 | **intocado** |
| registro do `ph2d-ecs` | — | — | **intocado** (nenhum componente novo) |
| `VECTOR_SECTIONS` | **21** | 20 | a seção **States** entra no FIM da lista |
| ADRs novos | **nenhum** | — | ⇒ a linha fica **FORA** da disputa de número de ADR |
| contrato congelado | **intacto** | — | rodado, não auto-relatado (ver §5) |

⚠️ **O bump de schema é o único item de colisão desta linha**, e ele tem precedente doloroso: em
25/07, 27/07 e 01/08 a `line/physics` e a `line/FLIP` escreveram o **mesmo literal** e o
`project.rs` **não conflitou** — o git não sabe o que o número significa, e o bump de uma delas
teria evaporado com a suíte verde. *O valor se CONTA a partir do `main` do dia; ele não está em
nenhum dos dois lados* [[feedback_numbers_that_sum_across_lines_count_dont_pick]]. O conflito que
denuncia é o do `project_schema_tests.rs` ao lado.

### Crates e dependências

- **Crate nova: `ph2d-ui-state`** — folha por desenho: **sem relógio** (a lição W4.T7 do Motion,
  onde o `MotionTransport` morreu porque dois relógios divergem), **sem ECS**, **sem UI**, e **sem
  motor próprio de forma nem de cor** (a geometria vai pelo `ph2d-vec-blend`, a tinta pela porta
  OKLab da mesma crate).
- **Nenhuma dep EXTERNA nova.** O `Cargo.lock` ganha **uma linha** — o nome da crate nova. As duas
  arestas de `Cargo.toml` são internas: `ph2d-ui-state` ← `ph2d-stroke-width` (a largura viva) e o
  shell ← `ph2d-ui-state`.

---

## 4. Onde o merge encosta (os pontos sensíveis)

1. **`crates/ph2d-editor-core/src/ids/chrome/vector_sections.rs`** — a lista `VECTOR_SECTIONS`
   ganha **uma entrada no fim**. É lista compartilhada: só ADICIONE, e conte contra o `main` do dia
   [[feedback_a_shared_list_is_merged_against_todays_main]].
2. **`shells/desktop/src/project.rs`** — o campo `ui_states` no `ProjectState` **e** o literal do
   schema. ⚠️ O `project.rs` foi **PARTIDO** pela `line/sculpt3d` em 04/08 (`project_load_from` saiu
   para `project_load.rs`): uma edição no corpo daquela função **funde limpa contra um arquivo de
   onde ela saiu** [[feedback_clean_text_merge_can_be_semantically_broken]]. Confira que a
   instalação da tabela sobreviveu ao corte.
3. **`crates/ph2d-panel-vector/src/lib.rs`** — o `pub mod` do painel de estados e a row da seção.
4. **`shells/desktop/src/render_loop/mod.rs`** — duas linhas: o `dispatch` das máquinas e o `dt`
   que sai dos ticks do frame. As duas são cobertas por arch-gate (§5).
5. **`crates/ph2d-stroke-width/src/lib.rs`** — `WidthStops::mix` é **aditivo** (nenhuma assinatura
   existente muda).

---

## 5. Gates — 61 no total, e o que cada família prova

| arquivo | n | o que prova |
|---|---|---|
| `ph2d-ui-state/src/tests.rs` | 16 | o casamento por id sobrevive a renomear e reordenar · a rotação vai pelo arco curto · a forma vai pelo motor do Blend, **byte a byte contra ele** · o par só-de-cor custa **0 `Plan`** · a pose viva carrega a forma em que está |
| `ph2d-ui-state/src/machine_tests.rs` | 14 | a máquina **não conta o tempo** · a chegada é **exata e não deriva** (50 voltas com `dt` feio) · o recuo para o Default · **os três gates do report** |
| `vec_ui_state_edit_tests.rs` | 9 | o estado grava a **sub-árvore** · a pose é **LOCAL** (mover o hospedeiro não a invalida) · os três verbos |
| `vec_ui_state_shape_tests.rs` | 6 | os **dois repros da forma** · o Fillet arredonda ao longo do caminho · só-a-pilha morfa · a largura viaja · **a fronteira do Cut, medida** |
| `panel-vector/tests/seam_ui_states.rs` | 5 | as rows são **pintadas, registradas e vivas sob o mouse** (`click_at` real) |
| `the_ui_state_machines_run_and_undo_waits.rs` | 4 | **arch-gate**: o frame anda as máquinas, o `dt` sai dos ticks, e o undo **espera** — nenhum teste de unidade alcança o `run_render_frame` |
| `the_ui_states_survive_the_undo.rs` | 3 | a tabela atravessa o `ProjectState` |
| `ui_states_smoke.rs` | 3 | **fixture**: o controle é mesmo um controle · o ponto cabe dentro do Play · as cinco formas não se tocam |
| `ui_state_bridge.rs` | 1 | **o REPRO do produto** (a sequência que o artista faz) |
| `ph2d-stroke-width/src/lib_tests.rs` | +1 | as pontas da mistura são exatas onde a exatidão é alcançável |

**Mutações: 13 corridas, 13 sangram.** ⚠️ Duas "sobreviventes" foram investigadas e **nenhuma era
buraco**: uma acusou a minha **afirmação** (eu cozia com `.cooked()` na transição, e
`compound::rings` **já coze** — era uma segunda porta, removida) e a outra era um `str.replace` que
**não casou** — o no-op silencioso que a memória já nomeia
[[feedback_python_replace_silent_noop_after_fmt]]. A terceira era buraco de verdade e gerou o gate
`two_states_that_differ_only_in_the_effect_stack_morph`.

### Varredura de fechamento (rodada, não auto-relatada)

- `cargo test --workspace` — **verde**
- `cargo test -p ph2d-host-desktop --tests` — **109 alvos verdes** (os arch-gates que um
  `cargo test -p` por crate **não alcança**)
- `cargo test -p ph2d-editor-core --tests` — **40 alvos verdes**
- `cargo clippy --workspace --all-targets` — **limpo**
- suíte em **release** (`ui-state`, `stroke-width`, shell) — verde
- `cargo machete` — limpo · LOC caps (workspace e shell) — verdes
- contratos congelados: `architecture_contract_surface` (nodes) · `architecture_tool_contract_surface`
  · `architecture_adr_numbers_are_unique` — **verdes**

⚠️ **Flake conhecida, PRÉ-EXISTENTE e alheia:**
`flip_fit_budget_tests::a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` é um kill
de wall-clock da `line/FLIP` e **reprova sob a carga da suíte inteira**; passa isolado. Re-rode
sozinho antes de suspeitar do merge.

---

## 6. Medições — os números que decidiram, e os que foram REJEITADOS

**⛔ A MOLA foi medida e o solver NÃO se justifica — não o reconstrua.**
A forma já está no catálogo: `Elastic Out` mede pico **1,373** / assenta **0,631** / **4**
travessias, contra **1,309 / 0,600 / 3** de um oscilador massa-mola macio. E a pergunta de verdade
é a **INTERRUPÇÃO**: revertendo a 30% do caminho, a volta arranca a **1,34×** a velocidade com que
a ida chegava sob o `Cubic Out` que shipa — o olho não separa isso de 1,00×. Os dois regimes onde
ela morde (`InOut` **0,00×**, `Elastic` **7,02×**) são **inalcançáveis hoje**, porque o seletor de
curva não existe. ⚠️ **O dia em que ele nascer, esta medição volta à mesa** (§0: quem move o número
reconfere a nota). Sonda: `measure_spring` (`-- --ignored --nocapture --test-threads=1`).

**⛔ Sub-dividir os vãos na mistura de largura foi CONSTRUÍDO, medido e REJEITADO.**
Com quatro partes por vão o par patológico (dois presets de joelhos diferentes) cai de **0,1778
para 0,0264** — e os **dois pares exatos sobem de 0,0000 para 0,0667**, porque uma lista
sub-dividida já não é o perfil autorado, é uma **reamostragem** dele. *Melhorar o caso que não
acontece estragando os que acontecem.* Sonda: `measure_width_mix`.

**O custo do `Plan`:** 0,64 ms por par **mesmo com as formas iguais**, contra 0,0001 ms de um
passo — **13 079×**. Sonda: `measure_plan_cost`.

### A fronteira do Cut, medida

A faca **destrói o id** (`scene.remove_path` + peças novas), e um estado guarda poses chaveadas por
`VecPathId`. Daí as duas metades, e as duas são honestas:

- **Gravar os dois estados DEPOIS do corte funciona como qualquer outra forma** — a peça é uma
  forma normal, e editá-la morfa. É este o caminho que a wave entrega.
- **Um estado gravado ANTES do corte não ressuscita o que a faca consumiu.** O membro sai do
  documento e a transição o faz **desvanecer**.

⚠️ **A fronteira tem nome:** *um estado sabe restaurar uma POSE, nunca CRIAR um objeto.* Ressuscitar
exigiria que ele fosse dono do conjunto de objetos (criar path, entidade, hierarquia e um id novo —
que o estado já não referencia), e isso é **outra feature**, não um campo a mais.

---

## 7. Smoke

```
env PH2D_BUILD_SMOKE=61 cargo run -p ph2d-host-desktop --release
```

⚠️ **A cena imprime o número que a torna válida** — se a linha não aparecer, PARE:

```
[ui-states] 6 poses gravadas (Play, Card e Shape, Default+Hover); a transicao do
            Card custou 0 Plan(s) e a do Shape 1.
```

O `0` do Card é o gate de custo do motor. O **`1` do Shape é a wave de 05/08**: se a forma mudou e
ninguém a casou, o Show **troca** a forma no fim em vez de a fazer viajar — que era o defeito
reportado. A cena imprime um roteiro de 13 passos; os que julgam esta jornada são o **11** (a forma
morfa: o nó sobe, as quinas arredondam, o traço engrossa) e o **12** (volte ao Default, entre no
modo Node — **as alças de quina continuam lá**; se sumissem, o Show teria assado o desenho).

---

## 8. Aberto — nomeado, com o preço ao lado

1. **O hover não anima nada, e a ausência é DECISÃO.** Um hover que animasse enquanto o artista
   trabalha tornaria o editor inutilizável (o Figma põe a interação num **modo de apresentação**
   separado), e há um segundo motivo que é nosso: o undo deste editor é por **DIFF do mundo**, então
   uma pose escrita por hover viraria passo de undo a cada vez que o rato passasse por cima de um
   botão. **Ligar o mouse exige um modo de preview com história própria** — decisão de produto.
2. **O seletor de curva não existe.** São 11 famílias × 3 modos = **33 combinações**, e o
   `ph2d-anim` **não dá nome a nenhuma** — um dropdown hoje pintaria identificadores em inglês crus
   (HR-15). O catálogo precisa de nomes antes do knob; e é esse knob que re-abre a medição da mola.
3. **A geometria intermédia passa pelo DOCUMENTO.** A pose do meio é geometria já cozida com a
   pilha vazia, e a chegada devolve a autorada — a passagem é transitória e **cura-se sozinha**,
   mas um Ctrl+S no meio dos 150 ms guardaria a forma cozida. O preço de o Show ter de deixar a
   cena *editável no estado que mostra*. A cura, se um dia doer, é publicar num `LiveGeometry` em
   vez de escrever (o padrão dos outros sete produtores) — e ela **não dispensa** a escrita na
   chegada.
4. **Mistura de largura entre dois perfis de joelhos diferentes desvia 0,1778** de multiplicador
   (medido). Zero nos dois pares que a autoria produz.
5. **Rows do plano UI/UX ainda não atacadas:** W4b/c (aliases/math/DTCG + o token que anima),
   W6.3/W8b (a árvore autorada → `Panel`), W8a (runtime), W9 (export).
