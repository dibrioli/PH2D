---
name: feedback-removing-a-middle-variant-from-a-serialized-enum-silently-rewrites-saved-files
description: O serde derivado grava o ÍNDICE da variante — tirar uma do meio puxa todas as de baixo, sem erro nenhum
metadata:
  type: feedback
---

Retirar uma variante **morta** de um enum parece higiene. Se o enum for serializado, é uma
migração de formato disfarçada.

Medido em 2026-08-30, ao retirar `TextureFilter::NearestAniso` (um modo fisicamente inalcançável —
o `wgpu` recusa anisotropia sem os três filtros lineares, logo o sampler dele era **campo a campo**
o do `NearestMipmap`): o `TextureFilter` viaja como **postcard** dentro do `.ph2dproj`, e o
`Serialize` **derivado** grava o *índice de variante*. Tirar a do meio puxava `LinearAniso` de `6`
para `5` ⇒ todo projecto gravado com anisotropia linear passava a ler *Nearest Mip*, **sem uma
linha de erro**.

**Why:** um enum tem duas identidades — a **posição** (o que o derive grava) e a **tag** (o que o
código significa). Enquanto ninguém as separa, elas coincidem por acidente, e a primeira remoção
descobre a diferença nos ficheiros de quem já gravou.

**How to apply:**
1. Antes de tirar uma variante, pergunte **se ela atravessa disco ou rede**. Meça os bytes
   (`postcard::to_allocvec` do valor) — não deduza.
2. Se atravessa: `Serialize`/`Deserialize` **manuais pela TAG**, com golden dos bytes antigos e a
   metade justa no gate (*a tag retirada ainda LÊ, e cai no modo que ela sempre foi na prática*).
3. **O slot não se reaproveita** — a tag é o formato. Quem não é oferecido some da lista de
   rótulos, não do enum: o padrão da casa é `[Option<&str>; N]` com `None` = buraco, porque a
   POSIÇÃO é a tag e encurtar a lista faria o `zip` casar o rótulo `n+1` com o id `n`.

⚠️ E o discriminante explícito é o que impede a próxima: `LinearAniso = 6` diz o número em vez de
o herdar da ordem. Relacionado: [[feedback_a_label_must_promise_what_the_model_delivers]].
