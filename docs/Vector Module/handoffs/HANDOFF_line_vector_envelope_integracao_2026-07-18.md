# HANDOFF de INTEGRAÇÃO — `line/Vector`: Envelope COMPLETO — Fatias 1–5 + C + D + E (ADR-0129)

**Para:** o **agente integrador** (e o próximo implementador da linha).
**De:** a sessão de 2026-07-17/18 que assumiu a linha pelo `HANDOFF_line_vector_continuacao_2026-07-17.md`
(§4.A itens 1–5 do Envelope).
**Estado:** ✅ **LINHA FECHADA — a fila do ADR-0129 acabou, e TUDO foi smokado e aprovado pelo Enio
(2026-07-17/18).** Motor + host live já estavam na `main` (Fatias A+B).
✅ **INTEGRADO em 2026-07-18** — o integrador rebaseou a linha (`b32a46a9` → `8864e4b6`) e fundiu.
**Este documento está CUMPRIDO: é referência, não pendência.** Quem assume a linha daqui pra frente
entra por [`HANDOFF_line_vector_continuacao_2026-07-18.md`](HANDOFF_line_vector_continuacao_2026-07-18.md);
aqui ficam o DESENHO de cada gesto (§8–§10) e as lições dos fixes pós-smoke (§11), que continuam a valer.
- **Fatia 1** = a alça própria de canto no Node (arrasta os 4 cantos, convexidade obrigatória).
- **Fatia 2** = mover/girar/escalar o envelope inteiro no Select (geometria LOCAL + pose no `Transform`).
- **Fatia 3** = o **CONTAINER de N filhos** (*warp group*): o `VecEnvelope` saiu do path e foi para uma
  **entidade-container** (sem path próprio) com `children: Vec<VecEnvelopeChild>`. Um só modelo — 1 filho
  é o caso `N=1`, vários é o grupo.
- **Fatias 4+5** = a **seção Envelope no painel** (Create) + as duas saídas (**Expand**/**Release**).
  Até elas, `envelope_live::create` só era chamado pela env `PH2D_BUILD_SMOKE`: a feature existia no
  motor, gateada e smokada, e **não existia para o artista**.
- **Fatia D** = a gaiola ganha **dois gestos**: **Perspective** (homografia, lados retos — o de sempre,
  e o default) e **Mesh** (patch de **Coons**, os lados DOBRAM: 2 controles por lado, 8 alças novas no
  modo Node). Chips `Cage: Perspective | Mesh` no painel.
- **Fatia C** = **7 presets que GERAM a gaiola** (Arc · Arc Upper · Arc Lower · Bulge · Flag · Wave ·
  Squeeze) + slider **Bend** (`-1..1`). O preset é **promovível**: arrastar uma alça o solta.
- **Fatia E** = **PINOS** (MLS-rigid, o *puppet warp*) — 3º gesto, sem gaiola: prega pontos no Node e
  arrasta. **A fila do ADR-0129 fecha aqui.**

> ⚠️ **A ordem da fila do ADR foi INVERTIDA de propósito** (ele lista 4=Release, 5=painel). Motivo: o
> Release é um BOTÃO — sem a seção do painel ele não teria onde morar (ou viraria um atalho de teclado,
> 2ª porta para a mesma pergunta); e criar sem desfazer é **porta de mão única**. Os dois viraram uma
> fatia só.

> Este handoff **supersede** o `HANDOFF_line_vector_envelope_fatia1_integracao_2026-07-17.md` (removido).
> A **descrição do modelo da Fatia 2** ("envelope = path com componente") está superada pela Fatia 3
> ("envelope = container; os paths são FILHOS"). A mecânica de pose/gizmo é a MESMA, um nível acima.

> **Mapa de leitura** (o documento cresceu com a linha; leia SÓ o que a sua tarefa exige):
> **integrador** → §1 (identidade) · §11 (os fixes pós-smoke e o artefato aberto) · §12 (runbook) ·
> §3–§4 (riscos e contratos). **Próximo implementador da linha** → §6 (a fila) · §8–§10 (o desenho de
> cada gesto). §2/§5 são históricos da Fatia 3 e ficam para contexto.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **HEAD da linha** | `73d59ff4` |
| **Base do fork (merge-base com `main`)** | `cdc3acc1` |
| **`main` desde a base** | **0 commits** — a linha está sobre a `main` de hoje; **`--ff-only` limpo, sem rebase** |
| **Diff total** | 53 arquivos · +7391 / −878 |
| **Contratos congelados encostados** | **NENHUM** (§4) |
| **Smoke** | ✅ **TUDO APROVADO** — F1/F2/F3, F4+F5, D, C, E e os 4 fixes pós-smoke (§11) |

**Os 8 commits de FEATURE**, em ordem (os de `docs`/`chore` intercalam e são inertes):

| Commit | O quê |
|---|---|
| `207d10b9` | F1 — arrastar os cantos da gaiola no Node |
| `5bddd9e4` | F2 — mover/girar/escalar o envelope no Select (geometria LOCAL + pose) |
| `10889f0e` | F3 — o **container** de N filhos (*warp group*) |
| `d5695795` | F4+F5 — a seção **Envelope** no painel + **Expand**/**Release** |
| `5b6f754c` | **D** — gaiola de lados CURVOS (patch de Coons) + chips `Cage` |
| `3f4e0c91` | **C** — 7 presets geradores de gaiola + slider **Bend** |
| `adc2f228` | **E** — **pinos** (MLS-rigid, o *puppet warp*) |
| `30d7115c` `f5d59c96` `2e2a951d` | os **fixes pós-smoke** (§11) |

---

## §2 — O que a Fatia 3 entrega (30 s)

Um envelope agora é um **container**: uma entidade sem `VecPathRef` que carrega o `VecEnvelope` e tem
as formas envolvidas como **filhos**. Uma gaiola só (`corners`) deforma **todos** os filhos, sobre a
**bbox-união** das fontes deles — é o *warp group* do Affinity/Illustrator. Um envelope de uma forma só
é o caso `N=1` do MESMO modelo: não há dois caminhos de código, o gesto/gizmo/recook nunca perguntam
"isto é um-ou-vários?".

**Como um filho entra:** o `create` (síncrono) **assa a geometria cozida de cada forma em MUNDO** como
a fonte do filho, cria o container na identidade, **reparenta cada filho na identidade** (`ChildOf` +
`Transform::IDENTITY`) e pendura o `VecEnvelope`. Como a pose do filho foi assada na fonte e o container
nasce na identidade, no nascimento **container-local == mundo**. Mover o container (Fatia 2) move o
grupo por parentesco; o `recook` reescreve só a geometria local dos filhos.

**Por que síncrono (sem `pending`/`upkeep`, ≠ blend/morph):** o blend/morph CRIAM um path novo que
precisa nascer no `sync`; o envelope **não cria path nenhum** — envolve formas que já existem e adiciona
só um container SEM path. Então assar+reparentar+pendurar acontece na hora.

---

## §3 — Riscos de INTEGRAÇÃO (DIRETRIZ §1.5.9.2–3)

### 3.1 Foundational tocado — ⚠️ a Fatia 3 RESHAPE o componente `VecEnvelope`

`crates/ph2d-ecs/src/vec_envelope.rs` mudou de **forma** (é foundational, editável em Modo L, ADR-0107):

```
  ANTES (F1/F2):  VecEnvelope { source: Vec<u8>, corners: [[f64;2];4] }
  AGORA (F3):     VecEnvelope { corners: [[f64;2];4], children: Vec<VecEnvelopeChild> }
                  VecEnvelopeChild { path: u64, source: Vec<u8> }   // struct NOVA
```

O que isto significa para o merge:
- **Registro por NOME, intocado** (`registry.rs:281` registra `"ph2d::ecs::VecEnvelope"`). **NÃO há
  "números que somam"** — nenhum componente NOVO foi registrado, então a tríade de contagens
  (`ph2d-ecs`/`-render`/`-script`) está intacta e um merge que traga outro registro não colide.
- **É uma mudança de SCHEMA do componente** (serde/postcard posicional). **Sem migração necessária:**
  nenhum projeto salvo em disco tem um `VecEnvelope` (a feature acabou de nascer). Se o integrador tiver
  um `ph2d_project.postcard` local com um envelope da era F1/F2, ele **não** desserializa — apague-o
  (é gitignorado e efêmero).
- `settle_origins`: o gate de "geometria derivada pula o settle" segue verde. Os **filhos** do envelope
  são pulados **de graça** (têm `ChildOf` — a linha 186 já os exclui); o **container** não está no
  `VecEntityMap` (não tem path), logo o `settle` nunca o itera. A cláusula `VecEnvelope` no `settle`
  (linha ~209) agora é defensiva/inerte (nenhum path carrega o componente) — deixada de propósito.

### 3.2 O que foi tocado na Fatia 3 (arquivos)

| Arquivo | O quê |
|---|---|
| `crates/ph2d-ecs/src/vec_envelope.rs` | **reshape** `VecEnvelope` + `VecEnvelopeChild` novo + docstrings do container |
| `crates/ph2d-ecs/src/lib.rs` | `pub use vec_envelope::{VecEnvelope, VecEnvelopeChild}` |
| `shells/desktop/src/envelope_live.rs` | `create` (síncrono, container/reparent/bake) + `recook(sim, scene)` por QUERY + `union_control_bbox` porta única. **Removidos** `attach`/`upkeep`/`control_bbox`. |
| `shells/desktop/src/envelope_live_tests.rs` | 10 gates reescritos p/ o container (multi-filho, reparent, bake, pose) |
| `shells/desktop/src/envelope_gesture.rs` | alvo = **bits do container** (não `VecPathId`); sem `VecEntityMap` |
| `shells/desktop/src/envelope_gesture_tests.rs` | 8 gates: press/drag/view atravessam a pose do CONTAINER |
| `shells/desktop/src/vec_selection.rs` | **seleção-só-o-container** (`sole_envelope_container`) + gate |
| `shells/desktop/src/vec_gizmo_view.rs` | `container_view` (caixa-união) + `gizmo_view_from` porta única + gate |
| `shells/desktop/src/render_loop/snapshots.rs` | `build_view`: ramo do container (chama `container_view`, gate `vec_gizmo_on`) |
| `shells/desktop/src/render_loop/mod.rs` | `recook(sim, vec_scene)` (2 args); `upkeep` do envelope REMOVIDO; `view` lê `hero.gizmo.selection` |
| `shells/desktop/src/input_dispatch.rs` | `press` recebe `env_container` (=`gizmo.selection`); `drag` sem `VecEntityMap` |
| `shells/desktop/src/app_state.rs` | `vec_envelope_drag: Option<(u64, usize)>` (era `(VecPathId, usize)`); `vec_envelope_pending` REMOVIDO |
| `shells/desktop/src/main.rs` | init de `vec_envelope_pending` removido |
| `shells/desktop/src/vec_entities.rs` | `next_root_order` → `pub(crate)` (usado pelo `create` do container) |
| `shells/desktop/src/build_smoke.rs` | cena 11 (síncrona); **cena 12 NOVA** (warp group, 2 formas) |

### 3.2b O que foi tocado nas Fatias 4+5 (o painel e as saídas)

| Arquivo | O quê |
|---|---|
| `shells/desktop/src/envelope_live.rs` | **`dissolve` + enum `Keep`** (Expand/Release são a MESMA função) + **porta única `container_of`/`sole_container`** |
| `shells/desktop/src/vec_selection.rs` | passa a CHAMAR `envelope_live::sole_container` (as cópias privadas foram apagadas) |
| `crates/ph2d-editor-core/src/ids/chrome/vector.rs` | 4 ids novos em bloco **append-only** + `VECTOR_SECTION_ENVELOPE` na `VECTOR_SECTIONS` |
| `crates/ph2d-editor-core/tests/node_id_collisions.rs` | os 4 ids na tabela |
| `crates/ph2d-i18n/src/lib.rs` | `"panel.vector.section.envelope" => "Envelope"` (só o HEADER — labels de botão são literais em todo este painel) |
| `crates/ph2d-panel-vector/src/paint_envelope.rs` | **NOVO** — a seção (módulo irmão, teto de 600 LOC) |
| `crates/ph2d-panel-vector/src/populate_envelope.rs` | **NOVO** — o registro dos 3 widgets |
| `crates/ph2d-panel-vector/src/{paint_sections,populate}.rs` | `#[path]` dos irmãos + a chamada (ordem da seção / registro) |
| `crates/ph2d-panel-vector/src/{state,lib}.rs` | `set_current_has_envelope` (publisher) + re-export |
| `crates/ph2d-panel-vector/src/event.rs` | os 3 ids na allowlist `forwards_plain_click` |
| `crates/ph2d-panel-vector/tests/seam.rs` | 2 gates novos + a contagem de seções **20 → 21** |
| `shells/desktop/src/render_loop/mod.rs` | 3 flags `pending_*_envelope` + drain + consumo + o publisher `set_current_has_envelope` |

⚠️ **Números que somam neste merge:** a contagem de seções do `seam.rs` (`VECTOR_SECTIONS.len() == 21`)
e a tabela de `node_id_collisions`. Se outra linha adicionar uma seção/id ao painel vetorial, o valor
certo **não está em nenhum dos dois lados** — conte a lista combinada.
⚠️ `crates/ph2d-panel-vector/src/populate.rs` está em **591/600 LOC**: a próxima seção deste painel
tem de orçar o split.

**Seam-risk num merge:** `input_dispatch.rs`, `render_loop/{mod,snapshots}.rs` são arquivos quentes. As
inserções são localizadas (o ramo do container no `build_view` fica após o de `FlipObjectRef`; o `press`
lê `gizmo.selection` no braço `None if node_mode`). Conflito aqui é de contexto, não de símbolo —
Mergiraf resolve; senão, o gate de compilação da árvore combinada (`foundational-integrate.sh`) pega.

### 3.3 O que SÓ o `ship.sh` pega

Rodei `cargo check -p` (workspace dos crates tocados), `cargo clippy -p ph2d-ecs -p ph2d-host-desktop
--bins --tests` (limpo), a suíte `envelope_`/`vec_selection`/`vec_gizmo_view` (20+ verde), `ph2d-ecs
--lib` (79 verde), e o gate `no_tofu_glyphs` (verde). **Não** rodei o `ship.sh` completo
(machete/deny/audit/typos/nextest --workspace) — é do integrador. Nenhuma dep nova.

---

## §4 — Contratos congelados (§1.5.9.4)

**Nenhum encostado.** `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` intactos; o
`architecture_vector_contract_surface` (do `ph2d-vector-doc`) intacto. O `VecEnvelope` (em `ph2d-ecs`)
**não é contrato congelado** — é componente foundational, editável em Modo L. Nenhum ADR novo (o
ADR-0129 já cobre; a Fatia 3 é o "container multi-filho" da fila dele, e o modelo container-sempre foi
escolha do Enio nesta sessão).

---

## §5 — Estado dos gates e do SMOKE (§1.5.9.6)

> ✅ **TODO o smoke desta linha foi aprovado pelo Enio.** A tabela abaixo é da Fatia 3 (histórica); o
> estado consolidado dos gates está no **§12** e os fixes pós-smoke no **§11**.

**Gates da Fatia 3 (todos mutação-testados):**

| Gate | Onde | Prova |
|---|---|---|
| repouso = identidade | `envelope_live_tests` | gaiola em repouso não muda a forma (e ela fica CHEIA) |
| gaiola puxada = MOTOR | idem | a saída é `warp_path(fonte, QuadWarp)` byte-a-byte, não a ingênua |
| **UMA gaiola, DOIS filhos** | idem (`a_two_shape_envelope_deforms_both_by_the_one_cage`) | os 2 filhos passam pelo MESMO warp sobre a bbox-união |
| reparent na identidade | idem | filhos `ChildOf(container)` + `Transform::IDENTITY`; container sem `VecPathRef` |
| **assa a pose de mundo** | idem (`create_bakes_the_child_world_pose_into_the_source`) | forma assentada → fonte em MUNDO (não local-centrada) |
| pose no container não vaza | idem | mover o `Transform` do container não entra na geometria do filho |
| fonte autorada sobrevive | idem | `child.source` intacto após recook |
| gaiola degenerada congela | idem | 3 cantos colineares → recook pula, forma fica |
| create sobre nada = None | idem | ids inexistentes não deixam container órfão |
| **seleção-só-o-container** | `vec_selection` (`selecting_an_envelope_child_selects_only_the_container`) | clicar um filho → gizmo = `{container}` só (não `{filho, container}`) |
| **gizmo = caixa-união** | `vec_gizmo_view` (`an_envelope_container_publishes_a_union_gizmo_box`) | o container publica a bbox-união; grupo comum não |
| gesto atravessa a pose | `envelope_gesture_tests` (8, fixture pose `[100,50]` no container) | press/drag/view convertem local↔mundo pela pose do container |

**Gates das Fatias 4+5:**

| Gate | Onde | Prova |
|---|---|---|
| **Release ressuscita a fonte + liberta** | `envelope_live_tests` | geometria == fonte autorada; filho sem `ChildOf`, com `RootOrder`, na identidade; container despawnado; pen seleciona os libertados |
| **Expand mantém a deformada** | idem | geometria == a deformada, ≠ a fonte |
| **a pose é ASSADA (a arte não se move)** | idem (`dissolving_bakes_the_container_pose_…`) | container movido+escalado → dissolve → bbox de MUNDO idêntica (< 1e-6) |
| sem envelope, dissolve é no-op | idem | forma comum intocada, devolve `false` |
| **os 3 botões CHEGAM ao bus quando clicados** | `panel-vector/tests/seam.rs` | `paint` → hit-rect → ponteiro real → dispatcher → `event.rs` → bus (pega não-pintado / não-populado / fora da allowlist) |
| **Expand/Release NÃO existem sem envelope** | idem (par de AUSÊNCIA) | e o Create existe SEMPRE (porta de entrada da feature) |

**4 mutações-chave das Fatias 4+5 (mutei → RED sobre visto-verde → restaurei):**
- tirar os 3 ids da allowlist do `event.rs` → o botão fica **clicável e MORTO** → seam RED.
- `dissolve` sem `bake_xform` → a arte se move ao dissolver → gate RED.
- `Keep::Authored` lendo a cena (Release colapsa em Expand) → gate RED.
- (o "sem `populate`" já é coberto pela 1ª asserção do seam — o ponteiro não vira `Click`.)

**Mutações da Fatia 3** (idem): desligar o `sole_container` → gizmo fica `[filho, container]` · o
`container_view` usar só o 1º filho → caixa não une · pular o `bake_xform` no `create` → fonte fica
local-centrada.

**Smoke — PENDENTE do Enio (Fatias 4+5):** (caminho ABSOLUTO — o relativo só funciona a partir da raiz
do repo, e é onde este comando já falhou uma vez)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  cargo run -p ph2d-host-desktop --features panel-vector
```

**Sem env nenhuma** — é esse o ponto: agora a feature nasce do painel.
1. Desenhe 2 formas, selecione as duas, painel → seção **Envelope** → **Envelope**.
2. **Node:** arraste os cantos da gaiola → as duas deformam juntas.
3. **Select:** o gizmo abraça as duas e move/gira/escala o grupo.
4. **Mova o envelope** com o gizmo e então clique **Expand** → a arte tem de ficar EXATAMENTE onde
   está (é o gate da pose assada, visto no olho) e virar formas comuns.
5. Refaça e clique **Release** → as formas voltam **sem a deformação**, na posição do envelope.
6. Com nada selecionado (ou uma forma comum), **Expand/Release não aparecem** — só o **Envelope**.

As cenas antigas continuam: `PH2D_BUILD_SMOKE=12` (warp group, 2 elipses) e `=11` (o caso `N=1`).

---

## §6 — A FILA (a ordem é do Enio; ADR-0129 §Plano é a fonte)

Fatias 1, 2, 3, **4 e 5** fechadas. Resta da 4.A (fechar o Envelope):

6. ~~**D — 4 curvas de lado (Coons)**~~ — **FECHADA** (`5b6f754c`). Ver §8.
7. ~~**C — presets de gaiola**~~ — **FECHADA** (`3f4e0c91`). Ver §9.
8. ~~**E — pinos / MLS**~~ — **FECHADA** (`adc2f228`). Ver §10.

**A fila do ADR-0129 ACABOU.** Depois da integração, o que a linha `line/Vector` tem pela frente é a
**4.B herdada**: Live Path Effects como nós (o multiplicador — a costura fonte≠cozido do ADR-0121 já
é o pré-requisito) · morph vivo (`t` animável) · blend em CADEIA (>2 formas) · tipos de quina · texto
em caminho · trim path · repeater · largura variável. (a mais delicada — exige o `break_cusp` que hoje volta `None` de propósito;
   ADR §3.2, e o `folds()` da Fatia D é o precedente do guard que ela vai precisar).

E a 4.B herdada (Live Path Effects como nós, morph vivo, blend em cadeia, etc.).

**Aberto no Envelope, de propósito (não são bugs — são escopo não pedido):**
- **Fidelity/`accuracy` não é exposta.** O `accuracy` é relativo (0,1% da diagonal da união) e não
  tem knob. Quando houver queixa de "a curva perdeu detalhe", é um slider na seção — e o lugar dele
  já está lá.
- ~~**Presets de gaiola** — "cada um é só um conjunto de `corners`; é UI, não motor".~~ **Esta frase
  estava ERRADA e a Fatia D é o motivo** — ver §8.3.
- **Envelope aninhado** (envolver um envelope). O `container_of` **já sobe a cadeia** e pára no mais
  próximo, então a seleção e o dissolve se comportam; o que NÃO foi exercitado é o `create` sobre um
  container (ele resolve `ids` de PATHS, e um container não tem path) — hoje envolver um envelope
  envolve os **filhos** dele. Se isso for pedido, é uma decisão de modelo, não um fix.
- **A gaiola só é editável no Node**, e isso é ADR §3.3 (cerca de Chesterton — um gizmo sobre
  geometria que se move dobra; 5 tentativas revertidas no Blend).

---

## §8 — Fatia D: a gaiola com lados que dobram (`5b6f754c`)

### 8.1 — Dois gestos, e por que não é um knob

| Gesto | Mapa | Lados | Alças |
|---|---|---|---|
| **Perspective** (default) | homografia dos 4 cantos | RETOS | 4 cantos |
| **Mesh** | patch de **Coons** das 4 curvas de bordo | dobram | 4 cantos + 8 controles |

**Eles não são o mesmo mapa com um parâmetro.** Com os lados retos o Coons é **bilinear** e a
homografia é **projetiva**: concordam nos 4 cantos e **divergem no miolo** (sob bilinear uma reta
interior vira parábola; sob homografia toda reta continua reta — que é o que "perspectiva" quer
dizer). É a tabela do ADR §4 ao pé da letra, e é por isso que Photoshop separa *Distort* de *Warp*.

**Em repouso os dois são a identidade**, então trocar de gesto numa gaiola intocada não move um
pixel. Trocar **depois** de deformar muda o desenho — e isso é o mapa mudando, não um bug. O gate
`perspective_and_mesh_agree_at_rest_and_differ_off_it` crava as duas metades: se ele ficasse verde
nos dois ramos, um dos dois mapas seria supérfluo.

### 8.2 — As quatro decisões que decidem o resto

1. **O termo bilinear NEGATIVO não é enfeite.** Cada régua já entrega os cantos por conta própria;
   somá-las os conta **duas vezes**. Subtrair o bilinear cancela a duplicata *exatamente*, e é isso
   que faz `S(u,0) = B(u)` ao bit — **o bordo desenhado é o bordo do mapa**. Uma alça que pousasse
   *perto* do bordo tornaria a gaiola uma sugestão, não um contrato.
2. **Dois guards, e a diferença é epistemológica.** Perspective usa **convexidade**, que tem um
   teorema atrás (§5 do ADR). Para um patch de Coons **não existe critério fechado equivalente** ⇒
   `cage_folds` responde por **grade** (17×17 de `det J`) e o código DIZ que é amostragem. Dobra em
   vetor é pior que em raster (contorno auto-interseccionado = a saga da lasca) e o `break_cusp`
   volta `None` de propósito.
3. **A reta entra na forma canônica (⅓, ⅔).** A degenerada `(P0,P0,P3,P3)` não é afim em `t` — a
   `ph2d-vec-blend` já pagou (as intermediárias ondulavam); aqui o preço seria o repouso deixar de
   ser identidade **exata**. `rest_edges` é a porta única de "os lados são retos", 3 chamadores.
4. **Em Perspective os `edges` são FATO DERIVADO, não estado livre.** Valem sempre os canônicos da
   gaiola atual, re-emitidos a cada movimento de canto **e** na troca de gesto. Sem isso, trocar para
   Mesh depois mostraria alças penduradas na gaiola que existia antes do último arrasto — o *"funciona
   e depois esquece"* de chapéu novo.

Detalhes menores, mas que a próxima fatia vai encostar: no Mesh, mover um canto **leva os 2 controles
vizinhos junto** (uma alça de Bézier pertence à sua âncora); o espaço de índices é **UM** (`0..4`
cantos · `4 + 2·lado + j` controles) e atravessa hit-test, arrasto e desenho; `offered_edges` é a
porta única de *"esta gaiola tem alça de lado?"*, então **alça pintada é sempre alça viva**.

### 8.3 — ⚠️ A ordem C↔D estava invertida na fila, e a Fatia D é a prova

A fila do ADR lista **C (presets) antes de D (Coons)**, e o §6 deste handoff dizia que presets eram
*"só um conjunto de `corners`; é UI, não motor"*. **Isso vale só para os presets QUAD-expressáveis**
(Perspective, Free Distort). **Arc, Flag, Wave, Bulge, Fish — os presets que a palavra "preset"
evoca — precisam de LADOS CURVOS**: com 4 cantos retos um "Arc" é um trapézio, e trapézio não é arco.

O próprio ADR já dizia a versão certa (*"o preset só vale primeiro se for GERADOR de gaiola … como
gerador, **Quad e 4-curvas** saem quase de graça"*) — a fila é que ficou na ordem alfabética. Com a
Fatia D fechada, **C agora é de fato quase de graça**: cada preset é uma função
`(bend %, corners) -> (corners, edges)` + `kind = Mesh`, e a porta (`VecEnvelope`) já aceita tudo.

### 8.4 — Dívida de LOC das Fatias 3/4, paga aqui

`build_smoke.rs` (666) e `vec_gizmo_view.rs` (610) **já estavam acima do teto de 600 desde as Fatias
3/4**, e eu não peguei: o gate HR-18 do **shell** mora em `shells/desktop/tests/file_loc_caps.rs` e
**não roda com `cargo test -p ph2d-editor-core`** (que foi o que rodei ao fechar aquelas fatias). É a
mesma classe de [[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]], num diretório diferente.

Splits (mecânicos, sem mudar comportamento): **`envelope_smoke.rs`** (as cenas 11/12 — elas só usam
os frames 3 e 4 e nenhum braço compartilhado, então sair do `match` é a MESMA sequência) e
**`vec_gizmo_view_tests.rs`** (seguindo o idioma `#[path]` que o próprio arquivo já usava para os
testes de hit). As duas cenas foram **verificadas vivas** depois do split.

> **Para quem fechar a próxima fatia:** rode `cargo test -p ph2d-host-desktop --tests` (não só as
> gates da `editor-core`) — é lá que moram os arch-gates do shell.

### 8.5 — Smoke da Fatia D

Sem env nenhuma:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  cargo run -p ph2d-host-desktop --features panel-vector
```

1. Desenhe 1–2 formas, selecione → painel → **Envelope** → **Envelope**.
2. A linha **Cage** aparece com **Perspective** aceso. Clique **Mesh**: **nada deve se mover** (a
   gaiola está em repouso, e os dois mapas coincidem ali).
3. Modo **Node**: agora há **8 bolinhas a mais**, uma por controle de lado, cada uma com uma haste
   até o seu canto. Arraste uma → o lado **curva** e a arte curva com ele.
4. Arraste um **canto** no Mesh → os 2 controles vizinhos vão junto (o lado acompanha rígido).
5. Empurre um controle **muito** para o outro lado da gaiola → a alça **para** (o guard de dobra).
6. Volte para **Perspective** → os lados endireitam e as 8 bolinhas somem.
7. Cenas antigas seguem: `PH2D_BUILD_SMOKE=12` (warp group) e `=11` (`N=1`).

---

## §9 — Fatia C: os presets de gaiola (`3f4e0c91`)

### 9.1 — Um preset é uma BARRIGA por lado

Ele **não move canto nenhum**: são 8 números dizendo o quanto os 2 controles de cada lado saem da
corda, **ao longo da normal EXTERNA**. Escolher a normal (e não `y` de mundo) é o que torna a tabela
legível — *Bulge* é "todo mundo para fora", *Squeeze* é "os laterais para dentro" — e faz a
assimetria de percurso (o lado de cima vai de TR para TL, ao contrário do de baixo) **se cancelar**
contra o sinal da normal: por isso *Flag* e *Bulge* saem com os dois lados escritos IGUAL. Numa
tabela em `y` de mundo cada linha teria de lembrar de qual lado ela fala, e alguém erraria uma.

**A TABELA mora no `ph2d_ecs::EnvelopeWarp`; a MATEMÁTICA no `ph2d-vec-envelope`.** A tabela é
*dado* (o que a fundação guarda) e transformar barrigas em gaiola é geometria. O enum na crate de
geometria esbarraria na mesma regra que mantém o `VecEnvelopeChild::source` em bytes; um enum de cada
lado seriam duas listas para driftar. **Acrescentar um preset = 3 linhas no componente e ZERO
mudança de painel** — o painel se popula da lista PUBLICADA (o idioma da rack de áudio).

### 9.2 — ⚠️ A amplitude é MEDIDA, e o primeiro valor estava errado

`AMP` é o teto que garante que **qualquer** combinação de barrigas, em qualquer direção, em toda a
faixa de `bend`, **não dobra o patch** — e por isso vale para a linha que alguém acrescentar amanhã,
não só para as sete de hoje.

`0.35` (o primeiro valor tentado) é seguro para as formas da tabela **atual**, que envergam um par
de lados por vez, e **dobra quando os quatro lados envergam juntos** — um caso que nenhum preset de
hoje produz, e que por isso teria passado despercebido até o preset que o produzisse. Medido: `0.30`
é o maior valor que sobrevive ao caso de quatro lados; o preço é 14% de curso, invisível na tela.

É a mesma lei da Fatia D dita ao contrário: a alça do gesto Mesh **para na fronteira** porque a MÃO
pode pedir o impossível; a faixa de um preset é **desenhada**, então ela nunca pede. Um slider que
"para de funcionar" no fim do curso é bug de desenho, não guard.

### 9.3 — O preset é PROMOVÍVEL, e é isso que impede dois donos

`warp` + `bend` são a **lembrança de de-onde-a-gaiola-veio**, não um segundo dono dela: a derivação é
de **mão única e por EVENTO** (mudou o preset ou o bend ⇒ re-escreve a gaiola), **nunca por frame**.
**Arrastar qualquer alça solta o preset** (`warp = None`) e o Bend deixa de ser oferecido — sem essa
regra o próximo toque no slider apagaria o gesto do artista sem aviso.

Decisões menores que a próxima fatia vai encostar: o preset **põe a gaiola em Mesh** (com lados retos
não há preset a exprimir) · ele **reseta os cantos ao repouso** (*Reset with Warp*: pedir um arco
depois de puxar um canto dá um arco, não um arco torto) · o `bend` nasce em `0.5` e **não em zero**,
porque `bend = 0` **é** a gaiola em repouso ao bit e o primeiro clique em "Arc" não moveria um pixel
· a conversão bipolar (track `0..1` → `-1..1`) mora na **fronteira do painel**, então o shell recebe
o número do documento e nunca sabe que existe um track.

### 9.4 — Dois gates de LOC no caminho, pagos com SPLIT

`apply_event` passou de 200 LOC (o 3º slider bipolar foi a gota) ⇒ `forward_track`, que colapsa as
**três cópias do mesmo corpo** (Bend, Morph `t`, Blend Steps). E os literais do `populate` viraram
constantes nomeadas **do domínio do documento** (HR-15, `no_magic_numeric`). Nenhum allowlist.

### 9.5 — Smoke da Fatia C

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  cargo run -p ph2d-host-desktop --features panel-vector
```

1. Envolva uma forma (**Envelope**) → aparecem os **7 botões de preset**.
2. Clique **Arc** → a forma curva **na hora** (o bend nasce em 0.5; um preset que precisasse de dois
   cliques para se anunciar seria um botão morto na estreia).
3. O slider **Bend** aparece. Arraste-o: a forma re-carimba ao vivo, e a ponta **esquerda** é o arco
   ao contrário (a faixa é bipolar).
4. Percorra os 7 — cada um tem de dar uma forma **diferente**; nenhum "para de funcionar" no fim do
   curso do slider.
5. Modo **Node** → arraste uma alça: **o Bend some** (a gaiola virou manual). Clique um preset de
   novo e ele volta.
6. Com **Perspective** aceso, clicar um preset o troca para **Mesh** sozinho.

---

## §10 — Fatia E: os pinos (`adc2f228`)

### 10.1 — A jacobiana é fechada, e o que a tornou escrevível foi um cancelamento

O `Warp` exige a derivada REAL (uma diferença finita faz o `fit_to_bezpath` **não convergir** — ele
*trava*, não falha), e derivar `f = (S/|S|)(v − p⋆) + q⋆` parece proibitivo porque `p⋆`, `q⋆` e `S`
são **todos** função de `v`. Até se notar que **`Σwᵢp̂ᵢ = 0` e `Σwᵢq̂ᵢ = 0` por definição do centróide
ponderado**: os dois termos de correção de `∂S` são exatamente essas somas e **desaparecem**,
deixando

```
∂S/∂v = Σ (∂wᵢ/∂v)·q̂ᵢ·conj(p̂ᵢ)      com  ∂wᵢ/∂v = wᵢ/(pᵢ − v)
```

O resto é regra do quociente em cálculo de **Wirtinger** (`∂/∂v` e `∂/∂v̄` por estágio), e a jacobiana
real sai de `A = ∂f/∂v` e `B = ∂f/∂v̄`. Quem for mexer nisto: o gate de diferença central é o oráculo,
e ele mata as 4 mutações estruturais (R constante · p⋆ constante · termo conjugado descartado · sinal
de `∂w`).

### 10.2 — Os dois guards NÃO são casos especiais: são o método

- **`v` em cima de um pino** ⇒ devolve o pino movido (o guard do Krita que o ADR mandou copiar). Sem
  ele, pregar em cima de uma âncora é `NaN` no 1º frame.
- **`|S| ≈ 0`** ⇒ **translação pura**. E este guard **É** o caso de 1 pino (com um só, `p̂ = q̂ = 0`),
  que é o critério de aceitação #5 do ADR — não um ramo escrito à mão.

### 10.3 — ⚠️ A dobra foi respondida pela OUTRA ponta, e isso REVOGA um aviso do próprio ADR

O `fit.rs` dizia que a Fatia E quebraria a premissa *"nenhum mapa em escopo dobra"* e que quem
levasse o MLS adiante teria de implementar `break_cusp`. **A resposta desta linha à degenerescência é
sempre a mesma — torná-la inalcançável pela mão** — então `pins_fold` **recusa o arrasto que
dobraria**, e o `break_cusp` em `None` volta a ser honesto. O docstring dele foi reescrito
(comentário velho mente).

Não é preguiça: aproximar um fold com carinho produz um contorno **auto-interseccionado** (a saga da
lasca da booleana), não um bico bem fitado. **Preço registrado:** não se torce um pino além de ~90°.
É limite do MÉTODO (o ADR mediu `det J` mudar de sinal aí), não do guard.

### 10.4 — Duas coisas contra-intuitivas que o smoke vai encontrar

- **Com 2 pinos não se deforma nada.** Uma isometria de um par determina uma rigidez ÚNICA ⇒ o mapa
  devolve movimento rígido do plano inteiro. **É preciso um 3º pino não-colinear.** Há gate no motor
  e no produto; quem for depurar *"os pinos não fazem nada"* tem de os encontrar antes de mexer na
  matemática.
- **O suporte é GLOBAL** — o deslocamento cresce com a distância, e nenhum parâmetro conserta (o α do
  paper não tem efeito no campo distante; foi por isso que o slider *"Flexibility"* do Krita foi
  **recusado** no ADR §6). A mitigação é estrutural: **o container é o escopo**.

⚠️ **E o contra-sinal do ADR continua de pé:** se o smoke mostrar que o gesto quer **posar
personagem** (membro perto do tronco), o MLS **vai** falhar e nenhum parâmetro salva — a decisão é
**reaberta** (ARAP/LBS), não calibrada.

### 10.5 — Uma mutação SOBREVIVEU, e o gate que faltava

**Hit-testar o pino pela posição de REPOUSO** passou verde: o fixture nunca tinha arrastado um pino
antes de o agarrar, então `rest == moved` e o teste não conseguia distinguir. Gate estendido (arrasta
→ agarra no lugar NOVO → confirma que o lugar VELHO já não pega) ⇒ a mutação morre.

E **dois oráculos meus** que os gates derrubaram: *"mover 1 de 2 pinos não deforma"* é **falso**
(mover um só muda a distância entre eles, logo não é isometria — a afirmação é sobre movimento
RÍGIDO do par); e medir forma por **bbox alinhada ao eixo** não é invariante a rotação (acusou o par
isométrico de encolher 1%; o que encolheu foi a caixa de um círculo cozido em 4 cúbicas). Rigidez
preserva **distância** — o oráculo agora é o diâmetro do conjunto de âncoras.

### 10.6 — ⚠️ O undo fazia o overlay SUMIR (fix `a64a9ced`, pós-smoke)

*"com pins o undo faz os pins sumirem (embora ainda funcionando, estão invisíveis)"* (Enio).

**A causa não estava no envelope, e por isso vale a pena ler:** o `vec_selection` lia *"os meus bits
sumiram do gizmo"* como **"a árvore deselecionou"**, quando significava **"o mundo foi re-spawnado
debaixo de mim"** — o `ProjectState::restore` despawna o editável e re-spawna com **ids NOVOS**.
Resultado: o ramo 2 limpava o pen e o gizmo ficava para sempre com bits de uma entidade morta.

O sintoma é o que o torna confuso: **a ferramenta continua a funcionar e fica invisível.** O recook
varre por QUERY (a arte segue deformada); o overlay é desenhado pelos BITS (que já não nomeiam
nada). **A gaiola sofria do mesmo mal, em silêncio** — os três gestos desenham pelos mesmos bits.

Fix: *"sumiram"* e *"morreram"* são fatos diferentes, e distingui-los é barato (uma entidade
deselecionada continua **existindo**). Todos os bits mortos ⇒ respawn ⇒ re-deriva do pen, cuja
seleção é `VecPathId` e viaja no snapshot.

⚠️ **A mutação `all` → `any` SOBREVIVEU, e a diferença é real:** com *algum* bit morto (delete de uma
de duas formas selecionadas) a verdade está no **gizmo**, que ainda tem os sobreviventes — re-derivar
do pen ali deixaria o id da forma apagada pendurado na seleção para sempre. O gate novo
(`deleting_one_of_two_selected_shapes_prunes_the_pen`) é o **par** do gate de undo: os dois falam de
bits mortos e pedem respostas **opostas**.

> **Para a próxima linha que desenhar um overlay:** identidade de overlay por `Entity` bits é frágil
> por construção neste app — o undo respawna tudo. Ou se deriva a identidade de algo estável
> (`VecPathId`, `Name`) a cada frame, ou se detecta o respawn. A timeline resolveu o mesmo problema
> pelo `wire_id`; aqui foi pela detecção.

### 10.7 — Smoke da Fatia E

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  cargo run -p ph2d-host-desktop --features panel-vector
```

1. Envolva uma forma → chip **Pins** (a fileira Cage agora tem 3). Clique nele: a gaiola **some**.
2. Modo **Node**: clique em 3 pontos da arte → 3 pinos (bolinha cheia + circulinho de repouso).
3. Arraste o 3º → **agora** deforma. ⚠️ **Com só 2 pinos a arte MOVE mas não deforma — isso é o
   método, não um bug** (§10.4).
4. Puxe um pino para muito longe → ele **para** (o guard de dobra).
5. **Clear Pins** → tudo volta.
6. Voltar a **Persp**/**Mesh** traz a gaiola de volta, e os pinos ficam guardados.
7. **Ctrl+Z** (o fix `a64a9ced`): os pinos — e a gaiola nos outros gestos — **continuam visíveis**.

---

## §11 — Os fixes PÓS-SMOKE (leia esta seção antes de qualquer outra)

Quatro defeitos que só o smoke do Enio expôs. Estão aqui em primeiro lugar porque **três deles são
lições transferíveis** — não são detalhes do envelope.

### 11.1 — O undo fazia o overlay SUMIR (`a64a9ced` + `30d7115c`)

*"o undo faz os pins sumirem, embora ainda funcionando"*. O `ProjectState::restore` despawna o mundo
editável e re-spawna com **ids NOVOS**, e o `apply_project` zerava a seleção inteira por causa disso.
A defesa está certa quanto aos **bits** — mas levou junto a seleção do **pen**, que não é feita de
bits (`VecPathId`, e ela **viaja no snapshot**).

Sintoma traiçoeiro, e vale memorizar a forma dele: **a ferramenta funcionando e invisível.** O recook
varre por QUERY (a arte segue deformada); o overlay é desenhado pela SELEÇÃO (que já não existe).

⚠️ **O 1º fix (`a64a9ced`) estava num caminho que o produto não percorre** — gateei o
`sync_selection` diretamente e ele ficou verde sobre um mecanismo que o undo nunca alcança. O
`apply_project` **exige `gfx`** (janela + GPU) e por isso nenhum teste headless chega lá. A resposta
foi extrair a POLÍTICA para uma função pura (`surviving_selection`, 3 gates) **+ um arch-gate sobre o
FONTE** provando que o `apply_project` a chama, e que a captura vem ANTES do restore.

> **Para o integrador:** `shells/desktop/tests/the_undo_preserves_the_vector_selection.rs` lê
> `src/undo.rs` por texto. Se o merge reformatar o `apply_project`, ele fala — é intencional.

### 11.2 — "Os pontos travam ao arrastar" (`f5d59c96`)

**Medido, não deduzido:** arrastar uma âncora de uma forma envolvida move o ponto por um frame e o
`recook` o **reverte ao bit** no seguinte. O ponto anda e volta.

A geometria de um filho é COZIDA — função pura das fontes + gaiola. Agora um clique sobre a ARTE do
envelope é **consumido sem armar nada**; clique fora dela continua a cair no pen (desselecionar
funciona). É a mesma regra da alça de raio numa Live Shape (ADR-0121): **geometria que uma relação
viva possui não é editável à mão**; quem quer os pontos de volta usa **Expand**.

**Duas gates existentes falharam com este fix, e as duas estavam CERTAS** — foram reescritas para
afirmar o fato que importa (uma sondava o CENTRO da forma; a outra media o RETORNO do `press` quando
a pergunta sempre foi se ele *armou*).

### 11.3 — O guard de dobra perguntava pela CAIXA (`2e2a951d`)

O `PH2D_VEC_OVERLAY_DIAG` nomeou: `pinos=13` + `RECUSADO pino: ... dobraria a arte`. Quantificado:
com 13 pinos o guard recusava qualquer arrasto além de **0,70 unidades num domínio de 2,80**.

A recusa nem sempre estava errada; **a pergunta estava**. O mal que ele impede é um contorno
**auto-interseccionado**, e um contorno só se auto-intersecta onde HÁ contorno — eu amostrava a
bbox-união, que tem cantos por onde nenhuma curva passa. Agora as amostras são os pontos da **arte**
(`envelope_live::art_samples`).

E **Alt+clique remove um pino** (idioma do Puppet Warp): 13 pinos era acúmulo, não autoria — todo
clique no vazio prega e o `Clear Pins` é tudo-ou-nada. Isto fecha o *"aberto de propósito"* que o §6
listava; a disputa de UX resolveu-se com a evidência.

### 11.4 — ⚠️ Um artefato visual ABERTO, e o que já está EXCLUÍDO

O Enio fotografou uma **linha reta longa** saindo de uma forma envolvida. **Não reproduziu** no
re-smoke e **não tem causa conhecida**. O que a investigação já ELIMINOU, por medição — para ninguém
repetir:

1. **Cancelamento catastrófico na jacobiana do MLS perto de um pino.** Falso: com os pinos em repouso
   o mapa é a identidade e `J` mede exatamente `[[1,0],[0,1]]` até 1e-13 do pino.
2. **Haste de comprimento zero no overlay** (pino recém-pregado tem `rest == moved`). Falso: o
   stroker devolve ZERO elementos.
3. **Segmento reto/degenerado `(P0,P0,P3,P3)` a chegar ao fitter.** Falso: retângulo, retângulo
   arredondado e a degenerada explícita atravessam `warp_path` (Quad e MLS) sem ponto disparado.
4. **Ponto de controle disparado na cena.** Falso, e agora com evidência do PRODUTO: o log real do
   `PH2D_VEC_OVERLAY_DIAG` mostrou `alcance_das_alcas=1.00x` em toda forma, em todo frame.

A cor da linha na foto é a do **traço da forma**, não a do overlay (azulado) — o que aponta para
geometria/render, não para o chrome do envelope. **Não é bloqueio de integração** (não reproduz, e
nada no diff o explica), mas fica registrado com as 4 portas já fechadas.

> `PH2D_VEC_OVERLAY_DIAG=1` fica no código de propósito. ⚠️ A 1ª versão dele **mentia**: passei
> `undo.depth()` como contador de frames, e `depth % 60 == 0` só é verdade em 0/60/120 — ele
> emudecia na primeira ação desfazível, e o smoke concluiu "não reproduziu" sem ter sido observado.
> O contador agora é interno: **não há parâmetro para passar errado**, o que é melhor que um gate.

---

## §12 — RUNBOOK do integrador

Sem rebase: `merge-base == main == cdc3acc1`, **0 commits** de drift.

```
cd /home/enio/Documentos/Projetos/PH2D
git merge --ff-only line/Vector
```

**Depois do merge, rode o gate da árvore combinada** (DIRETRIZ §1.5.3). O que ESTA linha já verificou
localmente no HEAD `73d59ff4`, e que o integrador deve ver verde de novo:

| Gate | Estado local |
|---|---|
| `cargo check --workspace --all-targets` | ✅ |
| `cargo fmt --all -- --check` | ✅ |
| clippy `--all-targets` nas 6 crates tocadas | ✅ sem um warning |
| 33 binários de arch-gate da `ph2d-editor-core` | ✅ |
| `ph2d-host-desktop` tests + bins (**731**) | ✅ |
| `ph2d-vec-envelope` (42 unit + 8 do gate-mãe) | ✅ |
| `ph2d-panel-vector` seam (**25**) | ✅ |
| `typos` · `cargo machete` | ✅ (drenados em `73d59ff4`) |

⚠️ **Latentes que o ship costuma acordar nesta linha** (2-4 iterações é o normal): `typos` já foi
drenado, mas ele **varre a árvore combinada** — outra linha pode trazer palavra nova. E o gate de LOC
do **shell** mora em `shells/desktop/tests/file_loc_caps.rs` e **não roda** com `cargo test -p
ph2d-editor-core`; esta linha já pagou esse pedágio duas vezes.

### Pontos de atrito prováveis num merge com outra linha

| Arquivo | Por quê |
|---|---|
| `crates/ph2d-ecs/src/lib.rs` | 1 linha de `pub use` (append) |
| `crates/ph2d-editor-core/src/ids/chrome/vector.rs` | bloco **append-only** no fim + `VECTOR_SECTIONS` |
| `crates/ph2d-editor-core/tests/node_id_collisions.rs` | tabela (append) |
| `crates/ph2d-editor-core/tests/arch_mode_has_reconcile.rs` | 1 entrada em `BENIGN_SET_MODE` |
| `.typos.toml` | 1 chave nova — ⚠️ **chave duplicada mata o gate no PARSE**, confira antes de resolver |
| `shells/desktop/src/render_loop/mod.rs` | drain + publisher + 2 blocos de overlay |
| `shells/desktop/src/main.rs` | 2 `mod` novos |

**Nenhuma contagem "que soma"**: nenhum componente NOVO foi registrado (o `VecEnvelope` já existia e
só mudou de forma), então a tríade `ph2d-ecs`/`-render`/`-script` está intacta.

---

## §13 — Resumo de fechamento

- **Fatias 1–5 do Envelope (ADR-0129 §4.A.1–5) construídas e gateadas.** F1/F2/F3 SMOKADAS; F4+F5
  pendentes.
- **Fatia 3:** o envelope virou um **container de N filhos** (*warp group*). `create` síncrono
  (assa em mundo + reparenta na identidade + pendura); `recook(sim, scene)` por QUERY; gesto por bits
  do container; **seleção-só-o-container** (senão os filhos cisalham); gizmo = **caixa-união**.
- **Fatias 4+5:** a seção **Envelope** no painel (Create sempre; **Expand**/**Release** só com um
  envelope na seleção) — a feature deixou de existir só atrás de uma env. Expand e Release são o
  MESMO `dissolve` (enum `Keep`), e a **pose do container é assada na geometria** ao dissolver: o
  inverso exato da entrada, e o que impede a arte de teleportar.
- **Foundational tocado:** `VecEnvelope` mudou de FORMA (`{corners, children}`), registrado por nome
  (sem contagem nova, sem contrato congelado). Sem migração (nenhum save tem envelope). +4 ids no
  bloco append-only do painel vetorial.
- **Gates:** 24 no envelope (shell) + 2 de seleção/gizmo + 2 de seam que CLICAM; clippy/fmt/LOC/tofu/
  a11y/colisão-de-id/wiring-parity verdes; **7 mutações-chave provadas** (3 da F3 + 4 das F4/F5).
- **Fatia D:** a gaiola ganhou **dois gestos** (Perspective/Mesh) — ver §8. **Nenhum é o outro com um
  knob:** bilinear ≠ projetivo, e o par de gates crava que eles concordam em repouso e divergem fora
  dele. +2 ids, +1 crate-módulo (`coons.rs`), `VecEnvelope` ganhou `edges` + `kind`.
- **Gates da D:** 8 no motor Coons + 6 no gesto + 4 no host + **2 no GATE-MÃE sobre o mapa de
  produção** (o de invariância à subdivisão rodava só sobre um `Bilinear` escrito à mão) + os 2 de
  seam do painel **estendidos** (as listas de "o que a seção oferece" ganharam os chips — uma lista
  nova driftaria). **9 mutações RED→GREEN.**
- **Fatia C:** 7 presets que GERAM a gaiola + Bend, **promovíveis** — ver §9. A `AMP` foi **medida**,
  não escolhida: o primeiro valor comprava a garantia só para as formas que eu já tinha escrito.
- **Gates da C:** 7 no motor + 6 no host (a varredura dos presets REAIS) + 4 de seam. **9 mutações
  RED→GREEN.** Dois gates de LOC pagos com split, nenhum allowlist.
- **Fatia E:** os **pinos** (MLS-rigid) — ver §10. **Um fix pós-smoke:** o undo fazia o overlay
  (gaiola E pinos) sumir — a ferramenta funcionando e invisível — porque a seleção espelhava bits de
  entidade e o `restore` respawna tudo (§10.6). A jacobiana é **fechada** (Wirtinger + o
  cancelamento do centróide), e a dobra é **recusada** em vez de aproximada, o que **restitui** a
  premissa do `break_cusp` que o ADR dava por perdida.
- **Gates da E:** 9 no motor + 2 no gate-mãe + 7 no host + 1 de seam. **8 mutações, 1 SOBREVIVENTE →
  gate novo** (§10.5). Dois oráculos meus caíram no caminho.
- **Commits (locais, sem push):** `adc2f228` (Fatia E) · `3f4e0c91` (Fatia C) · `5b6f754c` (Fatia D) · `d5695795` (F4+F5) · `10889f0e` (F3) ·
  `5bddd9e4`/`43b918f5`/`207d10b9` (F2/F1) + docs.
- **Fixes pós-smoke:** 4, todos gateados (§11) — e três com lição transferível: *a ferramenta
  funcionando e invisível* (identidade de overlay por bits de entidade, que o undo recicla) · *o
  recook é o dono da geometria derivada* (não a ofereça à mão) · *o guard deve perguntar pela
  geometria que EXISTE, não pela caixa em volta dela*.
- **Um artefato ABERTO** (a linha reta da foto): não reproduz, sem causa, **4 hipóteses eliminadas
  por medição** (§11.4). Não é bloqueio.
- **Ship latents drenados** (`73d59ff4`): `typos` limpo, `machete` limpo, fmt e clippy sem um
  warning, `check --workspace` verde.
- **Ordem do Enio recebida (2026-07-18): INTEGRAR.** A linha entrega este handoff e **PARA aqui** —
  quem funde é o **agente integrador** (§0.7 do CLAUDE.md), e o **ship/push continua a ser do Enio**,
  por ordem separada. Runbook: **§12**.
