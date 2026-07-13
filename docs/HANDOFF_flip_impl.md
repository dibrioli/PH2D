# HANDOFF — linha `line/FLIP`, Waves W0 (dados) + W1 (render GPU) + W2 (tool + painel + desenho + borracha, **COMPLETA**)

> 🟥 **PRÓXIMO AGENTE: comece por [`HANDOFF_flip_NEXT.md`](HANDOFF_flip_NEXT.md)** — o Modo L
> (o seu contrato), o estado da linha, as seis lições que custaram caro, e a **sua 1ª tarefa**: o
> problema aberto do balde (a referência do fill vs. a espessura da linha — causa já PROVADA, com
> os números). Este arquivo aqui é o tracker exaustivo do estado — leia-o DEPOIS.

> **Dois leitores:** (a) o **integrador** (§1.5.9) — o que fundir, símbolos que
> colidem, gates; (b) o **próximo implementador desta linha** — W2 fechou; o
> próximo tópico é **W3 (Frames · Ghost · Tween)**, guia em §W3-NEXT abaixo. A
> linha está **aberta, commitada local, NÃO integrada/pushada** — commits
> `--no-verify`, fast mode.
>
> **W2 fechou (2026-07-11, esta sessão):** o painel docado `ph2d-panel-flip`
> (Mode/Brush/Color/Layers), a borracha (Soft/Hard/Stroke), o seam painel↔tool +
> ops de camada no drain, a camada-ativa como alvo do traço, e o ready-to-smoke
> (ativar Flip num doc vazio cria um objeto). **Todas as decisões interinas de
> §W2.4 foram revertidas/resolvidas.** Gate W2 + auditoria abaixo.

## 1. Identidade

- **Branch:** `line/FLIP`
- **HEAD:** `d1d1ec5c`
- **Base (merge-base com `main`):** `1c7c9a22`
- **Commits na linha:** 27 (todos `--no-verify`, fast mode)

```
# — W2 (COMPLETA — painel + borracha + seam de camada) —
d1d1ec5c W2 T2.9/T2.12/T2.16/T2.17 — shell wiring (bridge, layers, eraser, ready-to-smoke)
b2effe49 W2 T2.10-T2.15 — docked panel ph2d-panel-flip (Mode/Brush/Color/Layers)
33985e58 W2 editor-core — FLIP_* ids + per-layer id family + scrollbar id + z-order
6e14b8fd W2 tool — panel setters + handle_panel_event + Select default + layer reorder
# — W2 (tool + desenho, 1º corte) —
ee5996c8 docs — handoff detalhado W2
0ce648aa W2 T2.8 — pen-up simplify RDP
b83d22ae W2 T2.7 — active smoothing (o "assentar")
11200f02 W2 fix — preview ao vivo do traço
80529e8e W2 T2.5-T2.6 — flip_bridge + desenho (pointer→mundo→traço)
792dea44 W2 T2.4 — pill Flip no topbar (aparece + alterna)
7e1a572c W2 T2.1-T2.3 — tool ph2d-tool-flip + IconId::Flip + tool-sync
# — W0+W1 (fechados) —
89ec5df0 docs — handoff W0+W1
# — W1 (render GPU) —
48b590d7 fix(flip): BTreeMap no cache de tesselação (ADR-0022 disallowed HashMap)
42c75414 style(flip): fmt canônico (pin 1.95) nos arquivos de T1.7
6efd8521 feat(flip): W1 T1.8 — troca de quadro barata (cache de tesselação por-desenho)
681105be feat(flip): W1 T1.7 — compositing por-camada (blend/opacity via LayerCompositor)
61278c21 feat(flip): W1 T1.6 — fill triangulado (ear-clipping) + pipeline de fill
e986adfd feat(flip): W1 T1.5 — ordem 2D por depth (GP §2, teste GREATER)
37d03d64 feat(flip): W1 T1.4 — passe do traço no present.rs + demo ready-to-smoke
1682f7da feat(flip): W1 T1.2/T1.3 — shader wgpu (vertex expand + fragment hardness)
be1212f0 feat(flip): W1 T1.1 — crate ph2d-flip-render + packing SoA->GPU (headless)
# — W0 (dados) —
33e9d855 docs(flip): handoff de integração W0 (DIRETRIZ §1.5.9)
66c76e64 fix(flip): W0 gate — count ecs 25 em render/script + PROJECT_SCHEMA v2
dbeb0991 style(flip): derive Default (clippy derivable_impls)
0d01ff1b feat(flip): W0 T0.9-T0.11 — ponte ECS + undo + save (shell)
1fb2e593 feat(flip): W0 T0.8 — componente ECS FlipObjectRef (ph2d-ecs)
a491f400 feat(flip): W0 T0.1-T0.7,T0.12 — modelo ph2d-flip (clean-room GP 5.2)
```

> **W1 é MUITO isolado:** uma crate nova (`ph2d-flip-render`) + wiring no shell.
> **NÃO toca nenhum foundational** (ph2d-render/ph2d-ecs/contratos intactos no W1),
> **não bumpa registry/count nenhum**, não adiciona `IconId`/token/`NodeOp`. O único
> risco de colisão do W1 são os campos novos em `AppGfx` (§3). O grosso do risco de
> integração está no W0 (o componente ECS + os 3 counts).

## 2. Foundational / compartilhado tocado (+ por quê)

Tudo **aditivo** (nada reescrito). Foundational-não-contrato = editável pela linha
sob o gate testado (ADR-0107).

| Arquivo | Mudança | Por quê |
|---|---|---|
| `crates/ph2d-flip/**` (crate NOVA, 10 arquivos) | modelo de documento puro | drop-crate isolada; entra no workspace pelo glob `crates/*` (sem editar `members`) |
| `crates/ph2d-ecs/src/flip_object_ref.rs` (NOVO) | componente `FlipObjectRef(u64)` | ponte objeto↔entidade (espelha `vec_path_ref.rs`) |
| `crates/ph2d-ecs/src/lib.rs` | `pub mod flip_object_ref;` + `pub use FlipObjectRef` | export do componente |
| `crates/ph2d-ecs/src/scene/registry.rs` | registro + `reg.len()` **24→25** | save/undo precisa do componente registrado |
| `crates/ph2d-render/src/registry.rs` · `crates/ph2d-script/src/registry.rs` | count ecs+próprio **25→26** | somam `register_ecs_components`; SÓ a batched gate pega (não o `check -p`) |
| `shells/desktop/src/flip_entities.rs` (NOVO) | ponte `sync`/`rebuild_map` | espelha `vec_entities.rs` |
| `shells/desktop/src/render_loop/mod.rs` | destructure `flip` + 1 chamada `flip_entities::sync` (ao lado do vetor, todo frame) | reconciliação doc↔entidades; **no-op no W0** (sem tool que crie objetos) |
| `shells/desktop/src/app_state.rs` | `AppGfx.flip: FlipDoc` + `App.flip_entities: FlipEntityMap` | estado vivo |
| `shells/desktop/src/init.rs` · `main.rs` | init dos 2 campos + `mod flip_entities;` | boot |
| `shells/desktop/src/undo.rs` | `ProjectState` ganha 3º campo `flip: FlipDoc`; capture/restore/apply cobrem o Flip | undo global |
| `shells/desktop/src/project.rs` | `PROJECT_SCHEMA` **1→2** (o `flip` mudou o formato do save) | HR-14 |
| `shells/desktop/Cargo.toml` · `Cargo.lock` | dep `ph2d-flip` | — |

**Nenhum ponto de extensão central foi editado de forma não-append-only.** O
componente é um arquivo irmão novo; o registro é append no fim de
`register_ecs_components`.

### W1 (render GPU) — tudo aditivo, isolado

| Arquivo | Mudança | Por quê |
|---|---|---|
| `crates/ph2d-flip-render/**` (crate NOVA) | pipeline wgpu do traço + fill + composição por-camada | drop-crate isolada (glob `crates/*`); NÃO vai pela `vello::Scene` — passe wgpu dedicado (ADR-0114) |
| `crates/ph2d-flip-render/Cargo.toml` | **dev-deps** `ph2d-render`/`ph2d-gpu`/`ph2d-painter-effects` | SÓ o teste e2e de T1.7 (compositor real). **Não** é dep de runtime — o reuso do compositor é orquestrado no shell |
| `shells/desktop/src/render_loop/flip_pass.rs` (NOVO) | orquestração stage→inject→composite→blit + `TessCache` | o passe que compõe o Flip no `game_rt`; reusa o `LayerCompositor` 22-modos |
| `shells/desktop/src/flip_demo.rs` (NOVO) | cena demo ready-to-smoke (gated `PH2D_FLIP_DEMO=1`) | ver o render + o blend Multiply na hora |
| `shells/desktop/src/render_loop/present.rs` | destructure + 1 chamada `flip_pass::render` (pass 1b, entre sprites e tonemap) | põe o traço no `game_rt` HDR, tonemapeado junto |
| `shells/desktop/src/render_loop/mod.rs` | destructure ganha `flip_compose: _`/`flip_composite: _`; `mod flip_pass` vira `pub(crate)` | campos novos de `AppGfx`; o tipo `FlipComposite` é referenciado em `app_state` |
| `shells/desktop/src/app_state.rs` · `init.rs` · `main.rs` | `AppGfx.flip_compose` (eager) + `AppGfx.flip_composite: Option` (lazy) + `mod flip_demo;` | estado vivo do render |
| `shells/desktop/Cargo.toml` · `Cargo.lock` | dep `ph2d-flip-render` | — |

**Espaço de cor (decisão de arquitetura, ratificada pelo Enio):** o compositor do
Painter é 8-bit straight-sRGB; o Flip rasteriza premult/linear/16F. Cada camada é
resolvida (un-premult + sRGB-encode) → injetada GPU→GPU (`inject_slice_from_texture`,
sem readback) → composta → blitada de volta pro `game_rt` 16F. O round-trip 8-bit é
imperceptível (linha SDR ∈ [0,1]); o ganho é **blend bit-idêntico ao Painter**.

## 3. Símbolos novos que podem COLIDIR com outra linha (grep-áveis)

- **Componente ECS** (nome canônico, string estável): `"ph2d::ecs::FlipObjectRef"`.
  → `ComponentRegistry` len: **25** (era 24). Downstream: render/script tests = **26**.
- **Crate nova:** `ph2d-flip` (nome de pacote).
- **Const nova:** `ph2d_flip::FLIP_SCHEMA_VERSION = 1`.
- **Const alterada:** `PROJECT_SCHEMA = 2` (era 1) em `shells/desktop/src/project.rs`.
- **Campos novos:** `AppGfx.flip`, `App.flip_entities`, `ProjectState.flip` (3º campo).
- **Módulo shell novo:** `mod flip_entities;` em `main.rs`.
- **Sem** `IconId`/`NodeId`/`ColorToken`/token novo (W2 traz a tool + o painel).
- **W1:** campos novos `AppGfx.flip_compose` + `AppGfx.flip_composite` (nos 3 sites
  de destructure: `present.rs`, `render_loop/mod.rs`, e o literal em `init.rs`).
  Crate nova `ph2d-flip-render`. `mod flip_pass` = `pub(crate)`. Tipo novo
  `render_loop::flip_pass::FlipComposite`. **Nenhum count/registry/contrato** no W1.
- **W2 Select/gizmo (2ª rodada):** módulos shell novos `mod flip_gizmo_view;` +
  `mod flip_transform;` (`main.rs`) e `mod flip_pass_cache;` (`render_loop/mod.rs`).
  Símbolos novos em `ph2d-flip`: `FlipObject::geometry_bbox`/`bake_affine`; em
  `ph2d-flip-render`: `FlipGpuData::append`. **Edições ADITIVAS em arquivos SHARED**
  (colisão textual possível se outra linha tocar os MESMOS sites — Mergiraf funde,
  mas confira): `input_dispatch.rs` (~6 sites de pick/marquee/gizmo_anchor_half, TODOS
  ao lado do bloco vetorial existente — mesmo padrão), `render_loop/snapshots.rs`
  (branch Flip no `build_view` + 2 params novos em `publish`), `render_loop/mod.rs`
  (`settle_origins` do Flip + `flip_gizmo_on` no `publish`), `present.rs`
  (`flip_transform::build` + param `models` no `flip_pass::render`), `flip_draw.rs`/
  `flip_erase.rs` (fronteira world→local). **Nenhum count/registry/contrato novo.**

> Colisão mais provável com outra linha: se **outra linha também bumpou
> `register_ecs_components`** (novo componente), o `reg.len()` esperado soma —
> reconcilie o número (não é 25, é 24 + Σ dos componentes novos das linhas) nos 3
> sites: `ph2d-ecs`, `ph2d-render`, `ph2d-script`. Mergiraf funde as duas linhas
> de `register(...)`; o número do `assert_eq!` é o resíduo semântico a acertar.

## 4. Contratos congelados encostados (§4)

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/
`CanvasPaintTool`/`PanelEvent`, `Vector`(`ph2d-vector-doc/-traits`) — **intactos**.
O `ComponentRegistry` do ECS **não é** contrato congelado (é ponto de extensão
append-only). Não exige ADR.

## 5. O que SÓ o `ship.sh` pega (a gate de integração não roda)

- **fmt pré-fork:** rodei `rustup run 1.95 cargo fmt` nas crates tocadas → limpo no pin.
- **machete (deps não-usadas):** `ph2d-flip` usa todas as 4 deps (ph2d-core,
  ph2d-painter-effects, serde, postcard); a dep `ph2d-flip` no shell é usada. Sem
  dep órfã — mas o ship confirma.
- **deny/audit (RUSTSEC):** zero crate externa nova (só path-deps + serde/postcard
  já no workspace). Sem superfície nova de advisory.
- **typos:** comentários em pt-BR (mesma convenção de `vec_entities.rs`, que passa
  CI). Baixo risco; ship confirma.
- **W1 machete:** as **dev-deps** de `ph2d-flip-render` (`ph2d-render`/`ph2d-gpu`/
  `ph2d-painter-effects`) são TODAS usadas no `tests/composite_blend.rs`; a dep de
  runtime `ph2d-flip-render` no shell é usada em `flip_pass.rs`/`app_state.rs`. Sem
  dep órfã. Zero crate EXTERNA nova no W1 (só path-deps já no workspace) → sem
  superfície nova de deny/audit.

## 6. Ordem / dependências + o que smoke-testar

- **Ordem dos commits:** linear, sem interdependência frágil. `a491f400` (crate) →
  `1fb2e593` (ecs) → `0d01ff1b` (shell) → fixes. Integrar como um bloco.
- **Smoke W1 (VISUAL, ready-to-smoke):**
  `cd Worktrees/line-FLIP && PH2D_FLIP_DEMO=1 cargo run -p ph2d-host-desktop --release`
  → deve aparecer uma moldura cinza + um retângulo **amarelo** preenchido (BG) e um
  quadrado **magenta** (FG, blend **Multiply**). Onde o magenta cruza o amarelo, a
  interseção **escurece** (Multiply) — a prova visual de T1.7. Dê **play** (o
  transporte) → o quadrado FG salta entre 3 quadros-chave (0/8/16 @12fps) — a prova do
  render por-quadro. `PH2D_FLIP_STATS=1` junto loga `packs`/`hits` do cache (T1.8):
  num *hold*/parado deve logar `0 pack(s)` (zero re-tesselação).
- **Sem a env:** o app sobe normal (Flip = no-op; nenhuma tool cria objetos ainda).
- **NÃO smokado (por não existir ainda):** desenhar/criar objeto Flip pela UI. É W2.

## Gate W0 — resultado (rodado nesta linha, 1× sobre o diff acumulado)

- `cargo test -p ph2d-flip` → **28 verdes** (tabela GP `{0:d0,5:d1,10:end,12:d2}`,
  refcount+remap, ops de frame, round-trip serde, amostragem por playhead).
- `cargo test -p ph2d-ecs` (registry + full) → verde; render/script registry → verde.
- Shell: `flip_entities` (3) + `undo` (7, inclui o flip novo) + `project` (1) → verde.
- `bash scripts/nextest-impacted.sh` → **957 passed, 0 failed** (40 GPU-skipped).
- `cargo clippy --all-targets -D warnings` em ph2d-flip / ph2d-ecs / ph2d-host-desktop → limpo.
- `cargo fmt --check` (pin 1.95) → limpo. LOC: maior arquivo 455 (cap 700). HR-5:
  zero transcendental. Sem hex / f32-UI / tofu em string literal.

## Auditoria (DIRETIVA §3 — 2 lentes, ASSERÇÃO-VERMELHA real, não "compila OK")

**LENTE: correção (port clean-room das ops de frame).**
CLAIM: `drawing_at`/`add_frame`/`remove_frame` reproduzem a semântica de hold +
end-sentinel do GP 5.2.
TRAÇO: `layer.rs:92 drawing_at` = `range(..=frame).next_back().and_then(|f| f.drawing)`
← lido contra `grease_pencil.cc:1617` (`upper_bound`+recua); `add_frame` (`layer.rs:151`)
← `grease_pencil.cc:1535` (overwrite-end, remove-leading-ends, sentinela em key+dur);
`remove_frame` (`layer.rs:194`) ← `grease_pencil.cc:1565` (replace-with-end quando o
anterior é fixo).
ASSERÇÃO-VERMELHA: `layer::tests::drawing_at_follows_hold_and_end_sentinel` dirige a
tabela canônica do GP e quebraria se o hold/sentinela regredisse;
`remove_frame_with_fixed_prev_becomes_end` prova o branch replace-with-end.
NÃO-CHECADO-PELA-COMPILAÇÃO: a IGUALDADE numérica com o GP (a compilação não sabe
que d1 aparece 5..9) — coberta pelos testes tabelados.
LOC LIDAS: `grease_pencil.cc` 1505-1610 + 3207-3530 (fonte) + os 413 de `layer.rs`.

**LENTE: wiring (undo/save — o risco "compila mas está morto").**
CLAIM: capturar→restaurar de fato round-trips o `FlipDoc` E reconstrói a ponte
objeto↔entidade (senão o `sync` seguinte duplicaria objetos).
TRAÇO: `undo.rs:capture` grava `flip.clone()` → `restore` despawna Transform,
respawna do snapshot (que carrega `FlipObjectRef` **porque o registrei**), chama
`flip_entities::rebuild_map` → `apply_project` atribui `gfx.flip` + `self.flip_entities`.
ASSERÇÃO-VERMELHA: `undo::tests::flip_survives_capture_restore_and_rebuilds_bridge` —
muda o flip, restaura, e afirma (a) o doc voltou ao capturado, (b) `fmap` tem o
objeto apontando uma entidade VIVA, (c) capturar 2× é idêntico (sem passo espúrio).
Sem o registro do componente, (b) falharia (o snapshot descartaria `FlipObjectRef`).
NÃO-CHECADO-PELA-COMPILAÇÃO: que o componente está REGISTRADO (compila sem ele; só o
teste de restore-com-entidade-viva pega) e que o diff não regista passo fantasma.
LOC LIDAS: `undo.rs` inteiro (475) + `vec_entities.rs` (519, o espelho) + `registry.rs`.

## Gate W1 — resultado (rodado nesta linha, 1× sobre o diff acumulado)

- `cargo test -p ph2d-flip-render --include-ignored` → **16 verdes**: 10 unit (pack/
  fill) + **2 e2e GPU de composição** (`composite_blend`: Multiply 0.6×0.5=0.30 +
  opacity 0.5, no alvo HDR real) + 4 GPU de traço (`gpu_render`). Também verde em
  **`--release`** (o pipeline otimizado bate os mesmos valores).
- `cargo test -p ph2d-host-desktop --bins flip_pass::tests` → **3 verdes** (o
  `TessCache`: *hold* = 0 re-tesselações; mudança de conteúdo re-tessela; hash
  estável+sensível).
- `cargo test -p ph2d-flip` → **28 verdes** (inalterado do W0).
- `cargo clippy --all-targets` em `ph2d-flip-render` + `ph2d-host-desktop` → limpo
  (peguei o `disallowed_types` do `HashMap` → `BTreeMap`, ADR-0022).
- `cargo fmt --check` (pin 1.95) → limpo. HR-5: zero transcendental na Rust (o `pow`
  do sRGB é só no WGSL, como no `layer_composite.wgsl`). Sem hex/f32-UI/tofu.
- **Build `--release` do shell** → OK (1m15s). Smoke visual = §6 (pro Enio).

## Auditoria W1 (DIRETIVA §3 — 2 lentes, ASSERÇÃO-VERMELHA real)

**LENTE: correção do seam de composição (o risco "compila mas compõe errado").**
CLAIM: o caminho stage→inject→composite→blit produz o MESMO blend do Painter, no
espaço de cor certo (premult/linear/16F ↔ straight/sRGB/8-bit).
TRAÇO: `composite.rs:stage_layer` renderiza premult-over no 16F → `fs_resolve`
(un-premult + `linear_to_srgb`, literais idênticos ao `layer_composite.wgsl`) →
`inject_slice_from_texture` (GPU→GPU) → `LayerCompositor::composite` (já testado em
ph2d-render) → `fs_blit` (`srgb_to_linear` + premult) no `game_rt`.
ASSERÇÃO-VERMELHA: `composite_blend::two_layers_multiply_composites_like_painter`
mede na sobreposição **0.30** (= 0.6×0.5) no alvo HDR e afirma que é ESTRITAMENTE
mais escuro que qualquer camada só — quebraria se o resolve/blit invertesse premult,
errasse o sRGB, ou o inject não landasse a fatia. `top_layer_opacity_fades…` prova o
opacity (branco@0.5 sobre preto = 0.5 linear).
NÃO-CHECADO-PELA-COMPILAÇÃO: a corretude NUMÉRICA do blend e a ordem de submissão
(inject antes do próximo stage sobrescrever o scratch) — cobertas pelo e2e de 2
camadas (passa → a ordem está certa).
LOC LIDAS: `layer_compositor/{api,buffers,mod}.rs` (a superfície do compositor +
`inject`) + `composite.rs`/`flip_pass.rs` inteiros.

**LENTE: o cache não pode servir geometria obsoleta (T1.8).**
CLAIM: o `TessCache` reusa a tesselação num *hold*/pan/zoom, mas re-tessela quando o
conteúdo muda (edição W2 ou reuso posicional de `DrawingId` pós-compactação).
TRAÇO: `flip_pass::TessCache::ensure` chaveia por `(object_id, drawing_id)` E valida
por `drawing_hash` (FNV-1a de posições/larguras/cores/fill/…); hash diverge →
re-pack. A geometria é camera-INDEPENDENTE (mundo), logo cachear entre frames de
câmera diferente é correto (a câmera entra só no `CameraRaw` do shader).
ASSERÇÃO-VERMELHA: `content_change_forces_repack` muda 1 ponto sob a MESMA
`cache_key` e afirma `packs+1`; `hold_reuses_tessellation_zero_repacks` afirma
`packs==0` em 12 quadros seguidos do mesmo desenho.
NÃO-CHECADO-PELA-COMPILAÇÃO: que hash-colisão não sirva geometria errada (mitigado:
chave `(obj,did)` limita colisões a UM desenho; hash só decide re-pack).
LOC LIDAS: `flip_pass.rs` inteiro + `drawing.rs`/`stroke.rs` (os campos que o hash lê).

---

# W2 — Tool de desenho + Painel + Borracha (**COMPLETA**)

**Status:** **fechada.** Toda a Wave 2 landou: a mão de desenho (T2.1–T2.8), o
painel docado (T2.10–T2.16), a borracha (T2.9) e o ready-to-smoke (T2.17). Plano:
`docs/Flip/01_plano_waves.md` §W2 (untracked na árvore primária). Referência
clean-room: `docs/Flip/02_referencia_algoritmos_blender_5.2.md` §5.

## W2.1 — O que está PRONTO (T2.1–T2.17, tudo)

| Task | Entregue |
|---|---|
| T2.1–T2.3 | Crate `ph2d-tool-flip` (drop-crate ADR-0040) + `IconId::Flip` + tool-sync. |
| T2.4 | Pill **FLIP** no topbar (aparece + alterna a tool). |
| T2.5–T2.6 | `flip_bridge` + desenho: pointer→mundo→`FlipStroke` no `FlipDoc`. |
| T2.7 | **Active smoothing** (blur binomial, cauda assenta — `flip_smooth.rs`). |
| T2.8 | Pen-up **simplify RDP** (traço enxuto no commit). |
| **T2.9** | **Borracha** `flip_erase.rs`: **Soft** (reduz opacidade + cleanup no up), **Hard** (corta e divide o traço), **Stroke** (apaga o traço tocado). Camada travada recusa. |
| **T2.10–T2.11** | Crate **`ph2d-panel-flip`** (painel docado) + node-ids em `ids/chrome/flip.rs` + 6 sites de registro + panel-sync + `FLIP_SCROLLBAR_ID`. |
| **T2.12** | `flip_bridge` estendido: visibilidade (dock takeover) + toggle do inspector + read-back do picker OKLCH + publica `set_current_flip_style`/`_layers`. |
| **T2.13–T2.14** | Seções **Mode** (Select/Draw/Erase) + **Brush** (Size/Hardness/Opacity/Smoothing) + **Color** (swatch → picker OKLCH compartilhado). |
| **T2.15** | Seção **Layers** (idioma Painter): add/delete + por-camada visibility/lock/reorder ↑↓/opacity/**blend dropdown** (22 modos, popover compartilhado); linha ativa destacada; ids de linha runtime (`flip_layer_widget_id`). |
| **T2.16** | Seam painel↔tool: `FlipTool::handle_panel_event` (modo+brush) + **ops de camada no drain do shell** (`flip_layers::apply_panel_event` → `FlipDoc`). |
| **T2.17** | Ready-to-smoke: ativar a tool Flip num doc **vazio** cria um objeto ("Flip", 1 camada) — desenha na hora, sem env. |
| §W2.4 | **Decisões interinas revertidas:** default = **Select** (ADR-0112); o traço vai pra a **camada ATIVA** (fallback topo), camada travada recusa. |

## W2.2 — Arquitetura do desenho (o fluxo, ponta-a-ponta)

```
pill FLIP (chrome/flip_toggle.rs) → EditorAction::ActivateTool{"flip"}
  → render_loop::mod drain ativa a tool (gate 'flip_tools', 3 sites)
render_loop::flip_bridge::publish(tools)  [1×/frame, downcast ALLOWLISTADO]
  → App.flip_active + App.flip_style  (cache; input_dispatch NÃO faz downcast)
on_mouse_input / on_cursor_moved  (input_dispatch.rs)
  → App::flip_canvas_down/move/up  (flip_draw.rs) → screen_to_world → FlipDraw
present::run_present_phase → App::flip_preview_data() [antes do borrow de gfx]
  → flip_pass::render(preview) → draw_overlay (rasteriza o traço em curso)
pen-up → flip_canvas_up → bake_stroke: active_smooth → simplify_rdp → push no
  desenho ativo (1º objeto, camada TOPO, quadro atual; cria chave se preciso)
```

**Padrão-chave (herdado do Vector):** *documento ≠ tool*. A `FlipTool` só guarda
estilo+modo; o documento (`FlipDoc`) e a interação (traço em curso) vivem no shell.
O **downcast concreto** só no `flip_bridge` (allowlistado em
`tests/architecture_no_downcast_to_concrete_tool_in_shell.rs`); o `input_dispatch`
lê o **cache** publicado — nunca faz downcast.

## W2.3 — Arquivos do W2 (onde mexer)

**Crate da tool** (`crates/ph2d-tool-flip/`): `lib.rs` (make/register/MANIFEST,
cluster `flip_tools`), `tool.rs` (`FlipTool` = estilo+modo, `handle_panel_event`
é **stub VAZIO** esperando os node-ids do painel), `params.rs` (`FlipMode`
Select/Draw/Erase, `EraseMode`, `FlipStyleSnapshot`, mapas de slider), `icon.rs`.

**Fiação do pill (chrome hand-maintained — NÃO é derivado do manifest!):**
`ids/chrome/topbar.rs` (`TOPBAR_FLIP`), `chrome/flip_toggle.rs` (z=271, gerado no
dispatch por chrome-sync), `screens/hero/fixture.rs` (pill), `topbar/mod.rs`
(`populate`), `render_loop/mod.rs` (gate `flip_tools` em 3 sites).

**Desenho (shell):** `flip_draw.rs` (`FlipDraw` amostragem + `bake_stroke` +
`build_stroke` + métodos `flip_canvas_*` no App + `flip_preview_data`),
`flip_smooth.rs` (`active_smooth` binomial + `simplify_rdp`), `render_loop/
flip_bridge.rs` (publish), `flip_pass.rs` (`render` ganhou `preview` +
`draw_overlay`), `input_dispatch.rs` (braço Down/Up + guard de move),
`app_state.rs` (`flip_active`/`flip_style`/`flip_draw`), `present.rs` (preview).

## W2.4 — Decisões interinas (RESOLVIDAS nesta sessão)

Todas as 5 pendências do 1º corte foram fechadas:

1. **default `Draw`→`Select`** ✅ — `FlipTool::default().mode = Select` (ADR-0112,
   gizmo só no Select); `fresh_tool_defaults` volta pra Select. Os botões de modo
   do painel trocam via `handle_panel_event`.
2. **Alvo do traço** ✅ — `bake_stroke` usa a **camada ATIVA** (`App.flip_active_
   layer`, setada pela seleção de linha no painel), com fallback pra a de topo;
   **camada travada recusa** o traço. `flip_active_layer` = `None` no boot; o
   `flip_bridge` publica a de topo como ativa por default.
3. **Doc vazio** ✅ — T2.17: ativar a tool Flip num doc vazio cria um objeto
   ("Flip", 1 camada) no `render_loop/mod.rs` (borda de ativação). Sem env.
4. **Modo Draw só** ✅ — a linha de modos do painel dá Select/Draw/Erase; a
   borracha (T2.9) roda no modo Erase (`flip_wants_erase` no `input_dispatch`).
5. **Pressão = 1.0** — **mantido** (mouse). Pen real (Apple Pencil) + curva de
   pressão editável (falloff do Painter) segue como refino futuro, fora do W2.

## W2.5 — Gotchas aprendidos no W2 (painel + borracha)

- **Latentes que só a gate batched pega** (o inner loop só rodou `cargo check`):
  `arch_mode_has_reconcile` (o `set_mode` do 1º corte já era offense → benign-list),
  `no_tofu_glyphs` (setas `→` em `assert!` de `flip.rs`/`flip_pass.rs`/`undo.rs` →
  `->`), `architecture_panel_wiring_parity` (o swatch é picker → allowlist; o X
  precisa `button()` no populate), `architecture_interactive_crate_has_behavioral_
  test` (todo painel precisa de `tests/seam.rs`), `scrollable_panels_intercept_the_
  wheel` (todo painel com scrollbar-thumb precisa entrar no `cursor_over_hero_panel`).
  **Lição:** rode o bloco de gates 1× ANTES de declarar a wave fechada.
- **Blend por-camada = `InteractiveState::Dropdown` + popover compartilhado**
  (`paint_dropdown_chip` + `paint_dropdown_popover_scrolled` do editor-core). O
  chip abre/fecha pelo dispatch genérico; a OPÇÃO clicada volta como `Click(opt_id)`
  que o `event.rs` decodifica → `SelectOption`. Ids de linha são runtime
  (`flip_layer_widget_id(layer_u64, kind)`), decodificados por brute-force contra
  o snapshot de camadas publicado (painel) e o `FlipDoc` (drain do shell).
- **Ops de camada = edição de DOCUMENTO, não de tool** — caem no drain do shell
  (`flip_layers::apply_panel_event`), não no `FlipTool` (que ignora os ids de
  camada). Mesmo padrão do Vector Boolean/Arrange.
- **Borracha Soft** reduz opacidade por-ponto durante o gesto; o **pen-up** dropa
  pontos < 0.05 e divide o traço (`cleanup_soft`). **Hard** divide em runs fora do
  círculo; **Stroke** apaga o traço tocado inteiro. HR-5: falloff é rampa linear.

---

# W3-NEXT — o próximo tópico desta linha (Frames · Ghost · Tween)

W2 fechou. O próximo é **W3** (`docs/Flip/01_plano_waves.md` §W3): a **tira de
frames** própria (não a timeline global), transporte (play/loop/pingpong/FPS),
**Ghost Frames** (onion) e **Tween** (inbetween). Pontos de partida já prontos:

- **Onde a tira mora:** a parte inferior do painel `ph2d-panel-flip` (adicione uma
  seção Frames abaixo de Layers, ou um dock inferior próprio). O `FlipObject` já
  tem `frames`/`insert_frame`/`duplicate_frame`/`remove_frame`/`move_frame` +
  `onion: OnionSettings` (T3.3) e `fps`.
- **Transporte:** roda sobre `App.playhead` (`ph2d_core::Playhead`); `frame_at` já
  mapeia tempo→quadro por FPS. O render por-quadro (W1) troca de desenho por
  rebind (zero re-tesselação).
- **Tween:** reusar `ph2d-anim::{Interp, Easing}` (o plano manda) + o resample por
  arco de §4. Correspondência de traço por índice + padding ao MÁX.
- **Reference clean-room:** `02_referencia` §1 (onion) + §3 (tween) —
  `gpencil_engine_c.cc`/`cache_utils.cc` (onion), `interpolate_curves.cc` (tween).

## W2.7 — Símbolos novos do W2 (grep-áveis, p/ o integrador)

**1º corte (T2.1–T2.8):** Crate `ph2d-tool-flip`; `IconId::Flip` (enum + `ALL_ICONS`,
entre FitView/Folder); `ids::TOPBAR_FLIP`; `chrome::flip_toggle`; cluster
`"flip_tools"`; campos `App.flip_active`/`flip_style`/`flip_draw`; módulos shell
`flip_draw`/`flip_smooth`/`render_loop::flip_bridge`; entrada na `DOWNCAST_ALLOWLIST`.

**Painel + borracha (esta sessão):**
- **Crate nova `ph2d-panel-flip`** (nome de pacote; feature `panel-flip`, no
  `default` do shell + em `ph2d-panel-registry-init`). `FlipPanel` no `EXPECTED_
  TYPED` (hand-maintained) + push gerado pelo panel-sync.
- **editor-core `ids/chrome/flip.rs`** (módulo novo): consts `FLIP_PANEL`/`FLIP_
  CLOSE`/`FLIP_MODE_*`/`FLIP_SIZE(_NUM)`/`FLIP_HARDNESS(_NUM)`/`FLIP_OPACITY(_NUM)`/
  `FLIP_SMOOTHING(_NUM)`/`FLIP_STROKE_SWATCH`/`FLIP_ERASE_*`/`FLIP_LAYER_ADD`/
  `FLIP_LAYER_DELETE`; enum **`FlipLayerWidget`** + `flip_layer_widget_id(u64,kind)`
  + `flip_layer_blend_option_id(u64,mode)` (família runtime, espelha painter).
- **`FLIP_SCROLLBAR_ID = NodeId(835)`** (widget/scrollbar.rs) + branch em
  `scroll.rs::scrollbar_panel_for_id` + `forwarding.rs::cursor_over_hero_panel`.
- **Campos novos:** `App.flip_active_layer: Option<LayerId>`, `App.flip_erasing:
  bool`. Módulos shell `flip_erase`/`flip_layers`.
- **ph2d-flip:** `FlipObject::raise_layer`/`lower_layer` (append-only).
  ph2d-tool-flip: setters `set_width_px`/`set_hardness`/`set_opacity`/`set_smoothing`/
  `set_erase_mode` + consts `DEFAULT_HARDNESS`/`_OPACITY`/`_SMOOTHING` +
  `WIDTH_SLIDER_*`/`OPACITY_SLIDER_SCALE` + `FlipStyleSnapshot: Default`.
- **Gates hand-maintained tocados** (não são contrato congelado): `node_id_
  collisions` (chrome + dynamic tables), `architecture_panel_wiring_parity`
  (allow FLIP_STROKE_SWATCH), `arch_mode_has_reconcile` (benign `set_mode`/
  `set_erase_mode`), `widget/scrollbar.rs` test list.

**Nenhum contrato congelado tocado** (Tool=12/PanelEvent=4 intactos; `ComponentRegistry`
não mexido no W2). **Nenhum count de ECS registry bumpado no W2** (o de W0 continua).
Colisão mais provável: outra linha que também adicione um `*_SCROLLBAR_ID` — reconcilie
o número (usei 835) e o branch do `scroll.rs`.

## Gate W2 — resultado (rodado 1× sobre o diff acumulado)

- `cargo test` em `ph2d-flip` (29) + `ph2d-tool-flip` (10) + `ph2d-panel-flip`
  (seam 2) + `ph2d-host-desktop` (incl. `flip_erase` 4, `flip_layers`) → **verde**.
- **Arch-gates verdes:** `node_id_collisions` (6), `architecture_panel_wiring_parity`,
  `architecture_panel_loc_cap`, `no_magic_numeric`, `arch_mode_has_reconcile`,
  `architecture_interactive_crate_has_behavioral_test`, `no_tofu_glyphs`,
  `scrollable_panels_intercept_the_wheel`, `architecture_no_downcast_to_concrete_
  tool_in_shell`, registry-init (panel + tool + ecs counts), full editor-core +
  shell suites (46 + 26 binários, 0 falhas).
- `cargo clippy --all-targets` em ph2d-flip/-tool-flip/-panel-flip/-editor-core/
  -host-desktop → **limpo**. `cargo fmt --check` (pin 1.95) → **limpo**. LOC: maior
  arquivo novo 451 (`paint_layers.rs`, cap 700). HR-5: borracha usa rampa linear.
- **Build `--release` do shell** → OK (linka). Smoke visual = seção abaixo (Enio).

## Auditoria W2 (DIRETIVA §3 — 2 lentes, ASSERÇÃO-VERMELHA real)

**LENTE: o seam painel→tool→documento (o risco "compila mas está morto", a classe
das vector-pills).**
CLAIM: um slider/botão do painel de fato muda o `FlipTool` (brush/modo) E uma op de
camada muda o `FlipDoc` — populate→apply_event→bus→handle_panel_event / drain.
TRAÇO: `event.rs` classifica `ValueChanged(FLIP_SIZE)`→`SetValue`→`FlipTool::set_
width_px`; `Click(FLIP_MODE_DRAW)`→`mode=Draw`; ids de camada→`flip_layers::apply_
panel_event`→`FlipDoc`. O gate `panel_wiring_parity` prova que TODO id hit-indexado
está registrado (senão `is_focusable()==false`, clique morto).
ASSERÇÃO-VERMELHA: `tests/seam.rs::size_slider_drag_reaches_tool` (largura chega a
`WIDTH_MAX_PX`) + `draw_mode_button_switches_the_tool_mode` (Select→Draw), rodando
o caminho real headless via `MockPanelHost`. Quebraria se qualquer braço do
`event.rs` ou do `handle_panel_event` sumisse.
NÃO-CHECADO-PELA-COMPILAÇÃO: que o id pintado == o id registrado == o id no drain
(compila com ids divergentes; só o seam + o wiring-parity gate pegam).
LOC LIDAS: `event.rs`/`paint_sections.rs`/`paint_layers.rs` + `flip_layers.rs` +
`arch_mode_has_reconcile.rs` + `architecture_panel_wiring_parity.rs`.

**LENTE: a borracha muda o documento certo (o risco "apaga a camada errada / ignora
o lock").**
CLAIM: a borracha age no desenho da CAMADA ATIVA do quadro atual, recusa camada
travada, e Soft/Hard/Stroke têm a semântica do GP.
TRAÇO: `flip_erase::active_drawing_mut` resolve (ativa→topo), retorna `None` se
`locked` OU se o quadro não tem chave (nunca cria desenho); `erase_at` ramifica por
modo; `flip_erase_canvas_down/move/up` no `input_dispatch` (gated `flip_wants_erase`).
ASSERÇÃO-VERMELHA: `flip_erase::tests` — `stroke_mode_removes_the_whole_touched_
stroke`, `hard_mode_splits_the_stroke_at_the_gap` (5 pts, apaga o meio → 2 traços),
`soft_mode_reduces_opacity_then_cleanup_removes_faded`, `locked_layer_refuses_erase`.
NÃO-CHECADO-PELA-COMPILAÇÃO: a IGUALDADE geométrica (o split em 2 runs; o lock
recusar) — coberta pelos testes tabelados.
LOC LIDAS: `flip_erase.rs` inteiro + `erase.cc` §5 (fonte) + `stroke.rs` (a API SoA
que o split/soft leem).

---

## Smoke do Enio (2026-07-11) — 3 corrigidos, 1 gap aberto (Select)

O Enio smokou o W2 e apontou 4 itens. **3 corrigidos** (commit `ce88bbc7`/`f84dcdb2`):
1. **Desenho suave ≠ traço assado** ✅ — o bake decimava com RDP 0.75px (mais
   anguloso que o preview). Agora assa o MESMO `active_smooth` do preview (RDP
   0.05px, só colinear puro). `flip_draw::bake_stroke`.
2. **Blend "não funciona"** ✅ — a lógica decode→apply está PROVADA (testes
   `flip_layers`); o sintoma era a camada de **fundo** (compõe contra nada = blend
   no-op). Agora o chip de blend some no fundo (igual ao Painter). `paint_layers`.
3. **Borracha Soft com borda dura no bake** ✅ — o `cleanup_soft` dividia o traço
   nos pontos apagados (cap plano = borda dura). Agora preserva os pontos de
   opacidade reduzida (gradiente macio); só descarta traços 100% apagados.

**4. "Select não funciona" — RESOLVIDO (2ª rodada, commits `feat(flip): Select/gizmo`
+ `refactor split`).** Paridade ADR-0111 completa: o objeto Flip agora é
selecionável (Hierarquia OU clique no canvas) e movido/girado/escalado pelo **gizmo
de sprite**, como uma forma vetorial. Peças:
- `flip_transform` (espelho de `vec_transform`): geometria LOCAL + `Transform`;
  `settle_origins` põe o pivô no centro da arte no fim do gesto (pula o objeto EM
  GESTO); `move_origin_to` bake sobre TODOS os desenhos (`FlipObject::bake_affine`).
- `flip_gizmo_view` (espelho de `vec_gizmo_view`): `anchor_half`/`view`/
  `contains_world`/`pick_all_at_world`/`pick_in_world_rect` da bbox local + pose.
- **render aplica o model por-objeto** (`flip_pass::fold_model`): `world_to_clip ·
  model` + `px_per_world · mean_scale` (o traço engrossa junto na escala). Sem mexer
  shader. Identidade = sem custo (caminho comum, byte-idêntico ao antes).
- **draw/erase localizam** na fronteira MUNDO→LOCAL (`flip_active_world_to_local`);
  identidade num objeto novo (desenho normal intacto).
- **wiring**: `settle` no reconcile; `GizmoView` publicada em `snapshots` (fora dos
  modos Draw/Erase); picking ADITIVO ao vetor nos ~6 sites de `input_dispatch`
  (cíclico, marquee, pivô, over-art) + `gizmo_anchor_half` ganhou branch Flip (drag
  de scale/rotate correto).
- 20 testes novos (bbox/bake, settle, gizmo pick/view, `fold_model`). O **seam de
  canvas não é unit-testável** — precisa do smoke do Enio (mas a Hierarquia +
  gizmo-drag já são exercitáveis e a math toda está coberta).

**Blend em TEMPO REAL (2ª rodada, commit `fix(flip): blend do preview em tempo real`):**
o preview ao vivo era `draw_overlay` (Normal) SEMPRE por cima → o blend só "aparecia"
no pen-up. Agora o traço em curso é DOBRADO na fatia da camada ativa
(`FlipGpuData::append` + `collect_layers` atribui o preview à camada-alvo), então
compõe pelo blend/opacity dela a cada frame (byte-idêntico ao bake). Camada-alvo
oculta/irresolvível cai no overlay Normal (fallback: nunca desenhar às cegas).

## Smoke do Enio (2026-07-11, 3ª rodada) — auto-overlap + hardness (GPU-verificados)

1. **"O mesmo traço passando por cima de si mesmo é pintado por baixo"** ✅ — a
   profundidade é **por-stroke** (todos os pontos no mesmo depth), com teste `GREATER`;
   no auto-overlap o 2º fragmento (depth igual) falhava → a parte mais VELHA por cima.
   Trocado p/ **`GreaterEqual`** (`pipeline::depth_greater_equal`): entre strokes/fills
   o sid maior segue ganhando (depth estritamente maior), mas no MESMO depth a parte
   desenhada DEPOIS compõe por cima — como o GP / uma caneta real.
2. **"Alpha estranho com hardness < 1"** ✅ — o fragment usava `pow(1-dn, 10·(1-hard))`
   (decai cedo demais → traço translúcido). Trocado pelo **perfil redondo do GP**:
   `mask = 1 - smoothstep(hardness, 1, dn)` (núcleo cheio até `hardness`, queda suave
   até a borda) com o AA de ~1px dobrado na MESMA `smoothstep`. `flip.wgsl:fs_main`.
   **Verificados em GPU REAL** (o adapter roda neste Linux, não só Metal): novo teste
   `a_stroke_crossing_itself_draws_the_later_part_on_top` + `hardness_controls_edge_falloff`
   + os 4 render/composite existentes passam.

**"Select do traço não" — feature FUTURA, não bug.** O que landou é seleção do OBJETO
inteiro (gizmo de sprite, igual sprite/vetor). Selecionar/editar um TRAÇO individual é
o **Edit Mode do Grease Pencil** — um modo à parte com hit-test por-stroke, estado de
seleção de traço/ponto, realce e transform do subconjunto. É um pacote próprio (não um
fix); candidato natural a W3/edit-mode. NÃO foi feito nesta rodada.

## Smoke do Enio (2026-07-11, 4ª rodada) — cobertura analítica GP + brush absoluto

1. **Artefatos de sobreposição nas junções/curvas com hardness baixo** ✅ — a
   cobertura vinha de uma coordenada `v_perp` **por-quad**, que DISTORCE nas junções
   (o miter deforma a perpendicular) → spikes + double-blend. Reescrito pra
   **cobertura ANALÍTICA como o GP** (`gpencil_stroke_segment_mask` em
   `draw_grease_pencil_lib.glsl`): o quad só COBRE a fita; a forma exata sai no
   fragment, da **distância do pixel à linha-de-centro**, clampada ao segmento →
   junções/tampas REDONDAS de graça, sem miter/spike, e **sem double-blend** (quads
   adjacentes calculam a mesma distância → cobertura consistente, o depth só escolhe
   um). `flip.wgsl` reescrito (vertex = quad-stadium; fragment = distância + perfil GP
   `pow`+smoothstep + AA `fwidth`). Camera uniform agora visível ao FRAGMENT
   (`viewport.y` p/ o flip-Y). **8 testes GPU verdes** (novo
   `a_sharp_corner_is_a_round_join_without_an_outward_spike`).
2. **Tamanho de brush relativo ao zoom → ABSOLUTO** ✅ — a largura era MUNDO e o
   render multiplicava pelo zoom. Agora `camera_raw` passa escala de espessura `1.0`
   e `build_stroke` guarda `width_px` de tela → espessura constante na tela em
   qualquer zoom. Escalar o OBJETO pelo gizmo ainda engrossa (`fold_model` ×
   `mean_scale`); só o zoom da câmera não. `flip_pass::camera_raw` + `flip_draw::build_stroke`.

> **Técnica durável:** a cobertura de traço estilo GP é ANALÍTICA (distância à
> linha-de-centro no fragment), NÃO uma coordenada por-quad. É o que dá junção
> redonda sem miter, sem double-blend, e alpha correto com hardness baixo. Ref viva:
> `/home/enio/Downloads/blender-5.2-grease-pencil-ref` → `draw_grease_pencil_lib.glsl`.

## WT — O TRAÇO: a mordida está MORTA (2026-07-12) — pendente o smoke do Enio

A cobertura do traço é agora a **UNIÃO GLOBAL da polilinha**, num único passe. **Doc definitivo:
[`docs/Flip/03_traco_rasterizacao.md`](Flip/03_traco_rasterizacao.md)** (mecanismo, as 4 peças,
o oráculo, as mutações, os kill-criteria). Resumo do que landou:

**O fix tem 4 peças** (a spec previa 1 — o vermelho dos testes revelou as outras 3):
1. **Janela de sequência `p0`/`p3`** — o vertex exporta os vizinhos (já os buscava para o
   miter); o fragment inclui as 2 cápsulas no `min`. Fecha a classe *quina quebrada*.
2. **Vizinhos GEOMÉTRICOS** (`neighbors.rs`, NOVO) — a janela ±1 **não bastava**: todo traço
   que volta sobre si mesmo (zigzag, laço, letra) tinha a mordida de longo alcance — a borda
   macia de um segmento (alpha 1/255!) vencia o depth e apagava o NÚCLEO de outro. Broadphase
   por grid no `pack` (cacheado por desenho) + loop no fragment ⇒ **união global, 1 passe, zero
   render passes extras**. Critério conservador `dist(i,j) < 2·r_i + r_j` (assimétrico: o raio
   do dono do quad entra dobrado).
3. **`capsule_dn` única** — o defeito D1 da análise adversarial era real: com largura por-ponto
   (pressão), o raio interpolado no QUAD ≠ o da cápsula, e a mordida sobrevivia em 2ª ordem.
   O teste do taper o pegou.
4. **Par clamp+fade sub-pixel** (`MIN_WIDTH_PX = 1.3` + `thickness` cru) — o fade do GP sozinho
   não salva a linha fina (ela não cobre o centro de pixel nenhum e SOME). No caminho, achamos
   um bug de AA que estava lá desde o W1: a cobertura de borda subestimava traço fino em 10×
   (a forma correta é `clamp(0.5 + (1-dn)/fwidth(dn), 0, 1)`).

Mais: **`safe_dir`** no miter — um ponto DUPLICADO fazia `normalize(0)` = NaN e **rasgava o
traço** (bug latente desde o W1).

**O tripé segue intacto** (miter/`miter_break`, depth por-stroke + GREATER estrito, discard).
Descoberta: com a união global, o **discard deixou de ser load-bearing** para a correção (a
mutação não sangra mais) — ele fica por proteger a degradação do cap e por economia. Não afrouxe.

**Gate:** 15 testes GPU + 18 unit + 2 composite verdes em **debug E release**; **5 mutações
provadas** (vizinhos geométricos, janela ±1, GreaterEqual, fade, clamp — cada uma sangra);
fmt (pin 1.95), clippy `--all-targets`, LOC caps, suite do shell — limpos.
**Perf (release):** traço real de 4000 pontos = **1.7 ms** de pack; rabisco patológico = 14 ms
(limitado pelo `PAIR_BUDGET`); `pack_perf.rs` guarda a ordem.

**Símbolos novos** (p/ o integrador): crate `ph2d-flip-render` — módulo `neighbors` (privado),
`pack::GpuSegRef` (pub), campos `FlipGpuData::{seg_extra_range, seg_extras}`, bindings 4 e 5 no
BGL (e `points` passa a ser VERTEX|FRAGMENT), teste `tests/pack_perf.rs`. **Nada fora da crate.**

**Smoke pedido:**
```
cargo run -p ph2d-host-desktop --release
```
Zigzag afiado com hardness alto E baixo · curvas densas · traço cruzando a si mesmo · linha
fina com zoom out (não pode piscar).

## Rodada 7 (2026-07-11) — 2 artefatos MORTOS, 1 NOVO (quina "mordida") — **REPROVADA no smoke**

> 🟥 **Veredito do Enio: "não ficou bom".** O acúmulo/spike/bead/escama **acabaram**
> (isso é ganho real e permanente), mas apareceu um artefato NOVO: **as quinas saem
> MORDIDAS** — um bocado reto arrancado do lado interno de cada virada afiada (o
> zigzag do smoke). **Diagnóstico + o fix, abaixo (§"A mordida").** O Enio decidiu
> **integrar assim mesmo** (o bug é cosmético e confinado a `flip.wgsl`) e resolver
> depois.

O caminho recomendado no `HANDOFF_flip_NEXT.md` §3 (fita conectada + bevel/miter_break
+ GREATER estrito + fragment analítico) estava certo — e a parte que eu **descartei**
dele (o refino **p0/p3 no fragment**) é justamente o que falta. Portei DUAS das três
peças do tripé:

1. **`miter_break` no vertex** (`gpencil_vertex`, `draw_grease_pencil_lib.glsl:696-724`):
   virada > 120° (`-dot(dir_in, dir_out) > 0.5`) NÃO mitra — o offset fica na
   perpendicular do próprio segmento e o quad **estende `r` ao longo da linha**
   (o `screen_ofs += line * x` do GP). A fita nunca dobra (fim do bowtie/spike);
   o esticão do miter nas quinas ≤ 120° é ≤ 2 por construção (o clamp
   `MITER_LIMIT=4` antigo saiu). `flip.wgsl` vertex.
2. **`discard` de fragmento ~transparente** (`gpencil_frag.glsl:548`:
   `a < 0.001`): sem ele, fragmento com alpha≈0 ESCREVE depth e fura a geometria
   sobreposta que chega depois — **era o mecanismo exato do "escamado" do beco #3**
   (stadium + GREATER), que a matriz do handoff dava como beco sem explicação.
   `flip.wgsl` fragment + `flip_fill.wgsl` (paridade: o frag do GP é compartilhado).
3. **GREATER estrito** (era GreaterEqual): `pipeline.rs::depth_greater`
   (renomeada). Estado EXATO do GP 2D (`gpencil_cache_utils.cc:449`).

**Mudança de semântica DELIBERADA (flag pro smoke):** auto-cruzamento agora pinta
**uma vez** — a parte desenhada PRIMEIRO fica por cima ("the stroke cannot overlap
itself", `gpencil_vert.glsl:92-96` — o default do GP; com cor sólida é união
invisível). O "parte nova por cima" da 3ª rodada é **incompatível** com
zero-acúmulo no mesmo depth; o GP resolve com o modo opcional de material
`GP_STROKE_OVERLAP` (depth por-PONTO, aceita acumular) — não portado; se o Enio
quiser, é um flag de stroke + 1 linha no depth do vertex.

**Oráculo novo: paridade CPU↔GPU pixel-a-pixel.**
`gpu_render.rs::assert_matches_analytic` replica a geometria do vertex (quads
miter/break/ext, ponto-no-triângulo como o raster) + a máscara do fragment na CPU
e compara TODO o alvo (fundo incluso; pula só faixa de aresta e limiar de
discard). **Mutações provadas** (asserção-vermelha real): `GreaterEqual` de volta →
hairpin desvio 248 + cruzamento 191 (o acúmulo 0.75); `discard` removido →
desvio 254 no canto estendido que o traço cruza de volta.

> 🔴 **LEIA ANTES DE CONFIAR NO ORÁCULO:** ele modela a **implementação**
> (first-wins por depth), **não a aparência DESEJADA**. Por isso ficou verde com a
> mordida na tela — a mordida É o first-wins. **Primeira coisa a fazer no fix:**
> troque o `expected_alpha` para o **máximo** da máscara sobre TODOS os segmentos
> que cobrem o pixel (= a distância à POLILINHA, a união real). Aí ele fica
> **VERMELHO no código de hoje** e vira o alvo irrefutável do fix.

### A mordida (o bug do smoke) — mecanismo e fix

**Mecanismo (deduzido, ainda não instrumentado):** numa quina QUEBRADA, os quads dos
segmentos A (anterior) e B (seguinte) **se sobrepõem** no disco da junção (ambos
estendem `r`). Mesmo `sid` → **mesmo depth** → com `GREATER` estrito **A vence TODOS
os pixels compartilhados** (chega primeiro). Mas a máscara que A pinta ali é a
**queda RADIAL** dele (distância ao seu próprio ponto final, clampada) — enquanto os
pixels que estão sobre o **eixo de B** deveriam ter cobertura ~1 (são o núcleo de B).
Com hardness < 1 a queda radial é < 1 → **um "mordido" macio no lado interno da
quina**. Com hardness = 1 a máscara é degrau (1 dentro do disco) → invisível — por
isso os testes de geometria com borda dura passam.

**O fix (é o refino p0/p3 que eu descartei — o handoff anterior estava certo):** com
depth first-wins, **o fragmento vencedor precisa conhecer a vizinhança** — ele tem de
computar a distância à **polilinha** (mín. entre os segmentos p0→p1, p1→p2, p2→p3),
não só ao seu próprio segmento. É exatamente por isso que o GP passa `p0`/`p3` ao
`gpencil_stroke_segment_mask`. Passe os 2 vizinhos (já dá pra ler do storage buffer
no vertex, como o `sp`/`sn` do miter) e use `dist = min(...)` na região de quina.
**Alternativa** (se o p0/p3 não fechar): render do traço em 2 passes — cobertura numa
scratch com blend **MAX** (união sem acúmulo, sem truque de depth) + 1 composite —
é o jeito padrão de traço macio; custa um alvo a mais.

**Gate:** 23 verdes em debug E `--release` (12 unit + 2 composite e2e + 9 GPU).
`rustup run 1.95 cargo fmt --check` + clippy `--all-targets` limpos. Diff: só
`ph2d-flip-render` (flip.wgsl, flip_fill.wgsl, pipeline.rs, tests/gpu_render.rs) —
zero shell, zero foundational, zero contrato.

**Smoke (reprovado):**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && cargo run -p ph2d-host-desktop --release
```

## Smoke do Enio (2026-07-11, 6ª rodada) — acúmulo de cor nas QUINAS (spikes/estrelas)

**Quinas afiadas acumulavam cor (spike/estrela na bissetriz); cruzamentos não** ✅ — o Enio
suspeitou das "normais da face" e acertou: a **fita conectada por miter DOBRAVA** sobre si
numa quina afiada (bowtie = triângulo invertido/auto-sobreposto), e o premult-over acumulava
ao longo da dobra. Fix = **estado EXATO do GP 2D** (`gpencil_cache_utils.cc:449`:
`WRITE_DEPTH | BLEND_ALPHA_PREMUL | DEPTH_GREATER`): geometria volta a quad-**stadium**
CONVEXO (nunca dobra) + depth **GREATER estrito** + write-depth → a 2ª face no mesmo pixel
(quina/junção/cruzamento da mesma linha) é **descartada, não misturada** → zero acúmulo. Numa
cor sólida a sobreposição vira união limpa. O fragment analítico segue carregando a forma
redonda. Isto **subsome a 5ª rodada** (o descarte também mata o bead das junções retas).
`GreaterEqual` + fita-conectada foram ABANDONADOS (acumulavam / bowtie). GPU: novos
`a_sharp_corner_does_not_accumulate_color` + `a_stroke_crossing_itself_is_a_clean_opaque_union`
+ 6 = 8 render + 2 composite verdes.

## Smoke do Enio (2026-07-11, 5ª rodada) — "mastigado" (bead) com hardness baixo

**Linha "mastigada" com hardness < 1** ✅ — beads a cada ponto. Causa: os quads-stadium
de segmentos ADJACENTES se sobrepunham na junção (cada um estendia `r` na tampa redonda),
e o `GreaterEqual` deixava o 2º compor por cima do 1º (premult-over) → onde a cobertura é
parcial (borda macia), `1-(1-a)² > a` → bead. O GP (draw ref: per-stroke depth +
`DEPTH_GREATER` + fita conectada) não sobrepõe. Fix: a geometria virou uma **FITA
CONECTADA** — segmentos adjacentes computam o MESMO vértice de junção (miter da bisetriz
prev/nn compartilhado) e ABUTAM em vez de sobrepor → um pixel coberto por UM segmento →
sem double-blend → sem bead. Extensão de tampa redonda só nos EXTREMOS. Cruzamentos REAIS
(não-adjacentes) ainda se sobrepõem → `GreaterEqual` mantém a parte nova por cima (ganho
da 3ª rodada preservado). O fragment analítico segue igual (o miter é só o teto da
geometria; a forma redonda sai do fragment). GPU: novo
`a_soft_stroke_has_no_bead_at_the_joints` + 6 anteriores + 2 composite verdes.

## W3 — Frames · Ghost Frames · Tween (LANDOU 2026-07-12, pendente o smoke)

O Flip virou um app de **animação** (antes era um app de desenho com playhead). O doc
definitivo — modelo de tempo, ciclos, algoritmo do onion, autokey por-tool, tween — é
[`docs/Flip/05_frames_ghost_tween.md`](Flip/05_frames_ghost_tween.md). Resumo do que existe agora:

- **Modelo** (`ph2d-flip`): `span`/`cells`/`set_exposure` (a exposição EMPURRA o resto) ·
  `cycle.rs` (pre/post behavior None/Hold/Loop/PingPong — os defaults reproduzem o
  pré-W3 byte a byte) · `onion.rs::ghosts()` (função PURA, port do `get_frame_id`) ·
  `autokey.rs::ensure_key` (política por ferramenta) · `tween.rs` (pareamento por índice +
  padding ao MAX + auto-flip + breakdowns idempotentes). **`FLIP_SCHEMA_VERSION` 1→2**
  (camada ganhou `cycle`+`use_onion`; `OnionSettings` ganhou `kind_filter`) e
  **`PROJECT_SCHEMA` 2→3** por tabela.
- **Render**: `CameraRaw::with_ghost_tint` + `ghost_tint: vec4` nos DOIS shaders (traço e
  fill) — o fantasma é a silhueta 100% recolorida, alpha `1/|Δ|` piso 0.1. Passe em
  `render_loop/flip_pass_ghosts.rs` (antes do composite; some no play; usa o cache de
  tesselação por desenho — custo ~zero).
- **Tira** (`ph2d-panel-flip-frames`, painel NOVO, faixa inferior `layout.flip_strip`):
  células = botões canônicos com a exposição, transporte (play + flip por DESENHO), Ghost,
  Auto/Additive, Add/Dup/Delete/Hold/±1, Tween, Cycle. Sobe acima do timeline global quando
  ele está aberto.
- **Shell**: `flip_strip.rs` (estado de autoria + drain dos eventos) · `flip_autokey.rs` (o
  ponto ÚNICO que decide o desenho-alvo — caneta = branco/Additive; borracha = SEMPRE
  duplicata) · atalhos `↑`/`↓` (flip por desenho) e `,`/`.` (±1 quadro **no FPS do objeto**).
- **Gates novos**: `ph2d-panel-flip-frames/tests/seam.rs` (todo controle chega ao barramento —
  botão novo sem braço = vermelho) + 1 teste GPU do fantasma + ~30 unit no modelo/shell.

**Carry-overs declarados:** drag de célula/borda na tira · multi-seleção de chaves (destrava o
modo `Selected` dos fantasmas, já pronto no modelo) · picker de easing + fade-in de órfãos na UI
do tween (o motor suporta) · cache de playback (só com bench antes) · light table.

## W4 — Fill (o balde) (LANDOU 2026-07-12, pendente o smoke)

Doc definitivo: [`docs/Flip/06_fill_balde.md`](Flip/06_fill_balde.md).

- **Solver** (`ph2d-flip-fill`, crate NOVA, CPU pura, headless): `gap.rs` (Gap Closure —
  pontas + quinas apertadas pela bissetriz externa, corte por colisão, extensão que não
  colide é descartada) · `raster.rs` (buffer de FLAGS; fronteiras **no EIXO da polilinha**
  — ver §W4.1, que revogou o `radius_scale = 0.5` do 1º corte; span fill + filtro de
  vazamento CRUZADO; Grow/Shrink; o flood REPORTA o vazamento) · `trace.rs` (marching
  squares — os buracos saem de graça — + RDP).
- **Modelo**: `FlipStroke.holes` + `hide_stroke`. Um fill é UM traço com seus buracos (não N
  traços com `fill_id`): é uma unidade de seleção/undo/delete/animação.
  **`FLIP_SCHEMA_VERSION` 2→3.**
- **Render**: `fill_holes.rs` — decomposição trapezoidal **even-odd** (exata, robusta a
  buracos e auto-interseção). Ear-clipping com pontes travaria.
- **Costura**: modo **Fill** na tool (4º modo) + seção do painel (cor PRÓPRIA do balde,
  Paint/Behind/Unpaint, Gap/Grow/Precision) + `flip_fill.rs` no shell (a fronteira
  modelo↔solver) + o clique no `input_dispatch`. O desenho-alvo vem do autokey por-tool
  com política **Modify** (preencher é MODIFICAR: no rabo de um hold a chave nasce
  duplicata, nunca em branco).
- **O twist do Harmony**: o fechamento de gap vira **traço invisível PERSISTENTE** — re-fill
  com outra cor, no quadro vizinho, ou amanhã, não depende de a ferramenta estar com os
  mesmos parâmetros.

**Carry-overs:** fill multiframe (depende da multi-seleção de chaves da tira) · ajuste modal
ao vivo do Gap Closure (o `closures()` já devolve os segmentos; falta o overlay) · modo Radius
· Colorize (LazyBrush/trapped-ball) é wave própria.

## W4.1 — A âncora do fill é o EIXO da linha (2026-07-12, fecha o BUGS #14 — pendente smoke)

O 5º smoke do balde ("Piorou. Linhas finas nem têm valor no slider… grow 0 e −1") tinha causa
provada: espessura **absoluta em px de TELA** × fill **assado em unidades de DOCUMENTO** — a
silhueta do zoom do clique transborda `(w/2)·(zoom−1)` px quando a câmera aproxima depois. A
saga completa (medições antes/depois, trade-offs decididos, lições) está em
[`BUGS_flip.md` #14](Flip/BUGS_flip.md); o resumo do que mudou no código:

- **`fill_at` passo 3:** parede E `INK` rasterizam **no eixo** (raio 0). A espessura só folga
  o bbox. `max_ink_px` morreu.
- **`fill_at` passos 5/6, SEM RAMO:** `expand_under_ink(AXIS_COVER_PASSES = 3)` crava a borda
  da cor em cima do eixo (senão ela para na face interna da parede, ~1 px aquém), e o Grow é
  `grid.grow(params.grow)` direto — offset assinado do eixo, **contínuo em 0**.
- **`Grid::strip_ink` DELETADO** (era a âncora dupla que saltava w+1 px entre 0 e −1 — o
  commit `111637cd` ficou superseded por construção).
- **Testes movidos para `ph2d-flip-fill/src/tests.rs`** (o lib.rs estourou o cap de 700; o
  gate exclui `src/tests.rs`). Gates novos/reescritos, todos provados VERMELHOS antes do fix:
  `the_baked_fill_stays_under_the_line_at_any_later_zoom` ·
  `the_grow_slider_is_continuous_through_zero` ·
  `the_colour_stops_at_the_line_axis_at_any_width` ·
  `a_negative/positive_grow_*_the_contour_the_same_at_any_line_width` · `sweep_table`
  (`--ignored --nocapture`: a régua espessura×zoom em px de tela).
- **Comportamento novo (deliberado):** clicar no CORPO de uma linha grossa preenche o lado
  clicado (só o eixo recusa com `OnBoundary`); corpos que se sobrepõem sem os eixos se
  cruzarem dependem do Gap Closure (toast já sugere).
- Gate batched: 34 fill + 66 flip + 7 tool + 10 painel + shell OK; 17 GPU do traço OK;
  clippy/fmt-pin/typos/LOC-caps limpos; release build OK. `flip_live` intacto (a âncora não
  depende do zoom da vista — o `px_to_world` da criação continua correto).

## Aberto (fora do W0..W4, por design)

- **W5 (próxima):** Reshape (escultura de traço) — os 9 pincéis, com TODAS as constantes já
  tabeladas em `docs/Flip/02 §7`.
- **W6 (timeline global): ADIADA** — a timeline principal ainda está em desenvolvimento
  (Enio 2026-07-12). O playhead do Flip JÁ é o global, então a integração não terá relógio a
  reconciliar.
- **Refinos do Select (não-bloqueantes):** escala NÃO-uniforme engrossa o traço pela
  escala MÉDIA (`mean_scale`) — aproximação; espessura anisotrópica exigiria passar o
  afim ao shader. Persistência da pose Flip no `ProjectState` (o `Transform` é ECS →
  já entra no `WorldSnapshot`; a geometria local idem — deve funcionar, mas não
  smoke-testei o round-trip pós-move).
- **Refinos do painel/borracha (não-bloqueantes):** duplicar/agrupar camada
  (só `add`/`delete`/reorder landaram; `FlipObject` não tem `duplicate_layer`);
  reorder por DRAG (só ↑↓ por botão); máscaras de camada na UI (`FlipLayer.masks`
  existe no modelo, sem UI); curva de pressão editável + pen real (pressão=1.0 no
  mouse). Borracha: raio dedicado (hoje = tamanho do brush) + preview do círculo.
- **Deferido no W1 (v1 usa flat caps + miter clampado):** round caps, bevel/round
  joins. Máscaras de camada (`FlipLayer.masks`) — o modelo carrega, o compositor v1
  não aplica (o op-list GPU não tem máscara; igual ao Painter).
- **Cache de tesselação:** sem LRU (cresce com nº de desenhos únicos vistos —
  bounded pelo documento, ok pro W1). W2 pode adicionar cap se necessário.
- Persistir `flip` cross-sessão já funciona (entra no `ProjectState`); a UI real de
  Save/Open continua stub (herança do estado atual da persistência).
- **Docs de planejamento** (`docs/Flip/`, `docs/architecture/decisions/0114-*.md`,
  `project-memory/project_flip_module_grease_pencil_2d.md`) estão **untracked na
  árvore primária** — NÃO os commitei nesta linha (senão o `merge --ff-only` da
  integração quebra com "untracked working tree files would be overwritten"). O Enio
  deve commitá-los ao `main` por fora, antes ou depois da integração.
