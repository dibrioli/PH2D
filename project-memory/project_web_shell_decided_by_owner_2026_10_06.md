---
name: project-web-shell-decided-by-owner-2026-10-06
description: "O dono decidiu (06/10) que o PH2D TERÁ uma versão no navegador — a shell web é frente aprovada, por abrir como linha nova"
metadata:
  node_type: memory
  type: project
  originSessionId: 0313dd64-487d-47dc-9427-42b17d54677e
  modified: 2026-10-06T22:52:44.455Z
---

Em 2026-10-06, no fecho da W17 da `line/components`, o Enio respondeu **SIM** a «quer que o PH2D tenha uma versão que
rode no navegador?». A shell web deixou de ser hipótese e passou a ser frente de PRODUTO aprovada. ⚠️ Logo a seguir, no mesmo dia: *«não faremos esse trabalho agora»* — está aprovada, mas ESTACIONADA. Não a proponha como próximo passo; espere que ele a peça. Ela ainda não
existe: é preciso abrir uma linha nova (`/pd-linha-abrir`), e nenhuma linha de módulo a abre por conta própria.

**Why:** até 06/10, a cura do «web numa thread» (ADR-0180, adenda) e o degrau do `wasm32` ficavam parados à espera
desta decisão.

**How to apply:** o 1.º degrau já está medido (ADR-0180, adenda 2): `ph2d-nav`, `ph2d-navmesh` e `ph2d-orca`
compilam para `wasm32-unknown-unknown`. A `ph2d-physics-ecs` pára SÓ no codec AVIF (`libavif-sys`, `libdav1d-sys`,
`rav1e`), que chega por `ph2d-ecs → ph2d-asset → ph2d-imageio-registry-init → ph2d-imageio-avif`. O passo seguinte
é tirar o AVIF desse caminho no `wasm32`; depois vem o `rayon` sobre Web Workers (o resultado não muda, há um gate
de 1 contra 8 threads). Atenção ao §0.1 do CLAUDE.md: a web é uma SHELL, não plugin WASM — o norte continua a ser
um monorepo Rust.
