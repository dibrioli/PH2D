---
name: reference-ltx-desktop-video-gen-installed
description: "LTX Desktop 1.2.7 (LTX 2.5 Fast, geração de vídeo+som local) instalado na workstation em 2026-10-04; onde mora, como dirigir por script, números medidos."
metadata:
  node_type: memory
  type: reference
  originSessionId: 65dc2f5a-ed18-487a-913b-10ee3021775c
  modified: 2026-10-04T15:57:05.026Z
---

**LTX Desktop 1.2.7** (Lightricks, LTX 2.5 Fast 22B destilado, bf16, modo `streaming_models_loading`)
instalado em 2026-10-04, tudo no disco do SISTEMA (XPG, o «disco 1» do Enio), nunca no de projetos:

- AppImage: `~/Apps/ltx-desktop/` · menu: `~/.local/share/applications/ltx-desktop.desktop`
- dados + modelos (~100 GB: ltx-2.5 68 G com Gemma local, Z-Image-Turbo 31 G): `~/.local/share/LTXDesktop/`
- vídeos gerados: `~/.local/share/LTXDesktop/outputs/`
- Sem chave LTX API; codificador de texto LOCAL ⇒ nada sai da máquina, custo zero.
  Licença LTX-2.x Community (grátis < US$10 M/ano), aceita pelo Enio no HF.

⛔ **Lançar a partir do Claude Code/VSCode falha com `bad option: --no-sandbox`**: o ambiente herda
`ELECTRON_RUN_AS_NODE=1`. Use `env -u ELECTRON_RUN_AS_NODE ./LTX-Desktop-x86_64.AppImage`.

**Dirigir por script:** o backend FastAPI escuta numa porta aleatória em 127.0.0.1 (ver `ss -ltnp`
do PID de `ltx2_server.py`), auth `Bearer $LTX_AUTH_TOKEN` (ler de `/proc/<pid>/environ`).
`POST /api/generate` `{"prompt","resolution":"540p","duration":5,"fps":24,"audio":true,"seed":42}`
é síncrono e devolve `video_path`. Durações locais: 540p 5–20 s · 720p 5–10 s · 1080p 5 s.

**Medido (1.ª corrida, load ~17 com 2 rustc a correr):** 540p×5 s com som = **149 s** de ponta a ponta
(texto 35 s frio · 8 passos a 512×288 20 s · 3 passos a 1024×576 24 s · decode vídeo+áudio 44 s);
saída 1024×576 24 fps; pico VRAM **8,8 GB** de 16; RAM do app ~29 GB. Imagem muito boa; áudio
gerado mas BAIXO (média −58,6 dB, pico −34,6 dB).
