# Oráculo das camadas — GIMP corrido sem interface

**Pergunta:** as camadas juntam-se em que espaço? (ADR-0177 · doc Painter 45 §5)

**Triagem de licença (passo 1, §0.9):** `pacman -Qi gimp` ⇒ **GPL-3.0-or-later** — só se CORRE,
caixa-preta. O fonte não foi lido.

| ficheiro | o que é |
|---|---|
| [`oraculo.py`](oraculo.py) | o arnês (`python-fu-eval`): imagem `FLOAT_NON_LINEAR`, base + UMA camada, blend space = composite space = o da corrida, composição `UNION`; lê o visível em float straight |
| [`corre.sh`](corre.sh) | corre-o e escreve a fixtura com cabeçalho (versão, licença, comando, data, formato) |
| `gimp_3.2.6.bin` | a fixtura: cabeçalho de texto até `#FIM`, depois as entradas e 84 corridas em bytes |

```
bash docs/Painter/ferramentas/oraculo_camadas_gimp/corre.sh      # ~2 s
```

**Entrada (nossa):** coluna = alfa da camada (rampa `0..255` em 17 degraus); linha = alfa da base
(`255`, `140`) × 6 cores da base × 6 da camada. 21 modos × «perceptual»/«linear» × opacidade
`100`/`60`.

**Quem lê:** `ph2d-tool-painter` `compositor::oraculo_gimp_tests` — o gate do controlo
(`o_gimp_linear_e_a_mistura_w3c_em_luz`) e a sonda `diag_o_compositor_contra_o_gimp` (a tabela
inteira: compositor · W3C em luz · W3C em tons de ecrã, cada uma contra o GIMP).

**O que ele respondeu (03/10):** nos 10 modos de fórmula partilhada, o «linear» do GIMP é o
compositor de então (≤ 1) e o «perceptual» é a mesma matemática W3C sobre valores codificados (≤ 1).
Os resultados por modo estão no ADR-0177.

⚠️ **Armadilhas medidas:**
- num modo sem função de mistura (Normal, Darken, Lighten, os HSL, Erase) o GIMP deixa o *blend
  space* em `auto` — a fixtura grava o que ele ACEITOU (`blend=` na linha `RUN`);
- `BEHIND` é modo só de pincel: o GIMP devolve a camada a `NORMAL` em silêncio (o arnês recusa-o e
  regista-o no cabeçalho);
- pixel de alfa `0` à saída tem a cor indefinida — compara-se só o alfa.
