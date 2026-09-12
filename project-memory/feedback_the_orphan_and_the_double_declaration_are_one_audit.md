---
name: feedback-the-orphan-and-the-double-declaration-are-one-audit
description: Mover código produz DUAS avarias mudas e simétricas — o ficheiro que nenhum `mod` declara (deixa de correr) e o que DOIS declaram (corre a dobrar); o instrumento é um só audit de alcançabilidade
metadata:
  type: feedback
---

Ao mover ficheiros entre árvores (W2/L2 Fase B, 2026-09-11) nasceram **13 órfãos** e
**3 duplicados**, e nenhum dos dois dá erro, aviso ou falha de compilação.

- **ÓRFÃO** — o ficheiro mudou-se e a linha `mod X;` que o declarava ficou no `mod.rs`
  antigo, e foi apagada com os vizinhos que de facto saíram. Ele **deixa de ser
  compilado**: a suíte fica verde com menos gates do que tinha.
- **DUPLICADO** — um gerador listou a pasta e declarou um ficheiro que já tinha pai por
  `#[path]`. Ele compila como **dois módulos**: os gates dele correm a dobrar e a
  contagem sobe sem ninguém ter escrito um teste.

**Why:** são a mesma avaria com o sinal trocado — *quantos pais este ficheiro tem?*
`0` e `2` são ambos errados, e ambos silenciosos. Um `cargo check` verde não distingue
nenhum dos casos.

**How to apply:** um **audit de alcançabilidade** a partir da raiz do módulo (`lib.rs`/
`mod.rs`), seguindo `mod X;` e `#[path]`, responde às duas metades de uma vez: quem não
é alcançado é órfão, quem é alcançado por dois é duplicado. ⚠️ O duplicado também aparece
no `nextest-list-diff` como um nome que está **ao mesmo tempo** em `MOVED` e em `ONLY-B` —
essa é a assinatura, e foi ela que o denunciou (22 660 contra 22 655 esperados).
⛔ A contagem total sozinha não chega: 13 órfãos e 13 duplicados cancelam-se.

Relacionado: [[reference_topic_gate_discipline]] · [[reference_topic_git_hazards]]
