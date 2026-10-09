# MiroClone — o quadro de planear, organizar, brainstorm e equipa, dentro do PH2D

Canvas infinito no idioma do Miro/Excalidraw. Cada quadro é um DOCUMENTO numa aba da barra de cima
(nunca um objecto da cena), com núcleo próprio (`ph2d-board-model` · `ph2d-board-render` ·
`ph2d-app-board`) que reusa o roteador de setas `ph2d-vec-connect` e a geometria das formas.
Linha `line/MiroClone`, aberta em 2026-10-05.

| doc | o quê |
|---|---|
| [01_pesquisa_miro_excalidraw.md](01_pesquisa_miro_excalidraw.md) | as features que mais valem por objectivo, onde os dois falham, o que já temos, os oráculos e as decisões do dono |
| [02_plano.md](02_plano.md) | o veredito sobre formas e setas, a arquitectura, a velocidade, as ondas W0–W8 e a Etapa 2 (ao vivo) |
| [03_guias_e_alinhamento.md](03_guias_e_alinhamento.md) | W5b: como oito apps alinham objectos rodados (Excalidraw e tldraw corridos), as leis do PH2D com fonte e número, as recusas e a régua do encaixe |
| [04_planear_mapa_e_modelos.md](04_planear_mapa_e_modelos.md) | W6: o mapa mental do Miro medido na imagem oficial (faixas de 54, 104 entre níveis, braço 0,39) e três apps corridos, as leis M1–M19 e os seis modelos, as recusas e a régua do layout |
| [05_facilitacao.md](05_facilitacao.md) | W7: o cronómetro, a votação, os carimbos e agrupar do Miro, pela ajuda e pelas imagens oficiais medidas (o selo 0,22→0,34 do lado, a grelha ⌈√n⌉), o oráculo da palavra (Snowball 3.1.1 = `libstemmer`), as leis F/V/S/C, a auditoria, as recusas e a régua |
| [handoffs/](handoffs/) | registo de sessão — o último: [integração W7](handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W7_2026-10-09.md) |
| [ferramentas/tldraw_oracle/](ferramentas/tldraw_oracle/README.md) | o tldraw 5.5.2 corrido por script (licença tldraw: uso de desenvolvimento) — guias e encaixe (W5b) |
| [ferramentas/facilitacao_oracle/](ferramentas/facilitacao_oracle/README.md) | a `libstemmer` 3.1.1 (BSD-3) e o `Intl.Segmenter` do Node corridos por script — a raiz e as palavras de agrupar por palavra (W7) |
| [ferramentas/mindmap_oracle/](ferramentas/mindmap_oracle/README.md) | mind-elixir, simple-mind-map e markmap corridos por script (MIT) — layout e teclas do mapa mental (W6) |
| [ferramentas/excalidraw_oracle/](ferramentas/excalidraw_oracle/README.md) | o Excalidraw 0.18.1 corrido por script — instalação, uso, determinismo, o que já mediu |
