---
titulo: "Handoff de integração — line/sculpt3d W2 (O BARRO)"
tags: [modulo/3d, tipo/handoff, assunto/integracao, status/aguardando-ordem]
status: aguardando ordem do Enio
modulo: 3D
atualizado: 2026-07-31
resumo: "A W2 fechou: o pick, a lei do traço, doze verbos, máscara, simetria, octree e upload incrementais, undo e o gesto no shell. Seis commits, zero schema, zero contrato congelado, zero foundational."
relacionados: ["[[06.1-Waves-riscos-e-alvos]]", "[[04.1-Pinceis]]", "[[03.5-Onde-roda-o-motor]]"]
---

# `line/sculpt3d` — W2, **O BARRO**

**Estado: FECHADA, 1ª rodada de smoke ABSORVIDA, pendente de RE-SMOKE e de
ordem de integração do Enio.** Branch `line/sculpt3d`, 8 commits sobre `main`
(`98eb502a2`).

---

## 1. O que entra

| # | Commit | Entrega |
|---|---|---|
| 1 | `c2d3ae2d8` | **O PICK** — raio→malha com poda front-to-back no octree |
| 2 | `b546cbe6a` | **A LEI DO TRAÇO** + os 12 verbos + máscara + simetria |
| 3 | `9d3140f2e` | **O OCTREE SEGUE A MALHA** — refit incremental por região |
| 4 | `e1e1f71f3` | **O UPLOAD É DA PEGADA** — região em vez da malha |
| 5 | `8e49f4805` | **O GESTO** — o pick vira traço, com undo, simetria e teclas |
| 6 | `e4e6a5717` | split por cap de LOC (a LEI e os VERBOS em irmãos) |
| 7 | `ce3f84219` | os docs do cofre + este handoff |
| 8 | `3fc7bbaad` | **os 3 defeitos da 1ª rodada de smoke** (§4b) |

**26 arquivos, +3700/−360.**

---

## 2. Superfície: o que NÃO foi tocado

- **`PROJECT_SCHEMA` = 46** — intocado. Uma escultura ainda não é salva (a
  persistência é wave própria; hoje ela vive só no smoke).
- **Contrato congelado** (`NodeOp=2` / `OpResolver=1` / `NodeManifest=8` /
  `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4`):
  **intacto**, conferido por grep. ⚠️ E continua sendo intacto pelo mesmo motivo
  do W1: **o gesto mora no SHELL, nunca numa `Tool`**.
- **Zero crate foundational tocada · zero ADR novo** (tudo sob o ADR-0150) ·
  **zero id / token / chave de i18n** · **zero componente ECS**.
- **Nenhuma dep externa nova.** A única mudança de `Cargo.toml` fora das crates
  do módulo é `shells/desktop`, que ganha `ph2d-sculpt3d` como dep **opcional**
  atrás da feature `sculpt3d` que já existia.
- ⚠️ **`ph2d-sculpt3d` entra em `[dev-dependencies]` da `ph2d-mesh-render`**,
  nunca em `[dependencies]`: o gate do upload incremental dirige um traço REAL
  para produzir a lista que a shell vai passar. O `src/` não a toca ⇒
  **machete-safe** (o padrão que a `line/Flip` usou com `ph2d-painter-brush`).

### API pública nova

`ph2d-mesh`: `Ray` · `Hit` · `Mesh::raycast` · `Octree::ray_visit_leaves` ·
`Octree::refit` · `RefitScratch` · `Aabb::ray_slab` / `ray_hit` ·
`RegionScratch::refreshed` / `forget`.

`ph2d-sculpt3d`: `Brush` · `Falloff` · `Verb` · `Symmetry` · `Dab` ·
`SculptStroke` · `REACH_FRACTION`.
⚠️ **`apply_dab` / `DabScratch` / `falloff()` MORRERAM** — o traço os subsumiu, e
manter os dois seria a segunda porta para *"aplicar um dab"*. Nenhum consumidor
fora da própria crate os usava.

`ph2d-mesh-render`: `Camera3d::ray_through` · `MeshRenderer::upload_region` /
`last_region_verts` · o módulo `upload`.

---

## 3. As decisões que decidem o resto

**A LEI DO TRAÇO** (`docs/3D/04.1`, e o irmão 3D do que a `line/Painter` curou
quatro vezes em 2D): pen-down congela o `pre` **preguiçosamente, por vértice
TOCADO** — um traço de 20 mil vértices numa malha de 5 M paga 20 mil, não 5 M —,
cada dab acumula um **ENVELOPE** (`max`) e guarda o **ALVO do dab que VENCEU**, e
o aplicador faz `lerp(base, target, accum)`. Três propriedades caem daí, e as
três têm gate: independência de espaçamento, idempotência sob re-stamp, e **undo
trivial** (`base` É o estado anterior, `touched` É a janela — não há um segundo
sistema a construir).

⚠️ **A pegada é consultada nas posições VIVAS.** O pincel age onde a superfície
está agora, que é o que o artista vê e o que Blender e SculptGL fazem. A
consequência honesta: mover a superfície muda quem cai sob o dab seguinte, então
os verbos de geometria **não** podem prometer independência de ordem. O
acoplamento entra pela CONSULTA, nunca pelo acumulador — e a afirmação exata
sobrevive no verbo de máscara, onde tem gate.

**A SIMETRIA é expandida na LISTA DE DABS, num ponto único.** É por isso que o
gate `every_verb_inherits_symmetry_from_the_one_place_it_is_expanded` passa sem
um gate por verbo, e é literalmente a lição do `stamp_dabs_inner` do Painter 2D.

**O `reach` é FRAÇÃO DO RAIO, nunca distância absoluta** — a lição que o impasto
do Painter pagou em 2026-07-14: com altura absoluta, um pincel pequeno e um
grande picam no mesmo valor e o grande vira uma poça chata.

---

## 4. Os dois defeitos que os gates pegaram

**(a) A máscara bloqueava a própria limpeza.** Com `w = falloff · (1 − mask)`,
uma região totalmente mascarada zerava o peso de QUALQUER dab — inclusive o que a
limparia. A máscara ficava permanente, e um botão "Clear" seria um controle morto
que *parece* funcionar em toda região parcial. A máscara gateia quem move
GEOMETRIA; quem edita o próprio canal a lê como dado, não como freio.

**(b) O conjunto que muda de NORMAL é maior que o que se MOVE.** Um vizinho
parado ao lado de uma face que girou tem a normal mudada — o `refresh_region` já
o conserta na CPU, e a doc dele já avisava. Subir só os movidos deixava a malha
iluminada por normais velhas numa faixa de **um anel de largura, bem na BORDA do
pincel**. Nasceu daí o `last_refreshed()` (superconjunto de `last_moved()`), e o
oráculo foi comparar o quadro incremental com o quadro do upload cheio, byte a
byte, num gate de GPU.

---

## 4b. Os três defeitos que o SMOKE pegou (1ª rodada)

Report do Enio: *"os movimentos de rot do mouse no canvas estão invertidos"* +
*"o local onde está esculpindo não coincide com a posição do mouse"*.

**(1) A órbita estava invertida nos DOIS eixos.** `yaw` positivo leva o OLHO para
`+X`, e a câmera indo para a direita faz o modelo *parecer* ir para a esquerda —
então manipulação direta pede `yaw -= dx`. E arrastar para BAIXO mostra o TOPO,
que é `pitch += dy`; eu passava `-dy` e **o próprio comentário ao lado afirmava o
contrário do que a linha fazia**. ⚠️ O gate que fecha isso mede o **modelo NA
TELA**, não o sinal do ângulo: argumentar sobre sinais foi como o erro entrou, e
um gate que argumentasse do mesmo jeito herdaria o erro.

**(2) Um clique sobre PAINEL era do modelo.** A cena devolvia `true`
incondicionalmente ⇒ engolia todo botão do app, inclusive os do rail, e o
dispatch 2D nunca via o evento. ⚠️ O `Move` e o `Up` **não** fazem a pergunta, de
propósito (regra de captura), e isso está gateado para ninguém "completar" a
correção e quebrar o traço longo.

**(3) O espelho nascia LIGADO.** O artista clicava de um lado e via uma segunda
protuberância do outro, sem nada na tela explicando por quê. O ZBrush nasce com
espelho e **mostra**; nós ainda não mostramos ⇒ o default honesto é desligado.

⚠️ **A hipótese óbvia — a geometria do pick estaria errada — foi REFUTADA por
medição**, e a refutação deixou dois instrumentos:

- **`the_pixels_the_ray_hits_are_the_pixels_the_mesh_painted`** — o oráculo que
  faltava. O round-trip prova raio↔**MATRIZ**; este prova raio↔**IMAGEM**, e
  entre os dois mora tudo que pode deslocar o pincel do cursor (viewport de outro
  tamanho, flip de Y, aspect divergente). Medido: **99,99 % dos pixels
  concordam**, e a discordância é a BORDA da silhueta. Câmera **assimétrica** de
  propósito — com o modelo centrado um espelho é indistinguível do certo.
- **`PH2D_SCULPT3D_DIAG=1`** — reprojeta o acerto e imprime o erro em pixels.

E nasceu **`Camera3d::project`**, o inverso EXATO do `ray_through`, com três
consumidores que precisam concordar. O gate de round-trip passou a usá-la em vez
de reimplementar a conversão NDC→pixel — uma segunda conta no teste concordaria
com o erro em vez de o expor.

## 5. Números

**O custo de um dab** (`measure_brush_kernel`, `--release`, re-medido pela porta
do produto — os da W1 eram de um `apply_dab` que não existe mais):

| triângulos | raio | vértices | dab ms |
|---|---|---|---|
| 100 k | 2 % | 449 | 0,012 |
| 1 M | 2 % | 6 364 | 0,067 |
| 5 M | **2 %** | 31 621 | **0,81** |
| 5 M | 10 % | 158 419 | 8,2 |
| 5 M | 30 % | 483 992 | 36,1 |

⚠️ **A aposta central do ADR-0150 continua valendo:** com a **pegada FIXA**, 10×
a malha custa **0,68×**. O custo é da PEGADA.

**O refit do octree**, por ablação pela porta do produto (mesma máquina, mesmos
minutos): pincel de detalhe **+0,03 ms** (dentro do ruído) · 10 % **+0,15** ·
30 % **+6,1 sobre 32,3**. Ele é **de graça onde o artista trabalha** e custa 19 %
só no regime que já estourava o K1.
⚠️ Uma primeira leitura deu **62 ms e NÃO reproduziu** (a máquina estava
carregada). *Repita antes de explicar.*

**O upload incremental:** 3385 de 10930 vértices (**0,31**) num traço de 5 dabs
com pincel de 45 % do raio do modelo.

---

## 6. Gates

| Onde | Quantos | Mutações |
|---|---|---|
| `ph2d-mesh` (lib) | 50 | 4 do pick + 1 do refit, **todas sangram** |
| `ph2d-sculpt3d` (lib) | 27 | **10, 10 sangram** |
| `ph2d-mesh-render` (lib) | 22 | 6 do plano de upload + **3 da câmera** |
| `ph2d-mesh-render` (GPU, `#[ignore]`) | **7** | rodados na RTX: **7/7** |
| `shells/desktop` (arch-gate) | 10 | **9 realistas, 9 sangram** |

⚠️ **Os gates de GPU são `#[ignore]` e precisam de adapter** —
`cargo test -p ph2d-mesh-render --release -- --ignored`. Sem adapter fazem *skip
gracioso*, **que não é verde**.

⚠️ **Três oráculos meus nasceram FRACOS e ficam registrados:**
1. o de espaçamento **saturava** em força alta e ficava verde sobre um `+=`
   clampado — o regime em que somar e envelopar divergem é o **não-saturado**;
2. o do Pinch comparava dois máximos de **vértices diferentes**;
3. nenhum alcançava *"o Smooth lê o anel congelado"* (o `continue` do envelope
   blinda), o que exigiu um **oráculo ANALÍTICO** — calcular a resposta certa e
   compará-la, em vez de inferi-la de comportamento.

⚠️ **E um gate meu SOBRE-AFIRMAVA:** *"um traço é um conjunto de dabs"* é falso
para os verbos de geometria (§3). A afirmação exata sobrevive no verbo de
máscara.

⚠️ **Um arch-gate nasceu vermelho lendo a PROSA de um doc-comment.** Um gate que
dispara em documentação ensina a não documentar ⇒ o scanner tira os comentários
antes de varrer.

⚠️ **Uma mutação sobreviveu e é INVÁLIDA, não buraco:** desligar um bloco com
`cfg(any())` é invisível a um scanner de fonte **por construção**; a regressão
realista — mover o bloco para depois do store — sangra.

---

## 7. O SMOKE

```
env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

⚠️ **`--release` não é preferência:** o kernel é aritmética por-vértice, e em
debug mede o `opt-level=0`.

A cena **imprime o que montou** e os controles. ⚠️ **Se essas linhas não
aparecerem, pare** — o resto do smoke não significa nada.

⚠️ **Diagnóstico:** `PH2D_SCULPT3D_DIAG=1` imprime, a cada dab, o pixel clicado e
o acerto reprojetado — se eles divergirem, o número diz quanto e para onde.

**Os controles:** ESQUERDO esculpe (fora do modelo, **gira**) · DIREITO gira ·
MEIO desloca · RODA aproxima · **Shift** = Smooth enquanto segurar · **Ctrl** =
inverte (cava) · **1..9,0** escolhem o verbo · **M** máscara · **`[` `]`**
tamanho · **X/Y/Z** espelho · **Ctrl+Z** desfaz.

**O que julgar:**
1. **Esculpir e ver.** Um traço deixa um domo liso, não um ouriço nem um degrau
   na borda do pincel.
2. **Devagar × rápido.** O MESMO caminho, arrastado devagar e depois rápido, tem
   de deixar o MESMO relevo. É a lei do traço, e é o que quase todo app erra.
3. **A rotação segue a mão** — arrastar para a direita vira o modelo para a
   direita; arrastar para baixo mostra o topo. (Era o defeito (1) do §4b.)
4. **A simetria nasce DESLIGADA**; `X` a liga e aí o outro lado acompanha.
5. **Clicar num painel não esculpe** — mas um traço começado no modelo continua
   se o cursor cruzar um painel.
6. **Os verbos**, um a um. O `2` (Inflate) tem de **engordar** onde o `1` (Draw)
   só levanta; o `3` (Smooth) tem de derreter um pico que o `4` (Sharpen)
   aprofunda; o `5` (Flatten) tem de deixar um platô.
7. **Shift e Ctrl** no meio de uma sessão — os dois atalhos universais.
8. **A borda do pincel.** Não pode haver costura de iluminação num anel em volta
   do que você acabou de esculpir (era o defeito (b) do §4).
9. **Ctrl+Z** devolve o traço inteiro, de uma vez.
10. **Rode uma vez SEM a env var** — é a metade do smoke que prova a inércia: o
   app 2D tem de estar byte-idêntico.

---

## 8. Aberto, com o número ao lado

- **O K1 dispara a 5 M triângulos com pincel de 30 %** (36,1 ms contra 8). ⚠️ **O
  regime é extremo:** esse pincel cobre **19 % da malha inteira**; no pincel de
  detalhe (2 %) o dab custa **0,81 ms**, 10× sob o teto. Os três caminhos
  (paralelizar a descoberta da vizinhança — que a W1 mediu em **88 %** do refresh
  · migrar o kernel para a GPU · **aceitar**, porque é exatamente onde a
  multiresolução da W4 existe para pôr o artista num nível mais baixo) seguem
  sendo **decisão de produto do Enio**. ⚠️ E o `rayon` a mais tem cerca própria: o
  ADR-0109 exige **ADR novo para todo uso novo**.
- **A escultura não é salva.** `PROJECT_SCHEMA` intocado de propósito — persistir
  malha é wave com desenho próprio (formato, tamanho, e o que acontece com o
  octree no load).
- **Sem cursor 3D na tela.** O artista aponta e o pincel age, mas não há anel
  desenhado sob o mouse. É o irmão do gizmo de pincel do Painter e mora no shell.
  ⚠️ **A 1ª rodada de smoke o promoveu de "polimento" a candidato a próximo
  item:** sem anel, *"onde isto vai cair?"* só se responde esculpindo, e foi
  parte do que fez o report (3) parecer um erro de coordenadas. A porta já
  existe (`Camera3d::project`).
- **Sem painel.** Verbo, raio, força, falloff e simetria são teclas; o painel
  docado é wave própria (e é lá que a **curva customizada** do falloff entra,
  reusando o `ParamWidget::Curve` que o repo já tem — nunca um segundo editor).
- **Simetria radial** não construída (§`04.1`).
- **`ph2d-sdf` e `ph2d-light` seguem vazias**; o matcap segue procedural.
- **A W3 (a DOAÇÃO) continua dependendo só da W1** e é o ponto de decisão do
  módulo — ela pode ser puxada a qualquer momento.

---

## 9. Para o integrador

1. `git rebase main` (a linha estava em `main` no fork; conflitos esperados:
   nenhum — o módulo é drop-crate e o único arquivo compartilhado é
   `input_dispatch/keyboard.rs`, que ganhou **um bloco** antes do `on_key`).
2. Gate da árvore combinada. ⚠️ **Rode os `#[ignore]` de GPU** (`ph2d-mesh-render`
   e os demais do repo) — o `ship.sh` não os alcança.
3. ⚠️ **Rode a suíte em DEBUG e em RELEASE.** Nada desta linha depende de perfil,
   mas a política do repo existe porque o Flip já pagou por não a seguir.
4. **Nada a renumerar:** zero ADR novo, zero schema, zero id.
