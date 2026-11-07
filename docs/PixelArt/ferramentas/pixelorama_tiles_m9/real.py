"""M9: confirma por ponteiro REAL (xdotool em Xvfb) o que Tools.handle_draw deu. A extensao em PROBE_MODE=real publica
coords_<caso>.json (origem do canvas em ecra e escala) e espera go_<caso>. uso: python3 -I real.py <out> <display>"""
import json, os, subprocess, sys, time
O, D = sys.argv[1], sys.argv[2]
env = dict(os.environ, DISPLAY=':' + D)
def x(*a): subprocess.run(['xdotool', *map(str, a)], env=env, check=True)
CASOS = {'manual_d': ((10, 5), (13, 5)), 'stack_b': ((2, 11), (5, 11)), 'auto_d': ((10, 5), (13, 5)),
         'auto_q2a': ((6, 1), (9, 1)), 'ctrl_manual_lapis': ((11, 1), (11, 1))}
time.sleep(5)
x('mousemove', 1168, 214, 'click', 1); time.sleep(.5)  # fecha o ecra de abertura, que come o ponteiro
for k, (a, b) in CASOS.items():
    p = f'{O}/coords_{k}.json'
    for _ in range(600):
        if os.path.exists(p) and os.path.getsize(p): break
        time.sleep(.1)
    c = json.load(open(p)); s = c['s']
    S = lambda q: (int(c['o'][0] + (q[0] + .5) * s), int(c['o'][1] + (q[1] + .5) * s))
    A, B = S(a), S(b)
    print(k, c, A, B)
    x('mousemove', A[0] + 2, A[1]); time.sleep(.2); x('mousemove', *A); time.sleep(.3)
    cmd = ['mousedown', '1']; n = max(1, abs(b[0] - a[0]))
    for i in range(1, n + 1):
        cmd += ['sleep', '0.1', 'mousemove', A[0] + (B[0] - A[0]) * i // n, A[1]]
    x(*cmd, 'sleep', '0.2', 'mouseup', '1'); time.sleep(.6)
    subprocess.run(['import', '-window', 'root', f'{O}/real_{k}.png'], env=env)
    open(f'{O}/go_{k}', 'w').write('1')
    time.sleep(.5)
