---
name: reference_manual_apps_in_home_apps_are_invisible_to_cachy_update
description: "cachy-update (= arch-update) só vê o pacman; apps em ~/Apps são invisíveis; e o passo do AUR aborta na revisão do paru (curado 15/10 com SkipReview/SudoLoop/CleanAfter)"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 71283a04-cdb6-4b23-ac27-d628b22dcbea
  modified: 2026-08-23T14:02:45.635Z
---

`cachy-update` é um symlink para `arch-update`; ele só enxerga o que o **pacman** conhece
(repos + AUR via `paru -Qua`, auto-detectado). Em 2026-07-03 o `google-chrome` do AUR foi
**construído** pelo paru mas nunca instalado — o `.pkg.tar.zst` foi descompactado em
`~/Apps/chrome`, com symlink em `~/.local/bin/google-chrome-stable` (que vem ANTES de
`/usr/bin` no PATH) e um `~/.local/share/applications/google-chrome.desktop` com o MESMO id
do do sistema. Resultado: Chrome parado em 150 enquanto o AUR ia a 151, e invisível ao
notificador.

**Cura (23/08):** pacote 151 construído em `~/.cache/paru/clone/google-chrome/`, e o script
`instalar-e-limpar.sh` ao lado faz `pacman -U` + remove o symlink, o .desktop local e
`~/Apps/chrome`, e reaponta `whatsapp-web.desktop`. O `tsups-open` do nobreak já usa
`google-chrome-stable` pelo PATH, não precisa de nada.

**Como aplicar:** se um app «não atualiza» ou «não aparece no cachy-update», a primeira
pergunta é `pacman -Qo $(which app)` — sem dono = instalação manual. Os outros cinco de
`~/Apps/` (obsidian, keepassxc, github-desktop, alchemy, gh) seguem nessa condição; todos
têm pacote no AUR ou em `extra`. **Cascadeur** (13/09, 2026.2.2) também mora em `~/Apps/cascadeur`
e é o único SEM pacote no AUR: tgz em `https://cascadeur.com/rest/download/build?platform=linux&buildId=<n>`
(sem login; o `buildId` é o `data-build-id` do botão em /download), lançador `~/.local/bin/cascadeur`
com `QT_QPA_PLATFORM=xcb` (o pacote só traz o backend X11) + `FONTCONFIG_PATH=/etc/fonts`. Atualizar
= baixar de novo e trocar a pasta. **GDevelop** (23/09) entrou do jeito certo desde o início: `gdevelop-bin` do AUR (AppImage oficial do GitHub, arrastando `electron32-bin` do AUR + `asar` de `extra`), construído com `makepkg` e instalado por `pacman -U` ⇒ visível ao cachy-update. ⚠️ Instalar pelo pacman exige senha (sudo) — prepare o
pacote e entregue UM comando, por [[feedback_a_red_checksum_is_acted_on_by_the_agent_not_escalated]].

**15/10/2026 — «o gestor não atualiza o AUR»:** o passo do AUR saía em 45 s sem construir nada
(journal: nenhum `pacman` do paru entre o -Syu dos repos e o `paccache` final) = abortou na
REVISÃO interativa do paru. O mesmo `paru --devel -Syu` com `--noconfirm` instalou os 5.
Cura em `~/.config/paru/paru.conf`: `SkipReview` + `SudoLoop` + `CleanAfter` (o cache tinha
14 GB de pacotes velhos). Para reproduzir sem terminal: `SUDO_ASKPASS=<script> paru ... --sudoflags=-A`,
e apague o script no fim.
