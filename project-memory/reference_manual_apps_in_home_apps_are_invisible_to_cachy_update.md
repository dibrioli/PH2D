---
name: reference_manual_apps_in_home_apps_are_invisible_to_cachy_update
description: "Apps descompactados à mão em ~/Apps (chrome, obsidian, keepassxc, github-desktop, alchemy, gh) não existem para o pacman, logo o cachy-update (= arch-update) nunca os lista; o Chrome foi migrado para o AUR via paru em 2026-08-23"
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
têm pacote no AUR ou em `extra`. ⚠️ Instalar pelo pacman exige senha (sudo) — prepare o
pacote e entregue UM comando, por [[feedback_a_red_checksum_is_acted_on_by_the_agent_not_escalated]].
