# B08c — antes → depois no app (Radeon 610M RADV, Xvfb, papel/relevo do dono)

- **Depois:** HEAD `892acec6f9f2` (`line/PainterWatercolor`, árvore limpa salvo os 2 roteiros deste commit). Binário
  `CARGO_PROFILE_SMOKE_STRIP=none bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` (09:42, 1 min 07 s).
  **Antes:** `cd02ac77ef4f` (ANTES.md; a célula do antes escolhe-a o `compara_antes_depois.py`). Mesma placa, roteiro e perfil `smoke`.
- **Corrida:** 2026-11-11 09:43–10:13, `timeout 1800 bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/corre_depois_b08c.sh`,
  SOZINHO na máquina (nenhum outro agente). Carga por grupo (`target/mede-depois/carga.txt`): 4K papel 09:43 **load 17,6**
  (o build acabara 1 min antes) → 09:53 0,66 · cantos 0,66–0,82 · 4K branco 0,82 → 0,49 · 1080p papel 0,49 → 0,60 ·
  4K blocos 0,60 → 0,58 · 1024² blocos 0,58, cortado pelo prazo de 1800 s; Filbert e Express Oil a 1024² blocos medidos à
  parte (M2, 10:27–10:29, load 0,54–0,67, o MESMO binário: `matriz 1024x1024 blocos … "Filbert:wet" "Express Oil:wet"`).
  Carga de cada lançamento no `.meta`. Quadros crus: [`depois/`](depois/).
- **Parado ✗ em todas as células, antes E depois (~50 % de um núcleo):** o app parado sob Xvfb + WSI de software
  (`MESA_VK_WSI_DEBUG=sw`); não mudou com a missão.

## Tabela por célula (antes → depois, ms; ✓/✗ = alvo do ANTES.md)

| pincel | painel | tela | fundo | toque | traço | soltar | +1 s | Ctrl+Z | parado | Wet |
|---|---|---|---|---|---|---|---|---|---|---|
| Express Oil | wet | 1024x1024 | blocos | 18.1 → 17.1 ✓ | 12.9 / 14.0 → 13.0 / 14.1 ✓ | 15.8 → 14.0 ✓ | 15.8 → 14.0 ✓ | 30.1 → 31.0 ✓ | 55.0 / 54.3 → 53.9 / 54.3 ✗ | 40.3 → 39.9 ✓ |
| Filbert | wet | 1024x1024 | blocos | 22.9 → 20.9 ✓ | 19.8 / 23.0 → 17.7 / 20.9 ✗ | 18.9 → 43.2 ✗ | 18.9 → 43.2 ✗ | 35.0 → 34.0 ✗ | 54.9 / 54.3 → 54.6 / 54.6 ✗ | 40.0 → 39.5 ✓ |
| Watercolor Basic | water | 1024x1024 | blocos | 21.9 → 22.9 ✓ | 14.0 / 17.3 → 13.0 / 14.1 ✓ | 22.2 → 20.8 ✓ | 23.3 → 20.8 ✓ | 16.8 → 11.8 ✓ | 62.9 / 67.9 → 52.9 / 52.3 ✗ | — |
| omissão | digital | 1024x1024 | blocos | 13.9 → 10.8 ✓ | 11.0 / 12.3 → 9.8 / 10.4 ✓ | 10.9 → 10.8 ✓ | 11.7 → 11.0 ✓ | 14.0 → 11.0 ✓ | 66.6 / 56.0 → 53.3 / 53.9 ✗ | — |
| omissão | impasto | 1024x1024 | blocos | 26.4 → 18.9 ✓ | 14.5 / 18.9 → 10.3 / 11.1 ✓ | 14.8 → 14.8 ✓ | 16.9 → 14.8 ✓ | 14.1 → 14.3 ✓ | 65.6 / 56.6 → 51.3 / 51.3 ✗ | — |
| omissão | water | 1024x1024 | blocos | 33.9 → 23.0 ✓ | 14.9 / 16.0 → 13.1 / 14.7 ✓ | 21.0 → 21.0 ✓ | 21.0 → 21.0 ✓ | 15.0 → 12.9 ✓ | 54.3 / 68.9 → 52.9 / 53.0 ✗ | — |
| omissão | wet | 1024x1024 | blocos | 14.8 → 16.0 ✓ | 11.3 / 12.9 → 11.1 / 12.0 ✓ | 14.9 → 14.0 ✓ | 14.9 → 14.0 ✓ | 26.2 → 29.9 ✓ | 57.0 / 62.3 → 54.3 / 54.9 ✗ | 40.2 → 39.6 ✓ |
| Express Oil | wet | 1920x1080 | papel | 34.9 → 27.3 ✓ | 14.9 / 17.3 → 13.9 / 14.9 ✓ | 17.0 → 16.9 ✓ | 33.0 → 16.9 ✓ | 44.9 → 37.2 ✗ | 53.6 / 53.6 → 53.3 / 53.3 ✗ | 39.8 → 40.1 ✓ |
| Filbert | wet | 1920x1080 | papel | 38.3 → 28.0 ✓ | 20.1 / 23.8 → 19.0 / 22.0 ✗ | 23.9 → 38.2 ✗ | 33.9 → 38.2 ✗ | 47.8 → 39.1 ✗ | 54.6 / 55.0 → 54.3 / 54.3 ✗ | 39.8 → 39.7 ✓ |
| Watercolor Basic | water | 1920x1080 | papel | 55.8 → 28.1 ✓ | 17.1 / 18.0 → 15.0 / 16.1 ✓ | 24.0 → 24.0 ✓ | 36.0 → 24.0 ✓ | 80.2 → 15.0 ✓ | 63.6 / 70.5 → 50.3 / 50.0 ✗ | — |
| omissão | digital | 1920x1080 | papel | 34.9 → 13.8 ✓ | 13.0 / 14.8 → 11.0 / 11.1 ✓ | 13.9 → 11.1 ✓ | 31.7 → 11.1 ✓ | 32.2 → 12.8 ✓ | 53.3 / 51.6 → 52.3 / 49.9 ✗ | — |
| omissão | impasto | 1920x1080 | papel | 35.9 → 13.8 ✓ | 13.0 / 14.4 → 11.0 / 11.6 ✓ | 15.2 → 15.9 ✓ | 35.1 → 15.9 ✓ | 30.9 → 14.8 ✓ | 53.0 / 52.6 → 49.6 / 50.0 ✗ | — |
| omissão | water | 1920x1080 | papel | 52.1 → 27.1 ✓ | 14.4 / 16.9 → 15.2 / 16.2 ✓ | 21.2 → 22.9 ✓ | 34.8 → 22.9 ✓ | 36.0 → 14.9 ✓ | 53.6 / 52.6 → 50.3 / 50.0 ✗ | — |
| omissão | wet | 1920x1080 | papel | 34.9 → 25.4 ✓ | 13.0 / 14.0 → 12.0 / 13.0 ✓ | 13.6 → 15.0 ✓ | 13.8 → 15.0 ✓ | 40.2 → 31.1 ✓ | 54.3 / 64.3 → 52.6 / 53.0 ✗ | 40.3 → 40.5 ✓ |
| Express Oil | wet | 3840x2160 | blocos | 46.0 → 31.0 ✓ | 13.4 / 14.9 → 13.1 / 14.6 ✓ | 15.7 → 22.0 ✓ | 20.8 → 22.0 ✓ | 69.8 → 68.3 ✗ | 51.3 / 51.3 → 51.6 / 52.0 ✗ | 39.3 → 39.4 ✓ |
| Filbert | wet | 3840x2160 | blocos | 53.0 → 43.0 ✗ | 20.8 / 23.2 → 18.7 / 21.1 ✗ | 23.2 → 54.0 ✗ | 23.2 → 54.0 ✗ | 70.2 → 69.1 ✗ | 54.9 / 53.6 → 53.3 / 53.3 ✗ | 39.9 → 40.1 ✓ |
| Watercolor Basic | water | 3840x2160 | blocos | 41.2 → 34.0 ✗ | 19.8 / 21.0 → 20.1 / 21.7 ✗ | 26.1 → 32.1 ✓ | 26.1 → 32.1 ✓ | 38.0 → 16.2 ✓ | 51.0 / 50.6 → 51.0 / 50.0 ✗ | — |
| omissão | digital | 3840x2160 | blocos | 18.2 → 12.0 ✓ | 10.1 / 10.4 → 9.5 / 10.1 ✓ | 10.8 → 10.1 ✓ | 17.7 → 11.1 ✓ | 14.9 → 15.0 ✓ | 54.6 / 50.3 → 55.0 / 54.6 ✗ | — |
| omissão | impasto | 3840x2160 | blocos | 62.4 → 64.2 ✗ | 18.0 / 18.5 → 10.9 / 11.6 ✓ | 24.3 → 18.4 ✓ | 43.0 → 18.4 ✓ | 33.5 → 42.1 ✗ | 56.6 / 49.3 → 51.9 / 50.3 ✗ | — |
| omissão | water | 3840x2160 | blocos | 41.9 → 35.1 ✗ | 20.2 / 21.9 → 20.0 / 21.0 ✗ | 28.0 → 33.0 ✓ | 28.0 → 33.0 ✓ | 38.4 → 16.2 ✓ | 54.3 / 57.0 → 50.6 / 50.3 ✗ | — |
| omissão | wet | 3840x2160 | blocos | 26.3 → 28.0 ✓ | 10.9 / 12.3 → 11.9 / 12.3 ✓ | 17.9 → 21.1 ✓ | 17.9 → 21.1 ✓ | 64.1 → 63.1 ✗ | 50.6 / 50.6 → 51.3 / 51.6 ✗ | 38.9 → 39.1 ✓ |
| Express Oil | wet | 3840x2160 | branco | 105.0 → 71.3 ✗ | 14.9 / 16.9 → 12.9 / 14.4 ✓ | 17.2 → 23.8 ✓ | 52.9 → 23.8 ✓ | 103.9 → 64.0 ✗ | 52.6 / 65.6 → 51.3 / 51.6 ✗ | 38.1 → 39.9 ✓ |
| Filbert | wet | 3840x2160 | branco | 217.2 → 83.9 ✗ | 65.1 / 168.5 → 18.9 / 21.8 ✗ | 87.0 → 50.0 ✗ | 87.0 → 50.0 ✗ | 107.2 → 61.8 ✗ | 58.6 / 57.0 → 53.0 / 53.6 ✗ | 38.2 → 40.1 ✓ |
| Watercolor Basic | water | 3840x2160 | branco | 41.8 → 33.0 ✓ | 20.8 / 22.0 → 19.2 / 20.9 ✗ | 27.8 → 33.8 ✗ | 54.0 → 33.8 ✗ | 77.9 → 16.8 ✓ | 57.6 / 53.6 → 50.6 / 50.9 ✗ | — |
| omissão | digital | 3840x2160 | branco | 60.1 → 11.1 ✓ | 13.8 / 19.6 → 9.8 / 10.5 ✓ | 17.1 → 11.6 ✓ | 66.4 → 11.6 ✓ | 55.9 → 15.7 ✓ | 67.3 / 59.3 → 54.9 / 50.9 ✗ | — |
| omissão | impasto | 3840x2160 | branco | 101.6 → 57.7 ✗ | 20.1 / 56.5 → 11.0 / 11.1 ✓ | 46.8 → 17.2 ✓ | 134.5 → 17.2 ✓ | 135.3 → 67.9 ✗ | 68.9 / 58.6 → 50.3 / 49.3 ✗ | — |
| omissão | water | 3840x2160 | branco | 39.9 → 33.8 ✗ | 19.5 / 20.2 → 20.0 / 21.9 ✗ | 27.8 → 31.4 ✓ | 53.1 → 31.4 ✓ | 72.6 → 16.6 ✓ | 53.3 / 51.6 → 51.3 / 50.6 ✗ | — |
| omissão | wet | 3840x2160 | branco | 97.9 → 67.2 ✗ | 17.9 / 30.1 → 11.8 / 12.3 ✓ | 31.1 → 20.9 ✓ | 32.8 → 20.9 ✓ | 198.2 → 44.4 ✗ | 65.3 / 68.9 → 51.3 / 51.3 ✗ | 25.5 → 39.7 ✓ |
| Chalk | digital | 3840x2160 | papel | 99.0 → 24.9 ✓ | 24.0 / 25.0 → 12.0 / 12.2 ✓ | 24.1 → 12.9 ✓ | 100.1 → 12.9 ✓ | 101.0 → 14.9 ✓ | 55.9 / 55.6 → 53.0 / 51.9 ✗ | — |
| Composite Brush | digital | 3840x2160 | papel | 97.1 → 35.0 ✗ | 12.8 / 25.8 → 11.8 / 15.9 ✓ | 25.0 → 15.9 ✓ | 99.1 → 15.9 ✓ | 102.4 → 17.2 ✓ | 52.6 / 53.0 → 50.6 / 50.0 ✗ | — |
| Crayon | digital | 3840x2160 | papel | 97.9 → 24.8 ✓ | 23.8 / 24.3 → 12.0 / 12.2 ✓ | 23.9 → 12.0 ✓ | 101.3 → 12.0 ✓ | 103.0 → 16.9 ✓ | 56.6 / 56.6 → 53.3 / 52.3 ✗ | — |
| Express Oil | wet | 3840x2160 | papel | 98.0 → 64.8 ✗ | 27.0 / 28.8 → 13.9 / 14.8 ✓ | 27.9 → 22.1 ✓ | 97.0 → 22.1 ✓ | 101.9 → 61.7 ✗ | 55.3 / 54.3 → 52.3 / 53.0 ✗ | 40.1 → 40.1 ✓ |
| Filbert | wet | 3840x2160 | papel | 101.1 → 76.0 ✗ | 34.0 / 37.2 → 19.0 / 22.0 ✗ | 37.2 → 51.0 ✗ | 101.1 → 51.0 ✗ | 100.9 → 62.9 ✗ | 54.3 / 55.3 → 54.3 / 54.0 ✗ | 39.9 → 39.6 ✓ |
| Finger Tip | digital | 3840x2160 | papel | 98.1 → 22.0 ✓ | 23.6 / 24.5 → 11.0 / 11.9 ✓ | 23.8 → 11.1 ✓ | 100.9 → 11.1 ✓ | 100.1 → 51.1 ✗ | 52.6 / 48.6 → 50.3 / 49.6 ✗ | — |
| Pastel | digital | 3840x2160 | papel | 99.9 → 25.7 ✓ | 24.0 / 25.0 → 12.0 / 12.2 ✓ | 24.8 → 12.1 ✓ | 98.0 → 12.1 ✓ | 99.2 → 16.0 ✓ | 55.6 / 55.0 → 52.6 / 51.3 ✗ | — |
| Round Mixing | digital | 3840x2160 | papel | 98.0 → 22.9 ✓ | 24.1 / 25.2 → 11.6 / 12.2 ✓ | 24.9 → 11.3 ✓ | 101.1 → 11.3 ✓ | 100.0 → 16.3 ✓ | 49.6 / 51.3 → 50.3 / 49.3 ✗ | — |
| Watercolor Basic | water | 3840x2160 | papel | 154.5 → 33.2 ✓ | 20.0 / 20.9 → 20.1 / 21.2 ✗ | 28.0 → 33.1 ✓ | 98.8 → 33.1 ✓ | 99.9 → 18.1 ✓ | 52.9 / 53.9 → 50.6 / 50.0 ✗ | — |
| Wet Liner | wet | 3840x2160 | papel | 97.1 → 66.7 ✗ | 24.1 / 25.1 → 12.2 / 13.1 ✓ | 24.0 → 22.3 ✓ | 31.0 → 22.3 ✓ | 105.4 → 66.0 ✗ | 55.9 / 56.3 → 52.3 / 52.6 ✗ | 35.9 → 39.4 ✓ |
| Wet Wash Mix | digital | 3840x2160 | papel | 101.8 → 16.1 ✓ | 25.0 / 25.9 → 12.9 / 13.7 ✓ | 24.9 → 13.7 ✓ | 102.8 → 13.7 ✓ | 99.0 → 15.8 ✓ | 56.6 / 55.6 → 52.3 / 52.3 ✗ | — |
| omissão | digital | 3840x2160 | papel | 97.9 → 20.9 ✓ | 23.1 / 23.9 → 11.0 / 12.0 ✓ | 23.9 → 11.8 ✓ | 101.9 → 11.8 ✓ | 98.9 → 15.9 ✓ | 50.6 / 51.3 → 50.6 / 51.6 ✗ | — |
| omissão | impasto | 3840x2160 | papel | 104.2 → 20.2 ✓ | 23.7 / 24.5 → 11.0 / 12.1 ✓ | 30.8 → 18.9 ✓ | 109.4 → 18.9 ✓ | 99.1 → 22.0 ✓ | 53.3 / 55.0 → 50.3 / 50.3 ✗ | — |
| omissão | water | 3840x2160 | papel | 155.2 → 33.9 ✗ | 19.9 / 20.9 → 20.8 / 21.9 ✗ | 27.9 → 36.8 ✗ | 101.1 → 36.8 ✗ | 100.1 → 17.0 ✓ | 54.6 / 53.9 → 50.6 / 50.0 ✗ | — |
| omissão | wet | 3840x2160 | papel | 97.2 → 63.7 ✗ | 23.9 / 24.3 → 12.0 / 13.0 ✓ | 24.0 → 20.8 ✓ | 28.6 → 20.8 ✓ | 104.0 → 46.9 ✗ | 54.9 / 54.6 → 52.6 / 51.9 ✗ | 38.5 → 39.9 ✓ |

| fase | passam antes | passam depois | células |
|---|---|---|---|
| toque | 8 | 28 | 43 |
| traço | 10 | 32 | 43 |
| soltar | 40 | 36 | 43 |
| +1 s | 19 | 36 | 43 |
| Ctrl+Z | 9 | 27 | 43 |
| parado | 0 | 0 | 43 |
| Wet | 16 | 16 | 16 |

## Regressões vistas (depois pior que antes, mesma célula; 1 amostra, load < 1 salvo 4K papel)

- **Filbert (wet), soltar:** 4K papel 37,2 → 51,0 · 4K blocos 23,2 → 54,0 · 1080p papel 23,9 → 38,2 · 1024² blocos 18,9 → 43,2 ms
  (traço e Ctrl+Z melhoraram).
- **Water (omissão e Watercolor Basic), soltar 4K:** 27,9 → 33–37 ms (✓ → ✗ no 4K papel/branco).
- **Impasto 4K blocos, Ctrl+Z:** 33,5 → 42,1 ms.

## M2 — as regressões por A×B no MESMO binário (11/11, `target/mede-depois/ph2d-host-desktop`, sozinho)

`SO=M<n> … mede_pen_up_4k.sh faixas 3840x2160 blocos 5` (A·B intercalados por rodada; saída `target/mede-m2/matriz`).
Mínimo de 5 por variante, ms; load 0,18–1,36.

| caso | A | B | soltar A (5) | soltar B (5) | mín A × B | traço p50 mín A × B | veredito |
|---|---|---|---|---|---|---|---|
| M1 Filbert 4K blocos | espectral (de fábrica) | `PH2D_WET_MISTURA=km` | 50,9 · 48,4 · 48,0 · 54,9 · 49,9 | 50,1 · 56,0 · 49,0 · 50,0 · 48,0 | **48,0 × 48,0** | 18,8 × 18,8 | a mistura B32.1 **não** é a causa |
| M3 Filbert 4K blocos | luz Simples no gesto, Cheia ao parar (de fábrica, B14.2) | `PH2D_COMPOSITE_LUZ_GESTO=cheia` | 54,9 · 49,0 · 47,8 · 48,2 · 49,1 | 20,0 · 26,0 · 19,9 · 21,8 · 21,9 | **47,8 × 19,9** | 18,2 × 18,9 | **culpado: o salto Simples→Cheia no pen-up** |

- **Mecanismo (perf, `mede_pen_up_4k.sh perf 3840x2160 Filbert:wet`, `LENTO_MS=30`):** um único quadro de 41–50 ms a
  +60 ms do mouse-up, ~55 % em `ph2d_wet_paint::oleo_real` (`oleo_horizonte::visibilidade_e_ceu` 11,8 %,
  `oleo_sombra::rebote_no_papel` 8,9 %, `luz_directa` 6,0 %, `Sectores::ate` 5,9 %, `compoe_a_otica` 4,5 %): a 1.ª
  imagem Cheia do repouso recompõe de uma vez o que o gesto compôs em Simples (`wet_luz_cheia_no_repouso`). No ANTES
  (`b08b-cont4`, `mede-matriz-b08b3`) não havia quadro > 30 ms no 1 s depois do up: o trabalho ia espalhado a 16–29 ms.
  Com a Cheia também no gesto o quadro do up some (máx 22 ms) e o traço não fica mais caro (p50 18,2 × 18,9, p95
  21,2 × 21,2 mín.) — a Simples do gesto não paga aqui o que custa no pen-up.
- **Water soltar 4K:** não confirmada. O quadro do up é 15,3 ms; os 36,8 ms são o `up` fora do quadro (20,5 ms), e no
  antes esse mesmo `up` deu 11,3 · 17,2 · 22,5 ms em três corridas (`mede-matriz`, `b08b3`, `b103`): cabe na dispersão.
- **Impasto 4K blocos Ctrl+Z (M4, A = B = o «depois», 3 rodadas):** 42,4 · 44,5 · 43,7 · 42,7 · 43,0 · 43,9 ms (mín 42,4) ×
  antes 32,4 · 32,8 · 33,5 (`mede-matriz-b08b3`, `b08b-cont3`, ANTES.md) ⇒ **regressão real, +10 ms**, sem interruptor. perf
  (`FUNDO=blocos … perf 3840x2160 omissão:impasto`): `premultiply_rgba8` 27 %, `Confine::add_relevo::<[u8;7]>`
  (`PlaneDeltas::confined_sujo`) 16 %. Suspeitos e próximo passo: BACKLOG B08c.M2.

## B17.5 — Ctrl+Z de dois toques em cantos opostos do 4K (papel)

`CANTOS="390 140 1240 890" CURTO=1` (traço curto no canto alto-esquerdo, toque no baixo-direito, `ctrl+z ctrl+z` seguidos;
saída `target/mede-depois/cantos`), load 0,66–0,82. Só o binário «depois»: não há binário da base guardado (sem A×B).

| pincel | painel | tela | fundo | base ms | toque ms | traço p50/p95 ms | soltar ms | +1 s máx ms | Ctrl+Z ms | parado CPU % (antes/depois) | Wet img/s |
|---|---|---|---|---|---|---|---|---|---|---|---|
| omissão | digital | 3840x2160 | papel | 10.0 | 19.8 | 11.0 / 12.0 | 11.2 | 11.2 | 14.1 | **50.3 ✗** / **51.3 ✗** | — |
| omissão | impasto | 3840x2160 | papel | 10.2 | 21.2 | 10.8 / 11.8 | 16.9 | 16.9 | 24.0 | **50.3 ✗** / **49.9 ✗** | — |
| omissão | wet | 3840x2160 | papel | 11.0 | **62.5 ✗** | 11.0 / 12.8 | 21.1 | 21.1 | **56.0 ✗** | **53.6 ✗** / **52.9 ✗** | 39.9 |
