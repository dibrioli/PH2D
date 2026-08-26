---
name: a-map-the-tick-clears-must-be-opened-not-looked-up
description: Um verbo que corre DEPOIS do tick no mesmo quadro não pode PROCURAR o que o tick apaga fora do modo — ele tem de ABRIR a entrada, senão o botão só liga o modo
metadata:
  type: feedback
---

Vector / Morph States, 2026-08-26 (report do Enio: *"o morph não consegue segurar os estados
atribuidos no momento do Rec"*).

O mapa de máquinas vivas é **propriedade do `tick`**, e o `tick` faz `machines.clear()` em todo
quadro fora do modo de pré-visualização. O botão **▶ Play** corre **depois** do `tick`, no mesmo
quadro, e estava escrito como:

```rust
if let Some(m) = self.morph_machines.get_mut(&host.to_bits()) { m.travel(&g, row); }
```

⇒ vindo de **fora** do modo — que é de onde o artista sempre vem — ele encontrava o mapa **vazio**,
ligava o modo, e **não viajava**. A forma só mudava ao **segundo** clique.

**Why:** o sintoma não parece um bug de ciclo de vida — parece o botão «não funcionar às vezes». E
o defeito é invisível a todo gate de unidade: `travel`, `capture` e `install` estavam **todos
certos e todos gateados**. Ele vivia só na **ordem dentro do quadro**.

**How to apply:**
- Antes de escrever `get_mut`/`get` sobre um mapa de runtime, pergunte **quem o esvazia e quando**.
  Se o dono o limpa condicionalmente, o consumidor tardio tem de usar `entry(..).or_insert_with(..)`
  — **abrir**, nunca procurar.
- A cura certa é uma **porta** (`fn play(...)`) partilhada com o dono, e não duas linhas no braço do
  despacho: braço de `match` dentro do laço de render **não é alcançável de um teste**.
- ⚠️ O gate que apanha isto é o da **COMPOSIÇÃO** (o quadro inteiro: verbo → tick → mundo), nunca o
  da unidade. Ver [[feedback_i_write_the_right_guard_and_do_not_gate_it]] e
  [[feedback_paint_and_dispatch_must_read_the_same_source]].
