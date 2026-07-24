# Handoff de integração — `line/Painter` · GPU Ondas 1 e 2 (o compositor para de recusar o documento comum)

**Para:** o agente integrador (DIRETRIZ §1.5.9). **Data:** 2026-07-23.

> ⚠️ **Esta branch carrega DUAS waves independentes.** A anterior (transferência
> sRGB tabelada do Wet Paint) tem handoff próprio:
> [`HANDOFF_INTEGRACAO_line_Painter_wet_transfer_2026-07-23.md`](HANDOFF_INTEGRACAO_line_Painter_wet_transfer_2026-07-23.md).
> As duas não se tocam em arquivo nenhum; integram juntas por estarem na mesma branch.

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| HEAD | `df3b49207` |
| base do fork | `df91ef6ec` |
| commits desta wave | **6** (`97f0ab0a2` … `df3b49207`) |

1. `97f0ab0a2` — `fix`: camada de **REFERÊNCIA** para de derrubar o documento na CPU + os 3 doc-comments obsoletos + o gate de conjuntos.
2. `b7b6037e6` — `feat`: **MÁSCARA e CLIPPING viram ops** (o grosso da wave).
3. `1c3152b86` — `test`: os sítios de construção da `ph2d-flip-render`.
4. `af0dd7648` — `perf`: o **orçamento de camadas vem do dispositivo** (8 → 16 a 4K).
5. `bbbdb4e78` — `docs`: doc 25 §10, o que as ondas entregaram.
6. `df3b49207` — `style`: fmt canônico em 2 arquivos.

## 2. O que muda, em uma frase

O compositor GPU **já existia e recusava o documento comum**. Uma máscara de
camada mandava a pilha inteira para o produtor CPU: medido a 4096², a composição
ia de 0,665 para 74,0 ms e um arrasto de slider HSB de 0,738 para **652,9 ms**
(1,5 fps), disparado por um checkbox, sem nada na tela dizendo por quê.

| documento | tela | antes | agora | ganho |
|---|---|---|---|---|
| 2 camadas, uma mascarada | 4096² | 74,02 ms | **0,741 ms** | **100×** |
| 1 raster + HSB mascarado | 4096² | **652,92 ms** | **0,831 ms** | **786×** |
| 16 camadas (sem máscara) | 4096² | ⛔ recusado → 254 ms | **3,88 ms** | **65×** |

E a frase que resume: **a máscara passou a custar +9%** (0,741 vs 0,680 ms), não
uma troca de máquina.

## 3. Foundational / compartilhado tocado

**Sim — `ph2d-render` é foundational e o contrato público dela MUDOU.**

### 3.1 Superfície pública alterada (ponto de atenção nº 1 do integrador)

| símbolo | mudança |
|---|---|
| `LayerOp::Layer` | **+2 campos**: `mask: Option<LayerMask>`, `clipping: bool` |
| `LayerOp::Adjustment` | **+1 campo**: `mask: Option<LayerMask>` |
| `LayerMask` | **NOVO** (`{ key: u64, inverted: bool }`), re-exportado por `ph2d_render` |
| `flatten_layer_ops` | **+1 argumento**: `mask_slot_of: impl Fn(u64) -> Option<u32>` (3 → 4) |
| `LAYER_CACHE_BUDGET_BYTES` | **RENOMEADO** → `LAYER_CACHE_BUDGET_SHARED_BYTES` |
| `LAYER_CACHE_BUDGET_DISCRETE_BYTES` | **NOVO** |
| `layer_cache_budget(DeviceType) -> u64` | **NOVO** |
| `LayerCompositor::cache_budget_bytes()` | **NOVO** (leitor) |
| `layer_compositor::ops` | módulo **NOVO** (interno; re-exporta pelo caminho antigo ⇒ **nenhum chamador quebra**) |
| `adjustments::gpu_codes` | módulo **NOVO** (interno; os 3 métodos seguem em `AdjustmentKind`) |

⚠️ **Apender campo a variante de enum quebra TODO sítio de construção** — foram
**75**, em 8 arquivos, quase todos de teste. Se outra linha tiver criado um
`LayerOp::Layer` novo, o merge vai **compilar-quebrado**, não conflitar: o
sintoma é `E0063 missing fields \`clipping\` and \`mask\``. A correção é mecânica
(`mask: None, clipping: false`).

### 3.2 Arquivos

`ph2d-render`: `layer_compositor/{mod,ops(NOVO),tests}.rs` ·
`layer_compositor/compositor/{mod,api}.rs` · `shaders/layer_composite.wgsl` ·
`lib.rs` · `tests/{layer_compositor_gpu,layers_no_alloc}.rs`.
`ph2d-painter-effects`: `adjustments/{mod,tests,gpu_codes(NOVO)}.rs`.
`ph2d-flip-render`: `tests/composite_blend.rs` (só sítios de construção).
`shells/desktop`: `render_loop/{painter_gpu_flatten,painter_gpu_preview,flip_pass}.rs`.

⚠️ Os dois arquivos NOVOS são **módulos irmãos** (DIRETRIZ §1.5.2.1) — nasceram
de splits de LOC, não engordaram nada compartilhado.

## 4. Contratos congelados encostados

**NENHUM.** `Tool` / `RasterEditTool` / `CanvasPaintTool` / `PanelEvent` intactos
(conferido por grep, não por auto-relato). `NodeOp`/`OpResolver`/`NodeManifest`
idem. **Nenhum schema bumpou**: `PROJECT_SCHEMA` fica **29**, `DOC_VERSION` e
`VEC_SCENE` intactos — a wave não toca persistência.

⚠️ `LayerOp` **não é serde** e não viaja em arquivo nenhum; é a linguagem que o
shell fala com o compositor dentro de um frame.

## 5. Símbolos que podem COLIDIR com outra linha

Nada numerado: sem id de widget, sem token, sem chave i18n, sem `NodeId`, sem
número de ADR, sem variante de enum serializado. O ponto de colisão real é
**estrutural**, não textual: qualquer linha que construa um `LayerOp::Layer`.

Consts novas no WGSL (`NO_MASK_SLOT`, `FLAG_CLIPPING`, `FLAG_MASK_INVERTED`,
`NO_CLIP_BASE`, `MAX_STACK`) vivem só em `layer_composite.wgsl`.

## 6. O que só o `ship.sh` pega

- `cargo fmt --all --check` limpo em toda a árvore; `clippy --all-targets` limpo
  nas 4 crates tocadas; `typos` limpo.
- **Nenhuma dep nova**, nenhum `Cargo.toml` tocado, `Cargo.lock` intocado.
- Gates de workspace rodados **no ÚLTIMO commit** (a lição que esta branch já
  pagou uma vez): `architecture_workspace_file_loc_cap` ✓ · `arch_safe_clamp_only` ✓
  · `shells/desktop::file_loc_caps` ✓.
- `cargo nextest --workspace`: **8896 passaram, 0 falharam**.
- ⚠️ **A suíte GPU é `#[ignore]`d e o `ship.sh` NÃO a roda.** Rodei aqui:
  `cargo test -p ph2d-render --release --test layer_compositor_gpu -- --ignored` →
  **35/35**. **Rode-a na integração** — é ela que prova a paridade.
- LOC: dois arquivos estouraram e foram **divididos**, não isentados —
  `layer_compositor/mod.rs` 1077 → 789 (via `ops.rs`) e
  `adjustments/mod.rs` 956 → 863 (via `gpu_codes.rs`).

## 7. O que smoke-testar

**`PH2D_IMPASTO_SMOKE=1 cargo run -p ph2d-host-desktop --release`** (ou qualquer
cena que abra o Painter) → painel **Layers**:

1. **Adicione uma MÁSCARA** a uma camada e pinte nela. A arte deve responder
   exatamente como antes — o que mudou é o custo.
2. **Arraste um slider de ajuste** (HSB) num documento **com máscara**. Antes
   isto era ~1,5 fps a 4K; deve estar instantâneo.
3. **CLIPPING**: marque uma camada como clipada sobre outra. A tinta deve
   aparecer só onde a de baixo tem alpha, e **duas clipadas seguidas** devem
   ambas ler a MESMA base (não compor uma sobre a outra).
4. **Máscara invertida**: o botão de inverter deve inverter — e não inverter
   *todas* as máscaras do documento.
5. **Um documento 4K com 9..16 camadas** deve continuar fluido (antes caía para
   a CPU na nona).
6. ⚠️ **A regra-mãe do smoke: a APARÊNCIA não pode mudar.** Esta wave é
   roteamento e custo. Se alguma cor/borda mudar, é regressão — e a paridade
   está gateada, então seria um caso que os gates não contêm.

**Não smokado por mim:** só rodei gates headless (com device real); nenhuma
janela foi aberta.

## 8. Ordem / dependências

Os 6 commits são sequenciais. O conflito plausível é em
`crates/ph2d-render/src/layer_compositor/mod.rs` (o arquivo foi **dividido**, o
que faz um diff grande) e nos sítios de construção de `LayerOp`. A resolução é
**semântica**: o modelo de op mora em `ops.rs` agora, e todo construtor precisa
dos dois campos novos.

## 9. Aberto (nomeado, não escondido)

- **Onda 3** (passes de Watercolor/Impasto) e **Onda 4** (o ADR do Wet Paint)
  não foram tocadas — o doc 25 §7 tem o plano.
- **Seis ajustes** seguem forçando a CPU: `ColorBalance`, `GradientMap`,
  `PhotoFilter`, `SelectiveColor`, `ChannelMixer`, `BlackAndWhite`. ⚠️ Nem todos
  cabem no orçamento de 3 escalares do `AdjParams`; o **GradientMap é um LUT de
  256 entradas e já cabe na máquina de `adj_luts`** que o Curves/Levels usa —
  é a próxima peça de melhor razão custo-benefício.
- **Grupo mascarado/clipado** continua recusado **de propósito** (a referência
  CPU ignora os dois; honrá-los na GPU faria a arte depender de qual produtor
  ganhou o frame). Fechar exige consertar a CPU primeiro.
- **Máscara de ajuste ESPACIAL** cai na CPU: o combine do pass-graph não tem
  entrada de máscara.
- Subir o orçamento além de 1 GiB exige **alocação falível**
  (`push_error_scope(OutOfMemory)`), não um literal maior.
- ⚠️ **Flake conhecida, pré-existente:** `the_cost_of_depth_is_linear_not_explosive`
  (`ph2d-timeline`) — gate de razão sensível a carga; passa isolada.

Detalhe completo, com as duas tabelas e as decisões de espelhamento:
[`docs/Painter/25_avaliacao_gpu.md`](Painter/25_avaliacao_gpu.md) §10.
