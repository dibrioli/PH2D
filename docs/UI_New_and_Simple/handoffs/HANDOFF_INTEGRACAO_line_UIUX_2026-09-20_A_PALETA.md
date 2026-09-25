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
| — | §9-bis .. §9-quaterdecies | ⚠️ **a partir daqui esta tabela não cresce:** cada wave seguinte da linha tem a secção `§9-*` dela, na ordem em que foi escrita. `git log --oneline ac39f40ad..HEAD` dá os commits |

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
> volta — [handoff §9-bis..§9-quater, arquivado](docs/archive/uiux-paleta-2026-09-24/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20_A_PALETA.md)):
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


## §9-arquivo — As waves `§9-bis` a `§9-vicies-sexies` foram ARQUIVADAS (2026-09-24)

O handoff chegou a **`199 KB`**, o dobro do joelho medido (`80`–`110 KB`: acima dele o `Read` deixa de
caber e o acesso vira raspagem). As `26` waves do meio — da largura da coluna (§9-bis) à coluna do
painel (§9-vicies-sexies) — estão **verbatim** no [arquivo](../../archive/uiux-paleta-2026-09-24/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20_A_PALETA.md), cortadas por
`scripts/doc-split.py` com a remontagem provada por `sha256`. Ficam vivos o que o INTEGRADOR lê
(§1–§9, §10, §11) e as waves desde a §9-vicies-septies. ⚠️ Uma referência a `§9-ter`,
`§9-septdecies` etc. nas secções vivas resolve-se **no arquivo**, com o mesmo título.

### ⛔ Recusas MEDIDAS que foram para o arquivo — não as reconstrua

| recusa | o mecanismo MEDIDO | onde |
|---|---|---|
| A fracção da coluna como tecto sobre **qualquer** largura (`dock_w_ceiling` sem `min(1,0)`) | Construída com porta, gate e `4/4` mutações, e **REVERTIDA**: um gate pré-existente (`the_width_grows_with_x_on_the_left…`) exige que o arrasto CRESÇA a coluna — o tecto comia o gesto do dono | §9-ter |
| *«Três chevrons mortos»* no `paint_core_sections` | Hipótese **refutada pelo fonte**: `Name` e `Visibility` são fileiras SEM cabeçalho, não há promessa quebrada | §9-duodecies.8 |
| Um gate NOVO para os selectores de cor | **Apagado**: o `as_formas_de_um_selector_de_cor_so_encolhem` já media `109` selectores e prescrevia a porta — era uma segunda resposta à mesma pergunta, e pior | §9-septdecies.2 |
| A régua da amostra de cor pela **borda direita** / *«não mais estreita que o vizinho mais largo»* | **Refutadas**: a 1.ª fica verde sobre o defeito; a 2.ª reprova os oito, porque o vizinho mais largo é um botão de linha inteira — *uma estatística sobre população heterogénea mede a variedade* | §9-septdecies.3 |
| O menu suspenso ao lado do nome, no lugar da PALETA | **Recusado pelo dono** (2026-09-23, *«o formato atual»*); registado no doc da `paint_choice_row` | §9-vicies-ter.3 |
| A barra de cor no painel TOKENS (`paint_color_row`, e a etiqueta à direita) | **Recusada duas vezes por medição**: `108` e depois `29` nomes comidos no dock mínimo contra `5` — numa lista cujo conteúdo são os nomes, qualquer valor à direita come o nome | §9-vicies-quater.1 |
| Encolher o recuo da seta do chip · subir a catraca dos cortes | **Revertidas**: o `None` continuou cortado (`26 px` de orçamento medido, não `38`); subir a catraca é proibido — a cura foi a grelha do vetor passar à `wrapped_cells_for` | §9-vicies-quinquies |

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

## §9-vicies-septies — ⭐⭐⭐ A SECÇÃO *INSPECT* DO GRID SNAP PASSA PELAS PORTAS, e o ARRASTO de um painel deixa de pôr a coluna no sítio errado

⛔⛔ **Report do dono, 2026-09-23, foto com duas setas (o nome `Probe B` e a caixa da sonda):** *«painel grid fora
do padrão»*. A secção pintava o nome À ESQUERDA por `paint_text`, o valor por `paint_text` com a cor de rótulo, e
as duas linhas das sondas montavam `NumberInput` à mão com a metade da coluna calculada ali — três respostas
próprias a perguntas que a `property_row` responde para o app inteiro.

### .1 — A cura ([`inspect.rs`](../../../crates/ph2d-editor-core/src/grid_snap/inspect.rs))

- **UMA `Seccao::medida(ts, 2, nomes)`** para as linhas de leitura E as das sondas (`campos = 2` porque a sonda
  tem X e Y). Os nomes pela `paint_label_row` (à direita, `TypeToken::Sm`, `Text2`); o valor de leitura na coluna
  do controlo; a continuação hexagonal (sem nome) usa a MESMA geometria por `colunas_da_linha`.
- **As sondas pela `paint_fields_row`** — o valor sai da LOJA, que o painel já re-semeia do estado na unidade
  activa antes de pintar (`sync_meter_inputs_to_display_unit_impl`, que já cobria os quatro ids). ⇒ a secção
  deixou de precisar da unidade: `display_unit`/`pixels_per_meter` saíram da assinatura do `inspect::paint`, do
  `paint_inspect_section` e do `paint_body`. ⚠️ A porta passa sempre o `buffer` ao pintor (a mão só o passava com
  foco) — é o comportamento de toda caixa do app.
- `LABEL_FONT_SIZE = 12.0` privado **morreu**; `FONTE_DO_INSPECT` do gate passou a `TypeToken::Sm.px()`.

### .2 — A régua e as mutações ([`um_orcamento_de_texto_e_a_largura_de_um_espaco.rs`](../../../crates/ph2d-editor-core/tests/it/um_orcamento_de_texto_e_a_largura_de_um_espaco.rs))

⚠️ **A porta do nome pergunta DUAS vezes** (a coluna, para o balão; e a largura MEDIDA do que coube, porque o
nome encosta à direita) ⇒ a coluna de um rótulo é a MAIOR largura em que ele foi perguntado, e o gate afirma que
ela é uma só em toda a secção, que as duas sondas passam DUAS vezes pela porta (leitura + campos), e que todo nome
que coube foi perguntado também abaixo da coluna (= encostado à direita).

⛔⛔ **A 1.ª redacção media a `252` só, e DUAS mutações sobreviveram** — `252` é ponto NEUTRO: ali a secção medida
com um campo, com dois, e a que só declara os campos dão TODAS `118`. Medido pela porta:

| largura | medida, 1 campo | medida, 2 campos | só os campos (2) |
|---|---|---|---|
| `180` | `86` | `86` | **`82`** |
| `252` | `118` | `118` | `118` |
| `300` | **`142`** | `131` | **`142`** |

⇒ o gate corre `[180, 252, 300]`. **Mutações 3 de 3:** sondas com `Seccao::apenas_campos(2)` · nome à esquerda
por `paint_text` · secção medida com `1` campo. (`150`/`400` NÃO separam nada — foram medidos e trocados.)

### .3 — ⛔⛔⛔ O defeito que o gate do Grid Snap achou era MEU, da §9-vicies-sexies

O `a_seccao_poe_todas_as_caixas_na_mesma_coluna` (grid-snap) reprovou no **CONTROLO** depois da cura: estreitado o
painel em `84 px`, o quadro do redimensionamento punha a caixa em `620` e o painel convergia em `653` no quadro
seguinte. **A causa era a `base_a_partir_de`:** ela lia TODA largura nova da primeira linha como *mudança de
espécie* (um cartão a vir primeiro) e presumia a linha recuada POR IGUAL dentro da base antiga — estreitado o
painel, a primeira linha lia-se «recuada `42` de cada lado», todas as linhas passavam a cartões e o tecto do
controlo deixava de contar. ⚠️ **Num ARRASTO cada quadro tem largura nova ⇒ a coluna ficava errada durante o gesto
INTEIRO e só acertava ao largar.** Ele só apareceu agora porque a secção *Inspect* passou a declarar uma linha de
DUAS caixas, cujo tecto é o que a leitura errada deitava fora.

**A cura** ([`coluna_do_painel.rs`](../../../crates/ph2d-editor-core/src/widget/property_box/coluna_do_painel.rs)):
a `Base` guarda também o rect ABSOLUTO; largura nova com a primeira linha **simétrica dentro** da base antiga ⇒
mudança de espécie (base antiga); largura nova e NÃO simétrica ⇒ o painel mudou de largura (uma borda fica, a outra
anda) e a primeira linha leva o recuo que tinha. ⛔ **E o gate irmão `arrastar_o_dock_acerta_no_mesmo_quadro`
afirmava só que as linhas ALINHAVAM entre si** — alinhadas no sítio errado passavam. Hoje ele compara o quadro do
arrasto, linha a linha, com o painel CONVERGIDO na geometria nova, em três gestos (dock pela esquerda, estreitar e
alargar pela direita). **Mutações 3 de 3:** toda largura nova como espécie (o defeito) · nunca espécie · sem o teste
de simetria.

**Portão:** `nextest-impacted` `17 689/17 690` — o único ✗ é
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (`ph2d-tool-painter`), gate de RAZÃO de custo,
**3 de 3 verde sozinho a `load 23`**, zero linhas de diff naquela crate ⇒ família de flakes de fan-out (§11.2).
clippy `-D warnings` zero · `cargo fmt --all --check` · `censos-da-arvore-combinada.sh` `127/127`.


## §9-vicies-octies — ⭐⭐ AS TRÊS SECÇÕES DO INSPECTOR QUE PINTAVAM O NOME À MÃO

Depois do smoke aprovado da §9-vicies-septies (*«smoke OK. Siga»*), a mesma pergunta — *quem ainda
pinta o nome por `paint_text` numa coluna própria?* — foi feita ao Inspector por censo textual. Três
secções, todas com uma coluna `N alturas de letra` e tecto em fracção da linha, o nome À ESQUERDA:

| secção | linhas | a coluna que tinha |
|---|---|---|
| [`wheel.rs`](../../../crates/ph2d-panel-inspector/src/sections/wheel.rs) | `Mounted On` · `Gear` · `Rope` | a lista das três, tecto `0,42` |
| [`joint_pair_rows.rs`](../../../crates/ph2d-panel-inspector/src/sections/joint_pair_rows.rs) | `Body A` · `Body B` | `3,6 × fonte`, tecto `0,4` |
| [`player_live.rs`](../../../crates/ph2d-panel-inspector/src/sections/player_live.rs) (era `player.rs`) | a leitura ao vivo | `5 × fonte`, tecto `0,42` — com o comentário *«a mesma coluna das rows»*, falso |

⇒ as três pela `rows::property_label_row` com uma `Seccao::medida` sobre os nomes que a secção PODE
pintar (a do jogador mede os quatro, senão a coluna mudava no quadro em que a corrida começa), o valor
e os ícones dentro da `linha.control`. ⚠️ O `player.rs` foi a `607` contra o tecto de `600` e partiu-se
por responsabilidade: a leitura ao vivo, a única parte que não se edita, é o irmão `player_live.rs`.

**O gate novo** — [`nenhuma_linha_pinta_o_nome_numa_coluna_propria`](../../../crates/ph2d-panel-inspector/tests/it/nenhuma_linha_pinta_o_nome_numa_coluna_propria.rs):
um `paint_text` cujo bloco nomeia uma coluna de nome (`label_w` · `label_col` · `nome_w` · …), com
pisos medidos (`70` ficheiros, `68` chamadas), obsolescência, e a única excepção nomeada (`timers.rs`
é uma LISTA). ⚠️ **Porque é textual:** a régua do alinhamento só vê quem passa pela porta — uma linha
pintada à mão é invisível a ela por construção, que é o defeito. **Mutações 3 de 3.**

**O preço, atribuído por A/B ficheiro a ficheiro** — a catraca do degrau estreito `83 → 85` / `79 → 81`,
e são VALORES, não nomes: o nome do corpo em que a roldana monta (`Pivot` na fixtura, `28 px` ao lado
de dois ícones) e a velocidade ao vivo do jogador (`1.00, 0.00 m/s`). A junta não custa nenhum. A
primeira leitura foi que o `Pivot` era o segmento da linha `Sort Point` — instrumentada a decisão
«ao lado / por baixo», ela deu o MESMO resultado nos dois estados; o contexto da varredura (os oito
textos antes dele) é que o nomeou.

**Portão:** `nextest-impacted` `17 691/17 693` — os dois ✗ são gates de RAZÃO de relógio sob `load 108`
(`the_cost_of_sampling_a_path_is_flat_in_its_anchors`, já na lista, e o do painter da §9-vicies-septies),
**3 de 3 verdes sozinhos a `load 60`** · clippy `-D warnings` zero · `fmt --check` · censos `12/12`.


## §9-vicies-novies — ⭐⭐ O BOTÃO DE ACÇÃO vai para a coluna do VALOR — quando o rótulo lá cabe

⛔⛔ **Report do dono, 2026-09-24, foto do cartão da junta com uma seta** (depois de aprovar a
§9-vicies-octies): *«esses botões que atravessam de lado a lado talvez fiquem melhor na coluna do lado
direito»*. O `Swap A / B`, o `Copy Properties` e o `Delete Joint` eram `Rect::new(x, y, w, h)`; o censo
achou a mesma forma em **`29`** botões do Inspector (e `~25` noutros painéis — §9-vicies-novies.3).

### .1 — A porta ([`property_row/botao.rs`](../../../crates/ph2d-editor-core/src/property_row/botao.rs))

`caixa_do_botao(ts, x, w, y, h, rótulo) -> Rect`: a coluna do valor (`property_row_columns`, o MESMO
pedido que a linha de escolha faz ⇒ não mexe na coluna do painel) **se o rótulo lá cabe**; senão a linha
inteira, como antes — a lei da `paint_choice_row` (ao lado se cabe, paleta se não).

⛔⛔ **Porque a condição, medido antes de a escrever:** com TODOS os botões na coluna, à largura de
fábrica `5` passavam a sair cortados (`Reimport at current px/m` · `Draw Joint on Canvas` · `Rig 2 Parts
from Hierarchy` · `Remove Physics Body` · `Fit Crouch to Collider`), e no degrau estreito **`18`** — o
`Swap A / B` e o `Delete Joint` da própria foto incluídos. *Um botão cortado lê-se pior do que um nome
cortado: o nome tem o controlo ao lado a explicá-lo; o botão É a explicação.* Com a condição: **zero**
cortes novos, as catracas das elisões intocadas.

⚠️ **A pergunta «cabe?» é a do pintor** (largura em `MEDIUM` na `Button::label_font_px`, contra o
`label_budget` da caixa) e mede-se SEM o `text_elide::coube`, que REGISTA no censo das elisões — uma
pergunta de disposição lida como pintura poria lá um rótulo que ali não foi pintado. ⚠️ **A altura é do
chamador**: `30` contra `22` continua a decisão aberta da §9-octodecies.6; a porta decide ONDE.

### .2 — Gates e mutações

Quatro unitários na porta (curto vai · longo atravessa · a fronteira por VARREDURA fina da largura, com o
controlo de que a varredura a atravessa · a pergunta não entra no censo) + o censo
[`nenhum_botao_atravessa_a_linha_a_mao`](../../../crates/ph2d-panel-inspector/tests/it/nenhum_botao_atravessa_a_linha_a_mao.rs)
(piso: `29` pela porta, `70` ficheiros). **Mutações 5 de 5** — ⚠️ a da fronteira (`< orçamento − 1`)
**sobreviveu à 1.ª redacção**, que media UM rótulo com vários píxeis de folga: *uma fronteira mede-se onde
a folga passa por zero*.

### .3 — ⏳ O que fica

- **Os pares `+ Add … | x Remove …`** (`8` secções: gatilhos, acções, estados, temporizadores, tweens…)
  repartem a linha pelo `segment_rects_for` e ficam à largura inteira: dois rótulos desses nunca cabem na
  coluna (já cortam à largura inteira no degrau estreito) — a porta mandá-los-ia atravessar na mesma.
- **Os outros painéis** (`bgremoval` `9`, `padding` `2`, `physics` `2`, `sculpt3d` `2`, `painter_layers`
  `4`, `grid_snap` `2`, `upscale`, `equalize_sizes`, `vector` `2`) têm a mesma forma, e são sobretudo o
  botão de APLICAR de uma ferramenta — **pergunta devolvida ao dono** antes de os mexer.

**Portão:** `nextest-impacted` **`17 699/17 699`** · clippy `-D warnings` zero (os `24` `&tr(…)` de
empréstimo inútil que a conversão escreveu, curados) · `fmt --check` · censos `12/12`.

## §9-tricies — ⭐⭐ DEPOIS DE UM BOTÃO VEM O VÃO DE TODA LINHA

⛔⛔ **Report do dono, 2026-09-24, foto com duas setas** (o cartão da junta, depois da §9-vicies-novies):
*«sem espaçamento nenhum. corrija»* — o `Swap A / B` encostado ao `Collide`, e o `Copy Properties`
encostado ao `Delete Joint`.

⭐ **A causa não era a porta da coluna, era o AVANÇO:** `11` dos `29` sítios que pintam um botão faziam
`yy += h` (a altura do botão e mais nada) enquanto as linhas de propriedade avançam
`h + control_gap_px()`. Enquanto o botão atravessava a linha inteira o encosto lia-se como «um bloco de
botões»; com o botão na coluna do valor, ao lado de um NOME, ele lê-se como o que é — duas linhas sem vão.
⚠️ *A mudança anterior não criou o defeito: tornou-o legível.*

⇒ porta irmã [`abaixo_do_botao(caixa)`](../../../crates/ph2d-editor-core/src/property_row/botao.rs)
`= caixa.y + caixa.h + control_gap_px()`, **nos `29` sítios** (não só nos `11`). ⚠️ **O diff contou
SEIS formas escritas à mão para o mesmo avanço** (`yy += h` ×7 · `h + control_gap_px()` ×11 ·
`row_pitch_px()` ×3 · `ALTURA_DE_BOTAO` sem vão ×3 · `ROW_H_PX` ×1 · `h + Spacing::Sm` ×1) e **três**
fins de secção que somavam a almofada directamente à base do botão — *seis maneiras de escrever um
avanço divergem no dia em que o token mudar, e já divergiam*. O fim de secção passa a
`fold.finish(…, abaixo_do_botao(btn) + SECTION_BOTTOM_PAD_PX)`, igual às secções que acabam numa linha
de propriedade (cujo `yy` já traz o vão).

**Gates:** o unitário `depois_do_botao_vem_o_vao_de_toda_linha` (com a guarda de que o token não é zero —
senão a igualdade seria trivial) e o censo `depois_de_um_botao_o_vao_e_o_da_porta` no mesmo ficheiro do
censo da coluna: **por ficheiro**, cada `caixa_do_botao` tem de ter um `abaixo_do_botao`, piso `25`
(medido `29`). **Mutações 2 de 2:** repor `yy += h` num sítio (sangra o censo) · tirar o vão da porta
(sangra o unitário).

**Portão:** `nextest-impacted` **`17 701/17 701`** · clippy `-D warnings` zero · `fmt --check` · censos
da árvore combinada `12/12`.


## §9-untricies — ⭐⭐ A MESMA PORTA NOS OUTROS PAINÉIS, e o censo que a mantém

Ordem do dono (2026-09-24, depois do smoke do §9-tricies): *«smoke ok. siga»* — a resposta à pergunta
de alargar a regra do botão (coluna do valor quando o rótulo cabe, linha inteira quando não) aos outros
painéis.

**O que passou pela porta** ([`caixa_do_botao`](../../../crates/ph2d-editor-core/src/property_row/botao.rs)),
`19` botões em `11` ficheiros: remover-fundo (`8` — ilhas, acrescentar/limpar área, conta-gotas,
detectar sujeito, proteger, mostrar/limpar máscara) · igualar-tamanhos (o `paint_toggle_button`, os
três chamadores) · padding (o pivô) · camadas do pintor (fonte do clone, camadas do documento, os dois
de escolher no canvas da simetria) · vector (pré-visualização do morph, dos estados, *+ Add* dos sinais)
· grelha (o *Reseed*) · escultura (os helpers `toggle`/`command`, ~35 chamadores).

**⭐ Na escultura os helpers partiram-se em duas espécies**: `toggle`/`command` recebem a LINHA e pedem
a caixa à porta (e devolvem `abaixo_do_botao`); `toggle_na_celula`/`command_na_celula` recebem uma
célula que o chamador já repartiu (os três eixos da simetria, o *Isolate/Merge*, o `row_of_two`). ⛔ Uma
porta só com um modo decidido por comparação de larguras mediria a FATIA como se fosse a linha. Os três
`+ gap` (`Spacing::Xs`) escritos à mão depois de um botão saíram — o vão passou a ser o da porta.

**⛔ O que NÃO passou, cada um nomeado na lista do censo com o porquê:**

| sítio | porquê |
|---|---|
| os `5` Reset das ferramentas de imagem | RODAPÉ: ficam em cima do par `Cancel \| Apply`, que atravessa a linha; na coluna o destrutivo ficaria por cima do Apply |
| *Apply Mask* do pintor | CTA de fecho da secção (accent) |
| *Snap* da grelha | CTA-herói, altura própria |
| menu de propriedades da timeline | é um MENU flutuante, não linhas de propriedade |
| `arrow_button` do vector | CÉLULA: o quadrado `<`/`>`, os parâmetros só se chamam `x`/`w` |

**⛔⛔ A física foi convertida e REVERTIDA pela medição.** O `dentro_de_um_troco_o_valor_arranca_numa_coluna_so`
reprovou: a coluna que o painel de física arranca é a da grelha das camadas (`x = 26`), e o `Enabled` do
sono na coluna da porta (`x = 136`) abria uma segunda. ⚠️ Os sliders dele usam o `property_label_col_w`,
mas registam o acerto na LINHA inteira, logo o censo das colunas não os vê — alinhar a física pede o
painel inteiro a registar na coluna do valor, que é outra wave. Os botões dela já tinham o vão
(`+ row_gap`, que é o `control_gap_px`).

**⬆️ A altura de abertura da escultura subiu `2 021 → 2 051`, com a conta fechada** (catraca
`a_altura_de_abertura_de_um_painel_so_encolhe`): `11` botões de linha inteira avançavam só a altura e
passam a ter o vão (`+3` cada), `3` somavam o `Xs` à mão (`4 → 3`, `−1` cada) ⇒ `33 − 3 = 30`. ⛔ Nenhuma
secção nasceu aberta; a subida é a ordem do dono do §9-tricies, e o comentário na catraca di-lo.

**O censo novo** — `an_action_button_asks_the_door_where_it_goes` (`ph2d-editor-core/tests/it/`):
procura `let v = Rect::new(<início>, _, <largura>, _)` seguido de `paint_button` com `v`, para os pares
que os painéis usam para a linha inteira (`inner_x`/`inner_w`, `layout.inner_x`, `self.inner_x`,
`x`/`w`, `x`/`content_w`, `list.x`/`list.w`), em todos os `ph2d-panel-*` menos o Inspector (que tem o
dele). Três metades: nenhum sítio novo · nenhuma excepção obsoleta · pisos de população (`≥ 250`
ficheiros, `≥ 16` chamadas à porta; medido `19`). ⚠️ A 1.ª redacção lia uma vírgula final de chamada
partida como um 5.º argumento e o controlo do extractor apanhou-a; e contava só o caminho longo
(`property_row::caixa_do_botao(`), que o remover-fundo deixou de escrever quando o `import` curou o
tecto de LOC dele (`617 → 562`).

**Prova:** mutação — repor o `Rect::new(inner_x, y, inner_w, row_h)` no *Separate Islands* reprova o
censo pelo nome, e a árvore restaurada volta a verde. Portão: `nextest-impacted` **17 703/17 703** ·
clippy `-D warnings` zero nas `9` crates tocadas · `fmt` · censos da árvore combinada **127/127**.

## §9-duotricies — ⭐⭐ A FÍSICA TAMBÉM: a matriz das camadas é declarada, não o botão

Ordem do dono (2026-09-24, *«smoke OK. Siga»*, depois da oferta de alinhar a física). ⛔ **O §9-untricies
revertia a física e a leitura dele estava pela metade:** o segundo arranque do painel não era o botão, era
a **matriz das camadas de colisão** (a 1.ª célula de cada fileira em `x = 26`). Os sliders da física já
usavam a coluna da porta (`property_label_col_w` → `coluna_do_painel::no_painel`, a MESMA função da
`property_row_columns`); só não apareciam no censo das colunas porque registam o acerto na LINHA inteira.
⇒ o botão na coluna da porta está alinhado com os pares do painel, e o que é de outra espécie é a matriz
— uma MATRIZ, não um par nome/valor, que a `300 px` não cabe na coluna do valor (8 células).

**A cura:** os `toggle`/`command` da física pedem a caixa à porta (2 chamadas, `21` no censo fora do
Inspector), a excepção sai da `LINHA_INTEIRA_OK`, e a física entra em `COLUNAS_DECLARADAS_POR_PAINEL`
com `2` e o porquê — o mesmo lugar e a mesma forma das abas da timeline. ⚠️ A lista é uma IGUALDADE:
se a matriz um dia entrar na coluna, a entrada reprova e tem de sair. A altura de abertura da física
**não mexe** (os botões dela já tinham o vão, `+ row_gap` = `control_gap_px`).

**Portão:** `nextest-impacted` **17 703/17 703** · clippy `-D warnings` zero · `fmt` · censos da árvore
combinada **127/127**. A prova de que a declaração é necessária é a corrida do §9-untricies: sem ela o
`dentro_de_um_troco_o_valor_arranca_numa_coluna_so` reprova com `[(26.0, 8, "physics.layer_0_0"),
(136.0, 3, "physics.sleep_spin")]`.

## §9-tritricies — ⭐⭐ O BOTÃO TEM A ALTURA DE UM CAMPO — e a porta deixa de receber altura

Decisão do dono (2026-09-24), à pergunta aberta desde o §9-octodecies.6 (arquivado), posta com o número:
*«Igualar à altura dos campos»*. Os botões do Inspector mediam `30` (`ALTURA_DE_BOTAO`) e os campos `22`
(`ROW_H_PX`); os dos outros painéis **já** eram `22`.

**A cura é estrutural, não um número trocado:**

- [`caixa_do_botao`](../../../crates/ph2d-editor-core/src/property_row/botao.rs) **perde o parâmetro
  `h`** e devolve sempre `ROW_H_PX`. ⛔ Enquanto a porta recebia a altura, a mesma lista tinha botões a
  `30` e a `22` conforme a secção, e a lei do dono dependeria de cada chamador se lembrar dela.
  `52` chamadas em `30` ficheiros reescritas por parser com contagem (as `5` ocorrências dentro de
  strings e comentários — os censos que procuram a porta — ficaram intactas).
- A `ALTURA_DE_BOTAO` foi **APAGADA**, não posta a `22`: dois nomes para o mesmo número são duas respostas
  à espera de divergir. As `22` utilizações restantes (pares `+ Add | x Remove`, avanços) passaram à
  `ALTURA_DE_CAMPO`, e o censo `nenhuma_seccao_declara_a_propria_altura` aponta o `BTN_H` para ela.
- ⛔ **Um `30` escapava ao censo**: `let reimport_h = 30.0_f32` no `render_source.rs` (um `let`, não um
  `const` — a régua só lê `const`). Morreu com o parâmetro.
- Sete ligações de altura ficaram sem uso e saíram (entre elas o `h` de dois pintores do jogador, que só
  o passavam à porta).

**Medido pela sonda `diag_o_ritmo_de_cada_seccao_do_inspector`:** nenhuma linha de `30` nas secções; o
único `30` que resta é o `+` do cabeçalho (`insp_add_component`), que é CROMO do painel e não linha da
lista. As alturas de abertura não se movem (as secções com botões nascem dobradas).

**Prova:** mutação — `ROW_H_PX + 8` na porta reprova `um_rotulo_curto_vai_para_a_coluna_do_valor`
(`(r.y, r.h) == (40, ROW_H_PX)`). Portão: `nextest-impacted` **17 703/17 703** · clippy `-D warnings`
zero nas `11` crates tocadas · `fmt` · censos da árvore combinada **127/127**.

## §9-quatertricies — ⭐⭐ O PAINEL VECTOR: `24 → 8` comandos, e a maior parte da dívida era do INSTRUMENTO

Ordem do dono (2026-09-24), escolhida entre três: *«Arrumar o painel Vector»*.

**Passo zero: a lista, não o número.** A sonda nova `diag_os_comandos_do_vector` (`#[ignore]`, no
`ph2d-panel-registry-init`) pinta o painel à largura do dono e lista cada botão com o NOME (colhido dos
literais com forma de id no `ph2d-panel-vector`, `ph2d-tool-vector` e `ph2d-editor-core`, porque o id é
um hash) e se ele está num grupo declarado. ⚠️ O `45` da tabela do §7.3 era de 20/09; a catraca
`a_carga_de_comandos_de_um_painel_so_encolhe` lia `24` distintos hoje, e a lista partiu-os em DUAS
espécies:

| espécie | quantos | o que era | cura |
|---|---:|---|---|
| peças da `button_grid` (os `15` modos da ferramenta + o *Pick Shapes*) | `16` | uma ESCOLHA (*uma de N*, com a acesa a dizer qual) que **não se declarava composto** | `composto::grupo` dentro da `button_grid` — os três chamadores dela são escolhas |
| acções (Blend ×4, Morph, *Both* dos marcadores, fechar) | `8` | comandos a sério, a ATRAVESSAR a linha | a coluna do valor, pela porta |

**⛔ A causa da segunda: o ajudante partilhado vive no NÚCLEO.** O `RowCtx::action_button_kind`
(`ph2d-editor-core/src/panel/rows.rs`) pinta os `52` botões de acção do Vector e os do Esqueleto, e o
censo `an_action_button_asks_the_door_where_it_goes` varria só `ph2d-panel-*` — logo a regra aprovada
no Inspector e alargada no §9-untricies nunca lhes chegou. ⇒ o ajudante pede a caixa à porta e avança
pela `abaixo_do_botao`; o censo passa a varrer aquele ficheiro também (mutação: repor o `Rect::new` a
toda a largura no ajudante reprova-o).

**⛔ O `Accent` ATRAVESSA a linha, de propósito:** é o *commit* (o *Apply* da pilha de efeitos, o da
simetria) — o mesmo lugar do *Apply Mask* do Painter e do par `Cancel | Apply` das ferramentas de
imagem.

**⚠️ Declarar a grelha acordou o `nenhuma_escolha_do_app_e_montada_a_mao`:** uma escolha a toda a
largura sem nome ao lado. ⇒ **excepção NOMEADA** em `FORA` (`vector.mode.select`): a grelha escolhe a
FERRAMENTA na mão, não uma propriedade do objecto, e o título da secção (`TOOL`) já a nomeia; a FORMA
dela espera a decisão do dono de partir o `DrawMode` nos dois eixos (§5 do `CLAUDE.md`). A lista tem
censo de obsolescência, logo a excepção sai sozinha quando a grelha mudar.

**Números:** catraca `vector` `24 → 8` (a metade *«desceu — escreva o número»*). ⚠️ O `tokens` do §7.3
**não é dívida**: `110` botões, `4` comandos distintos — a própria catraca o diz; a escolha do dono
pelo Vector estava certa.

**Prova:** mutação — tirar o `composto::grupo` da grelha põe a catraca em `24 contra 8`; repor o rect a
toda a largura no ajudante reprova o censo dos botões. Portão: `nextest-impacted` **17 702/17 703**, a
única ✗ a ser `a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` (`ph2d-app-flip`,
família `…precisao::orcamento` da lista de flakes do §5.0): **3/3 verde sozinha a `load 38–42`**, zero
linhas de diff na crate · clippy `-D warnings` zero · `fmt` · censos da árvore combinada **127/127**.

## §9-quinquetricies — ⭐⭐ O PAINEL FÍSICA: `49 → 4` comandos, e NENHUM era dívida de produto

Ordem do dono (2026-09-24, *«siga»* depois do smoke do Vector). A sonda do §9-quatertricies passou a
servir qualquer painel (`PH2D_PAINEL=<id>`, nomes colhidos de TODAS as crates, porque o id de um
painel pode ser fabricado noutra) e, sobre a Física, partiu os `49` em três:

| espécie | quantos | o que era | cura |
|---|---:|---|---|
| células da matriz de camadas | `36` | um VALOR (o mapa de colisão; cada célula liga um par) | `composto::grupo` no `matrix::paint`, como a `bitmask_grid32` do Inspector |
| cabeçalhos de secção | `8` | registados como `Button` no `populate` | secções dobráveis da casa (`mark_collapsible_section`) |
| comandos | `4` | fechar · *Reset* · e dois interruptores pintados como botão (*Enabled* do sono, *Show Colliders*) | ficam |

⭐⭐ **Os cabeçalhos eram mais do que contagem: o painel tinha a SUA cópia da dobra.** O cânone da UI
é o cabeçalho sem estado de botão, marcado dobrável, e o `apply_click` do despacho dobra-o ANTES de
emitir o `Click`. ⇒ o braço do `event.rs` fica só a CONSUMIR o clique — ⛔ dobrar lá outra vez
desfaria o gesto (é a 1.ª mutação abaixo).

⛔⛔ **E o gate da dobra media a metade que o despacho salta:** o
`folding_a_section_never_touches_the_world` empurrava um `Click` à mão para o painel, o que só provava
a cópia local. Reescrito para pintar e clicar no centro do cabeçalho pelo despacho real
(`click_at`) — a mesma forma do vizinho `every_painted_control_is_clickable_where_it_is_drawn`.

**Nada muda na tela.** A catraca `physics` `49 → 4`.

**Prova:** mutações — o braço a dobrar outra vez (desfaz o clique) e o cabeçalho de volta a `Button`
(não dobra) reprovam o gate da dobra; sem o grupo da matriz a catraca volta a acusar as `36` células.
Portão: `nextest-impacted` **17 703/17 703** · clippy `-D warnings` zero · `fmt` · censos da árvore
combinada **127/127**.

⏳ **Pergunta de produto que fica:** os dois interruptores são BOTÕES acesos; o idioma da casa para uma
propriedade ligada/desligada é a CAIXA DE MARCAR (Inspector, e o próprio Vector o escreve no
`checkbox_row`). Trocá-los muda o que o artista vê — é do dono.

## §9-sexiestricies — ⭐ FÍSICA: os dois interruptores viram CAIXAS DE MARCAR (`4 → 2` comandos)

Ordem do dono (2026-09-24): *«Caixas de marcar na Física»*. O *Enabled* do sono e o *Show Colliders*
eram botões acesos; hoje passam pela porta da casa (`property_row::paint_check_row`).

- **Registo:** `populate.rs` regista os dois como `InteractiveState::Checkbox`; o despacho vira-os e
  emite `WidgetEvent::Toggled(id)`. O `event.rs` casa `Toggled` (e deixou de chamar o
  `seam_reset_button`, que era do botão). ⚠️ **O valor NOVO sai do MODELO** (`!settings.sleep_enabled()`
  / `ToggleColliders`), nunca do store — a lei da caixa *Playing* do Inspector.
- **Catraca `CARGA_DE_COMANDOS`:** `physics` **`4 → 2`**, medido; ficam fechar e *Reset to Defaults*.
- ⛔⛔ **As duas catracas de elisão do degrau ESTREITO apanharam um corte** que a suíte do painel
  não via: `physics: 1 cortes (declarado 0) — ["Show Colliders"]`. Com `Seccao::apenas_campos(1)` a
  coluna do nome é a METADE da linha e o rótulo não cabe no degrau estreito. ⇒ o `check()` recebe a
  `Seccao` do chamador: o *Enabled* fica em `apenas_campos(1)` (alinha com as linhas de número da
  secção *Sleep*), e o *Show Colliders*, que é a **única** linha da secção *Debug*, usa
  `Seccao::medida` sobre o próprio rótulo (a caixa precisa de pouco; não há vizinho com quem alinhar).
  **A catraca não subiu** — o corte foi curado.
- **Gates:** `the_collider_toggle_asks_the_shell_to_flip_its_flag` e o censo
  `every_painted_control_is_clickable_where_it_is_drawn` passam a esperar `Toggled` para a caixa e
  `Click` para o *Reset*; `the_sleep_switch_writes_the_sign_the_solver_reads` corre o evento que o
  despacho devolve.
- **Mutação 2 de 2:** devolver cada braço do `event.rs` a `Click` reprova exactamente o gate da
  caixa respectiva (`1 failed` em `25`).
- **Portão:** `nextest-impacted` **17 703 / 17 703** · clippy `-D warnings` zero nas duas crates ·
  `cargo fmt --check` limpo · censos da árvore combinada **127 / 127** (12 de 12 correram).

## §9-septiestricies — ⭐ O MIXER no molde da Física: caixas de marcar, a escolha do *Key*, `22 → 17`

*«siga»* depois da Física (2026-09-24). O painel `audio_mixer` era o 5.º do censo de comandos
(`22`), e **cinco** deles eram o mesmo defeito que a Física pagou: liga/desliga pintados como
botões ACESOS a toda a largura (`paint_toggle` em `Rect::new(x, y, w, MUTE_H)`).

- **As quatro activações do master** (*Limiter* · *Reverb* · *Delay* · *Ducking*) são CAIXAS DE MARCAR
  pela porta (`paint_check_row`): `populate` regista-as `Checkbox`, o `event.rs` casa `Toggled` e o
  valor novo sai do `snapshot::toggle_*` (o modelo), nunca do store.
- ⭐ **O *Key* do ducking deixou de CICLAR** (`Key: Music → SFX → …`, um clique por passo e só a opção
  actual à vista) e é uma ESCOLHA pela porta da casa (`paint_choice_row`), com `AMIX_DUCK_KEY_BUS`
  (uma peça por sub-barramento, alinhada com `SUB_BUS_LABELS`) e `snapshot::set_duck_key(i)` no lugar
  do `cycle_duck_key`. A chave de texto `panel.audio_mixer.master.key` passa de `"Key: {bus}"` a
  `"Key"`. ⚠️ **Ela é PALETA** (nome por cima, grupo a toda a largura): as quatro peças medem `281 px`
  e não cabem ao lado do nome em nenhuma coluna do dock — é a lei da porta e a decisão do dono de
  23/09, não uma escolha deste painel.
- **A coluna do nome das caixas e do *Key* é MEDIDA sobre os nomes delas** (`Ctx::caixas`, uma
  `Seccao::medida` para as cinco), e **não** a `col` das barras: as barras são a CAIXA ÚNICA da casa
  (o nome DENTRO), logo não há coluna com quem alinhar, e a coluna de omissão cortava o nome no degrau
  estreito — a lição do *Show Colliders* (§9-sexiestricies), aplicada antes de a catraca a cobrar.
- **O *Play Test* fica botão** (é uma ACÇÃO, e o tom aceso é o que diz «a tocar») mas pela
  `caixa_do_botao` — coluna do valor, `ROW_H_PX`.
- **Catracas, MEDIDAS:** `CARGA_DE_COMANDOS` ganha `("audio_mixer", 17)` (os `17` são comandos da mesa:
  fechar, *Mute*/*Solo* das faixas, limpar o clip de cada medidor, *Play Test*, e o grupo do *Key*);
  a altura de abertura desce `1209 → 1207`. ⚠️ **A paleta do *Key* CRESCE a altura**; o que a paga é
  o passo da casa nas caixas, o `ROW_H_PX` do *Play Test* e o `+ Spacing::Sm` **escrito à mão** que
  saiu de depois do *Limiter* (medido a subir `+4` antes dessa última cura).
- **Gate novo** `every_master_effect_control_answers_where_it_is_drawn`: pinta o painel com as secções
  abertas e carrega no centro de cada caixa (espera `Toggled`) e de cada peça do *Key* e do *Play Test*
  (espera `Click`). O antigo `duck_key_click_cycles_the_key_bus` virou
  `duck_key_segment_chooses_its_own_bus` (inclui VOLTAR a um barramento anterior, que o botão que
  ciclava só alcançava dando a volta inteira).
- **Mutação 4 de 4:** braço do *Limiter* de volta a `Click` · peça do *Key* desviada de um · *Limiter*
  registado como botão · peças do *Key* por registar — cada uma reprova exactamente um gate.
- **Portão:** `nextest-impacted` **17 704 / 17 704** · clippy `-D warnings` zero (`audio-mixer`,
  `i18n`, `registry-init`) · `fmt` limpo · censos da árvore combinada 12 de 12.

## §9-octiestricies — ⭐ GRID & SNAP: seis escolhas montadas à mão, `20 → 2` comandos

*«siga»* depois do mixer (2026-09-24). Os `20` «comandos» do `grid_snap` eram **18 peças de SEIS
escolhas** pintadas com o pintor de peça da casa (`paint_segmented_button_in_group`) e **nenhum
grupo declarado** — mais o fechar.

- **As quatro escolhas com nome próprio** (*Neighborhood* nas cinco famílias que o têm ·
  *Orientation* e *Offset* do hex · *Parity* do escalonado · *Layer* do *Display*) passam pela porta
  `paint_choice_row`: `paint_labeled_segmented_row` e `paint_neighborhood_button_row` delegam-lhe, e
  **deixaram de pintar o nome SEMPRE por cima** (`paint_text` + `Spacing::Xs`). ⚠️ **A forma é da
  porta:** no dock de omissão as peças não cabem ao lado do nome e sai PALETA (o nome continua em
  cima, agora com o degrau da casa); com o painel mais largo elas vão para a coluna do valor.
- **A coluna da secção passou a contar os nomes das escolhas** que cada bloco pinta (cada
  `paint_*_cfg` acrescenta o *Neighborhood*/*Orientation*/*Offset*/*Parity* à `seccao`), e o
  *Display* ganhou `paint_rows::seccao_display` — UMA função lida pela caixa *Show Grid* e pela
  escolha *Layer*, para as duas não medirem sobre listas diferentes.
- **O *Kind* (`3 × 3`) e o *Target* (coluna de cinco) mantêm a FORMA** — o nome deles é o título da
  secção — e passam a declarar o grupo (`composto::grupo`).
- **Catraca:** `CARGA_DE_COMANDOS` ganha `("grid_snap", 2)`, medido.
- **Gate novo** `as_escolhas_passam_pela_porta` (duas metades: a porta vê o *Neighborhood* e o
  *Layer*; o clique REAL no centro de `Moore8` e de `Behind` chega ao `GridSnapState`, com o CONTROLO
  de que os valores de omissão são os opostos).
- **Mutação 4 de 4:** grelha do *Kind* sem grupo · coluna do *Target* sem grupo (as duas apanhadas
  pela catraca de comandos) · a escolha pintada FORA da porta · a 2.ª peça com id errado.
- **Portão:** `nextest-impacted` **17 706 / 17 706** · clippy zero · `fmt` limpo · censos 12 de 12.

## §9-noniestricies — ⭐⭐ O TÍTULO DE SECÇÃO DO APP É O DO GRID · as secções do Grid DOBRAM · a grade vai ATRÁS de verdade

Três ordens do dono de 2026-09-24, com foto do painel *Grid Settings*:
*«quero que essa seja a formatação exata (Font, tamanho da Font, etc) para todo o APP. Contudo o
separador azul não quero em baixo do título da seção, mas sim nas mesma linha à direita. No grid
as seções não fecham»* — e, no mesmo smoke: *«Behind deixa o grid mais discreto mas não atrás dos
objetos. corrija»*.

### O título (o `SectionHeader` da casa — os 49 sítios que o pintam)

- `paint_section_header` passa a pintar o nome **como está escrito** (⛔ saiu o `to_uppercase()`),
  no corpo `TypeToken::Md` (a porta nova `section_title_px()`) e no peso `SemiBold` do
  `paint_text_title` — exactamente o título que o Grid pintava à mão.
- ⭐ **O separador azul corre na MESMA linha, à DIREITA do nome** — da ponta do nome (medida no
  MESMO peso em que é pintado: medir em `Medium` punha a linha dentro da última letra) até à borda
  ou ao ornamento da direita (pastilha / círculo de cor), a meia altura do texto. A geometria é a
  função pura `regua_do_titulo`, com gate (`a_regua_corre_na_linha_do_titulo_a_direita_do_nome`).
- ⚠️ **O chevron FICA:** a foto do Grid não o tinha, mas a mesma ordem pede que as secções fechem, e
  o chevron é o que diz *«isto dobra»*. *A formatação pedida é a do TEXTO.*
- ⚠️ **As alturas dos cabeçalhos são dos chamadores** e nenhuma catraca de altura se mexeu; as
  elisões do app (a escada de quatro larguras) ficaram verdes com o corpo maior.

### As secções do Grid

- *Grid Kind* · *Target* · *Display* · *Inspect* eram texto pintado à mão (`paint_section_label`,
  **APAGADO**) ou um `SectionHeader` de id `NodeId(0)` — nenhum registava hit-rect. Hoje são o
  cabeçalho da casa com `SectionFold` (`paint_body_sections::dobravel`, um só sítio), ids
  `GS_SEC_KIND`/`GS_SEC_TARGET`/`GS_SEC_DISPLAY` (panel) + o `GS_INSPECT_HEADER` que a fundação já
  declarava, registados por `mark_collapsible_section`.
- ⚠️ **Uma chamada por id, nunca um laço** — o `the_painted_control_reaches_a_consumer` reconhece o
  despacho por KIND pelo TEXTO `mark_collapsible_section(ids::X)` e leu os quatro como órfãos
  quando estavam num array.
- A *Inspect* partiu-se na fundação: `grid_snap::inspect::paint_body` (o corpo sem cabeçalho,
  devolve o `y`) e o `paint` antigo delega-lhe — as duas formas não divergem.
- Gate `as_seccoes_dobram` (clique REAL no centro de cada título ⇒ `is_collapsed`).

### A grade ATRÁS dos objectos

- ⛔ O `Behind` era uma APROXIMAÇÃO escrita por extenso no `hero/paint.rs` — a opacidade × `0,4` —
  com o caminho verdadeiro deixado como *TODO: «a second Vello intermediate + a 3-layer
  compositor»*. ⭐ **Esse caminho já existia com outro nome:** o acumulador do mundo das FAIXAS DE
  DESENHO (ADR-0154 Fase 2) monta o quadro de trás para a frente e o compositor lê-o.
- ⇒ `screens::hero::grid_layer` (porta nova, UMA decisão de «onde»): `paint_in_chrome` pinta a grade
  no chrome **só à frente**; `paint_behind` pinta-a numa cena própria (`AppGfx::grid_behind_scene`)
  **só atrás**, e devolve `true`. No `present.rs`, `plan.banded |= grid_behind` FORÇA o quadro em
  camadas, e o `draw_lower_bands` desenha a grade **logo depois de limpar o fundo e antes de toda
  faixa**. Sem `Behind` e sem intercalação, o quadro é o de sempre.
- ⚠️ **Forçar o modo em camadas numa cena que não intercala é seguro, por medição do código:** com
  `needs_banding == false` o `doc_bands_of` devolve vazio, o documento continua na cena do chrome, e
  o `band_doc_scenes.get(i)` dos dois passes devolve `None` — nada se desenha duas vezes.
- **Foto** (roteiro `fotografa_cena.sh`, `PH2D_PHYSICS_SMOKE=1`, `grid_in_front` forçado a `false`
  só para a foto e reposto): à frente as linhas passam por cima do chão e da caixa; atrás elas ficam
  no fundo, com a MESMA força, e os objectos por cima.
- Gates: `grid_layer_tests` (a grade sai de UMA cena só, medido pelo que cada cena EMITE —
  `probe_bin_info_words`; e a metade da FORÇA por texto, porque a régua das palavras é cega à cor) ·
  `present_bands_grid_tests` (o `|=`, a engrenagem e a ORDEM limpar → grade → faixas).
- ⚠️ A régua da força apanhou a MINHA prosa a citar o código antigo (`opacity *=` no doc) — reescrita.

**Mutação 6 de 6** (régua no nome · régua por baixo · *Target* sem dobra · grade sempre no chrome ·
`paint_behind` a devolver `false` · o `|=` apagado). **Portão:** `nextest-impacted` **17 712 /
17 712** (a `load 100`) · clippy `-D warnings` zero (`editor-core`, `grid-snap`, a shell) · `fmt`
limpo · censos da árvore combinada 12 de 12.

## §11 — O que esta linha recomenda a quem a integrar

1. **Correr o `diag_onde_cai_a_pista_do_pente` da `line/sculpt3d` DEPOIS da fusão** e reescrever com
   a saída dele as **duas** tabelas vivas do §5 (o `CLAUDE.md` §5 e o doc-comment do
   `scenes_pente.rs`). ⛔ A do handoff `…_sculpt3d_2026-09-17.md` §94 **fica como está**.
2. **Promover à lista de flakes do §5.0:** `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`
   (`ph2d-tool-painter`) — único ✗ de `17 690`, gate de RAZÃO de custo, 3/3 verde sozinho a `load 23`, zero
   linhas de diff na crate (§9-vicies-septies).
3. A linha do §5 está no §9, pronta a colar.
