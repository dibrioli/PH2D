# 14 — A ordem de SUPERAR, e o que a medição respondeu antes da primeira linha

> **Ordem do dono, 2026-09-20:** *«faça tudo que for necessário para superar o objetivo. se for
> necessário reescrever o motor então faremos»* — precedida de *«faça uma pesquisa do que de melhor
> existe»* e de *«BATTLE for NEIGHBORVILLE é o alvo»*.

⚠️ **Este documento existe porque a ordem contém uma hipótese — *«reescrever o motor»* — e a §0.0
manda MEDIR antes de aceitar um limite ou um custo.** A medição respondeu, e a resposta não é a que
a ordem antecipava. Nada aqui foi afirmado de memória: cada linha traz o ficheiro onde se confere.

---

## §1 — ⛔⛔⛔ A resposta curta: NÃO é preciso reescrever o motor, e a evidência é estrutural

As leis do pipeline que sete waves construíram **não estão presas ao campo de distância**. São
crates-folha de **zero dependências**, cada uma com um gémeo em WGSL que devolve o próprio texto de
shader:

| crate | dependências | porta de shader |
|---|---|---|
| [`ph2d-material`](../../crates/ph2d-material/) (OpenPBR) | **nenhuma** | `src/wgsl.rs` |
| [`ph2d-style`](../../crates/ph2d-style/) (a camada de estilo) | **nenhuma** | `src/wgsl.rs` — `source()` |
| [`ph2d-bloom`](../../crates/ph2d-bloom/) (o brilho) | **nenhuma** | `src/wgsl.rs` |
| [`ph2d-view-transform`](../../crates/ph2d-view-transform/) (cor e exposição) | **nenhuma** | `src/wgsl.rs` — `SOURCE` |

⭐⭐⭐ **Uma lei que devolve o próprio código de shader serve QUALQUER geometria.** O
[`ph2d-field-gpu/src/paint.rs`](../../crates/ph2d-field-gpu/src/paint.rs) declara-o por escrito:
*«as três leis do pintor já atravessaram»*. Um consumidor novo — um rasterizador de triângulos —
inclui o mesmo `source()` e obtém a **mesma lei, com paridade por construção**, em vez de a
reimplementar.

⇒ *O que a ordem chama «reescrever o motor» é, medido, **ligar um consumidor novo às leis que já
existem**.* A reescrita seria a forma mais cara de ignorar a medição.

---

## §2 — ⛔⛔ O buraco REAL, que nenhum documento deste módulo nomeia

A frase que abriu esta obra é [`01`](01_o_alvo_decomposto.md): *«desejo isso para essa **game
engine**»*. Sete waves entregaram o pipeline — **dentro do modo Render do modelador de campos**.

Medido hoje, esta casa tem **TRÊS** motores de render, e o que sabe fazer PBR não é o que desenha o
jogo:

| motor | desenha | quem o usa | iluminação |
|---|---|---|---|
| [`ph2d-render`](../../crates/ph2d-render/) | **sprites 2D** | **o JOGO** (os componentes do TOP-20) | ⛔ nenhuma |
| [`ph2d-mesh-render`](../../crates/ph2d-mesh-render/) | malhas de triângulos | a escultura | ⛔ matcap + Blinn-Phong |
| [`ph2d-field-render`](../../crates/ph2d-field-render/) | campos de distância | o modelador | ✅ o pipeline inteiro |

⚠️ **A prova de que o jogo é 2D está no tipo**, não numa opinião:
[`RenderInstance`](../../crates/ph2d-render/src/sprite/instance.rs) tem `world_pos: [f32; 2]` e
`size: [f32; 2]`. E nenhuma crate de
[`ph2d-app-components`](../../crates/ph2d-app-components/) referencia `mesh_render` ou
`field_render`.

⇒ **O pipeline do alvo existe e não chega ao produto que o alvo nomeia.** Esse é o buraco, e ele é
de FIAÇÃO, não de motor.

---

## §3 — ⭐⭐⭐ A alavanca medida: as sondas declaram uma propriedade que o código não cumpre

O cabeçalho de [`probes.rs`](../../crates/ph2d-field-render/src/probes.rs) escreve a lei certa:

> *«as sondas são de MUNDO, logo só se refazem quando a cena ou a luz mudam, **nunca quando a
> câmera roda**»*

**O código faz o contrário**, e são três elos:

1. [`paint.rs:513`](../../crates/ph2d-field-gpu/src/paint.rs) — `let b_sondas = device.create_buffer(…)`,
   um buffer **novo a cada quadro**;
2. [`paint.rs:413`](../../crates/ph2d-field-gpu/src/paint.rs) — `let p_assa = (pintor.ao_rays > 0).then(…)`,
   o assar só corre com o ricochete;
3. [`gpu_frame.rs:317`](../../crates/ph2d-app-field3d/src/gpu_frame.rs) — `ao_rays: if assente && sonda.ricochete`,
   e o ricochete só corre no quadro **parado**.

⇒ **A luz indirecta é calculada e deitada fora, imagem após imagem, e não existe enquanto a mão
mexe.**

⚠️ *Isto não é um defeito de quem a construiu* — a wave que a trouxe tinha de provar a lei antes de
a tornar barata, e provou-a. É a dívida que sobra, e o cabeçalho dela já a nomeia.

### O que a indústria faz, medido nos cinco (2026-09-20)

| motor | como paga a luz indirecta |
|---|---|
| **Unreal** (Lumen) | traça contra *proxies* + *surface cache*; amortiza |
| **Godot** (SDFGI) | `frames_to_converge`, `frames_to_update_lights`, `gi/use_half_resolution`, TAA, FSR |
| **Unity** (HDRP) | volumes de sondas, SSGI, TAA, upscaling |
| **Blender** (EEVEE Next) | raios de ECRÃ + redutor espacial **e temporal**, TAA com reprojecção, `gi_diffuse_bounces = 3` |
| **Frostbite** (o ALVO) | ⭐ **Enlighten: transferência PRÉ-CALCULADA**, lightmaps assados por um traçador offline |
| **PH2D** | tudo, à resolução cheia, num quadro — **ou nada** |

⭐⭐⭐ **Nenhum deles calcula a luz toda em cada imagem, e o ALVO menos que todos.** O
[SIGGRAPH 2026](https://advances.realtimerendering.com/s2026/index.html) confirma a direcção: uma
*memória de radiância* (EA), *orçamento variável de raios por pixel* (Activision) e *upscaling por
aprendizagem* (Sony).

⇒ *O alvo do dono é alcançado com luz que PERSISTE, não com luz recalculada.* As sondas de mundo
**são** o nosso lightmap — só lhes falta sobreviver ao quadro.

---

## §4 — ⭐⭐ A vantagem que nenhum dos cinco tem, e porque ela é estrutural

- A **Unreal** traça contra um campo de distância **pré-cozido por malha**, com o erro disso.
- O **Godot** traça contra **voxels**.
- O **EEVEE** traça contra o **ECRÃ** — não vê o que está fora do enquadramento (limitação
  declarada no manual dele).
- O **Frostbite** usa transferência **pré-calculada**: a cena tem de ser estática.

⭐ **Nós traçamos contra a forma verdadeira**, exacta e analítica, e sobre geometria que o artista
está a mudar naquele instante. ⇒ *amortizada como eles amortizam, a nossa luz indirecta fica mais
correcta que a deles, porque a fonte não é uma aproximação.*

---

## §5 — ⭐⭐⭐ A DECISÃO DO DONO, dada em 2026-09-20 — e ela REFUTA a pergunta que esta secção fazia

Esta secção perguntava se o jogo devia «passar a 3D». **A pergunta estava mal posta, e o dono
respondeu-a com a arquitectura inteira:**

> *«A game engine é 2d no sentido de que os games produzidos nela não terão a terceira dimensão do
> ESPAÇO … Contudo os objetos dentro do canvas 2d podem ser imagens mas também podem ser objetos 3d
> reais com iluminação 3d real e animação 3d real, contudo projetada no canvas 2d do ponto de vista
> FUNCIONAL. … posso ter um catavento animado em 3d mas seu collider é 2d e ele interage como objeto
> 2d. … teremos a simplicidade do 2d com o melhor da aparência 3d. Se o usuário quiser usar o 3d
> para transformar tudo em imagem 2d também será possível.»*

⇒ **o jogo ser 2D não é uma limitação a remover: é o DESENHO.** A jogabilidade, a física e o colisor
ficam 2D (é o que os torna simples e determinísticos); o que sobe a 3D é a **APARÊNCIA**.

⛔⛔ **E a minha conclusão do §2 — *«falta ao jogo um caminho 3D»* — era o erro que daí vinha.** O
`RenderInstance` ser `[f32; 2]` **está certo**: a posição do objeto no mundo do jogo é mesmo 2D. O
que falta não é uma dimensão no jogo; é **a aparência do objecto poder vir de uma malha**.

⭐⭐⭐ **E isto já tem documento de arquitectura ACEITE, de 2026-07-30**, que cita o pedido original
do dono à letra: [`docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`](../3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md)
— *«o objeto será uma sprite que tem como filho uma malha 3D que emprestará à sprite o seu shader
avançado»*. Ele define o modelo (uma hierarquia só), o G-buffer intermédio e **as duas rotas**:

| rota | quando | custo |
|---|---|---|
| **A — ASSADO** (o padrão) | a pose da malha não muda em runtime | o de um sprite normal-mapeado; **roda em telemóvel** |
| **B — AO VIVO** (opt-in) | a forma **gira, deforma** ou muda de geometria | um passe de geometria por objecto vivo |

*O catavento do dono é a rota B.* E a frase *«se o usuário quiser transformar tudo em imagem 2D»* é
a rota A.

### O que está CONSTRUÍDO, medido hoje contra o código

| peça | estado |
|---|---|
| **Rota A** — malha assada em canais do sprite, **RE-ILUMINÁVEL** | ✅ [`baked_form`](../../crates/ph2d-form-donation/src/baked_form.rs): guarda `base` + `form = [nx,ny,nz,peso]` + `rig` |
| `BakedForm` como componente do ECS, no catálogo | ✅ [`ph2d-ecs/src/baked_form.rs`](../../crates/ph2d-ecs/src/baked_form.rs) — `ObjectKind::Sculpt3D` |
| A luz 3D a acender um sprite, **dentro do motor 2D** | ✅ `ImpastoLightPass` em [`ph2d-render`](../../crates/ph2d-render/src/impasto_light.rs) |
| Hierarquia ECS pai/filho | ✅ ADR-0110 |
| **Componente `Mesh3D` / `MeshShading`** | ⛔ **não existe** (medido: zero ocorrências no `ph2d-ecs`) |
| **Rota B — a malha filha a rasterizar por quadro** | ⛔ **não existe** |
| **Animação 3D** do objecto | ⛔ o esqueleto que existe é 2D |
| O G-buffer completo do `02.2` (`normal·depth·AO·cavity·material`) | ⚠️ hoje só **normal + peso** |
| A lei que acende o sprite | ⚠️ é um **modelo de TINTA**, não PBR — medido em [`impasto_light.wgsl`](../../crates/ph2d-render/src/shaders/impasto_light.wgsl): difuso envolvido + especular por LUT, **sem GGX e sem conservação de energia** |

⭐⭐⭐ **E é aqui que se SUPERA, com a frase do próprio `02.2`:** a rota B é *«o efeito que nenhum
sprite normal-mapeado comum consegue»*. Unity, Godot e Unreal fazem 2D com **normal maps fixos** —
gira-se o sprite e a luz não acompanha, porque o mapa é uma fotografia da forma. **O nosso é
re-derivado da forma verdadeira, por quadro.** Nenhum deles entrega isso, e a razão é a mesma do §4:
eles não têm a forma em tempo de execução, e nós temos.

---

## §6 — O plano, por ordem de dependência

### `F1` — ⭐⭐⭐ A luz sobrevive ao movimento *(serve as duas leituras)*

O que a `W9` deixou nomeado, agora com a causa identificada no §3.

1. **As sondas persistem entre quadros.** Buffer próprio, reassado quando a **cena** ou a **luz**
   mudam — nunca quando a câmera roda. *A lei já está escrita no cabeçalho delas; falta cumpri-la.*
2. ✅ **A oclusão a meia resolução — CONSTRUÍDA em 2026-09-24** no quadro de MOVIMENTO
   ([`03` §W9](03_o_plano.md), «a oclusão a passo»): nó `109 → 55,5 ms`, vaso `18,1 → 11,7`, sem
   halo (gate no nó e na rosca). ⚠️ O passo `1` também já está feito (as sondas guardadas na placa).
   *(A redacção original: «já medida e não construída… traz uma classe de artefacto própria (halo na
   descontinuidade de profundidade), que é o que a wave tem de medir».)*
3. **Reprojecção temporal.** O quadro anterior é informação; hoje é deitado fora.

- **Régua:** a luz indirecta e a sombra **ligadas** com a câmera a mexer, dentro do orçamento de
  `16,7 ms` a `1920×1080`.
- **Kill-criterion (§5 da DIRETIVA, declarado ANTES do build):** se depois dos passos `1` e `2` o
  quadro de movimento com luz ligada não descer abaixo de `16,7 ms` nas cenas que hoje já são
  nítidas, **a `F1` não existe nesta forma** e o problema muda de classe (passa a ser de
  resolução/upscaling, não de amortização).
- ⚠️ **Antes de tocar em código:** re-medir a linha de partida com a máquina abaixo de `load 10`,
  em `--release`, com a ociosidade real ao lado — as duas armadilhas que o
  [handoff de 2026-09-20 §10.3](../3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-20.md)
  já pagou.

### `F2` — O material deixa de ser uma cor *(serve as duas leituras)*

- **Texturas.** Hoje um objecto tem **uma cor só** — medido: a
  [`ph2d-material`](../../crates/ph2d-material/src/lib.rs) não tem uma única entrada de textura ou
  de UV. É a diferença mais visível numa comparação lado a lado com qualquer dos cinco.
- **Os campos que faltam do OpenPBR: `21` de `41`** (contados no `struct`). Ficam de fora
  `transmission_*` (vidro), `fuzz_*` (tecido), `thin_film_*` (iridescência), a anisotropia e
  `geometry_opacity`.

### `F3` — ⭐⭐⭐ A ROTA B: o objecto 3D **ao vivo** dentro do canvas 2D

*É o catavento.* O `02.2` já a desenhou; falta construí-la.

1. **Os componentes `Mesh3D` + `MeshShading`**, filhos de um `Sprite` na mesma hierarquia — logo
   herdam selecção, nome, undo, save e os gestos de linha **de graça** (ADR-0110).
2. **O passe que rasteriza a malha filha para o G-buffer**, do tamanho do rectângulo do sprite, com
   **dirty flag**: só re-rasteriza se a pose, a malha ou a câmera mudarem. O
   [`ph2d-mesh-render`](../../crates/ph2d-mesh-render/) já rasteriza triângulos com `wgpu`, e o
   render para textura fora de ecrã já existe (`game_rt`).
3. ⚠️ **A malha NUNCA é desenhada na tela** — quem aparece é o sprite. É o que mantém a composição,
   a ordem de profundidade e o colisor 2D intactos.

- **Régua:** girar um objecto e a luz acompanhar — *o que um normal map fixo não consegue*. E o
  colisor continua 2D, com o mesmo hash determinístico da física.
- **Pergunta em aberto do `02.2`, a medir e não a opinar:** quantos objectos em rota B cabem no
  orçamento, e a que fracção da resolução do sprite o G-buffer pode viver.

### `F4` — A lei que acende o sprite passa a ser a BOA ✅ **ENTREGUE em 2026-09-21** (a **1.ª obra** do [`15` §5](15_as_metas.md))

⛔⛔ **A frase que aqui esteve — *«hoje quem acende um `BakedForm` é o `ImpastoLightPass`, um modelo
de tinta»* — era verdade no dia em que foi escrita e é FALSA desde 2026-09-21:** a
[`Lei::Forma`](../../crates/ph2d-form-donation/src/lei_da_luz.rs) (o OpenPBR) é o **valor de
fábrica**, e `PH2D_FORM_PBR=0` é o que volta à de tinta. O smoke do dono está **aprovado**.

⚠️ **E ela custou mais do que esta secção previa:** pôr a lei boa a acender o sprite abriu **cinco**
reports de *«o bake não é idêntico ao que se vê em 3d»*, com cinco causas empilhadas — o modo do
visor, a lei do bake, a **matéria**, a **curva sRGB** e a **selecção**. Tudo em
[`../Render/01_o_assado_e_identico_ao_que_se_ve.md`](../Render/01_o_assado_e_identico_ao_que_se_ve.md).

⏳ **O que FICA desta obra:** a escolha da lei ser **por objecto e gravada** (hoje é global e não
viaja no ficheiro), mais a cauda medida do [`15` §7](15_as_metas.md).

O modelador tem o OpenPBR inteiro, e era essa a distância a fechar:

⇒ **É aqui que o §1 se paga:** as leis são crates-folha com gémeo em WGSL, logo o passe do sprite
passa a incluir `ph2d_material::wgsl` — e ganha, de uma vez, material a sério, a subsuperfície (a
folha com o sol atrás, que é *a assinatura do alvo*), a camada de estilo e a gestão de cor.

⛔ **Sem reescrever nada**: o `ImpastoLightPass` fica, porque é a lei certa para a TINTA do Painter.
O que muda é o sprite com forma passar a ter um passe próprio.

### `F5` — Animação 3D do objecto

O catavento **gira**. Hoje o esqueleto desta casa é 2D
([`ph2d-skeleton`](../../crates/ph2d-skeleton/)). Uma malha filha animada precisa de pose 3D por
quadro — e ela é exactamente o que faz a rota B valer a pena, porque é quando o normal map fixo
falha.

---

## §7 — ⛔ O que este plano NÃO faz, e porquê

- **Não persegue GI dinâmica a 60 Hz.** O alvo não a tem (§3): o Frostbite pré-calcula. Persegui-la
  seria gastar no sítio que o próprio alvo evitou.
- **Não constrói Nanite nem streaming.** [`02` §4](02_o_estado_da_arte.md) já os recusou por
  medição, e o alvo não os usa.
- **Não compra o jogo para medir, ainda.** A pergunta técnica que a compra responderia
  (*como é que o alvo ilumina?*) já está respondida — Enlighten, pré-calculado. O que a compra
  compraria é **calibração fina**, e calibrar antes da `F1` seria afinar um motor que ainda vai
  mudar de forma. ⚠️ Ele também corre mal em Linux (ProtonDB **prata**, a tender para bronze).

---

## ⛔ Recusas MEDIDAS desta página

| recusa | mecanismo | onde |
|---|---|---|
| **reescrever o motor** | as leis são crates-folha de zero dependências com gémeo em WGSL; falta um consumidor, não um motor | §1 |
| **levar o ray-march de SDF para o jogo** | o custo é `instruções × passos × píxeis`, e cada objecto acrescenta instruções que TODO pixel corre — `308` para `21` objectos | §2 |
| **GI dinâmica a 60 Hz como alvo** | o alvo (Frostbite/Enlighten) **pré-calcula**; a nossa persistência é o equivalente | §3 |
| **comprar o BfN agora** | a pergunta técnica já está respondida de graça; a calibração vem depois da `F1` | §7 |
