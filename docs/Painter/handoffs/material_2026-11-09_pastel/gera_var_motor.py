# Rodadas 3–4 do Pastel (09/11): as variantes das peças de MOTOR sobre a receita H_d08_j07 — grão fino/médio, maciez do
# Height (`softness`) e a ponta na profundidade (`tip_depth`). Cada papel tem a sua variante `papel_<tex>` (a altura).
# Uso: python3 -I gera_var_motor.py 3 > var_pastel_motor.tsv · python3 -I gera_var_motor.py 4 > var_pastel_motor4.tsv
import json
import sys

D5 = [0.5] * 6


def p(contraste=0.5, detalhe=0.5):
    return [contraste, 0.5, detalhe, 0.5, 0.5, 0.5]


TEX = {  # nome → (kind, size, params = Contrast · Brightness · Detail · Roughness · Warp · —)
    'N48': ('Noise', 48, D5),
    'G48': ('Grain', 48, D5),
    'G32': ('Grain', 32, D5),
    'N16': ('Noise', 16, D5),
    'G64d3': ('Grain', 64, p(detalhe=0.3)),
    'G96d3': ('Grain', 96, p(detalhe=0.3)),
    'N32d3': ('Noise', 32, p(detalhe=0.3)),
    'G64d3c4': ('Grain', 64, p(0.4, 0.3)),
    'G96d3c4': ('Grain', 96, p(0.4, 0.3)),
}
COMUM = ['-spray_count=0', '-dab_jitter=0', 'jitter=0.0']


def textura(tex, **lei):
    kind, size, params = TEX[tex]
    l = {'depth_jitter': 0.7, 'depth_min': 0.0, 'depth_pressure': True, 'mode': 4}
    l.update(lei)
    return 'texture=' + json.dumps({'angle_deg': 0, 'flow': False, 'kind': kind, 'lei': l, 'mapping': 'Tiled',
                                    'offset': [0.0, 0.0], 'params': params, 'rake': False, 'size': [size, size],
                                    'stencil_angle_deg': 0, 'stencil_offset': [0.0, 0.0], 'stencil_size': [0.5, 0.5]},
                                   separators=(',', ':'))


def var(nome, tex='N48', d=0.8, dureza=0.85, extra=(), **lei):
    return '\t'.join(['Pastel', nome, *COMUM, textura(tex, **lei), f'grain_depth={d}', f'hardness={dureza}',
                      'falloff="Smooth"', 'opacity=1.0', *extra])


def papel(tex):
    t = json.loads(textura(tex, depth_jitter=0.0, mode=5)[len('texture='):])
    return '\t'.join(['Pastel', f'papel_{tex}', '-spray_count=0', '-dab_jitter=0', '-mistura_opaca=0', 'jitter=0.0',
                      'texture=' + json.dumps(t, separators=(',', ':')), 'grain_depth=1.0', 'hardness=1.0',
                      'falloff="Sharp"', 'opacity=1.0', 'mao.pressureOpacity=0.0', 'mao.pressureSize=0.0',
                      'color_jitter_sat=0.0'])


def rodada3():
    L = ['# Pastel 09/11 rodada 3: as peças de MOTOR (softness · tip_depth) e o grão fino, sobre H_d08_j07', 'Pastel\thoje']
    L += [papel(t) for t in ('N48', 'G48', 'G32', 'N16')]
    L += [var('H_d08_j07')]
    L += [var(f'{t}', tex=t) for t in ('G48', 'G32', 'N16')]
    L += [var(f'S{int(s * 100):02d}', softness=s) for s in (0.05, 0.1, 0.2)]
    L += [var('T', tip_depth=True), var('T_h06', dureza=0.6, tip_depth=True)]
    L += [var('G48_S10_T', tex='G48', softness=0.1, tip_depth=True),
          var('G48_S10_T_h06', tex='G48', dureza=0.6, softness=0.1, tip_depth=True),
          var('G32_S10_T', tex='G32', softness=0.1, tip_depth=True)]
    return L


def rodada4():
    """Rodada 3 (det3.png): o Krita a meia pressão = tufos de 6–10 px de borda MACIA e cor cheia; o G48 = sal de 1 px
    (oitavas sub-píxel), a maciez 0,2 = a borda macia certa, a ponta 0,6 = a borda quebrada; a opacidade pela pressão
    empalidece o traço inteiro. ⇒ grão médio de 3 oitavas, maciez 0,2–0,3 a d 1, ponta macia, pressureOpacity 0,3."""
    po = ('mao.pressureOpacity=0.3',)
    b = dict(d=1.0, dureza=0.6, extra=po)
    L = ['# Pastel 09/11 rodada 4: grão médio (3 oitavas) · maciez 0,2–0,3 · ponta macia · opacidade pela pressão 0,3',
         'Pastel\thoje']
    L += [papel(t) for t in ('G64d3', 'G96d3', 'N32d3', 'G64d3c4', 'G96d3c4')]
    L += [var('A_G64', tex='G64d3', **b, depth_jitter=0.8, softness=0.2, tip_depth=True),
          var('B_G96', tex='G96d3', **b, depth_jitter=0.8, softness=0.2, tip_depth=True),
          var('C_N32', tex='N32d3', **b, depth_jitter=0.8, softness=0.2, tip_depth=True),
          var('D_G64c', tex='G64d3c4', **b, depth_jitter=0.8, softness=0.3, tip_depth=True),
          var('E_G96c', tex='G96d3c4', **b, depth_jitter=0.8, softness=0.3, tip_depth=True),
          var('F_G64po', tex='G64d3', d=1.0, dureza=0.6, depth_jitter=0.8, softness=0.2, tip_depth=True),
          var('G_G96h75', tex='G96d3', d=1.0, dureza=0.75, extra=po, depth_jitter=0.8, softness=0.2, tip_depth=True),
          var('H_G96semT', tex='G96d3', **b, depth_jitter=0.8, softness=0.2),
          var('I_G96j7', tex='G96d3', **b, depth_jitter=0.7, softness=0.2, tip_depth=True),
          var('J_G96s0', tex='G96d3', **b, depth_jitter=0.8, tip_depth=True)]
    return L


def rodada5():
    """Rodada 4: o Grain (64/96) segue sal de 1 px; o Noise 32 de 3 oitavas tem o grão de 4 px do Krita; contraste 0,4 +
    maciez 0,3 dá os parciais do Krita (0,86); sem `tip_depth` a borda é lisa (quebra 0), com ponta 0,75 a quebra
    0,51 (Krita 0,39); sem maciez o grão é binário. ⇒ junta-se o melhor de cada e afina-se a curva da pressão (o Krita
    sobe de 0,19 a 0,88 entre p 0,35 e 0,65: mais íngreme) — contraste mais baixo estreita as alturas."""
    po = ('mao.pressureOpacity=0.3',)
    L = ['# Pastel 09/11 rodada 5: Noise 3 oitavas · contraste 0,3–0,4 · maciez 0,2–0,3 · ponta 0,75–0,85 · jitter 0,8–0,9',
         'Pastel\thoje']
    for t in ('N32d3c4', 'N32d3c35', 'N32d3c3', 'N24d3c4'):
        TEX.setdefault(t, ('Noise', int(t[1:3]), p(int(t.split('c')[1]) / (10 if len(t.split('c')[1]) == 1 else 100), 0.3)))
    L += [papel(t) for t in ('N32d3c4', 'N32d3c35', 'N32d3c3', 'N24d3c4')]
    L += [var('K1', tex='N32d3c4', d=1.0, dureza=0.75, extra=po, depth_jitter=0.8, softness=0.3, tip_depth=True),
          var('K2', tex='N32d3c35', d=1.0, dureza=0.75, extra=po, depth_jitter=0.9, softness=0.25, tip_depth=True),
          var('K3', tex='N32d3c3', d=1.0, dureza=0.75, extra=po, depth_jitter=0.9, softness=0.2, tip_depth=True),
          var('K4', tex='N24d3c4', d=1.0, dureza=0.75, extra=po, depth_jitter=0.8, softness=0.3, tip_depth=True),
          var('K5', tex='N32d3c4', d=1.0, dureza=0.85, extra=po, depth_jitter=0.9, softness=0.3, tip_depth=True),
          var('K6', tex='N32d3c35', d=1.0, dureza=0.75, extra=('mao.pressureOpacity=0.0',), depth_jitter=0.9,
              softness=0.25, tip_depth=True)]
    return L


print('\n'.join({'3': rodada3, '4': rodada4, '5': rodada5}[sys.argv[1] if len(sys.argv) > 1 else '3']()))
