#!/usr/bin/env python3
"""P14d: o Godot lê o NOSSO heroi.tres (+ folha, _n.png, JSON): abre, sequência a 1 ms, recorte da
normal e sinal da normal. Grava crates/ph2d-pixel-cena/tests/fixtures/oracle_godot_p14d.json.
Uso: PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_p14d.py <pasta>
"""
import hashlib, json, os, pathlib, subprocess, sys

AQUI = pathlib.Path(__file__).resolve().parent
RAIZ = AQUI.parents[2]
SAIDA = RAIZ / "crates/ph2d-pixel-cena/tests/fixtures/oracle_godot_p14d.json"


def linha(txt, chave):
    l = [x for x in txt.splitlines() if x.startswith(chave + " ")]
    assert l, txt[-800:]
    return json.loads(l[0][len(chave) + 1:])


def main():
    d = pathlib.Path(sys.argv[1]).resolve()
    d.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, PH2D_P14D_SAIDA=str(d))
    subprocess.run(["cargo", "test", "-p", "ph2d-pixel-cena", "--profile", "smoke", "--test", "it", "--",
                    "--ignored", "escreve_a_entrada_do_oraculo_p14d"], cwd=RAIZ, env=env, check=True,
                   capture_output=True)
    (d / "project.godot").write_text('[application]\nconfig/name="p14d"\n')
    subprocess.run(["godot", "--headless", "--path", str(d), "--import"], capture_output=True, timeout=120)
    r = subprocess.run(["godot", "--headless", "--path", str(d), "--fixed-fps", "1000", "--script",
                        str(AQUI / "oraculo_godot_p14d.gd")], capture_output=True, text=True, timeout=120)
    x = subprocess.run(["godot", "--headless", "--fixed-fps", "1024", "--script",
                        str(AQUI / "oraculo_godot_p14d_exato.gd")], capture_output=True, text=True, timeout=120)
    s = subprocess.run(["xvfb-run", "-a", "godot", "--rendering-driver", "opengl3", "--path", str(d),
                        "--script", str(AQUI / "oraculo_godot_p14d_sinal.gd")],
                       capture_output=True, text=True, timeout=120)
    v = subprocess.run(["xvfb-run", "-a", "godot", "--rendering-driver", "opengl3", "--path", str(d),
                        "--script", str(AQUI / "oraculo_godot_p14d_pivo.gd")],
                       capture_output=True, text=True, timeout=120)
    SAIDA.write_text(json.dumps({
        "_cabecalho": {
            "oraculo": "Godot 4.7.2 (MIT, pacote godot 4.7.2-2.1); sequencia/abre/recorte --headless "
                       "--fixed-fps 1000; sinal xvfb opengl3 (render real)",
            "comando": "PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_p14d.py <pasta>",
            "data": "2026-11-09",
            "entrada": "heroi_p14 + tags walk3 (0-2 Forward repeat 3) e back2 (3-5 Reverse repeat 2), exportada "
                       "(trim, border 2, shape 1, inner 1, Normal map sheet, Godot .tres); split/ = a mesma com "
                       "Split slices (um .tres por slice)",
            "pivo": "AnimatedSprite2D centered=false, offset = metadata/ph2d_offset, no em (64,64), xvfb opengl3; "
                    "T = caixa dos pixeis acesos do render - caixa do nosso cor<q>; desvio = T + pivo(q) - no",
            "regra": "sequencia[tag][t] = quadro do sprite em t ms (t = 0..2000), lido pela regiao da AtlasTexture",
            "diferenca": "o Godot mostra o quadro seguinte 1 passo DEPOIS da fronteira (convencao: exato mostra "
                         "a troca no passo 65 com 64 passos de 1/64; e o frame_progress e float32: 0.9999994 em "
                         "t=80). O produto segue o Aseprite (troca NA fronteira): o gate compara godot[t] com "
                         "nosso[t-1] — o Godot e fonte da ORDEM e das DURACOES, nao da fase de 1 ms.",
        },
        "entrada": {"tres_sha": hashlib.sha256((d / "heroi.tres").read_bytes()).hexdigest()},
        "abre": linha(r.stdout, "ABRE"),
        "recorte_normal": linha(r.stdout, "RECORTE"),
        "sequencia": linha(r.stdout, "SEQ"),
        "sinal": linha(s.stdout, "SINAL"),
        "apos_play": linha(r.stdout, "APOS_PLAY"),
        "progresso_idle": linha(r.stdout, "PROGRESSO"),
        "exato": linha(x.stdout, "EXATO"),
        "pivo": linha(v.stdout, "PIVO"),
        "split": linha(r.stdout, "SPLIT"),
    }, indent=1))
    print("gravado", SAIDA)


main()
