# Fixtures — o Wet Drip com as caixas K–M no PH2D (2026-10-10/11, prompt 16, Fases B–E)

A cena do smoke do dono ([`../smoke_dono_mistura_wet_drip/`](../smoke_dono_mistura_wet_drip/)) no nosso motor:
tela branca 600×400; duas passadas de VERMELHO `[220, 60, 60]` com o Wet Round (y 250 · 255, raio 30), molhado
(30 tiques) ou SECO (Dry canvas); 5 serpentinas de AZUL ESCURO `[20, 50, 110]` com o Wet Drip de fábrica
(y 175 · 200 · 225, raio 20); 600 tiques de escorrimento e o Dry canvas. Gerador: a sonda
`crates/ph2d-tool-painter/src/tool/paint/wetpaint/sonda_mistura_wet_drip.rs` (`PH2D_SONDA_OUT=<dir>`, PPM) e
[`ferramentas/prancha_mistura_wet_drip.py`](../../ferramentas/prancha_mistura_wet_drip.py) (as pranchas). A cena
e as réguas são as dos portões `wetpaint/mistura_wet_drip_tests.rs`.

- `<caixas>_<estado>_borda<n>[...]_8x.png` — Fase B (as quatro caixas de antes da cura); `faseC_*` / `prancha_faseC_*`
  — as ablações H1–H7; `faseD_*` — as candidatas da representação única (densidade, satura, contínua, curas
  locais), todas RECUSADAS por número (`05 §4b` «Mistura II»).
- **`faseE_*` — a cura (11/10):** `antes_*` = a lei de antes do «Glaze layering» (`Lei::Camada`, o controlo dos
  portões — os bytes são os do produto de antes, conferido por `cmp` contra a Fase D) contra `duas` / `glaze` /
  `nenhuma` do produto curado, molhado e seco; `faseE_prancha_<estado>_1x.png` (a tela 1:1 nos quatro momentos)
  e `_8x.png` (as quatro zonas: pontas · orla de baixo · fios · fronteira).
- Os `*_1x_reduzida.png` são as pranchas 1:1 das Fases B–D a metade da resolução (a pasta passava de 34 MB);
  as 1:1 de cada variante saíram (a sonda regera-as).
