# Oraculo das camadas, 2.a opiniao: Krita corrido sem interface (kritarunner, Qt offscreen),
# caixa-preta. Corre-se por `corre_krita.sh`. Mesma grelha do GIMP (entradas.py), documento RGBA
# U8 sRGB: o Krita a 8 bits compoe no espaco CODIFICADO. Escreve a fixtura em $ORACULO_SAIDA.
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from entradas import ALFAS, BASE_ALFAS, BASES, TOPOS, W, H, entradas  # noqa: E402
from krita import Krita  # noqa: E402

# O nome NOSSO -> os modos do Krita que dizem sê-lo (onde ha mais de uma formula, todas).
MODOS = [
    ('Normal', 'normal'), ('Multiply', 'multiply'), ('Darken', 'darken'),
    ('ColorBurn', 'burn'), ('LinearBurn', 'linear_burn'), ('Lighten', 'lighten'),
    ('Screen', 'screen'), ('ColorDodge', 'dodge'), ('Add', 'linear_dodge'), ('Add', 'add'),
    ('Overlay', 'overlay'), ('SoftLight', 'soft_light_svg'), ('SoftLight', 'soft_light'),
    ('HardLight', 'hard_light'), ('VividLight', 'vivid_light'), ('LinearLight', 'linear light'),
    ('LinearLight', 'linear_light'), ('Difference', 'diff'), ('Exclusion', 'exclusion'),
    ('Hue', 'hue'), ('Hue', 'hue_hsl'), ('Saturation', 'saturation'),
    ('Saturation', 'saturation_hsl'), ('Color', 'color'), ('Color', 'color_hsl'),
    ('Luminosity', 'luminize'), ('Luminosity', 'lightness'), ('Behind', 'behind'),
    ('Clear', 'erase'), ('Clear', 'clear'),
]
OPACIDADES = [100, 60]


def bgra(rgba):
    out = bytearray(rgba)
    out[0::4], out[2::4] = rgba[2::4], rgba[0::4]
    return bytes(out)


def corre(k, base, topo, modo, op):
    doc = k.createDocument(W, H, 'oraculo', 'RGBA', 'U8', '', 72.0)
    root = doc.rootNode()
    for n in list(root.childNodes()):
        n.remove()
    b = doc.createNode('base', 'paintlayer')
    root.addChildNode(b, None)
    b.setPixelData(bgra(base), 0, 0, W, H)
    t = doc.createNode('topo', 'paintlayer')
    root.addChildNode(t, b)
    t.setPixelData(bgra(topo), 0, 0, W, H)
    t.setBlendingMode(modo)
    t.setOpacity(round(op * 255 / 100))
    aceite = t.blendingMode()
    doc.refreshProjection()
    doc.waitForDone()
    out = bgra(bytes(doc.pixelData(0, 0, W, H)))
    doc.close()
    return out, aceite


def __main__(args):
    saida = os.environ['ORACULO_SAIDA']
    k = Krita.instance()
    base, topo = entradas()
    partes, recusados = [], []
    # ⚠️ O Krita ACEITA qualquer id (o `blendingMode()` devolve-o) e pinta Normal em silencio
    # (medido com um id inventado): um modo cuja saida e' a do Normal nao existe.
    normal = {op: corre(k, base, topo, 'normal', op)[0] for op in OPACIDADES}
    for nosso, kid in MODOS:
        for op in OPACIDADES:
            out, aceite = corre(k, base, topo, kid, op)
            if aceite != kid or (kid != 'normal' and out == normal[op]):
                recusados.append('%s(%s)' % (nosso, kid))
                break
            partes.append(('RUN %s %s krita-u8 %d\n' % (nosso, kid.replace(' ', '_'), op)).encode()
                          + out)
    with open(saida, 'wb') as f:
        f.write(os.environ.get('ORACULO_CABECALHO', '').replace('@VERSAO@', k.version()).encode())
        f.write(('# grelha ....: W=%d H=%d (alfas %s · base_alfas %s · bases %s · topos %s)\n'
                 % (W, H, ALFAS, BASE_ALFAS, BASES, TOPOS)).encode())
        f.write(('# sem oraculo: %s — id desconhecido do Krita (pinta Normal em silencio)\n'
                 % ', '.join(recusados)).encode())
        f.write(b'#FIM\n')
        f.write(b'BASE\n' + base)
        f.write(b'TOPO\n' + topo)
        for p in partes:
            f.write(p)
    with open(saida + '.log', 'w') as f:
        f.write('oraculo krita: %d corridas, recusados %s\n' % (len(partes), recusados))
