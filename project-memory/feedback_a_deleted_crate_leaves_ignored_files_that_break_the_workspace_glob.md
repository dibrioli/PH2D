---
name: feedback-a-deleted-crate-leaves-ignored-files-that-break-the-workspace-glob
description: Apagar uma crate no git não apaga a PASTA noutro checkout se lá houver ficheiros ignorados — o `crates/*` do workspace vê um membro sem Cargo.toml e TODO o cargo morre
metadata:
  type: feedback
---

Apagar uma crate por `git rm` só apaga ficheiros RASTREADOS. Num outro checkout (o primário depois do
`--ff-only`, ou uma linha depois do rebase) a pasta sobrevive se tiver ficheiros IGNORADOS — medido na
poda do 3D (05/10): `crates/ph2d-field-render/` ficou com duas imagens `.ppm` de uma sonda. O workspace é
`members = ["crates/*"]`, logo a pasta vira «membro» sem `Cargo.toml` e `cargo metadata` falha: o
`ship.sh` deu ✗ em fmt, clippy, deny, nextest… em **10 s** — cara de código partido, causa de AMBIENTE.

**Why:** o glob do workspace conta pastas, não ficheiros rastreados; e `git status` não mostra ignorados.

**How to apply:** depois de integrar uma linha que APAGA crates, no checkout que vai correr o gate:
`for d in crates/*/; do [ -f "$d/Cargo.toml" ] || echo "$d"; done` — cada resto é lixo ignorado
(`git status --short --ignored <pasta>` confirma `!!`) e apaga-se. Muitos ✗ em segundos ⇒ ambiente
primeiro (o `/pd-ship` já avisa). Ver [[feedback_a_commit_with_paths_never_picks_up_an_untracked_file]].
