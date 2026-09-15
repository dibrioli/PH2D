# PESOS HARMONICOS — a lei sem botao de alcance.
#   Resolve  laplaciano(w) = 0  no interior da arte, com w = 1 sobre o osso dele
#   e w = 0 sobre os outros; fronteira exterior LIVRE (fluxo nulo).
# O principio do maximo garante 0 <= w <= 1 e ZERO extremos espurios -- que e'
# exactamente a propriedade cuja ausencia produz a dobra.
import json, math, numpy as np
d = json.load(open('/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/288048cc-7a6f-40b2-8ea6-37cc673393d2/scratchpad/pesos_blender.json'))
V, F, ARCO, N = d["verts"], d["faces"], d["arco"], d["n"]
L, W_BL = ARCO/2, d["pesos"]
nx = ny = N + 1
X = np.array([v[0] for v in V]).reshape(ny, nx)
Y = np.array([v[1] for v in V]).reshape(ny, nx)

def dist_seg(x, y, a, b):
    vx, vy = b[0]-a[0], b[1]-a[1]
    wx, wy = x-a[0], y-a[1]
    n2 = vx*vx + vy*vy
    t = np.clip((wx*vx + wy*vy)/n2, 0.0, 1.0)
    return np.hypot(wx - t*vx, wy - t*vy)

EIXOS = [((0.0,0.0),(L,0.0)), ((L,0.0),(2*L,0.0))]

def harmonicos(nucleo_frac=0.15, iters=4000):
    """w0 harmonico; w1 = 1 - w0 (as condicoes de fronteira somam 1 onde sao postas)."""
    h = nucleo_frac * L
    d0, d1 = dist_seg(X, Y, *EIXOS[0]), dist_seg(X, Y, *EIXOS[1])
    fixo = (d0 <= h) | (d1 <= h)
    val = np.where(d0 <= h, 1.0, 0.0)
    w = np.where(fixo, val, 0.5).astype(np.float64)
    # ⚠️ Gauss-Seidel VERMELHO-PRETO. A 1.ª redacção sobre-relaxava uma iteração de JACOBI (todos os
    # pontos do valor antigo), e isso DIVERGE para ω>1: o resíduo ficou presto em 1,0 e o campo saiu
    # binário — que eu quase reportei como «os pesos harmónicos falham».
    ii, jj = np.meshgrid(np.arange(nx), np.arange(ny))
    cores = [((ii + jj) % 2 == 0), ((ii + jj) % 2 == 1)]
    OMEGA = 1.9
    res = 1.0
    for _ in range(iters):
        for cor in cores:
            up    = np.vstack([w[1:2, :], w[:-1, :]])
            down  = np.vstack([w[1:, :],  w[-2:-1, :]])
            left  = np.hstack([w[:, 1:2], w[:, :-1]])
            right = np.hstack([w[:, 1:],  w[:, -2:-1]])
            novo  = 0.25*(up + down + left + right)
            alvo  = w + OMEGA*(novo - w)
            w = np.where(cor & ~fixo, alvo, w)
        w = np.where(fixo, val, w)
    up    = np.vstack([w[1:2, :], w[:-1, :]]); down = np.vstack([w[1:, :], w[-2:-1, :]])
    left  = np.hstack([w[:, 1:2], w[:, :-1]]); right = np.hstack([w[:, 1:], w[:, -2:-1]])
    res = float(np.max(np.abs(0.25*(up+down+left+right) - w)[~fixo]))
    print(f"   [harmonicos] residuo final {res:.2e}  (min {w.min():.4f} max {w.max():.4f})")
    return w.reshape(-1)

def bump(raio):
    d0, d1 = dist_seg(X, Y, *EIXOS[0]).reshape(-1), dist_seg(X, Y, *EIXOS[1]).reshape(-1)
    def b(dd):
        x2 = (dd/raio)**2
        return np.where(x2 < 1.0, (1.0-x2)**2, 0.0)
    a, c = b(d0), b(d1)
    s = a + c
    return np.where(s > 0, a/np.maximum(s, 1e-30), np.where(d0 <= d1, 1.0, 0.0))

def poses(t):
    ms, ox, oy, fi, ang = [], 0.0, 0.0, 0.0, []
    for i in range(2):
        if i > 0: fi += t
        c, s = math.cos(fi), math.sin(fi)
        ms.append((c, s, -i*L, ox, oy)); ang.append(fi); ox += L*c; oy += L*s
    return ms, ang

def mede(w0, total):
    ms, _ = poses(total)
    w0 = np.asarray(w0); w1 = 1.0 - w0
    px = np.array([v[0] for v in V]); py = np.array([v[1] for v in V])
    x = np.zeros_like(px); y = np.zeros_like(py)
    for (c, s, dx, ox, oy), wi in zip(ms, (w0, w1)):
        qx, qy = px + dx, py
        x += wi*(qx*c - qy*s + ox); y += wi*(qx*s + qy*c + oy)
    inv = tot = 0.0
    for f in F:
        for tri in ((f[0],f[1],f[2]), (f[0],f[2],f[3])):
            i,j,k = tri
            s  = (x[j]-x[i])*(y[k]-y[i]) - (y[j]-y[i])*(x[k]-x[i])
            s0 = (V[j][0]-V[i][0])*(V[k][1]-V[i][1]) - (V[j][1]-V[i][1])*(V[k][0]-V[i][0])
            tot += abs(s0)
            if s*s0 <= 0: inv += abs(s0)
    # fidelidade: a arte sobre o 2.º osso roda quanto?
    pt_i = int(np.argmin((px-2*L)**2 + py**2))
    sel = px >= L*1.5
    r0 = np.arctan2(py[sel], px[sel]-2*L)
    r1 = np.arctan2(y[sel]-y[pt_i], x[sel]-x[pt_i])
    da = (r1 - r0 + math.pi) % (2*math.pi) - math.pi
    return 100*inv/tot, math.degrees(float(np.median(da)))

print(f"{'lei dos pesos':>34} | {'dobra 150°':>11} | {'segue o osso':>14} | {'peso 0 ou 1':>12}")
print("-"*82)
casos = [("HOJE: bump, raio = osso", bump(L)),
         ("bump, raio 2,08x a arte", bump(2.4*2.08)),
         ("BLENDER (calor automatico)", np.array([w[0]/max(1e-12, w[0]+w[1]) for w in W_BL])),
         ("HARMONICOS (sem botao)", harmonicos())]
for nome, w0 in casos:
    inv, ang = mede(w0, math.radians(150))
    degen = int(np.sum((w0 <= 1e-9) | (w0 >= 1.0-1e-9)))
    print(f"{nome:>34} | {inv:9.3f}%  | {ang:8.1f}° de 150 | {degen:5} ({100*degen/len(w0):4.1f}%)")
