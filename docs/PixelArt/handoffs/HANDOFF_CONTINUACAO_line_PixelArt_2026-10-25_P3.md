# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P3 (Camadas) + fecho dos abertos da P2 — 2026-10-25

> Leitor: a próxima LLM que assume a linha numa janela nova. A onda está **fechada e com o smoke do
> dono APROVADO (25/10: «smoke OK»)**; a **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada
> integrado, nada pushado. Lei do módulo: [ADR-0192](../../architecture/decisions/0192-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) (§10 = esta onda)
> · [plano](../02_plano_modulo.md) (§5 P3 com a nota medida) · handoffs anteriores: [P2](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P2.md) · [P1](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P1.md) · [P0b](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md) · [P0](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Merge-base `c3662cd8e4` (= o `main` em 25/10; o `git rebase main` desta janela foi no-op). **53 commits** na linha; os desta onda são `c3644375a2..3df26804aa` (10 commits + este handoff); `git diff --stat e974d8b386..HEAD`: 57 ficheiros, +5296/−89.

## 2. O que esta onda fez

**Passo 1, como o dono pediu: vetores do MVP em JSON antes do código.** `docs/PixelArt/ferramentas/extrai_camadas_mvp.mjs` → `crates/ph2d-pixel-core/tests/fixtures/mvp_layers.json` (compose sorteado 240 pilhas, LUT, cobertura Bayer 0..255, 40 sequências de ops com undo/redo, limites). O MVP é só de índices e tem duas leis que o Aseprite NÃO tem — opacidade como cobertura Bayer 8×8 e `clip` — **recusadas** (ADR §Alternativas). Paridade onde as leis coincidem (`layer_parity.rs`).

**Passo 2: o Trial observado.** `trial_ase_camadas.py` escreve 103 `.ase` NOSSOS pela spec pública (⚠️ o campo velho da contagem de blocos tem de levar o número: com `0xFFFF` o Trial 1.3.18 desalinha); `trial_camadas.py` abre cada um no Trial (Xvfb `:78`, HOME isolado no scratchpad, 400 % = 8 px/célula porque a UI está em escala 2), lê o centro de cada célula, com controlo Normal; `trial_fit_camadas.py`/`trial_fit_grupos.py` ajustam; `trial_fixture_camadas.py` grava `oracle_trial_blend.json`. **Todos os 103 casos EXACTOS** (6 144 células cada; o gate `blend_oracle.rs` compõe pelo nosso documento e só aceita 0 diferenças). Leis: cor = `d + trunc((B(d,s) − d)·mul(Sa, op)/255)`; Dodge/Burn/Divide arredondam a divisão; HSL (W3C, `f64`) trunca; **grupo pass-through, modo e opacidade do grupo NÃO entram** (o diálogo do grupo no Trial só tem o Nome); opacidade do cel = a da camada; Indexed mistura em RGB. ⛔ Lançar o Trial: `env -u WAYLAND_DISPLAY DISPLAY=:78 SDL_VIDEODRIVER=x11 HOME=<scratch> ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage` (o `squashfs-root` está incompleto); dois drivers ao mesmo tempo estragam a leitura (aconteceu: 1.º lote morto, 2.º lote sozinho).

**Núcleo `ph2d-pixel-core`:** `Layer` ganhou `locked`, `reference`, `opacity`, `blend`, `kind` (Image · Group{collapsed}), `parent` (+ `Layer::image`, `Sprite::is_shown`, `Sprite::reserve_id`); `blend.rs` (os 19, ids do `.ase`); `Op::Layers(LayerSet{layers, cels})` com `check_layers` (ids emitidos, pai = grupo, sem ciclo, Background única/primeira/de imagem, nenhum cel órfão nem em grupo); `layers.rs` = o menu Layer como construtores puros (new_layer/new_group/delete/edit/move_to(Place)/duplicate/merge_down/flatten(visible)/via/copy/paste_as_new + `Clip`); `compose.rs` reescrito por árvore (`Purpose::{View, Export}` — a referência não exporta nem funde), `compose_region`/`compose_window`, LUT em Indexed e cópia no Normal opaco; formato **PH2DPIXEL 1.1** (registo Layer: bits 2..5, tags 4 opacidade, 5 modo, 6 pai; 1.0 lê-se com os padrões).

**Vista `ph2d-pixel-render`:** `SpriteImage` recompõe só a janela do traço em curso (∪ a anterior) e, ao largar, só a dele (`settle(stamp)`); gate de bytes iguais à composição cheia. Medido (`mede_compose.rs`, ignorado, `--profile smoke`, load ~2): 512² × 20 camadas nos 19 modos com alfa parcial = 125 ms cheio, **0,12 ms a janela de um pincel**.

**Família `ph2d-app-pixel`:** `camadas.rs` (pedidos, linhas do painel, Solo com restauro, `sane_active`, recusa de pintar em trancada/escondida/grupo com aviso); `Doc::selected_all` (Select All/Deselect — o Trial só faz Copy e *via Copy/Cut* com selecção; a selecção a sério é a P6); atalhos Shift+N, Ctrl+J, Ctrl+Shift+J, Shift+X, Ctrl+A, Ctrl+D, Ctrl+C (o Alt+Shift+N do grupo não chega: `input::key` não recebe Alt); `smoke_camadas.rs` = **cena 7**; `paleta.rs` + `paleta_predefinicoes.rs` (P2: arrastar = `drag_permutation` → um `Remap`; Save as Preset/Default em `PH2D_PALETTES_DIR` · `$XDG_DATA_HOME/ph2d/palettes` · `~/.local/share/ph2d/palettes`, `default.gpl` é a do sprite novo e a do *Load Default Palette*).

**Painel `ph2d-panel-pixel-tools`:** `paint_camadas.rs` (lista com recuo, triângulo, olho, cadeado, Solo, duplo clique = renomear; modo (dropdown + popover depois do scroll) e opacidade 0..255 da activa; o grupo mostra só o nome; botões das acções, Copy/via só com selecção, Paste só com recorte); `seletor.rs` (os 5 seletores: posição↔cor puras, `de_rgb` idempotente e o brilho do Normal map por |n| = 1, imagens 256 px com borda suave e folga de `Spacing::Xs` no quadrado) + arrasto (`seam::arrasta_seletor`: com o botão premido a cor segue `last_pointer` a cada quadro; o seletor reconhece as cores que mandou — `Seletor::mandadas` — e não refaz durante o arrasto) + a cor escolhida à vista (amostra + hex). `PixelRequest` deixou de ser `Copy` (o `LayerRename{String}`, `LoadPreset(String)`).

**Vocabulário `ph2d-editor-core`:** `pixel_panel.rs` (`PixelLayerRow`, `PixelBlend`, `PixelLayerAction`, `PixelSelector`, pedidos novos, `has_clip`/`has_selection`/`presets` no instantâneo); arrasto de linhas de painel para a paleta: `interaction/event.rs`, `dispatch/hierarchy.rs`, `dispatch/pointer_up.rs` (`PanelRowDrop`/`PanelRowFamily`, append-only).

## 3. Smoke do dono — APROVADO em 25/10

Fotos: [`smoke_P3_cena7_2026-10-25.png`](smoke_P3_cena7_2026-10-25.png) (fábrica) · [`smoke_P3_cena7_dono_2026-10-25.png`](smoke_P3_cena7_dono_2026-10-25.png) (`HOME=/home/enio`; `~/.ph2d/layout.txt` copiado antes, sha256 `899b6d19…` igual antes/depois). Cena: `PH2D_PIXEL_SMOKE=7` (19 faixas de modo num grupo recolhido sobre o degradê da cena 4; em baixo 50 %, estrela escondida, trancada, referência, grupo). Seletores: `PH2D_PIXEL_SMOKE=4`, rolar o painel.

Reports do dono, todos curados com gate RED-primeiro: (1) a foto da cena 7 mostrou a Background cortada na banda de baixo (o Patch trocava a caixa do cel) e o 50 % branco sobre branco; (2) borda da roda em escada (96 px, vizinho) e barras que só clicavam; (3) a roda Normal Map escurecia ao arrastar (`de_rgb` tomava o brilho pelo maior canal → realimentação); (4) rodas a encostar nas laterais; (5) arrastar o brilho fazia o anel vibrar (o instantâneo chega um quadro atrasado e o seletor refazia os parâmetros da cor que ele mandara) + pedido da cor escolhida à vista. ⛔ **Lição (5):** um gate pelas CORES mandadas ficou verde com o defeito — o que treme é o ANEL (os parâmetros); o gate certo observa o que o dono vê (`PixelToolsPanelState::selector_params`).

## 4. Superfície de colisão NOVA

Soma-se às de [P2 §5](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P2.md), [P1 §5](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P1.md), [P0b §5](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0b.md), [P0 §3](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md). Contadores ECS/schema/`MODULE_TRUTHS` NÃO mudaram (o blob tem versão própria: PH2DPIXEL 1.0 → 1.1).
- `ph2d-editor-core`: `pixel_panel.rs` (`PixelRequest` sem `Copy`), `interaction/event.rs`, `dispatch/hierarchy.rs`, `dispatch/pointer_up.rs` — outra linha que mexa no arrasto de linhas de painel colide aqui.
- `ph2d-i18n/src/pixel.rs` (chaves `panel.pixel.layer*`, `panel.pixel.blend.*`, `pixel.layer.refuse.*`, seletores, predefinições).
- `ph2d-panel-pixel-tools/Cargo.toml`: dev-dep `ph2d-host` (Cargo.lock).
- `ph2d-pixel-core`: `Layer` com campos novos (literais de `Layer` fora do núcleo partem — use `Layer::image`); `compose_rgba` pass-through.

## 5. Prova de fecho (por nome, tudo verde)

- `ph2d-pixel-core` inteira (motivo: formato gravado 1.1): 64 verdes — `blend_oracle` (103 casos), `layer_laws` (cada acção um passo com inverso byte-igual, Merge Down/Flatten mantêm a imagem, grupo pass-through, referência, formato, a árvore recusa), `layer_parity` (MVP).
- `ph2d-pixel-render`: `the_stroke_window_and_the_settle_give_the_bytes_of_the_full_compose`.
- `ph2d-app-pixel`: `camadas::` 5 (as 19 faixas duas a duas diferentes, cada coisa da banda de baixo muda o que se vê, trancada recusa e avisa, acções = um passo, renomear/modo) · `paleta::` 19 (incl. arrastar, preset, default).
- `ph2d-panel-pixel-tools`: `seletor_leis` 8, `seletor_arrasto` 2 (pelo rato real, com o atraso do instantâneo), `seam` 14, `every_word` 3.
- Binário `target/smoke/ph2d-host-desktop` (`--profile smoke`) refeito depois do último commit. ⚠️ Clippy NÃO corrido (é do fecho).

## 6. ABERTO

- **Enquadramento:** o sprite enquadra-se centrado na JANELA, não na área livre — com o layout do dono (dock direita larga) a cena 7 (190 px de largo) sai cortada à direita (foto `_dono`). P0; curar com o rect do viewport.
- **GPU:** os 19 modos não estão num shader (o plano dizia P3): na CPU por janela o traço custa 0,12 ms; o cheio (commit fora de traço, undo) custa 125 ms em 512² × 20. Medir antes de decidir (§0.0).
- *New Reference Layer from File* → P9 (import). Copy/*via* com selecção a sério → P6. Alt+Shift+N (New Group) não chega à família.
- Opacidade do CEL: lei medida (= a da camada); o campo chega com os cels (P4).
- Recusas registadas: o modo/opacidade de grupo NÃO se implementam (o produto de referência não os tem); o `blend`/`opacity` de um `Layer` de grupo existe no tipo mas o compose ignora e o painel não mostra.
- **Mutação no fecho da linha** (somar às antigas): `check_layers` (cada lei), inverso do `Op::Layers` (ordem do `before.reverse()`), pass-through, `Purpose::Export` sem referência, as 3 divisões arredondadas e o truncar do HSL, a janela/assentar da vista, `de_rgb` idempotente, `mandadas`/arrasto do seletor.
- Fecho da linha pendente: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`; a suíte completa é do integrador.

## 7. Próximo item

**P4 — Frames, cels e timeline** (plano §5; estudo §1 Frame, §3): frames com duração, cels por camada × frame (posição, **opacidade** — a lei já está medida —, z-index, linkados), a timeline do Aseprite (camadas × frames; a lista de camadas desta onda é a coluna da esquerda dela), onion skin, tags (Forward/Reverse/Ping-pong, repeat), tocar. Primeiro passo: extrair os vetores do MVP (`docs/Pixel Art/app/tests/{frames,doc,history_selection,persist}.test.js` e o que tocar frames) para JSON, como `extrai_camadas_mvp.mjs`; depois observar no Trial (o método `trial_ase_camadas.py` escreve `.ase` com vários frames); só então código. Fotografe também com `HOME=/home/enio` (copie antes `~/.ph2d/layout.txt`); a cena de smoke tem de MOSTRAR a diferença de cada coisa (a próxima é a 8).
