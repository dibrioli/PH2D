---
name: reference-clip-studio-under-wine-needs-win81
description: Clip Studio Paint sob Wine 11 — o prefixo tem de dizer Windows 8.1, senão fecha ao escolher a avaliação; 1.13.2 funciona, 5.1.5 trava no login
metadata:
  type: reference
---

Dois prefixos: `~/Apps/clipstudio1` (CSP **1.13.2**, o do menu «CLIP STUDIO») e `~/Apps/clipstudio`
(CSP **5.1.5**, em `Program Files (x86)`). Lançadores `csp1.sh`/`csp.sh` usam Xvfb via
`~/Apps/pincel-oraculos/env.sh`.

Medido 30/10 (Wine 11.20): com o prefixo em Windows 10, clicar **PRO/EX** (avaliação) dá page fault
NULL em `triglavwinrt.dll+0x329a` logo depois de `RoGetActivationFactory` falhar para
`Windows.UI.Input.RadialController` (Surface Dial; o Wine não tem). Cura: `winecfg -v win81` no
prefixo — o CSP nem tenta o caminho WinRT. Depois disso: avaliação PRO, Ctrl+N, risco de lápis, sem
queda (Xvfb, llvmpipe). Bate com o Wine bug 58369 (CSP 4) e 59249 (CSP 3).

A 5.1.5 com win81 abre só uma janela BRANCA (login em WebView2) — não usável sem mais trabalho.

**How to apply:** CSP novo sob Wine → `winecfg -v win81` ANTES do primeiro arranque; prefira a 1.13.2.
Para achar o módulo de um crash sem winedbg: `WINEDEBUG=+loaddll` e procure a maior base ≤ endereço.
