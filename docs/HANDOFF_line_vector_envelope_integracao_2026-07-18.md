# HANDOFF de INTEGRAÇÃO — `line/Vector`: Envelope Fatias 1–5 (ADR-0129)

**Para:** o **agente integrador** (e o próximo implementador da linha).
**De:** a sessão de 2026-07-17/18 que assumiu a linha pelo `HANDOFF_line_vector_continuacao_2026-07-17.md`
(§4.A itens 1–5 do Envelope).
**Estado:** **Fatias 1, 2 e 3 fechadas e SMOKADAS pelo Enio; Fatias 4+5 fechadas e gateadas, PENDENTES
de smoke.** Motor + host live já estavam na `main` (Fatias A+B).
- **Fatia 1** = a alça própria de canto no Node (arrasta os 4 cantos, convexidade obrigatória).
- **Fatia 2** = mover/girar/escalar o envelope inteiro no Select (geometria LOCAL + pose no `Transform`).
- **Fatia 3** = o **CONTAINER de N filhos** (*warp group*): o `VecEnvelope` saiu do path e foi para uma
  **entidade-container** (sem path próprio) com `children: Vec<VecEnvelopeChild>`. Um só modelo — 1 filho
  é o caso `N=1`, vários é o grupo.
- **Fatias 4+5** = a **seção Envelope no painel** (Create) + as duas saídas (**Expand**/**Release**).
  Até elas, `envelope_live::create` só era chamado pela env `PH2D_BUILD_SMOKE`: a feature existia no
  motor, gateada e smokada, e **não existia para o artista**.

> ⚠️ **A ordem da fila do ADR foi INVERTIDA de propósito** (ele lista 4=Release, 5=painel). Motivo: o
> Release é um BOTÃO — sem a seção do painel ele não teria onde morar (ou viraria um atalho de teclado,
> 2ª porta para a mesma pergunta); e criar sem desfazer é **porta de mão única**. Os dois viraram uma
> fatia só.

> Este handoff **supersede** o `HANDOFF_line_vector_envelope_fatia1_integracao_2026-07-17.md` (removido).
> A **descrição do modelo da Fatia 2** ("envelope = path com componente") está superada pela Fatia 3
> ("envelope = container; os paths são FILHOS"). A mecânica de pose/gizmo é a MESMA, um nível acima.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **Commits da fatia** | `207d10b9` (F1) · `43b918f5` (fix smoke F1) · `5bddd9e4` (F2: local+pose) · `10889f0e` (F3: container) · **`d5695795` (F4+F5: painel + Expand/Release)** · docs |
| **Base do fork (merge-base com `main`)** | `cdc3acc1` |
| **`main` desde a base** | **0 commits** — a linha está sobre a `main` de hoje; **sem rebase** |
| **Contratos congelados encostados** | **NENHUM** (§4) |
| **Smoke** | F1+F2+F3 **APROVADOS (2026-07-17/18)**. **F4+F5 PENDENTES** — ver §5. |

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

6. **Os outros gestos** (cada um é um `impl Warp` novo): C presets · D 4-curvas/Coons · E pinos/MLS
   (a mais delicada — exige o `break_cusp` que hoje volta `None` de propósito; ADR §3.2). ← **próximo**

E a 4.B herdada (Live Path Effects como nós, morph vivo, blend em cadeia, etc.).

**Aberto no Envelope, de propósito (não são bugs — são escopo não pedido):**
- **Fidelity/`accuracy` não é exposta.** O `accuracy` é relativo (0,1% da diagonal da união) e não
  tem knob. Quando houver queixa de "a curva perdeu detalhe", é um slider na seção — e o lugar dele
  já está lá.
- **Presets de gaiola** (arco, bandeira, perspectiva…) — cada um é só um conjunto de `corners`; a
  porta (`VecEnvelope.corners`) já aceita. É UI, não motor.
- **Envelope aninhado** (envolver um envelope). O `container_of` **já sobe a cadeia** e pára no mais
  próximo, então a seleção e o dissolve se comportam; o que NÃO foi exercitado é o `create` sobre um
  container (ele resolve `ids` de PATHS, e um container não tem path) — hoje envolver um envelope
  envolve os **filhos** dele. Se isso for pedido, é uma decisão de modelo, não um fix.
- **A gaiola só é editável no Node**, e isso é ADR §3.3 (cerca de Chesterton — um gizmo sobre
  geometria que se move dobra; 5 tentativas revertidas no Blend).

---

## §7 — Resumo de fechamento

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
- **Commits (locais, sem push):** `d5695795` (F4+F5) · `10889f0e` (F3) · `5bddd9e4`/`43b918f5`/
  `207d10b9` (F2/F1) + docs.
- **A linha NÃO integra nem faz push** (§0.7): entrego este handoff e **PARO** — integração e ship só
  por ordem EXPLÍCITA do Enio.
