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
| [`06_resultados_cena_e_gizmo.md`](06_resultados_cena_e_gizmo.md) | ⭐ **O QUE FOI MEDIDO (W5+W6, 20/08).** A peça deixa de ser um objeto e vira uma **cena de objetos**; o **gizmo 3D** com os três verbos. As três medições que decidiram onde a pose 3D mora, os números derivados das alças, os 6 gates da costura ponteiro↔gizmo — e o **undo de um arrasto, que estava partido por um motivo que a nota anterior errava** |
| [`05_resultados_imagem.md`](05_resultados_imagem.md) | ⭐ **A IMAGEM (19/08).** Anti-serrilhado adaptativo (borda = 0,5–1,2 % dos pixels), traçado no tamanho real, composição pré-multiplicada — e **o 73× que não era o preço do raio, era o lote grande demais para haver threads que chegassem** |
| [`03_plano_implicito.md`](03_plano_implicito.md) | **O PLANO VIVO.** A rota escolhida: a tese, o motor (`fidget`, medido), o arredondamento e sua armadilha, a quina viva, arquitetura, waves e kill-criteria |
| [`02_o_que_torna_boolean_e_fillet_extraordinarios.md`](02_o_que_torna_boolean_e_fillet_extraordinarios.md) | **Por que esta rota.** Mede que o Blender 4.5 já resolveu a booleana e que o buraco é o arredondamento. As 3 famílias candidatas |
| [`00_plano_port.md`](00_plano_port.md) | ⛔ **Rota substituída** — não execute as waves. Continuam fonte: o **§1** (estudo do original: 9 leis, 19 operações) e o **§2** (inventário da PH2D). O **§7** segue válido: por que **não** se escreve um kernel do zero |

**Estado:** **W0 fechada e aprovada** pelo Enio no smoke de 19/08 (*"excepcional"*) · **W1, W3, W5 e W6
fechadas** · a W2 tem o traçado no shell e a perspectiva **aberta**; o canvas 3D de primeira classe
segue **aberto**.

| Wave | Estado | O que ficou |
|---|---|---|
| **W0** — spike + imagem | ✅ | Os dois arredondamentos exatos a **0,00 %**; vértice triplo fecha; JIT **5,3×**; a aresta viva da **malha** serrilha (item aberto, mecanismo nomeado) |
| **W1** — documento + ADR | ✅ | [ADR-0161](../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md) · [`ph2d-field`](../../crates/ph2d-field/) (o documento, 12 gates) · [`ph2d-field-eval`](../../crates/ph2d-field-eval/) (a avaliação, 9 gates) · [`ph2d-field-ecs`](../../crates/ph2d-field-ecs/) (a ponte, 4 gates + 2 no shell). **Um objeto de campo é salvo e desfeito** |
| **W2** — ver a coisa | 🔶 | [`ph2d-field-render`](../../crates/ph2d-field-render/) (o traçado, 11 gates) + o smoke no shell. Traçado **no tamanho real da área**, com **anti-serrilhado adaptativo por +3 % a +24 %** ([doc 05](05_resultados_imagem.md)). **Falta:** perspectiva |
| **W4** — a ferramenta | 🔶 | Navegação por mouse ([`field3d_input.rs`](../../shells/desktop/src/field3d_input.rs)): **rotação LIVRE** sem polos ([doc 05](05_resultados_imagem.md) §7), pan, zoom sem teto herdado, `Home`. E o **painel** ([`ph2d-panel-model3d`](../../crates/ph2d-panel-model3d/), 6 gates): ⭐ **o raio de cada operação, editável ao vivo** — a promessa do módulo virada em controle. **Falta:** o canvas de primeira classe |
| **W5** — a cena e o gizmo | ✅ | ⭐ **Cada primitiva é um OBJETO da cena** (`ph2d-field-ecs`: `cook`/`spawn_doc`, ida e volta gateada) e o documento é **cozido** do mundo a cada quadro · ⭐ o **gizmo 3D de mover** (3 setas + 3 planos + disco de vista, [`field3d_gizmo.rs`](../../shells/desktop/src/field3d_gizmo.rs)), com 9 gates de lei e **6 de costura**. Tokens `axis-x/y/z` no design system. [doc 06](06_resultados_cena_e_gizmo.md) |
| **W6** — os três verbos | ✅ | ⭐ **Rodar** (3 argolas + a de vista; o ângulo é medido no PLANO, não em pixels — uma volta fecha) e **escalar** (⛔ UMA alça: a escala é uniforme por [ADR-0161 §6](../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md), e três caixas por eixo prometeriam o que o modelo não dá). Seletor no painel + teclas `G`/`R`/`S`. E **um arrasto voltou a ser UM passo de undo**. [doc 06 §5](06_resultados_cena_e_gizmo.md) |
| **W3** — os perfis | ✅ | [`ph2d-field-profile`](../../crates/ph2d-field-profile/) (a costura com o editor vetorial, 8 gates) + `Extrude`/`Revolve`. **O desenho da caneta vira sólido**, e o raio de quina do editor arredonda as arestas verticais. `FIELD_DOC_VERSION` → **2** |

**Como entrar — DUAS portas:**
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && cargo run -p ph2d-host-desktop --release
```
⭐ **A pill `MODEL` no topo**, ao lado da `SCULPT`: ela cria a peça, abre o painel e desenha. É a
porta que um artista encontra — até 19/08 a única era a variável de ambiente, e *uma feature que só
existe para quem já sabe que ela existe não existe*.

A outra porta dirige uma cena específica:
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
`=1` junção de 3 com filete interno e aros externos · `=2` cubo arredondado · `=3` caixa furada com
a boca arredondada · ⭐ `=4` **cantoneira DESENHADA** (o perfil vem de um path do editor vetorial,
com raio vivo de quina) · ⭐ `=5` **o torno** (o mesmo tipo de contorno girado em torno de Y — um
vaso oco).

**A peça gira sozinha até alguém pegar nela.** Depois, o mouse manda — **os mesmos botões do módulo
de escultura**: arrastar com o **esquerdo ou o direito** gira · com o **do meio** desloca · a **roda**
aproxima · **Home** repõe a vista.

⭐ **A rotação é LIVRE** (não é prato giratório): não há polo onde o gesto morra, e uma diagonal roda
na diagonal. O preço é o horizonte poder ficar inclinado — daí a tecla de repor ([doc 05](05_resultados_imagem.md) §7).

⭐ **A peça é uma CENA DE OBJETOS.** A Hierarquia mostra `Model` com um filho por primitiva
(`Cylinder`, `Cylinder 2`, …) — cada um com nome, pose própria, salvo e desfeito. Clique numa linha
e **o gizmo aparece no objeto**. No topo do painel há três botões — **Move · Rotate · Size** — e as
teclas `G`, `R` e `S` fazem o mesmo (com o rato sobre a janela 3D):

- **Move**: arraste uma seta para andar num eixo, um quadrado para andar num plano, o anel do meio
  para andar na direção da tela.
- **Rotate**: arraste uma argola colorida para girar naquele eixo, ou a argola de fora para girar na
  direção da tela.
- **Size**: arraste o punho do canto para aumentar ou diminuir. ⚠️ **Uniforme** — é o que a peça
  entrega, e é por isso que o botão diz *Size* e não *Scale*.

O botão **direito continua a girar a vista** mesmo por cima do gizmo, e **um arrasto inteiro é um só
Ctrl+Z**.
⚠️ Até 19/08 a peça era **um** objeto com a árvore escondida dentro dele, e até essa manhã não era
objeto nenhum: o `FieldObject` da W1 estava registado e **sem produtor**, e o gate daquela wave
media a metade errada — que o componente *sobrevive* ao snapshot, nunca que alguma coisa o *põe* no
mundo. As duas frases do smoke (*"apenas um objeto"* e *"não há gizmo"*) eram **um** defeito: um
objeto que a cena não enumera não tem pose que um gizmo agarre. [doc 06](06_resultados_cena_e_gizmo.md)

⭐ **O painel abre à direita**, com uma linha por operação da peça. Arraste o controle de
**Radius** e o arredondamento muda **na peça, ao vivo** — é isto que o módulo promete e que nem o
Blender nem o MoI dão, porque lá o filete é geometria assada. O rodapé mostra quanto custou o último
quadro. ⭐ Aproximar mostra **mais forma**, não uma forma inchada: as tolerâncias da marcha descem
com o pixel ([doc 05](05_resultados_imagem.md) e o gate `zooming_in_does_not_inflate_the_part`).
⚠️ **O terminal imprime três linhas** — a cena montada, o traçado, e *"primeiro quadro desenhado —
N pixels de peça"*. **Se a terceira não aparecer, PARE**: a janela vazia é falha de caminho, não de
geometria.

⭐ **As medições da W0 viraram GATES**, não anedota: `an_exact_internal_fillet_delivers_the_radius_asked`
e `an_exact_external_round_delivers_the_radius_asked` afirmam o raio exato a cada corrida.

**Rodar o spike:** `cd spikes/field-spike && cargo run --release --features jit` (imagens em `out/`).

**Original estudado:** `/home/enio/Documentos/Recursos/MOI_Clone_2026-08-19` (clone de UX do
[MoI 3D](https://moi3d.com)). O **fluxo** dele é o alvo de UX; o kernel NURBS dele, não.
