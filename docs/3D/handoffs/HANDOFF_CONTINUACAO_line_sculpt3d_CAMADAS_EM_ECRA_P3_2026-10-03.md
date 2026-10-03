# HANDOFF de CONTINUAÇÃO — `line/sculpt3d` · as camadas em tons de ecrã: P3 fechada (2026-10-03)

> **Para quem é:** o agente que assume a linha para a **P4** — pelo bloco do
> [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).
> **SUPERSEDE** o [da P2](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_EM_ECRA_P2_2026-10-03.md) (válido
> para o que é da P0–P2). Não é handoff de integração: o de integração em vigor continua o
> [A_INCLINACAO](HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md). O mecanismo e as medições
> estão no [doc Painter 45 §8 (P3)](../../Painter/45_plano_as_camadas_juntam_se_em_tons_de_ecra.md) e
> no [ADR-0177](../../architecture/decisions/0177-as-camadas-do-painter-juntam-se-em-tons-de-ecra.md).

## §0. Estado

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` · merge-base `1ad60a1ce` (= `main` a 03/10, conferido no início desta janela) |
| commits desta janela | `cf7a509c3` (P3: código, oráculo, arnês) · `69b14e047` (os 3 gates que a mutação pediu + docs) · este handoff |
| decisões do dono | opção 1 (03/10) · placa ±1, gravar/exportar = CPU exacta · smoke do report aprovado — intocadas |
| ADR | `0177` (reconta-se na integração; a `line/3DModeling` tem `0176`) |
| `SCULPT_DOC_VERSION` · `PROJECT_SCHEMA` | intocados |
| contratos §6 | intocados (`Tool=12`, `PanelEvent=4`, `NodeOp`) |
| a shell (`shells/desktop/src`) | **0** linhas |

**Foundational tocado:** `ph2d-painter-effects` (o contrato dos ajustes), `ph2d-tool-painter` (o braço do
ajuste do compositor), `ph2d-render` (`layer_composite.wgsl`, docs do `layer_compositor`). Nenhuma crate
nova; nenhum id/const/variant novo.

## §1. O que a janela deixou

- **O contrato** de `ph2d_painter_effects::adjustments` (`apply_adjustment`, `apply_adjustment_windowed`
  e cada kernel PÚBLICO — `apply_gaussian`, `apply_bloom`, `apply_shadows_highlights`, `apply_noise`,
  `apply_halftone`, `apply_color_lookup`…) é o acumulador **CODIFICADO** (straight f32). A fronteira
  saiu do compositor e do `apply_adjustment_op` da placa para dentro de cada tipo; a tabela por tipo
  está no doc 45 §8. Ajudantes: `compute/shared.rs` `em_luz`/`em_tons_de_ecra`/`build_lut_em_luz`;
  na placa `em_luz3`/`em_tons_de_ecra3`.
- **A porta dos blurs:** `spatial.rs` `premultiply` (codificado → luz premultiplicada) /
  `unpremultiply` (→ codificado). Gaussian, Sharpen, Motion, Chroma e Bloom passam por ela — é ALI
  que a P4 muda o espaço do kernel, se o oráculo o pedir (na placa: o 1.º leitor `load_blur_tap`, os
  combines, o chroma, o brilho do bloom).
- **Shadows/Highlights é de ecrã de ponta a ponta** (o plano listava-o como de luz): CPU e placa sem
  conversão nenhuma.
- **Dois defeitos corrigidos, CPU e placa** (expostos pelo oráculo): (1) a mistura de volta do ajuste
  era `over` com o alfa da base — um ajuste a 100 % sobre píxel translúcido aplicava-se só em parte
  (79 degraus do GIMP); hoje as duas cores misturam-se como opacas e a cobertura fica; (2) o Threshold
  cortava em `t/255` e o cinzento exacto no limiar saía preto; hoje é o byte da luma `≥ t`.
- **O oráculo dos ajustes:** `docs/Painter/ferramentas/oraculo_camadas_gimp/oraculo_ajustes.py` +
  `corre_ajustes.sh` + `gimp_3.2.6_ajustes.bin` (8 corridas, deterministas); armadilhas no README.

## §2. Medido

| | antes | depois |
|---|---|---|
| composto de cada tipo NÃO neutro (24 × base opaca/translúcida), contrato novo × antigo, antes da correcção da mistura | — | **47/48 ao byte**; Invert opaco 12 bytes por 1 (o novo é o `1 − x` exacto) |
| idem, depois da correcção da mistura | — | base opaca: igual; base translúcida: os tipos por píxel mudam (é a correcção), os blurs não |
| Invert/Curves/Levels/Posterize/Threshold × GIMP «perceptual» | Invert `79` (translúcido) | `0`/`1`/`2`/`0`/`0` |
| controlo: GIMP «linear» × o ajuste em luz · × o nosso | — | `≤1` · `60`–`120` |
| ajuste neutro, 18 tipos | `0` | `0` |

## §3. Gates

| | |
|---|---|
| novos, sem placa | `compositor::oraculo_ajustes_tests::um_ajuste_de_ecra_e_o_do_gimp_perceptual` · `invert_is_the_exact_display_negative_at_every_byte` · `threshold_is_the_luma_byte_at_least_the_threshold` · `vibrance_desaturates_to_the_oklab_lightness_of_the_light` · `contrast_pivots_on_the_mid_gray_of_the_light` · sonda `diag_o_composto_de_cada_ajuste_nao_neutro` (`PH2D_SONDA_P3=<pasta>`) |
| novos, com placa | `layer_compositor_ajustes_gpu::{gpu_um_ajuste_cheio_sobre_um_pixel_translucido_aplica_se_inteiro, gpu_o_threshold_e_o_byte_da_luma_contra_o_limiar}` (ficheiro irmão; `try_headless_gpu`/`MapProvider` do `layer_compositor_gpu` passaram a `pub(super)`) |
| saiu | `invert_lut_tracks_direct_reference` (→ `invert_is_the_exact_display_negative_at_every_byte`) |
| reescritos para o contrato novo (a mesma afirmação no espaço do tipo) | 12 testes de `adjustments/tests.rs` (Exposure, Levels ×4, Curves, Color Balance ×2, Channel Mixer ×2, Gradient Map, Photo Filter `luma`, Gaussian energia **em luz**); os espelhos `cpu_adjust_op` (`layer_compositor_gpu.rs`) e `painter_cpu` (`fx_stack_adjust_gpu.rs`); 6 chamadas `luz(&mat)…ecra()` dos kernels canónicos no `layer_compositor_gpu.rs` |
| mutação | [`muta_as_camadas_em_ecra.sh`](../../Painter/ferramentas/muta_as_camadas_em_ecra.sh) reescrito: **42/42** (1.ª corrida 38 — doc 45 §8); `MUTA_FILTRO` novo |
| fecho batched | `nextest-impacted` **19 410/19 410** (11 553 fora do alcance; nenhum flake de carga) · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` · clippy `-D warnings --all-targets` · `machete` · `check-standalone-optional` · `check-workflow-packages` · `fmt --all --check` · `architecture_*` **102/102** · placa: `ph2d-render` **50/50** (`layer_compositor*` + `fx_stack_adjust` + `blend_mode_regression` + `layers_no_alloc`), sculpt3d `--ignored tinta_no_produto --skip diag_` **34/34** · memória `o_indice_da_memoria_conta_o_que_aponta` 3/3 · pré-voo dos 23 `muta_*.sh`: ⚠️ só as 3 âncoras do `main` em `docs/Skeleton/ferramentas/muta_a_silhueta_do_contacto.sh` (B1, B5, B14) — não são desta linha |

## §4. Para o integrador

- **`line/PainterWatercolor`** aberta: além do da P1/P2, re-pina qualquer gate dela que fixe um
  composto com uma CAMADA DE AJUSTE sobre zona translúcida (a mistura de volta mudou) ou um Threshold.
- **Assinaturas que mudaram de contrato sem mudar de tipo** (o compilador não avisa): todo kernel
  público de `ph2d_painter_effects::adjustments` passou de luz para codificado. Chamadores no repo: só
  o compositor e os testes acima. Uma linha que chame um deles em luz fica errada EM SILÊNCIO —
  `git grep 'adjustments::apply_\|apply_adjustment'` na árvore combinada.
- ADR `0177` reconta-se.

## §5. O que fica (nomeado) — a P4

- **O espaço do kernel de cada efeito de vizinhança pelo oráculo** (hoje luz, pela porta dos blurs):
  Gaussian, Sharpen, Motion, Chroma, Bloom. Corra um desfoque no GIMP (`gegl:gaussian-blur` por
  `Gimp.DrawableFilter` — não tem `trc`: meça o que ele faz) e no Krita 8 bits, sobre a grelha nossa.
- **Os outros consumidores** — levantamento parcial (agente, 03/10; confira):
  `ph2d-flip-render` entrega as camadas ao `LayerCompositor` da placa (`composite.rs` ~208–365) e não
  terá mistura própria — confirmar e ver os pinos `composite_blend.rs`; o FX raster do Vector
  (`fx_stack_shader.rs` `fx_blend` ~208–224) **mistura em luz premultiplicada** e partilha o
  `blend_modes.wgsl` — produto diferente (efeitos de formas vetoriais): decidir se a lei se aplica, e
  medir; a pré-visualização/Apply do Painter 2D e a peça 3D chamam o compositor (`composite_region`).
  **NÃO cobertos:** o bake de sprites e Wet Paint / Composite (procurar o padrão
  `(app − chão·(1−a))/a` como na aquarela).
- Corrigir o **doc Painter 01 §7** (prescrevia luz para o dab).
- A tabela de 256 do **Levels** erra 2 degraus junto do ponto preto (a função exacta dá 0) — a placa
  liga essa tabela; nomeado, não da lei.
- **O smoke 2D do dono** (antes × depois, camadas translúcidas + um modo + um ajuste) no fim da P4 — a
  P3 já muda a aparência dos ajustes sobre zonas translúcidas.
- Da W1b continuam: o tecto de camadas a `256x`; o fundo semeado por quadro.

## §6. Fecho e binário

Memória (2, cada uma na sua família, contagem subida no índice): `feedback_an_adjustment_changes_the_colour_never_the_coverage`
(padrões de código, 23) · `feedback_an_oracle_parameter_on_a_half_step_tie_measures_float_noise` (oráculo, 23).

`rm -rf target/*/incremental` (`11 G` + `2,8 G`), depois:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke   # 2.ª corrida
Finished `smoke` profile [optimized] target(s) in 0.25s
```

`bash scripts/agent-loop-profile.sh` (20 sessões): paralelismo `1,14` · respostas/sessão `196` ·
`test:check` `2,5` · edições pela `Edit` `37 %` · contexto por passo `354 mil` · início `63 mil`.

Smoke ao dono: nenhum nesta onda — o 2D (antes × depois) é o fecho da P4 (§5).
