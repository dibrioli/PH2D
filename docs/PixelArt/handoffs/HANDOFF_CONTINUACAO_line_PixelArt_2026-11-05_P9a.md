# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P9a (motor de Export/Import) — 2026-11-05

> Leitor: o próximo agente da `line/PixelArt`, que faz a P9b (a tela). A linha CONTINUA; nada da P8d nem da P9 está no `main`.
> Plano aprovado pelo dono: [P9_plano.md](../P9_plano.md). Leia §4 (UI), §6, §6b (recusas medidas) e §8. As decisões do dono estão na §9.

## 1. Identidade

Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `c61f228b05`. Commits da onda: `b87f026c2a` (plano), `9315def31b` (.ase), `ba3d7d6155` (motor), `8960fc8157` (prova e pendências) e o commit deste handoff. P9a não tem tela, logo não tem smoke.

## 2. O que existe agora (só motor)

| Pergunta | Porta |
|---|---|
| Ler e gravar `.ase` | `ph2d_aseprite::{parse_file, write_file}`; tradução `ph2d_pixel_io::ase::{ler, escrever}`; avisos `ase::Aviso` |
| O que sai em cada quadro | `ph2d_pixel_io::render::{Pedido, quadros, composto, ordem, escalado}`, sobre `ph2d_pixel_core::compose_picked` (novo) |
| Export As | `ph2d_pixel_io::saida::{exportar, Formato, Saida, AvisoSaida, png_do_documento, formato_json}` |
| GIF | `gif_cores::reduzir` (corte mediano, decisão do dono) + `ph2d_imageio_gif::encode_indexed` |
| APNG, WebP animados | `ph2d_imageio_apng::encode_animated`, `ph2d_imageio_webp::{encode_animated, decode_frames}` |
| PNG com paleta | `ph2d_imageio_png::{encode_pixel, decode_pixel, PixelPng}` |
| A folha | `ph2d_pixel_io::sheet::{OpcoesFolha, montar, json_da_folha, OpcoesJson}` + `ph2d_sprite_sheet::aseprite::sheet_json` |
| Abrir | `ph2d_pixel_io::abrir::{abrir, abrir_sequencia, importar_folha, Grelha, celulas}` |
| Documento de fora | `Sprite::from_parts(SpriteParts)` em `ph2d-pixel-core::codec`, com as leis de um projeto lido |

## 3. Provas (corridas, não só escritas)

- `oraculo_export.py` (`libresprite -b`, 2,5 s) → `crates/ph2d-pixel-io/tests/fixtures/oracle_export.json`. `--verifica` dá-lhe ficheiros nossos.
- `a_folha_bate_com_o_libresprite`: 30+ casos, dimensões, retângulos, nomes, durações e hash dos píxeis, com a tabela `FORA` justificada.
- `o_composto_bate_com_o_libresprite_em_cada_quadro`, `o_libresprite_le_o_nosso_ase` (hash dos bytes que ele leu: mudou o escritor, corra `--verifica` de novo, nunca regrave só o hash), `o_ase_nosso_volta_igual`.
- Lidos por fora, uma vez, à mão: Pillow e ffprobe contam 5 quadros em GIF, APNG e WebP; o LibreSprite abre GIF, APNG e `.ase` nossos. ⚠️ Isto NÃO é portão automático.
- Aseprite Trial abriu `completo`, `indexado`, `cinza`: [foto](trial_abre_o_nosso_ase_P9a_2026-11-05.png). Script ad hoc na pasta temporária: não ficou no repo.

## 4. Filtro `edges-find*` (M9): a lei mudou

Medido por `ferramentas/trial_edges_alfa.py` (chave `edges_alpha` em `oracle_trial_round3.json`): não são constantes. O «128» era o xadrez da tela sob alfa 0. Agora são matrizes reais (`div` 1, `bias` 0, RGBA), 0 células fora nos 6 casos. Teste `o_alfa_do_edges_find_bate_com_o_trial`, vermelho antes.
- **Muda o que o dono vê, avise no smoke:** Edges com o canal A ligado (o padrão) apaga uma imagem opaca, como no Trial; antes pintava cinza.
- Divisor que cai a 0 agora copia a célula. Só medido com `div` 1: nos outros é extrapolação.
- `kernels.rs` diz «Gerado»: foi editado à mão, com os dados também em `fit_convolucao_r3.py`, sem reexecutar `--rust` (falta `fit_r12.json`).
- Ainda dizem «128 constante», como história: handoffs P8 e `observed` de `round2_gray`/`A2`. A correção está em `edges_alpha.observed`.

## 5. Pendências da P8

Fechadas: FreeType mono, AA3 (nota em `crates/ph2d-pixel-text/fonts/README.md`), mistura parcial (saiu; `texto::mistura` virou `na_cor`), `edges-find*`.
⚠️ **`curve_a11y`/`range_a11y`: NÃO feito.** `sync_a11y` só publica a janela e as abas de família; nenhum widget de painel chega à árvore. Lacuna do app inteiro, fora desta linha: o dono ainda não foi consultado sobre abrir uma onda para ela.

## 6. Curas no leitor antigo (afetam o drag de `.ase`)

`parse` agora compõe sobre `parse_file`. Medido nos 18 `.aseprite` de `~/Downloads/ase-para-testar` (licença desconhecida, fora do repo): 16 saem iguais; `bg-index-3` muda (índice transparente lia o byte 24, não o 28) e `file-tests-props` muda (paleta só em `0x0004` dava rampa de cinza). Bit 1 do cabeçalho agora decide a opacidade de camada. Sem gate de regressão versionado para os 16: foi comparação local. Se precisar, repita com um teste temporário de hash.

## 7. O que a P9b tem de fazer (P9_plano §4, §8)

Crate `ph2d-panel-pixel-export`, em cartões (receita 11); `ph2d-app-pixel` só liga (tecto: leia `the_shell_only_shrinks`). Menu **File** (`hero/menu_tables.rs`, `file_menu.rs`) com um Pixel Sprite em Draw: Export As, Export Sprite Sheet, Repeat Last Export, Import Sprite Sheet, Open. Arrastar `.ase` fica como está. Atalhos `Ctrl+E`, `Ctrl+I`, `Ctrl+Shift+X`, `Ctrl+Alt+Shift+S`: confira em `input_teclas.rs`/`selecao.rs` (`Ctrl+Alt+I` é Sprite Size, `Ctrl+Shift+I` é Inverse).

Em aberto para a P9b:
- `ExportPreset` por sprite + MINOR 2→3 do `PH2DPIXEL` (Repeat). Ainda não existe.
- Medir folha 1024²×256 (M10): acima de 100 ms vai a operário com progresso. `montar` e `pecas` copiam muito; não otimize antes de medir.
- Trocar `Aviso`/`AvisoSaida`/`ErroSaida` por chaves i18n: as `Display` em inglês não são UI (HR-15).
- `saida::png_de` tem dois ramos iguais: simplifique. Abrir sequência numerada: como detetar os irmãos é da P9b.
- Import de slices: só aviso. `.ase` de tilemap vira imagem: nunca conferido na tela contra o Trial.
- `Formato::Png` com sprite Indexed escalado passa por `png_do_documento`, sem teste próprio.
- Godot (`o_godot_recorta_igual`), `um_importador_so_com_os_campos_do_formato`, mutações (§6) ainda NÃO escritas: entram antes do fecho da linha.
- Cena **20** (`smoke.rs:32`) com fotos. Smoke é do dono.

## 8. Avisos

- `cd` de um comando persiste; a árvore de testes em `scratchpad` não é do repo.
- O runner `cargo-test-narrow.sh` esconde a mensagem do assert: `cargo test -p <crate> --test <nome>` por `ph2d-run.sh` a mostra.
- `sed`/`replace` sem `assert` é recusado pelo guarda; os scripts desta onda têm o `assert` de contagem.
