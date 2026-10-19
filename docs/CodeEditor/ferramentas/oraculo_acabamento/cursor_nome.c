/* cursor_nome — imprime o cursor do X actual: NOME (XFixes), largura x altura, hotspot e um hash
 * FNV-1a dos pixels ARGB (o `import -window root` NÃO fotografa o cursor; isto é a régua dele).
 * (XFixesGetCursorImage da libXfixes ≥ 2 usa o pedido X_XFixesGetCursorImageAndName.)
 * Oráculo «acabamento» do CodeEditor (Ctrl + rato parado → mão). Corre só num Xvfb privado.
 * gcc -O2 -o cursor_nome cursor_nome.c -lX11 -lXfixes ;  DISPLAY=:118 ./cursor_nome [cursor.pam] */
#include <stdio.h>
#include <stdint.h>
#include <X11/Xlib.h>
#include <X11/extensions/Xfixes.h>

int main(int argc, char **argv) {
    Display *d = XOpenDisplay(NULL);
    if (!d) { fprintf(stderr, "sem display\n"); return 2; }
    int ev, er;
    if (!XFixesQueryExtension(d, &ev, &er)) { fprintf(stderr, "sem XFixes\n"); return 2; }
    XFixesCursorImage *c = XFixesGetCursorImage(d);  /* v2+: o pedido é GetCursorImageAndName */
    if (!c) { fprintf(stderr, "sem cursor\n"); return 2; }
    uint64_t h = 1469598103934665603ULL;
    for (int i = 0; i < c->width * c->height; i++) {
        uint32_t p = (uint32_t)c->pixels[i];
        for (int b = 0; b < 4; b++) { h ^= (p >> (8 * b)) & 0xff; h *= 1099511628211ULL; }
    }
    if (argc > 1) { /* grava a imagem do cursor como PAM RGBA (a prova visual do hash) */
        FILE *f = fopen(argv[1], "wb");
        if (f) {
            fprintf(f, "P7\nWIDTH %u\nHEIGHT %u\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n", c->width, c->height);
            for (int i = 0; i < c->width * c->height; i++) {
                uint32_t p = (uint32_t)c->pixels[i];
                unsigned char q[4] = { (p >> 16) & 0xff, (p >> 8) & 0xff, p & 0xff, (p >> 24) & 0xff };
                fwrite(q, 1, 4, f);
            }
            fclose(f);
        }
    }
    printf("nome=%s %ux%u hot=%u,%u hash=%016llx\n",
           (c->name && c->name[0]) ? c->name : "(vazio)", c->width, c->height,
           c->xhot, c->yhot, (unsigned long long)h);
    XFree(c);
    XCloseDisplay(d);
    return 0;
}
