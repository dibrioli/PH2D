---
name: reference-pocket-tts-installed
description: "Pocket TTS (Kyutai, TTS PT-BR local, código MIT, pesos CC-BY-4.0) instalado em ~/Documentos/Recursos/Pocket TTS — comando `falar`, página em :8077 na GPU; grande = 1,0 % WER, 7,8× tempo real"
metadata:
  type: reference
---

Instalado em 2026-10-11 a pedido do Enio (gostou dos testes; pediu «a melhor versão da melhor maneira»).

- **Onde:** `~/Documentos/Recursos/Pocket TTS/` — `LEIA-ME.md` (uso + secção técnica), projeto `uv`
  em `programa/` (pocket-tts fixado no commit `70ca5144` do main, à frente do 3.3.0 do PyPI; torch
  cu128), pesos e 27 vozes PT em `modelos/` (`HF_HOME`), saídas em `falas/`. Funciona offline.
- **Uso:** `falar "texto"` (em `~/.local/bin`), menu KDE «Pocket TTS» = **Estúdio** próprio (11/10,
  porta 8077, unidade `systemd --user` `pocket-tts`): trechos com tentativas, conferência por
  Whisper turbo na GPU (refaz sozinho se faltar palavra), 27 vozes com amostra e tom medido,
  dicionário de pronúncia, ajustes avançados com o que foi medido, exportação WAV/MP3/OGG/FLAC.
- **Medido (10 frases × 3, Whisper medium, load 60–80):** `portuguese_24l` 1,0 % de erro de
  palavras · `portuguese` retreinado 01/10 2,9 % · o de 24/09 3,5 %. GPU 7,8× / CPU 1,2× tempo real
  (grande); a GPU ganha até numa frase curta pelo terminal (5 s × 7–8 s). Erro típico: a 1.ª palavra.
- **Imitar voz** LIBERADO 11/10 (repo gated `kyutai/pocket-tts`; token de leitura da conta `dibrioli` em
  `modelos/token`, `HF_HOME` da pasta). Grande imita o tom quase exato (195→200 Hz); rápida mais longe (173).
- **Não ligado ao PH2D.** Se um dia entrar no módulo de Áudio: há portas Rust da comunidade
  (candle, wasm) — tratar como oráculo + triagem de licença ([[reference-craft-suite-installed]]).
