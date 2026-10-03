# Oraculo dos efeitos de VIZINHANCA (P4): GIMP 3.2.x corrido sem interface (python-fu-eval), caixa-preta.
# Corre-se por `corre_vizinhanca.sh` (ao lado). Escreve a fixtura binaria em $ORACULO_SAIDA.
#
# A entrada NOSSA (entradas.py `entrada_vizinhanca`) numa UNICA camada; o efeito corre sobre ela por
# Gimp.DrawableFilter (o caminho do menu Filtros). Nenhum destes filtros tem `trc` no GIMP: a pergunta
# «em que espaco borra?» responde-se pela SAIDA, contra os dois modelos (luz e ecra), no gate Rust.
# Cada corrida em duas precisoes da imagem (8 bits perceptual e float perceptual): o filtro e' o mesmo,
# o armazenamento nao.
import os
import struct
import gi

gi.require_version('Gimp', '3.0')
gi.require_version('Gegl', '0.4')
from gi.repository import Gimp, Gegl

import sys
sys.path.insert(0, os.getcwd())
from entradas import VW, VH, entrada_vizinhanca  # noqa: E402

PRECISOES = [('u8', Gimp.Precision.U8_NON_LINEAR), ('float', Gimp.Precision.FLOAT_NON_LINEAR)]

# (efeito, op do GEGL, propriedades). Os enums vao por TEXTO (o inteiro e' ignorado em silencio — a
# 1.a corrida ficou em 'auto'): filter 'fir' (o nucleo discreto, sem a aproximacao IIR), abyss-policy
# 'clamp' (a borda do nosso `sample_clamp`).
CORRIDAS = [
    ('Gaussian', 'gegl:gaussian-blur', {'std-dev-x': 1.5, 'std-dev-y': 1.5, 'filter': 'fir', 'abyss-policy': 'clamp'}),
    ('Gaussian', 'gegl:gaussian-blur', {'std-dev-x': 3.0, 'std-dev-y': 3.0, 'filter': 'fir', 'abyss-policy': 'clamp'}),
    ('Motion', 'gegl:motion-blur-linear', {'length': 9.0, 'angle': 0.0}),
    ('Sharpen', 'gegl:unsharp-mask', {'std-dev': 1.5, 'scale': 0.5, 'threshold': 0.0}),
    ('Bloom', 'gegl:bloom', {'threshold': 50.0, 'softness': 25.0, 'radius': 4.0, 'strength': 50.0}),
]


def corre(dados, prec, op, props):
    img = Gimp.Image.new_with_precision(VW, VH, Gimp.ImageBaseType.RGB, prec)
    lay = Gimp.Layer.new(img, 'entrada', VW, VH, Gimp.ImageType.RGBA_IMAGE, 100.0, Gimp.LayerMode.NORMAL)
    img.insert_layer(lay, None, 0)
    buf = lay.get_buffer()
    buf.set(Gegl.Rectangle.new(0, 0, VW, VH), "R'G'B'A u8", dados)
    buf.flush()
    f = Gimp.DrawableFilter.new(lay, op, '')
    cfg = f.get_config()
    for k, v in props.items():
        cfg.set_property(k, v)
    f.update()
    lay.merge_filter(f)
    aceite = ' '.join('%s=%s' % (k, getattr(cfg.get_property(k), 'value_nick', cfg.get_property(k)))
                      for k in props)
    raw = lay.get_buffer().get(Gegl.Rectangle.new(0, 0, VW, VH), 1.0, "R'G'B'A float", Gegl.AbyssPolicy.NONE)
    img.delete()
    vals = struct.unpack('<%df' % (VW * VH * 4), bytes(raw))
    return bytes(int(min(max(v, 0.0), 1.0) * 255.0 + 0.5) for v in vals), aceite


saida = os.environ['ORACULO_SAIDA']
dados = entrada_vizinhanca()
partes = []
for efeito, op, props in CORRIDAS:
    for nome, prec in PRECISOES:
        out, aceite = corre(dados, prec, op, props)
        partes.append(('RUN %s %s %s %s\n' % (efeito, op, nome, aceite)).encode() + out)

with open(saida, 'wb') as f:
    f.write(os.environ.get('ORACULO_CABECALHO', '').encode())
    f.write(('# entrada ...: W=%d H=%d (entradas.py entrada_vizinhanca)\n' % (VW, VH)).encode())
    f.write(b'#FIM\n')
    f.write(b'ENTRADA\n' + dados)
    for p in partes:
        f.write(p)
print('oraculo: %d corridas -> %s' % (len(partes), saida))
