---
name: feedback-the-orphan-and-the-double-declaration-are-one-audit
description: Mover código produz DUAS avarias mudas e simétricas — o ficheiro que nenhum `mod` declara (deixa de correr) e o que DOIS declaram (corre a dobrar); o audit de alcançabilidade EXISTE desde 12/09 como o gate `architecture_no_orphan_source_file`
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

## ✅ O INSTRUMENTO EXISTE desde 2026-09-12 — e esta memória prescreveu-o UM DIA antes

Gate `architecture_no_orphan_source_file` (`ph2d-editor-core/tests/it/`), as **duas** metades:
nenhum `.rs` da workspace sem pai, nenhum com dois pais **na mesma crate** (o duplicado mede-se por
RAIZ: a `lib` e cada ficheiro de `tests/` são crates distintas).

⚠️ **Ninguém o construiu depois desta memória.** Nasceu porque na integração da Fase D a `line/app-vec`
perdeu **3 gates** por um órfão — e ao correr sobre a workspace inteira ele achou **dois que ninguém
tinha visto**: os **8 testes das setas** (`ph2d-vec-scene/src/arrows_tests.rs`, fora do build desde
`ea2817b73` «remove a seta circular», que apagou a declaração do ficheiro em vez de uma entrada) e
**uma sonda de 583 linhas** (`ph2d-vec-art-live/src/brush_cost_probe.rs`, fora desde a Fase C).
*Uma memória que prescreve um instrumento não o constrói.*

**Como foi validado (o método, reutilizável):**
- **oráculo = o compilador**: os `.d` (dep-info) listam todo ficheiro construído — mas são cegos ao
  gémeo desligado por feature (`sculpt3d_absent.rs`), por isso o gate é TEXTUAL e o oráculo serve para
  o calibrar;
- ⚠️ a 1.ª régua textual **errou numa regra** e foi o oráculo que o disse: *um ficheiro carregado por
  `#[path]` resolve os filhos na PRÓPRIA pasta*, como um `mod.rs` (o `measure_preview_drain.rs` do
  Painter lia-se órfão). E a 1.ª régua pelos `.d` também mentiu: exigia caminhos absolutos, e eles
  são relativos — deu `7 533` órfãos em `7 533`, apanhado pelos controlos positivos;
- **prova por mutação nas duas metades**: comentar a declaração da sonda ⇒ o gate acusa exactamente
  esse ficheiro; declarar o `snap.rs` duas vezes (sob `#[cfg(any())]`, que o compilador ignora e o
  gate lê) ⇒ acusa exactamente esse. ⚠️ **Não apague só a linha `mod x;`** para mutar: o `#[cfg]` de
  cima fica pendurado no item seguinte ([[feedback_an_orphaned_cfg_attaches_to_the_next_item_and_the_default_build_is_blind]]).
