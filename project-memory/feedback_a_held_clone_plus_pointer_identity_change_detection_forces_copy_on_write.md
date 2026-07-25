---
name: feedback_a_held_clone_plus_pointer_identity_change_detection_forces_copy_on_write
description: "A consumer that holds a producer's Arc to detect change by pointer identity forces the producer's next make_mut to copy the whole buffer — per op. Fix = version counter."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: bac68944-e667-49ef-b3b4-f7b9e430eaca
  modified: 2026-07-24T22:17:19.612Z
---

Quando um consumidor (a shell) segura um **clone do `Arc` de um buffer vivo do
produtor** (um tool) para detectar mudança por **identidade de ponteiro**
(`Arc::as_ptr`), ele força o próximo `Arc::make_mut` do produtor a ver 2 donos e
**copiar o buffer inteiro — a cada operação**. O clone é *load-bearing* para a detecção
de mudança, então o bug não parece acidental: some se você trocar a detecção por um
**contador de versão monotônico** (o produtor bumpa; o consumidor compara), e aí o
consumidor pode possuir o próprio buffer (patch da região suja) e soltar o Arc do
produtor → escrita in-place.

**É a lei do ADR-0124 do áudio ("pergunte a versão, nunca o ponteiro"), e ela reincide.**
2ª instância medida: o Painter (2026-07-24) — `painter_preview.rgba` segurava o
`canvas_rgba` do tool ⇒ `stamp_dabs` copiava o canvas inteiro por movimento (0,34 ms @
2048², **10 ms @ 4096²**, plano no raio do pincel — um pincel de 0,5 px pagava 64 MiB
para mudar 1 pixel; "queda de FPS, parece dependente de CPU"). Fix: `canvas_version()` +
`own_preview_buffer` (a shell possui o mirror) → **9,8 → 0,1 ms/move, ~100×,
footprint-bound**. Doc 25 §11.

**Why:** identidade de ponteiro é um proxy de "mudou?" que **força a mudança que
detecta** (a cópia gera o ponteiro novo). Um contador de versão diz o mesmo sem obrigar
ninguém a copiar.

**How to apply:** ache o padrão *"segura o Arc do outro + compara `as_ptr`"* (grep
`Arc::as_ptr`, `arc_token`) em toda ponte tool↔shell/RT↔UI. Se o produtor MUTA aquele
buffer, é cópia por-op. Troque por versão; deixe o consumidor possuir o próprio buffer;
gate = o helper devolve buffer que **não aliasa** o do produtor (`!Arc::ptr_eq`, mutação
`Arc::clone(src)` → RED) + razão de perf plana no tamanho. Relacionado:
[[reference_arc_from_vec_always_copies]], [[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]].
