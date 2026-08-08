---
name: project_workstation_freeze_memory_reclaim
description: "A workstation travou 2× (2026-08-08) por livelock de recuperação de memória, não por bug do PH2D — os 577 GB de target/ são o combustível"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9a6dfcf7-82e6-4724-98ca-49062b0eb660
  modified: 2026-08-08T20:31:52.695Z
---

A workstation (9950X, 123 GiB, CachyOS 7.1.6) **travou duas vezes**, a 2ª em
2026-08-08 14:10. Não é bug do PH2D: é a configuração de memória do CachyOS
(dimensionada para laptop) contra o padrão de trabalho do Modo L.

**O mecanismo, medido no journal:**
- **13:22:21** — `ld.mold: page allocation failure: order:0` **dentro do caminho
  de swap-out do zram**. Zona Normal com **38 MB livres contra min de 66 MB** —
  abaixo da marca d'água. 105 GB em page cache ativo.
- **14:10:11** — `BUG: Bad page state ... nonzero mapcount` numa thread de LTO
  do rustc. `_mapcount` foi de `0xffffffff` para `0xffff00ff`: **um byte inteiro
  zerado, não um bit**.
- **14:10:35** — último log. Sem panic, sem oops → travamento total.

**Por que trava em vez de matar alguém:** o kernel nunca declara OOM, fica preso
em *direct reclaim* em 32 núcleos. Sem OOM daemon, ninguém morre. Com
`nowatchdog` no cmdline, ninguém detecta nem reseta.

**O combustível é o Modo L:** 577 GB de `target/` em 5 worktrees (line-Painter
237 G, line-physics 204 G) + 100 GB de sccache. Os 105 GB de page cache são o
btrfs cacheando isso. Pico da sessão medido em 2 boots: **116,7 e 115 GiB de
123 — 94% da RAM**.

**Aplicado em 2026-08-08** (detalhe: `/etc/sysctl.d/99-ph2d-memoria.conf`,
`/etc/default/earlyoom`): `min_free_kbytes` 66 MB→1 GiB e
`watermark_scale_factor` 10→200 (marcas d'água: min 65→1008 MB, low 189→3494 MB)
· **earlyoom** com `--prefer` nos builds e `--avoid` em `code`/`claude`/`node`
(as janelas de agente) · journal 47 MB→2 GB · zram 123→32 GiB e `nowatchdog`
removido + `panic=30` (**os dois só no próximo boot**).

**Why:** um agente sobrevive a um `rustc` morto e tenta de novo; não sobrevive à
máquina travada. E sem watchdog o próximo travamento também não deixa evidência.

**How to apply:** antes de abrir muitas linhas paralelas, confira o page cache e
pode os `target/` de worktrees já integradas. A hipótese de **hardware não foi
separada** — o byte zerado é assinatura mais de barramento DDR5 marginal (4 DIMMs
no AM5, sem ECC, EDAC sem contadores) que de célula, e há `rustc` SIGSEGV +
`mold` SIGBUS ×2 no histórico. Só um **memtest86+ overnight** responde; o A/B do
kernel é o `linux-cachyos-lts` 6.18.42, já instalado. Ver
[[project_modo_l_speed_hole_worktree_targets_slow_path]] e
[[feedback_a_ship_x_can_be_the_environment_not_the_code]].
