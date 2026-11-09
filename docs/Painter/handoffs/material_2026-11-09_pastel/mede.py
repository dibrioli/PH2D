# Réguas do «pastel × spray» sobre a cobertura k (0..1) da RETA da coluna vazia (y = 110, x 100..700 na tela cheia).
# Todas por janela x 150..650 (sem as pontas). `e` = escala (1 cheia, 0,5 = a do CSP).
import numpy as np


def janela(k, e=1.0):
    y0, y1 = int(round((110 - 45) * e)), int(round((110 + 45) * e))
    x0, x1 = int(round(150 * e)), int(round(650 * e))
    return k[y0:y1, x0:x1]


def mede(k, e=1.0):
    j = janela(k, e).astype(np.float64)
    prof = j.mean(1)  # perfil transversal (média ao longo do traço)
    pico = prof.max()
    ic = int(prof.argmax())
    acima = np.nonzero(prof >= 0.5 * pico)[0]
    larg50 = (acima.max() - acima.min() + 1) / e if len(acima) else 0
    # borda: distância 10 %→90 % do pico, média dos dois lados (px da tela cheia)
    def lado(p):
        i90 = next(i for i, v in enumerate(p) if v >= 0.9 * pico)
        i10 = max([i for i in range(i90) if p[i] < 0.1 * pico] or [0])
        return (i90 - i10) / e
    borda = (lado(prof[:ic + 1]) + lado(prof[ic:][::-1])) / 2
    # miolo: as linhas com perfil >= 80 % do pico
    mi = j[prof >= 0.8 * pico]
    cob = mi.mean()
    cv = mi.std() / max(mi.mean(), 1e-9)  # irregularidade do corpo
    # pontos (quase) soltos: píxel com QUALQUER tinta (k >= 0,02) com no máximo 2 dos 8 vizinhos com tinta, na faixa do
    # traço (perfil >= 2 % do pico) — os grãozinhos de 1–3 px cercados de papel limpo (0 vizinhos: 0 em todos)
    b = j >= 0.02
    viz = np.zeros_like(b, dtype=np.int32)
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            if dy or dx:
                viz += np.roll(np.roll(b, dy, 0), dx, 1)
    tinta = b & (prof >= 0.02 * pico)[:, None]
    soltos = (tinta & (viz <= 2)).sum() / max(tinta.sum(), 1)
    # buracos do miolo: fracção dos píxeis do miolo com k < 10 % da cobertura média (papel à mostra «a zero»)
    vazio = float((mi < 0.1 * cob).mean())
    # grão: autocorrelação do resíduo no miolo; o passo (px da tela cheia) em que cai abaixo de 1/e
    r = mi - mi.mean(1, keepdims=True)
    def passo(eixo):
        f = np.fft.rfft(r, axis=1) if eixo == 1 else None
        if eixo == 1:
            ac = np.fft.irfft(np.abs(f) ** 2, axis=1).mean(0)
        else:
            rr = r.T
            ac = np.fft.irfft(np.abs(np.fft.rfft(rr, axis=1)) ** 2, axis=1).mean(0)
        ac = ac / ac[0]
        n = next((i for i in range(1, len(ac) // 2) if ac[i] < 1 / np.e), len(ac) // 2)
        return n / e
    return dict(larg50=round(float(larg50), 1), borda=round(float(borda), 1), cob_miolo=round(float(cob), 3),
                cv_miolo=round(float(cv), 2), vazio_miolo=round(vazio, 3), quase_soltos=round(float(soltos), 3), grao_x=round(passo(1), 1),
                grao_y=round(passo(0), 1))


def corr_papel(k, h, e=1.0):
    """Correlação (Pearson) da cobertura com a altura do papel no miolo da reta."""
    j, hj = janela(k, e), janela(h, e)
    prof = j.mean(1)
    m = prof >= 0.8 * prof.max()  # só o MIOLO: fora dele a correlação mede o envelope do traço, não o grão
    return round(float(np.corrcoef(j[m].ravel(), hj[m].ravel())[0, 1]), 2)
