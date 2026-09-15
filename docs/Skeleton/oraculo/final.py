# A CELULA QUE FALTAVA: mistura melhor (centros de rotacao) POR CIMA de pesos suaves.
import json, math, numpy as np
exec(open('/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/288048cc-7a6f-40b2-8ea6-37cc673393d2/scratchpad/harmonicos.py').read().split('print(f"{\'lei dos pesos\'')[0])

def centros(w0, passo=3, sigma=0.1):
    """c_v = integral de p ponderada pela semelhanca dos vectores de peso (Le & Hodgins 2016)."""
    px = np.array([v[0] for v in V]); py = np.array([v[1] for v in V])
    w0 = np.asarray(w0); w1 = 1.0 - w0
    dx, dy, dw0, dw1 = px[::passo], py[::passo], w0[::passo], w1[::passo]
    cx = np.zeros_like(px); cy = np.zeros_like(py); ok = np.zeros(len(px), dtype=bool)
    for k in range(len(px)):
        t = dw0*w1[k] - dw1*w0[k]
        s = dw0*dw1*w0[k]*w1[k]*np.exp(-(t*t)/(sigma*sigma))
        ss = s.sum()
        if ss > 1e-12:
            cx[k] = (s*dx).sum()/ss; cy[k] = (s*dy).sum()/ss; ok[k] = True
    return cx, cy, ok

def mede_cor(w0, total):
    ms, ang = poses(total)
    w0 = np.asarray(w0); w1 = 1.0 - w0
    px = np.array([v[0] for v in V]); py = np.array([v[1] for v in V])
    def lbs(qx, qy, a, b):
        x = np.zeros_like(qx); y = np.zeros_like(qy)
        for (c, s, dxb, ox, oy), wi in zip(ms, (a, b)):
            rx, ry = qx + dxb, qy
            x += wi*(rx*c - ry*s + ox); y += wi*(rx*s + ry*c + oy)
        return x, y
    cx, cy, ok = centros(w0)
    th = w0*ang[0] + w1*ang[1]
    ccx, ccy = lbs(cx, cy, w0, w1)
    co, si = np.cos(th), np.sin(th)
    rx, ry = px - cx, py - cy
    x = np.where(ok, ccx + rx*co - ry*si, 0.0)
    y = np.where(ok, ccy + rx*si + ry*co, 0.0)
    lx, ly = lbs(px, py, w0, w1)
    x = np.where(ok, x, lx); y = np.where(ok, y, ly)
    inv = tot = 0.0
    for f in F:
        for tri in ((f[0],f[1],f[2]), (f[0],f[2],f[3])):
            i,j,k = tri
            s  = (x[j]-x[i])*(y[k]-y[i]) - (y[j]-y[i])*(x[k]-x[i])
            s0 = (V[j][0]-V[i][0])*(V[k][1]-V[i][1]) - (V[j][1]-V[i][1])*(V[k][0]-V[i][0])
            tot += abs(s0)
            if s*s0 <= 0: inv += abs(s0)
    pt_i = int(np.argmin((px-2*L)**2 + py**2)); sel = px >= L*1.5
    r0 = np.arctan2(py[sel], px[sel]-2*L); r1 = np.arctan2(y[sel]-y[pt_i], x[sel]-x[pt_i])
    da = (r1 - r0 + math.pi) % (2*math.pi) - math.pi
    return 100*inv/tot, math.degrees(float(np.median(da))), int((~ok).sum())

W_H = harmonicos()
W_B = bump(L)
print(f"\n{'lei':>44} | {'dobra 150°':>11} | {'segue o osso':>14}")
print("-"*76)
for nome, w, cor in [("HOJE: bump raio=osso + mistura linear", W_B, False),
                     ("harmonicos + mistura linear", W_H, False),
                     ("bump raio=osso + CENTROS DE ROTACAO", W_B, True),
                     ("harmonicos + CENTROS DE ROTACAO", W_H, True)]:
    if cor:
        inv, a, n = mede_cor(w, math.radians(150))
        extra = f"  ({n} sem centro)"
    else:
        inv, a = mede(w, math.radians(150)); extra = ""
    print(f"{nome:>44} | {inv:9.3f}%  | {a:8.1f}° de 150{extra}")
