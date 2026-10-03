# Oraculo dos efeitos de VIZINHANCA (P4), 2.a opiniao: Krita sem interface (kritarunner, Qt offscreen),
# caixa-preta. Corre-se por `corre_vizinhanca.sh`. Documento RGBA U8 sRGB (o Krita a 8 bits); a
# entrada NOSSA numa unica camada; o filtro aplicado a ela (Filter.apply). Escreve em $ORACULO_SAIDA.
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from entradas import VW, VH, entrada_vizinhanca  # noqa: E402
from krita import Krita, InfoObject  # noqa: E402

# (efeito, filtro do Krita, propriedades). O raio do 'gaussian blur' do Krita nao e' o sigma: o
# impulso (faixa 4) mede o nucleo; dois raios para ver a lei.
CORRIDAS = [
    ('Gaussian', 'gaussian blur', {'horizRadius': 4, 'vertRadius': 4, 'lockAspect': True}),
    ('Gaussian', 'gaussian blur', {'horizRadius': 9, 'vertRadius': 9, 'lockAspect': True}),
    ('Motion', 'motion blur', {'blurAngle': 0, 'blurLength': 9}),
    # ⚠️ o 'unsharp' pelo API devolve o DESFOQUE simples com qualquer amount/lightnessOnly/threshold
    # (medido: 4 configuracoes, a mesma saida) — o 'sharpen' (nucleo fixo, o impulso mede-o) responde.
    ('Sharpen', 'sharpen', {}),
]


def bgra(rgba):
    out = bytearray(rgba)
    out[0::4], out[2::4] = rgba[2::4], rgba[0::4]
    return bytes(out)


def corre(k, dados, nome, props):
    doc = k.createDocument(VW, VH, 'oraculo', 'RGBA', 'U8', '', 72.0)
    root = doc.rootNode()
    for n in list(root.childNodes()):
        n.remove()
    lay = doc.createNode('entrada', 'paintlayer')
    root.addChildNode(lay, None)
    lay.setPixelData(bgra(dados), 0, 0, VW, VH)
    f = k.filter(nome)
    cfg = f.configuration()
    for kk, v in props.items():
        cfg.setProperty(kk, v)
    f.setConfiguration(cfg)
    f.apply(lay, 0, 0, VW, VH)
    doc.waitForDone()
    aceite = ' '.join('%s=%s' % (kk, f.configuration().property(kk)) for kk in props)
    out = bgra(bytes(lay.pixelData(0, 0, VW, VH)))
    doc.close()
    return out, aceite


def __main__(args):
    saida = os.environ['ORACULO_SAIDA']
    k = Krita.instance()
    dados = entrada_vizinhanca()
    partes = []
    for efeito, nome, props in CORRIDAS:
        out, aceite = corre(k, dados, nome, props)
        partes.append(('RUN %s %s krita-u8 %s\n' % (efeito, nome.replace(' ', '_'), aceite)).encode() + out)
    with open(saida, 'wb') as f:
        f.write(os.environ.get('ORACULO_CABECALHO', '').replace('@VERSAO@', k.version()).encode())
        f.write(('# entrada ...: W=%d H=%d (entradas.py entrada_vizinhanca)\n' % (VW, VH)).encode())
        f.write(b'#FIM\n')
        f.write(b'ENTRADA\n' + dados)
        for p in partes:
            f.write(p)
    with open(saida + '.log', 'w') as f:
        f.write('oraculo krita vizinhanca: %d corridas\n' % len(partes))
