═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–20 em `line/motion-value`, REBASEADA no `main` `782f90b69` (10/10), POR INTEGRAR.
O ciclo 20 (O ACABAMENTO, [doc 129](../129_ciclo_20_o_acabamento.md)) está ABERTO: W0, W1 e W2
fecharam (a lente `fx.lens` na placa e no grafo, a cena `=135`, **smoke do dono APROVADO 10/10**).
Você abre a **W3 — os filtros da camada (`fx.filter`)**.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 &&
        git status --short — ANTES de abrir qualquer ficheiro. (`git cherry main HEAD` > 0; não
        rebaseie de novo sem o `main` ter andado.)

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8). Integrar/pushar só por ordem dele.
⭐ LEIA o doc 129 (§3.3, §4, §5.2–§5.3, §6) e o [doc 94](../94_plano_fx_adjust.md) INTEIROS (curtos).
⭐ CUSTO: leitura larga a `explorador` em paralelo; gate batched a um `verificador`.

O QUE A W2 DEIXOU (não reconstrua):
- `crates/ph2d-node-fx-lens` (o nó), `ph2d-app-motion/src/motion_lens.rs` (`MotionState::lente`:
  `prepara` no `coze_o_quadro`, `liga` no plano do quadro, `aplica` em `present_chrome.rs` depois da
  colagem da placa), `HeroScreen::cena_visivel` (⛔ o quadro de um efeito de câmera é a cena VISÍVEL
  — o mundo renderiza por baixo dos painéis; medido na foto, doc 129 §5.3), `FileKind::Lut`, a recusa
  da LUT na linha de leitura do cartão, a cena `=135` (`motion_state_acabamento_demo.rs`,
  `MAX_DEMO_LEVEL = 135`).

O DESENHO DA W3 (MEDIDO por dois exploradores em 10/10 — confira no código antes de crer):
1. **O tonemap está em BYPASS** (`ph2d-render/src/shaders/tonemap.wgsl`, `BYPASS_LUT = true`:
   `clamp(hdr, 0, 1)`) ⇒ uma camada do Motion desenhada à parte pode dar as MESMAS cores do passe
   fundido sem o tonemap. ⚠️ Se o AgX for ligado um dia, este passe tem de o seguir — escreva o
   gate que o diz.
2. **Tirar o Motion do passe fundido** num quadro com `fx.filter`: `motion_slice = &[]` e
   `motion_gpu = None` em `shells/desktop/src/render_loop/present.rs` ~229-241 (o ramo vazio já
   existe). Onion, partículas e barras ficam no fundido. ⚠️ Isso tira o Motion do Z intercalado com
   os sprites da cena: a camada filtrada vai POR CIMA do mundo (a camada do After Effects). Diga-o
   no handoff e no tutorial.
3. **A camada isolada, no espaço do DESENHISTA** (o mesmo do acumulador): (a) os sprites do Motion
   por `renderer.render_instances_only(alvo_linear Rgba16Float, camera, window, TRANSPARENT,
   cpu, gpu_extra, scene_viewport)` — a rota que o halo do glow já usa (`present_fx.rs` ~107-117;
   `gpu_extra` = o buffer do dispositivo); (b) colados num alvo `D` (Rgba16Float, limpo a
   transparente) por `BandBlit` com `BandSource::Sprites` (linear premult → sRGB premult); (c) a
   placa das formas por cima, `BandSource::Formas` (`PlacaDeFormas::camada()`, já é sRGB premult
   meio-float). ⚠️ Formas da rota da CPU (Vello, quando `formas_pedem_o_vello`) NÃO estão no
   acumulador nem na camada — meça quando isso acontece numa cena com filtro antes de decidir.
4. **A pilha** `ph2d_render::FxStackPass::run(gpu, src, dst, w, h, ops, geom)` (`fx_stack.rs:323`):
   o ingest (`fx_stack_shader.rs:350-358`) espera sRGB RETO; a fonte `D` é sRGB PRÉ-MULTIPLICADA ⇒
   um flag no `Globals` (o `_pad3` hoje sempre `0`, `fx_stack_shader.rs:36` / `fx_stack_res.rs:91`
   / `fx_stack.rs:398`) que des-pré-multiplica antes do `srgb_to_linear3` — o Vector escreve `0` e
   fica byte-idêntico (gate). A saída é `Rgba8Unorm` sRGB RETO = a convenção `BandSource::Vector`.
   ⚠️ O que o `geom` vazio faz (distância estimada?) — leia antes de assumir.
5. **Colar** a saída sobre o `WorldRt` com `band_blit.blit(..., BandSource::Vector)` em
   `present_chrome.rs`, NO LUGAR da colagem da placa (a placa já está dentro da camada) e ANTES de
   `motion.lente.aplica`.
6. **O nó** `fx.filter` (doc 94 §1 inteiro): a ficha DERIVADA da `FxOp::SPECS`
   (`ph2d-fx-op/src/kinds.rs:119`, 15 tipos, `op.rs:175-250`), um `ParamGate` por knob, sentinela
   `-1`; NÃO é singleton — os `fx.filter` no fio, na ordem do fio, são a pilha.
7. **Mundo → pixel**: `ph2d_app_vec::fx_live_resolve::resolve_ops(filter, camera: Affine)`
   (`fx_live_resolve.rs:21`, `sigma_px = radius · cam_scale`). ⛔ NÃO copie a conta: mova-a para uma
   porta partilhada (ex.: `ph2d-fx-op` ou `ph2d-render`) e chame-a dos dois lados. `Camera2d`
   (`ph2d-render/src/camera.rs`): `height_world` é público; px por unidade = `altura_px /
   height_world`.
8. ⏳ **A faixa do `radius` em MUNDO: MEDIR, não escolher** (doc 94 §1) — a cena `=135` trabalha
   numa galáxia de `±4,5` unidades com discos de `0,15`.
9. Gate do blur contra `crates/ph2d-lens/tests/fixtures/blur_linear_s3.txt` (ImageMagick, σ = 3 px
   em luz linear, BORDA REPLICADA) — ⚠️ a pilha usa borda TRANSPARENTE (`fx_stack_shader.rs:237`):
   a cena do gate precisa de margem, ou compare só o miolo.

AS WAVES QUE FALTAM DEPOIS (doc 129 §4): **W4** — a cena `=135` ganha um filtro (sombra ou blur no
fundo) e as FOTOS com o controlo (`docs/Components/ferramentas/fotografa_cena.sh`); o relógio do
grupo (`--release`, `loadavg` ao lado: o passe da lente faz 16 nós × 2 `Φ` por pixel — meça a 4K);
o tutorial 13 (`tutoriais/src/13_o_acabamento.html` → `bash scripts/tutorial-pdf.sh`) com o gate dos
passos. **W5** — mutação (o sentido da aberração, a quadratura, a ordem fixa, o tique do grão, a
neutralidade byte-idêntica, a recusa da LUT no cartão, o quadro VISÍVEL), gate batched, handoff
`HANDOFF_INTEGRACAO_..._CICLO_20.md` + §5.1 do CLAUDE.md.

ABERTOS NOMEADOS: [BUGS #19](../BUGS_motion_nodes.md) (o ângulo do `fx.glow` gateado ao contrário —
precisa de uma espécie de gate «≠ 1») · do ciclo 19: a fita acima de 32 MiB · uma linha de COR
inteira num gesto · vida e dano pedem dois objectos · `Bezier`/`Tint` OKLCH caros na CPU de reserva ·
gate de gesto pelo caminho REAL do rato (BUGS #18).
═══════════════════════════════════════════════════════════════════
