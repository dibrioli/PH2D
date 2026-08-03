---
name: feedback-a-doc-comment-naming-a-cfg-expires-grep-the-attribute
description: "Um doc-comment que afirma em que `cfg` um caminho vive é uma alegação que EXPIRA — grepe o atributo; e saiba que `cargo test --release` liga `cfg(test)`"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 39ec3808-26ec-4cf4-b80e-b2291882bc64
  modified: 2026-08-02T15:55:41.878Z
---

Dois fatos, e o segundo é o que morde.

**(a)** `cargo test --release` compila a lib com `--test`, o que **liga `cfg(test)`**. Código atrás de
`#[cfg(test)]` / `#[cfg(any(test, debug_assertions))]` está ATIVO numa sonda de unidade, mesmo em
release — ela **não pode** observar o caminho que o `cargo run --release` toma. Quando isso importar,
a sonda tem de **imprimir em que rota está**.

**(b)** ⚠️ **Mas um doc-comment que NOMEIA um `cfg` é uma afirmação sobre o mundo que expira em
silêncio** — o `#[cfg]` some numa promoção e a prosa fica. **Grepe o atributo; não leia a prosa.**

**Caso medido (PH2D, o undo por delta do Painter, 2026-08-02):** investigando quem segurava os planos,
dois doc-comments diziam que a rota do journal era `cfg(any(test, debug_assertions))` (*"em release ela
é hoje SEMPRE o caminho de sempre"*). Combinando isso com (a), publiquei o veredito *"a sonda não vê o
produto"* — **falso**: o arquivo não tinha **um** `#[cfg]`, e o journal do RELEVO shipava desde a wave
anterior (só o do CANVAS continuava debug-only). As duas frases eram verdadeiras dois degraus antes e
ninguém as reconferiu quando a promoção as tornou falsas. A causa real do achado era banal e estava a
dois `grep`: a outra porta tinha o MESMO limiar que eu havia ablacionado só de um lado.

**Why:** (a) é uma armadilha real e conhecida, e é justamente por ser plausível que ela **empresta
credibilidade** a um doc obsoleto: a prosa dá a hipótese, o mecanismo verdadeiro a confirma, e o
veredito sai coerente e errado. Nenhum teste falha, porque não há teste sobre onde um `cfg` está.

**How to apply:**
1. Antes de concluir *qualquer coisa* sobre "o caminho de release", **`grep '#\[cfg' <arquivo>`** — o
   atributo, nunca o comentário que o descreve.
2. Ao **promover** código de `cfg(debug)`/`cfg(test)` para release, varra os doc-comments que citam o
   gate: eles são parte do diff. Ver [[feedback_stale_comment_and_dead_code_lie]].
3. Sonda que possa estar do lado de dentro de um gate **publica a rota na saída**
   ([[feedback_a_silenced_instrument_reads_as_a_result]]).
4. Ablação por entrada: se um mecanismo tem **duas portas irmãs** (o mesmo limiar em duas funções),
   ablacionar uma e ver metade do efeito é o sintoma — procure a gêmea antes de inventar teoria.
