# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W16 → `main` (2026-10-12)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> **Integra-se JUNTO com a W12+W13, a W14 e a W15** ([W12+W13](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md) ·
> [W14](HANDOFF_INTEGRACAO_line_MiroClone_W14_2026-10-11.md) · [W15](HANDOFF_INTEGRACAO_line_MiroClone_W15_2026-10-11.md),
> que seguem válidos para o que é delas): a linha nunca foi ao `main` desde `8f20c60de`. Briefing da onda:
> [W16](HANDOFF_CONTINUACAO_W16_O_QUE_FICOU_DAS_IMAGENS_2026-10-11.md). Oráculo Excalidraw 0.18.1:
> [`excalidraw_oracle/`](../ferramentas/excalidraw_oracle/README.md) §W16 (`saidas/w16/`).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | `9e88bd0c31` (+ este handoff) |
| base (merge-base) | `8f20c60de`. O `main` andou **21 commits** (linha Vector e memória) e a linha **NÃO está rebaseada**: o integrador rebaseia |
| commits da W16 | 12: `git log --oneline 383637a100..HEAD` (`383637a100` = fim da W15) |
| diff da W16 em `crates/`+`shells/` | `git diff --shortstat 383637a100..HEAD -- crates shells`: 68 ficheiros, +3 881 / −335 |
| smoke | cena 17 (`PH2D_BOARD_SMOKE=17`): **ainda sem smoke do dono**. A W15 teve OK dele (11/10). Foto da cena: `ferramentas/soltar_wayland/quadro17_depois.png` — **a fotografar** (não existia ao escrever) |

Os 6 abertos herdados da W14/W15 num bloco só (§0.10):

1. **Recorte pelo gesto + espelho pela barra.** Oráculo corrido: `ferramentas/excalidraw_oracle/sonda_imagem_w16.mjs`,
   32 sondas, 0 erros, duas corridas idênticas byte a byte (`saidas/w16/w16_sondas.json`). Lei medida: entra por
   duplo-clique / Enter / «Crop image»; 4 pegas de canto (±6 px) + 4 de meio (±2 px, sem desenho); a pega move o LADO da
   janela e a imagem fica parada; proporção livre, Shift = natural; o recorte guarda-se em px naturais NÃO espelhados;
   pára nas bordas; mínimo 10×10 do mundo; cobrir tudo = `null`; sai por Enter / Escape / clique fora (fica); um passo de
   desfazer por arrasto; fora da janela a imagem esbate a α = 27/255. Espelho: `scale` [-1,1]/[1,-1], x/y/w/h ficam,
   ângulo 2π−θ (no Excalidraw só no menu de contexto). Código: `ph2d-board-edit/src/crop.rs`
   (`Editor::{begin_crop, end_crop, cropping, can_crop, know_files}`; o ponteiro passa por ele antes de tudo),
   `ph2d-board-geom::image_to_box` (a lei píxel natural → caixa num sítio só; `level_to_box` usa-a), `Overlay::crop`
   (pegas em «L», `StrokeToken::Heavy`), `RenderCache::cropping`, `Editor::flip_images` (à volta do centro da selecção,
   ângulo −θ), barra `Item::Flip(0|1)` e `Item::Crop` **no FIM** de `items()` (nenhum id se move), teclas Shift+H /
   Shift+V (antes das ferramentas) e Enter/Escape do recorte.
   ⚠️ **Divergências deliberadas do oráculo:** (a) o deslizar dentro da janela segue o dedo 1:1 (no Excalidraw anda 2× o
   ponteiro — lido como defeito deles); (b) o GIF TOCA (o canvas do Excalidraw mostra o 1.º quadro, limite do Chromium;
   o dono pediu GIF animado); (c) o espelho tem botões na barra (o dono pediu). Achado latente desde a W14: o
   duplo-clique numa imagem abria um texto VAZIO por cima dela — agora recorta.
2. **«Send to board…» com a selecção inteira e a pose.** A linha clicada DENTRO da selecção (`gizmo.iter_selected`)
   leva-a toda; fora, só a linha; numa leva, UM passo de desfazer. A SPRITE vai com a POSE da cena como pose do
   ELEMENTO (escala por eixo = `ppm·Sprite.size/natural`, ângulo −θ, espelho canonizado, o `flip_x/flip_y` próprio da
   sprite incluído): píxeis inteiros, editável no quadro. Uma sprite INCLINADA (skew da Inspector, que o quadro não
   exprime) redesenha-se à densidade natural (bilinear pesado pelo alfa, ≤ 8192, a escala na pose). Novos:
   `ph2d-board-edit` `Pose` + `Editor::insert_posed_images`; editor-core `board_images::insert_posed`; app-board
   `send_to_board::{Sent, deliver(hero, board, Vec<Sent>) -> Result<(BoardId, Option<Toast>), Toast>, targets,
   posed_sprite}`; i18n `board.send.left_out`. A fase da shell só compõe. Teste que atravessa a costura da shell:
   `shells/desktop/tests/it/o_send_to_board_leva_a_seleccao_com_a_pose.rs` (lê o FONTE: o `App::new()` nasce headless,
   como o gate do Colorize) + os de comportamento na família.
3. **Export SVG sem rasterizar à toa.** `ExportTarget` (Png · Pdf · Svg) substitui o `vector: bool` do `ExportLook` e do
   `paint_page`; o SVG grava `DrawOp::Embed` (`VectorScene::record_embedded_image`: só grava, a placa não vê) — nenhuma
   imagem descodificada; o PDF continua com os píxeis. MEDIDO (`measure_svg_export_images`, `--release`, mesmo processo,
   intercalado, mínimo de 5, `loadavg` 100): fotografia 4000×3000 + SVG de 2 000 círculos — **161,7 ms → 1,4 ms**; o SVG
   escrito é o MESMO (5 507 028 B).
4. **GIF animado + colar do Dolphin.** `ph2d-board-render/src/anim.rs`: um tocador por GIF à vista, um fio a descodificar
   os quadros em sequência para um canal de 1 lugar (memória de 2 quadros, qualquer duração), reduzido por metades ao
   nível pedido e sempre até caber no atlas, preto e branco aplicado, recomeça no fim; quadro ≤ 10 ms dura 100 ms (lei
   dos navegadores); o ritmo conta-se do fim do anterior (`t + d`); o export não toca (SVG leva os bytes do GIF; PNG/PDF
   o 1.º quadro). `app-board images::frames` (`image` `GifDecoder::into_frames`; `image` com a feature `gif`) e o 1.º
   quadro do decode sem juntar os outros.
   **Colar do Dolphin, MEDIDO** (`ferramentas/colar_dolphin/corre.sh`, KWin virtual, `saida_2026-10-11.txt`): o Dolphin
   oferece SÓ `text/uri-list` (+ `x-kde4-urilist`, portal, source-id), CRLF; o `arboard` sem `wayland-data-control` lia
   pelo X11, que o KWin só serve a uma janela X11 activa — o colar não via o que outro app copiava (nem texto); com a
   feature, `file_list()` devolve os caminhos com o `\r` do CRLF (o `arboard 3.6.1` parte só no `\n`). A sessão real do
   dono expõe `ext_data_control_manager_v1` (`wayland-info`). ⇒ `arboard` com `wayland-data-control` (shell +
   `ph2d-app-host`); o Ctrl+V num quadro lê ficheiros → imagem → texto (ficheiros antes do texto: a leitura de texto do
   Wayland devolve a própria lista); `paste_files` tira o `\r`. A leitura passou a ter PRAZO: `ph2d-app-host/src/clipboard.rs`
   (novo) corre-a num fio com 4 s (o `LONG_TIMEOUT` do caminho X11 do `arboard`) e diz `clipboard.no_answer` na tela — o
   colar do quadro e o de texto do resto do app (`forwarding.rs`) passam por ela (o `read_to_end` do data-control não
   tem prazo: um app dono travado congelava o PH2D).
5. **Memória do Flip grande.** MEDIDO antes (`ferramentas/memoria_flip/corre.sh`, `saida_2026-10-11.txt`; um processo por
   variante): 4 camadas a 8192² punham **+2 982 MiB** na placa (4·N + 32 B/px), 8 camadas **FALHAVAM** (orçamento do
   compositor 1 GiB = 4 fatias de 256 MiB), o scratch do ecrã ficava preso +1 446 MiB e o quadro seguinte realocava
   (+13–18 ms). ⇒ `pass_raster`: a leitura vai por LADRILHOS (`tile_for`: o tamanho dos scratch do ECRÃ —
   `FlipCompose::scratch_size`, novo — quando as fatias cabem no orçamento; senão o maior quadrado cuja leitura inteira
   4·N + 32 B/px cabe). DEPOIS (`saida_2026-10-11_depois_ladrilhos.txt`): 8 × 8192² sai (190 ms; antes falhava),
   1 × 8192² 85 ms (antes 180), 4 × 8192² 129 ms (antes 241); o alocador fica +0 MiB sobre a base depois da leitura; o
   ecrã não realoca. (A secção `PICO_DECOMPOSTO` da sonda é a composição de uma vez — o «antes» —, não o caminho novo.)
6. **Cena de smoke 17 «Crop, flip & GIF»** (`ph2d-app-board/src/smoke_w16.rs`; roteador `max_level` 17): frame Crop
   (fotografia inteira + cópia recortada), frame Flip (original + H + V), frame GIF (bola de 8 quadros a 120 ms), 6
   lições. Gate `the_w16_scene_does_what_its_lessons_say` pelo RATO real (duplo-clique + arrastar o canto).

## 2. Foundational / partilhado tocado — todo aditivo, e o que um merge pode partir

| onde | o quê | o que pode partir |
|---|---|---|
| `ph2d-vector` | `DrawOp::Embed` + `EmbedOp` + `record_embedded_image` | ⚠️ `DrawOp` é um enum público: um `match` exaustivo fora da linha deixa de compilar. grep feito no diff: só `record_tests`, `svg.rs`, `pdf.rs` (todos tratados). Uma linha do `main` que acrescente um matcher precisa do braço novo |
| `ph2d-board-render` | `ExportLook.target` substitui `.vector` (5 literais, um em editor-core `board_team_show.rs`); `RenderCache.cropping`; `Images::{set_animator, set_clock}`; `install_animator`; `anim.rs`; `image_pyramid.rs` (irmão novo, tecto de LOC) | quem construir um `ExportLook` com `vector:` falha a compilar |
| `ph2d-board-export` | `paint_page(…, target: ExportTarget, …)` em vez de `vector: bool`; `ExportTarget` re-exportado | idem |
| `ph2d-board-geom` | `image_to_box` | — |
| `ph2d-board-edit` | `Pose`, `insert_posed_images`, `flip_images`, `crop.rs`, `Overlay.crop`; `is_busy` inclui o arrasto do recorte, `finish_gesture` fecha-o; **depende agora de `ph2d-vector`** | `Cargo.toml` da crate |
| `ph2d-flip-render` | `FlipCompose::scratch_size` | — |
| `ph2d-editor-core` | `board_bar` `Item::Flip`/`Item::Crop` no FIM; `board_keys`; `board_images::insert_posed`; `documents::leave_board` fecha o recorte; `paint.rs` `know_files` + `cropping` | ids novos no fim, nenhum se move |
| `ph2d-app-host` | `clipboard.rs` (novo), dep `arboard` | — |
| `ph2d-app-board` | `send_to_board::{Sent, deliver, targets, posed_sprite}`, `images::frames`, `smoke_w16.rs` | família do quadro |
| `ph2d-i18n` | `board.style.flip_h/flip_v/crop`, `board.send.left_out`, `clipboard.no_answer` (em `chrome.rs`), `board.smoke.w16.*` | chaves novas |
| `shells/desktop` | `src` +109 −84 = **+25 linhas** (`fase_send_to_board.rs`, `keyboard_board.rs`, `forwarding.rs`); `tests/it` +1 ficheiro (80 linhas) | a catraca `the_shell_only_shrinks` conta a shell: reconte depois do rebase |
| `Cargo.lock` | `wl-clipboard-rs 0.9.4`, `os_pipe`, `petgraph`, `tree_magic_mini` (`cargo deny` ok) | regenerar no rebase |

**Contratos congelados (CLAUDE.md §6): nenhum tocado.** **Schema e formato do quadro: NÃO mudaram** (`Image.crop`/`flip`
já existiam; `FORMAT_VERSION` 13 e `PROTOCOL` 6 da W15 intocados).

## 3. O portão (1× sobre o diff acumulado, 2 passagens)

| passagem | resultado |
|---|---|
| 1.ª | reprovou em `no_magic_numeric` (literais dos ícones novos), um import `tr_with`, clippy `chunks_exact_mut` num teste de medição, `fmt` — curados |
| 2.ª `nextest-impacted` | **18 627 / 18 628**; o 1 ✗ = `workspace_src_files_under_loc_cap` (`image.rs` 702 > 700), curado MOVENDO a pirâmide para `image_pyramid.rs` (verde depois) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` · clippy como o `ship.sh` (`-D warnings`) · `fmt` · `typos` · `machete` · `deny` | ✓ |
| `check-standalone-optional` · `check-workflow-packages` · `file_loc_caps` | ✓ |
| GPU `#[ignore]` do Flip | 2/2 ✓ |
| censos-da-árvore-combinada | 117/117 + 12/12 ✓ |

**Auditoria de 2 lentes** (correcção + costura/ciclo de vida); achados CURADOS, cada um com teste: o recorte ABORTAVA numa
imagem com lado < 10 do mundo (`f64::clamp` com min>max) e num recorte de fora a sair da imagem; Escape/Enter a meio de
um arrasto deixavam uma mudança fora do desfazer; o recorte passava de aba (editor único para todos os quadros) e morria
com dois quadros com o mesmo número de ficheiros (`know_files` por contagem → por chaves); o esbatido nunca aparecia
(estava dentro do corte da caixa); o botão Crop era clique morto com duas imagens; trocar de ferramenta não saía; o
`flip_x/flip_y` da sprite perdia-se; a sprite inclinada amostrava 5× abaixo; GIF maior que o atlas; colar sem prazo.
Achado pelo gate da cena 17: o arrasto do recorte pelo rato real não chegava ao editor (`Editor::is_busy` não o contava).

**Mutação:** 17 mutações nas leis W16, 15 sangraram à 1.ª; as 2 sobreviventes (o mínimo = `min(10, o lado de agora)`; o
ritmo `t + d` do GIF) foram tapadas com testes e re-mutadas (sangram) ⇒ **17/17**.

## 4. Abertos (nomeados)

- (a) Na **apresentação** (`board_team_show.rs`, export look Png) o GIF mostra o 1.º quadro — decisão de produto por tomar.
- (b) «Send to board…»: uma entidade sem nome aparece como «Left out: , X»; entidades que sumiram entre o pedido e a
  escolha saltam-se em silêncio.
- (c) O Shift do recorte toma a proporção em px naturais (o oráculo só mediu kx = ky).
- (d) Um fio de tocador por GIF à vista, sem tecto de quantidade.
- (e) Os `<text>` de um SVG continuam a não se desenhar (recusa medida da feature `text` do usvg, de pé).
- Falta o **smoke do dono** da cena 17 e a **foto** `quadro17_depois.png`.

## 5. O integrador

1. `git -C Worktrees/line-MiroClone log --oneline 8f20c60de..HEAD` — W12+W13+W14+W15+W16 + handoffs. Rebase sobre o
   `main` (+21), `Cargo.lock` regenerado, conflitos pelos estágios (DIRETRIZ §1.5.5).
2. Reconte ADR (0182–0184 da linha) contra o máximo do `main`; `PROJECT_SCHEMA` não é desta linha.
3. Árvore combinada: `cargo check --workspace --all-targets` + os portões do §3; reveja os `match` sobre `DrawOp` e os
   literais de `ExportLook`/`paint_page` que o `main` tenha trazido.
4. **Entrada nova do §5.1 do CLAUDE.md** (≤ 700 bytes; só o link e a frase): troque o «Último: [handoff W15]…» por
   `[handoff W16](docs/MiroClone/handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md)` (com a W15, a W14 e a
   W12+W13 ao lado); `Smoke PH2D_BOARD_SMOKE=1..17`; na frase do módulo, acrescente «recorte pelo gesto, espelho, GIF
   animado» à lista das imagens (junto de «moldura, preto e branco, SVG nítido e “Send to board…”»).

## 6. Perfil do agente (`bash scripts/agent-loop-profile.sh`)

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
  ✗ paralelismo de ferramenta              1.10/passo   alvo: >= 1,5  (7% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                163   alvo: <= 800
  ✗ cargo test : cargo check                473 : 153   alvo: <= 1,0  razao 3.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  40%   alvo: >= 80%
  ✗ contexto relido por passo (media)         388 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## 7. Smoke do dono

### Binário do smoke compilado

(a 2.ª compilação: colar aqui)

Passos:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=17 ./target/smoke/ph2d-host-desktop`
   — abre o quadro «Crop, flip & GIF».
2. **Recortar:** duplo-clique na fotografia da esquerda. Aparecem cantos em «L» e a foto inteira fica clarinha por fora.
   Arraste um canto para dentro; arraste dentro da janela para deslizar a foto; **Enter** termina; **Ctrl+Z** desfaz.
3. **Espelhar:** clique numa foto — na barra dela estão os dois botões de espelhar (ou Shift+H / Shift+V).
4. **GIF:** a bola do GIF anda.
5. **Colar do Dolphin:** no Dolphin copie 2 imagens (Ctrl+C), volte ao quadro, Ctrl+V — entram lado a lado.
6. **Da cena para o quadro:** na aba da cena, seleccione várias coisas na Hierarchy (Ctrl+clique), botão direito numa
   delas ▸ **Send to board…** — vão todas, a sprite rodada/espelhada como na cena.
7. **Exportar o quadro em SVG:** é rápido e, no navegador, a bola anda.

**Como saber que deu errado:** o duplo-clique abre uma caixa de texto; a bola está parada; o Ctrl+V do Dolphin não traz
nada ou traz um texto «file://…»; só uma das coisas seleccionadas vai para o quadro.
