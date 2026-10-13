---
name: reference-vector-reference-apps-installed
description: Apps de referência vetor/ossos instalados em 2026-10-11 — Spine 4.3 Trial e Penpot Desktop em ~/Apps; Graphite e Rive como janelas do Chrome; atalhos no menu e na Área de trabalho
metadata:
  node_type: memory
  type: reference
  originSessionId: 0f78f672-4fca-4b87-b9fa-13b0fd26b120
  modified: 2026-10-11T18:51:38.895Z
---

Instalados em 2026-10-11 a pedido do Enio («para ref»), cada um com atalho em `~/.local/share/applications/`
e em `~/Área de trabalho/` (ícones em `~/Apps/webapps/icones/`).

- **Spine 4.3.26 Trial** (Esoteric, proprietário): `~/Apps/spine/SpineTrial/SpineTrial.sh` (JRE próprio; o
  lançador baixa o editor na 1.ª corrida). O teste NÃO salva, não empacota texturas e não exporta — serve de
  oráculo a olho (CLAUDE.md §0.9), abre com o projecto `spineboy-pro`.
- **Penpot Desktop 1.0.0** (author-more, AGPL-3.0 — embrulho Electron do Penpot, MPL-2.0): `~/Apps/penpot/squashfs-root/AppRun
  --no-sandbox`; na 1.ª corrida mostra a licença (zenity, «Agree») — é do dono aceitar. Liga a design.penpot.app (conta).
- **Graphite** (Rust, Apache-2.0 — porta ABERTA para ler código; usa kurbo como o PH2D): sem build desktop
  pública para Linux (out/2026 só RC pelo Discord) ⇒ `google-chrome-stable --app=https://editor.graphite.art`
  (o domínio mudou de graphite.rs para graphite.art).
- **Rive** (proprietário; a referência do vetor/ossos pelo ADR-0108): sem Linux ⇒ `--app=https://editor.rive.app`.
- Já estavam: Inkscape, Synfig, Glaxnimate, OpenToonz (BSD), Krita; VectorCraft ver [[reference-craft-suite-installed]].
- O Chrome corre nativo em Wayland: o `xdotool` não vê as janelas `--app` (não dá para fotografá-las por `import`).
