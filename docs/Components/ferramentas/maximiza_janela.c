/* MAXIMIZA a última janela mapeada da tela X indicada — o gesto que o dono descreve.
 *
 * ⛔ Nasceu porque o report *«tela fica em branco ao maximizar»* (2026-09-21) NÃO reproduz
 * abrindo a janela grande: o `fotografa_cena.sh` maximiza no MAPEAMENTO (a regra do kwin), e
 * isso ja' e' um resize que funciona. O que falta e' maximizar uma janela que JA' ESTA' A
 * DESENHAR, que e' outro caminho (surface reconfigurada a meio de um quadro em curso).
 *
 * ⚠️ Nem `xdotool` nem `wmctrl` estao nesta maquina; isto e' o ClientMessage `_NET_WM_STATE`
 * do EWMH, que e' exactamente o que aqueles dois enviam.
 *
 * ⛔⛔ Ele RECUSA o `:0` — a tela do dono. A lei da casa: nenhum instrumento deste repo toca
 * na sessao real dele (foi assim que o `spectacle` fotografou o ecra' dele em 19/09).
 */
#include <X11/Xlib.h>
#include <X11/Xatom.h>
#include <stdio.h>
#include <string.h>

#define _NET_WM_STATE_ADD 1

int main(int argc, char **argv) {
    const char *disp = argc > 1 ? argv[1] : NULL;
    if (disp && (strcmp(disp, ":0") == 0 || strcmp(disp, ":0.0") == 0)) {
        fprintf(stderr, "RECUSA: :0 e' a tela do dono\n");
        return 3;
    }
    Display *d = XOpenDisplay(disp);
    if (!d) { fprintf(stderr, "sem display\n"); return 2; }
    Window root = DefaultRootWindow(d);

    Atom list = XInternAtom(d, "_NET_CLIENT_LIST", False);
    Atom actual; int fmt; unsigned long n = 0, rest; unsigned char *data = NULL;
    if (XGetWindowProperty(d, root, list, 0, 1024, False, XA_WINDOW,
                           &actual, &fmt, &n, &rest, &data) != Success || n == 0) {
        fprintf(stderr, "sem janelas no _NET_CLIENT_LIST\n");
        return 2;
    }
    Window w = ((Window *)data)[n - 1];   /* a ULTIMA mapeada = a do app */
    XFree(data);

    XEvent e;
    memset(&e, 0, sizeof(e));
    e.xclient.type = ClientMessage;
    e.xclient.window = w;
    e.xclient.message_type = XInternAtom(d, "_NET_WM_STATE", False);
    e.xclient.format = 32;
    e.xclient.data.l[0] = _NET_WM_STATE_ADD;
    e.xclient.data.l[1] = (long)XInternAtom(d, "_NET_WM_STATE_MAXIMIZED_VERT", False);
    e.xclient.data.l[2] = (long)XInternAtom(d, "_NET_WM_STATE_MAXIMIZED_HORZ", False);
    e.xclient.data.l[3] = 1; /* fonte: aplicacao normal */
    XSendEvent(d, root, False, SubstructureNotifyMask | SubstructureRedirectMask, &e);
    XFlush(d);
    printf("maximizei a janela 0x%lx\n", (unsigned long)w);
    XCloseDisplay(d);
    return 0;
}
