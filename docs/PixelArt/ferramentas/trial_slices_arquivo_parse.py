"""P11 F2: le chunks 0x2022/0x2020 de .ase/.aseprite so por parse nosso (spec publica do formato). Sem copiar arte nem bytes.
uso: python3 -I trial_slices_arquivo_parse.py <f.ase> [...]   -> JSON em stdout, um objeto por ficheiro."""
import json, os, struct, sys

NOMES = {0x0004: 'pal_old', 0x0011: 'pal_old2', 0x2004: 'layer', 0x2005: 'cel', 0x2006: 'cel_extra', 0x2007: 'color_profile',
         0x2008: 'ext_files', 0x2016: 'mask', 0x2017: 'path', 0x2018: 'tags', 0x2019: 'palette', 0x2020: 'user', 0x2022: 'slice', 0x2023: 'tileset'}


def s(b, o):
    n = struct.unpack_from('<H', b, o)[0]
    return b[o + 2:o + 2 + n].decode('utf8', 'replace'), o + 2 + n


def user(d):
    fl = struct.unpack_from('<I', d, 0)[0]
    o, u = 4, {'flags': fl}
    if fl & 1:
        u['text'], o = s(d, o)
    if fl & 2:
        u['color'] = list(d[o:o + 4]); o += 4
    if fl & 4:
        u['props_bytes'] = struct.unpack_from('<I', d, o)[0]
    return u


def fatia(d):
    n, fl, _ = struct.unpack_from('<III', d, 0)
    nome, o = s(d, 12)
    keys = []
    for _ in range(n):
        f, x, y, w, h = struct.unpack_from('<IiiII', d, o); o += 20
        k = {'frame': f, 'x': x, 'y': y, 'w': w, 'h': h}
        if fl & 1:
            k['center'] = list(struct.unpack_from('<iiII', d, o)); o += 16
        if fl & 2:
            k['pivot'] = list(struct.unpack_from('<ii', d, o)); o += 8
        keys.append(k)
    return {'name': nome, 'flags': fl, 'keys': keys, 'sobra_bytes': len(d) - o}


def le(path):
    b = open(path, 'rb').read()
    _, magic, frames, w, h, depth = struct.unpack_from('<IHHHHH', b, 0)
    assert magic == 0xA5E0
    o, res, ordem = 128, {'frames': frames, 'canvas': [w, h], 'depth': depth, 'slices': [], 'bytes': len(b)}, []
    for fr in range(frames):
        fsz, m, old, _, new = struct.unpack_from('<IHHH2xI', b, o)
        assert m == 0xF1FA
        nchunks = new or old
        p, kinds, ultima = o + 16, [], None
        for _ in range(nchunks):
            sz, t = struct.unpack_from('<IH', b, p)
            d = b[p + 6:p + sz]
            kinds.append(NOMES.get(t, hex(t)))
            if t == 0x2022:
                ultima = fatia(d)
                ultima['chunk_frame'] = fr
                ultima['user_data'] = None
                ultima['pos_no_quadro'] = len(kinds) - 1
                res['slices'].append(ultima)
            elif t == 0x2020:
                if ultima is not None and kinds[-2] == 'slice':
                    ultima['user_data'] = user(d)
            p += sz
        if any(k == 'slice' for k in kinds):
            ordem.append({'frame': fr, 'chunks': kinds})
        o += fsz
    res['ordem_nos_quadros_com_slice'] = ordem
    return res


if __name__ == '__main__':
    print(json.dumps({os.path.basename(p): le(p) for p in sys.argv[1:]}, ensure_ascii=False))
