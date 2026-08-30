---
name: feedback_the_newest_possible_is_not_the_newest_count_the_ceilings_first
description: Antes de planear qualquer subida de dependência, conte os TETOS — uma dep segurada por outra não dá erro de resolução ao ser forçada, dá DUAS CÓPIAS, e a falha aparece como erro de TIPO na costura
metadata:
  type: feedback
---

**«O mais recente possível» ≠ «o mais recente».** Antes de planear qualquer subida,
corra `bash scripts/stack-audit.sh --tetos` e leia quem segura quem.

⛔ **Forçar um teto NÃO dá erro de resolução — dá duas cópias.** O cargo resolve as duas
felizmente, e a falha aparece muito depois, como **erro de tipo** na costura onde as
duas se encontram: um `Device` de uma não serve à outra.

⭐ **O veredito é POR CRATE, nunca geral.** Duas cópias são benignas numa folha
(compressão, `derive`) e venenosas em quem aparece na **nossa superfície de tipos**.
Medido em 2026-08-29: `miniz_oxide` e `thiserror` **já são duas cópias** e ninguém
repara; o `wgpu` não pode ser, porque
[`vello_pass.rs`](../crates/ph2d-render/src/vello_pass.rs) passa um `wgpu::Device`
nosso para dentro do `vello::Renderer`.

⚠️ **Um teto tem DONO, e é o dono que se vigia — não uma data.** A recusa do `wgpu` 30
cai no dia em que sair um `vello > 0.10.0` que peça `wgpu ^30`; nada mais precisa de
mudar do nosso lado, porque as 10 declarações são todas nossas e movem-se em lockstep.

**Why:** o `wgpu 30.0.1` está publicado e é inalcançável desde 2026-08-29 porque o
`vello 0.10.0` — a versão mais recente que existe — pede `^29.0.3`. Sem contar os tetos
primeiro, um plano de subida promete 30 e entrega uma semana perdida.

**How to apply:** `--tetos` antes de responder *«dá para atualizar X?»*. O `ship.sh`
imprime o inventário antes do veredito de push. Decisão e mecanismo:
[ADR-0168](../docs/architecture/decisions/0168-the-stack-rises-to-its-ceilings-and-four-dependencies-stay-behind-on-purpose.md).
Ver [[feedback_two_copies_of_a_dependency_can_be_the_mechanism_not_the_residue]].
