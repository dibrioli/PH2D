# As três features do modelador 3D, na escultura — e a gravidade em Y

> **Ordem do Enio, 2026-09-08**, depois do smoke do filtro de tecido:
> *«smoke parece OK. Contudo precisamos de algumas features antes de dar a
> opinião definitiva. Parece que a gravidade está em z mas neste app deve ser em
> y. Veja o módulo de Modelagem 3d. Ele tem viewports (4), gizmos de
> transformação e o gizmo da viewport. Traga esses features para esse módulo.»*

Linha `line/sculpt3d`. Quatro entregas, quatro commits.

---

## §0 — A lei que atravessa esta jornada

**Nenhuma das três features foi COPIADA.** As três já eram lei do módulo
vizinho (`shells/desktop/src/field3d_*`), e as três eram lei **quase**
agnóstica de câmera: o que as prendia ao modelador era a `ph2d_field_render::Orbit`.
A escultura tem a `ph2d_mesh_render::Camera3d` — outra órbita, com trava de
polo.

⇒ em cada caso o que se fez foi **estreitar o que a lei pede a uma câmera** e
dar-lhe um segundo consumidor:

| lei | o que ela passou a pedir | ficheiro |
|---|---|---|
| as seis bolas de eixo | `(direita, cima, para-o-observador)` | `field3d_navball::balls_from_basis` |
| as treze alças | três perguntas (`GizmoCamera`) | `field3d_gizmo::project_with` |
| a divisão do canvas | **nada** — já era pura | `field3d_layout` (lida tal e qual) |

⛔ **A `line/3DModeling` está VIVA**, então o custo no território dela foi
mantido mínimo e append-only: uma função nova + a antiga a delegar, uma
visibilidade (`Standard::yaw_pitch`), um `#[cfg(test)]` retirado
(`Anchor::global`), e o trait `GizmoCamera` com os internos a passarem a
recebê-lo. *Copiar as ~130 linhas de projecção do gizmo daria duas ideias de
onde uma alça está, e o sintoma da que envelhecesse é um gizmo que agarra ao
lado do que ele diz mover.*

---

## §1 — A gravidade cai contra o CIMA da câmera (`2cf887625`)

O report dizia *«parece que está em z»* e o defeito era de **proveniência**: o
`[0, 0, −1]` escrito no `cloth_filter_step_of` é a convenção do **alvo** da
espec §7 (que é `Z` para cima). Esta casa é **`Y` para cima** — a `Camera3d`
roda o `yaw` em torno do `+Y`, e o `field3d_navball` já o diz por escrito para
o módulo vizinho.

**Medido no enquadramento de omissão:** com o eixo antigo o pano **subia** na
tela (`−0,99 px`); com a cura desce (`+2,67 px`).

⚠️ **A cura não é um literal novo.** O «cima do mundo» tinha **quatro cópias sem
nome** dentro da `camera.rs` (`view`, `frame`, `ray_through`, `screen_basis`) —
logo nada no repo a que outra metade do app pudesse perguntar *«para que lado é
cima aqui?»*. Nasce `Camera3d::UP`. *Uma convenção sem nome é copiada da última
coisa que se leu.*

⚠️ **O braço da VISTA já estava certo**, e é por isso que o report diz
*«parece»*: ali o baixo é o `−cima do ECRÃ`. *Metade de uma lei correcta esconde
a outra metade errada.*

**Gate** `a_gravidade_do_filtro_cai_para_baixo_na_tela` — e a régua **não**
pergunta qual é a constante (seria a mesma linha escrita duas vezes, verde por
construção): ela corre a lei sobre uma malha, projecta o centróide antes e
depois, e exige que o `y` da tela **cresça**. O **controlo** é o valor que
shipava: com `[0,0,−1]` o mesmo gesto tem de dar o sinal oposto.

---

## §2 — O gizmo da viewport (`5118c6654`)

Seis bolas de eixo no canto do quadrante activo. **Arrastar orbita, clicar
salta** — e o clique é o pen-**up** sem movimento, nunca o down: a própria
pesquisa da Autodesk que criou o ViewCube mediu que arrastar é ~2× mais rápido
*«independentemente da representação»*, então saltar no down faria todo arrasto
começar com um corte de câmera.

Teclas `Numpad1/3/7` + `Ctrl` para a oposta — a tabela do vizinho
(`field3d_views::view_for_key`). **A memória de dedo é do Blender; os EIXOS são
os nossos.**

⛔ **A TRAVA DO POLO é a divergência declarada.** A `Top` pede `pitch = π/2` e
esta câmera não o guarda (ali a `look_at` produz `NaN`): ela guarda
`π/2 − 0,01`, **`0,57°`** de desvio (`cos 0,999950`, impresso pelo gate). Daí
`aim_of`: *o reconhecimento compara contra o valor que a câmera de facto guarda,
nunca contra o ideal que foi pedido.*

Portas novas na `Camera3d`: `aim`, `clamp_pitch`, `view_axis`.

**Seis gates**, repartidos de propósito: três a LEI, um a COSTURA com GPU (um
teste que **clica** na bola — esta casa pagou os quatro chips pintados,
indexados e mortos sob o ponteiro no `seam_bool`), um o CENSO de fonte das três
portas do despacho.

⚠️ **Uma mutação sobreviveu à primeira redacção**: saltar para a vista já no
pen-down deixava *«o yaw mudou»* verde, porque a órbita muda-o na mesma. A régua
certa é de **onde** a órbita partiu — a fixture passa a começar numa orientação
livre e exige o valor exacto.

---

## §3 — Os quatro viewports (`a3c54ecd2`)

`Ctrl+Alt+Q` abre e fecha — a MESMA tecla do módulo vizinho (§7-bis). Topo · direita · frente · a vista do artista, com
o artista no canto de **baixo à direita** (a disposição do Blender: é onde a mão
dele já está). Cada quadrante tem a sua câmera; o gesto corre na **activa**.

### ⭐⭐⭐ E a peça passou a ser desenhada na ÁREA, não na JANELA

O `present.rs` mandava `(window.width, window.height)`: a malha era rasterizada
por baixo dos painéis, da fila de ferramentas e das duas réguas — o **mesmo**
defeito que o Enio reportou ao módulo vizinho em 31/08 (*«a viewport ainda não
se encaixa na área correta para ela — veja que atravessa as réguas»*).

O campo `viewport: (u32, u32)` **morreu** e passou a ser derivado da área
publicada, da divisão e do quadrante activo. ⇒ o pick e o desenho passam a ser
duas metades da mesma conta, e só concordam por duas portas: `to_view`
(janela → vista) e `project_window` (mundo → janela).

### O substrato: `ph2d_mesh_render::ScreenRect` (`2ea127ef9`)

Os três passes da crate passaram a aceitar um sub-rectângulo (`render_in`,
`render_gbuffer_in`, `render_ssao_in`), com os antigos a **delegar** com
`ScreenRect::full(size)` — zero churn nos chamadores.

Três metades, cada uma com o seu modo de falha:

1. **`set_viewport` E `set_scissor_rect`**, por uma porta. O viewport
   *transforma* e não corta — com ele sozinho a geometria que sai pelo lado do
   frustum continua a escrever no quadrante vizinho.
2. **O aspecto é o da VISTA.** É a metade que uma implementação apressada
   esquece, e ela é invisível numa vista só: com quatro quadrantes o alvo
   continua deitado e cada vista fica quase quadrada, então o aspecto do alvo
   esticaria a peça **nas quatro por igual** — que é como um erro de escala
   deixa de se parecer com um erro.
3. **O uniform do SSAO ganha a ORIGEM**, e o WGSL subtrai-a. O
   `@builtin(position)` é absoluto no framebuffer **mesmo com `set_viewport`**:
   sem isto um pixel do quadrante de baixo à direita reconstruiria o NDC do
   canto do ecrã. *Um `set_viewport` move o rasterizador e não move a aritmética
   do shader.*

⚠️ A cor é `LoadOp::Load` e a profundidade `Clear`, e a assimetria é
load-bearing: a vista `k+1` apaga a profundidade da `k` (um clear não obedece ao
scissor) e **pode** — a cor da `k` já foi escrita.

**Quatro gates de GPU** (`gpu_viewport.rs`), e o primeiro é o que importa:
`render_in` no alvo inteiro é **byte a byte** o `render` de sempre (0 de 131 072
bytes).

### A invariante da câmera

O campo `camera` manda para o viewport **activo** (tem ~30 leitores neste
módulo, e todos querem sempre a vista em que a mão está) e `vp_cams[activo]`
está **velha**. Guardar e pegar num sítio só (`set_active_vp`), ler por uma
porta só (`cam_of`).

⛔ **Ela mordeu na primeira corrida**: `toggle_split` fazia `rebuild` e **só
então** `vp_active = n−1`, o que deixava a `camera` a apontar para o quadrante
`0` — abrir a divisão punha a vista de **topo** no canto do artista. O gate leu
`[Top, Right, Front, Top]`. *Uma invariante com dois escritores tem de ser
escrita numa transacção só.*

### ⛔⛔ O gate do teclado, que nasceu de um defeito meu

Liguei a divisão à crase **sem modificador** — e a crase sozinha já tinha dono
no mesmo ficheiro (abre o painel; o roteiro da `=37` manda usá-la). O braço do
painel corre antes e devolve `true`: a tecla nova **compilava, não dava warning
nenhum, e nunca corria**. *Nenhuma sonda deste repo vê isto* — o censo de ids
mede REGISTO e os `seam_*` medem que o clique chega à ferramenta.

⇒ `nenhuma_tecla_e_reivindicada_duas_vezes_com_a_mesma_guarda`, com **duas
redacções erradas antes da certa**:

* comparava guardas como **texto**, e a mutação SOBREVIVEU (`!ctrl && !shift` e
  `` são strings diferentes). Hoje cada guarda é **avaliada** sobre as quatro
  combinações de `(ctrl, shift)` e o gate exige conjuntos **disjuntos**;
* acusou o `K::KeyJ`, que é **vivo** — as duas ocorrências não colidem porque
  uma vive dentro de um `if shift`. *Um censo textual que não conhece o contexto
  acusa o vivo, e a cura que ele manda aplicar é a errada.* Hoje compara só arms
  na mesma indentação, e o ponto cego que isso deixa está **nomeado** no doc.

---

## §4 — O gizmo de transformação (`§4`)

Setas por eixo, quadrados por plano, anéis por eixo, o disco e o anel de vista,
e o punho de tamanho — a lei do vizinho, com esta câmera.

### ⭐⭐ O que ele acrescenta não é chrome: são gestos que não existiam

O transform modal entrega **três** gestos (mover no plano da tela, rodar em
torno da vista, escalar). O kernel sempre soube fazer mais: `Move { delta }`
aceita qualquer vector e `Rotate { axis, radians }` qualquer eixo. O que faltava
era **como pedir** — sem alças, mover só ao longo de `X` ou rodar só em torno de
`Y` eram **inexprimíveis por gesto nenhum**. É a mesma forma do buraco que o
picker de filtros curou na W9b.

⚠️ **A restrição é uma projecção do DESLOCAMENTO, nunca do ponto**: prender o
*destino* ao eixo faria a peça saltar para cima do eixo no primeiro pixel.

⚠️ **O eixo de um anel é virado para o OBSERVADOR antes de descer à peça.** A
varredura que dá o ângulo é medida no **ecrã**, então com o eixo a apontar para
trás o sinal cru inverteria a rotação — *«a argola de trás roda ao contrário»*.

⚠️ **Sem alça o transform corre livre**, que é o gesto modal de sempre. *Um
gizmo que tomasse conta do botão inteiro tiraria uma ferramenta que funciona
para dar outra.*

### O pivô: uma porta, dois leitores

O gizmo precisa do pivô **a cada quadro e sem sessão**, e a
`MaskTransform::begin` aloca três vectores do tamanho da parte livre. Nasce
`ph2d_sculpt3d::free_pivot`, e a **`begin` chama-a** em vez de repetir a
fórmula: custa-lhe uma travessia a mais, uma vez por gesto, e compra que o gizmo
e o barro nunca possam discordar sobre onde é o centro. *Duas cópias de um pivô
dão um gizmo desenhado num sítio e uma rotação em torno de outro, e o artista lê
isso como «o gizmo está torto».*

### A cerca que saiu

O `sculpt3d_transform` abria com *«Por que não há gizmo»*, e a razão era exacta
para o dia. ⚠️ **A premissa dissolveu-se porque um vizinho pagou a wave** — ⇒
§0.0: *quem move o número que tornava algo inalcançável tem de reconferir a
nota.*

**Cinco gates**, e o do anel teve a régua corrigida: a primeira redacção
comparava **magnitudes** entre a argola de `Z` e a de vista, e elas nunca foram
iguais (as duas argolas têm **raios diferentes**, logo o mesmo arrasto em pixels
varre ângulos diferentes). A régua certa é o **sinal**, e o discriminador é a
troca de lado: o mesmo arrasto, visto de frente e de trás, tem de rodar a peça
em sentidos **opostos** no mundo. Mutação (tirar o sinal): **morta**.

---

## §5 — O smoke

```
env PH2D_SCULPT3D_SMOKE=38 cargo run -p ph2d-host-desktop --release
```

O roteiro tem dez passos e cobre as três features; os rótulos citados saem do
**motor** (`Standard::ALL`, `TransformKind::ALL`), nunca de prosa.

---

## §6 — ABERTO, com o gatilho de cada item

| item | porquê ficou | o gatilho |
|---|---|---|
| a divisão dos viewports **no painel** | o gesto é a tecla, que é o idioma deste módulo (os verbos são dígitos, o espelho é `X`/`Y`/`Z`) | o primeiro report do dono a dizer que não achou a divisão. O molde é o `SCULPT3D_WIREFRAME` — seis sítios |
| **um gizmo por quadrante** | *um gizmo por quadrante seriam quatro respostas à mesma pergunta* — o gesto acontece numa vista de cada vez | ninguém; é decisão de desenho, e a do vizinho é a mesma |
| o **selector de referencial** (Global/Local) do gizmo | a `ph2d_mesh::Pose` de uma escultura **não tem rotação**, então os dois dariam os mesmos três vectores — a mesma medição que mantém o *World* fora do filtro de tecido | o dia em que uma peça puder ser rodada |
| **snap** e o número digitado no gesto | o vizinho tem-nos (`field3d_typed`); aqui seriam wave própria | o dono pedir precisão numérica no transform |
| a **moldura do activo** com uma vista só | o pintor é no-op abaixo de dois rectângulos, e uma moldura permanente seria ruído | — |

⛔ **RECUSAS MEDIDAS**

| o que foi tentado | o que a medição deu |
|---|---|
| gatear o tamanho do uniform do SSAO por um **literal** (`96`) | o campo novo reprovou-o com uma frase sobre a versão anterior do shader; hoje o gate **soma os campos declarados no WGSL** |
| `coverage()` do `gpu_render` a somar três `u8` | pânico em debug, **envolvimento silencioso** em release — `(86, 85, 85)` soma `256`, que módulo 256 é `0`, e a régua contava-o como **fundo** |
| comparar guardas de tecla como **texto** | a mutação sobreviveu; a pergunta certa é *«podem as duas ser verdadeiras ao mesmo tempo?»* |
| comparar **magnitudes** entre a argola de eixo e a de vista | nunca foram iguais (raios diferentes); a lei é sobre o **sentido** |

---

## §7 — ⚠️ Um achado de PAREDE, pré-existente e não desta jornada

O `cleanroom-sweep.sh` sobre a crate `ph2d-sculpt3d` sai **`✗`**:

```
crates/ph2d-sculpt3d/src/verb_layer_front_face_tests.rs:16
```

Aquela linha cita **quatro nomes de ficheiro internos do alvo restrito**. Ela
entrou no commit `1e03095b1` (*«cada ferramenta lembra a própria afinação»*),
muito antes desta jornada — `git show --stat` confirma que nenhum dos quatro
commits de 2026-09-08 lhe tocou, e a varredura sobre **só** o que eles
escreveram sai limpa.

⚠️ **É a espécie que o [`ACHADO_proveniencia_por_nome_interno`](cleanroom/ACHADO_proveniencia_por_nome_interno.md)
cataloga** (~460 notas no repo inteiro). Fica **registado e não curado aqui**:
reescrever um doc-comment de outra wave no meio desta seria misturar duas
histórias num diff, e a cura tem dono e endereço próprios.

⛔ **E o modo como ele quase passou despercebido vale mais que ele:** a corrida
foi `bash scripts/cleanroom-sweep.sh … 2>&1 | tail -2`, e **o `tail` destruiu o
código de saída** — a linha seguinte da cadeia correu como se estivesse verde. É
exactamente o que o `CLAUDE.md` §2 avisa. *Um `tail` é uma JANELA, nunca um
veredito.*

---

## §7-bis — ⛔⛔⛔ O SEGUNDO REPORT DO MESMO DIA, e os dois defeitos eram meus

> *«o atalho das 4 viewports não funciona. E já existia um atalho para isso, se
> não me engano Ctrl+Alt+Q.»*
> *«com as ferramentas de transformação ativadas anulo a rot do canvas. isso não
> pode acontecer.»*

### (a) A tecla estava MORTA — e o meu próprio gate não a viu

O `sculpt3d_key` tem um **catch-all**:

```rust
if ctrl {
    if code != K::KeyZ { return false; }   // ← daqui para baixo, Ctrl+ é dele
    …
}
```

Ele existe por um bom motivo (sem ele um `Ctrl+1` dispararia o verbo do dígito
`1`), e o preço é que **todo braço que exija `ctrl` e venha depois dele está
morto**. O meu `Ctrl` + crase vinha depois. ⚠️ **E levou junto o
`Ctrl+Numpad1/3/7`** — metade das vistas nomeadas —, sem um warning.

⛔⛔ **O gate que escrevi nesse mesmo dia contra esta família não o apanhou.** Ele
pergunta *«duas teclas iguais?»*, e a lei verdadeira é *«esta tecla é
ALCANÇÁVEL?»* — duas claims é **uma** das formas de uma ficar inalcançável; um
`return` a montante é outra, e a primeira régua é cega à segunda. *Uma régua que
mede um caso de uma família lê-se como se medisse a família.*

⇒ gate novo, `nenhum_braco_de_tecla_vive_debaixo_de_um_catch_all_do_mesmo_modificador`.
Mutação (repor a tecla debaixo do catch-all): **morta**, com a linha exacta.

### E a CURA da tecla não foi reposicioná-la

**O Enio tem razão na segunda metade também: o atalho já existia.** O
`field3d_quad_key` faz exactamente isto para o canvas vizinho, com a tecla do
Blender (*Toggle Quad View*). Ter duas gramáticas para *«dividir a janela 3D»*
nos dois módulos do mesmo app é a memória de dedo partida ao meio.

⇒ `App::sculpt3d_quad_key`, espelho do vizinho à letra — incluindo a lei dos
**três modificadores exigidos por nome** (um `Ctrl+Alt+Shift+Q` é de outra
pessoa) e a guarda de ponteiro. ⚠️ Ela mora **fora** do `sculpt3d_key`, no
despacho, e é isso que a mantém viva.

### (b) Armar o transform apagava a órbita — e é PRÉ-EXISTENTE

O braço do transform tomava o botão esquerdo **sem perguntar se o raio acertou
alguma coisa**. Com a ferramenta armada, o gesto mais comum do mundo —
*arrastar no vazio para girar a peça* — deixava de existir.

⚠️⚠️ **O `Sculpt3dScene::aim` já declarava a lei no próprio doc:** *«`false` se o
raio não achou nada (e aí o botão vira órbita, **como em todo gesto**)»*. Era
«todo gesto» **menos este**. *Uma porta que documenta a regra e um chamador que
não a honra é a forma mais barata de um defeito ficar à vista de toda a gente.*

⛔ **E a cerca que o justificava dissolveu-se com a wave do gizmo.** A nota
daquele braço dizia que um botão que *«às vezes transforma e às vezes gira a
câmera — conforme o que estava sob o cursor»* seria o mesmo gesto com dois
sentidos. Isso valia enquanto não houvesse **nada na tela** a dizer o que está
sob o cursor; com as alças desenhadas, o que era ambiguidade passou a ser uma
coisa que se vê. ⇒ §0.0 outra vez.

⚠️ **A ALÇA é perguntada primeiro**, e não é ordem arbitrária: a ponta de uma
seta espeta-se no **vazio**, e perguntar pelo barro antes tornaria essa alça
inalcançável.

⏳ **O FILTRO fica como está, e é decisão declarada**: com ele armado, arrastar
no vazio continua a filtrar. Um filtro age na peça **inteira** e a posição do
dedo só importa para a âncora do aperto — exigir-lhe barro por baixo tiraria um
gesto que funciona. O gatilho para rever é o dono reportar o mesmo sobre ele.

---

## §7-ter — ⛔⛔⛔ O TERCEIRO REPORT: **um uniform, quatro passes, um submit**

> *«com 4 views o mesh não está correspondendo às views e ao tentar esculpir nas
> outras views o pincel tem drift ou offset (esculpe no lugar errado)»*

**Dois sintomas, UMA causa** — e ela é a mais clássica do wgpu.

O renderizador tem **um** buffer de uniform de câmera, e o `render_in`
escreve-o com `queue.write_buffer` antes de gravar o passe. ⚠️⚠️ **Mas
`write_buffer` não é gravado no encoder: ele é agendado na FILA**, e toda
escrita feita antes de um `submit` acontece antes de **qualquer** comando desse
submit. ⇒ quatro `render_in` num encoder só desenham as quatro vistas com a
câmera da **última**.

E é isso que produz os dois sintomas de uma vez:

| o que ele vê | porquê |
|---|---|
| «o mesh não corresponde às views» | os quatro quadrantes desenham a **mesma** câmera, espremida em quatro rectângulos de aspectos diferentes |
| «o pincel esculpe no lugar errado» | o **pick** de cada quadrante usa a câmera **dele** (que está certa) — logo o dab cai onde a peça *estaria* e não onde ela *está desenhada* |

### ⛔ E o meu gate de duas vistas não o viu

O `duas_vistas_no_mesmo_quadro_sobrevivem_uma_a_outra` desenha as duas metades
com a **MESMA** câmera (só o aspecto difere), porque o que ele mede é a
assimetria `LoadOp::Load` na cor contra `Clear` na profundidade. *Uma fixtura em
que as duas metades são iguais não pode notar que uma delas ficou com a outra.*
É a **terceira** vez nesta jornada que a fixtura não produzia o fenómeno —
depois do *fit* (esfera numa vista larga) e da tecla morta.

⇒ gate novo `duas_vistas_no_mesmo_quadro_mostram_duas_cameras`, com uma peça
**assimétrica** (esticada `2,6×` em `x`) e duas câmeras a `90°`. A régua é a
razão largura/altura da silhueta: **`1,000` contra `1,000`** antes (a mesma
imagem duas vezes), **`1,580` contra `1,000`** depois.

### A cura é uma PORTA, e não um contrato em prosa

`MeshRenderer::render_views(…)` — um encoder e um `submit` **por vista**, lá
dentro. As escritas de cada vista ficam entre dois submits, que é o que as põe
na ordem certa.

⚠️ **Um contrato a dizer *«submeta entre as chamadas»* é exactamente o tipo de
coisa que o próximo chamador não lê** — e o modo de falha dele não dá erro
nenhum: dá uma imagem plausível na vista errada. ⇒ o `Sculpt3dScene::render`
deixou de receber um `encoder`: um encoder emprestado de fora não teria como ser
submetido no meio.

⚠️ **Conferido de caminho:** o rectângulo das vistas e o alvo saem os dois de
`surface.size()`, logo estão nas **mesmas unidades** — não há a segunda causa de
*offset* (um rect em pixels lógicos contra um alvo em físicos) escondida por
trás desta.
