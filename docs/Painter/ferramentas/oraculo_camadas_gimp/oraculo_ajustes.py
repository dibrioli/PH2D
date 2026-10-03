# Oraculo dos AJUSTES sobre o composto: GIMP 3.2.x corrido sem interface (python-fu-eval), caixa-preta.
# Corre-se por `corre_ajustes.sh` (ao lado). Escreve a fixtura binaria em $ORACULO_SAIDA.
#
# A mesma grelha NOSSA dos modos (entradas.py): base + UMA camada, Normal, opacidade 100, imagem
# FLOAT_NON_LINEAR, espaco PERCEPTUAL (= o compositor do ADR-0177). O ajuste corre sobre o VISIVEL.
# Cada ajuste em que o GIMP tem o espaco como parametro corre DUAS vezes: o espaco da lei
# (perceptual) e o controlo (linear) — a regua tem de distinguir os dois.
# Saida: o visivel ajustado, lido em float straight R'G'B'A, byte = floor(v*255+0,5).
import os
import struct
import gi

gi.require_version('Gimp', '3.0')
gi.require_version('Gegl', '0.4')
from gi.repository import Gimp, Gegl

import sys
sys.path.insert(0, os.getcwd())
from entradas import ALFAS, BASE_ALFAS, BASES, TOPOS, W, H, entradas  # noqa: E402

# Os parametros NOSSOS de cada ajuste (o gate monta o mesmo ajuste no compositor).
# 2 pontos: uma recta nos dois splines. Pontas que NUNCA caem num empate de meio degrau: com
# (0,1 ; 0,9) o byte de 1 em cada 5 entradas era 25,5 + 0,8·k — o arredondamento decidia-se pelo
# ruido de virgula flutuante de cada programa, nao pelo espaco (87 de 4 896 canais a 1, medido).
CURVA = ((0.0, 0.1037), (1.0, 0.9113))
LEVELS = dict(low_input=0.1, high_input=0.9, gamma=1.6, low_output=0.05, high_output=0.95)
POSTERIZE = 4
THRESHOLD = 0.5                            # canal VALUE; o gate so' compara pixeis cinzentos


def camada(img, nome, dados, pos):
    lay = Gimp.Layer.new(img, nome, W, H, Gimp.ImageType.RGBA_IMAGE, 100.0, Gimp.LayerMode.NORMAL)
    img.insert_layer(lay, None, pos)
    buf = lay.get_buffer()
    buf.set(Gegl.Rectangle.new(0, 0, W, H), "R'G'B'A u8", dados)
    buf.flush()
    per = Gimp.LayerColorSpace.RGB_PERCEPTUAL
    lay.set_blend_space(per)
    lay.set_composite_space(per)
    lay.set_composite_mode(Gimp.LayerCompositeMode.UNION)
    return lay


def filtro(vis, op, props):
    f = Gimp.DrawableFilter.new(vis, op, '')
    cfg = f.get_config()
    for k, v in props.items():
        cfg.set_property(k, v)
    f.update()
    vis.merge_filter(f)
    return {k: cfg.get_property(k) for k in props if k != 'curve'}


def ajusta(vis, ajuste, espaco):
    trc = Gimp.TRCType.PERCEPTUAL if espaco == 'perceptual' else Gimp.TRCType.LINEAR
    if ajuste == 'Invert':
        vis.invert(espaco == 'linear')
        return 'linear=%s' % (espaco == 'linear')
    if ajuste == 'Curves':
        f = Gimp.DrawableFilter.new(vis, 'gimp:curves', '')
        cfg = f.get_config()
        cfg.set_property('trc', trc)
        curva = cfg.get_property('curve')
        assert curva.get_n_points() == 2, curva.get_n_points()
        for i, (x, y) in enumerate(CURVA):
            curva.set_point(i, x, y)
        cfg.set_property('curve', curva)
        f.update()
        vis.merge_filter(f)
        return 'trc=%s pontos=%s' % (cfg.get_property('trc').value_nick, CURVA)
    if ajuste == 'Levels':
        aceite = filtro(vis, 'gimp:levels', {
            'trc': trc, 'channel': Gimp.HistogramChannel.VALUE,
            'low-input': LEVELS['low_input'], 'high-input': LEVELS['high_input'],
            'clamp-input': True, 'gamma': LEVELS['gamma'],
            'low-output': LEVELS['low_output'], 'high-output': LEVELS['high_output'],
            'clamp-output': True,
        })
        return 'trc=%s %s' % (aceite['trc'].value_nick, LEVELS)
    if ajuste == 'Posterize':
        filtro(vis, 'gimp:posterize', {'levels': POSTERIZE})
        return 'levels=%d' % POSTERIZE
    if ajuste == 'Threshold':
        # O intervalo do GIMP e' [low, high]: com high = 1 o branco puro cai FORA (medido: 1.0 -> preto).
        aceite = filtro(vis, 'gimp:threshold', {'channel': Gimp.HistogramChannel.VALUE, 'low': THRESHOLD, 'high': 2.0})
        return 'low=%s high=%s canal=VALUE' % (aceite['low'], aceite['high'])
    raise SystemExit('ajuste desconhecido ' + ajuste)


def corre(base, topo, ajuste, espaco):
    img = Gimp.Image.new_with_precision(W, H, Gimp.ImageBaseType.RGB, Gimp.Precision.FLOAT_NON_LINEAR)
    camada(img, 'base', base, 0)
    camada(img, 'topo', topo, 0)
    vis = Gimp.Layer.new_from_visible(img, img, 'visivel')
    img.insert_layer(vis, None, 0)
    aceite = ajusta(vis, ajuste, espaco)
    raw = vis.get_buffer().get(Gegl.Rectangle.new(0, 0, W, H), 1.0, "R'G'B'A float", Gegl.AbyssPolicy.NONE)
    img.delete()
    vals = struct.unpack('<%df' % (W * H * 4), bytes(raw))
    return bytes(int(min(max(v, 0.0), 1.0) * 255.0 + 0.5) for v in vals), aceite


# (ajuste, espacos): o espaco so' e' parametro do GIMP no Invert, Curves e Levels.
CORRIDAS = [
    ('Invert', ['perceptual', 'linear']),
    ('Curves', ['perceptual', 'linear']),
    ('Levels', ['perceptual', 'linear']),
    ('Posterize', ['-']),
    ('Threshold', ['-']),
]

saida = os.environ['ORACULO_SAIDA']
cabecalho = os.environ.get('ORACULO_CABECALHO', '')
base, topo = entradas()
partes = []
for ajuste, espacos in CORRIDAS:
    for espaco in espacos:
        out, aceite = corre(base, topo, ajuste, espaco)
        partes.append(('RUN %s %s %s\n' % (ajuste, espaco, aceite)).encode() + out)

with open(saida, 'wb') as f:
    f.write(cabecalho.encode())
    f.write(('# grelha ....: W=%d H=%d (alfas %s · base_alfas %s · bases %s · topos %s)\n'
             % (W, H, ALFAS, BASE_ALFAS, BASES, TOPOS)).encode())
    f.write(b'#FIM\n')
    f.write(b'BASE\n' + base)
    f.write(b'TOPO\n' + topo)
    for p in partes:
        f.write(p)
print('oraculo: %d corridas -> %s' % (len(partes), saida))
