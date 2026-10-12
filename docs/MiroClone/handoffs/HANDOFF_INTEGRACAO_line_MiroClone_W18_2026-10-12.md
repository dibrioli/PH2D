# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W18 → `main` (2026-10-12)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> **Integra-se JUNTO com a W17, a W16, a W15, a W14 e a W12+W13** ([W17](HANDOFF_INTEGRACAO_line_MiroClone_W17_2026-10-12.md) ·
> [W16](HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md) · [W15](HANDOFF_INTEGRACAO_line_MiroClone_W15_2026-10-11.md) ·
> [W14](HANDOFF_INTEGRACAO_line_MiroClone_W14_2026-10-11.md) · [W12+W13](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md),
> que seguem válidos para o que é delas). A W18 tem duas partes: **A** os abertos das imagens (A1..A5) e **B** o VÍDEO do
> YouTube no quadro. Briefing: [W18](HANDOFF_CONTINUACAO_W18_ABERTOS_E_VIDEO_YOUTUBE_2026-10-12.md). Oráculo Excalidraw 0.18.1:
> [`excalidraw_oracle/`](../ferramentas/excalidraw_oracle/README.md) §W18 (`sonda_w18.mjs`, 45+29 sondas, 0 falhas, sem rede
> e com o YouTube permitido). Motores medidos: [`youtube_motor/`](../ferramentas/youtube_motor/README.md). ADR:
> [0185](../../architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | ver `git log -1` (este handoff é o último commit) |
| base (merge-base) | `9ea7821383` = `main` de 12/10 |
| commits da W18 (do mais velho) | `14ee652020` A1 · `2bb9714f48` A4 · `f009badb03` oráculo+motores · `36853ade74` A2 · `e5762ab9ed` ADR-0185 · `0df8847989` base do vídeo · `073a6f4be2` A3 · `a74e8b480e` vídeo no quadro · `c615f46133` links na exportação · `ea5d5c3f71` links largados · `e03afc320c` cena 19 · `a38e9c55c4` ao vivo · `ac80943c42` portão · `bdd1b42661` auditoria+mutação · + este handoff |
| diff da W18 em `crates/`+`shells/` | `git diff --shortstat 91b1321543 HEAD -- crates shells`: 88 ficheiros, +6345 / −321. `shells/desktop/src` líquido **+9** (`input_drop.rs`, a chamada dos links largados); `the_shell_only_shrinks` verde |
| smoke | cena 19 (`PH2D_BOARD_SMOKE=19`, `smoke_w18.rs`; roteador `max_level: 19`). Foto: `ferramentas/soltar_wayland/quadro19_depois.png` |

### Parte A — os abertos das imagens

1. **A1 — o GIF anda na APRESENTAÇÃO.** `ph2d-board-render` ganha `ExportTarget::Screen` (o visual de exportação num ecrã
   vivo; detalhe do `Png`; `ExportTarget::live`); `board_team_show` usa-o. ⚠️ Escolha derivada do pedido do dono na W16 —
   **diga-lho** (era a decisão de produto aberta (a) da W17).
2. **A2 — sem fio por GIF.** Cada quadro do GIF é um `anim::Step` no pool de descodificação de imagens (um em voo por
   tocador). Régua `ph2d-board-render/src/anim_measure_tests.rs` (`#[ignore]`, `--release`, mesmo processo, intercalado,
   mínimo de 3, `loadavg` 1,7): **200 GIFs a 25 fps — fios 210→10, CPU 0,82→0,44 núcleo, pintura 0,37→0,20 ms, todos a
   24,6 fps**; RSS +47→+71 MB (arenas do alocador do pool, ~0,1 MB/GIF). **Nenhum tecto escrito** (§0.0: o pool é metade da
   máquina). Para mover quadros entre fios o GIF passou a ser lido pela crate `gif` 0.14 com a composição da `image`
   (`ph2d-app-board/src/gif_frames.rs`), com gate **pixel a pixel** contra a `image` (5 combinações de descarte + o GIF do
   oráculo da W16). Achado: a `image` lia GIFs **sem tecto** (65535² = 17 GB) — agora o tecto da porta de imagens
   (`ph2d_asset::loader::MAX_DIMENSION`, tornado `pub`). `Animate` passa a `+ Send`.
3. **A3 — o `<text>` de um SVG desenha-se.** Passagem em `ph2d-imageio-svg` `raster::parse` troca cada `<text>` por
   contornos (parley+fontique+skrifa). Família pela lei do Chromium (genérica/em falta → Liberation Sans/Serif, medido).
   Gate contra o Chromium: 16 casos `t01..t16` (letra ≤ 0,02 u; caixa de tinta ≤ 1 px a 1×/2×), vermelho primeiro, **7
   mutações mortas**. Custo 117 µs (t04) / 19 ms (5800 glifos), em cache por ficheiro; SVG sem texto não paga nada.
   `raster::text_gaps` → aviso do app nomeia as fontes em falta (`board.images.svg_fonts`) e as construções não suportadas
   (`board.images.svg_texts`, agora com `{what}`); `raster::parse_size` para o ingest (nenhuma fonte aberta no fio da
   janela). `dropped_texts` removido. Doc: [09_texto_dos_svg.md](../09_texto_dos_svg.md) (limites no §6).
4. **A4 — a imagem truncada mostra o que se lê.** `ph2d-app-board/src/salvage.rs` (instalado por
   `ph2d_board_render::install_salvager`): PNG linha a linha; JPEG pelo `zune-jpeg` tolerante (alfa≠255 = não lido →
   transparente). Fica **ESTRAGADA** (aviso + marca vermelha num canto no ecrã, `image_box.rs::broken_mark`; a exportação
   leva as linhas lidas). Achado do bloco: um JPEG truncado entrava INTEIRO com o fundo pintado e sem aviso — agora um JPEG
   sem EOI (`FFD9`) só é inteiro se todos os pixels foram lidos. Oráculo `w17`: PNG 173 linhas (nós 172..174), JPEG 142
   (nós 158: o Chromium retém 16 para o sobre-amostragem). Lição 1 da cena 18 reescrita para o comportamento novo.
5. **A5** — renomear na Hierarchy já recusa vazio (`ph2d-panel-hierarchy/src/event.rs:386`): nada a fazer.

### Parte B — o vídeo do YouTube

O elemento `ElementKind::Embed` (no FIM; formato inalterado, `Field::Embed`, `PROTOCOL` 6→7), a lei em `ph2d-board-embed`
(gate sobre as 24 sondas do oráculo), o mundo em `ph2d-board-video` (miniatura HTTPS; o tocador no navegador da PESSOA
por CDP num tubo, sem `unsafe`), o desenho (`ph2d-board-render::video`), o editor (`board_video.rs`: colar, o play, o
rato e as teclas), o app (`ph2d-app-board::video`), o `.excalidraw`, o export (`/URI`, `<a href>`), o largar e o ao
vivo. Régua do critério de morte: 29,99 q/s distintos, clique→quadro 9–24 ms. **O mecanismo inteiro:**
[`10_video_do_youtube.md`](../10_video_do_youtube.md).

## 2. Foundational / partilhado tocado — e o que um merge pode partir

| onde | o quê | o que pode partir |
|---|---|---|
| `ph2d-board-model` | variante `ElementKind::Embed` (no fim); `Field::Embed`, `Field::ALL` 39 | ⚠️ qualquer `match` exaustivo sobre `ElementKind` em código de OUTRA linha deixa de compilar; contagens de `Field::ALL` |
| `ph2d-board-sync` | `PROTOCOL` 7 | gates/fixtures com `6` |
| `ph2d-board-render` | `ExportTarget::Screen` (variante nova); `Animate: Send`; `Videos`/`video.rs`; `image_box.rs` (cortado); `install_salvager` | `match` sobre `ExportTarget` |
| `ph2d-board-export` | `pdf_blur.rs` (cortado do `pdf.rs`: 716→683, tecto 700); `write_with_uris` | `file_loc_caps`; chamadores de `pdf::write`/`svg::write` |
| `ph2d-editor-core` | `board_video.rs`; `documents.rs` (`video_press`/`video_ask`/`videos_mut`/`warn`/`take_video_ask`; `activate` zera o vídeo); `board_keys`; cadeia de `board_nav`; `board_bar` (`Item::Video`, `TOOLBAR_TAIL`); `board_bar_look` (ícone) | `Item`/`TOOLBAR_TAIL` e `items()` se outra linha mexeu na barra |
| `ph2d-asset` | `loader::MAX_DIMENSION` agora `pub` | — |
| `ph2d-wayland-dnd` | `UriList.links` | literais de `UriList` |
| `ph2d-imageio-svg` | módulo de texto; deps `parley`/`skrifa`/`simplecss`/`svgtypes` | `Cargo.toml`; `Cargo.lock` |
| `ph2d-i18n` | chaves novas `board.video.*`, `board.tool.video`, `board.images.svg_fonts`, `board.smoke.w18.*`; **mudaram de texto** `board.images.svg_texts` (`{what}`) e `board.smoke.w17.l1`; `dropped_texts` fora | censos i18n; chaves repetidas |
| crates NOVAS | `ph2d-board-embed`, `ph2d-board-video` (membros do workspace) | `Cargo.toml` raiz (lista de membros) |
| `shells/desktop/src` | `input_drop.rs` +9 líquido | a catraca `the_shell_only_shrinks`: reconte depois do rebase |
| `deny.toml` · `Cargo.lock` | banimentos de motores/YouTube; deps de rede/TLS e texto | **regenerar o `Cargo.lock`**; `deny` à árvore combinada |

**Contratos congelados (CLAUDE.md §6): nenhum tocado.** `FORMAT_VERSION` do quadro fica 13; `PROTOCOL` 6→7; **ADR novo
0185 (número provisório — recontar ao integrar)**; `PROJECT_SCHEMA` não é desta linha.

## 3. O portão (1×, BASE=`9ea7821383`, `loadavg` 21→38)

| passo | resultado |
|---|---|
| `fmt` | ✓ (depois das correcções) |
| `typos` como o `ship.sh` corre (projecto inteiro) | ✓ — o `typos` cru sobre a lista de ficheiros acusa 6 palavras PT em handoffs velhos W14/W15/W17: falsos positivos, a config trata-os |
| `machete` · `cargo deny --all-features check` | ✓ |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ |
| clippy como o `ship.sh` | ✗ depois ✓: `insert_embed` com 8 args (link e vídeo num par → 7), `as_chunks` no salvador, `?` no GIF; re-corrido em todas as crates tocadas + `ph2d-host-desktop` ✓ |
| `nextest-impacted` | **18 678 / 18 679**: o 1 vermelho era `file_loc_caps` (`pdf.rs` 716>700), curado movendo o desfoque (683) e re-corrido ✓ |
| `scripts/censos-da-arvore-combinada.sh` | 117/117 |
| depois do portão, crates tocadas re-corridas verdes | app-board 68 · editor-core 2201 · board-edit 218 · board-export 19 · board-video 3 (+2 `#[ignore]`) · board-render 56+ · board-embed 4 · board-model 124 · board-excalidraw 39 · board-sync 57 · i18n 24 · wayland-dnd 17 · imageio-svg 30 |

**NÃO corridos:** `cargo audit`, `check-standalone-optional.sh`, `check-workflow-packages.sh`, índices de ADR/doc.

**Mutação** (agente `mutacao`, 16 mutações, cada controlo verde primeiro, árvore restaurada e `touch`-ada): **15/16
sangraram** à 1.ª passagem — M1 comprimento do id · M2 segundos · M3 `live_allowed` tapado · M4 `covered()` · M5/M6 salvador
· M7 descarte `Background` do GIF · M8 reenvio do leitor · M9 `ExportTarget::live` · M10 falhado→escolher · M11 desactivar
fora · M12 colar→vídeo · M13 `embeddable` aceite · M14 `UriList.links` · M15 enquadrar ao activar. **M16 SOBREVIVEU**
(`READER_IDS` a `1<<40`): o antigo `min(i32::MAX)` mascarava-a e repetia ids ⇒ curada: os ids dão a volta DENTRO da faixa de
32 bits, teste puro `reader_ids_stay_32_bit_and_distinct_across_the_wrap`, e a faixa invertida já não compila
(`arithmetic_overflow`). Mais as 7 mutações próprias da A3.

**Auditoria de 2 lentes** (correcção do ciclo de vida do vídeo; costura). Curado: trocar de quadro mantinha o id do vídeo
activo (os ids são por quadro) ⇒ `Documents::activate` zera-o; um vídeo activado mas não vivo arrancava o navegador (som até
à pausa) ⇒ arranca só se vivo.

Achado no fim (a máquina limpa): um fecho abrupto fecha o navegador (o tubo) mas deixava a pasta do perfil em
`/tmp/ph2d-cdp-*` — `browser::sweep_stale_profiles` varre as de pids mortos ao lançar (teste novo).

## 4. Abertos (nomeados)

- (a) **Windows:** o tubo CDP é lançado por `sh` — lá ainda não há leitor (cai para abrir o navegador); macOS tem `sh`, não testado.
- (b) A **roda do rato** não é encaminhada ao leitor; o ecrã inteiro do próprio leitor fica dentro do rectângulo (sem ecrã inteiro verdadeiro).
- (c) Na **apresentação** o vídeo mostra miniatura+play (não toca lá).
- (d) O estado da reprodução **não** sincroniza ao vivo (por desenho, ADR).
- (e) Sem rede: marcador + play + aviso; sem título em texto no canvas.
- (f) Arrastar um link do navegador não mostra pré-visualização ao pairar (só o largar).
- (g) Limites do texto SVG em `09_texto_dos_svg.md` §6 (`textPath`, `rotate`, vertical, `text-decoration`… reportados no aviso).
- (h) Rascunho de medição em `~/Apps/w18_motores` (Servo nightly, `gst-plugins-good` extraído; ~190 MB, fora do repo — pode apagar-se).
- (i) **Número do ADR 0185 provisório.**
- (j) Aberto (f) da W17: o gate M5 da costura da shell continua a ler o fonte.

Fecharam aqui: W17 (a) GIF na apresentação, (b) fio por GIF, (c) `<text>` do SVG, (d) imagem truncada.

## 5. O integrador

1. `git log --oneline 9ea7821383..HEAD` — W12+W13…W18 + handoffs. Se o `main` andou, rebase; `Cargo.lock` regenerado;
   conflitos pelos estágios (DIRETRIZ §1.5.5).
2. **Recontar o ADR** (0185 é provisório; renumerar também o link neste handoff, no briefing, no README do `youtube_motor/` e
   nos comentários que o citem: `git grep 0185`). `PROJECT_SCHEMA` não é desta linha.
3. Árvore combinada: `cargo check --workspace --all-targets` + portões do §3; **reveja cada `match` exaustivo sobre
   `ElementKind`/`ExportTarget`/`Field::ALL`** e os literais de `UriList` que o `main` traga; `paint_page(`/`png::render(`
   (assinatura da W17) continuam a valer.
4. Recolha a memória do §1.10 do checkout primário.
5. **Entrada do §5.1 do CLAUDE.md** (≤ 700 bytes; só o link e a frase): link para
   `docs/MiroClone/handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W18_2026-10-12.md` (com a W17..W12+W13 ao lado); acrescente
   «vídeo do YouTube pelo navegador da pessoa» à frase; `Smoke PH2D_BOARD_SMOKE=1..19`.

## 6. Perfil do agente (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                166   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                577 : 145   alvo: <= 1,0  razao 4.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  41%   alvo: >= 80%  (1186 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         416 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## 7. Smoke do dono

⚠️ **Smoke do dono (12/10): defeitos no vídeo** — o ecrã inteiro do tocador deforma, o zoom às vezes pausa, o ícone do canal deixa o vídeo preto; e a decisão «nossos controles + Premium». Seguem na [continuação W19](HANDOFF_CONTINUACAO_W19_VIDEO_DEFEITOS_E_CONTROLES_2026-10-12.md).

### Binário do smoke compilado

`rm -rf target/*/incremental` (35 GB) e depois, a 2.ª de
`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`:
```
▸ linha line_miroclone · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

Foto da cena: `ferramentas/soltar_wayland/quadro19_depois.png` (`no_app.sh`, KWin virtual: a miniatura chegou da rede, o «Hello»/«board» do SVG desenhado, a cortada com as linhas de cima; as lições cabem). A lição 1 da cena 18 foi ENCURTADA depois da foto dela (cortava em «says so») — `quadro18_depois.png` refeita, cabe.

Passos:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=19 ./target/smoke/ph2d-host-desktop`
   — abre o quadro «Video and picture fixes»: frames com o vídeo do YouTube, o GIF, o SVG com texto, a foto cortada, e
   cinco notas amarelas (as lições).
2. **Vídeo:** clique no play vermelho. Se o vídeo é pequeno o quadro aproxima-se dele; o vídeo toca DENTRO do quadro, com
   som (Big Buck Bunny). Clique fora: ele pausa.
3. **Vídeo tapado:** arraste uma nota amarela por cima do vídeo — ele pausa, o play volta e a faixa de cima diz porquê.
4. **Vídeo novo:** copie um link do YouTube no navegador, clique no quadro e prima Ctrl+V — aparece um vídeo novo; ou use o
   botão «Video» no fim da barra da esquerda.
5. **GIF:** carregue em Present (em baixo à direita): o GIF mexe-se no slide.
6. **Texto do SVG:** «Hello» e «board» aparecem desenhados; a faixa nomeia a fonte em falta «No Such Font PH2D».
7. **Foto cortada:** vê-se a parte de cima, com uma marca vermelha no canto; a faixa diz que uma imagem está estragada.
8. **PDF:** exporte em PDF e clique no vídeo dentro do PDF — abre o YouTube.

**Como saber que deu errado:** o vídeo fica uma caixa cinzenta sem miniatura (sem rede) · o play não faz nada · o GIF fica
parado em Present · o SVG não mostra palavras · a foto cortada é uma caixa cinzenta riscada.
