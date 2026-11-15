# Folha de comparação: fotos e saídas MEDIDAS dos oráculos + maquete da recomendação.
import json, sys
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle
from PIL import Image

P, REPO, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
BG, FG, AZUL, AMBAR, CINZA, VERM = '#0b1020', '#d8dde8', '#4aa3ff', '#f2b84b', '#5a6478', '#ff5a5a'
plt.rcParams.update({'figure.facecolor': BG, 'axes.facecolor': '#111829', 'axes.edgecolor': CINZA,
                     'text.color': FG, 'axes.labelcolor': FG, 'xtick.color': FG, 'ytick.color': FG,
                     'font.size': 11})

bl = json.load(open(f'{P}/blender.json'))
gd = json.load(open(f'{P}/godot.json'))
na = json.load(open(f'{P}/natron.json'))

fig = plt.figure(figsize=(19, 19))
gs = fig.add_gridspec(4, 3, height_ratios=[1, 1, 1.15, 0.75], hspace=0.42, wspace=0.22)


def foto(ax, path, box, titulo):
    im = Image.open(path).convert('RGB').crop(box)
    ax.imshow(im)
    ax.set_xticks([]); ax.set_yticks([])
    ax.set_title(titulo, loc='left', fontsize=12)


# 1 — Blender, foto real
foto(fig.add_subplot(gs[0, 0]), f'{P}/fotos/blender_alem.png', (215, 55, 1310, 790),
     'Blender (foto): alça puxada além do possível\na curva do TEMPO passa do fim e volta')

# 2 — Blender e Godot, o mesmo gesto, medido
ax = fig.add_subplot(gs[0, 1])
u = np.linspace(0, 1, 201)
for c in bl['curvas'][:8]:
    if c['pedido']['saida'] == [0.9, 2]:
        ax.plot(u, c['mapa'], color=VERM, lw=3, label='Blender: alça longa e rápida')
    if c['pedido']['saida'] == [0.5, -1]:
        ax.plot(u, c['mapa'], color=AMBAR, lw=3, label='Blender: alça para baixo')
for c in gd['curvas']:
    if c['pedido'][:2] in ([0.9, 2.0], [0.5, -1.0]):
        ax.plot(u, c['mapa'], color='white', lw=1, ls='--')
ax.plot([], [], color='white', lw=1, ls='--', label='Godot: o mesmo gesto (igual)')
ax.add_patch(Rectangle((0, 0), 1, 1, fill=False, ec=CINZA, ls=':'))
ax.set_xlabel('tempo do trecho'); ax.set_ylabel('onde o movimento está (0 = início, 1 = fim)')
ax.set_title('Blender e Godot (medido): sem limite\no tempo sai do quadrado = anda para trás', loc='left', fontsize=12)
ax.legend(fontsize=9, facecolor=BG, edgecolor=CINZA, loc='lower right')

# 3 — Natron, medido
ax = fig.add_subplot(gs[0, 2])
r = np.array(na['rampa_speed_1_a_3_q1_a_q50']); v = np.array(na['volta_speed_1_a_menos1'])
ax.plot(r[:, 0], r[:, 1], color=VERM, lw=3, label='velocidade 1 → 3')
ax.plot(v[:, 0], v[:, 1], color=AMBAR, lw=3, label='velocidade 1 → −1')
ax.plot([1, 60], [1, 60], color=CINZA, ls=':', label='sem mudar')
ax.axhline(100, color=CINZA, lw=1)
ax.text(2, 102, 'fim do clip', fontsize=9, color=FG)
ax.set_xlabel('quadro mostrado'); ax.set_ylabel('quadro do clip original')
ax.set_title('Natron (medido): velocidade livre, o FIM anda\nacaba no quadro 50 e congela; negativa volta', loc='left', fontsize=12)
ax.legend(fontsize=9, facecolor=BG, edgecolor=CINZA, loc='lower right')

# 4 — o nosso hoje
foto(fig.add_subplot(gs[1, 0:2]), f'{REPO}/docs/Timeline/fotos/retime_6_alca_alem_do_tecto.png', (0, 160, 1120, 335),
     'PH2D hoje (Retime): a alça âmbar fica na curva de VELOCIDADE; o limite é uma curva escondida —\n'
     'perto dele a ponta sai de baixo do rato: encolhe e escorrega para o lado (filmado em 15/11)')

# 5 — a lei comum, o nosso caso
ax = fig.add_subplot(gs[1, 2])
a = np.linspace(0.05, 1, 200)
ax.fill_between(a, 0, np.minimum(1 / a, 12), color=AMBAR, alpha=0.18)
ax.plot(a, np.minimum(1 / a, 12), color=AMBAR, lw=2)
ax.set_ylim(0, 8); ax.set_xlim(0, 1)
ax.set_xlabel('comprimento da alça'); ax.set_ylabel('velocidade na ponta (× a média)')
ax.set_title('Onde a alça pode ir HOJE (vista de velocidade)\no limite é uma CURVA: rápida OU longa', loc='left', fontsize=12)


# 6 — maquete da recomendação
def bez(x1, y1, x2, y2, n=400):
    t = np.linspace(0, 1, n)
    bx = 3 * (1 - t) ** 2 * t * x1 + 3 * (1 - t) * t ** 2 * x2 + t ** 3
    by = 3 * (1 - t) ** 2 * t * y1 + 3 * (1 - t) * t ** 2 * y2 + t ** 3
    return bx, by


ax = fig.add_subplot(gs[2, :])
x0, w, h = 0.0, 1.0, 1.0
ax.add_patch(Rectangle((0, 0), 1, 1, fill=True, fc='#151e33', ec=AMBAR, lw=1.5, ls='--'))
x1, y1, x2, y2 = 0.45, 1.0, 0.75, 0.55   # alça de saída encostada no TOPO do quadrado (o limite)
bx, by = bez(x1, y1, x2, y2)
ax.plot([0, 1], [0, 1], color=CINZA, ls=':', lw=1)
ax.plot(bx, by, color=AMBAR, lw=3)
for (px, py), (kx, ky) in (((x1, y1), (0, 0)), ((x2, y2), (1, 1))):
    ax.plot([kx, px], [ky, py], color=AMBAR, lw=1.2)
    ax.plot(px, py, 'o', ms=11, mfc=AMBAR, mec='white')
ax.plot([0, 1], [0, 1], 'D', ms=10, color='white')
ax.annotate('o rato foi mais para cima:\na alça pára na BORDA, sob o rato\n(mesmo ponto = mesmo resultado)',
            xy=(x1, y1), xytext=(0.06, 1.14), color=FG, fontsize=10,
            arrowprops=dict(arrowstyle='->', color=FG))
ax.plot(x1, 1.25, 'o', ms=10, mfc='none', mec=VERM, mew=2)
ax.text(x1 + 0.025, 1.24, 'rato', color=VERM, fontsize=10, va='center')
# velocidade derivada (só leitura) por baixo
dy = np.gradient(by, bx)
ax.plot(bx, -0.55 + 0.08 * np.clip(dy, 0, 4.5), color=AMBAR, lw=1.5, alpha=0.8)
ax.axhline(-0.55, color=CINZA, lw=0.8)
ax.text(1.02, -0.5, 'velocidade\n(só mostra)', fontsize=9, color=FG, va='center')
ax.text(1.02, 0.5, 'tempo do\ntrecho\n(edita)', fontsize=9, color=AMBAR, va='center')
ax.set_xlim(-0.05, 1.12); ax.set_ylim(-0.65, 1.38); ax.set_xticks([]); ax.set_yticks([])
ax.set_title('RECOMENDAÇÃO (maquete) — como o DaVinci Resolve (curva «Retime Frame») e o editor de easing do CSS:\n'
             'em Retime, a curva âmbar é o TEMPO do trecho, de canto a canto; o quadrado tracejado é o limite visível',
             loc='left', fontsize=12)

# 7 — tabela curta
ax = fig.add_subplot(gs[3, :]); ax.axis('off')
linhas = [('Programa', 'A alça é', 'Além do possível'),
          ('Blender', 'ponto na curva do tempo', 'passa e volta (para trás)'),
          ('Godot', 'ponto na curva do tempo', 'passa e volta (para trás)'),
          ('After Effects (pela documentação)', 'velocidade + influência', 'passa e volta (para trás)'),
          ('Natron', 'velocidade solta', 'o fim do clip anda'),
          ('Premiere / Resolve', 'velocidade do trecho', 'o clip encurta/alonga'),
          ('Resolve (Frame)', 'ponto na curva do tempo', 'você VÊ o limite'),
          ('PH2D hoje', 'velocidade + influência', 'encolhe e escorrega'),
          ('PH2D proposta', 'ponto na curva do tempo', 'pára na borda do quadrado')]
for i, (a_, b_, c_) in enumerate(linhas):
    y = 1 - i * 0.105
    peso = 'bold' if i == 0 or 'proposta' in a_ else 'normal'
    cor = AMBAR if 'proposta' in a_ else FG
    ax.text(0.0, y, a_, fontsize=13, weight=peso, color=cor)
    ax.text(0.30, y, b_, fontsize=13, weight=peso, color=cor)
    ax.text(0.58, y, c_, fontsize=13, weight=peso, color=cor)

fig.savefig(OUT, dpi=80, facecolor=BG, bbox_inches='tight')
print('ok', OUT)
