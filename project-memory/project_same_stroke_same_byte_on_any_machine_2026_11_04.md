---
name: project-same-stroke-same-byte-on-any-machine-2026-11-04
description: Dono 04/11 — no motor de pintura o mesmo traço dá o mesmo byte em qualquer computador; o motor não chama a matemática do sistema (ph2d-det-math).
metadata:
  type: project
---

Dono 04/11 aprovou («pode consertar») a recomendação: o traço de pintura dá o mesmo byte em qualquer computador, para o alarme de
pincel só acender quando um pincel mudou de verdade.

**Why:** a versão da glibc mudava o byte (Bug #58) e acendia o alarme sem mudança nenhuma de pincel. Vale só para o traço na CPU: a lei
«mesmo bit por placa» da GPU continua ([[project_same_bit_law_is_per_device_2026_10_24]]).

**How to apply:** nas crates do censo `o_motor_nao_chama_a_libm_do_sistema`, use `ph2d_det_math::DetMath` (`det_sin`, `det_powf`…), nunca o método de
`std`. Prova ao vivo: `scripts/prova-libm/prova.sh`. Um pincel novo que mude de byte continua a ir ao dono
([[project-brush-tests-cheap-alarm-owner-decides-2026-11-03]]).
