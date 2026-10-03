# HANDOFF de CONTINUAÇÃO — `line/sculpt3d` · as camadas juntam-se em tons de ecrã: P0–P2 fechadas (2026-10-03)

> **Para quem é:** o agente que assume a linha para a **P3** (e a P4) — pelo bloco do
> [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).
> **SUPERSEDE** o [da W1b](HANDOFF_CONTINUACAO_line_sculpt3d_CAMADAS_W1b_2026-10-03.md) (que continua
> válido para o que é da W1b). Não é handoff de integração: o de integração em vigor é o
> [A_INCLINACAO](HANDOFF_INTEGRACAO_line_sculpt3d_A_INCLINACAO_2026-10-02.md). O mecanismo e as
> medições estão no [doc Painter 45 §8](../../Painter/45_plano_as_camadas_juntam_se_em_tons_de_ecra.md)
> e no [ADR-0177](../../architecture/decisions/0177-as-camadas-do-painter-juntam-se-em-tons-de-ecra.md).

## §0. Estado

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` · merge-base `1ad60a1ce` (= `main` a 03/10) |
| commits desta janela | `de30f6c7d` (P0) · `9f20c775b` (P1 CPU) · `053d71da5` (P1 placa) · `406ee3dca` (P2) · `87af729bb` (gates da fronteira + arnês) · `14f5bf285` (docs + memória) · `63920c9f2` (os 2 gates que a mutação pediu) · `a91a6c220` (lint do fecho) · este handoff |
| decisão do dono (03/10) | **opção 1**: as camadas juntam-se em tons de ecrã (Photoshop/Procreate/Krita 8 bits) |
| ADR | **`0177`** — o próximo livre CONTADO: o `main` acaba em `0175` e a `line/3DModeling` já tem um `0176` por integrar. ⚠️ Reconta-se na integração |
| `SCULPT_DOC_VERSION` · `PROJECT_SCHEMA` · `a_forma_da_pilha_gravada_e_pinada` | intocados (`7` · — · `83`/`3 590`) |
| contratos §6 | intocados (`Tool=12`, `PanelEvent=4`, `NodeOp`) |
| a shell (`shells/desktop/src`) | **0** linhas |

**Foundational tocado:** `ph2d-tool-painter` (o compositor de referência + a aquarela),
`ph2d-render` (o `layer_composite.wgsl` e a tabela do decode). Nenhuma crate nova.

## §1. O que a janela deixou

- **A lei** (P1): `decode = b/255`, `encode = round` no compositor de referência
  (`compositor/mod.rs`) e no gémeo (`layer_composite.wgsl`). Saíram as três tabelas sRGB da CPU e os
  seus 2 gates; o binding 4 da placa FICA com o conteúdo `b/255` (`decode_lut`) — ver §4.
- **A fronteira** (P1): tudo o que é definido em luz recebe `luz(acc)` e devolve `ecrã(resultado)`;
  a mistura de volta (opacidade/modo do ajuste) corre em ecrã. CPU: o braço do ajuste em
  `compose.rs` (`em_luz`/`em_tons_de_ecra`). Placa: `apply_adjustment_op`, o 1.º passe do blur, o
  chroma, o bright-pass do bloom, a luma do S/H, e os dois combines. Os intermédios do grafo de
  vizinhança guardam o acumulador CODIFICADO.
- **A aquarela** (`watercolor_render.rs`): o que imitava o compositor (base sobre o chão, franja de
  AA, des-premultiplicar, `gamut_alpha`) passou a ecrã; a óptica Beer–Lambert fica em luz.
- **Os oráculos** (`docs/Painter/ferramentas/oraculo_camadas_gimp/`): GIMP 3.2.6 (84 corridas) e
  Krita 6.0.4 (56), deterministas, entradas partilhadas (`entradas.py`), armadilhas no README.

## §2. Medido

| gate | antes | depois |
|---|---|---|
| (a) compositor × GIMP «perceptual», 10 modos | `73` | `≤1` |
| (b) peça 3D, cena 52: traço numa camada nova × o traço na base | `0,286` | **`0,000`** |
| (b) gémeo 2D | `42` | um dab `≤1`; um traço `≤2` raro |
| (c) CPU↔placa, pilha rica `8x` | `23`/`188 424` | `11`–`12` (nunca 2) |
| Krita 8 bits × compositor, 22 modos | — | `≤4` em `≤93`/`4 896` canais |
| ajuste neutro, 18 tipos | — | `0` bytes |

## §3. Gates

| | |
|---|---|
| novos, sem placa | `compositor::oraculo_gimp_tests` (4: o controlo «linear» · a lei «perceptual» · o Krita nos 22 modos · as divergências do GIMP são as fórmulas nomeadas) · `compositor::ajustes_na_fronteira_tests` (2: neutro = no-op ao bit · `+1 EV` vê a luz) · `tool::paint` `o_traco_numa_camada_nova_e_o_traco_na_base` · `the_byte_round_trip_is_the_identity` · `ph2d-render` `decode_lut_is_the_cpu_decode` |
| novos, com placa | `tinta_no_produto_tests::painel::o_traco_numa_camada_nova_e_o_traco_na_base` (era a sonda `diag_…`) |
| saíram | `decode_lut_is_bit_exact_with_srgb_to_linear_byte` · `encode_via_threshold_matches_linear_to_srgb_byte` · `decode_then_encode_round_trips_every_byte` · `the_srgb_byte_round_trip_is_the_identity` (→ `the_byte_round_trip_is_the_identity`) · `srgb_lut_matches_cpu_transfer` (→ `decode_lut_is_the_cpu_decode`) |
| re-pinados (o número ao lado) | `opacity_half_blends_toward_bottom` e `group_opacity_attenuates_the_stack` (`~188` → `~128`) · `smooth_edges_off_is_the_pre_aa_render_byte_for_byte` (`0x9744233f9f852066` → `0x03c97ba0b481a619`: 405 bytes / 135 px, todos por 1) · o espelho CPU do `tests/it/layer_compositor_gpu.rs` (a lei nova) · `FRACCAO_A_UM_DEGRAU` FICA (só o comentário) |
| com placa | `ph2d-render` `layer_compositor_gpu` + `blend_mode_regression` + `layers_no_alloc` **37/37** · sculpt3d `--ignored tinta_no_produto --skip diag_` **34/34** |
| mutação | [`muta_as_camadas_em_ecra.sh`](../../Painter/ferramentas/muta_as_camadas_em_ecra.sh): **17/17**. A 1.ª corrida deu 15/17: C6 (des-premultiplicar sobre o chão em luz) vivia porque o gate só tinha chão BRANCO (luz = ecrã ali) — o gate corre agora também sobre `(150, 120, 90)`; G2 (o encode final da placa trunca) vivia sob a paridade ±1 — novo `gpu_a_metade_em_tons_de_ecra_e_128_ao_byte`. As duas sangram (re-aplicadas). Pré-voo `MUTA_SO_ANCORAS=1`: 17/17 |
| fecho batched | `nextest-impacted` **19 406/19 406** · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` · clippy `-D warnings --all-targets` (a 1.ª corrida apanhou um `mut` a mais e dois `chunks_exact` nos testes novos — `a91a6c220`) · `machete` · `check-standalone-optional` · `check-workflow-packages` · `architecture_*` **102/102** · `fmt --check` · pré-voo dos 22 `muta_*.sh`: ⚠️ **3 âncoras partidas em `docs/Skeleton/ferramentas/muta_a_silhueta_do_contacto.sh`** (B1, B5, B14 casam 0 vezes) — o arnês e as crates dele (`ph2d-vec-boolean`, `ph2d-skeleton-live`, `ph2d-vec-render`, `ph2d-app-vec`) não foram tocados por esta linha: vem do `main`, é da linha do Vector/Esqueleto |

## §4. Para o integrador

- **`line/PainterWatercolor`** está aberta: qualquer gate dela que fixe um COMPOSTO (ou a aparência de
  uma camada de aquarela/Wet Paint numa camada transparente) re-pina com esta lei; o
  `watercolor_render.rs` desta linha mudou o des-premultiplicar (§1).
- **ADR `0177`** reconta-se (a `line/3DModeling` tem `0176`).
- O binding 4 do `layer_composite.wgsl` mudou de NOME (`srgb_lut` → `decode_lut`) e de conteúdo; os
  campos `srgb_lut_buffer`/`SRGB_LUT_LEN`/`build_srgb_lut` do `ph2d-render` passaram a
  `decode_lut_buffer`/`DECODE_LUT_LEN`/`build_decode_lut`. Uma linha que os nomeie colide.
- Premissas do plano que a medição derrubou: as 4 do doc 45 §8 (seam por dab, a tabela fica, a
  paridade não fecha ao bit, a aquarela assumia luz).

## §5. O que fica (nomeado) — a próxima janela

- **P3 — os ajustes, fronteira por fronteira** (doc 45 §6/§8): o contrato do módulo de ajustes
  (`ph2d-painter-effects::adjustments::apply_adjustment_windowed`, cujo ÚNICO chamador é o braço do
  ajuste do compositor, + o shader + os espelhos de teste) passa a receber o acumulador CODIFICADO; os
  de ecrã (Curves, Levels, Posterize, Threshold, Invert, ColorBalance, SelectiveColor, ChannelMixer,
  Noise, Color Lookup, Black & White, Gradient Map) largam a ida-e-volta, os de luz (HSB, Exposure,
  Vibrance, Brightness/Contrast, Photo Filter) convertem dentro deles. A régua já existe:
  `cada_ajuste_neutro_e_um_no_op_ao_bit` + as paridades com placa por tipo; meça também um NÃO
  neutro de ecrã (ex.: Curves) CPU↔placa e contra o GIMP «perceptual» (Curves/Levels/Invert/
  Posterize/Threshold o GIMP tem). A sonda `diag_o_ajuste_neutro_por_tipo` dá a tabela.
- **P4** — o espaço do kernel de cada efeito de vizinhança pelo oráculo (hoje: luz, pela fronteira;
  o GIMP borra em luz, o Photoshop no espaço do documento — corra os dois oráculos com um desfoque);
  os outros consumidores da mesma lei: `ph2d-flip-render` (`composite_blend`), o FX raster do Vector
  (`BLEND_MODES_WGSL` em `fx_stack`), o bake de sprites, a pré-visualização/Apply do Painter 2D (já
  passa pelo compositor — confirmar), Wet Paint / Composite; corrigir o doc Painter 01 §7 e fechar
  a tabela ⛔ Recusas MEDIDAS.
- **O smoke 2D do dono** (antes × depois de um documento com camadas translúcidas e um modo) vem no
  FIM da P4 — a aparência 2D ainda muda com a P3/P4.
- Da W1b continuam: o tecto de camadas a `256x`; o fundo semeado por quadro.

## §6. Fecho, smoke e binário

Depois de `rm -rf target/*/incremental` (`25 G` + `3,0 G`), 1.ª build `29,6 s`:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

`bash scripts/agent-loop-profile.sh` (20 sessões): paralelismo `1,14` · respostas/sessão `196` ·
`test:check` `2,4` · edições pela `Edit` `37 %` · contexto por passo `365 mil` · início `63 mil`.

Smoke ao dono (o report de 03/10): ver a mensagem final da janela — cena `52`, o mesmo traço preto
na `Layer 1` e numa `Layer 2` nova, as bordas iguais.
