# Oraculo das camadas: GIMP 3.2.x corrido sem interface (python-fu-eval), caixa-preta.
# Corre-se por `corre.sh` (ao lado). Escreve a fixtura binaria em $ORACULO_SAIDA.
#
# Entradas NOSSAS (nada do GIMP): uma base e UMA camada por cima, 17 x 72 px.
#   coluna x  -> alfa da camada = ALFAS[x] (rampa 0..255)
#   linha  y  -> (alfa da base, cor da base, cor da camada) = produto BASE_ALFAS x BASES x TOPOS
# Cada corrida: um modo x um espaco (blend = composite) x uma opacidade, composicao UNION.
# Saida: o visivel lido em float straight R'G'B'A, arredondado a byte (floor(v*255+0,5)).
import os
import gi

gi.require_version('Gimp', '3.0')
gi.require_version('Gegl', '0.4')
from gi.repository import Gimp, Gegl

ALFAS = [0, 16, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 255]
BASE_ALFAS = [255, 140]
BASES = [(0, 0, 0), (255, 255, 255), (128, 128, 128), (51, 51, 51), (230, 60, 30), (40, 90, 200)]
TOPOS = [(0, 0, 0), (255, 255, 255), (191, 191, 191), (70, 70, 70), (30, 200, 80), (240, 180, 20)]
W = len(ALFAS)
H = len(BASE_ALFAS) * len(BASES) * len(TOPOS)

# O nome NOSSO (ph2d_blend_mode::BlendMode, a ordem do enum) -> o modo do GIMP de nome igual.
# Os 4 HSL e o Soft Light tem FORMULA diferente no GIMP: a comparacao deles e' a P2 (doc 45 §6).
MODOS = [
    ('Normal', 'NORMAL'), ('Multiply', 'MULTIPLY'), ('Darken', 'DARKEN_ONLY'),
    ('ColorBurn', 'BURN'), ('LinearBurn', 'LINEAR_BURN'), ('Lighten', 'LIGHTEN_ONLY'),
    ('Screen', 'SCREEN'), ('ColorDodge', 'DODGE'), ('Add', 'ADDITION'),
    ('Overlay', 'OVERLAY'), ('SoftLight', 'SOFTLIGHT'), ('HardLight', 'HARDLIGHT'),
    ('VividLight', 'VIVID_LIGHT'), ('LinearLight', 'LINEAR_LIGHT'),
    ('Difference', 'DIFFERENCE'), ('Exclusion', 'EXCLUSION'),
    ('Hue', 'HSV_HUE'), ('Saturation', 'HSV_SATURATION'), ('Color', 'HSL_COLOR'),
    ('Luminosity', 'LUMINANCE'), ('Behind', 'BEHIND'), ('Clear', 'ERASE'),
]
ESPACOS = [('perceptual', 'RGB_PERCEPTUAL'), ('linear', 'RGB_LINEAR')]
OPACIDADES = [100, 60]


def entradas():
    base = bytearray()
    topo = bytearray()
    for ba in BASE_ALFAS:
        for b in BASES:
            for t in TOPOS:
                for a in ALFAS:
                    base += bytes([b[0], b[1], b[2], ba])
                    topo += bytes([t[0], t[1], t[2], a])
    return bytes(base), bytes(topo)


def camada(img, nome, dados, modo, espaco, opacidade, pos):
    lay = Gimp.Layer.new(img, nome, W, H, Gimp.ImageType.RGBA_IMAGE, float(opacidade), modo)
    img.insert_layer(lay, None, pos)
    buf = lay.get_buffer()
    buf.set(Gegl.Rectangle.new(0, 0, W, H), "R'G'B'A u8", dados)
    buf.flush()
    lay.set_mode(modo)
    lay.set_blend_space(espaco)
    lay.set_composite_space(espaco)
    lay.set_composite_mode(Gimp.LayerCompositeMode.UNION)
    lay.set_opacity(float(opacidade))
    return lay


def corre(base, topo, modo, espaco, opacidade):
    img = Gimp.Image.new_with_precision(W, H, Gimp.ImageBaseType.RGB, Gimp.Precision.FLOAT_NON_LINEAR)
    camada(img, 'base', base, Gimp.LayerMode.NORMAL, espaco, 100, 0)
    lay = camada(img, 'topo', topo, modo, espaco, opacidade, 0)
    feito = (lay.get_mode(), lay.get_blend_space(), lay.get_composite_space(), lay.get_composite_mode())
    vis = Gimp.Layer.new_from_visible(img, img, 'visivel')
    raw = vis.get_buffer().get(Gegl.Rectangle.new(0, 0, W, H), 1.0, "R'G'B'A float", Gegl.AbyssPolicy.NONE)
    img.delete()
    import struct
    vals = struct.unpack('<%df' % (W * H * 4), bytes(raw))
    out = bytes(int(min(max(v, 0.0), 1.0) * 255.0 + 0.5) for v in vals)
    return out, feito


saida = os.environ['ORACULO_SAIDA']
cabecalho = os.environ.get('ORACULO_CABECALHO', '')
base, topo = entradas()
partes = []
recusados = set()
for nosso, gimp_nome in MODOS:
    modo = getattr(Gimp.LayerMode, gimp_nome)
    for espaco_nome, espaco_gimp in ESPACOS:
        espaco = getattr(Gimp.LayerColorSpace, espaco_gimp)
        for op in OPACIDADES:
            out, feito = corre(base, topo, modo, espaco, op)
            # O blend space fica AUTO onde o modo nao tem funcao de mistura (Normal): grava-se o que
            # o GIMP ACEITOU; so' o composite space recusado invalida a corrida.
            if feito[0] != modo:
                # Modo so' de PINCEL no GIMP (Behind, Erase): a camada volta a NORMAL. Sem oraculo.
                recusados.add('%s(%s->%s)' % (nosso, gimp_nome, feito[0].value_nick))
                continue
            if feito[2] != espaco:
                raise SystemExit('o GIMP recusou %s/%s: %r' % (nosso, espaco_nome, feito))
            nick = lambda e: e.value_nick
            partes.append(('RUN %s %s %s %d blend=%s composite=%s\n' % (
                nosso, gimp_nome, espaco_nome, op, nick(feito[1]), nick(feito[3]))).encode() + out)

with open(saida, 'wb') as f:
    f.write(cabecalho.encode())
    f.write(('# grelha ....: W=%d H=%d (alfas %s · base_alfas %s · bases %s · topos %s)\n'
             % (W, H, ALFAS, BASE_ALFAS, BASES, TOPOS)).encode())
    f.write(('# sem oraculo: %s — o GIMP devolve a camada a NORMAL (modo so\' de pincel)\n'
             % ', '.join(sorted(recusados))).encode())
    f.write(b'#FIM\n')
    f.write(b'BASE\n' + base)
    f.write(b'TOPO\n' + topo)
    for p in partes:
        f.write(p)
print('oraculo: %d corridas -> %s' % (len(partes), saida))
