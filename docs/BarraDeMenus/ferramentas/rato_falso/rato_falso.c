// Rato falso para a tela VIRTUAL do KWin (org_kde_kwin_fake_input). Uso, com WAYLAND_DISPLAY da tela virtual:
//   rato_falso "move 800 20" "down" "sleep 100" "move 900 120" "up" ...
// Sem argumentos lê um passo por linha da entrada: o dispositivo vive a sessão inteira (cada
// ligação nova cria e apaga o rato do assento, e o app perdia o ponteiro a meio de um arrasto).
// ⛔ Recusa o socket do dono (wayland-0): nunca mexe no rato real.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <wayland-client.h>
#include "fake-input-client.h"

static struct org_kde_kwin_fake_input *fi;
static void reg(void *d, struct wl_registry *r, uint32_t name, const char *iface, uint32_t v) {
  (void)d;
  if (!strcmp(iface, org_kde_kwin_fake_input_interface.name))
    fi = wl_registry_bind(r, name, &org_kde_kwin_fake_input_interface, v < 4 ? v : 4);
}
static void unreg(void *d, struct wl_registry *r, uint32_t n) { (void)d; (void)r; (void)n; }
static const struct wl_registry_listener L = {reg, unreg};

int main(int argc, char **argv) {
  const char *s = getenv("WAYLAND_DISPLAY");
  if (!s || !strcmp(s, "wayland-0") || !strcmp(s, "wayland-1")) {
    fprintf(stderr, "RECUSA: WAYLAND_DISPLAY=%s pode ser o do dono\n", s ? s : "(nada)");
    return 3;
  }
  struct wl_display *dpy = wl_display_connect(NULL);
  if (!dpy) { fprintf(stderr, "sem ligação a %s\n", s); return 2; }
  struct wl_registry *r = wl_display_get_registry(dpy);
  wl_registry_add_listener(r, &L, NULL);
  wl_display_roundtrip(dpy);
  if (!fi) { fprintf(stderr, "o KWin não oferece org_kde_kwin_fake_input\n"); return 2; }
  org_kde_kwin_fake_input_authenticate(fi, "ph2d-rato-falso", "prova da faixa unica");
  wl_display_roundtrip(dpy);
  char linha[256];
  for (int i = 1; argc > 1 ? i < argc : fgets(linha, sizeof linha, stdin) != NULL; i++) {
    const char *passo = argc > 1 ? argv[i] : linha;
    linha[strcspn(linha, "\n")] = 0;
    if (argc == 1 && !*linha) continue;
    double a = 0, b = 0;
    if (sscanf(passo, "move %lf %lf", &a, &b) == 2)
      org_kde_kwin_fake_input_pointer_motion_absolute(fi, wl_fixed_from_double(a), wl_fixed_from_double(b));
    else if (!strcmp(passo, "down")) org_kde_kwin_fake_input_button(fi, 0x110, 1);
    else if (!strcmp(passo, "up")) org_kde_kwin_fake_input_button(fi, 0x110, 0);
    else if (sscanf(passo, "sleep %lf", &a) == 1) { wl_display_flush(dpy); usleep((useconds_t)(a * 1000)); }
    else { fprintf(stderr, "passo desconhecido: %s\n", passo); return 1; }
    wl_display_roundtrip(dpy);
  }
  wl_display_disconnect(dpy);
  return 0;
}
