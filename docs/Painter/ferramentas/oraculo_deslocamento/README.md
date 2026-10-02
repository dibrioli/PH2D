# Oráculo do deslocamento da borda — o `feDisplacementMap` do Inkscape

**Pergunta:** a lei com que o Ragged Edge (Flow) e o Paper Edge deslocam a silhueta da aguada é a lei
padrão de um mapa de deslocamento? (BUGS_painter #31, 2026-10-02.)

**Triagem de licença (passo 1, §0.9):** `pacman -Qi inkscape` ⇒ **GPL / LGPL**, versão **1.4.4**. É
CORRIDO como oráculo sobre entradas nossas; o fonte não foi lido. A lei é a do SVG 1.1 (W3C,
`feDisplacementMap`): `P'(x, y) = P(x + s·(XC(x, y) − ½), y + s·(YC(x, y) − ½))`.

| ficheiro | o que é |
|---|---|
| [`corre.py`](corre.py) | o arnês: disco 128², mapa R/G sintético (senos), `scale = 2·A` com `A = 12 px` |
| `disco.u8` · `mapa_rg.u8` | as entradas (u8 cru, linha a linha; o mapa intercala R e G) |
| `saida_inkscape_1.4.4.u8` | a fixtura: a saída do Inkscape 1.4.4, cinzento u8 |

**O que ele respondeu (2026-10-02):** a lei «a borda no texel `p` LÊ a cobertura em `p + A·u(p)`,
bilinear» reproduz o Inkscape em **todos** os texels (0 de 4 678 em desacordo a 0,5; erro médio 0,0).
O controlo com o sinal trocado (`p − A·u`) discorda em 2 373 — a régua distingue. O gate
`a_lei_do_deslocamento_e_a_do_fedisplacementmap` (`watercolor_flow.rs`) refaz a saída com o `Mapa::le`
e o `sample_bilinear` do motor.
