---
name: reference-display-topology-workstation
description: "Monitores da workstation Linux — qual GPU dirige cada tela, e o AOC é read-only em DDC/CI"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 6791cd7e-bfa2-4573-989a-a5f86469fcc3
---

Workstation Linux (CachyOS, KDE Wayland), 2 monitores em 2 GPUs distintas:

- **LG 25UM65** ultrawide 2560x1080 — **RTX 5060 Ti** (HDMI da placa dedicada). É o monitor
  **primário**, onde o trabalho acontece. DDC/CI completo (brilho, contraste, ganhos RGB).
- **AOC e22t** 1080p — **Radeon integrada** (HDMI da placa-mãe). Secundário.

**Para medir performance de GPU do PH2D, use a janela no LG.** Numa sessão Wayland, a tela em
que a janela está decide qual GPU apresenta; arrastar a janela entre os monitores troca a GPU
e passa a copiar o buffer entre as placas — queda de FPS que não tem nada a ver com o código.
(Os cabos foram trocados em 2026-07-14 justamente para o primário cair na placa dedicada.)

**O AOC e22t é read-only em DDC/CI**: responde a leituras e recusa TODA escrita, em qualquer
GPU, até com pacote MCCS montado à mão no `/dev/i2c-*`. Não tem DDC/CI no OSD porque não há o
que ligar. Não perca tempo re-investigando — nenhum software controla o brilho dele; só os
botões físicos. O plano B (rampa `vcgt` num perfil ICC por saída, aplicada pelo KWin) está
implementado no app `~/Documentos/Projetos/monitor-tune/` (fora do repo do PH2D).
