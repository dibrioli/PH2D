/* ORACULO DA COSTURA — quao suave e' a emenda interna quando um traco que PERDE CARGA volta
 * sobre si mesmo, num motor de pincel de referencia?  (docs/Painter/40, plano da line/PainterWatercolor)
 *
 * Alvo: libmypaint 1.6.1 — licenca ISC (pacman -Qi libmypaint), logo este arnes PODE viver no repo
 * (docs/_ComoInvestigarApps/01_o_arsenal.md §2-bis). Toca SO' o motor; zero GTK, zero janela.
 * As duas armadilhas medidas la' estao honradas: os tiles sao ZERADOS antes de pintar, e a escala
 * do canal e' 65535 = 1,0.
 *
 * ENTRADA NOSSA: um traco em U (desce em x=XA, vira, sobe em x=XB, com XB-XA = 1,2 raios, o regime
 * da foto do dono), com a carga a cair ao longo do arco pela NOSSA lei (1-u)^2. A carga entra pela
 * PRESSAO, mapeada em opaque_multiply — o pincel e' construido aqui, por API, e nao lido de ficheiro,
 * a nao ser que se passe um .myb (os 373 CC0) como argv[1].
 *
 * SAIDA: o perfil de alfa num corte horizontal + a largura 10-90 % da borda EXTERNA e da COSTURA
 * interna, em px e em fraccoes do raio.
 *
 *   gcc -O2 -o /tmp/costura costura.c $(pkg-config --cflags --libs libmypaint) -lm && /tmp/costura
 */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <mypaint-brush.h>
#include <mypaint-fixed-tiled-surface.h>
#include <mypaint-tiled-surface.h>

#define W 512
#define H 512

static double alpha[W * H];

/* largura 10-90 % de uma transicao monotona entre os indices [a,b] do corte (lo->hi ou hi->lo) */
static double width_10_90(const double *row, int a, int b) {
    double va = row[a], vb = row[b];
    double lo = va < vb ? va : vb, hi = va < vb ? vb : va;
    double t10 = lo + 0.1 * (hi - lo), t90 = lo + 0.9 * (hi - lo);
    double x10 = -1, x90 = -1;
    int step = a < b ? 1 : -1;
    /* anda do lado BAIXO para o ALTO com interpolacao linear */
    int from = (va < vb) ? a : b, to = (va < vb) ? b : a; step = from < to ? 1 : -1;
    for (int x = from; x != to; x += step) {
        double p = row[x], q = row[x + step];
        if (x10 < 0 && p <= t10 && q >= t10) x10 = x + step * (t10 - p) / (q - p + 1e-12);
        if (x90 < 0 && p <= t90 && q >= t90) x90 = x + step * (t90 - p) / (q - p + 1e-12);
    }
    return (x10 < 0 || x90 < 0) ? -1.0 : fabs(x90 - x10);
}

int main(int argc, char **argv) {
    /* argv[1] = ficheiro .myb, ou "-" para o pincel construido por API; argv[2] = hardness */
    const char *brush_file = (argc > 1 && argv[1][0] != '-' && argv[1][0] != 0) ? argv[1] : NULL;
    const double R = 32.0;
    float hardness = (argc > 2) ? (float)atof(argv[2]) : 0.5f;

    MyPaintFixedTiledSurface *fs = mypaint_fixed_tiled_surface_new(W, H);
    MyPaintSurface *surf = mypaint_fixed_tiled_surface_interface(fs);
    MyPaintBrush *brush = mypaint_brush_new();
    if (brush_file) {
        FILE *f = fopen(brush_file, "rb");
        if (!f) { fprintf(stderr, "sem pincel: %s\n", brush_file); return 2; }
        fseek(f, 0, SEEK_END); long n = ftell(f); fseek(f, 0, SEEK_SET);
        char *json = malloc(n + 1); if (fread(json, 1, n, f) != (size_t)n) return 4; json[n] = 0; fclose(f);
        if (!mypaint_brush_from_string(brush, json)) { fprintf(stderr, "pincel ilegivel\n"); return 3; }
        free(json);
    } else {
        mypaint_brush_from_defaults(brush);
        mypaint_brush_set_base_value(brush, MYPAINT_BRUSH_SETTING_HARDNESS, hardness);
        /* a CARGA entra pela pressao: opaque_multiply = pressao (0->0, 1->1) */
        mypaint_brush_set_base_value(brush, MYPAINT_BRUSH_SETTING_OPAQUE_MULTIPLY, 0.0f);
        mypaint_brush_set_mapping_n(brush, MYPAINT_BRUSH_SETTING_OPAQUE_MULTIPLY, MYPAINT_BRUSH_INPUT_PRESSURE, 2);
        mypaint_brush_set_mapping_point(brush, MYPAINT_BRUSH_SETTING_OPAQUE_MULTIPLY, MYPAINT_BRUSH_INPUT_PRESSURE, 0, 0.0f, 0.0f);
        mypaint_brush_set_mapping_point(brush, MYPAINT_BRUSH_SETTING_OPAQUE_MULTIPLY, MYPAINT_BRUSH_INPUT_PRESSURE, 1, 1.0f, 1.0f);
    }
    mypaint_brush_set_base_value(brush, MYPAINT_BRUSH_SETTING_RADIUS_LOGARITHMIC, (float)log(R));
    mypaint_brush_set_base_value(brush, MYPAINT_BRUSH_SETTING_COLOR_H, 0.0f);
    mypaint_brush_set_base_value(brush, MYPAINT_BRUSH_SETTING_COLOR_S, 1.0f);
    mypaint_brush_set_base_value(brush, MYPAINT_BRUSH_SETTING_COLOR_V, 1.0f);

    for (int ty = 0; ty < H / MYPAINT_TILE_SIZE; ty++)
      for (int tx = 0; tx < W / MYPAINT_TILE_SIZE; tx++) {
        MyPaintTileRequest z; mypaint_tile_request_init(&z, 0, tx, ty, FALSE);
        mypaint_tiled_surface_tile_request_start((MyPaintTiledSurface*)fs, &z);
        for (int k = 0; k < MYPAINT_TILE_SIZE * MYPAINT_TILE_SIZE * 4; k++) z.buffer[k] = 0;
        mypaint_tiled_surface_tile_request_end((MyPaintTiledSurface*)fs, &z);
      }

    /* O U: ida em XA de Y0 a Y1, curva, volta em XB de Y1 a Y0. Carga (1-u)^2 sobre o arco, span = 2,4 pernas
     * (Charge ~0,2 na nossa lei: a volta chega ao topo quase vazia, como na foto). */
    const double XA = 200.0, XB = XA + 1.2 * R, Y0 = 90.0, Y1 = 420.0;
    const double leg = Y1 - Y0, turn = XB - XA, total = 2 * leg + turn, span = 1.02 * total;
    mypaint_brush_reset(brush); mypaint_brush_new_stroke(brush);
    mypaint_surface_begin_atomic(surf);
    const int N = 1200;
    for (int i = 0; i <= N; i++) {
        double s = total * i / N, x, y;
        if (s < leg) { x = XA; y = Y0 + s; }
        else if (s < leg + turn) { x = XA + (s - leg); y = Y1; }
        else { x = XB; y = Y1 - (s - leg - turn); }
        double left = 1.0 - s / span; if (left < 0) left = 0;
        float p = (float)(left * left);
        mypaint_brush_stroke_to(brush, surf, (float)x, (float)y, p, 0.0f, 0.0f, 1.0 / 240.0);
    }
    MyPaintRectangle roi; mypaint_surface_end_atomic(surf, &roi);

    for (int ty = 0; ty < H / MYPAINT_TILE_SIZE; ty++)
      for (int tx = 0; tx < W / MYPAINT_TILE_SIZE; tx++) {
        MyPaintTileRequest req; mypaint_tile_request_init(&req, 0, tx, ty, TRUE);
        mypaint_tiled_surface_tile_request_start((MyPaintTiledSurface*)fs, &req);
        for (int j = 0; j < MYPAINT_TILE_SIZE; j++) for (int i = 0; i < MYPAINT_TILE_SIZE; i++)
            alpha[(ty * MYPAINT_TILE_SIZE + j) * W + tx * MYPAINT_TILE_SIZE + i] =
                req.buffer[(j * MYPAINT_TILE_SIZE + i) * 4 + 3] / 65535.0;
        mypaint_tiled_surface_tile_request_end((MyPaintTiledSurface*)fs, &req);
      }

    printf("# alvo=libmypaint-1.6.1(ISC) pincel=%s raio=%.0f hardness=%.2f XA=%.0f XB=%.1f (XB-XA=1,2R)\n",
           brush_file ? brush_file : "<api: soft round, opaque_multiply=pressao>", R, hardness, XA, XB);
    const int ys[3] = { (int)(Y0 + 0.10 * leg), (int)(Y0 + 0.50 * leg), (int)(Y0 + 0.90 * leg) };
    const char *nm[3] = { "topo", "meio", "fundo" };
    for (int k = 0; k < 3; k++) {
        const double *row = &alpha[ys[k] * W];
        int xa = (int)XA, xb = (int)XB;
        /* plateau escuro = centro da ida; plateau claro = lado de fora do alcance da ida, dentro da volta */
        int x_dark = xa, x_light = (int)(XA + R + 6.0); if (x_light > xb + (int)R - 4) x_light = xb;
        double outer = width_10_90(row, xa - (int)R - 6, xa);        /* borda externa ESQUERDA (papel -> escuro) */
        double seam  = width_10_90(row, x_dark, x_light);             /* costura interna (escuro -> claro)     */
        double outer_r = width_10_90(row, xb + (int)R + 6, xb);       /* borda externa DIREITA (papel -> claro) */
        printf("%-5s y=%3d  escuro=%.3f claro=%.3f | borda_ext_esq=%6.2fpx (%.3fR)  COSTURA=%6.2fpx (%.3fR)  borda_ext_dir=%6.2fpx (%.3fR)\n",
               nm[k], ys[k], row[x_dark], row[x_light], outer, outer / R, seam, seam / R, outer_r, outer_r / R);
    }
    printf("# perfil (topo), x de XA-40 a XB+40, passo 4:\n");
    { const double *row = &alpha[ys[0] * W];
      for (int x = (int)XA - 40; x <= (int)XB + 40; x += 4) printf("%d:%.3f ", x, row[x]); printf("\n"); }
    return 0;
}
