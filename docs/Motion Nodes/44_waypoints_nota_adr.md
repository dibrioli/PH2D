# 44 — Editor F2: waypoints (roteamento de fios) — **REVOGADO** — nota-ADR

> ## ⛔ REVOGADO pelo [doc 45](45_reroute_e_socket_de_entrada_nota_adr.md) (2026-07-12, mesmo dia)
>
> O smoke do Enio matou o desenho em uma pergunta: *"como se conecta esse ponto no meio do fio? ou não se
> conecta?"*. **Não se conectava** — era decoração. E um afordance que **parece** uma junção e não é, é um bug,
> mesmo com o código certo. O ponto no fio agora é um **NÓ de reroute** (Blender/Nuke), e os waypoints foram
> **deletados** (modelo, formato, painel, shell). Este documento fica como **histórico** — a tese do §3 ("UM
> caminho, ou a faca corta um fio que não está lá") e a lição do §6-bis (o grafo nunca via duplo-clique)
> continuam válidas e valem a leitura.

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F2** (fechamento)
**Status:** implementado, testado (2 mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** **nenhum**

---

## 1. A decisão (já anunciada no doc 43, e ela se pagou)

**Um waypoint muda como o fio é DESENHADO. Não muda NADA do que o grafo computa.**

Pô-lo na `ph2d_nodegraph::graph::Edge` empurraria decoração pra dentro do substrato — pra dentro do
fingerprint do cook, dos gates, de tudo que raciocina sobre uma aresta — e a `Edge` é vizinha do contrato
congelado. Então os waypoints foram **exatamente onde os backdrops foram** (doc 35), pela mesma razão: a
**seção UI-only do `MotionDoc`**.

**O prêmio é EXECUTÁVEL:** o guard `is_dirty` que os backdrops trouxeram permite **PROVAR** que arrastar um
waypoint **não re-cozinha**. Guarda: `no_waypoint_edit_ever_re_cooks_the_graph`. *Decoração que não pode sujar
o cook é decoração que você arrasta a 60 Hz sem pensar* — e um waypoint na `Edge` tornaria esse teste
**impossível de escrever**.

**Zero foundational:** o `GraphHitKind::Waypoint { edge, index }` **já existia** no editor-core, esperando.

## 2. A chave é o INPUT, porque é isso que identifica um fio

Um input aceita **no máximo uma** aresta — invariante do próprio grafo, a mesma em que o
`GraphIntent::Disconnect { to_node, to_port }` já se apoia. Então `(to_node, to_port)` **nomeia um fio,
exatamente**, sem a fonte na chave.

**Corte o fio e o roteamento vai junto** (`prune`) — no **MESMO passo de undo**, então um Ctrl+Z traz os dois
de volta. Pontos de um fio morto não são decoração, são **lixo** — e pior: eles **se re-grudariam** no próximo
fio solto naquele input, e uma conexão nova sairia **misteriosamente torta** por causa do roteamento de um fio
que não existe mais. *Mutante provado.*

## 3. UM caminho, ou a faca corta um fio que não está lá

A polilinha de um fio é usada por **três** coisas que não podem discordar: **o que é desenhado**, **o que a
faca cruza**, **o que o ponteiro passa por cima**. Então o roteamento entrou na `wire_path` — a função que as
três já chamavam — e **não no pintor**.

> Roteie só no pintor e o fio **parece** dobrado enquanto a faca continua cortando **a reta que ele tomava
> antes**: você riscaria o vazio e veria cair um fio um palmo de distância — e riscaria o fio que **vê** e não
> aconteceria nada.

Guarda: `the_knife_follows_the_routed_curve_not_the_straight_line`. **Mutante provado** (roteei só no pintor: o
corte na barriga da curva não pegou nada).

**A rota é o MESMO cubic, encadeado:** `p0 → w0 → … → p3`. Um fio roteado não muda de *espécie* — é a mesma
curva, dita pra onde ir; e um waypoint arrastado de volta pra reta deixa o fio visualmente **idêntico** ao que
era antes dos waypoints existirem.

## 4. Os gestos

| Gesto | O quê | Por quê |
|---|---|---|
| **Duplo-clique num FIO** | adiciona um ponto ali | o fio é a coisa que você quer dobrar, então é nele que você clica. Sem modificador pra decorar, e **não colide** com o alt-press que já **deleta** um fio |
| **Arrastar o ponto** | move | um passo de undo pro gesto, bracketado como o drag de um nó |
| **Duplo-clique no PONTO** | remove | **o mesmo gesto que o criou, na coisa que ele criou** — nada novo pra aprender, nenhum 3º modificador |

**O ponto entra na PERNA em que foi solto** (`route::insert_index`), não no fim da fila — um `push` ingênuo
mandaria o fio até o último ponto e de volta, **dando um nó nele**.

## 5. O formato

A seção `[backdrop]` é, na verdade, **a seção UI-only** — agora com dois cidadãos. Nome é histórico; separar
quebraria todo documento já em disco por nada.

```text
[backdrop]
z <base_z>
b <id> <x> <y> <w> <h> <color> <title...>
w <to_node> <to_port> <x0> <y0> <x1> <y1> ...
```

Um fio reto **não emite registro** (a ausência já diz "reto"); um registro corrompido (coordenada ímpar, `NaN`,
campo faltando) é **rejeitado**, nunca meio-lido — um waypoint `NaN` envenenaria a polilinha e o fio
**sumiria**, o que lê como *"o editor comeu meu fio"*.

## 6. Dívida paga de passagem

O `motion_bridge.rs` estourou o teto de 600 LOC (620). **Extraí, não fiz allowlist**
([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]): a **autoridade de conexão** (`apply_connect` + o
`connect_err_msg` + o `violation_blocks_edge` que só a servem) foi pra `motion_bridge_connect.rs`. É um recorte
com sentido próprio, não um corte pra caber: *o painel PROPÕE uma conexão, o shell DISPÕE.*

## 6-bis. O smoke do Enio: *"duplo click não funciona"* — a MESMA classe do Ctrl+D

**O grafo NUNCA viu um duplo-clique. Nunca. Nenhum painel, nenhum gesto, desde sempre.**

`GesturePhase::DoubleClick` existia no enum e só a superfície da **TIMELINE** o emitia. O `pointer_up` do
grafo escolhia entre **End** (arrastou) e **Click** — e ponto. Porque o **Down do grafo CAPTURA o ponteiro e
retorna CEDO**, passando ao largo da detecção geral de duplo-clique que mora no fim do `pointer_down`.

Então eu fiei a ponta do painel (o `match` no `GesturePhase::DoubleClick`) e **não verifiquei o produtor** — de
novo, exatamente como no Ctrl+D (doc 37). O braço compilava, o teste do painel passava, e o gesto **jamais
chegava**. [[feedback_tool_unit_green_integration_dead]]

**Fix:** espelhar o mecanismo da timeline (`graph_double`, armado no Down do grafo por `record_pointer_down`,
lido no Up pra promover `Click` → `DoubleClick`). Foundational, mas **aditivo**: um campo, dois acessores, uma
linha em cada ponta do dispatch.

**Guarda no PRODUTOR** (`a_second_tap_on_a_graph_surface_is_a_double_click`, em `editor-core`) — o teste que
teria pego isso. Mutante provado: sem o flag, o 2º tap volta como `Click` (`left: Click, right: DoubleClick`) —
**exatamente o bug que o Enio viu**. Mais `a_dragged_second_tap_is_an_end_not_a_double_click` (um gesto que se
moveu é um drag, seja qual for a contagem de taps).

## 6-ter. Dívida de LOC — e o gate que eu não estava rodando

O smoke destapou 5 gates vermelhos **acumulados nesta linha**: `paint.rs` (704) · `interact.rs` (684) ·
`interact_tests.rs` (931) · `ph2d-node-motion-integrate` (750/700) · um magic number no `probe.rs`.

**Por que passaram despercebidos:** o gate de LOC de PAINEL vive em **`ph2d-editor-core`**, e a lista de gates
do handoff desta linha manda rodar `-p ph2d-host-desktop -E 'test(loc_cap)'` — que é **outro gate** (o do
shell). Eu vinha rodando o do shell a cada fatia e o do painel **nenhuma vez**. É precisamente o latente que o
`ship.sh` do integrador pegaria ([[project_integrator_ship_catches_latents_budget_iterations]]) — só que agora,
não na integração.

**Tudo corrigido por SPLIT, zero allowlist** ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]):

| Arquivo | Split |
|---|---|
| `paint.rs` 704 → 471 | **`paint_wire.rs`** — *tudo que é fio*: desenhar, achatar, testar contra a faca. Estão juntos porque **têm que CONCORDAR** (é a tese do §3) |
| `interact.rs` 684 → 530 | **`interact_backdrop.rs`** — os gestos de backdrop |
| `interact_tests.rs` 931 | → + `interact_f2_tests.rs` (backdrops+botões) + `interact_f2b_tests.rs` (duplicate/faca/probe) |
| `ph2d-node-motion-integrate` 750 → 255 | testes pro irmão `tests.rs` |

Uma entrada nova no `PANEL_A11Y_DELEGATE_OK` (HR-12) pro `paint_wire.rs`: ele **não tem widget** e **não
registra nada** — os nós AccessKit dos fios (e dos waypoints) são registrados pelo `hits.rs`. É a saída que o
próprio gate oferece, com o mesmo precedente de outros dois painéis.

> **Ação pro handoff:** a lista de gates da linha precisa ganhar
> `cargo nextest run -p ph2d-editor-core -E 'test(loc_cap) or test(a11y) or test(no_magic)'`.

## 7. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| `ph2d-motion-doc` | **`Waypoints { to_node, to_port, points }`** + `MotionDoc.waypoints` + o registro `w` (módulo irmão `waypoint.rs`) |
| `GraphEdgeView` | campo **`waypoints: Vec<(f32,f32)>`** (o `Eq` caiu — são floats) |
| `GraphIntent` | **`AddWaypoint`** · **`MoveWaypoint`** · **`RemoveWaypoint`** |
| painel | `route.rs` (rota + insert-index) · `interact_waypoint.rs` (gestos) · `paint::wire_path` |
| shell | `motion_bridge_waypoints.rs` (intents + `prune` + `stamp`) · `motion_bridge_connect.rs` (extraído) |
| **foundational** | `ph2d-editor-core`: `graph_double` + `set_graph_double`/`take_graph_double` (aditivo) — **sem isso o grafo não vê duplo-clique** |

## 8. **O EDITOR F2 ESTÁ FECHADO**

backdrops (35) · botões+Ctrl+D+faca (36) · probe+sparkline+smart-connect (37) · readouts inline + cards inertes
(43) · **waypoints (44)**.

**Aberto:** o **F3** (o polish "wow": activity-fire · influence por BFS · live-preview flaps · taper ·
gradiente nas portas Field).
