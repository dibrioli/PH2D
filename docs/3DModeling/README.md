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
| [`04_resultados_perfis.md`](04_resultados_perfis.md) | ⭐ **O QUE FOI MEDIDO (W3, 19/08).** O desenho da caneta virando sólido: o sinal sem `if`, as duas tabelas de custo que escolhem a tolerância, os dois oráculos independentes, e o **gatilho medido que NÃO foi construído** |
| [`05_resultados_imagem.md`](05_resultados_imagem.md) | ⭐ **A IMAGEM (19/08).** Anti-serrilhado adaptativo (borda = 0,5–1,2 % dos pixels), traçado no tamanho real, composição pré-multiplicada — e **o 73× que não era o preço do raio, era o lote grande demais para haver threads que chegassem** |
| [`03_plano_implicito.md`](03_plano_implicito.md) | **O PLANO VIVO.** A rota escolhida: a tese, o motor (`fidget`, medido), o arredondamento e sua armadilha, a quina viva, arquitetura, waves e kill-criteria |
| [`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](02_o_que_torna_boolean_e_fillet_extraordinarios.md) | **Por que esta rota.** Mede que o Blender 4.5 já resolveu a booleana e que o buraco é o arredondamento. As 3 famílias candidatas |
| [`00_plano_port.md`](00_plano_port.md) | ⛔ **Rota substituída** — não execute as waves. Continuam fonte: o **§1** (estudo do original: 9 leis, 19 operações) e o **§2** (inventário da PH2D). O **§7** segue válido: por que **não** se escreve um kernel do zero |

**Estado:** **W0 fechada e aprovada** pelo Enio no smoke de 19/08 (*"excepcional"*) · **W1 e W3
fechadas** · a W2 tem o traçado no shell e o canvas de primeira classe **aberto** (decidido: entra
com o painel, na W4).

| Wave | Estado | O que ficou |
|---|---|---|
| **W0** — spike + imagem | ✅ | Os dois arredondamentos exatos a **0,00 %**; vértice triplo fecha; JIT **5,3×**; a aresta viva da **malha** serrilha (item aberto, mecanismo nomeado) |
| **W1** — documento + ADR | ✅ | [ADR-0161](../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md) · [`ph2d-field`](../../crates/ph2d-field/) (o documento, 12 gates) · [`ph2d-field-eval`](../../crates/ph2d-field-eval/) (a avaliação, 9 gates) · [`ph2d-field-ecs`](../../crates/ph2d-field-ecs/) (a ponte, 4 gates + 2 no shell). **Um objeto de campo é salvo e desfeito** |
| **W2** — ver a coisa | 🔶 | [`ph2d-field-render`](../../crates/ph2d-field-render/) (o traçado, 8 gates) + o smoke no shell. Traçado **no tamanho real da área**, com **anti-serrilhado adaptativo por +3 % a +24 %** ([doc 05](05_resultados_imagem.md)). **Falta:** órbita por mouse e perspectiva |
| **W3** — os perfis | ✅ | [`ph2d-field-profile`](../../crates/ph2d-field-profile/) (a costura com o editor vetorial, 8 gates) + `Extrude`/`Revolve`. **O desenho da caneta vira sólido**, e o raio de quina do editor arredonda as arestas verticais. `FIELD_DOC_VERSION` → **2** |

**Smoke (roda agora):**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
`=1` junção de 3 com filete interno e aros externos · `=2` cubo arredondado · `=3` caixa furada com
a boca arredondada · ⭐ `=4` **cantoneira DESENHADA** (o perfil vem de um path do editor vetorial,
com raio vivo de quina) · ⭐ `=5` **o torno** (o mesmo tipo de contorno girado em torno de Y — um
vaso oco).

**A peça gira sozinha até alguém pegar nela.** Depois, o mouse manda — **os mesmos botões do módulo
de escultura**: arrastar com o **esquerdo ou o direito** gira · com o **do meio** desloca · a **roda**
aproxima. ⭐ Aproximar mostra **mais forma**, não uma forma inchada: as tolerâncias da marcha descem
com o pixel ([doc 05](05_resultados_imagem.md) e o gate `zooming_in_does_not_inflate_the_part`).
⚠️ **O terminal imprime três linhas** — a cena montada, o traçado, e *"primeiro quadro desenhado —
N pixels de peça"*. **Se a terceira não aparecer, PARE**: a janela vazia é falha de caminho, não de
geometria.

⭐ **As medições da W0 viraram GATES**, não anedota: `an_exact_internal_fillet_delivers_the_radius_asked`
e `an_exact_external_round_delivers_the_radius_asked` afirmam o raio exato a cada corrida.

**Rodar o spike:** `cd spikes/field-spike && cargo run --release --features jit` (imagens em `out/`).

**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (clone de UX do
[MoI 3D](https://moi3d.com)). O **fluxo** dele é o alvo de UX; o kernel NURBS dele, não.
