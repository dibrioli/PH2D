# Oráculo da costura — libmypaint corrido sem interface

**Pergunta:** quando UM traço que perde carga volta sobre si mesmo, quão dura é a costura entre a
perna escura e a pálida, comparada com a borda externa do próprio traço?

**Triagem de licença (passo 1, §0.9):** `pacman -Qi libmypaint` ⇒ **ISC** — porta ABERTA. O alvo é a
BIBLIOTECA (o app `mypaint` é GPL e não foi tocado, nem lido).

| ficheiro | o que é |
|---|---|
| [`costura.c`](costura.c) | o arnês: superfície 512², tiles ZERADOS, pincel montado pela API, traço em U NOSSO com a pressão a seguir a lei de depleção do MIX-1; mede larguras 10–90 % |
| [`saida_libmypaint_1.6.1.txt`](saida_libmypaint_1.6.1.txt) | a fixtura, com cabeçalho de proveniência |

```
gcc -O2 -o costura costura.c $(pkg-config --cflags --libs libmypaint) -lm
for h in 0.20 0.50 0.80 0.95; do ./costura - $h; done
```

**O que ele respondeu:** num motor de dab macio a costura interna tem a largura do perfil do próprio
dab (razão `~1,00` contra a borda externa nas linhas `meio`). É de onde sai a lei do
[`watercolor_reserve`](../../../../crates/ph2d-tool-painter/src/tool/paint/watercolor_reserve.rs):
o nível da reserva cede ao longo da cauda do depósito, e não num degrau. Plano e medições:
[doc 41](../../41_a_reserva_e_um_nivel_disputado.md).

⚠️ As duas armadilhas do arsenal (§2-bis) valem aqui: sem zerar os tiles a saída é lixo plausível, e
o canal vem em escala `65535 = 1,0`.
