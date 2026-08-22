---
name: project-disk-full-corrupts-objects-mold-sigbus
description: "Disco cheio a meio de um build TRUNCA os .o, e o mold morre com SIGBUS a cada retry — o cargo julga-os frescos e o build nunca recupera sozinho"
metadata: 
  node_type: memory
  type: project
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-22T21:25:20.850Z
---

⚠️ **Correção do Enio (2026-08-22, mesma noite): o disco NUNCA encheu** — `df` tinha 526 GB
livres. Era **metadata do btrfs sem espaço para crescer** (não-alocado 0), e os SIGBUS das 17:24 e
18:25 seguiram, em segundos, um `csum failed` no journal (artefato novo que volta do disco com
checksum errado — correlacionado com o kernel 7.2.0). Mecanismo, números e cura em
[[project_btrfs_metadata_starved_not_disk_full_2026_08_22]]. O que segue abaixo está certo
sobre o **sintoma** e a **cura imediata**; a causa é a de lá.

Quando um `.o` fica **inválido a meio de um build** (truncado por `ENOSPC` de metadata, ou com
checksum errado), o `cargo` regista-o como fresco, e cada tentativa seguinte falha no LINK: o
`ld.mold` faz `mmap` dele e morre com **SIGBUS**.

**A assinatura, e é o que a torna reconhecível:**
- `ps -o wchan=` do linker diz **`vfs_coredump`** e o CPU está a **0%** — ele não está a linkar,
  está a escrever core dump.
- `coredumpctl list` mostra **`/usr/bin/mold  SIGBUS`**, repetido uma vez por tentativa.
- O `df` já mostra espaço livre (o build que encheu o disco morreu e libertou), então **a causa
  não está visível no momento em que se olha**.

**Why:** o build fica preso num laço invisível — cada retry demora ~10 min (link de debug),
crasha, e nada no output diz «objeto corrompido». Em 2026-08-22 isto custou ~40 minutos de
esperas encadeadas, com a hipótese errada («é a carga da máquina», load estava em 228).

**How to apply:**
1. ⚠️ **Um link que demora muito mais que o normal: confira o `wchan`, não a carga.**
   `ps -eo pid=,etime=,pcpu=,wchan=,comm= | grep -E "mold|lld|ld\."` — 0% de CPU num linker é
   crash, nunca lentidão.
2. A cura é **`cargo clean -p <crate>`** (limpou 99,4 GB e 638 939 ficheiros aqui). ⛔ Apagar só
   `target/*/incremental` **não chega** — os `.o` corrompidos vivem em `target/debug/deps/`.
3. Prevenção: o disco desta máquina vai a 46% com 5 targets vivos (~110 GB cada). Ver
   [[project-modo-l-speed-hole-worktree-targets-slow-path]] e a DIRETIVA_FIM_DE_DIA.

⚠️ E o erro de método que o precedeu: eu corri `cargo test ... | tail -3`, e o **pipe mascarou o
exit code** — quando matei o processo por engano, o pipeline devolveu 0 e eu li «passou». Ver
[[feedback-pipe-masks-script-exit-code]]: redirecione para ficheiro e leia o `$status`, nunca
canalize um comando cujo veredito importa.
