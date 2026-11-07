"""P11 F2/F3/F4: abre .ase no Trial 1.3.18.6 sob Xvfb :83 (1600x1000), HOME isolado, e fotografa. So se le pela tela.
uso (biblioteca): u = Ui(saida); u.abrir(ficheiro); u.tecla('shift+c'); u.foto('nome'); u.fechar()
CLI: python3 -I trial_slices_arquivo_ui.py quadros <saida> <f.ase> <zoom_tecla> [quadros]    fotos de cada quadro com a Slice tool ativa."""
import os, shutil, subprocess, sys, time

APP = os.path.expanduser('~/.local/opt/aseprite-trial/Aseprite-trial.AppImage')
DISP = ':83'


class Ui:
    def __init__(self, saida):
        self.d = saida
        os.makedirs(os.path.join(saida, 'home', '.config', 'aseprite'), exist_ok=True)
        ini = os.path.expanduser('~/.config/aseprite/aseprite.ini')
        if os.path.exists(ini):
            alvo = os.path.join(saida, 'home', '.config', 'aseprite', 'aseprite.ini')
            linhas, em_gfx = [], False
            for l in open(ini, encoding='utf8', errors='replace'):
                if l.startswith('['):
                    em_gfx = l.startswith('[GfxMode]')
                    linhas.append(l)
                    if em_gfx:      # o ini do dono guarda uma janela maior que o Xvfb: a barra de estado e o timeline ficavam fora (medido)
                        linhas.append('Maximized = false\nFrame = 0 0 1600 1000\n')
                elif not em_gfx:
                    linhas.append(l)
            open(alvo, 'w', encoding='utf8').write(''.join(linhas))
        self.env = {k: v for k, v in os.environ.items() if k != 'WAYLAND_DISPLAY'}
        self.env.update(DISPLAY=DISP, HOME=os.path.join(saida, 'home'))
        self.xv = subprocess.Popen(['Xvfb', DISP, '-screen', '0', '1600x1000x24', '-nolisten', 'tcp'], stderr=subprocess.DEVNULL)
        time.sleep(1.5)
        self.ap = None

    def abrir(self, f, espera=6):
        for _ in range(6):
            self.ap = subprocess.Popen(['timeout', '600', APP, '--appimage-extract-and-run', f], env=self.env,
                                       stdout=open(os.path.join(self.d, 'app.log'), 'a'), stderr=subprocess.STDOUT)
            time.sleep(espera)
            if self.ap.poll() is None:
                return
        raise RuntimeError('Trial nao abriu')

    def x(self, *a):
        subprocess.run(['xdotool'] + [str(v) for v in a], env=self.env, check=False)

    def tecla(self, *ks, espera=0.3):
        for k in ks:
            self.x('key', k)
            time.sleep(espera)

    def clique(self, x, y, btn=1, n=1, espera=0.5):
        self.x('mousemove', x, y)
        time.sleep(0.15)
        for _ in range(n):
            self.x('click', btn)
            time.sleep(0.08)
        time.sleep(espera)

    def mover(self, x, y, espera=0.3):
        self.x('mousemove', x, y)
        time.sleep(espera)

    def foto(self, nome):
        p = os.path.join(self.d, nome + '.png')
        subprocess.run(['import', '-window', 'root', p], env=self.env, check=True)
        return p

    def fechar_app(self):
        if self.ap:
            self.ap.terminate()
            try:
                self.ap.wait(5)
            except subprocess.TimeoutExpired:
                self.ap.kill()
            self.ap = None

    def fechar(self):
        self.fechar_app()
        self.xv.terminate()
        self.xv.wait(5)


def sessao(d, f, passos):
    """passos, separados por ';': key K | click X Y | dbl X Y | rclick X Y | move X Y | drag X0 Y0 X1 Y1 | wait S | type T | shot NOME"""
    u = Ui(d)
    try:
        u.abrir(f)
        for p in passos.split(';'):
            a = p.split()
            if not a:
                continue
            c = a[0]
            if c == 'key':
                u.tecla(*a[1:])
            elif c == 'click':
                u.clique(a[1], a[2])
            elif c == 'dbl':
                u.mover(a[1], a[2]); u.x('click', '--repeat', 2, '--delay', 60, 1); time.sleep(0.6)
            elif c == 'rclick':
                u.clique(a[1], a[2], btn=3)
            elif c == 'move':
                u.mover(a[1], a[2])
            elif c == 'drag':
                u.mover(a[1], a[2]); u.x('mousedown', 1); time.sleep(0.2)
                u.mover(a[3], a[4]); u.mover(a[3], a[4]); u.x('mouseup', 1); time.sleep(0.4)
            elif c == 'wait':
                time.sleep(float(a[1]))
            elif c == 'type':
                u.x('type', ' '.join(a[1:]))
            elif c == 'shot':
                u.foto(a[1])
            else:
                raise ValueError(p)
    finally:
        u.fechar()


if __name__ == '__main__':
    cmd, d, f = sys.argv[1:4]
    if cmd == 'sessao':
        sessao(d, f, sys.argv[4])
        sys.exit(0)
    u = Ui(d)
    try:
        u.abrir(f)
        u.foto('0_aberto')
        u.mover(800, 500)
        u.tecla('shift+c')
        u.tecla(sys.argv[4])
        n = int(sys.argv[5]) if len(sys.argv) > 5 else 6
        base = os.path.splitext(os.path.basename(f))[0]
        for q in range(n):
            u.foto(f'{base}_q{q}')
            u.tecla('Right')
    finally:
        u.fechar()
