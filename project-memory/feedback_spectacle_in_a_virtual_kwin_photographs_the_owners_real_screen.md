---
name: feedback_spectacle_in_a_virtual_kwin_photographs_the_owners_real_screen
description: Nunca usar `spectacle` para fotografar uma cena — mesmo lançado dentro de `kwin_wayland --virtual` ele fala com o KWin REAL pelo D-Bus e fotografou o ecrã do dono; a receita segura é `import -window <id>` na Xwayland virtual, com recusa de DISPLAY=:0
metadata:
  type: feedback
---

Em 2026-09-16 (TOP-20 #16, `line/components`), para fotografar a cena de smoke antes de a mandar ao
dono, lancei o app numa sessão `kwin_wayland --virtual` e chamei o `spectacle` lá dentro. **A imagem
era o ecrã REAL do dono** — outra janela do PH2D e o VS Code com outra conversa. Apaguei-a sem a usar
e disse-lhe.

**Why:** o `spectacle` pede a captura ao KWin pelo **D-Bus da sessão**, e o bus de sessão herdado é o
do utilizador, não o do compositor aninhado. Nada no comando indica isso, e a foto parece plausível.
Mais duas armadilhas medidas no mesmo dia: o **XTest** é ignorado na Xwayland da sessão virtual (a
roda não rolou), e o **`ydotool`** escreve no `uinput` ⇒ move o **rato real** do dono.

**How to apply:**
- Fotografar cena = [`docs/Components/ferramentas/fotografa_cena.sh`](../docs/Components/ferramentas/fotografa_cena.sh)
  (na `line/components` até ser integrado): `kwin_wayland --virtual --xwayland` com um
  `XDG_CONFIG_HOME` isolado cuja `kwinrulesrc` maximiza tudo; o app corre com
  `env -u WAYLAND_DISPLAY` (X11 na Xwayland virtual); id da janela por
  `xprop -display $DISPLAY -root _NET_CLIENT_LIST`; foto por `import -display $DISPLAY -window $WIN`.
- ⛔ `import -window root` falha na Xwayland *rootless* («missing an image filename») — fotografa-se a
  JANELA.
- ⛔ Recusar correr se `DISPLAY` for `:0` ou `WAYLAND_DISPLAY` for `wayland-0` (o roteiro já o faz).
- ⛔ Nunca `spectacle`, nunca `ydotool`. Um CLIQUE prova-se num gate de costura
  (`MockPanelHost` + `dispatch_pointer_event`), não na foto.
- Um processo do dono a correr (outra worktree) não se toca, nem para «limpar».

⛔⛔ **E há uma TERCEIRA armadilha, medida em 2026-09-18 (`line/Vector`): o roteiro NÃO CONSTRUÍA.**
Ele exigia um binário (`[ -x "$BIN" ] || exit 2`) e fotografava, em silêncio, a **build anterior**.
O modo de falha é o pior que há: fotografei uma cena *«antes»* e *«depois»* de um corte de ficheiro,
vi **as duas fotos IGUAIS** e escrevi na mensagem do commit que a mudança estava verificada — e as
duas eram do **mesmo binário**. *Duas fotos do mesmo binário são sempre iguais, e lêem-se
exactamente como «a mudança não estragou nada».* ⇒ o roteiro passou a correr
`cargo build -q -p ph2d-host-desktop --profile smoke` **por dentro** (incremental: `2,5 s` com a
árvore quente), e não num passo que quem chama tem de se lembrar de escrever.

Irmã de [[feedback_a_smoke_for_the_owner_explains_what_each_thing_on_screen_is]] (a foto é
obrigatória) e de [[reference_topic_oracle_discipline]] (o XTest também não serve para um oráculo num
Xwayland aninhado).
