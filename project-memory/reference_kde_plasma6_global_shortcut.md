---
name: reference-kde-plasma6-global-shortcut
description: "Amarrar atalho global a um .desktop no Plasma 6/Wayland — a seção é [services][nome.desktop], e depois do login ainda falta o GRAB"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 273e7943-ca95-473e-87b3-5ecf7108cd0c
  modified: 2026-07-25T20:25:08.995Z
---

Máquina do Enio: **KDE Plasma 6.7, Wayland**. Receita medida em 2026-07-25 (custou
várias tentativas — não re-derive):

1. `.desktop` em `~/.local/share/applications/` (symlink para o arquivo do app serve;
   `Exec` entre aspas quando o caminho tem espaço/acento).
2. Em `~/.config/kglobalshortcutsrc`, a seção é **`[services][nome.desktop]`** com
   `_launch=F3,none,Descrição`. ⚠️ `[nome.desktop]` (sem `[services]`) é o formato do
   **Plasma 5** e é lido como nada — o daemon nunca cria o componente que sabe *lançar*.
   Controle positivo para conferir o formato: procure um `.desktop` que já funcione no
   próprio arquivo (ex.: `[services][org.kde.spectacle.desktop]`).
3. **Relogar.** Quem detém `org.kde.kglobalaccel` é o **kwin_wayland** (o
   `plasma-kglobalaccel.service` inicia e sai porque o nome D-Bus já está tomado), e ele
   lê esse arquivo só na inicialização da sessão.
4. ⚠️ Depois do login o componente existe e `shortcut()` já devolve a tecla, mas o daemon
   pode **não ter feito o grab** — `getGlobalShortcutsByKey(<tecla>)` volta VAZIO.
   Cura: `setShortcut` ao vivo (agora funciona, porque o componente existe):
   `gdbus call --session --dest org.kde.kglobalaccel --object-path /kglobalaccel
   --method org.kde.KGlobalAccel.setShortcut "['nome.desktop','_launch','X','X']" "[16777266]" 4`
   (`16777266` = Qt::Key_F3). *Configuração e grab são coisas diferentes.*

⚠️ `doRegister` + `setShortcut` **sem** o componente carregado cria ação **órfã**: a tecla
fica capturada e não executa nada — pior que não amarrar, porque rouba a tecla do app
(F3 é "Find Next" no VSCode). Solte com `setShortcut(..., [], 4)`.

⚠️ **O KWin bloqueia injeção de tecla por cliente Wayland**: `wtype` responde
*"Compositor does not support the virtual keyboard protocol"* ⇒ não dá para testar
atalho sinteticamente nem colar automaticamente, e **o espanso provavelmente não
consegue colar**. A via que resta é `ydotool` (uinput), que exige `ydotoold` como ROOT.
Corolário: o teste de um atalho é o Enio apertar a tecla.

Ver [[reference-prompt-deck-app]] (o primeiro app amarrado assim) e
[[feedback_a_negative_search_needs_a_positive_control]] — três verificações minhas
nesta sessão deram falso negativo (`pgrep` casando com a própria linha de comando,
`head -1` mascarando exit code, grep por `bin/fuzzel` quando `argv[0]` é `fuzzel`).
