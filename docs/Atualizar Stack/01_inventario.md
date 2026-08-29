# 01 — O inventário medido

> ⚠️ **Fotografia de 2026-08-29.** A fonte viva é `bash scripts/stack-audit.sh`.
> Rode-o antes de citar qualquer número daqui.

## §1 — Como isto foi contado (e o que ficou de fora, de propósito)

O `stack-audit.sh` varre **só os membros do workspace** — `crates/*`, `tools/*`, `shells/desktop`,
`tests/spike` — que é exatamente o que `cargo clippy --workspace` compila e o que o nosso
`-D warnings` policia.

⛔ **Fica de fora, e não é buraco:** **53 dependências** em **6 árvores de referência vendorizadas**
que não são código nosso e não entram no build:

| árvore | o que é | dependências |
|---|---|---|
| `crates/ph2d-audio-ml/vendor/deep_filter` | runner do DeepFilterNet ([ADR-0123](../architecture/decisions/0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md)) — `exclude` explícito do workspace | 12 |
| `docs/Pixel Art/Resources/editors-native/rx` | editor de pixel de referência | 19 |
| `docs/Tilling/.../vendor/bevy_ecs_ldtk` | referência de tilemap | 11 |
| `docs/Tilling/.../vendor/rs-tiled` (+ `macros`, `fuzz`) | referência de tilemap | 11 |

As duas pastas de `docs/` estão no `.gitignore` por decisão do Enio (§5.0 do `CLAUDE.md`).

> ⚠️ **Isto muda a conta em quase metade.** Uma leitura ingênua da árvore devolve **43** saltos
> maiores; **14** deles são do editor `rx`. *Uma dependência que ninguém compila não é dívida.*

## §2 — O retrato

| classe | quantas |
|---|---|
| **MAIOR** — salto que quebra API | **28** |
| **menor** — compatível, sobe com `cargo update` | **31** |
| **igual** — já no topo | **7** |
| **tetos** — o mais novo é inalcançável | **7** |

## §3 — ⚠️ Os 7 TETOS — «o mais recente possível» ≠ «o mais recente»

Esta é a seção que decide o plano. Sete dependências **não podem** ir ao topo, porque outra
dependência as segura. Forçar não dá erro de resolução — dá **duas cópias**, e aí um tipo
atravessa a fronteira entre elas e vira erro de tipo (ou, pior, um `Device` que a outra recusa).

| crate | temos | topo | **dá para usar** | quem segura |
|---|---|---|---|---|
| **wgpu** | 28 | 30.0.1 | **29.0.4** | `vello 0.10.0` pede `^29.0.3` |
| **skrifa** | 0.40 | 0.46.2 | **0.44.0** | `vello 0.10`, `parley 0.11.1`, `usvg 0.48.1` pedem `^0.44` |
| **accesskit** | 0.24 | 0.25.0 | **0.24.1** | `parley 0.11.1` pede `^0.24.0` |
| **pollster** | 0.4 | 1.0.1 | **0.4.0** | `rfd 0.17.2` pede `^0.4` |
| **core-graphics** | 0.23 | 0.25.0 | **0.23.2** | `winit 0.30.13` pede `^0.23.1` |
| **miniz_oxide** | 0.8 | 0.9.1 | *duas cópias* | `ctt`, `exr`, `png` pedem `^0.8`; `flate2` pede `^0.9` |
| **thiserror** | 2 | 2.0.20 | *duas cópias* | `psd 0.3.5` pede `^1` |

⚠️ **«Duas cópias» não é sempre defeito.** O veredito é **por crate**, e a pergunta é uma só:
*esse tipo atravessa a NOSSA superfície?*

- `miniz_oxide` e `thiserror` são **folhas** — compressão e um `derive`. Duas cópias custam bytes no
  binário e mais nada. **Aceitar.**
- `wgpu`, `vello`, `glam`, `skrifa` **atravessam** — um `wgpu::Device` de uma cópia não serve à outra.
  Duas cópias ali é o defeito, não a dívida.

⛔ **`naga` não aparece na tabela e também tem teto:** ele é `[só-dev]` nas nossas duas crates, mas
o `wgpu` traz o dele. **Tem de casar com o wgpu** — 29.0.4, nunca 30.

⛔ **`ndarray` tem o oitavo teto, e o script não o vê:** o `deep_filter` vendorizado pede `^0.15` e
está **fora** do workspace, então a varredura não o alcança. Subir o nosso `ndarray` para 0.17
carrega duas cópias **e** parte a fronteira por onde passamos arrays para ele. **Fica em 0.15.**

## §4 — Os 28 saltos maiores, com o tamanho da obra

`ficheiros` = quantos tocam a API dessa crate. `ocorr.` = quantas menções. Medido em 2026-08-29.

| crate | de → para | ficheiros | ocorr. | quem declara | bloco |
|---|---|---:|---:|---|---|
| **wgpu** | 28 → **29.0.4** | 194 | 4 125 | 9 crates + `shells/desktop` | **C** |
| **bevy_ecs** | 0.18 → 0.19.1 | 185 | 475 | `ph2d-ecs`, `-field-ecs`, `-physics-ecs`, `-render`, `-script`, `-timeline`, `shells/desktop`, `tests/spike` | **D** |
| **rapier2d** | 0.28 → 0.35.3 | 47 | 145 | `ph2d-physics` | **E** |
| **glam** | 0.30 → 0.33.6 | 32 | 52 | 8 crates | **F** |
| **rfd** | 0.15 → 0.17.2 | 20 | 41 | `shells/desktop` | **F** |
| **pollster** | 0.4 → *fica 0.4* | 14 | 28 | 5 crates | **F** ⛔ teto |
| **mlua** | 0.10 → 0.12.0 | 11 | 75 | `ph2d-script`, `tests/spike` | **F** |
| **vello** | 0.8 → **0.10.0** | 11 | 69 | `ph2d-render`, `ph2d-vector` | **C** |
| **naga** | 28 → **29.0.4** | 9 | 52 | `ph2d-gpu-cook`, `ph2d-render` | **C** |
| **accesskit** | 0.24 → *fica 0.24.1* | 7 | 21 | `ph2d-a11y` | **C** ⛔ teto |
| **linesweeper** | 0.3 → 0.4.0 | 4 | 7 | `ph2d-vec-boolean` | **F** |
| **ctt** | 0.4.0 → 0.5.0 | 4 | 28 | `tools/asset-cooker` | **F** |
| **miniz_oxide** | 0.8 → *fica 0.8* | 4 | 4 | `ph2d-aseprite`, `-quadextract`, `shells/desktop` | **F** ⛔ teto |
| **cpal** | 0.15 → 0.18.2 | 3 | 17 | `shells/desktop` | **F** |
| **fontique** | 0.6 → **0.11.1** | 3 | 4 | `ph2d-system-fonts`, `shells/desktop` | **C** |
| **taffy** | 0.12 → 0.14.0 | 3 | 5 | `ph2d-vec-layout` | **F** |
| **ndarray** | 0.15 → *fica 0.15* | 3 | 4 | `ph2d-audio-ml` | **F** ⛔ teto |
| **criterion** | 0.7 → 0.8.2 | 3 | 3 | 3 crates *(só-dev)* | **F** |
| **zip** | 2 → 8.6.0 | 3 | 3 | `ph2d-imageio-ora` | **F** |
| **skrifa** | 0.40 → **0.44.0** | 2 | 7 | `ph2d-vector-font` | **C** ⛔ teto |
| **parley** | 0.6 → **0.11.1** | 2 | 10 | `ph2d-text` | **C** |
| **symphonia** | 0.5 → 0.6.1 | 2 | 18 | `ph2d-audio-decode` | **F** |
| **toml** | 0.9 → 1.1.4 | 2 | 3 | 2 crates *(só-dev)* | **F** |
| **wasmtime** | 47 → 48.0.1 | 2 | 5 | `tests/spike` | **F** |
| **usvg** | 0.43 → 0.48.1 | 1 | 5 | `ph2d-imageio-svg` | **F** |
| **jxl-oxide** | 0.10 → 0.12.6 | 1 | 3 | `ph2d-imageio-jxl` | **F** |
| **roxmltree** | 0.20 → 0.21.1 | 1 | 3 | `ph2d-imageio-ora` | **F** |
| **core-graphics** | 0.23 → *fica 0.23.2* | 1 | 2 | `shells/desktop` | **F** ⛔ teto |

⚠️ **`bevy_ecs` mede 185 ficheiros com 475 menções — e a superfície real é MAIOR que isso.** Quase
todo consumidor faz `use bevy_ecs::prelude::*` e depois usa nomes nus (`Query`, `Commands`, `Res`).
*Contar o caminho qualificado subestima o alcance de um prelúdio.*

## §5 — As 31 compatíveis (sobem com `cargo update`, sem tocar em código)

`arboard 3→3.6.1` · `blake3 →1.8.7` · `bumpalo →3.20.3` · `bytemuck →1.25.2` · `clap →4.6.6` ·
`crossbeam-queue →0.3.13` · `dhat →0.3.3` · `exr →1.74.2` · `flate2 →1.1.10` · `flecs_ecs →0.2.2` ·
`gilrs →0.11.2` · `half →2.7.1` · `image →0.25.10` · `insta →1.48.0` · `kurbo 0.13→0.13.1` ·
`notify →8.2.0` · `ogg →0.9.2` · `png →0.18.1` · `postcard →1.1.3` · `rayon →1.12.0` ·
`rect_packer →0.2.1` · `serde →1.0.229` · `serde_json →1.0.151` · `serde_json5 →0.2.1` ·
`smallvec →1.15.2` · `tempfile →3.27.0` · `thiserror →2.0.20` · `tiff →0.11.3` ·
`vorbis_rs →0.5.6` · `walkdir →2.5.0` · `winit 0.30→0.30.13`

**Já no topo (7):** `fidget`, `ktx2`, `libavif-sys`, `libm`, `proptest`, `psd`, `realfft`.

## §6 — O Rust não limita nada

| | pede |
|---|---|
| `bevy_ecs 0.19.1` | **1.95.0** ← o mais exigente de todos |
| `wasmtime 48.0.1` | 1.95.0 |
| `vello 0.10`, `parley 0.11`, `fontique 0.11`, `zip 8`, `mlua 0.12` | 1.88 |
| `wgpu 29.0.4`, `naga 29.0.4` | 1.87 |
| `rapier2d 0.35.3`, `criterion 0.8.2` | 1.86 |
| todo o resto | ≤ 1.85 |

Estamos em **1.95** e vamos para **1.98**. **Nenhuma dependência conflita** — e nem se fôssemos ficar
onde estamos, porque o piso (`bevy_ecs`) é exatamente o nosso pin de hoje.

## §7 — ⛔ Duas afirmações que a pesquisa REFUTOU (não as reconstrua)

**1. «O rapier migrou de nalgebra para glam na 0.32.»** — o `CHANGELOG.md` do `master` diz isso, e
**nenhuma versão publicada faz**. Conferido no índice, versão a versão:

```
0.28 nalgebra=^0.34 parry2d=^0.23    …    0.32 nalgebra=^0.34 parry2d=^0.26
0.33 nalgebra=^0.35 parry2d=^0.28    …    0.35.3 nalgebra=^0.35 parry2d=^0.30.2
```

Não há `glam` em nenhuma. Aquela entrada é da série **não lançada** no `master`. Um plano construído
sobre ela reescreveria 47 ficheiros para uma API que não existe. *O changelog de um `master` descreve
o futuro; o índice descreve o que se pode instalar.*

**2. «O disco está com a metadata do btrfs em ENOSPC iminente.»** — o `btrfs-health.sh` dá vermelho,
e no disco novo isso é **falso**. A regra (linha 116) testa `metadata livre < 1 GiB` **sozinha**; o
perigo real — o que causou o `mold` em SIGBUS ([memória](../../project-memory/project_disk_full_corrupts_objects_mold_sigbus.md))
— exige as duas condições **juntas**: metadata apertada **e** não-alocado esgotado. Aqui o
não-alocado é **1 857 GiB**, e o próprio script o marca ✓ duas linhas acima. Numa árvore recém-criada
o btrfs aloca **um** pedaço de 1 GiB de metadata e usa 0,10 dele. Ver tarefa **T2**.
