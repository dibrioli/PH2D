---
name: reference-craft-suite-installed
description: "Suite Craft (storytold: PhotoCraft·VectorCraft·EffectCraft·DesignCraft), Rust, MIT/Apache-2.0 — código em ~/Documentos/Recursos/Craft, apps em ~/Apps/<nome>, cada um com <nome>-cli"
metadata:
  type: reference
---

Instalado em 2026-10-11 a pedido do Enio, para estudo. Clones de Adobe em Rust puro (wgpu + egui),
de `github.com/storytold/<nome>`, alfa inicial, releases quase diárias.

- **Código:** `~/Documentos/Recursos/Craft/{photocraft,vectorcraft,effectcraft,designcraft}` (git clone
  completo; `README.md` da pasta lista o commit e a versão). AppImages + SHA256SUMS em `_instaladores/`.
- **Apps:** `~/Apps/<nome>/squashfs-root/AppRun` (padrão do obsidian), atalho em
  `~/.local/share/applications/<nome>.desktop` **sem `MimeType`** (para não roubar png/svg/pdf).
  Versões: Photo 0.7.0 · Vector 0.9.0 · Effect 0.8.0 · Design 0.6.0. `~/Apps` é invisível ao cachy-update.
- ⭐ **Licença MIT OU Apache-2.0** (a API do GitHub mostra só Apache) ⇒ porta ABERTA: ler e portar é
  legal com atribuição — não é zona contaminada (contraste: [[reference-motionbricks-oracle-installed]]).
- ⭐ Cada app tem um **`<nome>-cli`** em `usr/bin/` — a porta para corrê-lo como oráculo por script
  (CLAUDE.md §0.9). Mais relevantes para o PH2D: VectorCraft (Vector) e EffectCraft (Motion/Timeline).
- 2026-10-11: comparado com o nosso vetor em `docs/Vector Module/Estudos/ESTUDO_vectorcraft_comparado_2026-10-11.md` — ⭐ `linesweeper` 0.5 zera os pânicos do nosso offset (Round 87→100/100 na estrela do gate), compila sem mudar código; suíte do PH2D sob 0.5 ainda NÃO corrida.
