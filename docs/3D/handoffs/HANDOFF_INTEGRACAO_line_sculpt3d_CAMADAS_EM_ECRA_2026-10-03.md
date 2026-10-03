# HANDOFF de INTEGRAÇÃO — `line/sculpt3d` · as camadas do Painter juntam-se em TONS DE ECRÃ + a pilha de camadas na peça 3D (2026-10-03)

> **Para quem é:** o agente INTEGRADOR (só por ordem do dono — CLAUDE.md §0.7). Superfície de colisão,
> contadores como DELTA, o que um merge pode partir, a prova de fecho e o que fica aberto. O mecanismo
> vive nos docs linkados: [doc Painter 45 §8](../../Painter/45_plano_as_camadas_juntam_se_em_tons_de_ecra.md)
> (P0–P4) · [ADR-0177](../../architecture/decisions/0177-as-camadas-do-painter-juntam-se-em-tons-de-ecra.md)
> · [doc 3D 30](../30_plano_camadas_e_efeitos_na_peca.md) (W1 §10, W2 §11, W3 §12, W1b §13).
>
> ⚠️ **SUPERSEDE** o [handoff A_INCLINACAO (02/10)](HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md)
> como documento de integração desta linha. Aquele **ainda NÃO está no `main`** (o merge-base não o
> tem): os seus 3 commits (`9f26085a1`, `c79e53b7b`, `b88d9462d`) vêm nesta mesma série — leia-o para o
> relevo (doc 29 §9: `Inclinacoes`, `@binding(8)` do `tinta.wgsl`, `upload_tinta_amostras_at` ganha
> `mesh: &Mesh`). Tudo o que ele declara continua válido e **soma-se** ao que está abaixo.
>
> Continuações (mecanismo; cada uma supersede a anterior): [W1](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W1_2026-10-02.md) · [W2](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W2_2026-10-03.md) · [W3](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W3_2026-10-03.md) · [W1b](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W1b_2026-10-03.md) · [P2](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_EM_ECRA_P2_2026-10-03.md) · [P3](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_EM_ECRA_P3_2026-10-03.md).

## §0. Estado em uma tabela

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` |
| merge-base | `1ad60a1ce` (= `main` a 03/10; rebase desnecessário à data) |
| commits | **48** (`git log --oneline 1ad60a1ce..HEAD`; HEAD `66f9bbcab` + o commit deste handoff) · **213** ficheiros, `+15 943 −2 178` |
| smoke do dono | 3D (report, W3) ✅ aprovado a 03/10 · **2D antes × depois: PENDENTE** (§7) |
| contratos §6 | **intocados** (`Tool=12`, `PanelEvent=4`, `NodeOp`; `collision-surface.sh`: «intocado» nos dois ficheiros) |
| `shells/desktop/src` | **0** linhas |

### Contadores — DELTA (a fonte é o código; reconte contra o `main` do dia)

| contador | esta linha | valor aqui · base |
|---|---|---|
| `PROJECT_SCHEMA` · tripla do gate · `VEC_SCENE` · `FLIP_SCHEMA` · timeline `DOC_VERSION` · `FIELD_DOC_VERSION` | **0** | 178 (178,13,22) · 22 · 13 · 18 · 23 — iguais à base |
| registos de componentes (ecs / render / script) | **0** | 108 / 109 / 109 |
| `SCULPT_DOC_VERSION` (`ph2d-app-sculpt3d/src/doc.rs`) | **+2** (5→7) | `7`: v6 = pilha de camadas (W1); v7 = o FUNDO da pilha (W1b). v5 e v6 abrem; o v6 está congelado em `doc_migracao.rs`. Gate `a_forma_da_pilha_gravada_e_pinada` (`doc_camadas_tests.rs`, pinos `83`/`3 590`) |
| ADR | **+1** | `0177` — ⚠️ a `line/3DModeling` tem `0176`; o próximo livre aqui é `0178`. **Reconta-se** (renumerar = ficheiro + `decisions/README.md` + as referências em docs/código deste ramo: `git grep 0177`) |
| `Cargo.lock` | **+1 pacote interno** | `ph2d-layer-ops` (crate NOVA, folha sem deps); nenhum pacote externo |
| `LayerCompositeError` | **+1 variante** | `AdjustmentInLightSpace` (§2) |
| `DropdownOption::disabled` (`ph2d-editor-core`, W3) | campo novo | todo literal de `DropdownOption` noutra linha ganha `disabled: false` |

## §1. O que a linha traz (uma linha por onda; o mecanismo está nos docs)

- **A lei (ADR-0177, dono 03/10):** as camadas do Painter — 2D e peça 3D — juntam-se em **tons de ecrã** (`decode = b/255`, `encode = round`), não em luz. Origem: o report «o traço na camada 2 é pior que na 1» = pincel codificado × compositor em luz (73 degraus). Oráculos: GIMP 3.2.6 (controlo «linear» = compositor antigo `≤1`; «perceptual» = o novo `≤1` nos 10 modos) e Krita 6.0.4 8 bits (22 modos, `≤4`).
- **W1–W3, W1b (peça 3D):** `PilhaDaPeca` (LayerStack do Painter sobre os planos de tinta, doc v6/v7),
  pintar na camada activa, painel de Layers sobre a pilha da peça (`PieceLayerOp`, sem variante nova de
  `PanelEvent`), composição **na placa** (arrasto de opacidade a 64x de ~39 ms para ~0,3 ms).
- **P1/P2/P3 (compositor):** CPU (`ph2d-tool-painter/compositor`) e gémeo WGSL em tons de ecrã; cada ajuste converte na SUA fronteira; dois defeitos corrigidos (mistura de volta do ajuste com o alfa da base; Threshold em `t/255`); aquarela alinhada.
- **P4 (esta janela):** os efeitos de vizinhança e os outros consumidores.
  - Oráculo ([`corre_vizinhanca.sh`](../../Painter/ferramentas/oraculo_camadas_gimp/corre_vizinhanca.sh)):
    **GIMP 3.2.6 borra em luz** (controlo: o nosso núcleo em luz `≤1` no miolo, 12 canais de borda `≤7`;
    o produto `≥60`); **Krita 6.0.4 8 bits borra em tons de ecrã pré-multiplicados** — Gaussian raio 9 =
    o nosso `gaussian_weights(9)` **ao byte** (0 de 10 800 canais), Motion `≤1`.
  - **Decisão:** Gaussian/Sharpen/Motion/Chroma em **tons de ecrã** (CPU `spatial.rs`
    `premultiply`/`unpremultiply`; placa `load_blur_tap`, `cs_combine`, `cs_chroma`). **Bloom fica em
    luz** (`premultiply_em_luz`/`unpremultiply_em_luz`; braço Bloom do `cs_combine`) — é óptico, não um
    kernel a decidir com os outros.

## §2. Foundational tocado e a superfície de API (o que um merge pode partir)

| item | mudança | quem parte |
|---|---|---|
| `ph2d_render::CompositeSpace { DisplayTones (default), Light }` · `LayerCompositor::with_space` | novos; **`LayerCompositor::new` = `DisplayTones`** | ⚠️ uma linha que construa `LayerCompositor::new` para um produto que junta em LUZ herda tons de ecrã em silêncio |
| `LayerCompositeError::AdjustmentInLightSpace` | variante NOVA: o compositor em luz recusa `Adjustment`/`SpatialAdjustment` nas 3 entradas (`composite_with_luts`, `composite_region_into_canvas`, `inject_slice_from_texture`) | ⚠️ um `match` exaustivo em `LayerCompositeError` noutra linha deixa de compilar |
| tabela de decode | 256 → **512** (cor + alfa); `DECODE_LUT_LEN` privado; constante de pipeline WGSL `override LIGHT_SPACE: bool` | um espelho CPU que assuma 256 entradas |
| `ph2d_flip_render::compositor_do_flip` | porta ÚNICA do Flip (`CompositeSpace::Light`), usada por `ph2d-app-flip/src/pass.rs` e `ph2d-app-motion/src/motion_flip_bake.rs`; `ph2d-gpu` passou de dev-dependency a **dependency** de `ph2d-flip-render` | um chamador novo do Flip que use `LayerCompositor::new` volta a partir o Flip |
| `ph2d_painter_effects::adjustments` — `apply_adjustment`, `apply_adjustment_windowed` e todo kernel público (`apply_gaussian`, `apply_bloom`, `apply_shadows_highlights`, `apply_noise`, `apply_halftone`, `apply_color_lookup`…) | o contrato passou de **luz** para o acumulador **CODIFICADO** (straight f32); a assinatura não mudou, o **compilador não avisa** | `git grep 'adjustments::apply_\|apply_adjustment\|LayerCompositor::new'` na árvore combinada; os blurs agora borram em ecrã |
| `ph2d-layer-ops` (NOVA, folha) | `LayerOp`/`LayerMask` saíram de `ph2d-render/src/layer_compositor/ops.rs` (que os reexporta: os caminhos `ph2d_render::layer_compositor::LayerOp` continuam) | quem tenha tocado `ops.rs` noutra linha |
| `ph2d-tool-painter` | `compositor/gpu_ops.rs` = `flatten_for_gpu` (veio de `ph2d-app-painter/src/painter_gpu_flatten.rs`, **módulo apagado**, 3 chamadores re-apontados); `pub use flatten_for_gpu`; o tradutor pilha → operações; `tool/layer_edit.rs`, `tool/piece_layers.rs`, `seed_user_adjustment`, `PieceLayerOp` público | quem edite `painter_gpu_flatten.rs` noutra linha (modify/delete) |
| `ph2d-mesh-render` | `tinta_achata.rs` + `shaders/tinta_achata.wgsl` (`AchataDaTinta`, `achata_tinta_at`, `le_tinta_at`); `tinta.wgsl` `@binding(8)` (A_INCLINACAO); buffer `amostras` ganha `COPY_SRC` | o fragmento do `tinta.wgsl` usa **7** buffers de armazenamento (piso WebGPU `8`): um binding novo noutra linha estoura |
| `ph2d-painter-effects` | `AdjustmentKind::reads_the_image_layout` (+ gate); `compute/shared.rs` `em_luz`/`em_tons_de_ecra`/`build_lut_em_luz`; Threshold = byte da luma `≥ t` | |
| `ph2d-mesh-colors` · `ph2d-sculpt3d` | `alfa.rs`, `inclinacao.rs`; leis de cor com opacidade (`tinta_fina*`, `tela_na_malha_pousa`, `preenche`, `tela_semente`); `TintaDoTraco::repinta` recebe/devolve `(cor, opacidade)` | |
| `ph2d-panel-painter-layers` · `ph2d-app-painter` · `ph2d-editor-core` · `ph2d-i18n` | painel em modo peça (`peca.rs`); a ponte publica `panel_layers`/`panel_selection`/modo peça; `DropdownOption::disabled`; chaves | ver §0 |
| `ph2d-app-sculpt3d` | `fn sync_mesh(&mut self, gpu: &GpuContext)` (assinatura nova); módulos `pilha_da_peca*`, `doc_camadas`, `doc_migracao`, `composto_na_placa*`, `tinta_da_peca_pilha*`, `painter_na_malha_camadas` | o censo `tests/it/the_sculpt_mesh_edits_are_wired.rs:819` seguiu a assinatura (única linha de `tests/` fora da família) |

### Consumidores conferidos (premissas do briefing derrubadas)

- **Flip: «herda» era o DEFEITO.** A P1 mudou o compositor partilhado e o Flip herdou em silêncio (branco
  50 % numa camada sobre preto: `0,216` linear vs `0,5` na mesma camada; o gate de placa
  `composite_blend::top_layer_opacity_fades_toward_backdrop` ficou vermelho, fora das corridas). Cura:
  `compositor_do_flip` (luz). Gates: `ph2d-flip-render tests/it/composite_em_luz.rs::{a_camada_de_cima_junta_se_como_um_traco_na_mesma_camada, a_porta_do_flip_recusa_um_ajuste}`.
- **FX raster do Vector: a lei NÃO se aplica (medido).** O mundo vetorial compõe em luz: âmbar
  `(235,175,60)` a meia cobertura = `(173,128,41)` = `encode(0,5·linear)` ao byte; em ecrã seria
  `(118,88,30)`. Recusa no ADR (tabela `⛔ Recusas MEDIDAS`).
- **Herdam:** Painter 2D (`painter_gpu_preview.rs`, `tool/runtime.rs`) e peça (`pilha_da_peca.rs`, `composto_na_placa.rs`). **Não são consumidores:** bake de sprites, Composite brush (pilha numa camada), Wet Paint, Impasto.

## §3. Para o integrador — linhas abertas

- **`line/PainterWatercolor`**: re-pina (a) qualquer composto com camada de ajuste sobre zona
  translúcida (a mistura de volta mudou), (b) Threshold, (c) **qualquer desfoque/sharpen/motion/chroma
  de camada de ajuste** (agora em ecrã), (d) pinos da aquarela (`watercolor_aa` já movido aqui: 405
  bytes, todos por 1).
- **`line/3DModeling`**: ADR `0176` ↔ `0177`; mais nada colide (`collision-surface.sh`: sem marcadores,
  Cargo.lock só `ph2d-layer-ops`).
- Tecto de LOC: nenhum arquivo da linha passa; `ph2d-app-painter/src/painter_bridge.rs` está a
  `699 / 700` (allowlist/marcador — um merge que lhe some linhas estoura).
- Catracas/listas BAIXADAS: nenhuma. Itens partilhados com usos apagados: `painter_gpu_flatten` (3 chamadores re-apontados); `tinta_da_peca::{empresta, garante_no_orcamento}`/`PlanoInteiro::de` passaram a `#[cfg(test)]`.

## §4. Gates novos desta janela (P4)

`ph2d-painter-effects adjustments::vizinhanca_tests` (5) · `ph2d-tool-painter compositor::oraculo_vizinhanca_tests`
(2) · placa `ph2d-render layer_compositor_ajustes_gpu::{gpu_os_desfoques_borram_em_tons_de_ecra (0 canais a 1
nos 4 casos), gpu_o_chroma_junta_em_tons_de_ecra}` · Flip (§2). **Re-pinados:**
`adjustments::tests::gaussian_blur_spreads_an_impulse` (energia em tons de ecrã); os espelhos
`cpu_combine`/`cpu_blur`/`cpu_motion_blur`/`cpu_seg` do `layer_compositor_gpu.rs` (renomeados, sem
`_linear`); `decode_lut_is_the_cpu_decode` (512, os dois espaços). Os gates das ondas anteriores estão nos docs linkados.

**Mutação:** [`muta_as_camadas_em_ecra.sh`](../../Painter/ferramentas/muta_as_camadas_em_ecra.sh) —
**50/50 sangram** (controlos: cpu 44, efeitos 147, tabela 1, placa 37, flip 19); corrida `flip` nova;
`PH2D_PRAZO=7200`.

## §5. Prova de fecho (03/10, batched sobre o diff acumulado)

| passo | resultado |
|---|---|
| `nextest-impacted.sh` | **19 417 / 19 417** (11 557 fora do alcance; nenhum flake de carga) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| `cargo clippy -D warnings --all-targets` · `fmt --all --check` | ✓ · ✓ |
| `machete` · `check-standalone-optional` · `check-workflow-packages` | ✓ · ✓ · ✓ |
| `architecture_*` | **102 / 102** · memória `o_indice_da_memoria_conta_o_que_aponta` **3 / 3** |
| placa | `ph2d-render` **52/52** (`layer_compositor` + `fx_stack_adjust` + `blend_mode_regression` + `layers_no_alloc`) · `ph2d-flip-render composite_` **19/19** · sculpt3d `--ignored tinta_no_produto --skip diag_` **34/34** |
| pré-voo dos 23 `muta_*.sh` | só B1/B5/B14 de `docs/Skeleton/ferramentas/muta_a_silhueta_do_contacto.sh` — partidas **desde o `main`**, não desta linha |

Memória: 2.º caso acrescentado a `project-memory/feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site.md` (o índice não mudou).

## §6. Perfil de agente (`bash scripts/agent-loop-profile.sh`, 20 sessões)

paralelismo `1,15` · respostas/sessão `167` · `test:check` `2,6` · edições pela `Edit` `39 %` · contexto
por passo `334 mil` · início `63 mil`.

## §7. Smoke (do dono: o 2D, antes × depois)

### § Binário

```
bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
<<SAIDA_DA_2A_CORRIDA>>
```

### Passos (texto para o Enio)

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/smoke_camadas_antes_e_depois.sh`
   (corre a lei velha num cantinho à parte e a nova aqui; demora uns minutos).
2. Quando acabar, abra a pasta `target/smoke_camadas/` e olhe, por esta ordem, `lado_a_lado.png` (a
   esquerda é o ANTES, a direita o DEPOIS), `diferenca.png` (o que mudou, mais claro = mais diferente),
   e `antes.png` / `depois.png` se quiser ver cada um sozinho.
3. O que tem de acontecer: as camadas meio transparentes e os modos de mistura ficam com a cor que o
   GIMP e o Krita dão; as meias-sombras deixam de «clarear». Cerca de 113 mil de 256 mil pontos mudam, o
   pior por 73 degraus de cor — é a correcção, não um estrago.
4. Deu errado se o DEPOIS aparecer preto, vazio, com quadrados soltos, ou igual ao ANTES (então nada
   mudou); ou se o comando terminar com erro vermelho.

## §8. `CLAUDE.md` §5.1

Aplicado neste ramo: só o link «Último» da entrada «3D / Sculpt» (`handoff 03/10`).

## §9. ABERTO (nomeado, não da lei)

- A tabela de 256 do **Levels** erra 2 degraus junto do ponto preto (a função exacta dá 0); a placa liga
  a tabela.
- O **Chroma da CPU** diverge do da placa em sinal/centro/normalizador (documentado em `spatial.rs`; sem
  chamador de produção).
- O **Sharpen da placa** mistura em straight e o da CPU em pré-multiplicado (só difere em píxeis
  translúcidos; pré-existente).
- W1b: **tecto de camadas a `256x`**; o fundo semeado por quadro.
- Do relevo (A_INCLINACAO §9): re-subida do plano inteiro a cada quadro de escultura (`6,1` ms a 64×; pré-existente).
- ✅ Fechados: o report do dono (traço em camada nova = traço na base, `0,000` na peça); a etapa 4 do
  Painter na peça (camadas e efeitos — era o ⏳ do A_INCLINACAO); o Flip protegido por porta única.
