/* O oráculo libmypaint (ISC) da BATERIA do EXAME — o motor do MyPaint 2.x a pintar os NOSSOS gestos.
 *
 * Lê comandos de stdin (escritos por mypaint_bateria.py a partir de bateria.json):
 *   tela W H                 superfície W×H, tiles 64, zerada (transparente)
 *   pincel <ficheiro.myb>    carrega o pincel (JSON v3) — reset do estado
 *   hsv h s v                base_value color_h/s/v (0..1)
 *   raio_log r               base_value radius_logarithmic (= ln(size_px/2))
 *   traco N dt               seguem N linhas «x y pressao»; dt = segundos por evento
 *   limpa                    zera a superfície
 *   foto <ficheiro.raw>      despeja W×H×4 uint16 pré-multiplicado (1.0 = 1<<15), linha a linha
 *
 * Superfície própria: MyPaintTiledSurface2 (a do MyPaint 2.x: pigmento espectral, `paint_mode`), sobre um
 * buffer calloc — a fixed_tiled_surface nasce com lixo (docs/_ComoInvestigarApps/01_o_arsenal.md §2-bis).
 * Pedidos de tile fora da tela vão para um tile descartável zerado a cada pedido.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <mypaint-brush.h>
#include <mypaint-tiled-surface.h>

#define TS 64

typedef struct {
    MyPaintTiledSurface2 parent;
    int w, h, ntx, nty;
    guint16 *tiles;    /* ntx*nty tiles de TS*TS*4 */
    guint16 *descarte; /* TS*TS*4 */
} Superficie;

static void pede_tile(MyPaintTiledSurface2 *t, MyPaintTileRequest *r) {
    Superficie *s = (Superficie *)t;
    if (r->tx < 0 || r->ty < 0 || r->tx >= s->ntx || r->ty >= s->nty) {
        memset(s->descarte, 0, sizeof(guint16) * TS * TS * 4);
        r->buffer = s->descarte;
        return;
    }
    r->buffer = s->tiles + (size_t)(r->ty * s->ntx + r->tx) * TS * TS * 4;
}

static void devolve_tile(MyPaintTiledSurface2 *t, MyPaintTileRequest *r) {
    (void)t;
    (void)r;
}

static void destroi(MyPaintSurface *surf) {
    Superficie *s = (Superficie *)surf;
    mypaint_tiled_surface2_destroy(&s->parent);
    free(s->tiles);
    free(s->descarte);
    free(s);
}

static Superficie *superficie_nova(int w, int h) {
    Superficie *s = calloc(1, sizeof(Superficie));
    mypaint_tiled_surface2_init(&s->parent, pede_tile, devolve_tile);
    s->parent.parent.parent.destroy = destroi;
    s->parent.threadsafe_tile_requests = FALSE; /* determinismo: um fio só */
    s->w = w;
    s->h = h;
    s->ntx = (w + TS - 1) / TS;
    s->nty = (h + TS - 1) / TS;
    s->tiles = calloc((size_t)s->ntx * s->nty * TS * TS * 4, sizeof(guint16));
    s->descarte = calloc(TS * TS * 4, sizeof(guint16));
    return s;
}

static void foto(Superficie *s, const char *caminho) {
    FILE *f = fopen(caminho, "wb");
    if (!f) {
        perror(caminho);
        exit(3);
    }
    for (int y = 0; y < s->h; y++) {
        int ty = y / TS, py = y % TS;
        for (int tx = 0; tx < s->ntx; tx++) {
            int n = TS;
            if (tx * TS + n > s->w) n = s->w - tx * TS;
            guint16 *lin = s->tiles + ((size_t)(ty * s->ntx + tx) * TS * TS + (size_t)py * TS) * 4;
            fwrite(lin, sizeof(guint16), (size_t)n * 4, f);
        }
    }
    fclose(f);
}

static char *le_ficheiro(const char *caminho) {
    FILE *f = fopen(caminho, "rb");
    if (!f) {
        perror(caminho);
        exit(3);
    }
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = malloc((size_t)n + 1);
    if (fread(buf, 1, (size_t)n, f) != (size_t)n) exit(3);
    buf[n] = 0;
    fclose(f);
    return buf;
}

int main(void) {
    Superficie *s = NULL;
    MyPaintBrush *b = mypaint_brush_new();
    mypaint_brush_from_defaults(b);
    char linha[4096];
    while (fgets(linha, sizeof linha, stdin)) {
        char cmd[32] = {0};
        if (sscanf(linha, "%31s", cmd) != 1) continue;
        if (!strcmp(cmd, "tela")) {
            int w, h;
            sscanf(linha + 4, "%d %d", &w, &h);
            if (s) mypaint_surface_unref((MyPaintSurface *)s);
            s = superficie_nova(w, h);
        } else if (!strcmp(cmd, "pincel")) {
            char *nome = linha + 7;
            nome[strcspn(nome, "\n")] = 0;
            char *json = le_ficheiro(nome);
            mypaint_brush_unref(b);
            b = mypaint_brush_new();
            mypaint_brush_from_defaults(b);
            if (!mypaint_brush_from_string(b, json)) {
                fprintf(stderr, "pincel ilegivel: %s\n", nome);
                return 4;
            }
            free(json);
        } else if (!strcmp(cmd, "hsv")) {
            float h, sat, v;
            sscanf(linha + 3, "%f %f %f", &h, &sat, &v);
            mypaint_brush_set_base_value(b, MYPAINT_BRUSH_SETTING_COLOR_H, h);
            mypaint_brush_set_base_value(b, MYPAINT_BRUSH_SETTING_COLOR_S, sat);
            mypaint_brush_set_base_value(b, MYPAINT_BRUSH_SETTING_COLOR_V, v);
        } else if (!strcmp(cmd, "raio_log")) {
            float r;
            sscanf(linha + 8, "%f", &r);
            mypaint_brush_set_base_value(b, MYPAINT_BRUSH_SETTING_RADIUS_LOGARITHMIC, r);
        } else if (!strcmp(cmd, "traco")) {
            int n;
            double dt;
            sscanf(linha + 5, "%d %lf", &n, &dt);
            MyPaintSurface2 *sf = (MyPaintSurface2 *)s;
            mypaint_brush_reset(b);
            mypaint_brush_new_stroke(b);
            mypaint_surface_begin_atomic((MyPaintSurface *)sf);
            for (int i = 0; i < n; i++) {
                float x, y, p;
                if (!fgets(linha, sizeof linha, stdin) || sscanf(linha, "%f %f %f", &x, &y, &p) != 3) {
                    fprintf(stderr, "traco curto\n");
                    return 5;
                }
                mypaint_brush_stroke_to_2(b, sf, x, y, p, 0.0f, 0.0f, dt, 1.0f, 0.0f, 0.0f);
            }
            MyPaintRectangle r;
            MyPaintRectangles rs = {1, &r};
            mypaint_surface2_end_atomic(sf, &rs);
        } else if (!strcmp(cmd, "limpa")) {
            memset(s->tiles, 0, sizeof(guint16) * (size_t)s->ntx * s->nty * TS * TS * 4);
        } else if (!strcmp(cmd, "foto")) {
            char *nome = linha + 5;
            nome[strcspn(nome, "\n")] = 0;
            foto(s, nome);
            printf("ok %s\n", nome);
            fflush(stdout);
        } else {
            fprintf(stderr, "comando desconhecido: %s\n", cmd);
            return 2;
        }
    }
    mypaint_brush_unref(b);
    if (s) mypaint_surface_unref((MyPaintSurface *)s);
    return 0;
}
