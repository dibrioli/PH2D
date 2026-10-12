# Motor do vídeo do YouTube no quadro — as medições (W18, 2026-10-12)

> Pergunta: *como um app Rust/Vello/winit mostra o tocador OFICIAL do YouTube (o embed
> `youtube-nocookie.com/embed/<id>`) DENTRO do canvas — roda, segue o pan/zoom, fica por baixo das notas — no
> KDE Wayland do dono?* Cada linha da tabela é uma corrida desta pasta, sobre o mesmo vídeo (`aqz-KE-bpKQ`,
> Big Buck Bunny), a 560×315, `loadavg` 8–22 (CPU com folga). Decisão: o ADR da W18.

## A tabela

| caminho | o vídeo anda? | quadros/s entregues | CPU | memória | o que custa | prova |
|---|---|---|---|---|---|---|
| (i) **janela-filha** nativa (`wry`/WebKitGTK por cima do retângulo) | — | — | — | — | o `wry` só é filha em **X11**; no Wayland um cliente não posiciona janelas; XWayland **recusado** na W14 | por construção |
| WebKitGTK 4.1 (instalado) **fora do ecrã** (`GtkOffscreenWindow`) com aceleração | não | 0 (`Failed to create GBM buffer`) | — | 181 MB | — | `saidas/webkitgtk.json` |
| WebKitGTK 4.1 fora do ecrã **sem** aceleração | **não** (o tocador abre, fica no círculo em 0:00) | 60 (o círculo) | 2 s/25 s | 192 MB | faltam `matroskademux`/`qtdemux`/`vpx` (o `gst-plugins-good` não está instalado) | `saidas/webkitgtk_semgpu.json` |
| WebKitGTK + `gst-plugins-good` 1.28.8 extraído ao lado (`GST_PLUGIN_PATH`), com e sem a caixa de areia | **não** (igual) | 59–60 | 2,5 s/25 s | 191 MB | Linux só | `saidas/webkit_com_good*/` |
| **Servo** 0.7.0 `servoshell` sem ecrã + os codecs ao lado | **sim** | — (fotos por WebDriver a cada 2 s) | 5 s/25 s | 657 MB | GStreamer do sistema; sem MSE ⇒ o YouTube cai no MP4 de 360p | `saidas/servo/` |
| **Servo 0.7.0 EMBUTIDO** (`servo_spike/`: `SoftwareRenderingContext` + `read_to_image`) | **sim** | **29,9** distintos | **0,09 núcleo** | 646 MB | build release frio **172 s** (606 crates, sem `wgpu`/`winit`); binário 155 MB; o fim abrupto cai no `mozjs` (encerramento ordenado obrigatório) | `saidas/servo_embutido/` |
| **Chrome do sistema sem ecrã + CDP screencast** (= a janela virtual do navegador da pessoa; também o proxy do CEF fora do ecrã) | **sim** (alta qualidade) | **29,97** | **0,15 núcleo** | 1 336 MB (a árvore do navegador) | **zero** a instalar/embarcar; só a família Chrome (Chrome, Chromium, Edge, Brave, Vivaldi, Opera) | `saidas/chromium_google-chrome-stable.json` |
| o mesmo, **som e clique** | o som sai pelo sistema (fluxo «Google Chrome» activo no PipeWire) | clique → 1.º quadro em **7–28 ms** | — | — | é preciso tirar o `--mute-audio` | `saidas/chromium_som_clique.json` |
| CEF (`cef` 154, Apache/MIT; a lib `cef` 152 do repo BSD-3) | (o motor do Chrome: acima) | — | — | — | 116 MB baixados / 364 MB instalados; o `accelerated_osr` pede `wgpu` **30** (o nosso é 29, preso pelo `vello`) ⇒ só o caminho por CPU | `cargo info cef` |
| (iii) tirar o fluxo do vídeo (`yt-dlp` + descodificar) | — | — | — | — | ⛔ viola os termos do YouTube | — |

O Excalidraw (o oráculo) usa um `<iframe>` vivo por cima do canvas: [`../excalidraw_oracle/README.md`](../excalidraw_oracle/README.md) §W18.

## O futuro do Servo (pesquisa de 12/10)

Financiamento real (LF Europe, Igalia, Sovereign Tech Fund 10/2025, NLnet «Servo Multimedia» 06/2026,
~7,8 mil USD/mês de doações), 400–560 PRs aceites por mês, crate `servo` no crates.io desde 04/2026 com uma
linha LTS. MAS: a API de embutir **quebra todo mês** («not a 1.0»), **não há MSE** (o PR #40423 está parado; a
verba da NLnet começou em 06/2026) — por isso o YouTube toca em 360p — e o vídeo pede GStreamer também no
Windows/macOS. ⇒ plano B, para quando houver MSE.

## Decisão do dono (12/10)

*«Sim, janela virtual»*: o PH2D usa o navegador da família Chrome que JÁ está no computador, invisível, e
projecta os quadros no quadro; sem ele, a miniatura com «Abrir no navegador».

## Correr

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 120 node docs/MiroClone/ferramentas/youtube_motor/sonda_chromium_fora_do_ecra.mjs /usr/bin/google-chrome-stable aqz-KE-bpKQ 20 <saida>
PH2D_MEM_MAX=8G bash scripts/ph2d-run.sh timeout 120 node docs/MiroClone/ferramentas/youtube_motor/sonda_chromium_som_clique.mjs /usr/bin/google-chrome-stable aqz-KE-bpKQ <saida>
SEM_GPU=1 python3 docs/MiroClone/ferramentas/youtube_motor/sonda_webkitgtk_fora_do_ecra.py aqz-KE-bpKQ 25 <saida>
# Servo: ~/Apps/w18_motores/servo (nightly 0.7.0) e os codecs em ~/Apps/w18_motores/gst (fora do sistema)
GST_PLUGIN_PATH=~/Apps/w18_motores/gst/usr/lib/gstreamer-1.0 bash docs/MiroClone/ferramentas/youtube_motor/sonda_servo.sh aqz-KE-bpKQ <saida>
CARGO_TARGET_DIR=~/Apps/w18_motores/servo_spike/target cargo build --release --manifest-path docs/MiroClone/ferramentas/youtube_motor/servo_spike/Cargo.toml
```

⚠️ O `servoshell` sem ecrã **ignora o TERM**: três ficaram órfãos a segurar o som (12/10) — a sonda mata-o com `-9`.
