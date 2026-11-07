"""P11: junta batch.json (Trial) + parse dos reais + a leitura da tela numa fixture so de numeros, sem arte de terceiros.
uso: python3 -I trial_slices_arquivo_fixture.py <batch.json> <batch2.json> <f2.json> <dir_ase> <saida.json>
Confere por assert o que o doc afirma: ordem invertida, folha igual com --split-slices/--slice, reais iguais ao JSON."""
import collections, json, os, sys

b1, b2, f2 = [json.load(open(p)) for p in sys.argv[1:4]]
ASE, SAIDA = sys.argv[4], sys.argv[5]
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_slices_arquivo_parse as P


def sem_seg(v):
    return {k: x for k, x in v.items() if k != 'seg'}


assert set(b1) == set(b2)
diferentes = [c for c in b1 if sem_seg(b1[c]) != sem_seg(b2[c])]
assert not diferentes, diferentes


def meta_slices(c):
    return b1[c]['meta']['slices']


FIX = ['plain', 'keys', 'full', 'hidden', 'late', 'weird', 'tarde', 'ud', 'tags', 'transp', 'zeros', 'realorder', 'c_late', 'c_hidden', 'c_tarde', 'c_weird', 'c_zeros', 'c_zero2']
fixturas = {}
for n in FIX:
    arq = P.le(os.path.join(ASE, n + '.ase'))
    js = meta_slices(f'meta_{n}')
    assert [s['name'] for s in js] == [s['name'] for s in arq['slices']][::-1], n        # JSON = inverso da ordem no ficheiro
    assert b1[f'meta_{n}']['frames'] == b1[f'split_{n}']['frames'], n                    # --split-slices nao muda a folha
    fixturas[n] = {'slices_no_ficheiro': [s['name'] for s in arq['slices']], 'meta_slices_do_trial': js}

for c in ('slice_moves', 'x_slice_depois', 'slice_inexistente'):
    assert b1[c]['frames'] == b1['meta_keys']['frames'], c                               # --slice nao muda a folha

reais = {}
for c, nome in (('meta_real_props', 'file-tests-props.aseprite'), ('meta_real_app', 'appElements.aseprite'), ('meta_real_icons', 'icons.aseprite')):
    r = f2[nome]
    js = meta_slices(c)
    S = r['slices']
    assert [s['name'] for s in js] == [s['name'] for s in S][::-1], c
    for a, b in zip(js, S[::-1]):
        k = b['keys'][0]
        assert a['keys'][0]['bounds'] == {x: k[x] for x in 'xywh'}, c
        if 'center' in k:
            assert list(a['keys'][0]['center'].values()) == k['center'], c
    reais[nome] = {
        'frames': r['frames'], 'canvas': r['canvas'], 'depth': r['depth'], 'n_slices': len(S),
        'chunk_em_quadro': dict(collections.Counter(str(s['chunk_frame']) for s in S)),
        'flags': dict(collections.Counter(str(s['flags']) for s in S)),
        'keys_por_slice': dict(collections.Counter(str(len(s['keys'])) for s in S)),
        'quadro_das_keys': dict(collections.Counter(str(k['frame']) for s in S for k in s['keys'])),
        'keys_escondidas': sum(1 for s in S for k in s['keys'] if not k['w'] or not k['h']),
        'user_data_logo_a_seguir': sum(1 for s in S if s['user_data']),
        'user_data_flags': dict(collections.Counter(str(s['user_data']['flags']) for s in S)),
        'com_cor': sum(1 for s in S if 'color' in s['user_data']),
        'com_texto': sum(1 for s in S if 'text' in s['user_data']),
        'nomes_padrao_slice_n': sum(1 for s in S if s['name'].startswith('Slice ')),
        'nomes_repetidos': len(S) - len({s['name'] for s in S}),
        'ordem_no_quadro': sorted({k for o in r['ordem_nos_quadros_com_slice'] for k in o['chunks']}),
        'slices_antes_dos_cels': all(o['chunks'].index('slice') < o['chunks'].index('cel') for o in r['ordem_nos_quadros_com_slice']),
        'slices_depois_das_camadas': all(max(i for i, k in enumerate(o['chunks']) if k == 'layer') < o['chunks'].index('slice') for o in r['ordem_nos_quadros_com_slice']),
        'parse_igual_ao_json_do_trial': True,
    }

casos = {c: {k: b1[c].get(k) for k in ('argv', 'exit', 'recusas', 'stdout', 'ficheiros')} | {
    'n_frames': len(b1[c]['frames']) if 'frames' in b1[c] else None,
    'frames_amostra': [{'nome': f['nome'], 'frame': f['frame'], 'sss': f['sss'], 'trimmed': f['trimmed']} for f in b1[c]['frames'][:3]] if 'frames' in b1[c] else None,
    'meta_size': b1[c]['meta']['size'] if 'meta' in b1[c] else None,
    'meta_keys': list(b1[c]['meta']) if 'meta' in b1[c] else None,
} for c in b1 if not c.startswith(('meta_real', 'meta_c_', 'split_c_'))}

tela = {
    '_como': 'rect = x,y,w,h do contorno lido na foto a 8 px de tela por pixel; quadro = barra vermelha da arte. 2 corridas de fotos: 0 pixeis diferentes em c_late, c_hidden, c_tarde, c_weird, c_zeros, full (6 quadros cada), c_zero2 (Q0, Q3) e 7 dialogos/listas comparados.',
    'keys': {'0': [8, 8, 16, 16], '1': [8, 8, 16, 16], '2': [24, 8, 16, 16], '3': [24, 8, 16, 16], '4': [24, 24, 32, 16], '5': [24, 24, 32, 16]},
    'c_late': {str(q): [8, 8, 16, 16] for q in range(6)},
    'c_hidden': {'0': [8, 8, 16, 16], '1': [8, 8, 16, 16], '2': None, '3': None, '4': [32, 32, 16, 16], '5': [32, 32, 16, 16]},
    'c_tarde': {'0': [8, 8, 16, 16], '1': [8, 8, 16, 16], '2': [8, 8, 16, 16], '3': [32, 8, 16, 16], '4': [32, 8, 16, 16], '5': [32, 8, 16, 16]},
    'c_weird_unsorted': {'0': [4, 44, 8, 8], '1': [4, 44, 8, 8], '2': [4, 44, 8, 8], '3': [44, 44, 8, 8], '4': [44, 44, 8, 8], '5': [44, 44, 8, 8]},
    'c_weird_farframe_key_99': {str(q): [40, 8, 8, 8] for q in range(6)},
    'c_zero2_q0_desenhados': ['one_px', 'center_zero', 'w0_later'],
    'c_zero2_q3_desenhados': ['one_px', 'center_zero'],
    'sem_cor_nao_desenhados': ['late', 'hidden', 'tarde', 'weird', 'zeros', 'ud.text_only', 'ud.no_ud', 'ud.quo"te\\'],
    'fora_do_canvas': 'contorno cortado no canvas: nada desenhado fora (out, past)',
    'propriedades_lidas': {'full.nine.q0': {'bounds': [4, 4, 24, 24], 'nine': [8, 8, 8, 8], 'pivot': None}, 'full.pivot.q0': {'bounds': [36, 4, 24, 24], 'nine': None, 'pivot': [12, 24]},
                           'full.both.q0': {'bounds': [4, 36, 24, 20], 'nine': [4, 4, 16, 12], 'pivot': [0, 0]}, 'full.both.q3': {'bounds': [36, 36, 16, 20], 'nine': [2, 6, 12, 8], 'pivot': [8, 10]},
                           'c_late.q0': {'bounds': [8, 8, 16, 16]}, 'c_weird.badcenter': {'bounds': [8, 32, 16, 16], 'nine': [12, 12, 20, 20]}, 'c_weird.badpivot': {'bounds': [32, 32, 16, 16], 'pivot': [-5, 99]},
                           'c_weird.out': {'bounds': [-8, -8, 24, 24]}, 'c_weird.farframe': {'bounds': [40, 8, 8, 8]}},
    'listas_ordenadas_por_nome': {'barra_da_ferramenta': ['badcenter', 'badpivot', 'farframe', 'out', 'past', 'twin', 'twin', 'unsorted'], 'export_area': ['Canvas', 'Selection', 'Slice: badcenter', 'Slice: badpivot', 'Slice: farframe', 'Slice: out', 'Slice: past', 'Slice: twin', 'Slice: twin', 'Slice: unsorted']},
}

dialogo = {
    'export_sprite_sheet': {
        'abas': {'Layout': ['Sheet Type', 'Constraints', 'Merge Duplicates', 'Ignore Empty', 'Size=x²'],
                 'Sprite': ['Source: Sprite|Sprite Grid|Tilesets', 'Layers + Split Layers', 'Frames + Split Tags'],
                 'Borders': ['Border Padding', 'Spacing', 'Inner Padding', 'Trim Sprite', 'Trim Cels', 'Extrude'],
                 'Output': ['Output File', 'JSON Data', 'Hash|Array', 'Meta: Layers Tags Slices', 'Item Filename {title} {frame}.{extension}', 'Item Tagname {tag}']},
        'split_slices': 'nao existe', 'dropdown_de_slice': 'nao existe',
        'meta_slices': 'aba Output, so aparece com JSON Data ligado; ligado por omissao em HOME isolado',
        'botao_export': 'cinzento com Output File e JSON Data desligados',
        'json_data_export': 'escreve o .json (o Trial nao bloqueia); status «Sprite Sheet Generated»',
        'output_file_export': 'status «Sprite Sheet Generated», nenhum ficheiro no disco',
    },
    'export_file': {'campos': ['Output File', 'Resize', 'Area', 'Layers', 'Frames', 'Animation Direction', 'Export for Twitter', 'Ignore empty frames'],
                    'area': 'Canvas, Selection, Slice: <nome>...', 'png_com_slice': 'aviso «Do you want to export the animation in 6 files? x1.png, x2.png...»; Agree: Console «Save operation is not supported in trial version.»'},
}

saida = {
    'o_que_e': 'P11, medicao de oraculo de Slices: ficheiro, linha de comando e exportacao. So numeros e nomes das nossas fixturas; arte e nomes de terceiros ficam de fora.',
    'fonte': 'Aseprite Trial 1.3.18.6-x64-trial, corrido como caixa preta (EULA: nada copiado, nenhum codigo lido). Entradas: .ase nossos, docs/PixelArt/ferramentas/trial_slices.py + trial_slices_arquivo.py (spec publica).',
    'data': '2026-11-07',
    'versao_da_ferramenta': b1['meta_keys']['meta']['version'],
    'comando': 'python3 -I docs/PixelArt/ferramentas/trial_slices_arquivo.py correr <dir_ase> <saida>',
    'corridas_batch': 2,
    'corridas_iguais': True,
    'recusa_do_trial': 'stdout: «Save operation is not supported in trial version.» + link; exit 0; PNG e .ase nao saem; o JSON de --data sai.',
    'leis_confirmadas_por_assert': ['meta.slices = inverso da ordem no ficheiro', '--split-slices nao muda frames da folha em nenhuma fixtura', '--slice nao muda frames da folha', 'parse nosso = JSON do Trial nos 3 ficheiros reais'],
    'por_fixtura': fixturas,
    'casos_batch': casos,
    'tela': tela,
    'dialogos': dialogo,
    'ficheiros_reais_forma': reais,
}
json.dump(saida, open(SAIDA, 'w', encoding='utf8'), indent=1, ensure_ascii=False)
print('ok', os.path.getsize(SAIDA), 'bytes', len(casos), 'casos')
