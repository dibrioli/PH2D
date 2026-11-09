# As SIMULAÇÕES do conserto (retoque de imagem; nada no motor): partem da cobertura k do NOSSO Pastel.
import numpy as np

W_LIM = 0.12  # largura (em altura do papel 0..1) da transição tinta/vale


def gauss(a, s):
    """Desfoque gaussiano por FFT (sem scipy), bordas periódicas — longe das bordas da cena."""
    fy = np.fft.fftfreq(a.shape[0])[:, None]
    fx = np.fft.rfftfreq(a.shape[1])[None, :]
    g = np.exp(-2 * (np.pi * s) ** 2 * (fx ** 2 + fy ** 2))
    return np.fft.irfft2(np.fft.rfft2(a) * g, s=a.shape)


def papel_sintetico(h, w):
    """Papel tipo Cold Press (NOSSO, sintético, semente 9): dente ~2–3 px + ondulação ~8 px; altura em ranking 0..1."""
    r = np.random.default_rng(9)
    n1, n2 = r.standard_normal((h, w)), r.standard_normal((h, w))
    d = gauss(n1, 1.0) - gauss(n1, 4.0)
    o = gauss(n2, 3.0) - gauss(n2, 10.0)
    a = d / d.std() + 0.4 * o / o.std()
    rk = np.empty(a.size)
    rk[np.argsort(a, axis=None)] = np.linspace(0, 1, a.size)
    return rk.reshape(h, w)


def envelope(k):
    """Quanta tinta o nosso põe em cada sítio, sem o grão (desfoque 4 px)."""
    return np.clip(gauss(k, 4.0), 0, 1)


def sim1(k, papel):
    """A MESMA tinta, mas o papel decide ONDE: cobre os picos mais altos até a fracção E (cobertura conservada)."""
    E = envelope(k)
    s = np.clip((papel - (1 - E)) / W_LIM + 0.5, 0, 1) * np.clip(E / 0.04, 0, 1)
    return np.clip(s * k.sum() / max(s.sum(), 1e-9), 0, 1)


def sim2(k, papel):
    """Corpo contínuo à moda do Chalk Soft: o envelope levado a ~0,9 no miolo, modulado pela altura do papel."""
    E = envelope(k)
    En = np.clip(E / max(np.percentile(E[80:140, 150:650], 90), 1e-6), 0, 1)
    return np.clip(En ** 0.8 * np.clip((papel - (1 - En) * 0.55) / 0.35, 0, 1), 0, 1) * 0.92


def sim3(k, papel=None):
    """CONTROLO: o mesmo spray com o grão engrossado (desfoque 1,2 px, a mesma tinta) — o papel NÃO decide."""
    s = gauss(k, 1.2)
    return np.clip(s * k.sum() / max(s.sum(), 1e-9), 0, 1)
