# 3D Modeling — modelador NURBS/B-Rep (porta do módulo)

> ⚠️ **Não confundir com [`docs/3D/`](../3D/README.md)**, que é o módulo de **escultura**
> (`ph2d-sculpt3d`, malha + verbos, ADR-0150). Este aqui é **modelagem CAD paramétrica**:
> curva NURBS, sólido B-Rep, booleana, fillet, STEP. São dois módulos, duas linhas.

| Doc | O que é |
|---|---|
| [`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](02_o_que_torna_boolean_e_fillet_extraordinarios.md) | ⚠️ **LEIA PRIMEIRO.** O alvo reformulado: o que o Enio quer é booleana que nunca falha + arredondamento bonito. Mede que **o Blender 4.5 já resolveu a booleana** (adotou o Manifold) e que o buraco é o **arredondamento**. As 3 famílias candidatas, e a recomendação |
| [`00_plano_port.md`](00_plano_port.md) | O plano do port. Estudo do original, as 9 leis herdadas, as 19 operações, o que a PH2D já tem, arquitetura e waves. ⚠️ §§1-2 válidas; **stack (§3) e waves (§5) sub judice** pelo doc acima |

**Estado:** plano escrito, zero código. A escolha de família (B-Rep exato **ou** implícito/SDF) é
**decisão do Enio por imagem**, e a W0 vira o teste comparativo visual — ver `02_...` §5.

**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (TypeScript/Vite,
clone de UX do [MoI 3D](https://moi3d.com); marcos 0-6 fechados, 7 parcial, 118 testes).
