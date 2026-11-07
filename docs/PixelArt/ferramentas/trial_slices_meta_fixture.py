"""P11: corre o Trial em -b sobre os .ase NOSSOS e grava o ramo meta.slices cru, byte a byte, como fixtura.
uso: python3 -I trial_slices_meta_fixture.py <dir_com_os_ase> <fixtura.json> [--app <appimage extraido/AppRun>]
O Trial recusa imagens mas escreve o JSON de --data (P11_oraculos_arquivo.md F1)."""
import json, os, subprocess, sys, tempfile

CASOS = ['plain', 'keys', 'full', 'hidden', 'late', 'tarde', 'weird', 'c_zero2', 'zeros', 'ud', 'realorder']


def corre(app, ase):
    with tempfile.TemporaryDirectory() as d:
        out, env = os.path.join(d, 'd.json'), dict(os.environ, HOME=d)
        env.pop('DISPLAY', None)
        cmd = [*app, '-b', '--list-slices', ase, '--data', out, '--sheet', os.path.join(d, 's.png')]
        subprocess.run(cmd, env=env, timeout=120, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        with open(out, encoding='utf-8') as f:
            t = f.read()
    return t[t.index('"slices"'):]


def main():
    origem, fixtura = sys.argv[1], sys.argv[2]
    app = [os.path.expanduser('~/.local/opt/aseprite-trial/Aseprite-trial.AppImage'), '--appimage-extract-and-run']
    if '--app' in sys.argv:
        app = [sys.argv[sys.argv.index('--app') + 1]]
    casos = {nome: corre(app, os.path.join(origem, nome + '.ase')) for nome in CASOS}
    assert casos == {nome: corre(app, os.path.join(origem, nome + '.ase')) for nome in CASOS}, 'duas corridas diferem'
    with open(fixtura, 'w', encoding='utf-8') as f:
        json.dump({
            'o_que_e': 'P11: o ramo meta.slices cru, do texto "slices" ao fim do ficheiro, que o Trial escreve para cada .ase nosso.',
            'fonte': 'Aseprite Trial 1.3.18.6-x64-trial, -b --list-slices <ase> --data d.json --sheet s.png; corrido 2x, iguais.',
            'data': '2026-11-07',
            'entradas': 'crates/ph2d-pixel-io/tests/fixtures/slices/*.ase, de docs/PixelArt/ferramentas/trial_slices.py e trial_slices_arquivo.py',
            'casos': casos,
        }, f, ensure_ascii=False, indent=1)
        f.write('\n')


if __name__ == '__main__':
    main()
