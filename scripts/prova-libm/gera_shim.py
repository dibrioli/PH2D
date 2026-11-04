#!/usr/bin/env python3
"""Gera a libm ADVERSÁRIA (Bug #58): interpõe as transcendentais da libc, regista quem as chama e, com
LIBM_ADVERSARIA=1, devolve o resultado a 1 ulp. Um motor que não dependa da libm do sistema não muda um byte."""
import sys

UM = "sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh exp exp2 expm1 log log2 log10 log1p cbrt".split()
DOIS = "atan2 pow hypot".split()


def corpo(nome, tipo, args, chamada):
    prox = "nextafterf" if tipo == "float" else "nextafter"
    alvo = "1e30f" if tipo == "float" else "1e300"
    return f"""
{tipo} {nome}({args}) {{
    static {tipo} (*real)({args});
    if (!real) real = dlsym(RTLD_NEXT, "{nome}");
    regista("{nome}", __builtin_return_address(0));
    {tipo} r = real({chamada});
    return adversaria() ? {prox}(r, {alvo}) : r;
}}
"""


CABECA = r"""#define _GNU_SOURCE
#include <dlfcn.h>
#include <math.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#define N 65536
static _Atomic(void *) vistos[N];
static const char *nomes[N];
static atomic_ulong contas[N];
static int adversaria(void) {
    static int v = -1;
    if (v < 0) v = getenv("LIBM_ADVERSARIA") != NULL;
    return v;
}
static void regista(const char *nome, void *ret) {
    unsigned long h = ((unsigned long)ret * 11400714819323198485ul) >> 48;
    for (unsigned i = 0; i < N; i++, h = (h + 1) % N) {
        void *atual = atomic_load(&vistos[h]);
        void *livre = NULL;
        if (!atual && atomic_compare_exchange_strong(&vistos[h], &livre, ret)) {
            nomes[h] = nome;
            atual = ret;
        }
        if (atual == ret || livre == ret) {
            atomic_fetch_add_explicit(&contas[h], 1, memory_order_relaxed);
            return;
        }
    }
}
__attribute__((destructor)) static void despeja(void) {
    const char *caminho = getenv("LIBM_REGISTO");
    if (!caminho) return;
    FILE *f = fopen(caminho, "a");
    if (!f) return;
    for (unsigned i = 0; i < N; i++) {
        void *r = atomic_load(&vistos[i]);
        Dl_info d;
        if (r && dladdr(r, &d))
            fprintf(f, "%s %s 0x%lx %lu\n", nomes[i] ? nomes[i] : "?", d.dli_fname,
                    (unsigned long)((char *)r - (char *)d.dli_fbase), atomic_load(&contas[i]));
    }
    fclose(f);
}
"""

SINCOS = r"""
void sincos(double a, double *s, double *c) {
    static void (*real)(double, double *, double *);
    if (!real) real = dlsym(RTLD_NEXT, "sincos");
    regista("sincos", __builtin_return_address(0));
    real(a, s, c);
    if (adversaria()) { *s = nextafter(*s, 1e300); *c = nextafter(*c, 1e300); }
}
void sincosf(float a, float *s, float *c) {
    static void (*real)(float, float *, float *);
    if (!real) real = dlsym(RTLD_NEXT, "sincosf");
    regista("sincosf", __builtin_return_address(0));
    real(a, s, c);
    if (adversaria()) { *s = nextafterf(*s, 1e30f); *c = nextafterf(*c, 1e30f); }
}
"""

saida = [CABECA]
for n in UM:
    saida.append(corpo(n, "double", "double a", "a"))
    saida.append(corpo(n + "f", "float", "float a", "a"))
for n in DOIS:
    saida.append(corpo(n, "double", "double a, double b", "a, b"))
    saida.append(corpo(n + "f", "float", "float a, float b", "a, b"))
saida.append(SINCOS)
sys.stdout.write("".join(saida))
