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
| [06_equipa_e_saida.md](06_equipa_e_saida.md) | W8: comentários, versões nomeadas, apresentar e exportar do Miro, pela ajuda e pelas imagens oficiais medidas (a bolha com tamanho de interface, a margem 0,8 %, a viagem T 1,33 s, o PNG por área 4/16/49 MP), o `.excalidraw` pelo oráculo, a triagem dos validadores de SVG/PDF, as leis K/R/P/E/X, as recusas e as réguas |
| [07_fidelidade_excalidraw.md](07_fidelidade_excalidraw.md) | W9 (estudo e cura — [handoff W9](handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W9_2026-10-10.md)): a fidelidade do `.excalidraw` medida em DUAS versões do oráculo (0.18.1 e o `next` do excalidraw.com, 70 casos, 220 fixtures): porque as setas do dono saíram curvas e entravam nas formas, a ligação `orbit` (vão 5 + sw/2, erro ≤ 0,04 px contra o ficheiro dele), a ponta min(25, trecho/2), a tabela do formato inteiro, as recusas e o plano |
| [example.excalidraw](example.excalidraw) | o ficheiro da lição 5 da cena 10 (abrir um `.excalidraw`) |
| [bv.excalidraw](bv.excalidraw) · [bv_excalidraw.png](bv_excalidraw.png) | o ficheiro do dono (smoke 10/10) e a foto que o Excalidraw desenha dele — as lições 1 e 2 da cena 11 (W9, `PH2D_BOARD_SMOKE=11`) |
| [handoffs/](handoffs/) | registo de sessão — o último: [integração W8](handoffs/HANDOFF_INTEGRACAO_line_MiroClone_W8_2026-10-10.md) |
| [ferramentas/tldraw_oracle/](ferramentas/tldraw_oracle/README.md) | o tldraw 5.5.2 corrido por script (licença tldraw: uso de desenvolvimento) — guias e encaixe (W5b) |
| [ferramentas/facilitacao_oracle/](ferramentas/facilitacao_oracle/README.md) | a `libstemmer` 3.1.1 (BSD-3) e o `Intl.Segmenter` do Node corridos por script — a raiz e as palavras de agrupar por palavra (W7) |
| [ferramentas/mindmap_oracle/](ferramentas/mindmap_oracle/README.md) | mind-elixir, simple-mind-map e markmap corridos por script (MIT) — layout e teclas do mapa mental (W6) |
| [ferramentas/excalidraw_oracle/](ferramentas/excalidraw_oracle/README.md) | o Excalidraw 0.18.1 corrido por script — instalação, uso, determinismo, o que já mediu |
