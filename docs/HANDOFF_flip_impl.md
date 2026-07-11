# HANDOFF de integração — linha `line/FLIP`, Waves W0 (dados) + W1 (render GPU)

> Entregável §1.5.9 (DIRETRIZ). A linha está **fechada e PARADA** — não integrei
> nem pushei. Este doc é o que o Enio passa ao agente integrador.

## 1. Identidade

- **Branch:** `line/FLIP`
- **HEAD:** `48b590d7`
- **Base (merge-base com `main`):** `1c7c9a22`
- **Commits na linha:** 15 (todos `--no-verify`, fast mode)

```
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
| `crates/ph2d-flip-render/**` (crate NOVA) | pipeline wgpu do traço + fill + composição por-camada | drop-crate isolada (glob `crates/*`); NÃO vai pela `vello::Scene` — passe wgpu dedicado (ADR-0113) |
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

## Aberto (fora do W0/W1, por design)

- **W2 (tool + painel):** criar/editar objeto Flip pela UI (pill + `IconId` + painel
  docado), desenhar traço com pointer. O `flip_entities::sync` já está wirado e vira
  ativo assim que a tool criar objetos.
- **Deferido no W1 (v1 usa flat caps + miter clampado):** round caps, bevel/round
  joins. Máscaras de camada (`FlipLayer.masks`) — o modelo carrega, o compositor v1
  não aplica (o op-list GPU não tem máscara; igual ao Painter).
- **Cache de tesselação:** sem LRU (cresce com nº de desenhos únicos vistos —
  bounded pelo documento, ok pro W1). W2 pode adicionar cap se necessário.
- Persistir `flip` cross-sessão já funciona (entra no `ProjectState`); a UI real de
  Save/Open continua stub (herança do estado atual da persistência).
- **Docs de planejamento** (`docs/Flip/`, `docs/architecture/decisions/0113-*.md`,
  `project-memory/project_flip_module_grease_pencil_2d.md`) estão **untracked na
  árvore primária** — NÃO os commitei nesta linha (senão o `merge --ff-only` da
  integração quebra com "untracked working tree files would be overwritten"). O Enio
  deve commitá-los ao `main` por fora, antes ou depois da integração.
