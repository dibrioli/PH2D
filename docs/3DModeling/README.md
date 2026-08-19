# 3D Modeling — modelagem por **campo implícito** (porta do módulo)

> ⚠️ **Não confundir com [`docs/3D/`](../3D/README.md)**, que é o módulo de **escultura**
> (`ph2d-sculpt3d`, malha + verbos, ADR-0150). Este aqui é **modelagem**: booleana e arredondamento
> de aresta que **não podem falhar**, porque são aritmética de campo e não topologia de malha.
> São dois módulos, duas linhas — e eles **se encontram** (uma malha esculpida pode entrar na
> booleana deste, via `ph2d-sdf`).

**A promessa, em uma frase:** o modelo é uma **função**, não uma malha — então união e arredondamento
são `min` e um operador sobre dois números, e o **raio do fillet fica editável para sempre**.

| Doc | O que é |
|---|---|
| [`01_resultados_spike.md`](01_resultados_spike.md) | ⭐ **O QUE FOI MEDIDO (W0, 19/08).** As imagens, os números e as 6 decisões que eles forçaram. Nenhum kill-criterion disparou; a **quina viva** reprovou e tem mecanismo nomeado |
| [`03_plano_implicito.md`](03_plano_implicito.md) | **O PLANO VIVO.** A rota escolhida: a tese, o motor (`fidget`, medido), o arredondamento e sua armadilha, a quina viva, arquitetura, waves e kill-criteria |
| [`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](02_o_que_torna_boolean_e_fillet_extraordinarios.md) | **Por que esta rota.** Mede que o Blender 4.5 já resolveu a booleana e que o buraco é o arredondamento. As 3 famílias candidatas |
| [`00_plano_port.md`](00_plano_port.md) | ⛔ **Rota substituída** — não execute as waves. Continuam fonte: o **§1** (estudo do original: 9 leis, 19 operações) e o **§2** (inventário da PH2D). O **§7** segue válido: por que **não** se escreve um kernel do zero |

**Estado:** **W0 fechada e aprovada** pelo Enio no smoke de 19/08 (*"excepcional"*) · **W1 em curso**.

| Wave | Estado | O que ficou |
|---|---|---|
| **W0** — spike + imagem | ✅ | Os dois arredondamentos exatos a **0,00 %**; vértice triplo fecha; JIT **5,3×**; a aresta viva da **malha** serrilha (item aberto, mecanismo nomeado) |
| **W1** — documento + ADR | 🔶 | [ADR-0161](../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md) · [`ph2d-field`](../../crates/ph2d-field/) (o documento, 10 gates) · [`ph2d-field-eval`](../../crates/ph2d-field-eval/) (a ponte, 9 gates). **Falta:** a ponte ECS |
| **W2** — ver a coisa | ⬜ | O traçado do campo no viewport |

⭐ **As medições da W0 viraram GATES**, não anedota: `an_exact_internal_fillet_delivers_the_radius_asked`
e `an_exact_external_round_delivers_the_radius_asked` afirmam o raio exato a cada corrida.

**Rodar o spike:** `cd spikes/field-spike && cargo run --release --features jit` (imagens em `out/`).

**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (clone de UX do
[MoI 3D](https://moi3d.com)). O **fluxo** dele é o alvo de UX; o kernel NURBS dele, não.
