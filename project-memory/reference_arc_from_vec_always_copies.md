---
name: reference-arc-from-vec-always-copies
description: "`Arc::from(Vec<T>)` SEMPRE realoca e copia; `collect::<Arc<[T]>>()` de um iterador TrustedLen aloca UMA vez — sem unsafe"
metadata:
  type: reference
---

**`Arc<[T]>` não consegue adotar o buffer de um `Vec<T>`.** Um `Arc` guarda o refcount **inline,
imediatamente antes dos dados** (`ArcInner`), e a alocação de um `Vec` não tem espaço para ele.
Então `Arc::from(vec)` / `vec.into()` **aloca um segundo buffer e faz memcpy do todo** — 2 blocos,
2× o pico. O mesmo vale para `Box<[T]> → Arc<[T]>`.

Isso é estrutural na escolha do tipo, não um descuido do std — e é fácil de não ver, porque a
linha parece uma conversão de graça.

**A saída segura (sem `unsafe`, sem `Arc::new_uninit_slice` + `assume_init`):**
`impl FromIterator<T> for Arc<[T]>` **especializa em `TrustedLen`** — aloca o `ArcInner` uma vez,
no tamanho certo, e escreve direto dentro dele.

```rust
// 2 blocos, 2× o pico:
let a: Arc<[f32]> = vec.into();

// 1 bloco:
let b: Arc<[f32]> = (0..n).map(f).collect();            // Map<Range> é TrustedLen
let c: Arc<[f32]> = src.iter().copied().collect();      // Copied<slice::Iter> é TrustedLen
let d: Arc<[f32]> = std::iter::repeat_n(0.0, n).collect(); // RepeatN é TrustedLen
```

**`Chain` NÃO é `TrustedLen`** — `head.iter().chain(mid).chain(tail).collect::<Arc<_>>()` cai no
caminho lento (Vec + cópia). Para concatenar, use `from_fn` com um branch por índice.

Para mutar em cima: colete o `Arc` (1 bloco) e pegue `Arc::get_mut` — é `Some` porque o `Arc`
acabou de nascer. É como `SampleData::{from_fn, map_in_place, build}` estão implementados em
`crates/ph2d-audio/src/buffer.rs` ([ADR-0117] D2), e é o que deixou `ph2d-audio` e
`ph2d-audio-edit` manterem `#![forbid(unsafe_code)]`.

**A especialização é detalhe de implementação da std — então MEÇA, não comente.** O gate
`ph2d-audio-edit/tests/measure_arc_build.rs` afirma 2 blocos vs 1 bloco com dhat: se uma std
futura parar de alocar uma vez, ele fica vermelho em vez de a cópia voltar em silêncio.

Onde isso morde: qualquer buffer grande e compartilhado (`Arc<[f32]>` de áudio, `Arc<[u8]>` de
pixels). Ver [[feedback_zero_alloc_gate_capacity_not_global_counter]] para o gotcha do dhat
(contador global → um `#[test]` por binário).
