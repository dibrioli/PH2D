# Oráculo das camadas — GIMP e Krita corridos sem interface

**Pergunta:** as camadas juntam-se em que espaço? (ADR-0177 · doc Painter 45 §5)

**Triagem de licença (passo 1, §0.9):** `pacman -Qi gimp` ⇒ **GPL-3.0-or-later**; `pacman -Qi
krita` ⇒ **GPL3** — os dois só se CORREM, caixa-preta. O fonte não foi lido.

| ficheiro | o que é |
|---|---|
| [`entradas.py`](entradas.py) | a grelha NOSSA, a mesma para os dois oráculos |
| [`oraculo.py`](oraculo.py) | o arnês do GIMP (`python-fu-eval`): imagem `FLOAT_NON_LINEAR`, base + UMA camada, blend space = composite space = o da corrida, composição `UNION`; lê o visível em float straight |
| [`corre.sh`](corre.sh) | corre-o e escreve a fixtura com cabeçalho (versão, licença, comando, data, formato) |
| `gimp_3.2.6.bin` | a fixtura: cabeçalho de texto até `#FIM`, depois as entradas e 84 corridas em bytes |
| [`oraculo_krita.py`](oraculo_krita.py) · [`corre_krita.sh`](corre_krita.sh) | a 2.ª opinião: `kritarunner` com Qt *offscreen*, documento RGBA U8 sRGB (o Krita a 8 bits compõe em codificado) |
| `krita_6.0.4.bin` | a fixtura do Krita, mesmo formato (56 corridas) |

```
bash docs/Painter/ferramentas/oraculo_camadas_gimp/corre.sh        # ~2 s
bash docs/Painter/ferramentas/oraculo_camadas_gimp/corre_krita.sh  # ~2 s
```

As duas corridas são deterministas: regenerar o GIMP deu o corpo igual byte a byte (só o cabeçalho
muda, pela data).

**Entrada (nossa):** coluna = alfa da camada (rampa `0..255` em 17 degraus); linha = alfa da base
(`255`, `140`) × 6 cores da base × 6 da camada. 21 modos × «perceptual»/«linear» × opacidade
`100`/`60`.

**Quem lê:** `ph2d-tool-painter` `compositor::oraculo_gimp_tests` — o gate do controlo
(`o_gimp_linear_e_a_mistura_w3c_em_luz`) e a sonda `diag_o_compositor_contra_o_gimp` (a tabela
inteira: compositor · W3C em luz · W3C em tons de ecrã, cada uma contra o GIMP).

**O que responderam (03/10):** nos 10 modos de fórmula partilhada, o «linear» do GIMP é o
compositor de então (≤ 1) e o «perceptual» é a mesma matemática W3C sobre valores codificados (≤ 1).
O Krita a 8 bits é o compositor novo nos **22** modos (pior 4 degraus em ≤ 93 canais de 4 896 — o
arredondamento inteiro dele). As divergências do GIMP são fórmulas NOMEADAS, com desvio 0: `B` sem
corte a `[0, 1]` (Add, ColorBurn, ColorDodge, LinearBurn, LinearLight) e o Soft Light Pegtop. Gates e
sondas: `oraculo_gimp_tests` (`diag_que_formula_cada_oraculo_usa`). Por modo: ADR-0177.

⚠️ **Armadilhas medidas:**
- num modo sem função de mistura (Normal, Darken, Lighten, os HSL, Erase) o GIMP deixa o *blend
  space* em `auto` — a fixtura grava o que ele ACEITOU (`blend=` na linha `RUN`);
- `BEHIND` é modo só de pincel: o GIMP devolve a camada a `NORMAL` em silêncio (o arnês recusa-o e
  regista-o no cabeçalho);
- ⛔ **o Krita ACEITA qualquer id** de modo (o `blendingMode()` devolve-o) e pinta Normal em silêncio
  — medido com um id inventado. O arnês compara cada corrida com a do Normal e recusa a igual
  (`linear_light`, `clear`: o certo é `linear light`, com espaço, e `erase`);
- o `kritarunner` engole o `stdout` e morre com `BadWindow` se vir um `DISPLAY`: corre com
  `env -u DISPLAY -u WAYLAND_DISPLAY QT_QPA_PLATFORM=offscreen` e escreve para ficheiro;
- o `pixelData` do Krita em RGBA U8 vem em **BGRA**;
- pixel de alfa `0` à saída tem a cor indefinida — compara-se só o alfa.
