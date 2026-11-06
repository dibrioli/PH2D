# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, P10a (motor dos tilemaps) — 2026-11-06

> Leitor: a próxima janela da linha, que faz a P10b (tela). A linha continua; não integra nem faz push.
> Plano: [P10_plano.md](../P10_plano.md). Medições: [P10_oraculos.md](../P10_oraculos.md).

## Estado

- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `bbfefaf393`, 7 commits fora do `main`.
- `286ce1ea95` e abaixo: conta-gotas e zoom, com `Smoke OK` do dono, à espera de ordem para integrar.
- `4952bcf0ee`..`b8060437f8`: P10a. **Sem smoke: não há nada na tela ainda.** Só o último commit passa nos tectos de LOC.
- Decisões do dono: §9 do plano. Abordagem: **carimbo à mão + gabarito de 47**. Nada de objeto de jogo (isso é do projeto Tiling), vários terrenos sem transição, iso e hex por último, saída PNG + JSON.

## O que existe e não se refaz

| Peça | Onde |
|---|---|
| `Tileset`, `TileGrid`, `TileRef`, `Op::Tiles`, `expand`, `new_tilemap_layer` | `ph2d-pixel-core/src/tiles.rs` |
| Formato `PH2DPIXEL` MINOR 4 | `codec.rs`; fixtura real MINOR 3 em `tests/fixtures/pixel_minor3_bbfefaf393.bin` |
| `.ase` ler e gravar com tilemap | `ph2d-aseprite/src/{file,write}.rs`, `ph2d-pixel-io/src/ase.rs` |
| Manual, Auto, Stack, dedupe | `ph2d-pixel-tiles/src/desenho.rs` |
| Draw Tiles (`carimba`) e grelha do mundo | `ph2d-pixel-tiles/src/mapa.rs` |
| Gabarito (`pinta`, `novo_terreno`) | `ph2d-pixel-tiles/src/terreno.rs` |
| Convert To | `ph2d-pixel-tiles/src/converte.rs` |

**A lei:** `Cel.tiles` é a verdade e `Cel.data` deriva. `Patch` em camada de tilemap recusa. O teste `o_derivado_e_o_mapa_expandido` guarda isto.

## Provas

- Conferido à parte pela janela principal, ao receber a onda: `os_tres_modos` 1, `o_terreno_bate` 2, `o_derivado` 1, `ph2d-pixel-core tiles::` 12 (1 ignorado), `a_tilemap_returns` 1, verdes em `b8060437f8`.
- Relatados pelo agente da onda, sem reconferir: 31 mutantes sangraram (2 sobreviventes resolvidos); `architecture_workspace_file_loc_cap`, `the_shell_only_shrinks`, `no_magic_numeric`, `no_literal_color`; crates inteiras de `ph2d-pixel-core`, `-io`, `-tiles`; clippy verde **antes** do último teste, 3 corridas.
- Medido por código real do `main`: lê o MINOR 4 com o composto igual; regravar mantém o desenho e perde só os tiles.
- ⚠️ **Não correram:** suíte completa, `nextest-impacted`, testes inteiros de `ph2d-app-pixel`, `ph2d-i18n`, `ph2d-editor-core`, gesto real do rato, Text, Trim, Crop, M8, M3 contra `ph2d-grid`.
- ⚠️ `cargo fmt` reformatou código antigo das crates tocadas: há ruído nos diffs.

## Abrir a P10b por aqui

1. **Velocidade, decidido pela janela principal:** M6 deu 1 693 ms por gesto em 256×256 células × 64 quadros, todas cópia do tile editado (o pior caso), e 426 ms em 1024×1024 × 1 quadro. Passa dos 100 ms. Aplicar a proposta 1 (`blit` por linha, com a imagem de cada combinação de flips pronta) e **medir de novo**. Se não chegar, a 2 (quadros em paralelo). A 3 (só o que está à vista) quebra a lei: não fazer sem voltar ao dono. ⚠️ Não medido: preview, GPU.
2. **Folga de `ph2d-app-pixel`: 7 linhas** (11 408 de 11 415). Mover antes de ligar a tela; nunca subir o número. Painel em crate nova `ph2d-panel-pixel-tiles`.
3. **Recusas:** §5.1 do plano. Os mudos (Rotate Canvas, Flip Canvas, Sprite Size, Canvas Size e Crop fora de múltiplo, arrastar cel para outra camada) têm de ficar cinzentos ou dizer porquê. Trim fora de múltiplo usa uma mensagem que não explica o motivo.
4. **`Space+Tab` e `Space+1/2/3`:** `Space` já é mover a vista (`compose.rs:179`). O `Tab` da timeline ainda está em `input_teclas.rs:179`, porque `828051653b` da `line/GameEngineFuncional` não está no `main`. ⚠️ Risco de conflito nesse ficheiro: anote no handoff de integração.
5. **Cena de smoke:** a próxima é a **21** (`max_level` 20→21), com camada de tiles já desenhada; depois a 22 na P10c.

## Por confirmar no smoke do dono (não há oráculo)

- Apagar: em Manual edita o tile; em Stack vira o tile vazio. A sonda do Pixelorama viu «nada muda», e pode ter sido defeito da sonda.
- Traço por várias células: um tile usado 2×, com as duas células no mesmo traço, dá tile novo à primeira e edita o original na segunda.
- Crop e Canvas Size movem o mapa e não cortam: células fora do canvas reaparecem se ele crescer.
- O tileset fica no sprite depois de Flatten, Delete e Convert To Image.

## Ainda da P10c

Tela do gabarito, iso e hex (M3 contra `ph2d-grid`; `.ase` sai como imagem, com aviso), Export Tileset e mapa em JSON, M8 no Godot, `--split-grid` e `--trim-by-grid`.
