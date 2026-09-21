# HANDOFF — a TINTA FINA: a metade VISÍVEL (`line/sculpt3d`, 2026-09-20)

> **Ordem do dono:** *«sim. siga»* — construir a parte visível (o botão, o
> desenho na placa, e uma cena para julgar), em resposta à pergunta que fechou a
> metade da LEI.
>
> **O documento do assunto é o [`27_o_estado_da_arte_de_onde_a_tinta_mora.md`](../27_o_estado_da_arte_de_onde_a_tinta_mora.md)**
> — o §14 tem a lei, e o **§16** tem esta wave com o mecanismo de cada decisão.
> Este handoff é o que o INTEGRADOR precisa: a superfície de colisão, o que o
> diff inverte, e o que fica aberto.

---

## §1 — O que o artista consegue FAZER agora

Fileira **`Paint Detail`** no painel da escultura, colada à caixa de cor, com
quatro chips: **`Mesh`** (o caminho de sempre, **ao bit**) · `2x` · `4x` · `8x`.
Com um deles armado a tinta deixa de ter a resolução da malha — **e a malha não
muda**. Cena **`=52`**.

---

## §2 — A superfície de COLISÃO

| grandeza | delta | nota |
|---|---|---|
| `PROJECT_SCHEMA` | **0** | o plano **não** viaja no ficheiro (a cor também não; §6) |
| registos do `ph2d-ecs` e os dois espelhos | **0** | zero componentes |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` | **0** | |
| contratos congelados (§6 do roteador) | **0** | `Tool` e `NodeOp` intocados |
| ADR | **0** | |
| `scenes::CENAS` | `51` → **`52`** | ⚠️ **CONTE-O no roteador**, nunca aqui |
| crates novas | **0** nesta wave | a `ph2d-mesh-colors` nasceu na wave da LEI |
| pacotes externos | **0** | o único `+name` do `Cargo.lock` é a dep de path nova |

**Crates tocadas:** `ph2d-app-sculpt3d` · `ph2d-panel-sculpt3d` ·
`ph2d-mesh-render` · `ph2d-gpu` · `ph2d-i18n`.
⚠️ **A shell NÃO foi tocada.**

### §2.1 — Onde um merge textual pode colidir

1. **`ph2d-gpu/src/context.rs`** — uma linha: `PRIMITIVE_INDEX` entra na
   intersecção de features pedidas. *Território partilhado com toda linha que
   peça uma capacidade de device.*
2. **`ph2d-panel-sculpt3d/src/ids/sculpt3d.rs`**, **`populate.rs`** e
   **`event.rs`** — uma fileira nova em cada. ⚠️ Os três são listas a que várias
   linhas apendam; a colisão funde limpa, mas o censo `populate_censo_tests`
   reprova se o par `populate` ↔ `event` ficar desirmanado.
3. **`ph2d-i18n/src/sculpt3d.rs`** e **`app_sculpt3d.rs`** — cinco chaves e uma
   frase de recusa.
4. **`ph2d-mesh-render/src/pipeline*.rs`** — o bind group por objecto ganha
   **seis** entradas (grupo `1`, bindings `1..6`). ⚠️ Território disputado com a
   `line/Vector` (o passe de sprites) — mas aqui é o passe de MALHA.

---

## §3 — ⛔ As SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O plano vive na PEÇA, não na cena.** O knob (`Sculpt3dScene::tinta_nivel`)
   é a ESCOLHA; o plano (`SceneObject::tinta`) é o EFEITO. O endereço de uma
   amostra é `(face, sítio)` **daquela** malha — um plano partilhado leria a
   tinta de uma peça na geometria de outra.
2. **Só a peça ACTIVA ganha um plano NOVO.** As outras mantêm o que já têm.
   Armar o knob não pode multiplicar `64` amostras por vértice por toda a cena.
3. **O plano é EMPRESTADO ao traço (um `take`), não clonado.** É o que mantém a
   assinatura do `SculptStroke::dab` intocada — a porta de TODOS os corpora de
   oráculo desta casa.
4. **Durante um traço NÃO se reconcilia nada.** Ali o `Option` da peça está
   vazio; reconciliar construiria um plano BRANCO por quadro que o
   `close_stroke` sobrescreveria. *Um alocador de dezenas de MB a 60 Hz,
   invisível a toda régua de cor.*
5. **O upload lê o plano de ONDE ELE ESTÁ.** Com o traço a segurar, lê o gesto —
   ler a peça subiria `armado = 0` e *a tinta fina desapareceria no instante em
   que o artista começasse a pintar*.
6. **`devolve` reescreve o canal por vértice**, e isso não é acabamento: tudo o
   que não lê o plano (o assado, a doação, o `.ph2dproj`, o próprio caminho
   `Mesh`) lê `Mesh::colors`.
7. **A quarta recusa do `recusa.rs` NÃO é uma ausência.** As três de antes dizem
   *«falta-te uma coisa»*; esta diz *«o que vais fazer vai CUSTAR uma que tu
   tens»*. Ela **não bloqueia** — põe o preço à vista.

---

## §4 — ⚠️ As QUATRO premissas minhas que a medição derrubou

1. **O tecto, errado por 2×.** Eu escrevi *«~3,1 M amostras, 37,7 MB»* para a
   peça de fábrica: contei os interiores de cada quad e **esqueci as ARESTAS**,
   que levam `L−1` amostras cada. Medido (gate, num toro de quads):
   `1 + 2(L−1) + (L−1)² = L²` ⇒ **`64` por vértice**, `6,29 M` amostras =
   **`75,5 MB`**. *Um número escrito de cabeça ao lado de um tecto é o palpite
   que o §0.0 proíbe, e a cura não é corrigir o número — é o gate medi-lo.*
2. **A fixtura desse gate teve de trocar de forma.** Uma esfera UV tem **leques
   de TRIÂNGULOS** nos pólos, e o multiplicador só descreve a família dos quads
   ⇒ toro. *Uma fixtura que não contém o regime que a constante descreve reprova
   sobre produto correcto* — e foi o que fez, à primeira.
3. **O meu doc dizia que a discordância «recusa em voz alta».** Falso sobre o
   `garante`: ela **reconstrói**, semeada do canal por vértice. Quem fala é o
   pen-down, e as duas coisas são leis diferentes.
4. **`amostras_previstas` não tinha consumidor**, e a cura não foi inventar-lhe
   um. A constante medida é `#[cfg(test)]` com a razão escrita: *pôr um
   consumidor artificial no produto para calar o `dead_code` é escrever código
   para o linter.*

---

## §5 — ⛔ O que o CLIPPY apanhou e o `check` não

Apagar o alias morto `pipeline::MESH_WGSL` partiu **cinco** sítios em
`lighting_tests.rs` e **três** em `shade_tests.rs`, com
`cargo check -p <crate> --all-targets` **VERDE**: ele não compila os testes de
uma DEPENDÊNCIA. É a lei que o §5 do roteador já escreve, e aqui mordeu na
direcção barata (falha alto).

---

## §5-bis — ⛔⛔⛔ Os DOIS defeitos que eu introduzi, achados a RELER

Nenhum foi apanhado por um teste, e os dois têm a mesma causa de fundo: **a rota
que o produto de facto toma não tinha régua nenhuma.** A bancada de paridade tem
arnês de compute próprio e nunca chama a `upload_tinta_at`; a suíte de desenho
nunca arma um plano.

1. **`cap_idx` era DERIVADO de `cap_tri`** ⇒ na primeira subida o `origem`
   realoca, `cap_tri × 3` já descreve o tamanho novo, e o `idx` — ainda o dummy
   de `16` B — recebe milhares de bytes: **erro de validação do `wgpu`**.
   ⚠️ As capacidades iniciais diziam `4` sobre buffers de `16`, conservadoras
   **por acidente** — e era esse acidente que escondia a classe.
   ⇒ `cap_idx` é campo; gate de produto com adaptador em
   [`tinta_no_device.rs`](../../../crates/ph2d-mesh-render/tests/it/tinta_no_device.rs),
   cuja fixtura é **a sequência que estoura** (subir, e subir outra vez MAIOR).
2. **O plano não contava no peso da peça** — e o braço `RemovedObject` guarda um
   `SceneObject` INTEIRO, logo apagar uma peça punha `75 MB` na fila de desfazer
   invisíveis ao tecto. ⇒ `Tinta::footprint_bytes` na crate que POSSUI os
   vectores, somado, com CONTROLO no gate.

⚠️ **Para o integrador:** o (1) toca `ph2d-mesh-render` e o (2) toca
`ph2d-mesh-colors` — as duas fora das crates da app.

## §5-ter — ⛔⛔ O arnês da mutação mentiu CINCO vezes, com o CONTROLO dentro

`9 de 15` na 1.ª corrida, com cinco abortos *«zero testes correram»* — um deles
sobre o **CONTROLO** (uma linha em branco). Corridas à mão provaram que as cinco
**SANGRAVAM** (a `M6` isolada: `rc = 101`, `257` contados, o teste certo
vermelho). *Um aborto MUDO lê-se exactamente como uma mutação que não entrou, e
o número final é o mesmo nas duas leituras.* ⇒ o ramo de aborto passa a imprimir
as últimas linhas da corrida.

⚠️ **Quem ler um relatório de mutação com abortos re-corre pelo menos um deles à
mão antes de acreditar no placar.**

## §5-quater — ⛔⛔⛔ E a prova que o arnês passou a imprimir acusou o BINÁRIO DE TESTE, não a mutação

A 2.ª corrida, já com o ramo de aborto a falar, deu `8 de 16` com **cinco**
abortos — e as linhas que ele imprimiu dizem a mesma coisa nas cinco:

```
error: test failed, to rerun pass `-p ph2d-app-sculpt3d --lib`
Caused by:
  process didn't exit successfully: …/ph2d_app_sculpt3d-… (signal: 11, SIGSEGV)
```

⭐ **Não era mutação nenhuma: é o `SIGSEGV` que esta linha já tem NOMEADO no
`CLAUDE.md` §5** (wave do `Pull Along Normal`, 19/09 — *«morreu `5` vezes nas
primeiras `9` corridas, `0` nas `15` seguintes»*, sem atribuição possível). Ele
mata o processo do `--lib` **inteiro**, e com ele o `test result:` que o arnês
contava ⇒ *o instrumento reportava o próprio acidente*.

⭐⭐ **A cura estava escrita na mesma nota:** o `nextest` corre **um processo por
teste** e lê `229/229` em 3 de 3 na mesma árvore. O `corrida()` passa a ser
`cargo nextest run`, e a população passa a ser o `N tests run` do `Summary` —
⚠️ **nunca o `running N tests` do libtest, que CONTA os `#[ignore]`** (a 4.ª
mentira de arnês que este repo já registou).

**Medido depois da troca:** `296 tests run: 296 passed, 0 skipped`, **zero
abortos**, `57,99 s`.

## §5-quinquies — ⛔⛔⛔ E com o arnês honesto sobrou UMA sobrevivente REAL: a M13, que era uma barra ESCOLHIDA

A mutação leva as `LATITUDES` da cena `=52` de `24` para `96` — a peça passa de
**`738` para `3 042`** vértices, **quatro vezes mais fina** —, e o gate da peça
grossa **passava**. Ele pedia duas coisas, e nenhuma nomeava recurso nenhum:

| barra antiga | o que a mutação lê | veredito |
|---|---|---|
| *«pelo menos `4×` mais grossa que a irmã»* | `3 042 × 4 = 12 168 < 13 682` | ✅ passa |
| *«entre `400` e `4 000` vértices»* | `3 042` | ✅ passa |

⭐⭐⭐ **A barra nova DERIVA-SE, e os dois lados dela já existem no produto:** a
fileira tem quatro chips e a peça da `=51` é a densidade em que a marca por
vértice **já sai limpa** — a cena que o dono aprovou. ⇒ *o chip que o roteiro
manda carregar tem de ser o **PRIMEIRO** da fileira que alcança essa densidade.*

Ela aperta pelos **dois** lados sem uma constante escolhida: com a peça fina de
mais o `4x` já lá chega (a mutação), com a peça grossa de mais nem o `8x` chega
e a cena promete um detalhe que o produto não entrega.

⚠️ **A contagem sai da `Tinta` e nunca da fórmula `L²`** — os pólos de uma
esfera UV são leques de TRIÂNGULOS, e o multiplicador só descreve quads; é a
mesma armadilha que obrigou o gate do tecto a trocar a esfera por um toro.

⭐ **E o CONTROLO do gate é a própria mutação:** ele constrói a peça `4×` mais
fina e exige que ela alcance a densidade limpa **antes** do topo. *Sem isso a
régua podia estar a responder «é o último» por vácuo.*

**Placar final, com o arnês honesto: `15 de 16 sangram`** — e a `16.ª` é o
**CONTROLO** (uma linha em branco), que **não pode** sangrar. *Foram precisas
três corridas para o número querer dizer alguma coisa, e as duas primeiras
mediam o instrumento.*

## §5-sexies — ⛔⛔ O PORTÃO apanhou QUATRO vermelhos que o laço interno não vê, e os quatro são desta linha

Todos vivem em `ph2d-editor-core/tests/it/`, a família que o `CLAUDE.md` §5
nomeia: *um portão que só corre as crates EDITADAS é cego aos gates de
arquitectura.* **Nenhum foi curado por isenção** — os três de LOC por **CORTE
por responsabilidade**, e o do índice pela cura que o próprio gate prescreve.

| ficheiro | base → agora | cura |
|---|---|---|
| `ph2d-panel-sculpt3d/src/state.rs` | `600 → 615` (tecto `600`) | as **leituras derivadas** saem para `state_leituras.rs` — *o `state.rs` é o MODELO, «qual chip está aceso?» é uma LEITURA dele* |
| `ph2d-mesh-render/src/pipeline_build.rs` | `698 → 719` (tecto `700`) | a **disposição por vértice** sai para `pipeline_vertex_layout.rs` — *este ficheiro cresce quando o PIPELINE ganha um estado, aquela lista cresce quando a MALHA ganha um canal*, e foi a MALHA que cresceu |
| `ph2d-mesh/src/mesh.rs` | `692 → 723` (tecto `700`) | a **travessia de índices** sai para `mesh_indices.rs` — *o que uma malha É* contra *como ela se entrega a um desenhador* |
| `project-memory/MEMORY.md` | `21 743 → 22 120 B` (alvo `22 000`) | ver abaixo |

⚠️⚠️ **E o corte do `pipeline_build.rs` partiu um gate que lê um ficheiro por
`include_str!`** — o `toda_entrada_de_vertice_do_shader_tem_buffer_no_pipeline`
compara os `@location` do `mesh.wgsl` com os atributos declarados **no texto**
daquele ficheiro. ⭐ **Ele falhou a COMPILAR**, que é a metade barata da
família: *uma régua que procurasse a agulha por varredura teria lido zero e
ficado VERDE sobre um pipeline sem atributo nenhum.* O `include_str!` foi
repontado e o doc dele nomeia a mudança de endereço.

### §5-sexies-bis — ⛔⛔⛔ E o índice da memória tinha um ponteiro ÓRFÃO que nenhuma leitura minha explicava

O gate do orçamento (`22 120` contra `22 000`) levou-me ao índice, e ao medi-lo
apareceu outra coisa: **`1` dos `155` ponteiros não resolvia nesta árvore** — e o
ficheiro que ele nomeia **existe na árvore PRIMÁRIA**, escrito por **esta**
sessão (o `originSessionId` dele é o desta janela).

⭐⭐⭐ **A causa é estrutural e vale para toda linha em worktree:** o symlink
`~/.claude/projects/<key>/memory` aponta para o `project-memory/` da árvore
**PRIMÁRIA**, logo uma memória escrita pela ferramenta aterra **fora** da
worktree — enquanto a linha do índice, editada por caminho relativo, aterra
**dentro**. *O ponteiro e o ficheiro separam-se, e só o lado de cá é medido.*

⚠️ **Fica para o INTEGRADOR:** os outros **três** ficheiros de memória desta
sessão continuam a viver **só** na árvore primária, por rastrear
(`feedback_a_per_texel_albedo_…`, `feedback_a_picker_swatch_…`,
`feedback_forcing_a_property_…`). Eles **não** têm ponteiro nesta árvore, logo
nenhum gate daqui os vê — a outra direcção do mesmo defeito.

⭐ A cura do orçamento é a que o gate prescreve — *descer a entrada mais longa
para o `reference_topic_*` da secção dela*: a lição do `NaN` passou a viver
dentro do `reference_topic_measurement_discipline` (`160 → 162` entradas, com a
contagem do índice **re-derivada do ficheiro**, que é o gate irmão), e o índice
fica em **`21 855` bytes com `145` de folga** — ⛔ e não nos `21 995` com `5` que
a primeira tentativa deu: *uma folga de cinco bytes devolve o vermelho à
primeira linha que acrescentar um ponteiro.*

## §5-septies — O PORTÃO, com os números

| régua | resultado |
|---|---|
| `scripts/nextest-impacted.sh` | **`18 553 / 18 553`** (verde a `load 44`) |
| suíte de GPU com adaptador (`ph2d-mesh-render --test it -- --ignored`) | **`79 / 79`**, incluindo os dois `tinta_no_device` e a paridade da retícula placa↔CPU |
| `scripts/censos-da-arvore-combinada.sh` | **`127 / 127`**, com o controlo do filtro (`12 de 12` censos correram) |
| `clippy --all-targets -- -D warnings` (7 crates tocadas) | **zero** |
| prova de mutação | **`15 de 16`** — a 16.ª é o CONTROLO |

⚠️ **A placa esteve `~40 min` em fila atrás de outra linha** (`line/3DModeling`
segurava a exclusão), e a 1.ª tentativa desistiu ao fim de `300 s` como a porta
manda. ⛔ *Não se força: duas linhas na placa ao mesmo tempo foi o que pendurou o
driver em 14/09.* Quem repetir usa `PH2D_GPU_ESPERA=2400`.

## §6 — ⏳ O que fica ABERTO, com o dono de cada item

| item | dono | nota |
|---|---|---|
| **o SMOKE da `=52`** | **o dono** | é o 1.º pedido desta wave |
| a cor não viaja no `.ph2dproj` | dívida herdada da wave da pintura | o plano herda-a |
| o upload é INTEIRO e por quadro durante um traço | **por medir** | a escrita é por AMOSTRA e não passa pelo `dirty`, que é uma janela de VÉRTICES — não há upload parcial a que recorrer |
| suprimir o passe de topologia com o plano armado | **decisão do dono** | hoje ele FALA e não bloqueia (§3.7) |
| voltar a `Mesh` perde o detalhe fino | **declarado** | o roteiro da `=52` di-lo no passo (5) |

---

## §7 — Como reproduzir

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=52 cargo run -p ph2d-host-desktop --profile smoke
```
