---
titulo: "Handoff de integração — line/sculpt3d, W3: a doação, sem a mão"
tags: [modulo/3d, tipo/handoff, assunto/integracao, status/pendente-de-smoke]
status: pendente-de-smoke
modulo: 3D
atualizado: 2026-07-31
resumo: "O rig de luz passa a ter um dono (ph2d-light), a escultura acende por ele, e a malha já DOA a normal. A metade que falta (M4) é a costura com o Painter."
relacionados: ["[[05.2-Doacao-de-sombreamento-para-2D]]", "[[06.1-Waves-riscos-e-alvos]]", "[[02.3-Modulo-removivel-e-mapa-de-crates]]"]
---

# W3 — a doação, sem a mão

> **A W2 foi aprovada no smoke** (*"Smoke OK"*, 2026-07-31). Isto é a wave seguinte, e ela **não está
> completa**: três das quatro fatias aterrissaram.

## O que entra

| | |
|---|---|
| **M1** | **`ph2d-light` deixa de estar vazia** — o rig (quantas lâmpadas, onde, com que força) e a conversão graus→vetor passam a ter **um dono**. O Painter re-exporta pelos nomes que já usava. |
| **M2** | **A escultura acende pelo rig do artista** — o matcap procedural da W1 sai; entra o mesmo modelo **RELATIVO** da tinta, o mesmo piso ambiente, as mesmas lâmpadas. |
| **S1** | `Rig::shade_over` — o passe compõe **duas** fontes de normal (sem forma, a aritmética é a que sempre shipou), e o plano atravessa as duas rotas de preview: **0 de 16384 bytes diferem**. |
| **M3** | **O G-buffer** — `MeshRenderer::render_gbuffer` rasteriza normal (no espaço do rig) + **cobertura**. É a *"segunda fonte de normal"* que o `05.2` pede. |

## Os três números que a wave produziu

- **312 de 312** direções diferem de um `sin`/`cos` (desvio geométrico `4,888e-6`) — o rotor de 1° do
  app não é intercambiável, então a dependência `ph2d-light → ph2d-painter-brush` é medida, não
  opinada.
- **`E232,8/D96,6` → `E103,8/D238,8`** ao atravessar a lâmpada principal — a forma reacende.
- **`0,0019`** de desvio entre a lei (a razão relativa, em Rust) e o barro na tela, amostrado pela
  normal que o G-buffer doa.

## A costura S1 — a LEI entrou; o produtor não

`Rig::shade_over` compõe as **duas fontes de normal**, e a composição não é uma escolha:

```text
v = [ form.x − dhx·K ,  form.y − dhy·K ,  form.z ]
```

o *blend UDN* dos normal maps, que **degenera exato nos dois extremos** — sem forma
(`NO_FORM = [0,0,1,0]`) `v` é *literalmente* `[-dhx·K, -dhy·K, 1]`, a expressão que sempre esteve ali,
sem ramo e sem `if`; com tinta plana, `v` é a normal da forma. Não há *"qual fonte manda?"* a
responder: o relevo da pincelada fica **por cima** da inclinação da forma, que é o que a mão faz.

⚠️ A promessa do `02.3` (*"com a flag off o caminho da tinta sai byte-idêntico"*) deixou de ser
promessa: `the_shade_with_no_form_is_the_shade_that_shipped` compara contra a **expressão antiga
congelada verbatim**, não contra uma imagem de regressão.

## A S1 FECHOU — as duas rotas de preview doam, e concordam ao byte

`PainterTool::set_donated_form` instala o plano; as duas rotas o consomem; **pior delta 0, 0 de 16384
bytes diferem**. Os dois lados entraram juntos de propósito — uma doação que aparecesse numa e não na
outra seria uma escultura que ilumina a tinta *até o artista redimensionar a janela*.

**São TRÊS perguntas, e cada uma tem gate:** `impasto_visible` (o passe corre?) · `impasto_fields` (há
planos?) · o early-out por texel (este pixel muda?). Um plano que passe em duas e morra na terceira é
invisível — e verde.

⚠️ **A doação não passa pelo `impasto_show`**: aquele interruptor pergunta *"mostrar o relevo da
TINTA?"*, e a forma de uma escultura não é relevo de tinta.

⚠️ **E o ausente viaja como um BIT** (`has_form`), não como uma tela de zeros: um `z` zero não é
"nenhuma forma", é uma normal DEITADA. A textura é persistente, então sem o bit um documento que perdeu
a escultura seguiria iluminado pela última forma.

## O que NÃO entra, e é a metade que decide

**S2 — o gesto.** `LayerKind::Sculpt3d(...)` **apendado** + o toggle *"iluminada pela forma abaixo"* na
pilha de camadas + o shell rasterizando a malha com uma câmera alinhada ao documento e chamando
`set_donated_form`.

⚠️ **Nada chama `set_donated_form` no produto ainda**, então a doação é inalcançável pelo artista. O
motor está pronto e gateado dos dois lados; o que falta é a mão.

⚠️ **Até a S2, a pergunta *"o módulo vale?"* continua aberta** — não porque a doação não funcione (ela
funciona e está medida dos dois lados), mas porque **o artista não a alcança**. O que ele pode ver hoje
é a escultura acesa pela luz certa.

⚠️ **E o `render_gbuffer` não tem consumidor de produto.** Quem o exercita são os gates. Se o M4 for
cancelado, **ele sai** — infraestrutura sem consumidor apodrece, e a removibilidade é literal.

## Números do estado

- `PROJECT_SCHEMA` **46, intocado** · contrato congelado **intocado** · nenhum id/token/variant.
- **Nenhuma dep externa nova.** As arestas novas são internas: `ph2d-light → ph2d-painter-brush`
  (folha→folha, pelo rotor), `ph2d-render → ph2d-light`, `ph2d-mesh-render → ph2d-light`,
  `ph2d-tool-painter → ph2d-light`, shell → `ph2d-light`.
- ⚠️ **`ph2d-light` passa a ser NÃO-REMOVÍVEL**, e isso é a decisão que `02.3` já tinha tomado para
  esta wave: depois que o Painter passa por ela, arrancá-la quebra o Painter.

## Duas dívidas alheias que fecharam de carona

Nenhuma das duas foi causada por esta linha; as duas são o assunto dela.

1. **O piso da elevação existia duas vezes** — `ELEV_MIN_DEG = 5.0` escrito à mão no painel e o clamp
   do resolvedor. Concordavam, e nada os obrigava: baixar só um daria um slider que anda e uma luz que
   não muda, silenciosamente clampada.
2. **`ph2d_render::IMPASTO_MAX_LIGHTS` era um espelho** com um comentário dizendo *"espelha
   `impasto_rig::MAX_LIGHTS`"* e um gate que o comparava contra o **literal `4`** — um oráculo que não
   podia falhar pelo motivo que alegava. E o gate de constantes do shader do impasto passou a
   **DERIVAR** a string do `AMBIENT`, fechando a direção a que ele era cego.

## O gate que fecha um buraco de W1

`a_mesh_turned_inside_out_lights_and_donates_like_one_that_is_not` — o **flip de verso** (que o
`cull_mode: None` do pipeline existe para tolerar) nunca esteve gateado em lugar nenhum.

⚠️ **Ele nasceu de uma mutação que SOBREVIVEU, e a mutação era INVÁLIDA, não um buraco:** numa esfera
FECHADA com teste de profundidade o verso nunca vence, então o flip é *semanticamente inerte* ali. A
fixture é que não continha o fenômeno. Uma malha virada do avesso contém.

## Como julgar

```bash
env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

A cena imprime o que montou — **se essa linha não aparecer, pare**. Depois:

1. **A esfera acende de cima e da esquerda** (a lâmpada principal do artista, 230°/30°) — não mais o
   matcap com direções cravadas.
2. **`Q`/`E` giram a lâmpada, `R`/`F` a sobem** — a forma tem de reacender, e o lado claro tem de
   trocar de lado quando a lâmpada atravessa.
   ⚠️ **É gesto de SMOKE, não a UI final:** o card de Lighting do Painter já é onde este rig se autora,
   e é ele que o M4 conecta. Um segundo card aqui seria a segunda porta para o mesmo número.
3. **Tudo o mais da W2 tem de continuar igual** — esculpir, os verbos, o espelho, o Ctrl+Z.
4. **Rode uma vez SEM a env var** — é a metade do smoke que prova a inércia: sem a cena armada, o
   frame 2D é byte-idêntico.

Gates de GPU (`#[ignore]`, precisam de adapter; sem ele fazem *skip gracioso*, **que não é verde**):

```bash
cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored
```

**13/13 na RTX.**
