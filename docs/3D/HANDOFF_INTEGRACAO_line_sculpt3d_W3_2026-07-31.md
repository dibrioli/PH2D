---
titulo: "Handoff de integração — line/sculpt3d, W3: a doação chega à tinta"
tags: [modulo/3d, tipo/handoff, assunto/integracao, status/pendente-de-smoke]
status: pendente-de-smoke
modulo: 3D
atualizado: 2026-08-01
resumo: "O rig de luz passa a ter um dono, a escultura acende por ele, a malha DOA a normal, e a tinta chapada sai acesa pela forma. A tecla D é o A/B."
relacionados: ["[[05.2-Doacao-de-sombreamento-para-2D]]", "[[06.1-Waves-riscos-e-alvos]]", "[[02.3-Modulo-removivel-e-mapa-de-crates]]"]
---

# W3 — a doação chega à tinta

> **A W2 foi aprovada no smoke** (*"Smoke OK"*, 2026-07-31). Esta é a wave que o `06.1` marca com ★:
> *"é aqui que o objetivo 1 existe, e é aqui que se decide se o módulo vale"*.

## O que entra

| | |
|---|---|
| **M1** | **`ph2d-light` deixa de estar vazia** — o rig (quantas lâmpadas, onde, com que força) e a conversão graus→vetor passam a ter **um dono**. O Painter re-exporta pelos nomes que já usava. |
| **M2** | **A escultura acende pelo rig do artista** — o matcap procedural da W1 sai; entra o mesmo modelo **RELATIVO** da tinta, o mesmo piso ambiente, as mesmas lâmpadas. |
| **M3** | **O G-buffer** — `MeshRenderer::render_gbuffer` rasteriza normal (no espaço do rig) + **cobertura**. É a *"segunda fonte de normal"* que o `05.2` pede. |
| **S1** | **A LEI** — `Rig::shade_over` compõe as duas fontes, e o passe a consome nas **duas** rotas de preview: **0 de 16384 bytes diferem**. |
| **S2** | **A MÃO** — o shell rasteriza a forma no tamanho do canvas, entrega o plano, e a tecla `D` dá ao artista o A/B. |

## Os números que a wave produziu

- **312 de 312** direções diferem de um `sin`/`cos` (desvio geométrico `4,888e-6`) — o rotor de 1° do
  app não é intercambiável, então a dependência `ph2d-light → ph2d-painter-brush` é medida, não
  opinada.
- **`E232,8/D96,6` → `E103,8/D238,8`** ao atravessar a lâmpada principal — a forma reacende.
- **`0,0019`** de desvio entre a lei (a razão relativa, em Rust) e o barro na tela.
- **pior delta 0, 0 de 16384 bytes** entre as duas rotas de preview com a forma doada.
- **uma doação custa 5,94 ms a 1024²** (1,54 a 512² · 27,72 a 2048² · 123,49 a 4096²).

## A LEI (S1) — duas fontes de normal, sem um `if`

```text
v = [ form.x − dhx·K ,  form.y − dhy·K ,  form.z ]
```

o *blend UDN* dos normal maps, que **degenera exato nos dois extremos** — sem forma
(`NO_FORM = [0,0,1,0]`) `v` é *literalmente* `[-dhx·K, -dhy·K, 1]`, a expressão que sempre esteve
ali, sem ramo e sem `if`; com tinta plana, `v` é a normal da forma. Não há *"qual fonte manda?"* a
responder: o relevo da pincelada fica **por cima** da inclinação da forma, que é o que a mão faz.

⚠️ A promessa do `02.3` (*"com a flag off o caminho da tinta sai byte-idêntico"*) deixou de ser
promessa: `the_shade_with_no_form_is_the_shade_that_shipped` compara contra a **expressão antiga
congelada verbatim**, não contra uma imagem de regressão.

**São TRÊS perguntas, e cada uma tem gate:** `impasto_visible` (o passe corre?) · `impasto_fields` (há
planos?) · o early-out por texel (este pixel muda?). Um plano que passe em duas e morra na terceira é
invisível — e verde.

⚠️ **A doação não passa pelo `impasto_show`**: aquele interruptor pergunta *"mostrar o relevo da
TINTA?"*, e a forma de uma escultura não é relevo de tinta.

⚠️ **E o ausente viaja como um BIT** (`has_form`), não como uma tela de zeros: um `z` zero não é
"nenhuma forma", é uma normal DEITADA. A textura é persistente, então sem o bit um documento que
perdeu a escultura seguiria iluminado pela última forma.

## A MÃO (S2) — como o plano atravessa o shell

```text
sculpt3d_donate_form()          → carimba, rasteriza, deixa no canal
   DonatedForm { news, canvas }   ← o canal, `Vec<f32>` e um par de u32
painter_bridge::dispatch()      → publica o tamanho, instala a notícia
```

⚠️ **O canal não menciona um único tipo do módulo 3D**, e é isso que mantém a removibilidade: o
`painter_bridge` — o único sítio que pode fazer downcast para `PainterTool` — instala uma forma sem
saber o que é uma malha. Apagar o módulo deixa o canal existindo e silencioso.

⚠️ **A câmera é a do ESCULTOR, com o aspecto do CANVAS.** Não há enquadramento novo a inventar: a
pose em que o artista deixou o modelo É a pose sobre a qual ele quer pintar. Consequência honesta: um
viewport 16:9 e um canvas 1:1 não mostram a mesma coisa (o FOV vertical é preservado, o horizontal
segue o canvas — o que uma câmera em perspectiva faz).

### O CARIMBO é o desenho, e o número prova

Uma doação **bloqueia** e custa **5,94 ms a 1024²** (123,49 a 4096²). O carimbo — malha · câmera ·
tamanho do canvas — é o que mantém isso fora do estado permanente: forma parada custa **zero**, e o
artista paga uma vez ao apertar `D`. Sem ele, a tabela inteira seria paga **por frame**.

⚠️ **A câmera entra por BITS, nunca por valor**: `NaN != NaN`, então um carimbo por valor nunca
diria *"nada mudou"* e a leitura bloqueante rodaria todo frame, para sempre, sem nada na tela
dizendo por quê.

### O interruptor tem TRÊS posições, e são três perguntas

`D` cicla **BARRO** (esculpir) → **LUZ** (a forma acende a tinta) → **DESLIGADA** (o controle do A/B).

⚠️ **Barro e Luz são exclusivos por CONSTRUÇÃO, não por política:** a malha é desenhada por cima do
2D (`LoadOp::Load`), então mostrar o barro esconde exatamente a tinta que a doação existe para
acender.

⚠️ **Com o barro fora da tela a cena DEVOLVE o ponteiro** (e a roda). Sem isso a feature seria
inalcançável pelo motivo mais bobo possível: o artista troca para LUZ, vai pintar, e cada clique
orbita um modelo invisível — a doação funcionando perfeitamente, e inútil.

⚠️ **Desligar APAGA o plano instalado**, não emudece: um interruptor que só sabe ligar deixa a tinta
acesa e o artista conclui que o botão quebrou.

⚠️ **A tecla é gesto de SMOKE**, como o `Q`/`E`/`R`/`F` da luz. A UI final é o toggle *"iluminada
pela forma abaixo"* na pilha de camadas, e ele espera a escultura ser uma CAMADA — ver abaixo.

## O que NÃO entra, e é a wave seguinte

**O MODELO DE DOCUMENTO.** A escultura vive num viewport solto (`AppGfx.sculpt3d`): ela não é uma
camada, não é salva, não tem z na pilha, e o `LayerKind::Sculpt3d` que o `02.3` lista como costura
**S2 não foi apendado**.

⚠️ **E não foi de propósito:** um variant que ninguém constrói é um variant morto, e construí-lo de
verdade arrasta a pilha inteira (o painel pinta, o compositor pula, o undo carrega, o save persiste)
— isso é uma wave, não um apêndice. O `02.3` fica correto como contrato; o que muda é *quando*.

**O toggle POR CAMADA** (*"iluminada pela forma abaixo"*) segue o mesmo caminho, e o desenho dele já
está resolvido: a máscara das camadas que optaram entra **no `impasto_fields`, na CPU**, pesando o
plano de forma antes de ele cruzar a costura — o mesmo princípio que já governa o relevo (*só a
ÓPTICA porta; o FOLD não*), então o shader não muda uma linha.

## Números do estado

- `PROJECT_SCHEMA` **46, intocado** · contrato congelado **intocado** · nenhum id/token/variant.
- **Uma dep nova:** `half` na `ph2d-mesh-render` (o G-buffer é `Rgba16Float` e a doação volta pela
  CPU). ⚠️ Não há decodificador escrito à mão de propósito — o workspace já tem UM (privado, na
  `ph2d-flip-render`) e o crate já estava no `Cargo.lock` (dep da `ph2d-tool-color-equalization`); um
  terceiro seria a terceira resposta a uma pergunta só, e esta alimenta os pixels do artista.
- Arestas internas novas: `ph2d-light → ph2d-painter-brush` (folha→folha, pelo rotor),
  `ph2d-render → ph2d-light`, `ph2d-mesh-render → ph2d-light`, `ph2d-tool-painter → ph2d-light`,
  shell → `ph2d-light`.
- ⚠️ **`ph2d-light` passa a ser NÃO-REMOVÍVEL**, e isso é a decisão que `02.3` já tinha tomado para
  esta wave: depois que o Painter passa por ela, arrancá-la quebra o Painter.

## Duas dívidas alheias que fecharam de carona

Nenhuma das duas foi causada por esta linha; as duas são o assunto dela.

1. **O piso da elevação existia duas vezes** — `ELEV_MIN_DEG = 5.0` escrito à mão no painel e o clamp
   do resolvedor. Concordavam, e nada os obrigava: baixar só um daria um slider que anda e uma luz
   que não muda, silenciosamente clampada.
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

## Três defeitos meus que ficam escritos

1. **Uma estimativa minha errava DUAS vezes.** A nota do canvas dizia *"a 1024² são ~4 MB, que o
   artista não sente"*: o plano é `[f32; 4]` = **16 B/texel**, não 4, e **5,94 ms** é quase um terço
   de um quadro. A sonda `measure_a_donation` põe a tabela inteira no lugar da frase.
2. **Um gate meu nasceu VÁCUO.** `only_the_clay_is_drawn` afirmava que os variants do enum são
   distintos — o que o `derive(PartialEq)` garante — e ficaria verde com `draws_clay` cravado em
   `true` (a malha desenhada por cima da tinta em toda posição, a doação inalcançável). O oráculo
   virou o COMPORTAMENTO das duas perguntas, e `draws_clay`/`donates` viraram fatos puros do papel.
3. **Um gate de porta-contra-porta precisa de fixture ASSIMÉTRICA.** Numa esfera centrada, inverter a
   ordem das linhas do plano devolve quase os mesmos números — o ponto cego que os dois lados têm
   quando se movem juntos. Com o modelo deslocado para um canto, a mutação sangra no texel 1987.

## Como julgar

```bash
env PH2D_SCULPT3D_SMOKE=2 cargo run -p ph2d-host-desktop --release
```

A cena imprime o que montou — **se essas linhas não aparecerem, pare**. Depois:

1. **Esculpa alguma coisa** — a esfera é barro, e a luz é a do artista (`Q`/`E` giram a lâmpada,
   `R`/`F` a sobem). Tudo da W2 tem de continuar igual.
2. **Aperte `D` até o terminal ler `LUZ`** — o barro sai da tela e sobra a tela branca.
3. **Pegue o Painter e pinte CHAPADO** (Digital, sem impasto). ⚠️ **É esta a pergunta da wave:** a
   tinta tem de sair **ACESA pela forma que você esculpiu** — sombreada onde a escultura vira para
   longe da lâmpada, clara onde ela vira para a luz.
4. **Aperte `D` de novo** (`DESLIGADA`) — a MESMA tinta fica plana. **Essa diferença é a wave.**
5. **Aperte `D` mais uma vez** e o barro volta: você pode continuar esculpindo, e o passo 3 mostra a
   forma NOVA.
6. **Rode uma vez SEM a env var** — é a metade do smoke que prova a inércia: sem a cena armada, o
   frame 2D é byte-idêntico.

E a cena `=1` (só o barro) continua sendo o smoke da W1/W2.

Gates de GPU (`#[ignore]`, precisam de adapter; sem ele fazem *skip gracioso*, **que não é verde**):

```bash
cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored     # 15/15 na RTX
cargo test -p ph2d-render --release --test impasto_light_gpu -- --ignored   # 6/6 na RTX
```

## Flake conhecida, PRÉ-EXISTENTE

`the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
(`shells/desktop/src/flip_fit_budget_tests.rs`) é kill de **wall-clock em debug**, da `line/FLIP`, e
reprova sob carga paralela — medido nesta linha: 1 de 3 corridas da suíte cheia, **sempre verde
isolado**. Nada desta wave toca o ajuste do Flip. Re-rode sozinho antes de suspeitar de um merge.
