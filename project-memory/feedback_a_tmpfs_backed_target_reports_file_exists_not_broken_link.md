---
name: feedback-a-tmpfs-backed-target-reports-file-exists-not-broken-link
description: Depois de todo boot, a 1ª build falha com "File exists (os error 17)" — é um link para tmpfs apagado, e o cargo não diz isso.
metadata:
  type: feedback
---

`PH2D/target/debug` e `PH2D/target/rust-analyzer` são **symlinks** para
`/mnt/ramtarget/PH2D/...`, um tmpfs de 48 GB declarado no `/etc/fstab`.

**Um tmpfs é apagado a cada boot.** Os dois links passam a apontar para o vazio,
e o cargo para com:

```
error: failed to create directory `.../target/debug`
Caused by: File exists (os error 17)
```

⚠️ **A mensagem não nomeia a causa.** *«File exists»* onde não existe nada é o
`mkdir` a esbarrar no **link pendurado**: para o `mkdir` o nome está ocupado, e
para o `ls` o alvo não existe. Procurar um diretório sobrando é procurar a coisa
errada — o que está lá é um link.

**Why:** medido em 2026-08-29, logo depois de um boot. A primeira build de todo
boot falhava, e o sintoma parecia dano da mudança de disco — não era: a cópia
antiga, intocada, tinha exatamente os mesmos dois links quebrados.

**How to apply:** curado de forma permanente em
`/etc/tmpfiles.d/ph2d-ramtarget.conf` (`d` com idade `-`, que cria no boot e
nunca é limpo pelo `systemd-tmpfiles-clean`). Se o erro voltar, confira
`ls -l crates/../target/` antes de suspeitar do repo — e lembre que
**todo diretório de build apontado para um tmpfs herda esta mesma falha**.
Relacionado: [[project_projects_live_on_a_dedicated_2tb_disk]].
