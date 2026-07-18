# HANDOFF de INTEGRAÇÃO — `line/Vector`: Envelope Fatias 1 + 2 (arrastar cantos + mover no Select, ADR-0129)

**Para:** o **agente integrador** (e o próximo implementador da linha).
**De:** a sessão de 2026-07-17 que assumiu a linha pelo `HANDOFF_line_vector_continuacao_2026-07-17.md`
(§4.A itens 1 e 2 do Envelope).
**Estado:** **Fatias 1 e 2 fechadas e SMOKADAS pelo Enio.** Motor + host live já estavam na `main`
(Fatias A+B). **Fatia 1** = a alça própria de canto no Node. **Fatia 2** = mover/girar/escalar o
envelope inteiro no Select, que exigiu tornar o envelope um **objeto de geometria LOCAL + pose no
`Transform`** (o modelo correto — o antigo forçava identidade + assava em mundo). Integração de
**baixo risco**: nenhum foundational (só docstrings em `vec_envelope.rs`), nenhum contrato congelado,
nenhuma contagem de registro mexida, **nenhuma mudança de schema** (mesmos bytes; muda só o FRAME
semântico da fonte/cantos: mundo → local).

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **Commits da fatia** | `207d10b9` (F1 feature) · `43b918f5` (fix do smoke — ver abaixo) · `5bddd9e4` (F2: local+pose) · docs neste arquivo |
| **Base do fork (merge-base com `main`)** | `cdc3acc1` |
| **`main` desde a base** | **0 commits** — a linha está em cima da `main` de hoje; **não precisa rebase** |
| **Contratos congelados encostados** | **NENHUM** (§4) |
| **Smoke** | **APROVADO pelo Enio (2026-07-17), Fatias 1 e 2** — `cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=11 cargo run -p ph2d-host-desktop --features panel-vector`: **NODE** = 4 cantos arrastáveis, re-deforma ao vivo, canto p/ dentro para na fronteira convexa (F1); **SELECT** = o gizmo move/gira/escala o envelope inteiro sem dobrar (F2). |

> ⚠️ **Fix pós-1º-smoke (`43b918f5`), e é um bug do DRIVER do smoke, não da fatia:** o 1º smoke não
> mostrava a gaiola. O envelope estava certo (o `recook` achava 1 envelope já deformado), mas o frame
> vinha em `mode=Build`: o arm `8 =>` do `build_smoke` era **catch-all** e forçava `DrawMode::Build`
> em TODO nível no frame 8, engolindo o Node que a cena 11 arma no frame 4 (sem Node,
> `overlay.edit`/`envelope_cage` são falsos e a alça não desenha). Gate `8 if level <= 6` (o arm é do
> Shape Builder). Corrige de quebra um latente: o **morph (nível 10) era jogado no Build** sem querer.
> +gate `the_upkeep_path_attaches_and_deforms` (o caminho REAL do app: `upkeep` anexa, não `attach`
> direto). Nenhum código de produto mudou — só o smoke e um gate novo.

---

## §2 — O que a fatia entrega (30 s)

Os 4 cantos da gaiola do Envelope viraram **alças próprias, arrastáveis no modo Node**. Antes o
`create`/`attach` só era exercitado pela cena de smoke e a gaiola era estática; agora é uma ferramenta:
seleciona o envelope, entra no Node, arrasta um canto → a homografia (`QuadWarp`) é re-cozida a cada
frame pelo `envelope_live::recook` que já existia. A **convexidade é obrigatória** (ADR-0129 §5): um
movimento que tornaria a gaiola não-convexa é recusado e o canto **para na fronteira** (o horizonte
fica fora da gaiola, sem clipping e sem epsilon).

**Por que alça PRÓPRIA e não o gizmo de sprite (§3.3):** a geometria de mundo do envelope é reescrita
pelo recook a cada frame; um gizmo pendurado nela giraria em torno de uma bbox que muda debaixo dos
pés (a lição de 5 tentativas revertidas do Blend, ADR-0128). No **Select** quem se move é a forma (o
gizmo); a gaiola só aparece e se edita no **Node**.

**Por que HOST e não PenTool:** a gaiola vive num **componente ECS** (`VecEnvelope.corners`), não como
âncora de um path da cena. O `PenTool` só conhece o `VecScene`. Então o gesto é do host — o padrão do
`blend_live` (gesto de Node que toca o ECS), não o do pen.

---

## §3 — Riscos de INTEGRAÇÃO (DIRETRIZ §1.5.9.2–3)

### 3.1 Foundational tocado

**NENHUM.** O componente `VecEnvelope` já existia e já estava registrado (Fatia B). Esta fatia **não
adiciona componente ECS**, então:

- **NÃO há "números que somam"** (as 3 contagens de registro em `ph2d-ecs`/`-render`/`-script` estão
  **intocadas**). Um merge que traga outro registro de componente **não colide** com esta fatia.
- O gate `settle_skips_every_derived_geometry` está **intocado** (o `VecEnvelope` já estava no `DERIVED`).

### 3.2 O que foi tocado (tudo aditivo)

| Arquivo | O quê | Forma |
|---|---|---|
| `crates/ph2d-vec-envelope/src/gesture.rs` | **NOVO** — `nearest_corner` + `move_corner_convex` (puro) | módulo próprio |
| `crates/ph2d-vec-envelope/src/lib.rs` | `mod gesture;` + `pub use` | aditivo |
| `crates/ph2d-vec-render/src/envelope.rs` | **NOVO** — `EnvelopeCageView` + `draw_envelope_cage` + `ENVELOPE_HANDLE_R_PX` | módulo próprio |
| `crates/ph2d-vec-render/src/lib.rs` | `mod envelope;` + `pub use` | aditivo |
| `shells/desktop/src/envelope_gesture.rs` | **NOVO** — host: `corners_of`/`press`/`drag`/`view` | módulo próprio |
| `shells/desktop/src/envelope_gesture_tests.rs` | **NOVO** — 8 gates de host | módulo próprio |
| `shells/desktop/src/main.rs` | `mod envelope_gesture;` + init `vec_envelope_drag: None` | aditivo |
| `shells/desktop/src/app_state.rs` | campo `vec_envelope_drag: Option<(VecPathId, usize)>` | aditivo |
| `shells/desktop/src/vec_overlay.rs` | flag `envelope_cage` no `VecOverlayPlan` (Node-only) + 1 gate | aditivo (a struct ganhou 1 campo — se outra linha construir `VecOverlayPlan` por literal, precisa do campo; hoje só a função o constrói) |
| `shells/desktop/src/render_loop/mod.rs` | desenha a gaiola após as alças de raio de quina | aditivo (bloco `if overlay.envelope_cage`) |
| `shells/desktop/src/input_dispatch.rs` | press (hit-test antes do pen), CursorMoved (`vec_envelope_corner_move`), release (limpa o drag) | 3 inserções + 1 método |
| `shells/desktop/src/build_smoke.rs` | cena 11 entra no Node + seleciona o envelope | aditivo |

**Seam-risk a conferir num merge:** `input_dispatch.rs` e `render_loop/mod.rs` são arquivos quentes
(muitas linhas mexem várias linhas). As inserções são localizadas (o press no braço `None if node_mode`,
o move antes de `vec_pen_drag_move`, o release após o bloco de gradiente, o draw após `draw_corner_handles`).
Um conflito textual aqui é de contexto, não de símbolo — Mergiraf resolve; se não, o gate de compilação da árvore combinada (`foundational-integrate.sh`) pega.

### 3.3 O que SÓ o `ship.sh` pega

Rodei `cargo nextest run --workspace` (7404 verde) + `cargo clippy --all-targets` (limpo) + `cargo fmt`
nos 3 crates tocados. **Não** rodei o `ship.sh` completo (machete/deny/audit/typos) — é do integrador.
Nenhuma dep nova foi adicionada (as 3 crates já dependiam do que uso).

---

## §4 — Contratos congelados (§1.5.9.4)

**Nenhum encostado.** `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` intactos; o
`architecture_vector_contract_surface` (do `ph2d-vector-doc`) intacto. O motor novo `ph2d-vec-*` não é
congelado. Nenhum ADR novo (o ADR-0129 já cobre esta fatia — é a "Fatia 1 da fila" do §Plano dele).

---

## §5 — Estado dos gates e do SMOKE (§1.5.9.6)

**Workspace:** `cargo nextest run --workspace` → **7404 passed, 142 skipped, 0 failed**. Clippy limpo, fmt ok.

| Gate | Onde | Prova (mutação-testada) |
|---|---|---|
| **cerca de raio** | `ph2d-vec-envelope/src/gesture.rs` (`the_radius_is_a_fence`, `nearest_corner_picks…`) | um canto além do raio não é pego; matar o `<= r2` → RED |
| **guard de convexidade** | idem (`pulling_a_corner_into_a_reflex_is_refused`, `a_bowtie_is_refused`) | reflexo/bowtie recusados; trocar `is_convex(...).then_some` por `Some(...)` → RED |
| **fio do host** | `shells/desktop/src/envelope_gesture_tests.rs` (8) | press arma pelo componente certo · drag escreve `corners` (convexo) · **não-convexo congela o canto** · `view` marca só o canto DESTA forma (matar o filtro de dono → RED) · press ignora não-envelope / sem seleção |
| **política de modo** | `shells/desktop/src/vec_overlay.rs` (`the_envelope_cage_belongs_to_node_mode_alone`) | a gaiola só aparece no Node; tool inativa não desenha |
| **Fatia 2: pose sobrevive + não vaza** | `envelope_live_tests.rs` (`the_pose_survives_recook_and_stays_out_of_the_local_geometry`) | pose preservada pelo recook (o antigo forçava identidade) E não entra na geometria local |
| **Fatia 2: cantos atravessam a pose** | `envelope_gesture_tests.rs` (fixture com pose `[100,50]`) | `press`/`drag`/`view` convertem local↔mundo; **`to_world` virar no-op → `press_arms`+`view_draws` RED** |

**Provas de mutação rodadas nesta sessão** (mutei, vi RED sobre visto-verde, restaurei): cerca de raio,
guard de convexidade, filtro de dono do `view` (Fatia 1), conversão local↔mundo da pose (Fatia 2). E a
pose-preservation é **garantia de TIPO** (o `recook` recebe `&SimWorld`, não pode zerar a pose).

**Smoke:** **APROVADO pelo Enio (2026-07-17), Fatias 1 e 2.** `PH2D_BUILD_SMOKE=11`. NODE: arrasta os
cantos (Fatia 1) — a prova da correção é a curva LISA ENTRE os cantos (⚠️ o canto obedecer engana; o
ingênuo também acerta o canto — o gate de invariância à subdivisão automatiza isso). SELECT: o gizmo
move/gira/escala o envelope inteiro (Fatia 2), sem dobrar.

---

## §6 — A FILA (a ordem é do Enio; ADR-0129 §Plano é a fonte)

Fatias 1 **e 2** fechadas. Restam da 4.A (fechar o Envelope):

3. **O container multi-filho** (1 gaiola p/ N formas; hoje é 1-para-1). ← **próximo**
4. **Release / Expand** — materializar a deformada como forma comum e soltar a gaiola.
5. **O painel** (seção Envelope docada: Fidelity/`accuracy` + presets + escolha de gesto).
6. **Os outros gestos** (cada um é um `impl Warp` novo): C presets · D 4-curvas/Coons · E pinos/MLS
   (a mais delicada — exige o `break_cusp` que hoje volta `None` de propósito; ver ADR §3.2 e o
   handoff de continuação §3.2).

E a 4.B herdada (Live Path Effects como nós, morph vivo, blend em cadeia, etc.).

**Como a Fatia 2 foi feita (o molde para quem generalizar):** o envelope virou **objeto normal** —
geometria LOCAL, pose no `Transform` (ADR-0111). O gizmo de sprite (que já aparece para a entidade —
`vec_gizmo_view` só suprime `VecConnector`/`VecBlend`) move a pose NATIVAMENTE, sem dobrar, porque no
Select a geometria local é ESTÁVEL (a do Blend dobrava por depender de fontes que se movem; a do
envelope é função pura de `corners`+fonte FIXOS — ADR §3.3). O `recook` passou a tomar `&SimWorld`:
**por tipo** não pode tocar a pose. As alças da Fatia 1 atravessam a pose (`envelope_gesture::
path_world_xform` = o mesmo afim que `vec_transform::build` publica); as assinaturas e call-sites não
mudaram. ⚠️ A **nota antiga** ("o envelope força identidade → mover reescreve corners+fonte / re-baka")
está **SUPERSEDIDA** — não se re-baka nada; a pose é a do `Transform`.

---

## §7 — Resumo de fechamento

- **Fatias 1 e 2 do Envelope (ADR-0129 §4.A.1–2) construídas, gateadas e SMOKADAS.** Fatia 1: alça
  própria de canto no Node, convexidade obrigatória, undo de graça. Fatia 2: mover/girar/escalar o
  envelope inteiro no Select via o gizmo de sprite — o envelope virou objeto de geometria LOCAL +
  pose no `Transform` (o modelo correto), e o `recook` (`&SimWorld`) não pode tocar a pose.
- **Sem foundational (só docstrings), sem contrato congelado, sem contagem de registro, sem schema.**
- **Gates verdes (workspace 7405), clippy limpo, mutações provadas.** Smoke Fatias 1+2 aprovado.
- **Commits (locais, sem push):** `207d10b9` (F1) · `43b918f5` (fix smoke F1) · `5bddd9e4` (F2) + docs.
- **A linha NÃO integra nem faz push** (§0.7): entrego este handoff e **PARO**.
