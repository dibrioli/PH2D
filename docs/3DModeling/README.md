# 3D Modeling — modelador NURBS/B-Rep (porta do módulo)

> ⚠️ **Não confundir com [`docs/3D/`](../3D/README.md)**, que é o módulo de **escultura**
> (`ph2d-sculpt3d`, malha + verbos, ADR-0150). Este aqui é **modelagem CAD paramétrica**:
> curva NURBS, sólido B-Rep, booleana, fillet, STEP. São dois módulos, duas linhas.

| Doc | O que é |
|---|---|
| [`00_plano_port.md`](00_plano_port.md) | **O plano.** O que o original é, o que a PH2D já tem, qual kernel Rust ganhou e por qual medição, arquitetura e waves |

**Estado:** plano escrito, zero código. A **W0 (spike do kernel) bloqueia todas as outras** e tem
kill-criterion congelado — ver `00_plano_port.md` §5.

**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (TypeScript/Vite,
clone de UX do [MoI 3D](https://moi3d.com); marcos 0-6 fechados, 7 parcial, 118 testes).
