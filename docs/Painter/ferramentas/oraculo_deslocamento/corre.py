#!/usr/bin/env python3
"""Oráculo do deslocamento da borda (BUGS_painter #31): o `feDisplacementMap` do Inkscape sobre ENTRADAS NOSSAS.

Gera um disco 128² e um mapa (canais R/G) sintético, pede ao Inkscape para deslocar o disco pelo mapa
(scale = 2·A, A = 12 px), e grava as três imagens como bytes crus (u8, linha a linha) — o gate
`a_lei_do_deslocamento_e_a_do_fedisplacementmap` lê-as com `include_bytes!` e refaz a saída com o
`Mapa::le` do motor. Corra:  python3 corre.py   (precisa de `inkscape` no PATH e de numpy + Pillow).
"""
import subprocess, numpy as np
from PIL import Image

W, A = 128, 12.0
y, x = np.mgrid[0:W, 0:W].astype(float)
disco = (((x - 64) ** 2 + (y - 64) ** 2) < 40 ** 2).astype(np.uint8) * 255
u = np.sin(x / 9.0) * np.cos(y / 13.0)
v = np.cos(x / 11.0 + 1.0) * np.sin(y / 7.0)
R = ((u + 1) / 2 * 255).round().astype(np.uint8)
G = ((v + 1) / 2 * 255).round().astype(np.uint8)
um = np.full_like(disco, 255)
Image.fromarray(np.dstack([disco, disco, disco, um])).save('/tmp/oraculo_disco.png')
Image.fromarray(np.dstack([R, G, np.zeros_like(R), um])).save('/tmp/oraculo_mapa.png')
svg = f'''<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{W}" height="{W}" viewBox="0 0 {W} {W}">
<filter id="f" x="0" y="0" width="{W}" height="{W}" filterUnits="userSpaceOnUse" primitiveUnits="userSpaceOnUse" color-interpolation-filters="sRGB">
<feImage xlink:href="/tmp/oraculo_mapa.png" x="0" y="0" width="{W}" height="{W}" result="m" preserveAspectRatio="none"/>
<feDisplacementMap in="SourceGraphic" in2="m" scale="{2 * A}" xChannelSelector="R" yChannelSelector="G"/>
</filter>
<image xlink:href="/tmp/oraculo_disco.png" x="0" y="0" width="{W}" height="{W}" filter="url(#f)" image-rendering="optimizeSpeed"/>
</svg>'''
open('/tmp/oraculo_caso.svg', 'w').write(svg)
subprocess.run(['inkscape', '/tmp/oraculo_caso.svg', '--export-type=png', '--export-filename=/tmp/oraculo_saida.png',
                f'--export-width={W}', '--export-background=black', '--export-background-opacity=1'],
               check=True, capture_output=True)
saida = np.array(Image.open('/tmp/oraculo_saida.png').convert('L'))
open('disco.u8', 'wb').write(disco.tobytes())
open('mapa_rg.u8', 'wb').write(np.dstack([R, G]).tobytes())
open('saida_inkscape_1.4.4.u8', 'wb').write(saida.astype(np.uint8).tobytes())
print('coberto pelo inkscape (>127):', int((saida > 127).sum()))
