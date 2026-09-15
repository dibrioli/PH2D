import json, math, sys
d = json.load(open(sys.argv[1]))
ARCO, V, F, W_BL = d["arco"], d["verts"], d["faces"], d["pesos"]
L = ARCO / 2.0

def dist2_seg(p, a, b):
    vx, vy = b[0]-a[0], b[1]-a[1]
    wx, wy = p[0]-a[0], p[1]-a[1]
    n2 = vx*vx + vy*vy
    t = 0.0 if n2 <= 0 else max(0.0, min(1.0, (wx*vx + wy*vy)/n2))
    dx, dy = wx - t*vx, wy - t*vy
    return dx*dx + dy*dy

def nossos_pesos(p, raio):
    """A lei de hoje: bump (1-x^2)^2 da distancia ao eixo, normalizado."""
    eixos = [([0,0],[L,0]), ([L,0],[2*L,0])]
    w, soma, perto, pd = [0.0,0.0], 0.0, 0, 1e30
    for i,(a,b) in enumerate(eixos):
        d2 = dist2_seg(p,a,b)
        if d2 < pd: perto, pd = i, d2
        x2 = d2/(raio*raio)
        v = (1.0-x2)**2 if x2 < 1.0 else 0.0
        w[i] = v; soma += v
    if soma > 0: return [v/soma for v in w]
    return [1.0 if i==perto else 0.0 for i in range(2)]

def poses(total_rad):
    """M_i = translate(o_i) . rotate(fi) . translate(-i*L, 0) — a mesma lei do fixture em Rust."""
    ms, ox, oy, fi = [], 0.0, 0.0, 0.0
    for i in range(2):
        if i > 0: fi += total_rad
        c, s = math.cos(fi), math.sin(fi)
        # p -> R*(p - (i*L,0)) + (ox,oy)
        ms.append((c, s, -i*L, ox, oy))
        ox += L*c; oy += L*s
    return ms

def aplica(p, ms, w):
    x = y = 0.0
    for (c,s,dx,ox,oy), wi in zip(ms, w):
        if wi == 0.0: continue
        px, py = p[0]+dx, p[1]
        x += wi*(px*c - py*s + ox)
        y += wi*(px*s + py*c + oy)
    return (x, y)

def dobra(pesos_fn, total_rad):
    ms = poses(total_rad)
    P = [aplica(v, ms, pesos_fn(i, v)) for i, v in enumerate(V)]
    inv = area_inv = area_tot = 0.0
    n_inv = 0
    for f in F:
        for tri in ((f[0],f[1],f[2]), (f[0],f[2],f[3])):
            a,b,c = (P[t] for t in tri)
            ra,rb,rc = (V[t] for t in tri)
            s  = (b[0]-a[0])*(c[1]-a[1]) - (b[1]-a[1])*(c[0]-a[0])
            s0 = (rb[0]-ra[0])*(rc[1]-ra[1]) - (rb[1]-ra[1])*(rc[0]-ra[0])
            area_tot += abs(s0)
            if s*s0 <= 0.0:
                n_inv += 1; area_inv += abs(s0)
    return n_inv, 100.0*area_inv/area_tot

def bl(i, v):
    """Os pesos do oraculo, NORMALIZADOS pela mesma lei que a nossa usa."""
    w = W_BL[i]; s = w[0] + w[1]
    return [w[0]/s, w[1]/s] if s > 0 else [1.0, 0.0]

MEIA_ARTE = 2.4
print(f"{'dobra':>6} | {'NOSSO (raio = osso)':>22} | {'BLENDER (calor auto)':>22} | {'NOSSO (raio 2,08x arte)':>24}")
print("-"*92)
for g in (60, 90, 120, 150):
    r = math.radians(g)
    a = dobra(lambda i,v: nossos_pesos(v, L), r)
    b = dobra(bl, r)
    c = dobra(lambda i,v: nossos_pesos(v, MEIA_ARTE*2.08), r)
    print(f"{g:5}° | {a[0]:5} tri {a[1]:7.3f}% | {b[0]:5} tri {b[1]:7.3f}% | {c[0]:5} tri {c[1]:7.3f}%")
