# `line/UIUX` · 2026-09-20 (2.ª volta) · **O SELECTOR DE PINCÉIS SAI DO PAINEL, E UM MODAL É DONO DA ENTRADA**

> ⚠️ Este documento descreve o mundo **no dia em que foi escrito**. O estado vivo é o `CLAUDE.md` §5.
>
> ⛔ Ele **não** é o handoff de [`…_2026-09-20.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-09-20.md) (a
> fronteira dos motores / as elisões / o balão), que já foi integrado. Esta é a **reabertura** da
> linha por ordem do dono depois daquela integração: `7` commits sobre o merge-base `395da6a55`.

---

## §1 — O que esta jornada entrega

| # | commit | assunto |
|---|---|---|
| 1 | `d5109f15b` | **o censo do degrau `G`** — de quem são as entradas de cada painel (comandos · valores · cromo · órfãos · altura contra a dobra) |
| 2 | `2d30f4b97` | sonda: onde cai cada secção do painel da escultura |
| 3 | `a7109b827` | sonda: o que come a secção `Tool`, e a tabela separa **vazio** de **armado** |
| 4 | `d2dc8d0eb` | a **paleta de pincéis** (`brush_palette`) |
| 5 | `517f2ff86` | o selector sai do painel — a secção `Tool` encolhe `276 px` |
| 6 | `3c1d0e632` | os reports 1, 2 e 4 do dono — o teclado do modal e o botão invisível |
| 7 | `05ac23b52` | o report 3 — **um modal de ecrã inteiro é dono do PONTEIRO, não só do teclado** |
| — | (este) | a régua que diz, **pelo NOME**, o que fica abaixo da dobra |

**Ordem do dono que a abriu** (2026-09-20, sobre a medição do degrau `G`): *«sim»* — tirar a secção
`Tool` inteira do painel para uma paleta.

---

## §2 — Superfície de colisão (colada do script, não escrita à mão)

```text
SUPERFÍCIE DE COLISÃO — line/UIUX contra main
  merge-base 395da6a55   ·   7 commit(s)   ·   28 arquivo(s)
    PROJECT_SCHEMA 160 (base: 160)   ·   tripla (160, 13, 22) (base: idem)
    VEC_SCENE_SCHEMA 22 · FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23  — todos na base
    ph2d-ecs 103 · ph2d-render 104 · ph2d-script 104  — todos na base
    contrato congelado (§6): INTOCADO
    ADR: esta linha não cria nenhum ⇒ fora de toda disputa de número
    Cargo.lock: nenhum '+name' novo
    tectos de LOC: nenhum ficheiro da linha passa
```

⇒ **zero contador partilhado, zero contrato, zero ADR, zero pacote externo.** O atrito possível é
textual, e está no §5.

---

## §3 — Os quatro reports do dono, e o que cada cura foi

O dono mandou **quatro** coisas numa mensagem só, com duas fotos. Três eram da paleta acabada de
nascer; a quarta é do app inteiro.

| # | o report | a causa MEDIDA | onde |
|---|---|---|---|
| 1 | *«o modal não captura o que escrevo»* | o teclado ia à cena 3D antes do modal | `3c1d0e632` |
| 2 | *«o painel lateral captura os atalhos»* | **o mesmo defeito**, visto do outro lado | `3c1d0e632` |
| 3 | *«ao selecionar o pincel, o modal deveria fechar»* | ⛔ **NÃO era a paleta: era a ORDEM do despacho** | `05ac23b52` |
| 4 | *«não se pode saber que é um botão, só aparece o nome»* | o `command` pintava rótulo sem fundo | `3c1d0e632` |

### ⛔⛔ O report 3 foi diagnosticado por DEDUÇÃO e voltou

A cura do teclado fechou os reports 1 e 2, e eu **deduzi** que o 3 tinha a mesma causa. O dono
respondeu *«modal ainda não fecha ao selecionar o pincel»*. Medido, a rota do quadro é

```text
pre_dispatch → os PAINÉIS (Consumed ⇒ return) → showcase → chrome::dispatch_all
```

e um item da paleta carrega o **MESMO `NodeId`** da ficha do painel (é isso que faz o *pick*
resolver pela lei que já existia) ⇒ o painel da escultura reconhecia-o, **consumia**, trocava o
pincel, e o `chrome::dispatch_all` — que é quem fecha a paleta — nunca corria.

⇒ o ponteiro de um modal de ecrã inteiro foi **hoistado** para o `pre_dispatch`, que é onde as
outras três famílias da mesma forma já vivem (a barra de menus, as abas de painel, as abas de
layout). Mecanismo e a quarta linha da tabela: [`medicoes/26`](../medicoes/26_tres_reports_do_modal_e_uma_causa_so.md).

⚠️ **E o meu gate estava ABAIXO da rotura:** a 1.ª redacção chamava o `chrome::dispatch_all`
directamente, com um comentário a gabar-se de *«pela porta REAL do chrome»*. A 2.ª entrou pelo
`apply_event` e **continuava verde**, porque a fixtura não ARMAVA o painel da escultura — sem
retrato publicado ele ignora o evento e a rota nunca chega a quem o consome. *Uma fixtura que não
arma o sujeito mede o caminho onde o defeito não existe.*

---

## §4 — A medição, e de QUAL instrumento cada número é

⛔⛔ **Há DOIS instrumentos e eles não se somam.** O desta linha pinta num arnês
(`quantas_entradas_tem_cada_painel`, viewport de `4000 px`); o da `line/sculpt3d` pinta na costura
daquele painel com o `Draw` na mão (`diag_onde_cai_a_pista_do_pente`). Eles concordam na **ordem**
das secções e em **o que cruza a dobra**, e discordam nos píxeis porque armam estados diferentes.
*Misturar dois instrumentos numa conta é a forma exacta de fabricar uma medição.*

### 4.1 — O arnês desta linha (dobra `880`)

| estado | secção `Tool` | painel inteiro |
|---|---:|---:|
| **pior caso** (`Pro`, filtro armado) — antes | `614 px` | `2 373 px` |
| **pior caso** — depois | `338 px` | `2 097 px` |
| **dia a dia** (`Basic`, filtro desarmado) — depois | `173 px` | `1 882 px` |

⚠️ **As DUAS leituras existem porque a armação de um painel é o estado MÁXIMO dele e não o do dia a
dia:** as catorze fichas do filtro valem `~221 px` e só são pintadas depois de o artista armar o
filtro. *Uma coluna que colapsa dois estados num número descreve um app que ninguém usa.*

### 4.2 — O instrumento da `line/sculpt3d` (o mesmo, antes e depois)

| o que | antes | depois | |
|---|---:|---:|---|
| secção `Tool` | `108` | `108` | visível |
| secção `Brush` | `534` | **`281`** | visível |
| `Radius` | `611` | **`358`** | visível |
| `Connected Only` | `922` ⛔ | **`669`** | ✅ passou a visível |
| secção `Symmetry` | `1 132` ⛔ | **`879`** | ✅ passou a visível (⚠️ §6) |
| **a dobra** | `880` | `880` | |
| secção `Topology` | `1 191` ⛔ | `938` ⛔ | |
| `Edge Flow` | `1 271` ⛔ | `1 018` ⛔ | |
| secção `Shading` | `1 557` ⛔ | `1 326` ⛔ | |

⇒ **`−253 px` naquele instrumento**, e **duas** fileiras atravessaram para o lado visível.

### 4.3 — A catraca dos cinco pincéis foi a ZERO, e não fui eu que a desci

O `os_controlos_proprios_de_um_pincel_cabem_no_encaixe` (costura do painel da escultura) nasceu com
**cinco** pincéis cujos controlos próprios caíam abaixo da dobra. A **metade da obsolescência** dele
acusou-os na primeira corrida a seguir à cirurgia:

```text
Cloth: ele passou a caber no encaixe (y = 854 <= 880) — APAGUE a linha dele
da catraca, senao ela vira licenca
```

A lista está **vazia**, e com ela o `else` daquele gate exige que **todos os sete** caibam.
*Uma catraca sem censo de obsolescência não desce: ela vira licença — esta desceu porque o tinha.*

---

## §5 — ⛔⛔⛔ O QUE ESTA LINHA TORNA OBSOLETO NOUTRA LINHA

A tabela do §4.2 está escrita, com os números **de antes**, em **três** sítios que **não são
desta linha** — e ⚠️ **só DOIS deles são para corrigir:**

| # | sítio | vivo? |
|---|---|---|
| 1 | `CLAUDE.md` §5 (3D/Sculpt, o bloco do `Edge Flow` de 19/09) — *«`Tool 108` · `Brush 534` · `Radius 611` · **dobra `880`** · `Connected Only 922` · `Symmetry 1132` · `Topology 1191` · **`Edge Flow 1271`** · `Shading 1557`»* | ✅ **vivo** (é o roteador) |
| 2 | `crates/ph2d-app-sculpt3d/src/scenes_pente.rs` — a mesma tabela no doc-comment do `announce()`, e o roteiro que o DONO lê | ✅ **vivo** (é código e é smoke) |
| 3 | `docs/3D/handoffs/…_sculpt3d_2026-09-17.md` §94 (linhas `4211`–`4229`) | ⛔ **registo MORTO — fica como está** |

⛔ **O nº 3 NÃO se corrige, e a razão é a lei dos handoffs desta casa:** *«cada handoff descreve o
mundo no dia em que foi escrito e não é actualizado depois»* (está no cabeçalho do índice deles).
Reescrevê-lo apagaria a medição que justificou a decisão do dono naquele dia. *Um registo morto que
alguém «actualiza» deixa de ser prova de coisa nenhuma.*

⚠️ **Nenhum deles foi editado por esta linha** — e ⛔⛔ **a razão que eu escrevi primeiro estava
ERRADA, derrubada por medição no mesmo dia.** Eu escrevi *«a `line/sculpt3d` fechou com 102 commits
e 594 ficheiros e o `scenes_pente.rs` é dela»*. Medido (`git merge-base` + `diff --name-only` nas
duas árvores):

| facto | medido |
|---|---|
| merge-base da `line/sculpt3d` | **`395da6a55`** — o MESMO que o desta linha, ou seja **o `main` de agora** |
| ⇒ aquele fecho de `102` commits | **já ATERROU**; o `scenes_pente.rs` vive no `main` |
| a `line/sculpt3d` hoje | **REABERTA**, `36` ficheiros (as manchas pretas + a caixa de cor) |
| `scenes_pente.rs` está nesses 36? | **NÃO** |
| ficheiros em comum com esta linha | **ZERO** |

⇒ *a minha razão era de calendário e o calendário já tinha virado.* O que fica de pé é a outra
metade, que é de **produto** e está no §7.2.

⇒ **acto do INTEGRADOR, depois de as duas linhas aterrarem:** re-correr
`cargo test -p ph2d-panel-sculpt3d --test it diag_onde_cai_a_pista_do_pente -- --ignored --nocapture`
e reescrever **as duas tabelas vivas** com o que ele imprimir.

✅ **O gate daquela linha continua VERDE nesta árvore** (`o_roteiro_da_49_diz_onde_o_edge_flow_esta`,
corrido aqui): ele é uma **EQUIVALÊNCIA** entre *«o `Edge Flow` cai abaixo da dobra»* e *«o roteiro
manda rolar a roda»*, e o `Edge Flow` continua abaixo (`1 018 > 880`). ⛔ **Isto é uma margem de
`138 px`, não uma garantia** — ver o §7.

---

## §6 — ⚠️ A FACA: a secção `Symmetry` cai em `879` contra uma dobra de `880`

Um pixel. Qualquer fileira acrescentada ao `Tool` ou ao `Brush` — por **qualquer** linha — empurra o
`Symmetry` de volta para fora do ecrã, e nada reprova.

⛔ **E isto NÃO virou catraca, de propósito.** Um ratchet a `1 px` de margem reprovaria em toda
adição e a cura de cada uma seria uma decisão de produto ⇒ ele seria ruído, não disciplina. O que
existe e chega é o gate irmão (`os_controlos_proprios_de_um_pincel_cabem_no_encaixe`, §4.3), que
mede a promessa que alguém de facto fez: *os controlos próprios de um pincel cabem*.

*Fica declarado aqui para que a próxima linha que acrescente uma fileira àquele painel saiba o que
gasta.*

---

## §7 — O que fica ABERTO, e porque ESPERA

### 7.1 — ⛔⛔ **DECISÃO DO DONO (2026-09-20): FICA COMO ESTÁ** — e as três saídas foram medidas

Posta a escolha com os números, ele respondeu **«fica como está»**. ⇒ a disposição daquele painel
**não muda**, e o que está abaixo fica como o registo do que foi medido e recusado — *não como
trabalho por fazer*.

| saída | ganho | preço |
|---|---:|---|
| só o `Transform` sai para a fila | `25 px` — a secção `Brush` passa a caber **inteira** | zero: é o precedente do `3D Model` que ele aprovou em 01/09, e é o único dos seis cuja **face** lê um estado |
| **as seis saem** | `148 px` — `Symmetry` inteiro + `Dynamic Topology` e `Detail` | cinco chips com face constante (⛔ a lei do `AreaMenu`: *«um chip que diz sempre a mesma coisa custa a mesma largura e não informa nada»*) + reescrever um passo do roteiro da `=49` |
| ✅ **fica como está** | — | `5` das `7` secções fora do ecrã; a última fileira do `Brush` cortada por `8 px` |

⚠️ **O mecanismo de destino EXISTE e tem um inquilino só** ([`interaction::area_menu`], hoje o
`ph2d-panel-model3d/src/area_bar.rs`), e os ids seriam **os mesmos** — *um comando com dois ids tem
dois sítios a apodrecer em separado*. ⛔ Não é falta de substrato que trava isto; é a decisão acima.

<details>
<summary>A medição que produziu a escolha (fica para quem a reabrir)</summary>

### O bloco da máscara são `148 px`

A régua nova (`o_que_o_artista_nao_alcanca`) diz, fileira a fileira, que no **dia a dia** a única
coisa da secção `Brush` que cruza a dobra é esta:

```text
  740..762     sculpt3d.mask_op.{0..3}        as quatro operações de máscara
  768..790     sculpt3d.extract
  796..818     sculpt3d.extract_thick(+num)
  821..843     sculpt3d.extract_smooth(+num)
  866..888  ⛔ sculpt3d.transform.{0,1,2}
  ────── a dobra: 880 ──────
```

e que os **knobs que o artista gira enquanto esculpe** estão todos acima, com folga:
`Radius 358` · `Strength 383` · a curva `428..519` · o padrão `545..636` · `Accumulate 664` ·
`Surface Only 692`. ⇒ **o objectivo da wave está cumprido**; o que cruza a dobra são **seis
COMANDOS**, que é exactamente o que a `D2` manda triar.

⛔⛔ **E ela NÃO foi feita nesta jornada, por DUAS razões medidas:**

1. ⚠️ **Preço cross-line — REAL, e mais barato do que eu disse:** `1 018 − 148 = 870 < 880` ⇒ o
   `Edge Flow` sobe para cima da dobra e o `o_roteiro_da_49_diz_onde_o_edge_flow_esta` **reprova**.
   ⭐ E isso **não é um acidente: aquele gate foi escrito a antecipar este dia** — ele é uma
   EQUIVALÊNCIA, e o doc dele diz por extenso *«no dia em que a arrumação que o dono anunciou
   trouxer esta fileira para cima da dobra, ela reprova — e a cura é apagar a frase da rolagem do
   roteiro, nunca afrouxar o número»*. ⇒ a cura está **prescrita pelo instrumento da outra linha**,
   e o ficheiro (`scenes_pente.rs`) está no `main` e **não** entre os `36` da reabertura dela (§5).
   *Isto deixou de ser um bloqueio e passou a ser um item de trabalho com preço conhecido.*
2. ⛔ **O que BLOQUEIA é decisão de PRODUTO, e há uma cerca com razão escrita.** O `mask_tools.rs`
   declara: *«as seis moram na secção do PINCEL, não numa própria: um artista que acabou de pintar
   máscara procura o que fazer com ela onde ele a pintou»*. ⚠️ Essa frase argumenta contra **uma
   secção própria três rolagens abaixo** — e o destino da `D2` não é isso, é o **chip-pulldown da
   fila**, que é um clique e está sempre à vista. *A cerca não responde à pergunta que a `D2` faz*,
   logo ela não decide — mas **onde um controlo mora é produto**, e quem decide é o dono. Ele tem a
   pergunta com os dois números (`148 px`; `Symmetry` + meia `Topology` a caberem).

</details>

### 7.2 — O resto do painel

Abaixo do `Symmetry` ficam `980 px` em cinco secções (`Topology` · `Shading` · `Scene` · `Bake`),
e o censo do degrau `G` mede este painel em **`102` comandos contra `28` valores**. A triagem da
`D2` dele é wave própria.

### 7.3 — Os outros painéis, pela régua

O censo (corrido no âmbito do app, `28` painéis) ordena a dívida de comandos assim:

```text
  painel        comandos  valores  cromo  órfãos  total   altura
  inspector          314      313     13      67    707   14 987
  tokens             110       21      0      86    217    2 866
  sculpt3d           102       28      0       0    130    2 097
  physics             60       24      0       0     84    1 293
  vector              45       18      0       9     72    1 349
```

⚠️ **O `inspector` é grande POR DIREITO** (ele é um painel de propriedades) e a altura dele é a do
estado com as `28` secções armadas, que nenhum objecto real tem. *Um censo que só medisse TAMANHO
poria-o no topo e mandaria a wave para o sítio errado* — é para isso que a coluna dos **comandos**
existe.

⚠️ E o `inspector` é escrito em grande parte pela `line/components`, que está **viva** — a mesma
pergunta de sobreposição do §5 tem de ser refeita **no dia** em que essa wave abrir, e não herdada
daqui: *uma medição de superfície de colisão vale para o dia em que foi tirada.*

---

## §8 — As premissas MINHAS que a medição derrubou

1. ⛔ **«faltam `22 px` para a secção `Brush` caber»** — eram a distância até ao **fim PADDED** da
   secção (`902`); o conteúdo acaba em `888` e o que fica cortado é **UMA fileira de chips**, por
   **`8 px`**. *Uma régua que mede até ao fim de uma secção mede a MARGEM dela e chama-lhe controlo.*
2. ⛔ **«a `Filter` é o próximo corte»** — medida, a secção `Tool` no dia a dia mede `173 px` e a
   `Filter` é uma fileira de `22`: cortá-la não move a dobra de nada. O que a cruza está `148 px`
   abaixo dela, noutra secção.
3. ⛔ **o report 3 tinha a causa do report 1** (dedução) — a causa real era a ordem do despacho (§3).
4. ⛔⛔ **«o `scenes_pente.rs` é de uma linha que acabou de fechar»** — ele está no `main`, e a
   `line/sculpt3d` está REABERTA sem ele no diff (§5). *Uma afirmação sobre o estado de outra linha
   mede-se com `git merge-base`, não com o que o roteador diz que aconteceu ontem.*
5. ⛔⛔ **Corri o ÂMBITO POBRE e o gate recusou-o com a cura na mensagem.**
   `cargo test -p ph2d-panel-registry-init` reprova **12** testes porque `flip`, `flip_frames`,
   `painter_layers` e `wet_tuning` chegam pelo `shells/desktop` e a `default` desta crate não os
   liga — a corrida certa é
   `--features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`. *O instrumento
   funcionou: fui eu que corri a corrida que ele existe para proibir* (a lei é do fecho de 20/09,
   e está escrita no `CLAUDE.md` §5).

---

## §9 — A linha do `CLAUDE.md` §5 (o INTEGRADOR escreve; a linha só entrega o texto)

> ⚠️ **Uma linha, e ela é paga por todo agente em toda janela** (o aviso do próprio §5). O
> mecanismo fica aqui; ali vai só o que muda uma decisão.

> ⭐⭐⭐ **E O SELECTOR DE PINCÉIS SAIU DO PAINEL** (20/09, 2.ª volta, ordem do dono: *«sim»* —
> [handoff](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20_A_PALETA.md)):
> as `38` fichas de verbo viraram **um botão** cuja face é o pincel na mão, com o catálogo numa
> **paleta** modal com busca — a secção `Tool` encolhe `276 px` e o `Connected Only` e a secção
> `Symmetry` atravessam para o lado visível da dobra (⚠️ o `Symmetry` fica a **`879` contra `880`**:
> um pixel, declarado e **não** transformado em catraca, porque a `1 px` ela seria ruído). ⭐ A
> catraca dos cinco pincéis foi a **ZERO**, e quem a desceu foi a **metade da obsolescência dela**.
> ⛔⛔⛔ **E dos quatro reports do dono, o terceiro NÃO era a paleta — era a ORDEM do despacho:** um
> item dela carrega o MESMO `NodeId` da ficha do painel, logo o painel consumia o clique e o
> `chrome::dispatch_all` — quem fecha o modal — nunca corria ⇒ o **ponteiro** de um modal de ecrã
> inteiro é hoistado para o `pre_dispatch`, a **quarta** família da mesma forma (as outras três já
> lá estavam). ⚠️⚠️ O meu gate ficou VERDE **duas vezes** sobre o defeito vivo: a 1.ª redacção
> entrava pelo chrome, **abaixo da rotura**, e a 2.ª não ARMAVA o painel da escultura — *uma fixtura
> que não arma o sujeito mede o caminho onde o defeito não existe.* ⭐⭐ **E o degrau `G` ganhou as
> duas réguas que lhe faltavam** (o censo das entradas por painel — `comandos · valores · cromo ·
> órfãos · altura contra a dobra`, `28` painéis — e a que diz **pelo NOME** o que fica abaixo dela),
> e a segunda corrigiu a minha nota na 1.ª corrida: *«faltam 22 px»* era a **MARGEM** da secção, e o
> que corta é **uma fileira**, por `8 px`. ⏳ **ABERTO:** o bloco das seis operações de máscara
> (`148 px`, seis COMANDOS pela `D2`) é a próxima fatia e **espera a integração** — ele baixa o
> `Edge Flow` para `870 < 880` e faz reprovar o `o_roteiro_da_49_diz_onde_o_edge_flow_esta` da
> `line/sculpt3d` (⭐ aquele gate foi escrito a antecipar este dia e prescreve a cura) — ⛔⛔ **e a
> DECISÃO DO DONO, com as três saídas e o preço de cada uma na mesa, foi «FICA COMO ESTÁ»**, logo
> aquele painel não muda e o §7.1 do handoff é o registo de uma recusa e não de trabalho por fazer.
> ⚠️ E a tabela do `Edge
> Flow` que este §5 publica **ficou obsoleta por esta linha** (`−253 px` em todas as fileiras): o §5
> do handoff diz os **dois** sítios vivos e o comando que os reescreve.

> ⭐⭐⭐ **E A LARGURA DE FÁBRICA DE UMA COLUNA PASSOU A SER UMA FRACÇÃO DA JANELA** (20/09, 2.ª
> volta — [handoff §9-bis..§9-quater](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20_A_PALETA.md)):
> `default_dock_w = (token × min(1, janela/1366)).max(PANEL_MIN_W_PX)` — um **TECTO em fracção**,
> **inerte** acima da referência (a coluna não cresce num ecrã grande) e com o piso do painel em
> baixo. Antes, as duas laterais eram `612 px` FIXOS: `45 %` de um iPad 12,9" e **`54 %` de um iPad
> mini**. ⛔⛔⛔ **E o smoke foi REPROVADO TRÊS vezes com a lei CERTA, com as causas a serem outras
> três coisas** (§9-ter): **(1)** o `~/.ph2d/layout.txt` do dono tinha uma largura gravada nas seis
> áreas de trabalho e o `install_saved` escreve-a **antes do 1.º quadro** — *uma lei de valor de
> FÁBRICA é invisível a quem já tem uma escolha gravada*, e o perfil de smoke passa a ser
> `HOME=/tmp/...`; **(2)** o meu roteiro mandava ESTREITAR e a coluna só se move entre `976` e
> `1 366 px`, com a janela a abrir em `1 024` — *a direcção onde não há nada para ver*; **(3)** um
> defeito de PRODUTO a sério — **um arrasto que aterra na largura de fábrica GRAVAVA-A como
> escolha** e a coluna saía da lei para sempre, e ele mordia com a janela apertada porque a lei
> comparava a largura CRUA enquanto o store **CORTA** no piso (`|80 − 220| = 140` ⇒ grava `220`, que
> é o número que a própria lei dava). ⇒ `ChromeBands::escolha_de_um_arrasto` (a lei **corta com o
> mesmo token** antes de comparar) e `set_dock_width(side, Option<f32>)` — ⚠️ a forma `Option` é o
> que a mantém a **zero referências novas** na catraca do DAG `interaction → screens`, que a 1.ª
> redacção levava de `18` a `20`. ⛔ **O gate de costura irmão era CEGO por PREFIXO:** o
> `the_border_gesture_reaches_the_panel` procura `set_dock_width`, que é **prefixo** do nome da
> porta nova ⇒ ficava verde com a regressão inteira dentro. ⭐⭐ **E o gate que faltava mede o
> PIXEL** (`a_coluna_pintada_encolhe_com_a_janela`, quatro quadros pela rota real em sete larguras):
> a lei tinha três gates — a lei, a porta do store e o **TEXTO** do quadro — e *nenhum percorria a
> rota até ao rectângulo que a coluna OCUPA*, que é a única coisa que o dono vê. ✅ **Smoke do dono
> APROVADO** (`PH2D_DOCK_LOG=1`, ~70 redimensionamentos de `1 920` a `647`): `escolha -` dos dois
> lados em todas as linhas, e a curva dele mede os **dois pisos separadamente** (`988` à direita,
> `976` à esquerda — cada um do seu token), que nenhuma fixtura desta linha fazia. ⏳ **DECISÃO DO
> DONO:** a largura arrastada é guardada em **píxeis absolutos**, logo num tablet que roda os
> `371,72 px` dele são `32,8 %` deitado e `50,0 %` em pé; curá-lo ou exige um tecto que ninguém
> mediu, ou muda o que «arrastar a borda» significa **e** o formato do `layout.txt`.
> ⛔⛔⛔ **E o 4.º report fechou a metade MANUAL, que estava morta: numa janela estreita arrastar
> a borda não fazia NADA** (*«pare de tentar. permita que manualmente o usuário consiga estreitar
> o painel»*) — ali a lei já entrega o mínimo e o piso da ESCRITA era o mesmo número, logo o gesto
> pedia `157` e o store devolvia `220`. ⭐ **Passam a ser DOIS pisos:** o de FÁBRICA fica em
> `PANEL_MIN_W_PX` (ninguém pediu para o app *nascer* ilegível) e o de uma ESCOLHA desce para
> **`210`**. ⭐⭐ **O `210` é DECISÃO e o `84` é o RECURSO**, e a separação é o que torna o número
> honesto: `84` é a largura MEDIDA em que o corpo de um painel ainda cabe (a `83` sai o primeiro
> controlo, e descer de `220` para lá acrescenta `1,0 px` de transbordo no pior dos **19** painéis
> docáveis), e o que shipa é **`2,5 ×`** isso, por DUAS ordens do dono no mesmo dia, cada uma
> depois de ver o número anterior no ecrã (*«no mínimo o dobro»* ⇒ `168`; *«ainda muito estreito.
> aumente 25%»* ⇒ `210`). ⭐ As duas cercas são **erro de compilação** e apertam a faixa legal nos
> dois lados (`210 ≤ piso < 220`), o que reduz a `10 px` a janela em que um piso solto pode
> mentir. ⚠️⚠️ **E esses `10 px` são também o que o gesto compra numa janela ESTREITA**, porque ali
> a lei de fábrica já entrega `220`: o curso a sério está nas janelas largas (`612 → 420` a
> `1 920`). *Quem manda numa janela estreita é a largura de FÁBRICA, e o dono mandou parar de lhe
> tocar.* ⚠️ O caminho de omissão é **byte-idêntico** (o `base` nunca desce
> dos `220`), e a lei do arrasto passou a ler o piso do store porque `screens → interaction` é a
> direcção que DESCE no DAG ⇒ **zero** na catraca que é dívida. ⛔⛔ **NOMEADO e não curado:** um
> controlo de `36 × 36` px transborda a coluna `7 px` **já na largura de fábrica**, e a posição
> dele não é monótona — foi ele que dominou as três primeiras versões da régua desta wave. ⛔ E
> *«o piso não está solto»* fica **mutação NOMEADA**: as três réguas construídas medem o piso e
> não o recurso, porque a única porta que escreve uma largura é a que o piso guarda.


## §9-bis — ⭐⭐⭐ A SEGUNDA WAVE: a largura de fábrica de uma coluna é uma FRACÇÃO da janela

> Ordem do dono, 2026-09-20, sobre a fila do §7.3: **«item 2»**.

### O defeito, medido

As duas colunas eram `308 + 304 = 612 px` **absolutos**, autorados contra a janela que o
`tokens.json` declara ([`HERO_VIEWPORT_W`], `1366`):

| alvo | largura | as duas colunas |
|---|---:|---:|
| iPad 12,9" | `1366` | `44,8 %` — a decisão, tal como foi tomada |
| iPad 11" | `1194` | `51,3 %` |
| iPad mini | `1133` | **`54,0 %`** |

### A lei, e porque ela não tem número novo

[`ChromeBands::default_dock_w`](../../../crates/ph2d-editor-core/src/screens/dock_seam.rs):
*a largura de **fábrica** de uma coluna nunca ocupa mais fracção da janela do que ocupa na
referência*. A fracção é `HIERARCHY_W / HERO_VIEWPORT_W` — **dois tokens que já existiam**, um a
dividir pela janela para que foi autorado. ⛔ *Não há número novo nesta lei: há a decisão que já
estava tomada, aplicada onde ela ainda não chegava.*

⛔⛔ **É um TECTO e nunca uma ESCALA, com o número ao lado:** escalar nos dois sentidos poria as
colunas em `1930 × 308/1366 = 435 px` cada na janela do dono — **`870`** contra `612`. *A cura
tornaria o app pior exactamente onde ele é usado todos os dias.* ⇒ acima da referência ela é
**inerte ao bit**, e há gate a exigi-lo.

⚠️ **Ela não toca na largura que o ARTISTA arrastou** (o `dock_width_choice`) — apertar uma escolha
explícita é o *«aceita e mente»* do §0.0. ⏳ O preço fica declarado: uma escolha gravada num ecrã
largo continua a valer o que vale num estreito.

### ⭐ Sem ramo nenhum, e as três propriedades saem da aritmética

`let escala = (janela_w / HERO_VIEWPORT_W).min(1.0);`

* `1366.0 / 1366.0` é **exactamente** `1.0` em IEEE ⇒ inerte acima da referência, ao bit;
* `f32::min` devolve o **outro** operando com `NaN` ⇒ uma janela sem largura recebe o token. *Toda
  guarda escrita com `<` ou `>` é cega ao `NaN`, e aqui não há guarda a ser cega* — a lei é a mesma
  que a `line/sculpt3d` registou no §30 dela;
* uma janela degenerada recebe o mínimo do painel, que é a única largura que ele sabe desenhar.

⚠️ A 1.ª redacção era `!(janela_w < HERO_VIEWPORT_W)` e **o clippy apanhou-a**
(`neg_cmp_op_on_partial_ord`). A forma sem ramo é melhor por três razões e não por uma.

### O que ela compra, medido pelo gate do orçamento

| alvo | área de desenho antes | depois |
|---|---:|---:|
| iPad 12,9" | `50,6 %` | `50,6 %` — a referência, intocada |
| iPad 11" | `44,0 %` | **`49,6 %`** (`+5,6`) |
| iPad mini | `40,9 %` | **`48,9 %`** (`+8,0`) |

⭐ **Quem mandou subir a catraca foi a METADE DA OBSOLESCÊNCIA dela**, na primeira corrida depois da
lei — não eu.

### ⛔⛔ E o gate do orçamento era CEGO ao defeito

Ele construía as bandas a partir da const `ChromeBands::DEFAULT`, logo media `612 px` **em qualquer
janela** e ficou **verde três semanas** sobre a linha que o [`medicoes/06 §1`](../medicoes/06_o_orcamento_de_ecra_em_tablet.md)
já escrevia. *Um gate que reconstrói a banda em vez de ler a LEI mede a fórmula e não o produto* —
⚠️ **a segunda vez no mesmo ficheiro**: a nota do `tool_bar_lines`, três parágrafos abaixo, regista
a primeira.

⚠️⚠️ **E o `50,6` do 12,9" não é desta wave:** o piso desceu `0,2` pontos em 2026-09-07 e a TABELA
do gate ficou para trás — ela dizia `50,8` sobre um produto que media `50,6` havia duas semanas.
*Quando um ficheiro imprime duas medidas da mesma grandeza e elas discordam, isso É o achado.*

### O gate novo tem SEIS metades, e TRÊS medem a ROTA

Porque esta jornada já pagou **duas** vezes por gates abaixo da rotura (o da paleta entrou pelo
chrome; este entrava pela const):

| metade | o que reprova |
|---|---|
| a fracção é a mesma nos três alvos | a lei |
| o CONTROLO: sem a lei, o mini pagaria mais | que a fixtura contém o fenómeno |
| acima da referência não toca em nada | que a cura não piora o ecrã grande |
| pára no mínimo do painel (+ o controlo de que ele não morde nos três) | que a fracção medida é a da LEI e não a do clamp |
| **a porta do store lê a lei** | o 1.º elo do fio |
| **a escolha do artista atravessa intacta** (+ controlo) | que a lei não aperta uma decisão dele |
| **o quadro entrega `viewport.w`** | o 2.º elo — ⚠️ a agulha é o **braço inteiro**, porque procurar `dock_width` sozinho ficaria verde com `dock_width(side, 1366.0)` escrito à mão, *que é exactamente a regressão* |

### Mutação: **5 de 5 sangram**

| # | mutação | resultado |
|---|---|---|
| M1 | o tecto some (a lei vira escala) | sangra, isola (`6 passed; 1 failed`) |
| M2 | o mínimo do painel some | sangra, isola |
| M3 | a lei inteira inerte | sangra **DUAS** sobre a suíte toda — a fracção **e** o orçamento ⇒ *os dois instrumentos veem a lei* |
| M4 | a porta do produto ignora a janela | sangra, isola |
| M5 | o quadro crava a largura | sangra, isola |

### Os três docs que esta wave reescreveu

`spec/01 §6` (o segundo eixo, que aquela secção não respondia) · `spec/02 §8` · `medicoes/06 §1 e §2`.
⭐ **O `spec/01 §6` não foi contrariado — foi COMPLETADO:** ele recusa a largura **crescer** com o
dedo, e esta lei impede-a de crescer com um ecrã pequeno. *As duas metades dizem a mesma coisa lida
dos dois lados.*

---

## §9-ter — ⛔⛔⛔ O SMOKE DA §9-bis foi REPROVADO — *«não funcionou»* — e a lei estava CERTA

### O que falhou foi o ROTEIRO, e o facto que faltava estava no disco dele

```
~/.ph2d/layout.txt   (active=nodes)
[animation]    dock_w_left=220        dock_w_right=220
[drawing_2d]   dock_w_left=252.83984  dock_w_right=348.1914
[flip]         dock_w_left=252.83984  dock_w_right=296.89063
[modeling_3d]  dock_w_left=220        dock_w_right=371.72266
[nodes]        dock_w_left=220        dock_w_right=220
[vector]       dock_w_left=220        dock_w_right=371.72266
```

**As SEIS bancadas têm largura gravada**, e a corrente é: `install_saved` corre **antes do primeiro
quadro** e chama `set_dock_width` ⇒ o store fica com `Some(w)` ⇒ `dock_width` lê
`stored.unwrap_or(base)` ⇒ **a base nunca é consultada**. ⚠️ E na bancada ACTIVA dele (`nodes`) as
duas colunas estão **no mínimo do painel**, onde nada as pode mexer.

⭐ **A lei protege quem ela tem de proteger:** o `layout.txt` é **por máquina** (`$HOME/.ph2d/`),
logo um tablet acabado de configurar não tem escolha nenhuma e recebe a fracção. *O defeito é o
roteiro mandar procurar o efeito onde ele não podia estar* — a mesma espécie que o §5.0 chama de
**pior que uma cena ausente**.

### ⛔⛔ A cura óbvia foi construída inteira, medida e REVERTIDA

*«A fracção é um tecto sobre QUALQUER largura, não só a de fábrica»* — escrita com porta
(`dock_w_ceiling` sem `min(1,0)`), clamp do gesto no `HeroLayout::dock_width_for`, a escolha
guardada intacta para voltar ao alargar, gate de três metades e **4 de 4 mutações a sangrar**.

**E um gate PRÉ-EXISTENTE reprovou-a:**

> `the_dock_border_resizes_the_column::the_width_grows_with_x_on_the_left_and_shrinks_on_the_right`
> — *«a coluna da esquerda tem de CRESCER 40 (308 contra 348)»*

⇒ **na janela de REFERÊNCIA a coluna de fábrica já ESTÁ no tecto**, logo o artista deixava de poder
**alargar** uma coluna a `1366 px` — para sempre, e hoje ele pode ir até `720`. *Uma cura que
retira um gesto que ninguém pediu é pior do que o defeito que ela cura.*

⛔ **E nenhum número derivado resolve os dois lados.** O único tecto relativo já medido nesta casa
é o `viewport.w * 0.7` do `clamp_panel_rect`, e ele **nunca morde** a maior escolha do dono
(`371,72` passa a `1 930`, `1 366`, `1 133` **e** `744`). *Preservar o arrasto na referência e
apertar a escolha excluem-se com os números que existem; escolher um terceiro é o palpite do §0.0.*

⚠️⚠️ **E a 1.ª ronda de mutação já tinha avisado, noutro sítio:** o `M7` (o tecto deixa de ABRIR no
ecrã largo) **sobreviveu** porque a minha fixtura usava `300` na coluna esquerda, que está **abaixo**
do token daquele lado (`308`) — *os números REAIS do dono estão ACIMA do token e a minha fixtura
estava abaixo: ela não continha o fenómeno*. Trocada pelo `371,722_66` dele, ela sangrou.

### ⭐⭐⭐ E o 3.º report trouxe o LOG, que não decidia nada — daí o readout

O dono colou o log de `resize` (`732x768` → `1920x1022`): ele diz a **janela** e cala o que o app
fez com ela. ⚠️ As duas causas possíveis — *a coluna está na largura de FÁBRICA, que segue a
janela* contra *está numa ESCOLHA gravada, que não segue* — **leem-se exactamente iguais no ecrã**,
e foi isso que deixou **duas** rondas sem conclusão. ⇒ `PH2D_DOCK_LOG=1`, uma linha por **mudança**
de largura (nunca por quadro):

```
[dock] janela=732   esq=220.0 (escolha -)  dir=220.0 (escolha -)  — '-' = largura de fabrica, …
[dock] janela=1920  esq=308.0 (escolha -)  dir=304.0 (escolha -)
```

⛔ **A linha vive DENTRO do `eprintln!`, e isso é MEDIDO:** a 1.ª redacção montava-a num `format!`
para uma variável e o censo de texto da shell **reprovou-a na hora** — a isenção do HR-15 ali é
*«sai por `eprintln!`, logo é terminal»*, e passar pela variável **perde-a sem tirar o literal do
binário** (a lei que a cena `=51` do sculpt3d já tinha pago).

⚠️⚠️ **E o gate do readout teve uma MUTAÇÃO SOBREVIVENTE, por defeito da agulha:** ela procurava
`format!("[dock]` **colados**, e no ficheiro o `cargo fmt` põe a macro e o literal em linhas
diferentes ⇒ *uma agulha que depende da formatação é cega exactamente à formatação que o ficheiro
tem*. Hoje ela acha o literal e olha para TRÁS numa janela de 40 caracteres, que é
layout-independente. Mutação **2 de 2** (a palavra que identifica a coluna · a linha a sair do
`eprintln!`).

### ⛔⛔⛔ E o readout apanhou o DEFEITO REAL: um arrasto que não muda nada tira a coluna da lei

A linha que o dono colou, com perfil NOVO:

```
[dock] janela=473  esq=220.0 (escolha -)  dir=220.0 (escolha 220)
```

A `473 px` a lei **já** entrega o mínimo do painel nas duas colunas. A esquerda está de fábrica; a
direita tem `escolha 220` — **o próprio número que a lei dava**. E o ficheiro do perfil
descartável confirmava-o no disco (`[drawing_2d] dock_w_right=220`), que é porque ele só era novo
na 1.ª corrida.

⇒ **tocar na borda de uma coluna com a janela apertada gravava a largura de FÁBRICA como se fosse
uma decisão do artista**, e a partir daí aquela coluna deixava de seguir a janela em toda largura,
sem outra saída além do *Reset Panel Layout*. *Um gesto que não move um pixel não pode ter uma
consequência permanente e invisível.*

⭐ **Não é desenho novo:** o doc do `dock_width_choice` já dizia *«persistir o valor de
`dock_width` escreveria o default como se fosse uma escolha»*. O default **mudou** nesse mesmo dia
(passou a seguir a janela) e o arrasto era quem o gravava como escolha.

**A cura, em três peças:**

| onde | o quê |
|---|---|
| `ChromeBands::escolha_de_um_arrasto` | a LEI: `None` quando o gesto aterra na largura de fábrica |
| `WidgetStore::set_dock_width` | passa a aceitar `Option<f32>` — `None` **apaga** a excepção |
| `dock_resize::dock_seam_move` | escreve a resposta da lei, nunca um `Some` montado ali |

⛔⛔ **A porta nasceu no sítio errado e foi a CATRACA DO DAG que a mudou:** posta no store, a
aresta `interaction → screens` subiu de `18` para `20` e a catraca reprovou — *a lei desta casa é
curar por movimento, nunca subir o número*, e ela apontou para onde a porta devia estar (os
números da decisão são os da `ChromeBands`, e o store é estado autorado que não conhece a janela).
⭐ E a mudança do setor para `Option<f32>` custa **zero** referências novas.

⛔⛔ **E o gate reprovou a 1.ª redacção da lei:** ela comparava a largura **CRUA** do gesto, e o
store clampa ao escrever — arrastar para lá do mínimo dava `|80 − 220| = 140` ⇒ *«é uma escolha»*,
e o que ficava gravado era `220`, **exactamente o caso do report**. *Uma lei que julga o pedido
enquanto o consumidor guarda o pedido CLAMPADO julga um número que ninguém grava.* ⇒ o piso entra
na lei, e um gate exige que ele seja o **mesmo token** que o store lê.

⚠️ **E o gate de costura irmão NÃO servia:** o `the_border_gesture_reaches_the_panel` procura
`set_dock_width` no `dock_seam_move`, e o nome da porta nova **contém-no** ⇒ ele ficava verde com
a regressão inteira dentro. *Uma agulha que é PREFIXO da cura não distingue a cura do defeito.*

Mutação **5 de 5** (a lei julga o cru · toda largura vira escolha · nenhuma vira · a shell volta à
porta crua · o piso do store deixa de ser o token).

### ⏳ ABERTO, com o número — DECISÃO DO DONO

Uma escolha é gravada em **pixels absolutos** e não sobrevive a uma mudança de forma da janela: no
iPad mini, `371,72 px` são **`32,8 %`** deitado (`1 133`) e **`50,0 %`** em pé (`744`). As duas
saídas têm preço: um tecto pede um número que ninguém mediu; guardar a escolha como **fracção** muda
o que arrastar uma borda significa **e** o formato do ficheiro de arrumação.

### ⛔⛔⛔ E o 2.º smoke TAMBÉM foi reprovado — *«não diminuiu os paineis»* — e outra vez o ROTEIRO

O perfil descartável funciona (`layout_persist::layout_file` lê `std::env::var_os("HOME")`), e a
lei chega ao pixel. O que faltava era **um número que nenhum documento tinha**:

| | |
|---|---|
| a coluna só se MEXE entre | **`976` e `1 366` px de janela** |
| a janela ABRE em | **`1 024 px`** (`init.rs`: `with_inner_size(1024, 768)`) |

Acima de `1 366` a lei é inerte **de propósito** (ela é um TECTO); abaixo de `976` o **mínimo do
painel** prende-a (`220 × 1366 / 308 = 976` à esquerda, `989` à direita). ⇒ **num perfil novo o app
abre já quase no chão**, e o meu roteiro mandava **ESTREITAR** — *a direcção onde não há nada para
ver*. É a espécie que o §5.0 chama de **pior que uma cena ausente**, pela segunda vez no mesmo dia.

⇒ o gesto que mostra a lei a partir de um perfil novo é **ALARGAR**:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX \
  && env HOME=/tmp/ph2d-smoke-novo cargo run -p ph2d-host-desktop --profile smoke
```

…e depois **maximizar** a janela: as duas colunas crescem `231 → 308` e `228 → 304` e **param**.

### ⭐⭐ E o gate que faltava mede o PIXEL, não a lei

A lei tinha **três** gates — a lei, a porta do store e o **TEXTO** do `frame_layout` — e *nenhum
percorria a rota até ao rectângulo que a coluna OCUPA*. O
[`a_coluna_pintada_encolhe_com_a_janela`](../../../shells/desktop/tests/it/a_coluna_pintada_encolhe_com_a_janela.rs)
pinta quatro quadros pela rota real em sete larguras e afirma a tabela:

| janela | esquerda | direita |
|---:|---:|---:|
| `1 930` · `1 600` · `1 366` | `308,0` | `304,0` |
| `1 194` | `269,2` | `265,7` |
| `1 133` | `255,5` | `252,1` |
| `1 024` | `230,9` | `227,9` |
| `900` | `220,0` | `220,0` |

⚠️ **O CONTROLO vem primeiro e não é decoração:** a 1.ª corrida do arnês leu **`0,0` em tudo** — o
`HeroScreen::new` nasce sem painel nenhum visível, e sem a semente do manifesto todas as
desigualdades de *«encolheu»* passariam **por vácuo**. Mutação **2 de 2** (cravar `1366.0` no
quadro · a lei inteira inerte).

---

## §9-quater — ✅ **O SMOKE DO DONO APROVOU a §9-bis + a cura da §9-ter, e o log DELE mede os dois pisos**

O dono correu o roteiro com o perfil limpo (`HOME=/tmp/ph2d-smoke-novo`, `PH2D_DOCK_LOG=1`) e colou
o readout de **~70 eventos de redimensionamento**, de `1 920` a `647 px`. Ele fecha as duas metades:

| janela (px) | esquerda | direita | escolha |
|---:|---:|---:|---|
| `1 920` · `1 780` · `1 595` · `1 370` | `308,0` | `304,0` | `-` · `-` |
| `1 335` | `301,0` | `297,1` | `-` · `-` |
| `1 187` | `267,6` | `264,2` | `-` · `-` |
| `1 020` | `230,0` | `227,0` | `-` · `-` |
| `991` | `223,4` | **`220,5`** | `-` · `-` |
| `983` | `221,6` | **`220,0`** | `-` · `-` |
| `977` | **`220,3`** | `220,0` | `-` · `-` |
| `965` … `647` | **`220,0`** | `220,0` | `-` · `-` |

⭐⭐⭐ **A metade que a cura da §9-ter defende:** em **todas** as linhas, dos dois lados, a coluna da
escolha lê **`-`**. O defeito reportado era exactamente o contrário — `dir=220.0 (escolha 220)` numa
coluna que ele nunca escolheu —, e ele não volta a aparecer numa varredura de ponta a ponta da
faixa. ⚠️ *Ele não tocou numa borda nesta corrida, que é precisamente a condição do gate.*

⭐⭐⭐ **E o log dele mede os DOIS pisos separadamente, que nenhuma fixtura desta linha tinha feito.**
A lei prende cada lado quando `token × janela/1366` chega a `PANEL_MIN_W_PX`, logo os dois pisos
caem em janelas **diferentes**: `220 × 1366 / 304 = 988,4` à direita e `220 × 1366 / 308 = 975,6` à
esquerda. O readout dele mostra a direita a assentar entre `991` e `983`, e a esquerda entre `977` e
`965` — **as duas janelas previstas, com `13 px` de separação entre elas**, medidas na máquina do
dono pela rota do produto. *O gate desta linha mede as duas na mesma corrida a `900 px`, onde já
assentaram; a curva dele é a prova independente de que cada piso é o do PRÓPRIO token.*

⚠️ **O que este smoke NÃO cobre, e fica nomeado:** os passos (3) e (4) do roteiro — arrastar uma
borda de propósito (a coluna passa a `escolha <número>` e sai da lei) e arrastá-la de volta ao
tamanho de fábrica (volta a `escolha -`). Essa metade tem os três gates da §9-ter
(`um_arrasto_que_aterra_na_largura_de_fabrica_nao_grava_excepcao`, o controlo
`e_um_arrasto_para_outra_largura_continua_a_ser_uma_escolha` e `o_piso_desta_lei_e_o_piso_do_store`)
e **não tem smoke do dono**.

⏳ **E a decisão que continua com ele** (§7): a largura arrastada é guardada em **píxeis absolutos**,
logo num tablet que roda os `371,72 px` do ficheiro dele são `32,8 %` do ecrã deitado e `50,0 %` em
pé. As duas saídas têm preço e nenhuma é derivável de uma medição desta linha.

---

## §9-quinquies — ⛔⛔⛔ O 4.º REPORT: *«pare de tentar. permita que manualmente o usuário consiga estreitar o painel»*

> *«não funciona. pare de tentar. permita que manualmente o usuário consiga estreitar o painel.»*
> — Enio, 2026-09-20, logo a seguir ao smoke que ele **aprovou** na §9-quater.

⚠️ **As duas frases não se contradizem, e ler isso mal era o risco desta wave.** O que ele aprovou
foi a coluna a seguir a JANELA (o log dele mede-a de `1 920` a `647`); o que ele reprovou foi o
passo (3) do roteiro — **arrastar a borda de propósito**. A ordem é: pára de trabalhar na lei
automática, e dá-me o controlo manual.

### A causa, MEDIDA antes de uma linha de cura

O log dele acaba em `647 px`. Ali a lei da fracção **já entrega o mínimo** nos dois lados, e até
esse dia o piso da ESCRITA era o mesmo número ⇒ o gesto pedia `157` e o store devolvia `220`.
Reproduzido pela rota real (pintar quatro quadros, achar a costura, arrastar `60 px`):

| janela | coluna | arrastar para estreitar |
|---:|---:|---|
| `1 920` | `308,0` | `308,0 → 245,0` ✅ |
| `1 024` | `230,9` | `230,9 → 220,0` (pára no piso) |
| `640` | `220,0` | **`220,0 → 220,0`** ⛔ **inerte** |

⭐ *Um gesto que existe, arma, segue o dedo e não muda um pixel lê-se como um gesto partido* — e é
a mesma forma que o arrasto fantasma da §9-ter pagou do outro lado. **Nenhum gate desta linha o
via**, porque os cinco mediam a largura de FÁBRICA e este é o caminho da ESCOLHA.

### ⭐⭐⭐ A LEI: o piso de FÁBRICA protege quem não escolheu; um arrasto É uma escolha

São **dois** pisos, de propósito:

- a largura de FÁBRICA (`ChromeBands::default_dock_w`) continua a parar em `PANEL_MIN_W_PX`
  (`220`) — ninguém pediu para o app **nascer** com uma coluna ilegível;
- a largura que o artista **arrasta** (`WidgetStore::DOCK_W_MIN`) pára mais abaixo, porque ele
  pediu — e **o número é dele** (ver §9-sexies).

⚠️ **O caminho de omissão é byte-idêntico:** o `dock_width` clampa `stored.unwrap_or(base)` e o
`base` nunca desce dos `220` por construção ⇒ *uma arrumação sem escolha nenhuma lê exactamente o
mesmo número de ontem*. ⭐ E a lei do arrasto passou a ler o piso **do store** em vez do token: a
direcção `screens → interaction` é a que DESCE no DAG da fundação (há sentinela a exigi-la), logo
custa **zero** à catraca `interaction → screens`, que é a dívida — a mesma catraca que na §9-ter
tinha apontado para onde a porta devia estar.

### ⚠️ O `84` é medido, e diz de que recurso é

Ele é a largura em que o **corpo de um painel docado** ainda cabe na coluna. Varrido pixel a pixel
pela rota real (baixando o piso à mão e lendo o índice de toque):

| coluna | controlos do corpo que saem dela |
|---:|---:|
| `86` · `85` · **`84`** | `0` |
| `83` | `1` (excesso `1,0 px`) |
| `80` | `2` |
| `64` | `3` |

⭐ **E a varredura correu com CADA um dos 19 painéis docáveis à frente, não só os de fábrica:**
descer de `220` para `84` acrescenta no máximo **`1,0 px`** de transbordo, e num só painel
(`physics`). Os outros dezoito acrescentam **zero**.

⛔ **O piso NÃO é o da faixa de abas, e ela foi medida:** o `tab_plan` ainda entrega uma aba a
`32 px` e só desiste a `24`. Parar aí entregaria uma coluna cujo corpo pinta por cima da área de
desenho — *o piso é do CORPO, que é o que falha primeiro*.

### ⛔⛔ PRÉ-EXISTENTE e NOMEADO, não curado aqui

Há um controlo de **`36 × 36` px** que transborda a coluna **`7,0 px` já na largura de fábrica de
`220`**, e a posição dele **não é monótona** na largura (`x` lê `191` a `220`, `50` a `84` e `90` a
`90`). Ele não é um problema de piso — a essa largura o produto de hoje shipa igual — e foi ele que
dominou as três primeiras versões da minha régua. ⇒ *uma régua de transbordo ABSOLUTO neste app
mede esse widget; a que decide é o **acréscimo** contra a largura de fábrica.*

### ⛔⛔⛔ A metade que NÃO é gateável, com o mecanismo

*«O piso não está SOLTO»* ficou como **mutação NOMEADA** (um piso de `120` sobrevive à suíte), e
não por falta de vontade: **três** réguas foram construídas e as três medem o piso em vez do
recurso.

1. *«alguma coisa toca a borda»* — quase toda fileira de painel é **elástica** e enche a coluna em
   qualquer largura ⇒ verdadeira em todo número que se escreva ali.
2. *«o controlo fixo mais largo»*, com o 2.º quadro em `piso + folga` — os dois quadros **movem-se
   com a constante medida** e um piso de `120` lê `118`. *Uma barra derivada da constante que ela
   mede não pode medi-la.*
3. A mesma, com a âncora FIXA no `PANEL_MIN_W_PX` — os controlos de largura fixa do cabeçalho são
   **alinhados à DIREITA**, logo o canto direito deles acompanha a coluna e a conta volta a ler
   `piso − 2`.

⚠️ **O que a fecharia é uma porta que escreva uma largura ABAIXO do piso**, e a única que existe é
a que o piso guarda. *Uma régua que quer ver o outro lado de uma cerca teria de derrubar a cerca*,
e um `set` sem clamp só para teste seria a segunda porta pela qual o defeito da §9-ter voltava.

### Os gates

| gate | o que afirma |
|---|---|
| `numa_janela_estreita_o_arrasto_ainda_estreita_a_coluna` (novo) | o report reproduzido: a `640 px`, com as duas colunas NO piso de fábrica, um arrasto de `60 px` estreita — e aterra **onde o dedo pediu**, não no piso |
| `no_piso_de_uma_escolha_nada_do_corpo_sai_da_coluna` (novo) | no piso, **nada** do corpo sai da coluna — ⚠️ ele nasceu `o_piso_de_uma_escolha_e_onde_o_corpo_do_painel_ainda_cabe` e **o nome passou a mentir** horas depois (§9-sexies) |
| `o_piso_desta_lei_e_o_piso_do_store` (**premissa morta**, reescrito) | a lei e o store clampam no MESMO número — e ele já não é o token, logo passa a ser afirmado em vez de herdado |
| `um_arrasto_que_aterra_na_largura_de_fabrica_nao_grava_excepcao` (**fixtura morta**, reescrito) | o gesto mudo passou a ser *aterrar exactamente na fábrica*; a metade nova mede que por baixo dela o gesto **já não é mudo** |
| `reopening_restores_the_width_the_column_had_before_the_drag` (fixtura **derivada**) | o literal `100` deixou de estar do outro lado da cerca; hoje o pedido sai do próprio piso |
| cerca de compilação em `dock_width_ops.rs` | o piso de uma ESCOLHA `<` o de FÁBRICA — o clippy recusou-a como asserção de teste, e tinha razão |

**Mutação: 5 sangram + 1 NOMEADA.** ⚠️ Duas reprovações foram **minhas** e ficam registadas: a 1.ª
redacção do gate do report exigia que o arrasto aterrasse no PISO (e ele aterra onde o dedo pediu —
`220 → 157` —, que é a cura a funcionar), e a metade nova do gate do arrasto fantasma foi escrita
**a meio** da função, partilhando o store com a asserção de cima: ela apagava o *«sem escolha»* que
aquela existe para medir. *Duas metades de um gate que partilham estado medem a segunda duas vezes
e a primeira nenhuma.*

### ⏳ O que este report NÃO pediu, e fica

O dono disse **«pare de tentar»** sobre a lei automática ⇒ a largura de FÁBRICA não se mexeu. Numa
janela de `473 px` as duas colunas continuam a nascer com `220` cada (`93 %` do ecrã) e o que muda
é ele poder agora levá-las a `84` cada (`36 %`). *Descer o piso de FÁBRICA é outra decisão, e é
dele.*

---

## §9-sexies — ⭐⭐⭐ OS 5.º E 6.º REPORTS SÃO NÚMEROS: *«no mínimo o dobro»*, depois *«mais 25 %»*

> *«a largura mínima precisa ser no mínimo o dobro que a largura mínima que vc definiu.»*
> — Enio, 2026-09-20, depois de ver a coluna a `84 px`. ⇒ `168`.

> *«ainda muito estreito. aumente 25%.»*
> — Enio, 2026-09-20, depois de ver a coluna a `168 px`. ⇒ **`210`**.

⇒ `WidgetStore::DOCK_W_MIN` **`84 → 168 → 210`**, que é `2,5 ×` o piso medido.

### ⚠️⚠️ E isto separa duas coisas que estavam coladas

O `84` era o **RECURSO** — medido, e é o que o produto CONSEGUE fazer. O que shipa é a
**DECISÃO** — o que ele DEVE fazer —, e ela é do dono. *Um piso posto no recurso entrega uma
coluna que cabe e não serve*, e foi exactamente o que ele viu.

⇒ o recurso ganha nome próprio (`WidgetStore::PISO_DO_CORPO_PX = 84`) e **fica**, porque é a
proveniência do que shipa: sem ele, *«o dobro»* deixa de ter **de quê**. O valor que shipa é
escrito à mão de propósito — ⛔ **não** `2.0 * PISO_DO_CORPO_PX`: assim escrito, a cerca que o
defende seria verdadeira por construção, e *uma linha que a mutação não consegue matar é
comentário com sintaxe de código*.

### ⭐⭐ As ordens dele apertaram a faixa legal NOS DOIS LADOS

| cerca | quem a põe |
|---|---|
| `DOCK_W_MIN ≥ 2,5 × PISO_DO_CORPO_PX` (`210`) | as duas ordens do dono, 2026-09-20 |
| `DOCK_W_MIN < PANEL_MIN_W_PX` (`220`) | senão o gesto volta a ser inerte numa janela estreita |

As duas são **erro de compilação** (`E0080`), ao lado das duas do degrau de fechar que já lá
estavam. ⚠️ **A cerca de baixo leva a ordem MAIS RECENTE e não as duas escritas lado a lado:** são
do mesmo dia e a segunda subsume a primeira, logo mantê-las às duas daria a UMA delas o poder de
aprovar um valor que a outra recusa. ⭐ Antes da 1.ª ordem só havia cerca por baixo e um piso
solto era inatacável em toda a recta; hoje a janela em que ele pode mentir tem **`10 px`**, e é
essa a mutação NOMEADA que sobra.

### ⚠️⚠️ O que o dono ganha — e a consequência que ele tem de ver

| janela | duas colunas de fábrica | o mínimo a que ele as leva | o que o gesto compra |
|---:|---:|---:|---:|
| `473` | `220 + 220` = **440** (93 %) | **420** (89 %) | `20 px` |
| `647` | `220 + 220` = **440** (68 %) | **420** (65 %) | `20 px` |
| `1 024` | `231 + 228` = **459** (45 %) | **420** (41 %) | `39 px` |
| `1 920` | `308 + 304` = **612** (32 %) | **420** (22 %) | `192 px` |

⛔⛔ **A consequência está nas duas primeiras linhas, e ela é o report de onde tudo isto nasceu:**
numa janela estreita a lei de FÁBRICA já entrega `220`, logo com o piso em `210` o arrasto compra
**`10 px` por coluna** e mais nada. *O curso a sério do gesto está nas janelas largas.*

⚠️ **Isto não é uma objecção — é a aritmética da decisão dele, posta à frente dele.** Quem manda
na largura numa janela estreita é a **lei de fábrica**, e ele mandou parar de lhe tocar
(*«pare de tentar»*, §9-quinquies). ⇒ o dia em que ele quiser espaço a `647 px` é o dia em que
essa nota reabre, e ela é **decisão dele**, não trabalho pendente.

### ⚠️ E DOIS gates meus reprovaram sobre produto correcto, os dois pela mesma forma

- `numa_janela_estreita_o_arrasto_ainda_estreita_a_coluna` arrastava **`60 px` à mão**, e com o
  piso a `168` esse gesto passou a pedir por baixo dele. ⇒ a mão passa a ir a **meio caminho
  entre a largura de fábrica e o piso**, que é um gesto legítimo em qualquer dos dois. *Uma
  fixtura escrita com um literal do outro lado de uma cerca move-se com a cerca* — é a **terceira
  vez** nesta jornada (as outras: o `100` da involução e o `80` do arrasto fantasma).
- `o_piso_de_uma_escolha_e_onde_o_corpo_do_painel_ainda_cabe` **passou a mentir no NOME**: o piso
  que shipa já não é onde o corpo deixa de caber, é o dobro disso. Renomeado para
  `no_piso_de_uma_escolha_nada_do_corpo_sai_da_coluna`, que é o que o corpo dele afirma. *Um nome
  que promete mais do que o teste mede mente em toda corrida verde.*

⛔⛔ **E a mutação «piso apertado» deixou de ser expressável sozinha:** com a cerca no sítio,
qualquer valor abaixo de `210` **não compila** ⇒ a prova de mutação passa a derrubar **a cerca E
o valor** (`60`), que é a forma honesta de perguntar *«e se alguém decidir descer isto?»*.

⚠️ **E a mutação NOMEADA teve de mudar de número:** ela era `210` — *«um piso solto dentro da
faixa legal»* — e `210` passou a ser **o valor que shipa**. Hoje é `215`, e a faixa em que ela
vive são os `10 px` que sobram. *Uma mutação cujo valor o produto adopta deixa de ser uma
mutação.*

**Mutação: 4 sangram + 2 CERCA + 1 NOMEADA, de 7.**

---

## §9-septies — ⛔⛔⛔ O 7.º REPORT: eu li um DEFEITO como um PEDIDO, e quase construí a coisa oposta

> *«hierarquia mais larga que inspector. Inspector OK»* — Enio, 2026-09-20.

⚠️⚠️ **Eu li isto como um PEDIDO** (*«faz a Hierarquia mais larga»*) e comecei a derivar um piso
**por lado**. Perguntado com as opções na mesa, o dono fechou-o numa frase: *«Hierarquia deve ser
como o inspector … mesmo limite de largura»* — era um **REPORT**: a Hierarquia está mais larga, e
não devia.

⭐⭐⭐ **As duas leituras dão produtos OPOSTOS** — uma alarga a Hierarquia, a outra iguala-a ao
Inspector —, e a frase sozinha admite as duas. *Uma frase que descreve um estado pode ser o pedido
para o criar ou o report de que ele existe, e o custo de adivinhar mal é construir o contrário.*

### ⭐ E o produto JÁ obedecia — medido antes de tocar em nada

Pela rota real (as quatro portas que o `dock_seam_move` encadeia), empurrando o dedo até ao
extremo do ecrã de cada lado:

| janela | coluna esquerda | coluna direita |
|---:|---:|---:|
| `1 920` | `210,0` | `210,0` |
| `1 366` | `210,0` | `210,0` |
| `1 024` | `210,0` | `210,0` |

E os rectângulos PINTADOS (`layout.hierarchy` / `layout.inspector`) são idênticos ao décimo nas
três. ⇒ **esta wave não cura nada no piso: ela PRENDE uma propriedade que ninguém tinha escrito.**
O gate `as_duas_colunas_tem_o_mesmo_limite` existe porque um piso por-lado — que eu estive a uma
decisão de escrever — a partiria **em silêncio**. *Uma simetria que só existe porque a constante é
uma só deixa de existir no dia em que houver duas.*

### ⛔⛔ E o ficheiro de arrumação do dono diz o CONTRÁRIO do report, nos SEIS espaços

`~/.ph2d/layout.txt`, no dia do report:

| espaço | `dock_w_left` | `dock_w_right` |
|---|---:|---:|
| `animation` | `220` | `220` |
| `drawing_2d` | `252,8` | `348,2` |
| **`flip`** (o activo) | `220` | `389,6` |
| `modeling_3d` | `220` | `371,7` |
| `nodes` | `220` | `270,7` |
| `vector` | `220` | `389,6` |

Em **todos**, a coluna da DIREITA é a mais larga. ⇒ ou a Hierarquia não está à esquerda na
bancada dele, ou o que ele viu não é a largura da coluna. **Não reproduzi**, e é isso que a wave
entrega: o instrumento que responde.

### ⭐⭐⭐ E o readout tinha um BURACO que custou esta ronda inteira

Ele imprimia *«uma linha por mudança de largura da JANELA»* ⇒ **arrastar uma borda não produzia
linha nenhuma**. O dono não tinha como me mostrar o que o gesto dele fazia, e eu não tinha como
distinguir *«o gesto não arma»* de *«o gesto arma e pára cedo»*. ⚠️ *Um instrumento que só vê a
metade do fenómeno que não está sob suspeita não bissecta* — e esta é a **segunda** vez que o
mesmo readout é alargado por não conter a pergunta do dia (a 1.ª foi a coluna da ESCOLHA, §9-ter).

Duas curas, as duas gateadas:

- a chave passa a ser a **trinca** `(janela, esquerda, direita)` ⇒ todo arrasto imprime;
- a linha **NOMEIA o painel** de cada coluna e diz o **piso** ⇒ quem a lê deixa de ter de
  adivinhar de quem é o número, que é exactamente o que me faltou aqui.

```
[dock] janela=1920  esq=210.0 (escolha 210) [hierarchy]  dir=210.0 (escolha 210) [inspector]  piso=210  — …
```

⚠️ **Tudo dentro de UM `eprintln!`**, pela razão medida da §9-ter: montar a linha num `format!`
perde a isenção de terminal do HR-15 **sem tirar o literal do binário**.

**Mutação: 7 sangram + 2 CERCA + 1 NOMEADA, de 10.** ⚠️ A M10 (*«o readout deixa de nomear o
painel»*) **não compilou** à primeira — tirar o marcador deixa o argumento órfão —, e o arnês
abortou-a em vez de a contar: *uma mutação que não compila lê-se exactamente como uma que sangra.*
Hoje ela é uma edição de duas partes.

---

### ✅ E o instrumento FECHOU o report na primeira colagem, com ZERO linhas de produto

O dono colou duas linhas:

```
[dock] janela=1920  esq=220.0 (escolha 220) [hierarchy]  dir=210.0 (escolha 210) [inspector]  piso=210
[dock] janela=1920  esq=210.0 (escolha 210) [hierarchy]  dir=210.0 (escolha 210) [inspector]  piso=210
```

⭐⭐⭐ **A coluna da ESCOLHA responde tudo:** o `220` da Hierarquia vinha com `escolha 220`, logo era
uma largura **GRAVADA** e não um limite — e depois do arrasto ela aterra em `210`, o mesmo do
Inspector. ⇒ **não havia defeito**: os dois limites sempre foram o mesmo, e o que o dono viu foi um
número velho do `~/.ph2d/layout.txt` dele (o `[flip]` tinha `dock_w_left=220`, escrito quando `220`
ERA o piso). Depois dos arrastos o ficheiro lê `210/210`.

⚠️ **As duas metades do readout que esta wave acrescentou foram as duas necessárias, e nenhuma
sozinha bastava:** sem a que dispara no ARRASTO não havia segunda linha; sem a que NOMEIA o painel
não se sabia de quem era o `220`. *O que resolveu não foi uma cura — foi o instrumento passar a
conter a pergunta.*

### ⏳ ABERTO e NOMEADO: cinco espaços de trabalho do dono ainda carregam escolhas velhas

O `~/.ph2d/layout.txt` dele, depois desta sessão:

| espaço | `dock_w_left` | `dock_w_right` |
|---|---:|---:|
| `animation` | `220` | `220` |
| `drawing_2d` | `252,8` | `348,2` |
| **`flip`** | **`210`** | **`210`** |
| `modeling_3d` | `220` | `371,7` |
| `nodes` | `220` | `270,7` |
| `vector` | `220` | `389,6` |

⛔ **Aqueles `220` são ESCOLHAS, logo aquelas colunas NÃO seguem a janela** — é o report original
(*«não diminuiu os painéis»*) ainda latente em cinco espaços. Eles foram gravados antes da cura da
§9-ter e podem ser **escolhas fantasma** daquele defeito, ou o dono a ter arrastado até ao mínimo
da época; ⚠️ **os dois casos são indistinguíveis no ficheiro** (não há marca de versão, e `220` é
uma largura legítima). ⇒ *o app não os pode limpar sozinho sem apagar uma escolha real*, e a cura é
do dono: arrastar a borda naquele espaço, ou apagar o ficheiro. **Decisão dele; não é trabalho
pendente desta linha.**

---

## §9-octies — ⛔⛔⛔ *«ataque o inspector»* — e o número que o punha no topo era uma MIRAGEM

> *«ataque o inspector»* — Enio, 2026-09-21.

### Passo zero: a superfície de colisão, medida NO DIA

O §7.3 avisava que o Inspector é escrito em boa parte pela `line/components`, **viva**, e que *«a
pergunta tem de ser refeita no dia»*. Refeita:

| linha | commits | ficheiros | tocam o Inspector |
|---|---:|---:|---:|
| **`line/components`** | `23` | `147` | **`58`** |
| `line/Vector` · `line/sculpt3d` · `line/3DModeling` · `line/motion-value` · `line/PainterWatercolor` | — | — | **`0`** |

⭐ **Mas os `58` são secções NOVAS** (arma · abanão · script · HUD · contador · gatilho): `6`
ficheiros de ids e `7` de secção. Cruzado com a dívida, elas valem **`24` dos `314` comandos
(`7,6 %`)**. *A colisão existe, é pequena e está nomeada.*

### ⛔⛔ E então a medição derrubou a premissa da wave

Os `314` que puseram o Inspector no topo **não são `314` comandos**. Olhados um a um, os `60` do
bloco base são:

| o que são | quantos |
|---|---:|
| `insp_vis_layer_bit_0..31` — as **32 camadas de colisão**, que são UMA grelha de bits | `32` |
| `insp_phys_join_kind_*` — **uma escolha** entre 9 tipos de junta | `9` |
| `insp_vis_mask_*` · `insp_vis_clip_*` · `insp_order_sp_*` · `insp_render_*` — selectores | `~14` |
| **comandos a sério** (`transform_reset` · `join_draw` · `rig` · `corner_equalize` · `on_screen`) | **`5`** |

⭐⭐⭐ **E o PRODUTO está certo:** as 32 camadas são pintadas por um widget só
([`BitmaskGrid32`](../../../crates/ph2d-editor-core/src/widget/bitmask_grid32.rs)), com o valor a
vir do documento; os 32 ids são **alvos de toque** de um controlo. *Quem mente é o censo* — ele
classifica pelo SUBSTRATO, e um `InteractiveState::Button` é um comando. ⇒ **um painel de
PROPRIEDADES, que é o que mais usa selectores, lidera a dívida por causa disso**, e a `D2` teria
mandado uma máscara de bits para a barra do topo.

### ⛔ A regra BARATA foi tentada e falha nos DOIS sentidos

*«um id dentro de um ARRAY é célula; um `const` escalar é comando»* — medido:
`INSP_ORDER_SP_CENTER`/`_PIVOT`/`_CUSTOM` são **três escalares** que formam um selector, e
`INSP_INSTANCE_DROP_ORPHAN: [NodeId; N]` é um **array que é uma LISTA** de botões distintos. ⇒ a
fonte não sabe responder: **quem sabe é quem PINTA**.

### ⭐⭐ A cura: os pintores canónicos declaram o grupo

[`ph2d_editor_core::widget::composto`](../../../crates/ph2d-editor-core/src/widget/composto.rs), no
molde do censo de elisões: **armado mede, desarmado é um `Cell::get` e um `return`**. Três ganchos
cobrem o app inteiro — `paint_segmented_group`, `paint_segmented_group_adaptive` e
`paint_bitmask_grid32`. ⭐ **As 12 cópias locais de «fileira segmentada» não foram tocadas:** elas
são embrulhos finos por cima do funil, e foi isso que tornou a wave pequena.

### O que a régua corrigida diz

| painel | antes | **agora** | |
|---|---:|---:|---|
| `inspector` | `314` | **`150`** | continua no topo, agora com um número honesto |
| `tokens` | `110` | `110` | não usa compostos |
| `physics` | `60` | `49` | |
| `sculpt3d` | `102` | **`36`** | |
| `vector` | `45` | `24` | |
| **`model3d`** | `39` | **`1`** | ⭐ era **quase só** selectores |
| **TOTAL** | `918` | **`610`** | |

⇒ *a ordem da fila mudou*, e o `model3d` — que uma wave teria «triado» — não tem dívida nenhuma.

### ⚠️⚠️ O gate de CONTROLO reprovou, e não foi afrouxado

O `o_painel_ja_triado_reproduz_o_numero_da_triagem` amarra a régua à triagem que o **dono** fez em
2026-09-01 (`~57` entradas no `3D Model`). Com os compostos colapsados ele lê **`21`** e reprovou.

⛔ **Alargar a banda mataria o controlo.** A triagem contou **ALVOS DE TOQUE** e a régua nova conta
**CONTROLOS** — são duas grandezas. ⇒ a `Contagem` passa a saber as duas (`total()` e `alvos()`), a
metade velha fica **intacta** sobre a grandeza antiga, e a metade nova afirma que elas **diferem**.
*Uma régua que troca de grandeza tem de conseguir reproduzir a antiga, senão ninguém sabe se ela
melhorou ou se se partiu.*

### ⚠️ E DUAS mutações acharam buracos no meu próprio gate

- **declarar ≠ contar:** um censo que declare o grupo e o conte como ZERO passava — o painel some
  da dívida sem uma linha mudar. ⇒ o gate afirma que um grupo conta como **um VALOR**.
- **o controlo do «desarmado» media a corrida ANTERIOR:** o `grupos()` devolve *o que foi
  registado desde o `arma`*, e o `medindo` só esvazia ao ARMAR ⇒ lê-lo depois de uma pintura
  desarmada devolve os 39 grupos da armada. *Um censo com memória mede-se pelo DELTA, nunca pelo
  valor.*

**Mutação: 5 de 5 sangram.**

### ⏳ O que fica para a wave seguinte, com o número honesto

O Inspector tem **`150` comandos** sobre `582` entradas. A `D2` agora pode ser feita sem mandar um
selector para a barra do topo — e **`24` deles são da `line/components`**, logo a triagem deles
espera a fusão.

---

### O portão desta wave

| passo | resultado |
|---|---|
| `scripts/nextest-impacted.sh` | **`15 490 / 15 490`** · ⚠️ uma reprovada, **flake de carga já NOMEADA** no §5.0 (`the_cost_of_sampling_a_path_is_flat_in_its_anchors`): `3` de `3` verde sozinha a `load 20`–`22`, e **zero linhas** desta linha na `ph2d-timeline` |
| `cargo clippy --all-targets -- -D warnings` | zero |
| `cargo fmt --check` | limpo |
| `scripts/censos-da-arvore-combinada.sh` | **`127 / 127`** |
| prova de mutação | **5 de 5 sangram** |

⚠️ **E o portão apanhou TRÊS vermelhos que o laço interno não vê**, os três legítimos: o bloco
`mod` do `widget/` é **gerado** (⇒ o módulo novo entra no `PUB_MODULE_OVERRIDE` do gerador, com a
razão escrita, e não editando a saída dele); a galeria de widgets exige que **todo** ficheiro de
`widget/` seja mostrado ou tenha isenção escrita (⇒ isenção, porque o `composto` não pinta um
pixel); e o índice da memória **conta** o que aponta (a lição que esta linha guardou mais cedo
deixou a contagem da família em `55` com `56` no ficheiro).

---

## §9-nonies — ⛔⛔⛔ *«Shape: Texture ficou com nomes com pontos»* — três cópias, duas traduziam

> Report do dono, 2026-09-21, com foto: a secção `SHAPE ▸ Texture` do Painter, com o padrão
> **Wood** escolhido, pintava `paint_brush.pattern_param.contrast` no lugar de `Contrast`.

⭐ **A tabela sabia traduzi-las** — medido, `tr` devolve `Contrast`, `Brightness`, `Turbulence`,
`Rings`. O que faltava era **alguém chamar `tr`**: o laço que monta aquelas fileiras estava escrito
**três vezes** (`paint_texture` · `paint_watercolor_paper` · `paint_shape`), e as duas primeiras
traduziam. ⚠️ *Uma lei escrita em três sítios viaja para os dois de que alguém se lembrou* — e o
`s.label` é uma CHAVE, logo esquecer o `tr` não dá erro de compilação nem de tipo: dá um
identificador pintado no ecrã do artista.

⇒ **uma porta** (`number_field::params_de_padrao`), com os três a passarem por ela.

### ⛔⛔ Porque NENHUM dos 30 censos de texto a via

Eles perguntam *«este LITERAL vem da tabela?»* e varrem o **FONTE**. Ali não há literal nenhum: há
um campo. *Um censo de fonte é cego a um rótulo que o programa CALCULA.*

⇒ régua nova que lê o **ECRÃ**: `nenhum_rotulo_do_app_pinta_uma_chave`. ⭐ **O discriminador é a
própria tabela, e não a forma do texto** — um texto pintado que a tabela SABE traduzir é, por
construção, uma chave que alguém esqueceu. ⛔ Uma régua de forma (*«tem ponto e não tem espaço»*)
acusaria `0.5` e nomes de ficheiro; esta não tem falso positivo nenhum.

### ⛔⛔⛔ E a régua nova nasceu VERDE sobre o defeito vivo

Mutada a cura de volta, ela **não sangrou**: o estado de FÁBRICA do Painter tem a textura em
`None`, e ali aquelas fileiras **não são pintadas**. *Um painel cujas fileiras dependem de uma
escolha é medido vazio exactamente na parte que o artista usa* — a mesma cegueira que o
`o_inspector_armado` fechou no painel ao lado.

⇒ `o_painter_armado`: o instantâneo que o próprio painel usa antes de a shell publicar um, com as
**três** famílias de padrão armadas (silhueta · grão · papel), no `Wood` **da foto do dono** —
⚠️ um padrão sem params extra mediria menos rótulos e a fixtura não conteria o fenómeno. Com ela, a
mutação sangra com **48 chaves**, e as quatro primeiras são as da foto.

### ⚠️ E a fixtura revelou dívida que era invisível

As duas catracas de elisão do `painter_layers` foram de `3` para `4`: o **`View Plane`** só existe
com um padrão escolhido, logo a régua nunca o tinha visto. ⛔ **Não é regressão — é a POPULAÇÃO a
crescer**, e está escrito nas duas entradas. *Um número que sobe porque a régua passou a ver mais
não é o mesmo que um número que sobe porque alguém partiu algo*; continua a ser dívida, e a cura é
a que o dono deu em 20/09 (encurtar, com o balão a guardar a explicação).

### O portão

| passo | resultado |
|---|---|
| `cargo test -p ph2d-panel-registry-init` (âmbito do app) | **`106 / 106`** |
| `scripts/nextest-impacted.sh` | **`15 491 / 15 491`** · ⚠️ uma reprovada, ver abaixo |
| `scripts/censos-da-arvore-combinada.sh` | **`127 / 127`** |
| `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check` | zero · limpo |

⚠️⚠️ **PROMOÇÃO PEDIDA à lista de flakes de carga do `CLAUDE.md` §5.0** (a linha pede, o integrador
escreve): **`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`**
(`ph2d-tool-painter`, `tool/paint/measure_input_cost.rs`). Ele é **uma RAZÃO de dois relógios de
parede** (`down / copy`), reprovou no meio de um fan-out de `15 491`, e passa **`3` de `3` sozinho a
`load 24`–`25`**, com **zero linhas** desta linha naquela crate (comitadas ou não). *Dividir dois
relógios não deixa de ser um relógio por a razão ser adimensional.*

---

## §9-decies — ⭐⭐⭐ O Inspector: **`314 → 150 → 88`**, e a `D2` não é o que lhe dá ecrã

Continuação da §9-octies, depois do smoke aprovado do Painter.

### O que faltava à régua

A §9-octies ligou os **pintores canónicos** (`paint_segmented_group{,_adaptive}` ·
`paint_bitmask_grid32`) e o Inspector caiu de `314` para `150`. ⛔ Mas nem todo selector do app
passa por eles: com a sonda nova `diag_compostos_por_declarar` — *fileiras de botões encostados que
ninguém declarou* — sobravam **`307`** em **16** painéis, `105` só no Inspector.

⚠️ **E `307` não é a dívida:** uma fileira de `Add`+`Remove` são **dois comandos** e têm de contar
dois. *A sonda diz onde olhar, nunca o veredito* — quem decide é o que o pintor DESENHA.

### ⭐⭐ Duas linhas cobriram quase tudo

| onde | o que cobre |
|---|---|
| `sections::tween_editor::grupo` (helper local do Inspector) | **16 sítios** em quatro secções (`tween` · `path_follow` · `shake` · `shake_emitter`) |
| `widget::paint_tabs_with_hover` (canónico) | **toda fila de abas do app** — o `Center / Pivot / Custom` do Inspector entrava três vezes |

Mais duas fileiras escritas à mão (a direcção da animação, o *onde* da fábrica). ⇒ o Inspector foi
de `105` para **`43`** botões por declarar, e desses a maioria são comandos a sério.

⚠️ **A porta passou a `pub`, e a razão é medida:** os pintores canónicos moram na fundação, mas
**16 sítios do Inspector passam por um helper local**. *Uma porta que só a fundação pode chamar
deixa de fora exactamente os painéis que a régua existe para medir.*

### ⛔⛔⛔ E a conclusão que muda a wave seguinte

| | comandos | valores |
|---|---:|---:|
| Inspector, como a `D2` o media | `314` | `313` |
| **Inspector, medido** | **`88`** | **`367`** |

**O Inspector é `81 %` VALORES.** A `D2` do dono manda *comando do app → barra; comando do editor →
chip; **propriedade → fica***. ⇒ **triar o Inspector devolve pouco ecrã**, porque o que o enche são
propriedades, e propriedades ficam pela regra dele. ⚠️ *A wave que a régua velha mandava fazer era
trabalho sobre um número que não existia.*

⭐ E ele **deixou de liderar**: o topo é hoje o `tokens` (`110`), que **não usa composto nenhum** —
a dívida dele é real.

### A catraca

`a_carga_de_comandos_de_um_painel_so_encolhe` prende os sete números medidos, **nas duas
direcções**: subir quer dizer que um composto deixou de se declarar (a cura é declará-lo, ⛔ nunca
subir a linha) e descer quer dizer que há um número novo para escrever. *Cada declaração que alguém
apague devolve a mentira em silêncio, e o defeito é MUDO — um número maior lê-se como «este painel
tem mais dívida», que é uma frase plausível.*

**Mutação: 5 de 5 sangram** (o helper dos 16 sítios · as abas · a animação · a fábrica · o
CONTROLO do número obsoleto).

### ⏳ O que fica ABERTO, com o número

- `43` botões do Inspector por declarar, dos quais **`7` são da `line/components`** (`hud_fit` ·
  `hud_source`) e **`11`** são a grelha de regiões do 9-slice — um picker espacial, que é um
  composto de outra forma. Os restantes `~25` são comandos a sério.
- Os outros **15 painéis** com fileiras por declarar (`264` botões). ⚠️ O `tokens`, que lidera, não
  está entre eles.

---

### O portão desta wave

| passo | resultado |
|---|---|
| `cargo test -p ph2d-panel-registry-init` (âmbito do app) | **`106 / 106`** |
| `scripts/nextest-impacted.sh` | **`15 492 / 15 492`** · ⚠️ uma reprovada, `the_cost_of_a_player_is_linear_in_their_number` — **flake de carga já NOMEADA no §5.0**, `3` de `3` verde sozinha a `load 52`–`55` e **zero linhas** desta linha naquela crate |
| `scripts/censos-da-arvore-combinada.sh` | **`127 / 127`** |
| `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check` | zero · limpo |
| prova de mutação | **5 de 5 sangram** |

✅ **E o smoke do dono APROVOU a §9-nonies** (*«smoke OK»*): a secção `SHAPE ▸ Texture` mostra
`Contrast` · `Brightness` · `Turbulence` · `Rings`.

---

## §10 — O portão do fecho

| passo | resultado |
|---|---|
| `scripts/nextest-impacted.sh` (1.ª volta, só testes e docs) | **`15 470 / 15 470`** · `44,4 s` · **zero flakes** |
| `scripts/nextest-impacted.sh` (2.ª volta, **depois da lei FOUNDATIONAL**) | **`15 477 / 15 477`** · `13 225` saltados · `51,2 s` · **zero flakes** |
| `ph2d-editor-core --test it` | **`502 / 502`** |
| `cargo test -p ph2d-panel-registry-init` (âmbito do app: `--features panel-painter-layers,panel-flip,panel-flip-frames,panel-wet-tuning`) | **`104 / 104`** |
| `cargo clippy --all-targets -- -D warnings` (as três crates tocadas) | **zero** |
| `git rebase main` | **no-op**: `merge-base == main == 395da6a55` |
| `scripts/censos-da-arvore-combinada.sh` (as **duas** voltas) | **`127 / 127`** · *«controlo do filtro: 12 de 12 censos correram ✓»* |
| `scripts/collision-surface.sh` | §2 — zero contador partilhado, zero contrato, zero ADR, zero pacote externo, nenhum tecto de LOC |
| sobreposição de ficheiros com as outras linhas vivas | **ZERO** contra a `line/sculpt3d` (medido, §5) |

⚠️ **A corrida de `-p` sem as quatro features reprova 12 testes, e isso é o instrumento a funcionar**
— ver o §8.5. Quem re-correr o portão desta crate corre-a **com** elas.

| `rm -rf target/*/incremental` (item 7) | **`56 G → 27 G`** · o binário do smoke (`86,9 MB`) sobrevive |
| binário do smoke, 1.ª volta | `0,55 s` / **`0,23 s`** — *nada de produto tinha mudado* |
| binário do smoke, 2.ª volta (a lei foundational) | `30,98 s` / **`0,26 s`** — a shell recompilou, e é isso que diz que o produto mudou |

⭐ **O relógio da 1.ª build é um instrumento e não um detalhe:** `0,55 s` na 1.ª volta prova que
nada de produto tinha mudado desde o smoke aprovado; `30,98 s` na 2.ª prova o contrário. *Uma linha
que diz «mudei a fundação» e recompila em meio segundo está a mentir sobre uma das duas coisas.*

⚠️ **A 1.ª volta não tinha nada para o dono smokar** (régua, handoff e uma decisão dele — zero
linhas de produto). ⭐ **A 2.ª tem**, e o roteiro é de uma linha: *estreitar a janela do app e ver
as duas colunas encolherem com ela, em vez de comerem uma fatia cada vez maior*. Acima de `1366 px`
de largura nada muda — que é a metade que o gate `acima_da_referencia_a_lei_nao_toca_em_nada`
defende.

---

### §10-bis — O portão das voltas 3 e 4 (o piso de uma escolha, e o dobro dele)

⚠️ **Corrido quatro vezes**: para o piso medido (`84`), para o dobro (`168`), para o `+25 %`
(`210`) e para o readout da §9-septies. Os números abaixo são os da **última**.

| passo | resultado |
|---|---|
| `scripts/nextest-impacted.sh` | **`15 486 / 15 486`** · `62,6 s` · zero flakes |
| `cargo test -p ph2d-panel-registry-init` (com as quatro features) | verde |
| `cargo clippy -p ph2d-editor-core -p ph2d-host-desktop --all-targets -- -D warnings` | **zero** |
| `cargo fmt --check` | limpo |
| `git merge-base HEAD main` | **no-op**: `merge-base == main == 395da6a55` |
| `scripts/censos-da-arvore-combinada.sh` | **`127 / 127`** · *«controlo do filtro: 12 de 12 censos correram ✓»* |
| prova de mutação | **7 sangram + 2 CERCA + 1 NOMEADA**, de 10 |
| mudança de PRODUTO, as três voltas somadas | **três linhas** (a constante do recurso, a do piso, e a lei a ler o store) |

⚠️ **A reprovada da última volta é `the_cost_of_a_player_is_linear_in_their_number`
(`ph2d-physics-ecs`), membro CONFIRMADO da família de flakes de fan-out** — ela está nomeada no
`CLAUDE.md` §5.0 e foi esta linha que a pediu para lá em 2026-09-07. Assinatura completa:
**`3` de `3` verde sozinha a `load 52`–`63`**, que é carga **MAIOR** do que aquela em que
reprovou, e **zero linhas** do diff desta linha naquela crate. ⇒ *o discriminador é o FAN-OUT e
não o relógio*, que é exactamente o que aquela entrada diz.

⭐⭐ **Duas mutações deixaram de poder sangrar porque passaram a ser ERRO DE COMPILAÇÃO**, e isso é
mais forte: reverter o piso da escolha para o de fábrica devolve `error[E0080]` com a frase do
report lá dentro. ⚠️ **O arnês teve de aprender a lê-lo** — a 1.ª corrida classificou-as como
*«não compila ⇒ abortado»*, que é a regra certa **para toda outra mutação** e a errada para uma
cerca; hoje ele exige o `E0080` **e** a mensagem dela, senão um erro de sintaxe qualquer contava
como lei defendida.

⭐ **E foi preciso uma mutação que PASSA a cerca** (`piso = PANEL_MIN_W_PX − 0,5`) para provar que
o gate do report continua vivo: sem ela, a cerca tornava-o impossível de matar, e *um gate que
nada consegue reprovar deixou de afirmar*.

⚠️ **Duas reprovações de ÁRVORE, as duas premissas mortas de gates que esta linha escreveu ontem**
(o arrasto fantasma e a involução da coluna) — reescritas, com a morte à vista no diff, e a segunda
com a fixtura **derivada do piso** para a próxima descida não a partir outra vez.

---

## §11 — O que esta linha recomenda a quem a integrar

1. **Correr o `diag_onde_cai_a_pista_do_pente` da `line/sculpt3d` DEPOIS da fusão** e reescrever com
   a saída dele as **duas** tabelas vivas do §5 (o `CLAUDE.md` §5 e o doc-comment do
   `scenes_pente.rs`). ⛔ A do handoff `…_sculpt3d_2026-09-17.md` §94 **fica como está**.
2. **Promover nada à lista de flakes do §5.0** — esta rodada não produziu nenhuma (`15 470` verdes
   à primeira).
3. A linha do §5 está no §9, pronta a colar.
