---
name: feedback-ispc-cross-process-concurrency
description: "asset-cooker ISPC encoders dão SIGBUS flaky (~50%) mesmo isolado+single-threaded; pior sob cargo concorrente — green-on-retry, não é bug seu"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 825e85bf-5944-4187-a5af-69c4406e3e47
---

A armadilha #1 conhecida ("ISPC SIGBUS em paralelo → sempre `RUST_TEST_THREADS=1`")
sugere que single-threaded resolve. NÃO resolve. Caracterizado em 2026-05-28
(W1.T15) num Mac M-series: os encoders ISPC vendored (bc7enc/astcenc/etcpak/intel)
dão **SIGBUS (signal 10) / SIGTRAP (signal 5) flaky ~50% das vezes MESMO com**
`RUST_TEST_THREADS=1`, **mesmo rodando UM ÚNICO teste isolado**. Mesmo binário,
2 runs consecutivos: run 1 SIGBUS, run 2 OK. É **não-determinístico**, não
layout-determinístico (uma edição de código que muda o layout do binário pode
parecer "introduzir" o crash, mas é coincidência — re-rode e passa).

Piora sob **cargo concorrente** (dois `cargo test` no mesmo target dir ao mesmo
tempo → crash mais frequente; sintoma = "Blocking waiting for file lock on
artifact directory" no output). Mas concorrência NÃO é a causa raiz — é só um
agravante.

**Why:** estado global C dos objetos ISPC prebuilt + algum init/alinhamento frágil
no primeiro toque. Rust não consegue corromper memória de objeto ISPC separado via
attrs (`#[non_exhaustive]`) ou chamadas inócuas (`reader.limits()`); então crash
SIGBUS em encoder ISPC ≠ regression do seu código Rust, por construção.

**How to apply:** (1) crash SIGBUS/SIGTRAP em test ISPC = **green-on-retry**, re-rode
2-3× antes de suspeitar da própria mudança; um teste que passa em QUALQUER run está
correto. (2) Confirme inocência rodando o caminho num binário SEPARADO (ex. o seam
integration test passou 7/7 cobrindo todos os encoders enquanto o lib unit test do
mesmo encoder flakava). (3) Ainda assim rode UM cargo de cada vez (sequencie
`cmd1; cmd2`) pra não AGRAVAR. (4) Pro CI: precisa retry-on-SIGBUS no job de cook,
ou nextest com retries — senão o job vai flakar ~50%. Generaliza
[[feedback-parallel-agent-commit-collision]] pro eixo build/test. Vide armadilha #1
do [[project-ktx2-phase2-v4-accepted-2026-05-27]].
