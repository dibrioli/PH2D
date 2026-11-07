"""P11: conduz a ferramenta Slice do Aseprite Trial 1.3.18.6 por ponteiro e teclado reais (caixa preta; o Trial nao grava: le-se a tela).
uso: trial_slices_tool.py <work> up                 Xvfb :84 1600x1000 (prazo proprio)
     trial_slices_tool.py <work> open <f.ase>       (re)abre o Trial com HOME limpo (screen_scale 2), matando so o PID anterior dele
        o binario e <work>/app/squashfs-root/AppRun (AppImage --appimage-extract): --appimage-extract-and-run partilha /tmp e da «Text file busy» com outro agente a arrancar
     trial_slices_tool.py <work> do 'op; op; ...'   aplica passos na sessao aberta
     trial_slices_tool.py <work> down               fecha Trial e Xvfb pelos PIDs guardados
ops: mv x y | click x y [botao] | dbl x y | drag x0 y0 x1 y1 | shiftdrag.. | kd tecla | ku tecla | key k [k...] | type texto | wait s | shot nome [crop x y w h]
drag assenta 2x antes do largar: o Trial perde o ultimo movimento (P10_oraculos.md M9)."""
import os, shlex, shutil, signal, subprocess, sys, time

APP = os.path.expanduser('~/.local/opt/aseprite-trial/Aseprite-trial.AppImage')
DISP = ':84'


def env(w):
    e = dict(os.environ, DISPLAY=DISP, HOME=os.path.join(w, 'home'), SDL_VIDEODRIVER='x11')
    e.pop('WAYLAND_DISPLAY', None)
    return e


def pidfile(w, n):
    return os.path.join(w, n + '.pid')


def kill(w, n):
    p = pidfile(w, n)
    if os.path.exists(p):
        pid = int(open(p).read())
        try:
            os.killpg(pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        os.remove(p)
        time.sleep(.4)


def spawn(w, n, cmd, e=None, log=None):
    out = open(log, 'w') if log else subprocess.DEVNULL
    p = subprocess.Popen(cmd, env=e, stdout=out, stderr=subprocess.STDOUT, start_new_session=True)
    open(pidfile(w, n), 'w').write(str(p.pid))


def x(e, *a):
    subprocess.run(['xdotool', *map(str, a)], env=e, check=True)


def drag(e, a, b, step=8, btn=1):
    x(e, 'mousemove', *a)
    time.sleep(.12)
    x(e, 'mousemove', a[0] + 1, a[1])
    time.sleep(.08)
    x(e, 'mousemove', *a)
    time.sleep(.12)
    cmd = ['mousedown', btn]
    n = max(1, max(abs(b[0] - a[0]), abs(b[1] - a[1])) // step)
    for s in range(1, n + 1):
        cmd += ['sleep', '0.02', 'mousemove', a[0] + (b[0] - a[0]) * s // n, a[1] + (b[1] - a[1]) * s // n]
    cmd += ['sleep', '0.25', 'mousemove', b[0] + 1, b[1], 'sleep', '0.25', 'mousemove', b[0], b[1], 'sleep', '0.3', 'mouseup', btn]
    x(e, *cmd)
    time.sleep(.3)


OX, OY, PX = 596, 112, 8  # origem do canvas na tela depois de 'setup' (400 %, 8 px de tela por pixel)


def P(cx, cy):
    return (OX + int(cx) * PX + PX // 2, OY + int(cy) * PX + PX // 2)


def do(w, script):
    e = env(w)
    for raw in script.split(';'):
        t = shlex.split(raw.strip())
        if not t:
            continue
        op, a = t[0], t[1:]
        if op == 'mv':
            x(e, 'mousemove', a[0], a[1]); time.sleep(.25)
        elif op == 'click':
            x(e, 'mousemove', a[0], a[1]); time.sleep(.2)
            x(e, 'mousemove', int(a[0]) + 1, a[1]); time.sleep(.08)
            x(e, 'mousemove', a[0], a[1]); time.sleep(.12)
            x(e, 'click', a[2] if len(a) > 2 else 1); time.sleep(.35)
        elif op == 'dbl':
            x(e, 'mousemove', a[0], a[1]); time.sleep(.2)
            x(e, 'click', '--repeat', 2, '--delay', 60, 1); time.sleep(.4)
        elif op == 'drag':
            drag(e, (int(a[0]), int(a[1])), (int(a[2]), int(a[3])))
        elif op == 'setup':
            x(e, 'mousemove', 1537, 723); time.sleep(.2); x(e, 'click', 1); time.sleep(.3)
            x(e, 'key', 'Tab'); time.sleep(.4)
            x(e, 'mousemove', 856, 400); time.sleep(.2); x(e, 'key', '3'); time.sleep(.4)
            x(e, 'keydown', 'space'); time.sleep(.1); drag(e, (856, 500), (856, 200)); x(e, 'keyup', 'space'); time.sleep(.3)
        elif op == 'pdrag':
            drag(e, P(a[0], a[1]), P(a[2], a[3]))
        elif op == 'pclick':
            p = P(a[0], a[1]); x(e, 'mousemove', *p); time.sleep(.2); x(e, 'mousemove', p[0] + 1, p[1]); time.sleep(.08); x(e, 'mousemove', *p); time.sleep(.12)
            x(e, 'click', a[2] if len(a) > 2 else 1); time.sleep(.35)
        elif op == 'pdbl':
            p = P(a[0], a[1]); x(e, 'mousemove', *p); time.sleep(.2); x(e, 'click', '--repeat', 2, '--delay', 60, 1); time.sleep(.4)
        elif op == 'pmv':
            x(e, 'mousemove', *P(a[0], a[1])); time.sleep(.3)
        elif op == 'fill':
            x(e, 'mousemove', a[0], a[1]); time.sleep(.15); x(e, 'click', 1); time.sleep(.15)
            x(e, 'key', 'ctrl+a'); time.sleep(.1); x(e, 'type', '--delay', 40, '--', a[2]); time.sleep(.2)
        elif op == 'frame':
            x(e, 'mousemove', 373 + 24 * int(a[0]), 795); time.sleep(.2); x(e, 'click', 1); time.sleep(.3)
        elif op == 'dlg':  # foto so do miolo do dialogo
            p = os.path.join(w, a[0] + '.png'); time.sleep(.15)
            subprocess.run(['import', '-window', 'root', '-crop', '540x290+530+356', '+repage', p], env=e, check=True)
        elif op == 'dl2':  # so as linhas de numeros
            p = os.path.join(w, a[0] + '.png'); time.sleep(.15)
            subprocess.run(['import', '-window', 'root', '-crop', '540x105+530+470', '+repage', p], env=e, check=True)
        elif op == 'dl3':  # nome + numeros
            p = os.path.join(w, a[0] + '.png'); time.sleep(.15)
            subprocess.run(['import', '-window', 'root', '-crop', '540x190+530+390', '+repage', p], env=e, check=True)
        elif op == 'stack':  # empilha fotos: stack saida a b c
            from PIL import Image
            ims = [Image.open(os.path.join(w, n + '.png')).convert('RGB') for n in a[1:]]
            cols = max(1, int(os.environ.get('COLS', '1')))
            W = max(i.width for i in ims); H = max(i.height for i in ims)
            rows = (len(ims) + cols - 1) // cols
            o = Image.new('RGB', (W * cols, H * rows), (255, 0, 255))
            for k, im in enumerate(ims):
                o.paste(im, ((k % cols) * W, (k // cols) * H))
            o.save(os.path.join(w, a[0] + '.png')); print(os.path.join(w, a[0] + '.png'))
        elif op == 'kd':
            x(e, 'keydown', a[0]); time.sleep(.1)
        elif op == 'ku':
            x(e, 'keyup', a[0]); time.sleep(.1)
        elif op == 'key':
            for k in a:
                x(e, 'key', k); time.sleep(.3)
        elif op == 'type':
            x(e, 'type', '--delay', 40, a[0]); time.sleep(.25)
        elif op == 'wait':
            time.sleep(float(a[0]))
        elif op == 'shot':
            p = os.path.join(w, a[0] + '.png')
            if 'keep' not in a:
                x(e, 'mousemove', 1590, 990); time.sleep(.25)
            time.sleep(.1)
            crop = [v for v in a[1:] if v != 'keep']
            cmd = ['import', '-window', 'root'] + (['-crop', f'{crop[2]}x{crop[3]}+{crop[0]}+{crop[1]}', '+repage'] if crop else []) + [p]
            subprocess.run(cmd, env=e, check=True)
            print(p)
        else:
            raise SystemExit('op? ' + op)


def main():
    w, cmd = os.path.abspath(sys.argv[1]), sys.argv[2]
    os.makedirs(w, exist_ok=True)
    if cmd == 'up':
        spawn(w, 'xvfb', ['timeout', '2600', 'Xvfb', DISP, '-screen', '0', '1600x1000x24', '-nolisten', 'tcp'])
        time.sleep(1.5)
    elif cmd == 'open':
        kill(w, 'app')
        h = os.path.join(w, 'home')
        shutil.rmtree(h, ignore_errors=True)
        os.makedirs(h + '/.config/aseprite')
        open(h + '/.config/aseprite/aseprite.ini', 'w').write('[GfxMode]\nMaximized = false\nFrame = 0 0 1600 1000\n[general]\nscreen_scale = 2\nui_scale = 1\n')
        spawn(w, 'app', ['timeout', '1800', os.path.join(w, 'app/squashfs-root/AppRun'), sys.argv[3]], env(w), os.path.join(w, 'app.log'))
        time.sleep(float(os.environ.get('BOOT', '6')))
    elif cmd == 'do':
        do(w, sys.argv[3])
    elif cmd == 'down':
        kill(w, 'app')
        kill(w, 'xvfb')


if __name__ == '__main__':
    main()
