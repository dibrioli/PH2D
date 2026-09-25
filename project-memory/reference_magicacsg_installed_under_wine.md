---
name: reference_magicacsg_installed_under_wine
description: "MagicaCSG (modelador SDF, Windows-only, proprietário) instalado em ~/Apps/magicacsg sob Wine 11; só abre com driver Wayland + vsync off; licença proíbe uso comercial e engenharia reversa ⇒ referência visual, não oráculo"
metadata:
  node_type: memory
  type: reference
  originSessionId: 9cf0b8e2-4517-484b-b3f2-bc39a95ae51b
  modified: 2026-09-23T19:18:48.365Z
---

Medido 2026-09-23, a pedido do Enio (*«dá para aprender com o app como aprendemos com o blender?»*).

**Instalação:** Demo 0.7.3 (build 4/26/2024) em `~/Apps/magicacsg/MagicaCSG-Demo`, prefixo Wine próprio em `~/Apps/magicacsg/wineprefix`, lançador `~/Apps/magicacsg/magicacsg.sh`, entrada de menu `~/.local/share/applications/magicacsg.desktop` (ícone extraído do .exe, só existe 32×32). Wine = `extra/wine` 11.17 (o `cachyos-extra-znver4/wine` falhou no mirror: *Maximum file size exceeded* no `.sig`).

**As duas condições para abrir (sem qualquer uma, NENHUMA janela e nenhum erro):**
1. `DISPLAY=` (driver **Wayland** do Wine). Pelo X11/Xwayland o app entra num laço infinito de `glGetError` logo após um `glBufferStorage` de 136 MB (37 M chamadas em 15 s, 100 % de um núcleo).
2. `__GL_SYNC_TO_VBLANK=0`. No Wayland o app desenha ANTES de mostrar a janela e o SwapBuffers com vsync espera para sempre um frame de superfície invisível (backtrace: thread principal dentro do `opengl32`, 0 % CPU). Com as duas: janela maximizada, NVIDIA, ~1,5 GB VRAM.
As linhas `MESA-EGL: warning` e `[Error] buffer error code (4)` são ruído — o app continua depois delas.

**Triagem (§0.9):** proprietário. readme: *«for testing and non-commercial usage only. selling and reverse engineering the software are disallowed.»* Beta (Patreon) com termos não lidos. ⇒ usar como **oráculo** de um produto comercial = zona proibida/cinzenta; **decisão do Enio**, como o Free do Cascadeur ([[reference_cascadeur_oracle_door_measured]]).

**Porta sem interface: NÃO medida como existente.** Cena `.mcsg` é texto ASCII (autorável por nós), há consola in-app (`F1`) e snapshot PNG (`6`/`Ctrl+6`) no `config/hotkey.txt`; a Demo **não exporta malha** (tabela da página). Logo a única saída seria IMAGEM, via GUI numa tela virtual (molde do Cascadeur). ⛔ Parei de ler `strings` do `.exe` (nomes de classe internos) — isso já é o lado da engenharia reversa que a licença proíbe.

**Porque o preview dele é rápido (23/09, só fontes públicas + observação de utilizador):** changelog público — *«Independent Resolution for each volume (Press ⊕/⊖)»* (0.1.0) e *«Marching Cube mesh export»* desde 0.0.1; o autor fez o MagicaVoxel Viewer (volumes esparsos até 2048³, LOD, SVO). VRAM medida contra o `buffer_size` documentado do `config.txt`: 512→1001 MiB · 1024→1623 · 2048→2907 (linear, pool pré-alocado). ⇒ ✅ **CONFIRMADO pelo dono com foto (23/09)**: o botão ⊖ «Decrease Resolution» (faixa acima da vista 3D, depois do campo «1» de duplicados; `config/ui/upper.ui`) parte os aros FINOS do robô em pedaços soltos — assinatura de grelha, feição mais fina que a célula some: o campo é ASSADO numa grelha de volume na placa ao editar, e girar a câmera só marcha a grelha (custo independente da complexidade). O nosso (auditado no mesmo dia): modo MODEL/matcap traça na **CPU** com a fita especializada por ladrilho, custo por aresta do contorno; a placa só no Render e avalia a fita INTEIRA por passo; **nenhuma grelha assada existe e nenhuma recusa medida dela** (o ADR-0161 já prevê «o grid existe como cache derivado, não como verdade»). Proposta devolvida ao dono: grelha para aproximar + fórmula exacta nos últimos passos.

**O que ele ensina sem oráculo:** a lista pública de features (sweep, glyphs/SVG, fillet variável, groove/chamfer blend, arrays/mirror avançados, mesh SDF, trim) contra o módulo 3D Modeling — uso normal de utilizador, que a licença permite para teste. Relacionado: [[reference_topic_implicit_field_laws]] · [[reference_manual_apps_in_home_apps_are_invisible_to_cachy_update]].
