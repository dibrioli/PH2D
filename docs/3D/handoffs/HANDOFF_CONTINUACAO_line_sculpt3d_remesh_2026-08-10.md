# HANDOFF — `line/sculpt3d`, o REMESH (2026-08-10)

> **A linha NÃO está fechada.** Seis commits de pé, a wave FECHADA, e o que
> resta é o smoke do Enio. Ele **supersede** o
> [`HANDOFF_CONTINUACAO_..._2026-08-10`](HANDOFF_CONTINUACAO_line_sculpt3d_2026-08-10.md)
> como *"comece aqui"* — aquele descreve a linha limpa, antes desta jornada.
>
> ⚠️ **Isto não é o estado do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../../CLAUDE.md)**.

---

## 1. O que foi pedido, e o que a medição fez com o pedido

O Enio escolheu a wave **"O REMESH GANHA MÃOS"**: a resolução vira slider (hoje o
botão usa `DEFAULT_RESOLUTION = 150` cravado), o *achatar* vira verbo explícito,
e o campo passa a carregar a máscara.

O `CLAUDE.md` §0 manda medir antes de escrever a faixa de um slider. A sonda
`measure_remesh` existia e nunca tinha sido rodada para isto. Ela achou outra
coisa, e a wave mudou de assunto.

---

## 2. ⬛ O remesh DESTRUÍA a escultura, e reportava sucesso — FECHADO (`21f091c0f`)

Acima de ~300 o `remesh` devolvia **`Ok` com zero vértices**, e o
`sculpt3d_history.rs` instalava isso no lugar da peça:

```rust
let (out, report) = ph2d_sdf::remesh(self.mesh(), resolution).ok()?;
let previous = core::mem::replace(self.mesh_mut()?, out);   // `out` VAZIO
```

O `Ctrl+Z` recuperava (o `StrokeUndo::Remeshed` guarda a malha anterior), então
não era perda permanente — mas a peça sumia da tela com log de sucesso.

⚠️ **E não é um teto: é ERRÁTICO, e alcança o default que shipa.** Varrendo a
esfera `uv(96,144)`, **onze das cem resoluções entre 100 e 200 vazam** — `112,
151, 160, 161, 168, 180, 181, 193, 194, 196, 197` —, e o default é **150**,
vizinho de uma delas. Quem decide não é a resolução e sim o alinhamento da grade
contra os triângulos, então **outra malha vaza noutros números**: o 150 não é
seguro, é sortudo. A resolução **316 funciona** no meio de uma faixa que falha.

**A cura:** `flood_fill` devolve quantas células ficaram DENTRO — *quem cria o
interior é quem sabe dizer se há um* —, e `remesh` recusa com
`RemeshError::NoInterior` **antes** da extração.

⚠️ **E a recusa do shell parou de mentir.** Três causas (pilha de multires ·
cena vazia · o motor) entravam num `Option` só, e o chamador elegia UMA mensagem
para as três — elegeu a da pilha, então um campo vazado mandava o artista
*"reverter os níveis"* que ele não tem. `RemeshRefusal` nomeia as três, nos
**dois** chamadores (a tecla `V` e o botão do painel, que carregavam cópias
divergentes da mesma mensagem).

---

## 3. ⬛ Uma causa do vazamento, fechada — o PROXY (`27eea417a`)

O voxelizador só disparava o raio de travessia quando o ponto mais **PRÓXIMO**
do triângulo caía à frente dentro de um passo:

```rust
let along = closest[ax] - p[ax];
if along < 0.0 || along > self.step { continue; }
```

É um pré-filtro que usa o ponto mais próximo como proxy do ponto de
**INTERSEÇÃO**, que é outro ponto. Num triângulo **oblíquo** o mais próximo cai
de lado, o proxy recusa, e uma aresta que fura a superfície fica sem marca.

⚠️ **O proxy é exato SÓ em geometria alinhada aos eixos, e é essa assimetria que
o denuncia:** um **cubo nunca vazou em 361 resoluções** enquanto uma esfera
vazava em 34% delas.

**Medido, 361 resoluções × 4 malhas:**

| malha | antes | depois |
|---|---|---|
| esfera uv(96,144) | 125 | **82** |
| esfera uv(24,32) | 177 | **80** |
| cubo | 0 | 0 |
| tubo aberto | 2 | 2 |

Tirar o proxy só pode **acrescentar** marcas, nunca inventá-las: a marca
continua saindo de um `ray_hit` real com alcance de um passo.

---

## 4. ⚠️ ~~O vazamento NÃO está fechado~~ — SUPERADA pela §6b; o que já foi ELIMINADO

Sobram **82 e 80** resoluções vazando: há pelo menos um segundo mecanismo. A
recusa da §2 é o que impede que ele destrua trabalho enquanto isso.

**Quatro hipóteses derrubadas por medição — não as reconstrua:**

1. ⛔ **A caixa do triângulo termina cedo demais** (deixando células em
   `INFINITY`, que são não-guardadas). Alarguei a caixa em uma célula: varredura
   **idêntica** — 125/177/0/2 — com 20% mais tempo, o que prova que a mudança
   estava ativa. Revertida, não commitada. Confirmada uma segunda vez: **zero**
   células `INFINITY` a menos de um passo da superfície, em 600 amostradas.
2. ⛔ **O `dist` guardado mente para maior** (célula coberta pela caixa de um
   triângulo longe, não pela do perto). **Zero** ocorrências em 225 células da
   banda.
3. ⛔ **O `ray_hit` perde travessias.** Comparado contra um **Möller-Trumbore
   independente**, escrito do zero: **900 arestas, zero divergências**.
4. ⛔ **A malha de teste tem buraco.** `uv_sphere(96,144)` fecha: 0 arestas de
   beira, 0 buracos tapados, 27.360 triângulos, bounds `[-1,1]³`.

---

## 5. ⚠️ A lição mais cara: TODO oráculo degenera na superfície

> ⚠️ Ela continua válida como método; a CAUSA que ela caçava está na §6b.

Quatro instrumentos, quatro modos de errar **no mesmo lugar** — e cada um me
custou uma rodada:

| oráculo | como ele degenera |
|---|---|
| **a esfera ideal (`r = 1`)** | a malha é um poliedro **inscrito** e mergulha abaixo disso no meio de cada face, então passos inteiramente do lado de fora eram acusados de travessia |
| **força bruta com o `ray_hit`** | o mesmo algoritmo que **constrói** a parede, dos dois lados da comparação — razão entre dois doentes, cega a uma falha sistemática dele |
| **paridade em `+x` da origem** | `(1,0,0)` é exatamente um **VÉRTICE** desta esfera: o raio acerta várias faces de uma vez e a paridade sai par, dizendo *"fora"* sobre o **centro** |
| **paridade oblíqua** | sã longe da casca, **indefinida** numa célula com `dist = 0`, que está em cima dela — é onde a busca parou |

⚠️ **E as auditorias por AMOSTRA não decidem nada aqui.** As três da §4 cobriram
~0,1% da banda, e **um único furo basta para vazar**: ausência de evidência em
0,1% não é evidência de parede íntegra. Elas servem para *derrubar* uma
hipótese (um contraexemplo basta), nunca para *confirmar* a integridade.

**O que a instrumentação estabeleceu como fato:** o flood alcança **1902 de
1902** células verdadeiramente internas (paridade oblíqua) — o vazamento é
total, não marginal. E a cadeia de pais da onda até uma dessas células **nunca
fura a malha** pelo teste de segmento.

**O último candidato, plausível e NÃO provado:** um **corredor de células sobre
a superfície** (`dist ≈ 0`), cujas arestas internas não furam nada — a onda
entra nele vindo de fora sem cruzar, anda por ele, e sai para dentro sem cruzar.
É o problema clássico de **separabilidade digital**: um conjunto de arestas
marcadas só é barreira 6-conexa se a superfície for 6-separante. Provar isto
exige um oráculo de dentro/fora **não-degenerado na casca** — *generalized
winding number* é o candidato — e esse oráculo é o próximo passo, antes de
qualquer conserto.

---

## 6. O que NÃO foi medido, e por quê

O **custo** de ter tirado o pré-filtro. A máquina passou a jornada com `load
15-25` (dois `rustc` de outras linhas a 282% e 158%, mais o app aberto), e a
régua deste repo é que **nada acima de ~5 fala sobre o código**. As contagens de
vazamento acima são determinísticas e não dependem disso.

---

## 6b. ⬛ O VAZAMENTO FECHOU — a grade é RE-AMOSTRADA (`c695d7403`)

⚠️ **A §4 acima está SUPERADA e fica como história.** O que sobrou dos 82/80 era
uma classe só, e ela não é *"um segundo mecanismo"* no sentido de outro bug: é
**degenerescência numérica**. Uma amostra da grade cai EXATAMENTE sobre a
superfície, a travessia pousa na fronteira entre duas janelas de aresta
consecutivas, o arredondamento a expulsa das duas, e o interior escoa.

⚠️ **E a fixture era o que faltava, não o oráculo.** A varredura da §4 usava
esferas *lisas*; o caminho do produto é **remesh de um remesh**, cuja entrada já
nasceu de uma grade — as duas grades ficam quase-comensuráveis, e a coincidência
exata deixa de ser rara. Foi o log do Enio que a nomeou (`567828 → 40 vértices`,
62,2 M células contra os 138 M que eu varria).

**A cura é a fase.** Re-amostrar noutra FASE não muda o modelo, muda só onde a
grade pergunta — e **toda** fase não-nula curou os dois casos que reproduzem:

| fase | 0,000 | 0,100 | 0,250 | 0,382 | 0,500 | 0,618 |
|---|---|---|---|---|---|---|
| res 280 | **0,000** | 1,002 | 1,003 | 1,004 | 0,998 | 0,999 |
| res 377 | **0,000** | 1,001 | 1,002 | 1,003 | 0,999 | 0,999 |

⚠️ **A REDE ficou, e é o volume.** A malha já está FECHADA quando o campo nasce,
então o teorema da divergência dá exatamente o espaço que ela encerra: banda sã
**0,9939–1,0111** em 361 resoluções × 3 formas, vazamento em **0,000**. Com um
fosso de duas ordens de grandeza, `MIN_INTERIOR_FRACTION = 0.5` fica 2× abaixo da
pior amostra sã. `RemeshError::Leaked` nomeia a recusa; `RemeshReport.nudged` é
publicado **para a raridade poder ser reconferida** — se ele passar a aparecer
sempre, a nota que diz *"~0,55%"* precisa de outra medição.

⚠️ **`for_bounds` delega com fase 0, e `1.51 + 0.0` É `1.51`** ⇒ AO e espessura
ficam byte-idênticos. **Smoke do Enio: aprovado** (*"parou de sumir"*).

---

## 6c. ⬛ A HISTÓRIA GANHOU TETO EM BYTES (`029e475da`, `a18766943`)

Medido pela residência do processo: um remesh a 512 empilha **146 MB** de
história e o campo transiente pede **922 MB** no pico, contra os 3500 MB que o
HR-13 declara para o app inteiro. Sem teto, *fazer remesh algumas vezes* é uma
escada até o fim da memória — o `orcamento 256 MB` do log do Enio é esta wave a
morder.

O orçamento é **função do documento** (`2 × documento + 256 MB`, o molde da linha
do Audio Editor no HR-13 e da U1 do Painter), e o `footprint_bytes` é um `match`
**EXAUSTIVO**: uma entrada nova não compila até dizer quanto pesa.

---

## 6d. ⬛ O VERBO DE ACHATAR (`96133feb2`) — e um conselho INVERTIDO em cinco lugares

O item 4 do plano, metade A. Três verbos recusam com a pilha montada (remesh,
fundir, topologia dinâmica) e **as cinco mensagens mandavam *"reverta os níveis
antes"***. Medido: `Multires::reverse` faz `levels.insert(0, coarse)` — ela
insere um nível por BAIXO e deixa a pilha mais **ALTA**. Seguir o conselho tornava
a recusa mais certa.

`Multires::flatten` sobe ao TOPO e colapsa. ⚠️ A malha que fica é a de lá porque
`levels[k]` acima do selecionado está **OBSOLETO** — é a `higher` quem o
sintetiza —, então ficar com a malha que o artista VÊ jogaria fora todo detalhe
acima dele, em silêncio.

⚠️ **O gate red-first pegou a subida VAZANDO para fora:** a pilha devolvida
apontava para o topo, e o Ctrl+Z de quem achatou no nível 0 o teleportava para
cima. A `sel` é restaurada **escrevendo o campo**, não por `select` — descer
passa pela `lower`, que re-encoda o detalhe e paga o ulp do round-trip de frame.

Custo NOMEADO: **um clone da malha do topo** (desfazer e refazer precisam dela ao
mesmo tempo). Entra no teto em bytes pelo `match` exaustivo.

---

## 6e. ⬛ A MÁSCARA ATRAVESSA O REMESH (`270f77fe7`) — o item 4, metade B

O cabeçalho do `merge` já dizia a lei (*"a máscara e a cor viajam junto …
descartá-los seria destruir trabalho autorado em silêncio"*); a fusão a honra
porque os vértices dela são uma CONCATENAÇÃO. O remesh não honrava.

`ph2d_mesh::transfer_authored` leva por PROXIMIDADE — o ponto mais próximo da
superfície de entrada, com as **barycêntricas** (`TriEdges::closest_bary`, que
virou o CORPO do `closest_to`). Não é o vértice mais próximo: a malha nova pode
ser muito mais grossa, e aí o vértice dá um degrau por face.

⚠️ **O AO e a espessura NÃO viajam** — são MEDIÇÕES da geometria, e carregá-las
entrega um número que descreve uma malha que não existe mais, sem sintoma.

⚠️ **A medição mudou o código DUAS vezes.** A primeira versão custava **62-79%
do remesh** — ela TRIPLICAVA o gesto. A decomposição: a consulta do octree é
**4%**, e o resto eram as **75 faces por consulta** (ele responde pelas FOLHAS
tocadas), cada uma com o `TriEdges` reconstruído do zero — o oposto do que o doc
daquele tipo prescreve.

| | µs/vértice |
|---|---|
| como nasceu | 2,82 |
| triângulos PREPARADOS uma vez | 1,50 |
| + rejeito por esfera envolvente | **0,49** |

A 512: **3,43 s (62%) → 0,46 s (18%)**. ⚠️ E aumentar a semente do raio **PIORA**
(893 → 1272 → 2200 ms a 256) — a busca não crescia o raio, testava faces demais.

⚠️ **A mutação que aperta o rejeito sobreviveu a TRÊS fixtures**, inclusive uma
com o destino flutuando longe da fonte: numa malha fina o centroide e o ponto
mais próximo quase coincidem e a folga nunca é exercitada. O gate que a mata usa
**força bruta** como oráculo e uma fonte **GROSSA**.

---

## 7. O estado da linha

| | |
|---|---|
| Branch | `line/sculpt3d`, **6 commits** sobre `main 76788440a` |
| Árvore | **limpa** |
| Suítes | `ph2d-sdf` + `ph2d-mesh` + `ph2d-sculpt3d` verdes · shell verde · clippy limpo · LOC verde |
| Schema | `PROJECT_SCHEMA` **intocado** · registro do `ph2d-ecs` intocado · contrato congelado intocado |
| `Cargo.toml` | **zero** — nenhuma crate nova, nenhuma dep nova |

**Superfície pública nova:** `ph2d_sdf::RemeshError` (+ `Leaked`) ·
`RemeshReport.nudged` · `VoxelField::for_bounds_phased` · `flood_fill` devolve
`usize` · `ph2d_mesh::signed_volume` · `Multires::flatten` ·
`Multires::footprint_bytes` · `Mesh::footprint_bytes` · `Mesh::put_colors` ·
`transfer_authored` · `TriEdges::closest_bary`.

**Sondas novas:** `probe_leak.rs` (quantas resoluções vazam, por malha) ·
`probe_repeat_remesh.rs` (o que repetir um remesh custa em RESIDÊNCIA) ·
`measure_transfer.rs` + `measure_transfer_probe.rs` (de que a travessia é feita).

---

## 8. O que fazer a seguir

⚠️ **Os quatro itens da lista antiga estão FECHADOS** — o oráculo
não-degenerado deixou de ser o gargalo quando a causa virou degenerescência de
FASE (§6b), o "segundo mecanismo" era ela, o slider landou (`c43318b74`) e o item
4 landou nas duas metades (§6d, §6e).

**O que resta é o SMOKE**, e ele é a cena `=26`:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d
env PH2D_SCULPT3D_SMOKE=26 cargo run -p ph2d-host-desktop --release
```

Ela conta a história inteira numa peça só: mascarar → reconstruir (a máscara
sobrevive) → subdividir → o remesh RECUSA e manda **ACHATAR** → achatar (o
detalhe do topo sobrevive) → reconstruir → Ctrl+Z devolve a pilha.

**Aberto, com o preço ao lado:**

- ⚠️ **O gate de regressão da cena `=6` não contém o fenômeno do vazamento** — 8
  rodadas encadeadas não vazam, e a taxa medida (~0,55%) diz que isso é amostra
  pequena, não ausência. Quem prova a cura são os dois casos do tubo aberto.
- A travessia é **serial**. Ela é um gather por-vértice (leitura pura, saídas
  disjuntas) — a forma exata que o **ADR-0156** sancionou para o traço de AO
  nesta mesma crate-família. `rayon` aqui é decisão do Enio (ADR-0109 §cerca).
- O campo **ainda não carrega cor/material**: quem os leva é o
  `transfer_authored`, o que é outra coisa e é o certo (o campo é uma grade de
  62 M células; um plano por canal seriam +250 MB de rascunho por canal).

---

## 9. Armadilhas de processo desta jornada

⚠️ **A cwd do Bash escorregou para a árvore PRIMÁRIA duas vezes** — uma me fez
rodar um gate contra o `main` (o *"0 tests"* não era resultado, era a árvore
errada) e outra escreveu uma sonda lá. **`main` foi conferida e está limpa nas
duas.** A regra é a do `MODELO_TROCA_DE_AGENTE_NA_LINHA`: **todo comando começa
com o `cd` da worktree**, sem exceção.

⚠️ **Um gate meu nasceu incapaz de falhar.** Ele afirmava que cada causa de
recusa *aparecia* no bloco de despacho; a mutação que colapsa duas num braço só
(`MultiresStack | EmptyScene`) deixa o nome no texto e **passou**. Agora ele
**conta braços**. O gate anterior, do `main`, tinha o mesmo vício por outra via:
ancorava no literal `return None` — uma **grafia**, não a propriedade.

⚠️ **E uma afirmação minha foi DERRUBADA por medição, depois de eu a ter
escrito neste handoff.** A §4 dizia que o vazamento parcial estava *"descartado
por medição"* — varri 66 resoluções sobre esferas lisas, não achei colapso e
declarei a hipótese morta. O log do Enio provou que era real. **A fixture era o
problema, não o oráculo:** o caminho do produto é *remesh de um remesh*, e uma
malha que já saiu de uma grade cai quase-comensurável com a seguinte. Uma
varredura sobre a forma errada é ausência de evidência, e eu a reportei como
evidência de ausência.

⚠️ **A cwd escorregou para a árvore PRIMÁRIA mais três vezes** (um comando em
background reseta a cwd rastreada). Duas escritas chegaram ao `main` — uma sonda
apensada e um arquivo criado —, as duas revertidas, e o `main` conferido limpo.
