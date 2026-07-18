# HANDOFF de INTEGRAÇÃO — `line/Vector`: Envelope Fatia 1 (arrastar os cantos da gaiola, ADR-0129)

**Para:** o **agente integrador** (e o próximo implementador da linha).
**De:** a sessão de 2026-07-17 que assumiu a linha pelo `HANDOFF_line_vector_continuacao_2026-07-17.md`
(§4.A item 1 — o 1º gesto vivo do envelope).
**Estado:** **fechado, pendente de smoke do Enio.** Motor + host live já estavam na `main` (Fatias A+B);
esta fatia é **só a UI** (a alça própria de canto no modo Node). Integração de **baixo risco**: nenhum
foundational, nenhum contrato congelado, nenhuma contagem de registro mexida.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **Commits da fatia** | `207d10b9` (feature) · `43b918f5` (fix do smoke — ver abaixo) · docs neste arquivo |
| **Base do fork (merge-base com `main`)** | `cdc3acc1` |
| **`main` desde a base** | **0 commits** — a linha está em cima da `main` de hoje; **não precisa rebase** |
| **Contratos congelados encostados** | **NENHUM** (§4) |
| **Smoke** | **APROVADO pelo Enio (2026-07-17)** — `cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=11 cargo run -p ph2d-host-desktop --features panel-vector`: a gaiola aparece no NODE com 4 cantos arrastáveis; a forma re-deforma ao vivo; canto puxado p/ dentro para na fronteira convexa. |

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

**Provas de mutação rodadas nesta sessão** (mutei, vi RED sobre visto-verde, restaurei): cerca de raio,
guard de convexidade, filtro de dono do `view`. Os três sangram.

**Smoke:** **pendente do Enio.** `PH2D_BUILD_SMOKE=11`. ⚠️ Armadilha do ADR-0129 §Aceitação: **arrastar
o canto engana** — o ingênuo também acerta o canto. A prova da CORREÇÃO é a curva LISA entre os cantos
(o gate de invariância à subdivisão já a automatiza). O que a Fatia 1 adiciona ao smoke é: **os cantos
obedecem ao dedo, e a gaiola recusa ficar não-convexa** (puxe um canto p/ dentro — ele para na fronteira).

---

## §6 — A FILA (a ordem é do Enio; ADR-0129 §Plano é a fonte)

Fatia 1 fechada. Restam da 4.A (fechar o Envelope):

2. **Mover o objeto-envelope inteiro** (modo Select) — a fonte está congelada em MUNDO no componente;
   mover o conjunto aplica um afim aos `corners` + à fonte (ou re-baka).
3. **O container multi-filho** (1 gaiola p/ N formas; hoje é 1-para-1).
4. **Release / Expand** — materializar a deformada como forma comum e soltar a gaiola.
5. **O painel** (seção Envelope docada: Fidelity/`accuracy` + presets + escolha de gesto).
6. **Os outros gestos** (cada um é um `impl Warp` novo): C presets · D 4-curvas/Coons · E pinos/MLS
   (a mais delicada — exige o `break_cusp` que hoje volta `None` de propósito; ver ADR §3.2 e o
   handoff de continuação §3.2).

E a 4.B herdada (Live Path Effects como nós, morph vivo, blend em cadeia, etc.).

**Nota p/ quem pegar a Fatia 2:** o `vec_envelope_drag` (runtime, em `app_state`) e o `press`/`drag` do
`envelope_gesture` são o molde. Mover o objeto inteiro no Select é OUTRO gesto (o gizmo de sprite já
existe) — mas o envelope força a identidade no recook, então "mover" tem de reescrever `corners` + a
fonte, não o `Transform`. É a diferença que o §3.3 do ADR nomeia.

---

## §7 — Resumo de fechamento

- **Fatia 1 do Envelope (ADR-0129 §4.A.1) construída e gateada.** Alça própria de canto no Node,
  convexidade obrigatória, undo de graça.
- **Sem foundational, sem contrato congelado, sem contagem de registro.** Integração aditiva.
- **Gates verdes (workspace 7404), 3 mutações provadas.** Smoke pendente do Enio (`PH2D_BUILD_SMOKE=11`).
- **A linha NÃO integra nem faz push** (§0.7): entrego este handoff e **PARO**.
