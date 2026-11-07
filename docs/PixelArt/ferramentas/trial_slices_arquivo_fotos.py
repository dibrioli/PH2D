"""P11 F2/F3/F4: as sessoes de tela do Trial, todas de uma vez. So fotografa; os .ase sao nossos (e um real, lido pela tela).
uso: python3 -I trial_slices_arquivo_fotos.py <dir_ase> <saida>     (Xvfb :83, ~4 min; o JSON do dialogo cai em <dir_ase> e move-se para <saida>)
Coordenadas: janela 1600x1000, screen_scale 2, canvas 64x64 em (599,263) a 8 px de tela por pixel depois de «move 855 525; key 3»."""
import os, shutil, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import trial_slices_arquivo_ui as U

REAL = '/home/enio/Downloads/ase-para-testar/file-tests-props.aseprite'
Z = 'move 855 525;key shift+c;key 3'
SAIDA_TAB = 'click 878 423;wait 0.5'


def quadros(nome):
    return Z + ''.join(f';shot {nome}_q{q};key Right' for q in range(6))


SESSOES = [(n, quadros(n)) for n in ('keys', 'c_late', 'c_hidden', 'c_tarde', 'c_weird', 'c_zeros', 'c_zero2', 'full', 'hidden', 'late', 'tarde', 'weird')] + [
    ('full', Z + ';dbl 727 391;wait 1;shot full_nine;key Escape;wait 0.4;dbl 920 391;wait 1;shot full_pivot;key Escape;wait 0.4;dbl 727 640;wait 1;shot full_both0;key Escape'),
    ('full', Z + ';key Right Right Right;dbl 950 640;wait 1;shot full_both3;key Escape'),
    ('c_late', Z + ';dbl 727 391;wait 1;shot late_props_q0;key Escape;wait 0.4;click 749 71;wait 0.6;shot late_lista;key Escape;wait 0.3;click 179 71;wait 0.5;shot late_all'),
    ('c_hidden', Z + ';key Right Right;shot hid2;click 749 71;wait 0.6;shot hid_lista;key Escape;wait 0.3;click 179 71;wait 0.5;shot hid_all'),
    ('c_weird', Z + ';dbl 727 583;wait 1;shot w_badcenter;key Escape;wait 0.4;dbl 919 583;wait 1;shot w_badpivot;key Escape;wait 0.4;dbl 615 280;wait 1;shot w_out;key Escape;wait 0.4;dbl 951 359;wait 1;shot w_far;key Escape;wait 0.4;click 749 71;wait 0.6;shot w_lista'),
    ('c_weird', 'move 800 300;key ctrl+alt+shift+s;wait 1.5;click 1020 418;wait 0.5;shot w_area'),
    (REAL, Z + ';shot real0;dbl 919 583;wait 1;shot real_s1;key Escape;wait 0.4;dbl 791 327;wait 1;shot real_s2;key Escape;wait 0.3;click 749 71;wait 0.6;shot real_lista'),
    ('tags', 'move 800 400;key ctrl+e;wait 1.5;shot t_layout;click 749 423;wait 0.5;shot t_sprite;click 923 463;wait 0.4;shot t_source;click 923 463;wait 0.3;click 923 503;wait 0.4;shot t_layers;click 923 503;wait 0.3;click 923 543;wait 0.4;shot t_frames;click 923 543;wait 0.3;click 812 423;wait 0.5;shot t_borders;' + SAIDA_TAB + ';shot t_output;click 537 491;wait 0.8;shot t_json;click 531 459;wait 0.8;shot t_ofile'),
    ('tags', 'move 800 400;click 21 10;click 50 208;wait 0.6;shot menu_export;key Escape;wait 0.3;key Escape;wait 0.3;key ctrl+alt+shift+s;wait 1.5;shot exportfile'),
    ('keys', 'move 800 400;key ctrl+e;wait 1.5;' + SAIDA_TAB + ';click 537 491;wait 0.8;click 892 667;wait 2;shot json_on'),
    ('keys', 'move 800 400;key ctrl+e;wait 1.5;' + SAIDA_TAB + ';click 537 491;wait 0.8;click 1011 533;wait 0.4;click 892 667;wait 2;shot json_off'),
    ('ud', 'move 800 400;key ctrl+e;wait 1.5;' + SAIDA_TAB + ';click 531 459;wait 0.8;click 890 550;wait 2;shot ofile_export'),
    ('full', 'move 800 400;key ctrl+alt+shift+s;wait 1.5;click 873 378;key ctrl+a;type x.png;wait 0.4;click 1020 418;wait 0.4;click 960 486;wait 0.5;shot png_slice;click 846 640;wait 1.5;shot png_notice;click 735 559;wait 2;shot png_recusa'),
]


if __name__ == '__main__':
    a, s = sys.argv[1], sys.argv[2]
    os.makedirs(s, exist_ok=True)
    for f, passos in SESSOES:
        U.sessao(s, f if f.startswith('/') else os.path.join(a, f + '.ase'), passos)
        for n in ('keys', 'ud'):
            j = os.path.join(a, n + '.json')
            if os.path.exists(j):
                shutil.move(j, os.path.join(s, n + '_' + passos.split('shot ')[-1] + '.json'))
