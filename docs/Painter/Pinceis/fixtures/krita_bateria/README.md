# Fixtures — a BATERIA do EXAME pintada pelo Krita 6 (o bundle padrão inteiro)

```
# cabecalho
#   app          = Krita 6.0.4.1 (pacote krita 6.0.4.1-1.1, GPL3) — so CORRIDO, nunca lido; Qt 6, PyQt6
#   presets      = os 144 presets carregados por omissao num perfil limpo com janela (bundles de /usr/share/krita;
#                  137 + os 7 «(mypaint)» que so a GUI carrega), 3 PNG cada = 432
#   bateria      = docs/Painter/Pinceis/ferramentas/bateria.json  sha256 cff23b71a54593d6118b2c1f5f7fe6cd5ed270f3094d53c5a5f3bfa741d914fa
#   tela         = documento 1000x640 RGBA U8 sRGB, 300 dpi, camada Background branca; zoom 1 (1 px = 1 px)
#   entrada      = eventos de CANETA espontaneos (QWindowSystemInterface::handleTabletEvent por ctypes):
#                  ferramenta de mao livre, suavizacao por omissao do perfil limpo, inclinacao 0, rotacao 0
#   tempo        = 10 ms de RELOGIO por evento (passo 3 px = 300 px/s lento; 25 px = 2500 px/s rapido) e timestamps
#                  10 ms; a caneta paira 10 eventos (pressao 0) antes de pousar, pousa, segue, levanta (pressao 0)
#   tamanho      = View.setBrushSize(size_px) (diametro); cor = View.setForeGroundColor(RGB da bateria)
#   molhado/seco = o Krita nao tem tempo de secagem: simula/seca sao no-ops e a foto A = B (grava-se uma so)
#   base (sobre) = as faixas vermelhas sao o MESMO preset, salvo os 23 que nao pousam tinta no papel (borrachas
#                  a), misturadores k), l) Adjust, v) Clone/Distort, x) Filter, j) WaterC Water-Pattern,
#                  t) Shapes Fill, y) Texture Starfield): esses tem as faixas pintadas com «b) Basic-1»
#                  (--base; o manifesto diz «base_sobre» por preset) — senao a cena sobre saia em branco
#   saida        = documento exportado (exportImage, RGBA) composto sobre branco, PNG RGB 8 bits sem perdas
#   isolamento   = perfil proprio ~/.cache/ph2d-oraculos/krita (XDG_*), Xvfb proprio (:87+), sem DISPLAY do dono
#   maquina      = workstation do dono, 2026-10-20
```

**Comando** (o lote inteiro, ~45 s por preset, ≈ 1 h 45 min; retoma onde parou):

```
python3 -I docs/Painter/Pinceis/ferramentas/krita_bateria.py
python3 -I docs/Painter/Pinceis/ferramentas/krita_bateria.py --presets "c) Pencil-2,d) Ink-2 Fineliner"
```

**Saída:** `<nome_seguro>_<cena>.png` (cenas `gestos` · `tamanhos` · `sobre`; `)`, espaços… → `_`).
`manifesto.json`: por preset, o ficheiro `.kpp`/bundle de origem, e por cena `px_com_tinta` (algum canal a > 2 do
papel) e o sha256 (16).

**Repetibilidade (medida, 2026-10-20, 5 presets × 3 cenas, duas corridas iguais):** `Basic-5 Size`, `Airbrush Soft`
e `Bristles-2 Flat Rough` saem **byte-idênticos**; `Pencil-2` varia 7–12 mil px até 35/255 de corrida para corrida
(aleatoriedade própria do preset, não semeável por script); `Ink-2 Fineliner` varia 1,2–2,4 mil px (o relógio).
⇒ preset com textura/aleatório compara-se por estatística (cobertura, perfil transversal), nunca pixel a pixel.

**Licença:** o Krita é GPL3 — a fixture é a SAÍDA do programa sobre entradas nossas; nenhum ficheiro `.kpp`,
textura ou ponta foi copiado para o repo. O que se porta é o comportamento medido.

⚠️ **O que esta fixture NÃO mede:** inclinação/rotação da caneta (0), o «pousar em movimento» (pousa parada), o tempo
de secagem (não existe), a mistura entre camadas (uma camada só) e o preset «sujo» (ajustes temporários do
utilizador): cada corrida parte do perfil limpo.
