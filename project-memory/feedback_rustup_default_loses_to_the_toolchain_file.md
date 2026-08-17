---
name: feedback-rustup-default-loses-to-the-toolchain-file
description: "Um job de CI que instala um toolchain e depois roda cargo no repo testa o PIN, não o toolchain instalado — rust-toolchain.toml vence rustup default"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3f8057cf-2924-478f-83ba-bb111ad80c49
  modified: 2026-08-17T23:23:56.225Z
---

Na precedência do rustup, `RUSTUP_TOOLCHAIN` (env) **vence** o override de
diretório, que vence o **`rust-toolchain.toml`**, que vence o `rustup default`.
Logo um job que faz `dtolnay/rust-toolchain@X` (= `rustup default X`) e depois
roda `cargo` dentro de um repo com `rust-toolchain.toml` **não roda X** — roda o
pin do arquivo, e o job fica verde sem nunca ter testado o que o nome dele promete.

**Why:** medido no PH2D em 2026-08-17. O job `MSRV (rust 1.92) check` passava há
meses; `rustup show active-toolchain` responde `1.95 (overridden by
'<repo>/rust-toolchain.toml')`, e `cargo --version` no repo dá **1.95** com o
default em 1.92. Rodado honestamente (`RUSTUP_TOOLCHAIN=1.92`), o workspace
**reprova** — e o piso verdadeiro era o próprio pin, falsificado por dois
mecanismos independentes (deps exigindo 1.94; nosso código usando `if let`
guards, estáveis só em 1.95). É a família do *verde por vácuo*
[[reference_topic_gate_discipline]], com o agravante de o gate ser de INFRA:
ninguém o lê ao mudar código.

**How to apply:** para medir um toolchain que não é o pin, use
**`RUSTUP_TOOLCHAIN=<v> cargo ...`** (ou `rustup run <v>`), nunca `rustup default`.
E ao auditar um job de toolchain alternativo, o teste decisivo é uma linha:
`rustup show active-toolchain` imprime **quem manda e por quê**. Se o piso medido
igualar o pin, o job não tem conteúdo — a cura é tornar a declaração verdadeira
(`rust-version` = o número medido), não afrouxar a barra
[[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]].
