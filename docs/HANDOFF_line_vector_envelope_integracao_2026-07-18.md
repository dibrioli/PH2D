# HANDOFF de INTEGRAÇÃO — `line/Vector`: Envelope Fatias 1 + 2 + 3 (ADR-0129)

**Para:** o **agente integrador** (e o próximo implementador da linha).
**De:** a sessão de 2026-07-17/18 que assumiu a linha pelo `HANDOFF_line_vector_continuacao_2026-07-17.md`
(§4.A itens 1–3 do Envelope).
**Estado:** **Fatias 1 e 2 fechadas e SMOKADAS pelo Enio; Fatia 3 fechada e gateada, PENDENTE de smoke.**
Motor + host live já estavam na `main` (Fatias A+B).
- **Fatia 1** = a alça própria de canto no Node (arrasta os 4 cantos, convexidade obrigatória).
- **Fatia 2** = mover/girar/escalar o envelope inteiro no Select (geometria LOCAL + pose no `Transform`).
- **Fatia 3** = o **CONTAINER de N filhos** (*warp group*): o `VecEnvelope` saiu do path e foi para uma
  **entidade-container** (sem path próprio) com `children: Vec<VecEnvelopeChild>`. Um só modelo — 1 filho
  é o caso `N=1`, vários é o grupo.

> Este handoff **supersede** o `HANDOFF_line_vector_envelope_fatia1_integracao_2026-07-17.md` (removido).
> A **descrição do modelo da Fatia 2** ("envelope = path com componente") está superada pela Fatia 3
> ("envelope = container; os paths são FILHOS"). A mecânica de pose/gizmo é a MESMA, um nível acima.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **Commits da fatia** | `207d10b9` (F1) · `43b918f5` (fix smoke F1) · `5bddd9e4` (F2: local+pose) · **`10889f0e` (F3: container)** · docs |
| **Base do fork (merge-base com `main`)** | `cdc3acc1` |
| **`main` desde a base** | **0 commits** — a linha está sobre a `main` de hoje; **sem rebase** |
| **Contratos congelados encostados** | **NENHUM** (§4) |
| **Smoke** | F1+F2 **APROVADOS (2026-07-17)**. **F3 PENDENTE** — ver §5. |

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

**3 mutações-chave confirmadas nesta sessão (mutei → RED sobre visto-verde → restaurei):**
- desligar `sole_envelope_container` → gizmo fica `[filho, container]` (2 bits) → gate RED.
- `container_view` usar só o 1º filho → caixa não une → gate RED.
- pular `bake_xform` no `create` → fonte fica local-centrada (x≈-2 em vez de ≈38) → gate RED.

**Smoke — PENDENTE do Enio (Fatia 3):** (caminho ABSOLUTO — o relativo só funciona a partir da raiz
do repo, e é onde este comando já falhou uma vez)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  PH2D_BUILD_SMOKE=12 cargo run -p ph2d-host-desktop --features panel-vector
```
- **NODE:** duas elipses sob UMA gaiola; arraste um canto → as duas re-deformam juntas pela mesma
  perspectiva (a prova é a curva LISA entre os cantos, não o canto obedecer).
- **SELECT** (pill do painel): o gizmo abraça as DUAS (caixa = união) e move/gira/escala o grupo
  inteiro **sem cisalhar**.
- `PH2D_BUILD_SMOKE=11` (uma elipse, o caso `N=1`) deve continuar idêntico ao smoke aprovado de F1/F2.

---

## §6 — A FILA (a ordem é do Enio; ADR-0129 §Plano é a fonte)

Fatias 1, 2 **e 3** fechadas. Restam da 4.A (fechar o Envelope):

4. **Release / Expand** — materializar a(s) deformada(s) como forma(s) comum(ns) e soltar a gaiola
   (dissolver o container, devolver os filhos como raízes com a geometria cozida). ← **próximo**
5. **O painel** (seção Envelope docada: Fidelity/`accuracy` + presets + escolha de gesto + botão
   "Envelope" que chama `envelope_live::create` sobre a seleção — hoje só o smoke o chama).
6. **Os outros gestos** (cada um é um `impl Warp` novo): C presets · D 4-curvas/Coons · E pinos/MLS
   (a mais delicada — exige o `break_cusp` que hoje volta `None` de propósito; ADR §3.2).

E a 4.B herdada (Live Path Effects como nós, morph vivo, blend em cadeia, etc.).

**Notas para quem faz Release/Expand:** as fontes dos filhos são `child.source` (mundo, no nascimento).
Expandir = escrever a geometria COZIDA (deformada) de cada filho no path dele (já está lá, o recook a
mantém), remover o `VecEnvelope`, e **des-parentar** os filhos (devolver `RootOrder`, remover `ChildOf`)
— espelho de `vec_entities::ungroup_entities`, que já faz exatamente o des-parenteamento. O container
vira um grupo vazio → despawn. NÃO precisa re-baka: o path do filho já está em container-local; ao
des-parentar com a pose do container ≠ identidade, ou você assa a pose do container na geometria do
filho (como o `create` fez na entrada), ou transfere a pose do container para cada filho. A 2ª é mais
simples se houver 1 filho; a 1ª (assar) é robusta p/ N filhos — decida no ADR se o Enio quiser preservar
a pose do container.

---

## §7 — Resumo de fechamento

- **Fatias 1, 2 e 3 do Envelope (ADR-0129 §4.A.1–3) construídas e gateadas.** F1/F2 SMOKADAS; F3 pendente.
- **Fatia 3:** o envelope virou um **container de N filhos** (*warp group*). `create` síncrono
  (assa em mundo + reparenta na identidade + pendura); `recook(sim, scene)` por QUERY; gesto por bits
  do container; **seleção-só-o-container** (senão os filhos cisalham); gizmo = **caixa-união**.
- **Foundational tocado:** `VecEnvelope` mudou de FORMA (`{corners, children}`), registrado por nome
  (sem contagem nova, sem contrato congelado). Sem migração (nenhum save tem envelope).
- **Gates:** 20 no envelope + 2 novos (seleção/gizmo), clippy limpo, tofu limpo, 3 mutações-chave provadas.
- **Commits (locais, sem push):** `10889f0e` (F3) + `5bddd9e4`/`43b918f5`/`207d10b9` (F2/F1) + docs.
- **A linha NÃO integra nem faz push** (§0.7): entrego este handoff e **PARO** — integração e ship só
  por ordem EXPLÍCITA do Enio.
