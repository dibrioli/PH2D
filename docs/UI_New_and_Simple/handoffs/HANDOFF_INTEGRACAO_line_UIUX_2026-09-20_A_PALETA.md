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

## §9-undecies — ⭐⭐⭐ A mesma cegueira virada 90°: o `tokens` lia **`110`** comandos e oferece **`4`**

O §9-decies fechou com *«o topo é hoje o `tokens` (110), que não usa composto nenhum — a dívida
dele é real»*. ⛔⛔ **Era falso, e a refutação veio de eu ir ATACAR o painel que o número
apontava.**

### §9-undecies.1 — O que a medição deu

Passo zero: correr o `diag_compostos_por_declarar` sobre o `tokens`. Ele lê **`2`** botões em
fileira ⇒ os `110` não são compostos escondidos. Passo seguinte: nomear as entradas. ⚠️ **A
varredura de texto que nomeia o Inspector lê ZERO aqui** — os ids deste painel são derivados do
índice da linha (`tokens.reset.{row}`), não literais ⇒ o mapa constrói-se chamando as **próprias
funções de id**, que são a fonte (`diag_de_quem_sao_as_entradas_do_tokens`).

| família | comandos | valores | órfãos |
|---|---:|---:|---:|
| **cor: elo** | **86** | 0 | 0 |
| **px: elo** | **21** | 0 | 0 |
| painel (fechar · import · export) | 3 | 0 | 0 |
| cor: swatch | 0 | 0 | **86** |
| px: campo | 0 | 21 | 0 |

⇒ **`107` dos `110` são UM botão**, o *elo*, pintado uma vez por linha — e por **decisão escrita**
no pintor: *«o elo é oferecido em TODA linha — qualquer token pode seguir qualquer outro, e
esconder o botão em linhas não-autoradas tornaria o gesto alcançável só onde ele já foi feito»*.
⚠️ E as `86` swatches de cor, que são o **editor de cada valor**, caem no balde dos *órfãos* por
não terem estado no store (elas são alvo de PICKER, também por decisão escrita).

### §9-undecies.2 — A lei, e ela é o gémeo da de ontem

> **Um controlo repetido ao longo de uma LINHA é um; um comando repetido ao longo de uma COLUNA
> também.** A primeira metade foi curada de manhã (o composto); esta é a segunda.

⚠️⚠️ *A primeira foi achada por SUSPEITA; a segunda só apareceu porque alguém foi gastar uma wave
no painel que o número apontava.* **Uma régua enviesada não se corrige sozinha: ela envia trabalho
para onde ela própria está errada.**

### §9-undecies.3 — ⛔ O discriminador NÃO é geométrico, e o controlo matou a 1.ª redacção

A 1.ª versão agrupava botões por **coluna** (mesmo `x`, mesma largura) e colapsava os que
estivessem em faixas de linha distintas. Ela leu:

```
inspector    88 botões →  29 distintos      ⛔ FALSO
tokens      110 botões →   4 distintos      ✅
```

O detalhe das colunas refutou-a: `[inspector] x=1626 w=268: 34 linhas` é uma coluna de **34 botões
de largura cheia** — *34 comandos diferentes empilhados*, colapsados em `1`. Já
`[tokens] x=1882 w=20: 107 linhas, passo 25` é o elo por linha.

⭐ O discriminador que fica é a **PROVENIÊNCIA do id**: um id escrito no fonte é um comando
**nomeado**; um que nasce de `hash_node_id_runtime(&format!("…{row}"))` é uma **instância** de uma
família — e **só dentro do balde dos derivados** a coluna decide.

### §9-undecies.4 — ⚠️⚠️ Há TRÊS formas de declarar um id, e uma que falte erra no sentido MAU

| forma | exemplo | quem a usa |
|---|---|---|
| `hash_node_id("<lit>")` | `hash_node_id("tokens.close")` | a maioria |
| `hash_node_id_runtime("<lit>")` | `hash_node_id_runtime("asset_browser.panel")` | `asset-browser` |
| **`NodeId(<n>)` cru** | `pub const GS_CFG_QT_MAX_DEPTH: NodeId = NodeId(1033);` | **`grid-snap`** |

⛔ Uma forma que falte manda os ids dela para o balde dos derivados, onde eles **colapsam** — e o
painel lê-se **mais barato do que é**. Medido durante a construção: sem a terceira forma o
`grid-snap` lia **`10`** comandos distintos em vez de **`20`**.

⇒ gate novo **`todo_painel_com_id_derivado_esta_nomeado`**: todo painel cujo bruto excede o
distinto tem de estar na `PAINEIS_COM_ID_DERIVADO` **com a fábrica escrita ao lado**, com a metade
da **obsolescência** (uma entrada que já não descreve nada reprova) e **piso de população** (uma
partição com um lado vazio não afirma nada). ⚠️ **A 1.ª redacção dele exigia a fábrica na crate do
PAINEL, e o `wet_tuning` desmentiu-a**: os *Reset* dele nascem em `ph2d-tool-painter`.

### §9-undecies.5 — A tabela honesta

| painel | botões | **distintos** |
|---|---:|---:|
| **inspector** | 88 | **88** |
| physics | 49 | 49 |
| sculpt3d | 36 | 36 |
| vector | 24 | 24 |
| audio_mixer | 22 | 22 |
| grid_snap | 20 | 20 |
| flip_frames | 20 | 20 |
| painter_layers | 18 | 18 |
| **wet_tuning** | 58 | **17** |
| timeline | 13 | 13 |
| **tokens** | **110** | **4** |

**Dois painéis mentiam, e os dois são LISTAS.** O Inspector é o líder e sempre foi.

### §9-undecies.6 — ⭐ A catraca mudou de coluna, e a razão é a falsa acusação

`CARGA_DE_COMANDOS` passa a medir `distintos`. ⛔ Com o número **bruto**, acrescentar um token de
desenho fá-lo subir e a mensagem acusa *«quase de certeza um composto deixou de se declarar»* —
**falso**, e manda a cura para o sítio errado. O distinto é invariante ao comprimento da lista, que
é o que uma dívida de capacidade tem de ser.

⚠️ **Ela continua a guardar os compostos**: apagar um `composto::grupo` devolve as células ao balde
dos botões, e como elas têm id nomeado o distinto sobe na mesma (mutação 4 sangra).

### §9-undecies.7 — Prova e portão

**6 de 6 mutações sangram**, com os três controlos do arnês (agulha casa `1×` · tem de compilar ·
população de `passed + failed`): a forma `NodeId(<n>)` esquecida · a instância sem coluna · a
coluna a decidir para todo botão (a redacção refutada) · as abas sem declarar o grupo · uma entrada
obsoleta na lista · a catraca a voltar ao bruto.

Portão: registry **`9/9`** no âmbito com as quatro features · `nextest-impacted`
**`15 493/15 493`** · censos da árvore combinada **`127/127`** · clippy `-D warnings` zero · fmt
limpo · tecto de LOC verde.

⚠️ **Para quem integrar:** a corrida `-p` desta crate **não regista** `painter_layers`, `flip`,
`flip_frames` e `wet_tuning` (eles chegam pelo `shells/desktop`) — o `o_censo_recusa_o_ambito_pobre`
di-lo em voz alta, e sem as quatro features o `wet_tuning` desaparece da catraca.

---

## §9-duodecies — ⭐⭐⭐ O Inspector ABRE DOBRADO: de `2,5`–`6,5` ecrãs para `0,8`

O §9-decies fechou com *«triar o Inspector devolve POUCO ecrã: o que o enche são propriedades»*.
Isto ataca a altura pelo outro lado — **o que aparece QUANDO**.

### §9-duodecies.1 — ⛔⛔ Passo zero, e ele mudou a wave

A fixtura [`o_inspector_armado::arma_tudo`] monta um **objecto IMPOSSÍVEL** (o cabeçalho dela di-lo
por escrito: *«ele não pretende ser um objecto que exista»*). Medir a ALTURA nela dá **`14 987 px`
= `17` ecrãs** — *um estado que nenhum artista alcança*, e uma wave gasta ali seria a terceira
perseguição a um fantasma neste ficheiro.

⇒ a fixtura ganhou a `PORTAS` (tabela `(nome, fn())` **derivada da `desarma_tudo`**, com gate de
cobertura nos dois sentidos), que a deixa **compor-se**: armar tudo e desarmar um subconjunto dá a
altura de um objecto que existe, e desarmar uma porta de cada vez dá o **preço** de cada secção.

### §9-duodecies.2 — A medição

| objecto | antes | depois |
|---|---:|---:|
| **sprite simples** (o mais comum do app) | `2 175 px` (`2,5` ecrãs) | **`673 px` (`0,8`)** |
| sprite + corpo físico | `3 146` (`3,6`) | **`673`** |
| herói de plataforma | `5 736` (`6,5`) | **`673`** |
| *o objecto impossível* | *`14 987` (`17,0`)* | — |

A dobra é `880 px`.

### §9-duodecies.3 — ⭐ O mecanismo estava completo; faltava a POLÍTICA

`SectionFold`, o chevron, a animação, o recorte e o clique já existiam e estão certos — medido,
**`0` de `38`** secções vivas têm dobra inerte. O que não existia era **uma secção nascer dobrada**:
a única linha do repo que semeava uma dobra era a máscara de *cull* da câmera.

### §9-duodecies.4 — ⚠️⚠️ A `Transform` não é preferência: é a única que CABE

O orçamento da dobra dá para **uma** secção. Medidas uma a uma:

| política | altura | cabe? |
|---|---:|---|
| **só a `Transform`** | **`849 px`** | ✅ |
| só a `Render` | `1 000` | ❌ |
| `Transform` + `Render` | `1 097` | ❌ |
| identidade (4 secções) | `1 097` | ❌ |

⭐ E o número **não depende do objecto** (`673`/`849` nos três cenários), porque tudo o que varia
entre eles está dobrado: *o painel abre sempre igual e cabe sempre.*

### §9-duodecies.5 — ⛔⛔ O SÍTIO da política mediu-se em gates partidos

| onde ela mora | gates que reprovam |
|---|---:|
| `Panel::populate` do Inspector | **342** |
| a porta de arranque do EDITOR (`pre_populate::marca_as_gavetas`) | **4** |

⇒ *o painel declara o que PODE mostrar; quem compõe o editor declara como ele ABRE* — e o painel
**não conhece a altura da janela**. A `marca_as_gavetas` foi extraída nesta wave (ela já era o sítio
que marca o conjunto das gavetas; passou a dizer também como elas nascem), e tem **dois**
chamadores: o produto e o arnês.

### §9-duodecies.6 — ⭐⭐ A `collapsed_choice` ganhou o primeiro consumidor do repo

O doc dela descrevia por escrito o defeito da semeadura crua — *«semear por cima de uma escolha do
artista reabriria a gaveta que ele fechou, a cada quadro»* — e ela **não tinha um único chamador**.
Hoje `set_collapsed_if_unchosen` torna a semeadura idempotente e os **quatro** sítios que semeiam
dobra passam por ela, com censo (`toda_semeadura_de_dobra_passa_pela_porta`) a proibir a recaída.

⚠️ **O perigo é LATENTE e não vivo, e a distinção está escrita:** hoje o `populate` do Inspector
corre **uma vez** (`HeroScreen::new` do arranque), mas **cinco** painéis da shell re-populam-se por
interacção. Reproduzido pela porta do produto sobre a máscara de *cull*:
`true` → clique → `false` → `populate` → **`true`**.

### §9-duodecies.7 — ⛔⛔⛔ A consequência atravessou TRÊS censos

Os três medem o **ECRÃ**, e o ecrã passou a dobrar:

| régua | o que passou a ler | cura |
|---|---|---|
| censo de entradas | `12` comandos onde há `88` | abre as gavetas antes de medir |
| varredura das elisões | deixa de ver rótulos dentro das gavetas | idem |
| `4` gates do painel | *«o id não foi pintado»* | `open_all_sections` no testkit |

⇒ **duas perguntas, duas réguas**: os censos perguntam *quanto o painel TEM*, o
`o_inspector_abre_dentro_da_dobra` pergunta *quanto ele MOSTRA ao abrir*. ⚠️ Chamar o
`open_all_sections` é uma **declaração** de que aquele gate é sobre conteúdo.

⚠️ **Uma catraca de elisão sobe `4 → 5`** no `painter_layers`: o `Use Color Ramp` vive numa secção
que aquele painel semeia dobrada e a varredura passou a abri-la. *População nova, não regressão.*

### §9-duodecies.8 — ⛔ Uma hipótese minha, refutada pelo fonte

Li o `paint_core_sections` (Name · Visibility · Transform passam por `begin_section`/`finish_section`
**sem consultar a dobra**) como *«três chevrons mortos»* — a família que 2026-08-21 curou em
Ordering · Sampling · Blend. **Falso:** a `Name` e a `Visibility` são **fileiras sem cabeçalho**,
logo não há promessa quebrada. ⚠️ E a 1.ª régua disso era fraca (`conta == base`); a que decide
imprime o **DELTA** de cada secção, e é ela que mostra a `visibility` a `−2` contra `physics −27` e
`player −62`.

### §9-duodecies.9 — ⛔ O arnês mediu outro programa duas vezes

- **`populate_shared` PENDURA o censo**: ele toca no registo de painéis e o censo já tem o *mutex*
  dele na mão — **10 min a `0 %` de CPU**. ⇒ a `marca_as_gavetas` existe também por isso.
- **As duas réguas da dobra não passavam pela mesma porta**: o «chão» lia `752 px` e a abertura
  `673` — *o painel com a `Transform` ABERTA lia-se mais baixo do que com tudo fechado*, porque só
  uma delas via as SUB-secções que o produto semeia (a grelha de 32 camadas, a máscara de *cull*).

### §9-duodecies.10 — Prova e portão

**6 de 6 mutações sangram**: a política sai · a `Transform` nasce dobrada também · a porta volta a
escrever por cima da escolha · o censo deixa de abrir as gavetas · o arnês deixa de as marcar (o
controlo dele) · uma semeadura CRUA volta a um `populate`.

Portão: `nextest-impacted` **`15 497/15 497`** · censos da árvore combinada **`127/127`** ·
`ph2d-panel-inspector` **`355/355`** · clippy `-D warnings` zero · fmt limpo · tectos de LOC verdes
(`pre_populate.rs` `676` de `700`).

⚠️⚠️ **ISTO MUDA O PRODUTO** e o smoke do dono está por fazer.

---

## §9-terdecies — ⭐⭐ O painel do Painter: `3,1 → 1,7` ecrãs, e a decisão do dono fica INTACTA

### §9-terdecies.1 — A régua apontada a todo o registo

A régua da dobra do §9-duodecies, sobre os `27` painéis, com a coluna que decide a wave seguinte:

| painel | abre com | tudo dobrado | **compra** |
|---|---:|---:|---:|
| **painter_layers** | `2 709` | `736` | **`1 973 px`** |
| vector | `1 349` | `104` | `1 245` |
| widget_gallery | `3 367` | `2 360` | `1 007` |
| audio_mixer | `1 209` | `408` | `801` |
| inspector | `918` | `821` | `97` |
| **sculpt3d** | `2 097` | `2 097` | **`0`** |
| **tokens** | `2 866` | `2 866` | **`0`** |

⛔⛔ **O `sculpt3d` compra ZERO porque as secções dele NÃO SÃO DOBRÁVEIS** — o que explica por que
a `line/sculpt3d` não a conseguiu curar e chamou ao caso *«decisão de produto»*
([§94](../../3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md): *«o painel está sobre o
orçamento, o `BRUSH` sozinho ocupa `657` dos `880`»*). A cura lá é outra e maior: **dar-lhes dobra
primeiro**.

⚠️ **Armadilha NOMEADA:** vários painéis leem `3 992`/`3 866` porque **ANCORAM à janela** — a altura
deles *é* a janela e não o conteúdo, e a coluna `compra` dá `0` neles por construção.

### §9-terdecies.2 — ⛔⛔ O achado é uma DATA

Este painel **já tinha** política de dobra, e ela é **decisão do dono** — a doc do
`populate_sections.rs`: *«Randomize Color, Color Ramp e Tiling START COLLAPSED; Texture + Stroke
start expanded (Enio 2026-06-24)»*.

A decisão nomeia **cinco** secções e o painel tem **treze**. Medido por `git log -S`:

| secção | nasceu | decidida? |
|---|---|---|
| `Texture` · `Stroke` | **2026-06-24** | ✅ abertas |
| `Randomize` · `Color Ramp` · `Tiling` | 2026-06-24 | ✅ recolhidas |
| **`Shape` (`807 px`)** · `Shape Ramp` | **2026-06-25** | ❌ |
| `Symmetry` | 2026-06-29 | ❌ |
| `Watercolor Paper` (`373 px`) | 2026-07-05 | ❌ |

⇒ *o maior bloco do painel não existia quando ele decidiu*, e as quatro que chegaram depois
nasceram **abertas** sem que ninguém revisse a nota — o §0.0 à letra, na direcção que ninguém
vigia.

### §9-terdecies.3 — O que esta wave faz, e o que NÃO faz

Nascem recolhidas **só** as que nunca tiveram decisão: `Shape` · `Shape Ramp` · `Symmetry` ·
`Watercolor Paper`. Medido: **`2 709 → 1 529 px`** (`3,1 → 1,7` ecrãs).

⛔ **Ele NÃO passa a caber, e o que falta é DECISÃO e não código:** o `Texture` (`450`) e o `Stroke`
(`368`) são a escolha dele, e o painel tem ~`700 px` de cromo fora de secção nenhuma. Com tudo
recolhido ele mediria **`736 px`**. *A pergunta está posta ao dono com o número ao lado.*

### §9-terdecies.4 — ⭐⭐ A catraca mede o modo de falha que a wave encontrou

`a_altura_de_abertura_de_um_painel_so_encolhe` — porque *uma secção nova nascer aberta e ninguém
rever a decisão* é o caminho silencioso por onde um painel volta a transbordar, e **nada neste repo
media a altura de ABERTURA de um painel**. ⭐ Ela guarda a escolha do dono nos **dois** sentidos: a
mutação que põe o `Texture` dele a nascer recolhido **também sangra**.

### §9-terdecies.5 — Prova e portão

**4 de 4 mutações sangram**: as quatro que chegaram depois voltam a nascer abertas · uma das três do
dono deixa de nascer recolhida · o `Texture` dele passa a nascer recolhido (a metade que DESCE) · um
gate de costura deixa de declarar que mede conteúdo de secção.

⚠️ Quatro gates de costura do painel passaram a declarar `host.open_all_sections()`, a mesma linha
que os quatro do Inspector levaram.

Portão: `nextest-impacted` **`15 498/15 498`** · censos da árvore combinada **`127/127`** ·
`ph2d-panel-painter-layers` **`179/179`** · clippy `-D warnings` zero · fmt limpo.

⚠️⚠️ **ISTO MUDA O PRODUTO** e o smoke do dono está por fazer.

---

## §9-quaterdecies — ⭐⭐⭐ O NOME perde a REGRA e ganha o BALÃO — e a UNIDADE vive no CAMPO

Ordem do dono (2026-09-21), a 4.ª das cinco direcções dele: *«quanto aos nomes grandes precisamos
reduzir, as dicas devem ser passadas para o mouse Hover»*.

### §9-quaterdecies.1 — Porque encurtar um nome NESTE painel é aritmética, não estética

A coluna do nome é `min(50 %, …)` da largura da fileira, e ela é propriedade da **SECÇÃO** ⇒
*o nome mais comprido de uma secção come a coluna do CONTROLO de todos os vizinhos dela*. Medido:
`Per-Corner Tint (vertex gradient)` deixava as quatro amostras a **`35 px`**; `Per-corner Tint`
deixa-as a **`59`** — `68 %` mais alvo, de uma string.

| régua (`diag_que_linhas_o_nome_espreme`, `--run-ignored all`, workspace) | antes | depois |
|---|---|---|
| linhas empurradas pelo próprio nome | `45` | **`18`** |
| pior empurrão | `+48 px` | **`+17 px`** (`insp_mount_pick`) |

As `18` que ficam: `+17` o `insp_mount_pick`, `+8` o bloco do áudio e os `insp_vis_*`, `+3` o bloco
da sprite.

### §9-quaterdecies.2 — ⛔⛔ Um parêntesis é uma de DUAS coisas, e encurtar mal converte uma na outra

`(0 = forever)` é uma **REGRA DE VALOR** e vai ao balão; `(s)`, `(m)`, `(deg/s)` é uma **UNIDADE** e
mora no **CAMPO** (o 12.º argumento do `paint_fields_row`, que já existia). ⚠️ Encurtar
`Lifetime (s, 0 = forever)` para `Lifetime (s)` **parece** a cura e deixa a unidade dentro do texto
— quem o apanhou foi um gate **PRÉ-EXISTENTE** (`no_row_label_carries_its_own_unit`), e a cura certa
não perde nada: o artista continua a ler `2 s` no campo e `0 = lives forever.` ao passar o rato.

Três sítios: `lifecycle.rs` (`Unit::Seconds`) · `projectile.rs` (`Meters`) · `topdown.rs`
(`DegreesPerSecond`). ⭐ **A CHAVE de i18n mantém o sufixo** (`..._s_0_forever`): ela é um ENDEREÇO,
nunca o texto.

### §9-quaterdecies.3 — O par `(controlo, dica)` escreve-se À MÃO

⛔ A 1.ª tentativa derivou-o por **proximidade no fonte** e mapeou o `Homing` para o
`INSP_PJ_SPEED` — *um balão no controlo errado é pior do que balão nenhum*. ⇒ só entram os pares em
que o `tr(<rótulo>)` é **imediatamente** seguido pelo id: **13** dos `21` rótulos com regra. Os
outros `8` ficam NOMEADOS em `AINDA_COM_REGRA` (`nenhum_nome_carrega_uma_regra.rs`), com catraca que
só encolhe — a forma de chamada deles (entradas de texto, linhas de lista) não põe o id ao lado do
rótulo.

### §9-quaterdecies.4 — A mutação que escreveu o terceiro gate

⛔ Apagar o laço de `populate_dicas::dicas()` deixava **verdes** as duas metades declarativas
(*«cada dica citada tem texto»* · *«nenhum rótulo carrega regra»*) — *uma régua que lê a DECLARAÇÃO
nunca vê o FIO* ⇒ `cada_dica_declarada_chega_ao_store`, que mede o `WidgetStore` **depois** do
`populate`. **5 de 5 mutações sangram.**

### §9-quaterdecies.5 — Catracas e portão

`CORTES_NO_DEGRAU_ESTREITO["inspector"]` **`102 → 90`** e
`LETRAS_PERDIDAS_NO_DEGRAU_ESTREITO["inspector"]` em **`85`** — as duas **exactas**, com o censo de
obsolescência a passar (uma catraca que passa ainda pode estar obsoleta; esta não está).
⚠️ A corrida autoritativa é `--workspace`: um `-p` lê `23` painéis / `12 418` rótulos contra o piso
de `27` / `12 000` e **recusa alto com a cura na mensagem**, que é a lei que a integração de 20/09
pagou.

Portão: `nextest-impacted` **`17 656/17 656`** · clippy `-D warnings` zero nas quatro crates
tocadas · `cargo fmt --check` limpo · as 5 catracas de elisão PASS a partir da workspace.

⚠️⚠️ **ISTO MUDA O PRODUTO** e o smoke do dono está por fazer.

---

## §9-quindecies — ⭐⭐⭐ DUAS GRANDEZAS COM NOMES PARECIDOS, e cinco secções trocaram-nas

Report do dono, 2026-09-21, com foto da secção `PROJECTILE MOTION`: *«Apenas o checkbox tem sua
moldura e ele próprio menores que o padrão. isso acontece em vários painéis do APP.»*

### §9-quindecies.1 — A causa, e porque ela encolhe DUAS coisas

| grandeza | porta | valor | o que é |
|---|---|---:|---|
| altura de uma LINHA | `ph2d_tokens::ROW_H_PX` | `22` | a caixa do controlo, em toda fileira |
| aresta da MARCA | `ph2d_tokens::CHECKBOX_BOX_PX` | `18` | o quadrado que leva o visto |

A marca vive **dentro** da caixa com um degrau de recuo de cada lado
(`lado = min(18, h − 2·Xs)`), logo escrever `18` onde se pedia a altura da linha encolhe as
**duas** coisas de que o report fala: a moldura `22 → 18` e a marca `14 → 10`. ⭐ *As duas frases
do report são um número só.*

### §9-quindecies.2 — ⛔⛔ A cura já tinha sido escrita — para UM dos treze sítios

O `sections/anchor_mount_row.rs` diz-o no próprio comentário desde **2026-09-15**: *«era `18.0`, o
MESMO literal em TREZE sítios»*. Quatro sítios foram curados nesse dia (`anchors`, `slice_nine`,
`visibility`, `anchor_mount_row`) e **cinco ficaram**, com um comentário a afirmar
`igual à das irmãs` — *uma frase que era verdade no dia em que foi escrita e que a cura da irmã
tornou falsa, sem nada deixar de compilar*.

⚠️ **Não havia censo.** É a lei do `CLAUDE.md` §5.0 (*«uma catraca sem censo de obsolescência não
desce: ela vira LICENÇA»*) um degrau abaixo: aqui nem catraca havia, e a única régua capaz de
encontrar aquilo era o olho do dono.

### §9-quindecies.3 — A medição, pela porta do produto

A régua nova ([`a_marca_tem_a_altura_da_linha.rs`](../../../crates/ph2d-panel-registry-init/tests/it/a_marca_tem_a_altura_da_linha.rs))
lê o **rect que o painel REGISTA** para cada id cujo estado no store é `Checkbox` ou `Toggle` — a
altura da linha pelo caminho do produto, e não um literal lido no fonte.

| | antes | depois |
|---|---|---|
| `inspector` | `18 px × 7` · `22 px × 33` | **`22 px × 40`** |
| `timeline` | `22 px × 10` | igual |
| `grid_snap` | `22 px × 1` · `44 px × 1` | igual (o `44` é uma isenção NOMEADA) |

**As sete, pelo nome:** `insp_factory_aim` · `insp_factory_pick_random` · `insp_hud_disabled` ·
`insp_part_emitting` · `insp_part_one_shot` · `insp_pj_face_velocity` · `insp_td_default_controls`.

⭐ A sonda NOMEIA-as porque reutiliza o mapa inverso `NodeId → slug` que o
`o_que_o_artista_nao_alcanca` já tinha — ele passou a receber a lista de fontes por argumento
(`nomes_de`), porque *a pergunta muda as árvores a varrer e o extractor não*.

### §9-quindecies.4 — A cura é a PORTA, não um número melhor

Os cinco sítios eram a mesma montagem à mão de quatro passos que o
`ph2d_editor_core::property_row::paint_check_row` existe para substituir — e dois deles
(`hud`, `particles`) tinham **cópias locais** da porta. Hoje os sete passam por ela, e **ela não
aceita altura nenhuma**: não há onde escrever o literal outra vez.

⭐⭐ **E isso trouxe de graça a coluna certa:** montadas à mão, as sete usavam
`Seccao::apenas_campos(1)` — *o nome de uma linha de marcar caía num `x` e o da linha de número
acima dela noutro*. Pela porta, cada uma entra na coluna da SECÇÃO.

⚠️⚠️ **E o preço apareceu num gate, não num smoke:** a `factory` mede a coluna de uma lista
escrita à mão que **não continha** os dois nomes de marcar, logo `"Aim from spawner"` — o nome mais
comprido da secção — saía **cortado**. Quem o apanhou foi o `nenhum_corte_novo_entra_sem_ser_nomeado`,
e a cura é a que o comentário três linhas acima da lista **já mandava por escrito**
(*«os nomes são os da secção INTEIRA»*): os dois entram na medição.

⭐ **E as duas catracas de elisão DESCERAM** (`90 → 89` cortes, `85 → 84` letras), exigido pelo
censo de obsolescência delas — *a coluna ficou mais CERTA, e nenhum nome foi encurtado para isso*.
As `18` linhas empurradas pelo próprio nome ficam em `18`.

### §9-quindecies.5 — ⚠️ O que a régua NÃO alcança, e o que fechou o buraco

A varredura só vê **três** painéis com marcas booleanas — um painel cujas caixas só aparecem com um
documento que a `paineis_armados::TABELA` não sabe montar é invisível a ela. ⭐ O buraco foi fechado
por **ENUMERAÇÃO**: só quem chama `paint_checkbox` **fora da porta** pode escolher a altura, e são
`17` ficheiros (inspector 5 · painter-layers 4 · vector 1 · wet-tuning 1 · a porta e os gates do
`editor-core`). **Todos os de fora do Inspector passam `ROW_H_PX`** ⇒ a doença era do Inspector e só
dele, e *«vários painéis»* eram cinco SECÇÕES do mesmo painel. ⛔ Isto é uma medição com data: quem
escrever o 18.º chamador não é avisado por nada.

⏳ **ABERTO, com o mecanismo:** o `sections/script.rs` passa ao pintor **só a coluna do controlo** em
vez da linha, logo o `colunas_da_linha` volta a partir esse rectângulo em duas e a caixa nasce a
meio dele. A altura é `22`, logo este gate não a vê.

### §9-quindecies.6 — Prova de mutação: **8 de 8 sangram**

| # | mutação | veredito |
|---|---|---|
| 1 | uma secção volta a registar a linha a `18 px` | SANGRA |
| 2 | a isenção do `grid_snap` desaparece | SANGRA |
| 3 | a isenção declara uma altura que ninguém tem | SANGRA |
| 4 | **a PORTA** passa a pintar a linha com a aresta da marca | SANGRA |
| 5 | o censo deixa de colher (o piso de população) | SANGRA |
| 6 | a isenção passa a casar só pelo PAINEL | SANGRA |
| 7 | o painel excluído passa a ser um nome que não existe | SANGRA |
| 8 | o CONTROLO da fixtura passa a ser o próprio id declarado | SANGRA |

⛔⛔ **A nº 6 SOBREVIVEU primeiro, e a cura não foi um gate a mais — foi uma FIXTURA:** hoje o
`grid_snap` tem **uma só** marca fora do padrão, logo a população do produto não discrimina
*«isento por painel»* de *«isento por par»*. *Um corpus que não contém o fenómeno não o pode
testar* ⇒ o discriminador é `a_isencao_e_do_par_e_nao_do_painel`, com o CONTROLO (um id do mesmo
painel que ninguém declarou) dentro.

⚠️ **E a nº 7 só discrimina numa direcção:** pôr `|| true` na verificação da exclusão **não sangra**
(hoje o painel existe); o que sangra é trocar o nome excluído por um que não existe. *Uma asserção
que só morde no dia em que alguém renomeia não se prova pelo lado do `true`.*

⚠️⚠️ **E o arnês mentiu à primeira:** `grep -cF` conta **LINHAS**, logo uma agulha multi-linha casou
`1` vez e ele leu `20` — a lei que este repo já tinha escrito, paga outra vez. A contagem passou a
ser de OCORRÊNCIAS.

### §9-quindecies.7 — Portão

`nextest-impacted` · clippy `-D warnings` zero nas três crates tocadas · `cargo fmt --check` limpo ·
as catracas de elisão **DESCERAM** com o censo de obsolescência a exigi-lo.

⚠️⚠️ **ISTO MUDA O PRODUTO** — as sete linhas ficam `4 px` mais altas e a marca delas `4 px` maior.

---

## §9-sedecies — ⭐⭐⭐ UM CAMPO DE TEXTO DIZIA PARA QUE SERVIA ATÉ ALGUÉM O USAR

Report do dono, 2026-09-22, com foto da secção `FACTORY` — cinco caixas seguidas, cinco setas
vermelhas: *«Campos de texto difíceis de saber para que servem. Como resolver isso?»*

### §9-sedecies.1 — A causa, em duas linhas medidas

| | medido 2026-09-22 |
|---|---:|
| caixas de texto do app com rótulo VAZIO | **`53`** |
| com rótulo | `3` |

E o espaço reservado — o único sítio onde o sentido vivia — é pintado **só enquanto a caixa está
vazia** (`text_input/mod.rs`: `if displayed.is_empty() && !input.placeholder.is_empty()`).
⇒ *um campo de texto deste app dizia para que servia exactamente até alguém o usar.*

A razão estrutural: o `text_row` do Inspector pedia as colunas ao
`widget::form_row_columns`, que **não tem coluna de nome** — ele devolve a largura toda menos o
ponto de animação.

### §9-sedecies.2 — ⛔⛔⛔ O diagnóstico JÁ ESTAVA ESCRITO, com a cura aplicada a UMA secção

O `sections/hud.rs` trazia isto, palavra por palavra, desde a wave dele:

> *«O nome vai POR CIMA, e não no `TextInput`** — a foto apanhou três campos seguidos sem um único
> nome à vista. O `text_row` pinta o controlo na largura toda e **não desenha o rótulo**; e pô-lo
> no `placeholder` seria pior do que nada, porque um placeholder desaparece exactamente quando o
> campo tem valor — que é quando o artista precisa de saber o que é.»*

⇒ **o diagnóstico estava certo, a cura foi LOCAL, e a porta ficou como estava** — as outras trinta
linhas de texto do app continuaram mudas. ⚠️ E o remendo (o nome POR CIMA) contrariava o próprio
dono: *«Label acima do campo numérico! Muito ruim!»* (2026-09-14). *É a mesma forma do `18.0` da
§9-quindecies, um dia antes: uma cura aplicada a um sítio, com a família escrita ao lado.*

### §9-sedecies.3 — A cura: uma porta que EXIGE o nome

`ph2d_editor_core::property_row::paint_text_row` — irmã do `paint_check_row` e do
`paint_fields_row`, com a mesma `row_and_layout`. ⚠️ **O `label` e a `Seccao` não têm valor de
omissão de propósito:** um `""` seria o defeito a voltar em silêncio.

**`34` sítios em `21` secções** passaram por ela, mais **cinco CÓPIAS locais** que foram apagadas
(`hud`, `particles`, `weapon`, `anchors`, `physics_tag_row` — a quinta dizia por escrito, no
cabeçalho, que copiar aquela porta era o que fazia a lei do espaço reservado perder-se).
`41` chaves de i18n novas, todas irmãs da chave do espaço reservado que já existia: *a de baixo é
o EXEMPLO, a nova é o NOME.*

### §9-sedecies.4 — A régua auto-calibrada

**Uma caixa de texto COMEÇA onde as caixas de número do mesmo painel começam.** Não há aqui número
escolhido nenhum: a coluna do controlo sai do próprio painel.

| | antes | depois |
|---|---|---|
| caixas de texto do Inspector na coluna do controlo | `0` de `50` | **`49` de `50`** |
| `x` das caixas de texto | `{1072, …}` | `{1179, 1206}` = o `x` dos números |

⚠️ A que fica é o **`insp_entity_name`** — o nome do OBJECTO, no topo do painel: ele é o título do
que está seleccionado e não uma propriedade entre outras. Isenção NOMEADA, com censo de
obsolescência.

⛔ **E a população da régua são os painéis que TÊM formulário.** O `hierarchy`, o `asset_browser`
e o `audio_editor` têm caixas de texto e **zero** caixas de número — ali não há coluna contra que
medir, e a caixa é nomeada pelo sítio onde vive. *Medir ali seria inventar uma barra.*

### §9-sedecies.5 — ⭐⭐ O que NÃO piorou, e é o que torna a wave barata

As duas catracas de elisão ficaram **exactamente onde estavam** (`89` cortes / `84` letras) com
`34` nomes novos no painel, e as linhas empurradas pelo próprio nome ficaram em `18`. ⭐ A razão é
a lei de 2026-09-21: *um nome perde a EXPLICAÇÃO antes de perder LETRAS* — os `41` são de uma ou
duas palavras (`Recipe`, `On Signal`, `On Exhausted`).

⚠️⚠️ **E «nenhum corte novo» lê-se igual a «a varredura não viu os nomes»** ⇒ a sonda
`diag_que_nomes_de_texto_o_inspector_pinta` confirma que eles são PINTADOS (`10` de `10` dos nomes
procurados, em `1 705` rótulos medidos). *Um censo que passa sem ver a população não afirma nada.*

### §9-sedecies.6 — O preço, declarado

A caixa perde a metade esquerda da linha. No degrau em que o dono trabalha (colunas no mínimo) ela
fica com `~97 px` contra os `~182` de antes — que é exactamente o que toda caixa de número deste
app já vive, e o piso delas (`NUMBER_INPUT_MIN_W_PX = 72`) continua a ser a cerca.

---

## §9-septdecies — ⭐⭐⭐ O SELECTOR DE COR: o defeito, e a CATRACA que impedia a cura dele

> Report do dono, 2026-09-21 (com desenho) e 2026-09-22: *«os seletores de cor de todo o app
> precisam ser padronizados»*.
>
> ⚠️ Esta wave levou **duas** sessões: a 1.ª mediu e **reverteu**, a 2.ª trocou a régua e shipou.
> As duas ficam escritas, porque o que bloqueou a 1.ª é a lição.

### §9-septdecies.1 — A medição, e as TRÊS populações que o nome não separa

Sonda `diag_que_forma_tem_cada_seletor_de_cor`, pelo caminho do produto e com a passagem **armada**
(sem ela cinco painéis pintam zero fileiras e o Inspector — o painel do report — fica invisível).
A 1.ª corrida leu **45** candidatos e **35 NÃO eram selectores**: `18 × 18` é o **ponto de cor do
CABEÇALHO** (`color_circle_hit_rect`) e `268 × 22` é a **banda de DOBRA** da secção. ⛔ Padronizar
qualquer uma «ao controlo» troca um cabeçalho e uma dobra por um campo.

**E a ALTURA já era uniforme (`22` em todos).** O que divergia era a **largura**, e num painel só:

| painel | x | largura |
|---|---:|---:|
| `inspector` (4 linhas) · `painter_layers` | `1206`/`1214` | `120` |
| **`vector` (fill · stroke)** | **`1300`** | **`32`** |

Medido no MESMO painel: os interruptores em `1207 + 125`, as pontas de seta em `1202 + 130`, os
botões em `1080 + 252` — **só a swatch começava `94 px` depois**. A largura vinha do
`SwatchSize::Md`, que o doc do `ColorSwatch` declara ser a aresta **sugerida** de uma amostra de
PALETA (*«callers may still hand any rect»*). *É o mesmo erro do `CHECKBOX_BOX_PX` no lugar da
altura da linha (§9-quindecies): duas grandezas com nomes parecidos, e a errada cabia.*

### §9-septdecies.2 — ⛔⛔⛔ A 1.ª sessão foi REVERTIDA, e por duas razões minhas

**(a) A lei já existia e eu não a li.** O gate `as_formas_de_um_selector_de_cor_so_encolhem` já
media **109** selectores com discriminador próprio (`is_picker_swatch` + `widget_color` + os estados
de picker), já trazia o report do dono e já prescrevia a porta na mensagem de erro. Eu escrevi um
gate NOVO — com catraca, censo de obsolescência e 4 de 4 mutações a sangrar — que era **uma segunda
resposta à mesma pergunta, e pior**. Apagado. *Antes de escrever uma régua, corra o portão: é ele
que mostra a que já existe, e não um `grep`.*

**(b) A catraca bloqueou a cura que ela própria prescrevia.** Ela contava **píxeis** e chamava-lhes
«formas». Medido nas duas tentativas de conversão do vetor:

| tentativa | largura no vetor | veredito |
|---|---:|---|
| coluna derivada no painel | `134` | forma NOVA ⇒ reprova |
| pela porta `paint_color_row` | `112` | forma NOVA ⇒ reprova |

⭐⭐ **O achado: a largura de um selector que «enche a coluna» NÃO É INVARIANTE — ela é função da
largura do PAINEL** (`inspector` `120`, `vector` `112`). *A condição de sucesso daquela catraca —
o conjunto encolher até um — era inalcançável por construção enquanto os painéis tivessem larguras
diferentes.* ⇒ geometria revertida, árvore verde: *não se shipa mudança visível que luta com um
gate aprovado, e não se afrouxa o gate para a mudança passar.*

### §9-septdecies.3 — A régua NOVA: a forma que o nome da antiga prometia

`um_seletor_de_cor_sozinho_na_fileira_ocupa_uma_caixa_estrutural` (substitui a de píxeis, com a
morte da premissa escrita no lugar dela).

⭐ **A partição é MEDIDA, não escolhida:** um selector que **partilha** a faixa de `y` com outros
controlos é uma grelha de paleta, uma célula de bloco (o per-corner, `2 × 2`), um ponto de cabeçalho
ou uma linha de lista — ali o chip é a forma certa. Um selector **sozinho** na fileira é um campo.

Para os sozinhos, o rect tem de ser uma de **duas** caixas, ambas derivadas da caixa de dentro do
painel e de **nenhum número escolhido**:

| caixa | quem a usa | como se deriva |
|---|---|---|
| **a coluna do controlo** | `inspector` · `painter_layers` · **`vector`** | `property_row::caixa_do_controlo` |
| **a fileira inteira** | `authored` (gerado por TABELA — a forma que o dono desenhou) | a própria caixa de dentro |

⚠️ **São duas porque há dois MODELOS de linha** (o nome à esquerda · o nome na linha de cima), e não
por tolerância. O que a régua recusa é a terceira coisa: **uma largura FIXA, que não sai de
estrutura nenhuma**.

⚠️ **A caixa de dentro deriva-se do controlo mais largo que NÃO é uma cor** — derivá-la do conjunto
todo seria circular no `authored`, cuja swatch É a coisa mais larga que ele desenha.

⛔ **Duas réguas construídas, medidas e REFUTADAS antes desta:** a **borda direita** (todos acabam
onde os vizinhos acabam — *e a swatch com o defeito também*: verde sobre ele) e **«não mais estreito
que o vizinho mais largo da coluna»** (reprovou os OITO, porque o vizinho mais largo de uma coluna é
um botão de largura inteira, que é legítimo — *uma estatística sobre população heterogénea mede a
variedade, não o defeito*).

### §9-septdecies.4 — A cura: uma porta, e as quatro cópias que ela apagou

O painel de vetor tinha **quatro** redacções da mesma montagem (rótulo + rect colado à direita +
`paint_color_swatch` + `register`), duas delas a desenhar ainda a **rachura do token** por cima.

- `property_row::caixa_do_controlo` — o rect que a fileira dá ao controlo, **sem pintar**, com
  **dois** leitores: a rachura do token e a própria régua. *Uma segunda aritmética para a mesma
  coluna diverge no dia em que a porta mudar, e então a rachura cai ao lado da swatch e a régua
  aprova o desalinhamento que existe para proibir.*
- `BodyCtx::colour_swatch_row_rect` — a fileira pela porta da casa, devolvendo também o rect.
- Os três sítios em linha passaram a delegar; **três blocos de imports ficaram órfãos** — o sinal de
  que a duplicação de facto saiu.

⚠️⚠️ **O `y` que a porta devolve é DESCARTADO, de propósito:** o passo dela é `ROW_H_PX +
control_gap_px()` (`25`) e o deste painel é `row_h + row_gap` (`26`). Misturar dois passos dentro de
uma secção é um defeito maior do que a largura que esta wave veio corrigir — *a porta decide a
GEOMETRIA da fileira; o painel continua dono do RITMO dele.*

⭐ **Medido depois:** `vector.fill_swatch` e `stroke_swatch` de `x=1300 w=32` para **`x=1760
w=112`** — a começar **exactamente** onde a linha de cor do Inspector começa (`x=1760`), com a
largura que a largura DAQUELE painel dá.

### §9-septdecies.5 — ⛔⛔ E o censo de ACESSIBILIDADE acusou os dois ficheiros CURADOS

O `hr12_widgets_a11y::every_widget_file_wires_a11y` reprovou o `paint_contour.rs` e o
`paint_sections_stroke.rs` — *sobre código melhor do que o de antes*. Eles deixaram de nomear
`paint_color_swatch` porque passaram a chamar a porta **que o chama**, e a cadeia ficou com um salto
a mais do que a lista de primitivos conhecia:

```
painel → BodyCtx::colour_swatch_row → property_row::paint_color_row
       → widget::paint_swatch_or_mixed → widget::paint_color_swatch   ← o primitivo
```

⭐ **O mecanismo já existia e nasceu do MESMO acidente:** as `PORTAS_DE_CRATE_VERIFICADAS` foram
criadas em 2026-09-19 quando o `ph2d-panel-audio-editor` adoptou a porta dele e três ficheiros
ficaram vermelhos na mesma condição. ⇒ duas linhas, **as duas necessárias**:

1. `paint_color_row` entra nos `WIDGET_DELEGATE_MARKERS` — *a lista dizia «keep in sync with
   `src/widget/`» e as portas de fileira vivem no `property_row`, que é da CASA e é canónico na
   mesma*;
2. `("colour_swatch_row", "ph2d-panel-vector", "src/paint_rows.rs")` entra nas portas verificadas.

⚠️⚠️ **E a (1) é o que torna a (2) HONESTA:** a verificação de uma porta exige que o ficheiro dela
contenha um marcador, e o `paint_rows.rs` contém `paint_button` de **outra** função — *um casamento
acidental*, que é exactamente o defeito que aquele ficheiro já regista sobre este mesmo painel
(`paint_color_swatch_row` a casar por subcadeia). Com a (1) o marcador que casa é a delegação REAL.

*Uma lista de primitivos que não acompanha as portas que a casa cria acusa precisamente quem as
adopta* — e o sinal é inconfundível: o gate reprova ficheiros que o diff **melhorou**.

### §9-septdecies.6 — Prova de mutação: **7 sangram + 1 NO-OP nomeada**

| # | mutação | veredito |
|---|---|---|
| M1 | o vetor volta ao chip fixo | ✅ sangra |
| M2 | a alternativa da FILEIRA INTEIRA desaparece | ✅ sangra (o `authored` acusa) |
| M3 | a partição SOZINHO-NA-FILEIRA desaparece | ✅ sangra (as grelhas acusam) |
| M4 | o censo deixa de colher (piso de população) | ✅ sangra |
| M5 | `caixa_do_controlo` larga o `property_fields_layout` | ⛔ **NO-OP, medido** |
| M5' | `caixa_do_controlo` devolve a FILEIRA em vez da coluna | ✅ sangra |
| M6 | a porta registada deixa de delegar | ⛔ **não compila** — o arnês abortou |
| M6' | a entrada nomeia uma porta que não existe | ✅ sangra (o censo de obsolescência) |
| M7 | a entrada some ⇒ os dois ficheiros ficam sem cobertura | ✅ sangra |

⚠️⚠️ **A M5 lê-se como sobrevivente e não é:** com **um** campo o `property_fields_layout` devolve a
coluna inteira (`control.w=120 cw=120` · `control.w=112 cw=112`, medido por sonda). *Uma mutação
no-op e uma sobrevivente dão exactamente o mesmo relatório* — a única maneira de as separar é medir
a grandeza que a mutação devia mover.

## §9-octodecies — ⭐⭐⭐ DEZOITO DECLARAÇÕES DE «A ALTURA», EM TRÊS VALORES, TODAS A DIZER QUE CONCORDAM

> Report do dono, 2026-09-21: *«várias seções muito confusas e desorganizadas»* + *«quanto ao
> alinhamento precisamos melhorar em todos os lugares»*.

### §9-octodecies.1 — A sonda que devia ter existido há três waves

`diag_o_ritmo_de_cada_seccao_do_inspector` arma **uma** secção de cada vez (as `PORTAS` do
`o_inspector_armado`) e imprime o histograma de alturas de linha que ela pinta, com os ids fora do
padrão nomeados. **38 secções medidas.**

Ela separa de uma vez o que é legítimo do que não é:

| altura | quem | veredito |
|---:|---|---|
| `22` | a esmagadora maioria | o padrão da casa (`ROW_H_PX`) |
| `24 × 2` | **toda** secção | a banda do cabeçalho + o chevron — não é fileira |
| `18 × 1` | 18 secções | o **ponto de cor** do cabeçalho — não é fileira |
| **`30`** | 12 secções | os **BOTÕES de acção** (`insp_*_add`, `_remove`, `insp_add_component`…) |
| **`24`** em campos | `emissive_row` · `slice_nine` | **o defeito** |
| `26 × 11` | `slice` | o lado de uma célula da **grelha 3×3** — outra população, fica |

### §9-octodecies.2 — ⛔⛔⛔ O que o fonte tinha por baixo

| grandeza | declarações | valores | o que o comentário de cada uma afirmava |
|---|---:|---|---|
| altura de BOTÃO (`BTN_H`) | **15** | `30` em todas | *«igual à das irmãs»* |
| altura de CAMPO (`FIELD_H`) | **3** | **`24` · `24` · `22`** | *«a altura de campo do Inspector»* |

⚠️⚠️ **A segunda linha é o defeito a sério:** três ficheiros diziam a MESMA frase sobre três números
que não eram o mesmo, e *a resposta que o artista via era a do ficheiro em que ele calhava de estar
a olhar*.

⭐ **E a primeira é COMO a segunda nasce.** Quinze cópias de `30.0` com um comentário a afirmar que
concordam é exactamente a forma que o `CHECKBOX_BOX_PX = 18` pagou em 21/09 (cinco cópias, **uma**
curada, e a frase das outras quatro ficou falsa sem nada deixar de compilar) e que o `SwatchSize::Md`
pagou em 22/09. *Uma frase de comentário não é uma lei: só uma PORTA é* — e esta é a **terceira**
ocorrência da mesma família em três dias.

### §9-octodecies.3 — A cura: duas portas, dezoito declarações a menos

Em `sections/mod.rs`:

- **`ALTURA_DE_BOTAO`** (`30`) — a ÚNICA declaração. ⚠️ Ela é declaradamente **maior** que uma
  fileira: *um botão de acção não é um campo*, e isso agora está escrito onde só havia um literal
  repetido.
- **`ALTURA_DE_CAMPO`** — **delega** em `ph2d_tokens::ROW_H_PX`, para não haver uma terceira
  resposta. Os dois `24` passaram a `22`.

**Medido pela sonda, antes → depois:** `sprite` `24×4 → 24×2` (com `22×27 → 22×29`) e `slice`
`24×4 → 24×2` (com `22×9 → 22×11`) — **quatro campos alinhados**, e o `24` que resta em toda secção
é a banda do cabeçalho.

### §9-octodecies.4 — A régua, e a 1.ª redacção que acusou código CERTO

`nenhuma_seccao_declara_a_propria_altura_de_linha` (no `ph2d-panel-inspector`, com piso de
população e controlo).

⛔⛔ **A 1.ª redacção proibia a DECLARAÇÃO e reprovou nove secções sobre código correcto:** elas
escrevem `const ROW_H: f32 = ph2d_tokens::ROW_H_PX;` — um **alias que DELEGA**, e um alias não pode
divergir, ele muda com a porta. ⇒ a régua passou a medir **o LITERAL**: `const <NOME>: f32 =
<dígito>`.

*Uma régua que mede a FORMA da linha em vez do que ela pode PARTIR acusa quem já está certo* — foi a
terceira vez hoje que eu paguei esta (a borda direita · o vizinho mais largo · esta).

⭐ **E o controlo tem DUAS metades**, porque as duas podem falhar: a régua reconhece a cópia
(`const BTN_H: f32 = 30.0`) **e** não acusa o alias que delega. Sem a segunda ela voltaria a
reprovar as nove.

⚠️ **A régua é TEXTUAL de propósito:** o censo do produto mede o que é PINTADO e não vê uma
constante que ainda não tem consumidor — *e uma cópia nasce sempre sem consumidor, no commit antes
daquele em que ela diverge*.

### §9-octodecies.5 — Prova de mutação: **4 de 4 sangram**

| # | mutação | veredito |
|---|---|---|
| M1 | uma secção volta a declarar a altura de botão | ✅ sangra |
| M2 | uma secção volta a fixar a altura de campo em `24` | ✅ sangra |
| M3 | a varredura deixa de ler ficheiros (piso de população) | ✅ sangra |
| M4 | a régua passa a acusar um alias que delega | ✅ sangra (o controlo) |

### §9-octodecies.5-bis — ⛔ E o tecto de LOC mordeu, com a causa a ser o NOME

O `script.rs` passou o tecto de `600` por **UMA** linha: os nomes das portas
(`ALTURA_DE_CAMPO`/`ALTURA_DE_BOTAO`) são mais longos que os literais que substituíram, e o
`rustfmt` quebrou duas chamadas `Rect::new` em cinco linhas cada.

⇒ **corte por RESPONSABILIDADE**, com precedente na própria crate: `script_avisos.rs`, irmão do
`particles_avisos.rs`, cortado pela mesma razão. `script.rs` `601 → 449`, o irmão `169`.
⛔ **Nunca uma entrada no `FILE_OVERAGE_OK`**, e nunca um alias local — este último re-introduziria
exactamente a forma que a wave veio apagar.

### §9-octodecies.6 — ⏳ O que fica ABERTO, com o número

**Os botões de acção medem `30` e os campos medem `22`.** A porta tornou o número único e
declarado; **se os dois devem ser iguais é decisão de PRODUTO**, e o dono tem agora a pergunta com o
número ao lado. ⛔ Eu não a decidi sozinho: mudar a altura de um botão é visível em 12 secções.


## §9-novendecies — ⭐⭐⭐ A MESMA FRASE EM VINTE E UMA SECÇÕES, POR SEIS PINTORES, E ONZE DECLARAÇÕES DE «PINTAR UM AVISO»

**Report do dono, 2026-09-21:** *«vários componentes cheios de mensagens»* + *«várias seções muito
confusas e desorganizadas»* + *«quanto ao alinhamento precisamos melhorar em todos os lugares»*.
**Ordem de 2026-09-22:** *«faça ciclos de implementações maIORES»*.

### §9-novendecies.1 — O instrumento que não existia

Esta casa tinha censo de TEXTO (*«vem da tabela?»*), censo de ELISÕES (*«coube?»*), censo de IDS
(*«é alcançável?»*) e gates de costura (*«o clique chega?»*) — e **nenhum** perguntava
***quantas frases o painel escreve de uma vez***.

⛔⛔ **E o censo das elisões não o podia responder, pela LEI dele:** um aviso é uma FRASE e ela
**quebra** em vez de cortar, logo nunca passa pela lei da reticência e **não deixa rasto**. *Um
censo cego àquilo de que o dono se queixa lê-se como um painel limpo.*

⇒ [`ph2d_panel_inspector::censo_dos_avisos`] (o molde do censo das elisões: `thread_local`, nasce
desarmado, `#[track_caller]` para a LOCALIZAÇÃO — *uma régua que diz «esta frase saiu 21 vezes» e
não diz de ONDE obriga a arqueologia*), com a sonda e os gates em
`quantas_mensagens_o_painel_escreve.rs`.

⚠️⚠️ **A fixtura não continha o fenómeno.** A `o_inspector_armado::arma_tudo` declarava
`selected_count: 1` em **28** sítios e a frase é gateada em `> 1` ⇒ *com a fixtura pregada em `1`
nenhuma régua desta casa jamais a via*. Daí a `com_seleccao_de`.

### §9-novendecies.2 — O que a medição achou

| | |
|---|---:|
| secções que pintavam *«estás a editar só a primária»* | **21** |
| redacções distintas da mesma frase | **4** |
| pintores distintos que a escreviam | **6** |
| **cópias IDÊNTICAS no MESMO quadro** (medido, 2 objectos) | **5** (+1 a dizê-lo por outras palavras) |
| declarações de *«pintar uma linha de aviso»* | **11** (a porta + 7 fn + 3 fechos) |
| das quais **byte a byte iguais** e **erradas do mesmo modo** | **10** |

⛔⛔⛔ **As dez cópias chamavam o `paint_text` (que CORTA) em vez do `paint_text_block` (que
QUEBRA) e devolviam UMA linha de altura qualquer que fosse a frase** — *os dois defeitos exactos
que o gate da porta (`um_aviso_quebra_e_o_pintor_de_rotulo_corta`) existe para impedir, num sítio
onde ele nunca olhou*.

⚠️ **É a QUARTA e a QUINTA ocorrência da mesma forma em três dias** — `CHECKBOX_BOX_PX = 18` (21/09)
· `SwatchSize::Md` como largura de linha (22/09) · as dezoito alturas (22/09) · **uma FRASE** ·
**um PINTOR**.

### §9-novendecies.3 — A cura

1. **O facto da selecção passa a ter UMA fonte** — `set_current_inspector_selecionados`, escrito
   uma vez por quadro do único sítio da shell onde `hero.gizmo.selected_len()` é lido, e pintado
   **uma vez** pelo cartão do topo ([`paint_cards::paint_selection_card`], ANTES dos outros dois:
   *«estás a editar uma de N»* muda o significado de tudo o que vem abaixo). **21 blocos apagados,
   21 chaves de i18n órfãs apagadas**, e a frase passa a levar **o NÚMERO** — que com vinte e uma
   cópias ninguém lhe punha.
   ⚠️ Os **24** campos `selected_count` por-secção FICAM: eles respondem *«este valor é MISTO?»*,
   que é outra pergunta.
2. **Os 10 pintores copiados morrem** — tudo passa por `sections::rows::aviso`. ⭐ De graça, **duas
   frases longas que eram CORTADAS passam a quebrar e a ler-se inteiras** (as duas linhas de dívida
   que o diziam por escrito — *«a cura é a frase QUEBRAR — outra wave»* — saíram **pagas**, e foi o
   censo de obsolescência a dizê-lo).
3. **A RENDER SOURCE fala a língua da casa** — ela punha o nome POR CIMA do valor (a única secção
   que o fazia) e tinha **altura de linha própria** (`Sm + vão = 15` contra `ROW_H_PX = 22`), logo
   nada nela alinhava com nada. Hoje é linha de propriedade, com a coluna medida sobre os DOIS nomes.

### §9-novendecies.4 — ⛔⛔ O gate que proibia isso NUNCA a viu, e o limite estava escrito nele

O `no_row_paints_its_name_above_its_control` procura o idioma pelo **nome da variável** (`label_h`)
e a `render_source.rs` escrevia `label_font + row_gap` ⇒ **verde durante uma semana sobre a secção
que o dono FOTOGRAFOU**. O doc-comment dele já dizia: *«uma secção que empilhe por outro caminho e
com outro nome continua invisível a uma régua textual»*. O detector aprendeu o segundo nome, com o
controlo positivo dos dois e o negativo da SUBTRACÇÃO (que CENTRA um texto e não empilha).

### §9-novendecies.5 — O preço da cura 3, nomeado

Com a linha alinhada o valor deixa de ter a largura do painel e passa a ter a da coluna
(`112 px`) ⇒ `Hand-packed · hero · idle_0` **é cortado**. ⚠️ O nome da folha é texto do **ARTISTA**:
encurtá-lo não é saída, e a lei da casa é o **BALÃO** — que o pintor passou a declarar
(`text_elide::balao::na_area`). A linha entra na dívida com o mecanismo; as duas catracas do
Inspector **descem** (`89 → 86`, `84 → 81`).

### §9-novendecies.6 — ⛔ Três gates partidos por MOVER código, e os três falharam ALTO

- `every_word…::cada_letra_solta` lia as quatro células da região no `render_source.rs`, e o bloco
  saiu para o irmão pelo tecto de LOC (curado por **CORTE POR RESPONSABILIDADE**, nomeado pelo
  próprio ficheiro-pai antes de ser feito).
- `o_inspector_armado` (×3) presumia que **todo** `set_current_inspector_*` é uma porta de SECÇÃO,
  com semântica de `Option`. Chegou o primeiro que não é (um FACTO DO PAINEL) e os três reprovaram
  sobre produto CERTO, um com o nome recortado a meio (`"selecionados(0);\n    "`).
  ⭐ **A cura é DERIVADA da assinatura e não uma excepção à mão:** *uma secção pode estar ausente,
  logo o setter dela aceita `None`*.
- `seam_anim` e `o_tutorial_nomeia_rotulos…` tinham a **premissa morta** e foram reescritos com a
  morte à vista no diff (o primeiro afirma hoje as duas metades: a secção **não** avisa · o painel
  avisa).

⚠️ **Limite NOMEADO do gate do tutorial:** ele lê o HTML **cru**, logo uma entidade (`&middot;`)
nunca casa o texto da tabela.

### §9-novendecies.7 — Portão

`nextest-impacted` **17 666**, com **uma** vermelha: `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
(`ph2d-tool-painter`). ⚠️ **Promoção pedida à família de flakes de fan-out do §5.0** — as três
assinaturas: **zero linhas do diff desta wave naquela crate** · **3 de 3 verde sozinho a
`load 45`–`63`**, que é MAIS carga do que a do fan-out · e é um **gate de RAZÃO entre dois
relógios**, cujo doc se declara curado da flake (*«medidos juntos, os dois números sobem e descem
juntos»*) — ⚠️ **verdade sobre o PERFIL e falso sobre o FAN-OUT**, que é a distinção que aquela
lista existe para guardar.

Clippy `-D warnings` zero · `cargo fmt --all --check` limpo · tectos de LOC e `the_shell_only_shrinks`
verdes · **6 de 6 mutações sangram** (com os três controlos do arnês: a agulha casa **uma** vez
contada em OCORRÊNCIAS, a mutação **compila**, e a corrida teve população não-nula).


## §9-vicies — ⭐⭐⭐ AS SECÇÕES DO INSPECTOR SAEM POR FAMÍLIA, E A ORDEM É **DERIVADA** DA PALETA

**Report do dono, 2026-09-21:** *«várias seções muito confusas e desorganizadas»*.

### §9-vicies.1 — A medição, antes da primeira linha

O Inspector pinta **37 secções** em `14 857 px`. Medido pelo `y` **PINTADO** (sonda
`diag_em_que_ordem_as_seccoes_aparecem`, nova e versionada), a sequência de índices de família das
**24 opcionais** lia:

```
11, 11, 12, 13, 11, 11, 9, 9, 11, 9, 11, 11, 14, 3, 11, 11, 11, 11, 11, 13, 13, 0, …
```

— ou seja, **a ordem em que elas foram CONSTRUÍDAS**, wave a wave, ao longo de quatro meses. A
`Tags`, que **todo** objecto pode ter, era a **última**, a `14 785 px` do topo.

⚠️ **A ordem lê-se do `y` e nunca de uma tabela:** o painel tem quatro orquestradores
(`paint_core_sections` · `paint_sprite_sections` · `paint_shared_sections` · as opcionais), e uma
tabela de declaração não diz em que ordem eles correm.

### §9-vicies.2 — A ordem não é escolhida: ela já existia no catálogo

O [`ph2d_component_desc::ComponentCategory::ALL`] declara **16 famílias, por ordem**, e é por elas
que a paleta do *Add Component* **já agrupa**. ⇒ *uma tabela com dois consumidores e só um a lê-la*.

⛔⛔ **E a `ids::LIVE_SECTIONS` NÃO podia ser a fonte:** ela é um **ÍNDICE DE ARMAZENAMENTO** — as
notas por secção indexam-na por POSIÇÃO, e o cabeçalho dela di-lo por escrito (*«mover uma entrada
renumera-as»*). *A ordem de ARMAZENAMENTO e a de LEITURA são duas coisas, e o nome não as separa.*

⇒ [`crate::paint_familias`](../../../crates/ph2d-panel-inspector/src/paint_familias.rs), com quatro
orquestradores por família (FÍSICA · LÓGICA · LÓGICA-2 · SAÍDA) e as quatro primeiras chamadas
(IDENTIDADE · RENDERING · ANIMAÇÃO · ÂNCORAS) no `paint_optional.rs`. A partição
`…_logica` / `…_logica_cont` é **só o tecto de LOC por função**, e está dito no ficheiro.

**A ordem depois:** `Tags` sobe de `14 785` para **`6 708 px`**, e a sequência passa a ser monótona
nas famílias.

### §9-vicies.3 — O gate, e o que ele deliberadamente NÃO afirma

[`a_ordem_das_seccoes_e_a_da_paleta`](../../../crates/ph2d-panel-registry-init/tests/it/a_ordem_das_seccoes_e_a_da_paleta.rs)
lê as bandas pintadas pela porta do produto, mapeia cada secção ao componente que ela edita
(⛔ a 2.ª coluna é **verificada contra o catálogo**: um nome que ele não conheça reprova) e exige
que os índices de família saiam **não-decrescentes**, com `PISO = 20`.

⚠️ **MUTAÇÃO NOMEADA, com a medição:** trocar o `paint_projectile_section` com o
`paint_topdown_section` — dois blocos completos, **da mesma família** — deixa-o **VERDE**. É o
desenho: a ordem intra-família é a de construção, e a única fonte de que ela poderia ser derivada é
a própria cadeia de chamadas — *afirmá-la seria copiar o código sob teste para dentro do teste*.
O que os gates defendem é o **AGRUPAMENTO**, que é o que o report nomeia.

### §9-vicies.4 — ⛔⛔ O `y` que se deita fora, outra vez — e desta vez fui eu

A chamada da `Tags` era a **última** do bloco antigo, logo uma **expressão de cauda sem
atribuição**. Ao movê-la para o meio da cadeia, o `;` que lhe acrescentei apagou o avanço: a `Tags`
e os `Timers` passaram a desenhar-se **na mesma banda** (`y 280,0..302,0`).

⭐ **Quem o apanhou foi o gate que existe exactamente para isso** — o
`two_live_sections_never_share_a_band`, escrito em 2026-09-09 sobre o mesmo defeito na wave do
`SignalActions`. *É a quarta ocorrência da forma nesta crate, e a primeira em que o instrumento já
lá estava.*

### §9-vicies.5 — ⛔⛔⛔ E o gate IRMÃO ficou a medir a ordem de ONTEM

O `each_section_starts_below_the_one_before_it` compara as bandas **na ordem de uma lista escrita à
mão**, e essa lista era `Transform · Timers · Signal Actions · Camera · Tags`. Com a ordem por
famílias ela deixou de descrever o painel.

⚠️⚠️ **E a fixtura perdeu poder sem reprovar:** o doc da `camera()` dizia *«sem ela a TAGS não tem
nada por cima»* — a `Tags` é hoje a **primeira**, logo a mutação *«deitar fora o `y` da câmera»*
passou a ser um **no-op**, porque nada armado era pintado depois dela. ⇒ a fixtura ganhou a secção
**SHAKE** (a primeira armável abaixo da câmera na família SAÍDA), e a mutação volta a **SANGRAR**.

⭐ *Uma reordenação não parte só a lista de um gate: ela troca QUEM está abaixo de quem, e uma
fixtura calibrada na cadeia antiga fica verde a afirmar menos do que prometia.*

⚠️ A lista fica **escrita à mão de propósito** e isso **não** é a segunda resposta à mesma pergunta:
a ordem é DERIVADA pelo gate do registo sobre as 24 opcionais, e esta é o **CONTROLO** dele na
crate do próprio painel, sem o arnês do registo. Se as duas discordarem, uma fica vermelha em voz
alta. *O que esta cobre e aquela não é o bloco FIXO contra o opcional.*

### §9-vicies.6 — ⛔⛔ Os OUTROS três vermelhos do portão, e um deles foi um palpite meu

| vermelho | causa | cura |
|---|---|---|
| `staleness::cargo_deps_in_sync_with_folder` | pus o `ph2d-component-desc` **DENTRO** dos marcadores `# <ph2d-panel-sync:deps:begin/end>`, que são **gerados** | a linha subiu para fora do bloco |
| `hr12_widgets_a11y` | o `paint_familias.rs` é painel e não fia a11y | entrada no `PANEL_A11Y_DELEGATE_OK` — **verificada**: `0` ocorrências de `NodeId` / `hit_index.` / `register(` |
| `nenhum_rotulo_do_app_pinta_nada::a_divida_do_degrau_estreito_so_encolhe` | `declarado 86, mede 85` | catraca a `85` |

⛔⛔ **E a 1.ª redacção da nota da catraca era um PALPITE com cara de medição:** eu escrevi que o
corte saíra por a ordem das secções ter mudado de família (*«uma secção que sobe fica debaixo de um
cabeçalho com outro recuo»*) — **falso**: reordenar não muda a largura de coluna de ninguém.

⭐ **Atribuído por A/B:** repondo o texto longo `"Repeat (0 = forever)"` a catraca lê **`86`** e
**nomeia-o na lista impressa**; com o curto `"Repeat"` lê `85`. O corte que saiu é a §11 Animation a
perder a regra de dentro do nome — a lei *«um nome perde a EXPLICAÇÃO antes de perder LETRAS»*
aplicada ao último rótulo que ainda a carregava.

⚠️ *Uma catraca que desce sem a atribuição medida é uma licença com um comentário bonito ao lado.*

### §9-vicies.7 — O portão

`cargo fmt --all` limpo · clippy `-D warnings` zero · a varredura impactada verde ·
**4 mutações sangram + 1 NOMEADA** (com os três controlos do arnês: a agulha casa **uma** vez
contada em OCORRÊNCIAS, a mutação **compila**, e a corrida teve população não-nula —
`passed + failed`, nunca o `running N` que conta os `#[ignore]`).

⚠️ **E a corrida das elisões vai a `--workspace`**, nunca `-p`: o `flip`, o `painter_layers` e o
`wet_tuning` não estão no `default` do `ph2d-panel-registry-init`, e num âmbito pobre a catraca lê
`0` como *«este painel não corta»*.


## §9-vicies-semel — ⭐⭐⭐ A LINHA DE COR PASSA PELA PORTA, E A RÉGUA SEPARA UMA FILEIRA DE UMA PALETA

**Report do dono, 2026-09-21:** *«os seletores de cor de todo o app precisam ser padronizados»*,
com um **desenho** ao lado — a amostra como uma **barra** na coluna do valor, e não o quadradinho
encostado à direita.

### §9-vicies-semel.1 — O que já existia, e o que faltava

A porta [`property_row::paint_color_row`](../../../crates/ph2d-editor-core/src/property_row.rs)
nasceu nesse dia, com a medição no doc dela: **`109` selectores em CINCO larguras** (`18`, `24`,
`32`, `120`, `268 px`). Dois painéis passaram por ela (Inspector, Vector); os outros continuaram a
montar a linha à mão.

Medido agora: **`16` ficheiros** pintam uma amostra com `paint_color_swatch` / `paint_swatch_or_mixed`.

### §9-vicies-semel.2 — ⛔ Nem toda amostra é uma FILEIRA, e forçá-las todas seria o erro

| sítio | o que é | veredito |
|---|---|---|
| `flip` ×4 (Fill · Stroke · Fill-section · Colorize) | rótulo + quadrado de `SwatchSize::Md` encostado a `inner_x + inner_w − w` | ⭐ **fileira — convertida** |
| `bgremoval` · `painter-layers/paint_mask` | **PALETA** (`flow_fixed(n, SWATCH_PX, …)`) | fica |
| `hierarchy/row` | **ETIQUETA** de uma linha de lista (`INLINE_ICON_PX`) | fica |
| `painter-layers/paint_ramp_widget` | linha de um **EDITOR RICO** (índice · posição · cor) | fica |
| `painter-layers/paint_shape_layers` | linha de **LISTA** de camadas | fica |
| `tokens/paint` · `vector/paint_stack_rows` · `model3d/paint_rows_swatch` | linha de **TABELA** com aritmética própria | ⏳ aberto |
| `inspector/sections/color_tint` | **GRELHA 2×2** dentro da coluna do valor | fica, declarado |

⭐ *Forçar uma paleta pela porta trocaria uma grelha por um campo.*

### §9-vicies-semel.3 — O discriminador é DERIVADO, não uma lista de painéis

⭐⭐⭐ A pergunta que separa as duas populações é ***«ele pinta um RÓTULO DE PROPRIEDADE no mesmo
fôlego?»***: um ficheiro que chama `paint_property_label(` **e** desenha a amostra à mão está a
escrever uma linha de propriedade com **uma segunda aritmética de colunas**. Uma paleta não chama
rótulo nenhum.

⇒ [`architecture_color_rows_use_the_door`](../../../crates/ph2d-editor-core/tests/it/architecture_color_rows_use_the_door.rs),
com as quatro metades que esta casa cobra: a acusação · a **obsolescência** do `FORA` (cada entrada
tem de continuar a descrever um ficheiro que a régua acusaria) · o **piso de população** (`12`, de
`16` medidos) · e o **controlo** nos dois sentidos (ele vê a forma que existe para acusar, e não a
inventa numa paleta).

⚠️ **O `FORA` tem DUAS entradas e as duas são verificadas:** a própria porta (ela chama o
`paint_swatch_or_mixed` por dentro — *uma excepção que não estivesse lá faria a porta reprovar o
gate que ela existe para impor*) e o tingimento por canto, que **já** lê `row.label`/`row.control`
do `property_row_columns`: a coluna dele é a mesma; o que ele não pode é colapsar quatro cantos
numa barra.

⚠️ **E a régua diz o que NÃO vê:** uma linha de propriedade cujo rótulo seja escrito com
`paint_text` à mão passa-lhe ao lado. *Ela mede a DUPLICAÇÃO da aritmética, não a existência da
linha* — as outras metades têm censos próprios.

### §9-vicies-semel.4 — ⭐⭐ O que a conversão revelou de graça

**(a) Um campo do contexto ficou ÓRFÃO.** O `BodyCtx::font` do Flip tinha **um** leitor no painel
inteiro — o `y + (row_h − font) * 0.5` do rótulo escrito à mão. Com as quatro linhas pela porta ele
deixou de ser lido, e quem o apanhou foi o `-D warnings`. *Um contexto que carrega o que já ninguém
pergunta é a sombra de uma aritmética que se mudou.*

**(b) Três chaves de i18n ficaram órfãs** (`panel.flip.tool.fill_color` · `…stroke_color` ·
`panel.flip.colorize.colorize_color`). Elas eram o **nome acessível** da amostra, e a porta tira-o
do **rótulo da própria linha** — ou seja, do que está na tela. ⚠️ Isto **não** é o caso do
tingimento por canto, onde as quatro amostras partilham um rótulo visível e por isso precisam de
nomes próprios (há gate a dizê-lo). Apanhou-as o `every_key_of_this_panel_exists_on_both_sides`.

### §9-vicies-semel.5 — ⏳ O que fica ABERTO, com o mecanismo

- **Três linhas de TABELA** (`tokens` · `vector/paint_stack_rows` · `model3d/paint_rows_swatch`)
  têm aritmética de colunas própria e **não** chamam `paint_property_label`, logo a régua não as
  acusa. A do `model3d` passa por um `rotulo_e_goteira` que é a versão dela da porta — convertê-la
  é uma wave, não uma linha.
- ⛔⛔ **A sonda das cores é CEGA a um id COMPUTADO, e agora DI-LO.** O `tokens_swatch_id(row)`, o
  `vector_paint_swatch_id(i)` e o `painter_shape_layer_color_swatch_id` não são literais, logo o
  mapa inverso não os sabe nomear e ela deixava-os cair **em silêncio**. Hoje conta-os e imprime
  uma linha `ANONIMO` por painel — e o número é grande (`35` no `color_equalization`, `39` no
  `grid_snap`, `32` na Hierarquia). ⚠️ **Esse total é do painel INTEIRO e não só das cores**: a
  sonda filtra por NOME, logo um selector ali dentro é invisível para ela. *Uma varredura que
  ignora o que não sabe nomear mede o alcance do NOME, não o do produto* — e a régua do §9-vicies-
  semel.3, que é TEXTUAL sobre o fonte, não tem essa cegueira.

### §9-vicies-semel.6 — O portão

`cargo fmt --all --check` limpo · clippy `-D warnings` zero nas crates tocadas · varredura impactada
verde · censo das elisões a `--workspace` `13/13` · **2 mutações sangram** (o Flip de volta ao par
rótulo + amostra · uma entrada do `FORA` a apontar para um ficheiro que não existe), com os três
controlos do arnês.


## §9-vicies-bis — ⭐⭐⭐ A ESCOLHA COM NOME TEM UMA PORTA, E A FORMA DELA SAI DE UMA MEDIDA

**Ordens do dono (2026-09-21):** *«várias seções muito confusas e desorganizadas»* · *«quanto ao
alinhamento precisamos melhorar em todos os lugares»* — e, na mesma mensagem, as dicas vão para o
rato e os nomes encolhem.

### §9-vicies-bis.1 — O que havia: OITO implementações de «uma escolha com nome»

Medido pela porta do produto (a varredura geométrica nova, `nenhum_nome_por_cima_do_controlo`):
o Inspector tinha `rows::seg_row` (nome ao lado), `anim::segmented_row`, `topdown::seg_row`,
`particles::seg_row`, `hud::seg_row`, `tween_editor::grupo` (nome POR CIMA) e quatro inline
(`Strategy`, `Format`, os dois segmentados das Acções — **sem nome nenhum**), mais o `Sort Point`
(um `Tabs` sem nome) e o `Where` da Fábrica (três `Button` em partes iguais). ⛔⛔ O gate TEXTUAL
`no_row_paints_its_name_above_its_control` era cego a esta família **três** vezes pelo NOME da
variável (`label_h` · `label_font` · `font`) — *uma régua que depende de como o autor chamou a
variável mede o autor, não o painel*. ⇒ a régua nova pergunta ao PRODUTO: pinta o painel pela porta
do registo e acusa todo grupo segmentado declarado (`widget::composto`) que COMEÇA na borda
esquerda do conteúdo — onde devia estar um nome.

### §9-vicies-bis.2 — A porta: [`property_row::paint_choice_row`](../../../crates/ph2d-editor-core/src/property_row/escolha.rs)

⭐⭐ **A forma sai de uma MEDIÇÃO e vive só ali:** ao lado do nome, na coluna do valor (~`128 px`
no dock de omissão), **só `4` das `22` escolhas cabem numa fileira**; `9` ocupam duas, `4` três,
`2` quatro, e as famílias de easing e os canais do Tween ocupariam **seis e sete**. ⛔ *«Sempre ao
lado»* faria torres. ⇒ **cabe numa fileira → ao lado, alinhada com os campos da secção; não cabe →
PALETA** (o nome é o cabeçalho, o grupo a toda a largura). ⚠️ **Nenhum chamador escolhe a forma** —
se o dono preferir um menu suspenso ao lado do nome para as paletas, a troca é nessa função e
nenhuma secção muda (é a pergunta de produto que vai com o smoke).

⭐ **Cada secção convertida passou a ter UMA coluna do nome** — as escolhas entram na `Seccao::medida`
da secção (`seccao_da_animacao`, `seccao_do_tween`, `seccao_do_percurso`, `seccao_da_acao`,
`seccao_do_render`, e as de `hud`/`particles`/`shake`/`shake_emitter`/`ordering`/`factory`). As
Acções ganharam **quatro nomes** que não tinham (`Target By` · `Source` · `Do` · `Tag`), a ordenação
ganhou `Sort Point`, e a `Strategy`/`Format` do Render deixaram de pintar o nome por cima — ⚠️ com a
**excepção declarada** da textura cozida, onde o que vem por baixo não é um controlo e sim a FRASE
que explica porque não há controlo.

### §9-vicies-bis.3 — O que morreu, com a premissa à vista

- ⛔ `tween_editor::cabem_por_fileira` (exportada como `chips_por_fileira`) e a `CHIPS_POR_FILEIRA`:
  a régua LOCAL de quantos chips cabiam. A porta usa as larguras NATURAIS do grupo adaptativo, logo
  *«quantos cabem»* deixou de ser um número da secção. O teste `nenhum_chip_do_tween_sai_cortado`
  perdeu a metade da CONTAGEM e a escada de larguras (as duas mediam a régua morta) e ficou com a
  que nunca dependeu dela — **o rectângulo PINTADO cabe o rótulo PINTADO** — mais o controlo da
  foto do dono (o `4` fixo reproduz os três cortes).
- ⛔ `paint_segmented_group_adaptive` saiu do `use` do Inspector: **nenhuma secção o chama
  directamente**; o único segmentado com nome que fica fora da porta é o `rows::seg_row`, que é
  OUTRO widget (`SegmentedAdaptive`, com id de grupo e estado «misto»), sempre ao lado, e foi
  aprovado assim pelo dono em 15/09.

### §9-vicies-bis.4 — Os gates

| gate | afirma |
|---|---|
| `nenhuma_escolha_do_inspector_e_montada_a_mao` | todo grupo a toda a largura do Inspector vem da porta; exceção NOMEADA com censo de obsolescência (`insp_vis_layer_bit_0` — a grelha de 32 bits é um MAPA de bits, não um-entre-N); piso de `20` pela porta |
| `os_outros_paineis_so_encolhem_nas_escolhas_a_mao` | catraca por painel, nos dois sentidos: `model3d 7 · physics 3 · sculpt3d 11 · skeleton 1 · upscale 1 · vector 9` |
| `a_forma_da_escolha_sai_da_medida` | `ao_lado ⇔ fileiras ≤ 1` sobre o Inspector armado, com piso nas DUAS formas |
| `o_chip_pintado_cabe_o_rotulo_pintado` | pela rota, os canais do Tween não saem cortados |

**Mutações (arnês com os três controlos):** sempre ao lado **sangra** (2) · sempre paleta **sangra**
(a lei) · limiar `≤ 2` **sangra** (2) · a porta sem registo no censo **sangra** (2) · a paleta em
partes IGUAIS **sangra** (o do Tween). ⚠️ **Uma sobrevive e é NOMEADA:** espremer o rectângulo da
paleta a `30 %` não corta rótulo nenhum — o grupo adaptativo dá a cada peça a largura NATURAL e
TRANSBORDA o rectângulo em vez de cortar. *Um transbordo é outra pergunta* (o controlo sai da
coluna), e nenhuma régua desta wave a faz.

**Catracas que DESCERAM com a porta** (medidas no âmbito do app, `--workspace`): cortes do
Inspector `85 → 79` · letras perdidas `81 → 75` · carga de comandos `88 → 81` · e **duas linhas da
dívida armada APAGADAS pelo censo de obsolescência** (`Authored`/`Counter` do HUD, que em paleta
levam a largura natural). ⛔⛔ **E a minha catraca nasceu calibrada no âmbito POBRE** (`-p`): o
portão impactado apanhou `painter_layers` (só registado com a unificação de features do app) —
*a mesma forma que a varredura das elisões pagou em 20/09*. Hoje ela conta `painter_layers 1` e
**reprova alto** numa corrida `-p`, com o comando certo na mensagem. ⚠️ E o `widget/mod.rs` é
gerado (`ph2d-widget-sync` só conhece `mod`/`pub mod`): a conta das larguras naturais chega à porta
por **re-export** `pub(crate)`, nunca por um `pub(crate) mod` escrito à mão.

### §9-vicies-bis.5 — ⏳ ABERTO

- **Os outros painéis** (`32` grupos na catraca): o `sculpt3d` e o `model3d` são os maiores, e o
  `model3d` tem a lei própria da caixa única (§9-vicies-semel). Cada um é uma wave.
- **A pergunta do dono:** paleta com o nome por cima, ou menu suspenso ao lado do nome.
- ⚠️ **Este handoff passou do joelho** (`~150 KB`, contra `80`–`110` do §5.0): quem o integrar lê
  pelos `§` endereçáveis; cortá-lo com `scripts/doc-split.py` é trabalho de fecho da linha.


## §9-vicies-ter — ⭐⭐⭐ A CATRACA DOS OUTROS PAINÉIS FOI A ZERO: toda escolha do app tem nome ao lado ou é PALETA

**Ordem do dono (2026-09-23):** *«Smoke OK. siga em ciclos de implementação maiores»* — depois do
Inspector, os outros painéis.

### §9-vicies-ter.1 — Onde as `33` moravam, e as QUATRO portas que as serviam

Medido no âmbito do APP (`--workspace`, a guarda do §9-vicies-bis): as escolhas sem nome ao lado
vinham de **quatro** pintores partilhados, e cada um passou a delegar na porta da ESCOLHA:

| pintor | painéis | antes | cura |
|---|---|---|---|
| `ph2d_editor_core::panel::RowCtx::segmented` | Vector (9) · Esqueleto (1) | nome POR CIMA | delega na porta; coluna de omissão = a `label_col_w` do próprio `RowCtx`; troca o vão da porta pelo `row_gap` do painel |
| `ph2d-panel-sculpt3d` `widgets::labelled_seg` | Escultura (11, 23 chamadas) | nome POR CIMA numa faixa `Sm + Md` | idem, ritmo `Sm` do painel |
| `ph2d-panel-physics` `interact::seg_row` | Física (3) | nome POR CIMA | idem |
| `ph2d-panel-model3d` `paint_chips` | Modelo 3D (7) | **sem nome nenhum** | cada fileira ganhou um: `Lasso` (a nota por cima virou o nome) · `Create` · `Combine` · `Verb` · `Blend` · `Modifiers` · `Actions` |

Mais o `Algorithm` do Upscale (fileira sem nome). ⚠️ O id de GRUPO do `SegmentedAdaptive` saiu da
assinatura da Física e da Escultura (`4 + 23` chamadas, removido por contagem de parênteses com
`assert`): ele é da ACESSIBILIDADE, o `populate` regista-o, e nunca entrou na pintura — o
`paint_segmented_adaptive` pinta pela MESMA função que o grupo simples. O `widgets::seg` da Escultura
ficou sem chamador e **saiu**.

⚠️ O Modelo 3D tem ids CALCULADOS (`model3d_*_button(slot)`), logo a sonda imprime `#hash`: o mapa
foi medido com um teste temporário (apagado) — `select #1bb3…` · `add #fa9b…` · `op #97c1…` ·
`verb #f466…` · `character #59be…` · `mod #9c0e…` · `act #555d…`. *Quem voltar a esta sonda num
painel de ids calculados precisa do mesmo mapa.*

### §9-vicies-ter.2 — O gate

A catraca `os_outros_paineis_so_encolhem_nas_escolhas_a_mao` **chegou a zero e morreu**, e o gate do
Inspector virou `nenhuma_escolha_do_app_e_montada_a_mao`: todo grupo a toda a largura em todo painel
vem da porta, com **duas exceções NOMEADAS** e censo de obsolescência — a grelha de 32 bits do
Inspector (um mapa de bits) e as ABAS `Brush · Layers` do Painter (navegação entre vistas, pedida
pelo dono em 09/09). ⛔ **Guarda de âmbito:** numa corrida `-p` ele reprova ALTO a dizer o comando
certo, porque ali o `painter_layers` nem existe e a exceção leria-se obsoleta. Mutações **no âmbito
do app** (o arnês ganhou o modo `WS`): o Upscale de volta a um grupo sem nome **sangra** · a
exceção a apontar a um id inexistente **sangra**.

**Catraca que desceu:** altura de abertura `sculpt3d 2 097 → 2 021` · `vector 1 349 → 1 262` ·
`physics 1 293 → 1 281` — os nomes deixaram de gastar uma linha própria. Portão: nextest-impacted
**17 670/17 670** · clippy `-D warnings` nas 10 crates tocadas · fmt.

### §9-vicies-ter.3 — ⏳ ABERTO

- ✅ **DECIDIDO pelo dono (2026-09-23): a PALETA fica** (*«o formato atual»*) — o menu suspenso ao
  lado do nome foi posto e recusado. Registado no doc da porta `paint_choice_row`.
- `rows::seg_row` do Inspector segue fora da porta de propósito (widget com estado «misto»,
  aprovado em 15/09) — hoje é o único segmentado com nome que não passa por ela.


## §9-vicies-quater — ⭐⭐⭐ AS CORES QUE SOBRAVAM: duas FORMAS e não uma, e a barra aberta traça o anel

Ordem do dono de 2026-09-21 (*«os seletores de cor de todo o app precisam ser padronizados»*), os três
itens que o §9-vicies-semel.5 deixou abertos. ⚠️ **Zero contador partilhado, zero contrato, zero ADR.**

### §9-vicies-quater.1 — ⛔⛔ A barra no TOKENS foi construída, MEDIDA e RECUSADA — duas vezes

O Tokens era a maior população de selectores do app (`86` quadrados de `32 px` ANTES do nome). A 1.ª
redacção passou-o pela porta `paint_color_row` (nome · barra · cauda de verbos fixa) e o censo das
elisões na ESCADA leu **`108`** nomes comidos no dock mínimo contra os `5` de antes: a coluna do valor
tem o piso da caixa (`90 px`, ordem do dono de 2026-05-24) e numa lista cujo CONTEÚDO são os nomes isso
deixa `~20 px` ao nome a `220`. A 2.ª (etiqueta quadrada à DIREITA do nome, a forma da pilha do vetor)
leu **`29`** — *qualquer* valor à direita come o nome, porque a cauda de verbos também está lá.
⇒ ⭐ **há DUAS formas legítimas, e a régua que as separa é o que a linha É:**

| a linha é… | a cor é… | onde |
|---|---|---|
| uma PROPRIEDADE (nome → valor) | a **barra** na coluna do valor (`paint_color_row`) | Inspector · Flip · Vetor · Painter |
| uma entrada de LISTA (o nome é o conteúdo) | a **etiqueta quadrada** da altura de uma linha (`ROW_H_PX`) | Tokens · pilha de aparência do vetor · camadas de forma do Painter |

⚠️ Numa BIBLIOTECA de cores (o Tokens) a etiqueta vem ANTES do nome — a vista de lista de estilos do
Figma e das amostras do Photoshop. Nas listas de CAMADAS vem depois (o nome é de um objecto; a cor é um
atributo dele). O Tokens ficou no desenho dele com três padronizações: etiqueta `32 → 22` (o quadrado de
lista), *Reset* de botão de texto de `48 px` para **ícone** (`IconId::Reset`, com a palavra no balão —
*«as dicas devem ser passadas para o mouse Hover»*), e o anel de foco. Catraca das elisões no degrau
estreito **`5 → 2`**. Gate `a_linha_de_token_e_uma_linha_de_lista` (etiqueta = quadrado, mesmo `x` em
toda linha — com uma autorada e uma que segue outra no CONTROLO —, chips alinhados, *Reset* ≤ uma
linha de largura); mutação **3 de 3**.

### §9-vicies-quater.2 — A porta traça o ANEL quando o selector a está a editar

`paint_swatch_or_mixed` ganhou `aberto: bool` (três chamadores: a porta, o per-corner, o teste) e a
`paint_color_row` passa `store.picker_target() == Some(id)`. ⚠️ **Cada conversão para a porta tirava ao
artista o anel** que as linhas à mão tinham (a borda `Accent` do Pincel/Papel, o `Focused` do modelador)
— a única resposta a *«o que é que esta roda está a mudar?»*. Gate
`a_barra_aberta_no_seletor_tem_o_anel` com o CONTROLO do selector aberto NOUTRA amostra; ⚠️ corre num
tema MODERNO de propósito (no clássico repouso e foco têm os dois anel, e a contagem de caminhos não vê
uma cor). Mutação **2 de 2** (`false` · «qualquer selector aberto»).

### §9-vicies-quater.3 — O PAINTER: as quatro cores por uma porta, e o CARTÃO na coluna da secção

- As cores do **Pincel** e do **Papel** eram `fill_rounded_rect` + moldura à mão (o doc da do Papel
  dizia-se *«Mirror of the Brush section's»*); a **luz** e a **cera** da Impasto eram um quadrado de
  `22 px` ANÓNIMO (nem nome visível nem acessível) encostado ao número. Hoje as quatro passam por
  `paint_brush_rows::color_row` → `paint_color_row`, com a coluna da secção da chave; luz e cera ganham
  LINHA própria (`impasto.light_color` «Color», `impasto.wax_color` «Wax Color») e os dois cartões uma
  linha a mais no `card_frame`.
- ⭐⭐ **O `CARD_LABEL_W = 96` MORREU** — a última entrada da catraca `COLUNAS_A_MAO` que era uma linha
  de propriedade. O `card_row` passa a receber a CHAVE (29 chamadas reescritas) e delega na linha
  numérica do painel (`number_field::paint_num_row`, que passa pela porta). Saiu também da tolerância
  de `the_label_column_is_one_answer` (o censo de obsolescência acusou-a na 1.ª corrida).
- ⚠️ **A declaração das secções ganhou a lista `cartao`**: as linhas DENTRO de um cartão de técnica
  medem-se à largura do cartão (`− 2·Sm`), as soltas à da linha. ⛔ Medir a secção inteira à largura do
  cartão acusava `Adjust Last Stroke` (fora de todo cartão) de um corte que o artista não vê.
- ⛔ **O tecto a `220` sobe `7 → 11`, e está escrito porquê:** as linhas dos cartões ENTRARAM na
  população (antes tinham `96 px` fixos e apertavam a caixa abaixo do piso do dono). Os quatro novos
  (`Concentration`, `Edge Darkening` ×2, `Erase Strength`) cortam só no mínimo, com balão; de `245` para
  cima nada mudou.
- Gate `as_cores_da_impasto_sao_barras_na_caixa_do_numero` (a barra ocupa exactamente a caixa do número
  de cima, numa linha própria, mais larga que duas alturas de linha). Mutação **2 de 2** + ⛔ **UMA
  NOMEADA:** trocar a coluna da secção pela cega é **inobservável em toda largura** para a Impasto — a
  coluna é `max(nome, metade)` apertada pelo tecto da caixa, e nenhum nome dela passa da metade da
  linha do cartão antes de o tecto apertar (medido a `1600`, `304` e `220`).
- `hr12`: porta de crate verificada nova (`color_row`), porque o `paint_impasto.rs` ficou sem
  primitivo nenhum no texto e reprovou sobre código melhor.

### §9-vicies-quater.4 — O VETOR e o MODELO 3D

- A etiqueta da pilha de aparência do vetor passa de `24 × row_h` (em pé) ao **quadrado** `ROW_H_PX`,
  a forma de lista das camadas do Painter. ⏳ Sem gate próprio: o censo geométrico corre o painel sem
  camadas na pilha.
- ⛔ **O Modelo 3D fica, e não é dívida de cor:** a amostra dele JÁ é barra na coluna do valor; o que
  difere é a lei de colunas do PAINEL inteiro (`colunas_da_fileira`, coluna de valor FIXA, medida e
  escrita no doc dele — a do formulário cresce com a largura). Convergir é uma wave de alinhamento do
  painel, não do selector.

### §9-vicies-quater.5 — O portão

`nextest-impacted` **17 673/17 673** · clippy `-D warnings` zero nas 7 crates tocadas · `cargo fmt` ·
censo das elisões a `--workspace` verde · mutações **9 a sangrar + 1 nomeada**, com o arnês de três
controlos.

## §9-vicies-quinquies — ⭐⭐⭐ O ALINHAMENTO: onde começa o valor, medido no produto

Ordem do dono (2026-09-21): *«quanto ao alinhamento precisamos melhorar em todos os lugares»* — e
depois do smoke das cores, *«smoke ok. siga»*.

### §9-vicies-quinquies.1 — A régua: `onde_comeca_o_valor`

[`onde_comeca_o_valor.rs`](../../../crates/ph2d-panel-registry-init/tests/it/onde_comeca_o_valor.rs)
pinta cada painel pela porta do registo (de fábrica e armado), colhe o que o índice de acerto
registou e, por fileira com nome à esquerda, o `x` onde o controlo começa. ⭐ **A unidade é o TROÇO**
(o que fica entre duas peças de largura inteira): *a coluna é uma resposta da SECÇÃO* (`Seccao`,
§6-ter), logo duas secções com colunas diferentes estão certas e duas fileiras do mesmo troço não.

O gate `dentro_de_um_troco_o_valor_arranca_numa_coluna_so` tem **duas metades**:

1. **colunas A MAIS dentro de um troço**, por painel, contra `COLUNAS_A_MAIS_DECLARADAS` em
   IGUALDADE (grid snap `1` — os títulos de secção dela são texto e não fecham troço; timeline `1` —
   as abas). ⚠️ **A unidade é a COLUNA e não o troço:** a 1.ª redacção contava troços e a prova de
   mutação **sobreviveu** (os chips da timeline puseram uma TERCEIRA coluna no troço já declarado);
2. **`UMA_COLUNA`** — os painéis que arrancam o valor numa coluna só de ponta a ponta (vetor,
   flip_frames, …) ficam assim. ⚠️ Ela existe porque o troço é cego a um degrau ENTRE troços: os
   marcadores do vetor vivem num troço deles, e a mutação que lhes devolvia o vão escrito à mão
   sobreviveu à 1.ª metade.

⛔⛔ **O ÂMBITO primeiro, e isto mordeu-me à letra da nota do §5 de 20/09:** calibrei a régua com
`-p` (24 painéis) e no `nextest-impacted` ela reprovou — `flip`, `flip_frames`, `painter_layers` e
`wet_tuning` só registam num build de WORKSPACE, e o Painter trazia um degrau que o `-p` não via.
Hoje o piso é o de workspace (`27` painéis, `455` fileiras, medido) e a corrida pobre reprova alto
com a causa na mensagem.

⚠️ **Três filtros da régua, cada um nascido de um falso positivo medido:** a fileira que arranca a
`< 24 px` da borda (botão, lista, paleta), a que só tem um botão ENCOSTADO à direita (`> 0,66` da
largura: escolher, fechar, remover) e a que arranca numa ALÇA de curva (`< 16 px`). E o separador de
troço aceita peças que arrancam **junto** da borda (`< 24 px`) — com `< 1 px` os cabeçalhos dos
cartões do Painter, que recuam, não fechavam nada e o painel inteiro lia-se como UM troço.

### §9-vicies-quinquies.2 — Os CINCO degraus, e a cura de cada um

| onde | degrau | mecanismo | cura |
|---|---:|---|---|
| Vetor · marcadores, catálogo, tokens, filtros, estados, conector | `4 px` | `inner_x + label_col_w + Spacing::Xs`, escrito à mão em **9** sítios (e no `RowCtx` partilhado com o Esqueleto); a porta usa `Spacing::Md` **e** desconta a coluna de animação | porta nova [`panel::value_col`](../../../crates/ph2d-editor-core/src/panel/rows.rs) = `property_row_columns(..).control` |
| Inspector · Câmera ▸ Alvo | `25 px` | uma **segunda** `Seccao` só com o nome do alvo, ao lado da da secção | uma `Seccao` para o corpo, com o nome dentro |
| Inspector · Áudio ▸ Som | `6 px` | o mesmo | o mesmo |
| Timeline · chips `Time`/`Frame`/`Length` | `17,5 px` | `CHIP_LABEL_W = 48` à mão ao lado da coluna MEDIDA dos toggles — e o pintor usava `Sm/2` onde a régua do fluxo usava `Xs/2`, com o doc a jurar que eram iguais | a célula do chip é a do toggle (`pad · nome · pad · valor`), medida sobre a mesma lista; [`transport_chips.rs`](../../../crates/ph2d-panel-timeline/src/transport_chips.rs) pelo tecto de 600 |
| Painter · parâmetros de PADRÃO (Paper, Grain) | `25 px` | `paint_num_params` media uma coluna própria sobre os nomes dela | recebe a `Seccao` do cartão, e `seccoes::COM_PADROES` mede os nomes de **todos** os padrões (senão trocar de padrão movia a coluna do cartão) |

⛔ **Ficam DECLARADOS e não curados:** os degraus ENTRE secções (Grid Snap `110`/`122`/`112,5`,
Inspector `111`/`136`/`141,8`) — a coluna de cada secção é medida sobre os nomes dela e cede quando a
secção tem duas componentes. ⏳ **Uniformizar isso é decisão do DONO** (uma coluna por PAINEL daria
alinhamento total e custaria largura de nome às secções de uma componente).

### §9-vicies-quinquies.3 — ⛔⛔ O preço do vetor, e a cura que o pagou

A coluna do valor passou a acabar onde as vizinhas acabam (a coluna de animação à direita) — os
marcadores passavam `18 px` para lá dela. No degrau estreito (o dock no mínimo) o chip das pontas
fica no piso de `72 px`, dos quais `46` são cromo, e o `None` passou a sair `N…`. ⛔ **Duas curas
construídas e REVERTIDAS:** encolher o recuo da seta do chip (o `None` continuou cortado — a medição
com a fonte do censo mostrou `26 px` de orçamento, não os `38` da minha conta) e subir a catraca
(proibido). ⭐ **A cura foi a grelha de ferramentas do vetor:** repartia sempre em **três** partes
iguais e cortava `Bucket`/`Chamfer`/`Connect` na mesma largura — passou à porta que a casa já tinha
(`wrapped_cells_for`, o `3` como TECTO). ⇒ o vetor desce de **3 → 1** corte nas duas catracas.

⛔⛔ **E essa porta tinha um defeito PRÓPRIO, apanhado pela catraca da altura de abertura** (`+46 px`
no vetor): ela quebrava pelas palavras SEM tecto e só DEPOIS partia pelo tecto — uma fileira gulosa
de quatro sob tecto de três virava `3 + 1`. Hoje o tecto entra NA quebra
(`segmented_row_counts_ate`), com gate próprio (`o_tecto_entra_na_quebra_e_nao_deixa_pecas_sozinhas`,
com o controlo de que sem tecto a mesma lista faz fileiras de quatro). Os outros chamadores (o mixer,
as propriedades do Inspector) passam pela mesma cura.

### §9-vicies-quinquies.4 — O portão

`nextest-impacted` **17 678/17 678** · clippy `-D warnings` zero nas 7 crates tocadas · `cargo fmt` ·
`censos-da-arvore-combinada.sh` **127/127** · a régua nova a `--workspace` verde.

**Mutações: 10 a sangrar + 1 nomeada** (arnês com controlo de filtro vazio e de não-compila):
o vão à mão no `value_col` · a 2.ª `Seccao` do alvo · a do som · o chip a `48` · a lista declarada
desactualizada · uma entrada órfã em `UMA_COLUNA` · os parâmetros de padrão com coluna própria · o
separador estrito (a régua a medir-se a si mesma) · o tecto fora da quebra · a grelha do vetor de
volta às partes iguais. ⚠️ **Nomeada:** tirar o `paper` de `COM_PADROES` **sobrevive** a esta régua —
é LEGIBILIDADE (os nomes do padrão cabem na coluna), não alinhamento; à largura medida a coluna do
Paper já é a mais larga, e quem a guarda é o censo das elisões.

⚠️ **Flake do fan-out, para promover:** `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`
(`ph2d-tool-painter`, gate de RAZÃO) reprovou numa corrida de `17 677` a `load ~37` e passou sozinho
ao lado do irmão já listado `the_mask_stroke_cost_does_not_follow_the_canvas` — zero linhas de diff
naquela crate.


## §9-vicies-sexies — ⭐⭐⭐ A COLUNA DO PAINEL: o valor arranca num `x` SÓ, por ordem do dono

**Enio, 2026-09-23:** *«quero tudo alinhado e padronizado»* — a resposta à pergunta que o §9-vicies-quinquies
lhe devolveu com o preço ao lado (*uma coluna por painel, mesmo que alguns nomes de secção percam espaço e saiam
cortados com balão?*). Até aqui a coluna era uma resposta da SECÇÃO (`Seccao::medida`) e o valor do Inspector
arrancava em `111` / `136` / `141,8` conforme a secção; o Grid Snap em `110` / `112,5`; o Painter em `117` / `123`.

### §9-vicies-sexies.1 — A lei ([`ColunaDoPainel`](../../../crates/ph2d-editor-core/src/widget/property_box/coluna_do_painel.rs))

- **Mora no `ErasedPanel`**, um por painel, e é armada pelo `paint` dele (`thread_local`, reentrante). Toda chamada
  a `property_label_col_w_for` dentro da pintura deixa um PEDIDO `(nome mais largo, o que o controlo precisa, recuo)`
  e recebe a coluna do PAINEL. ⚠️ **Fora de um painel nada muda, ao bit** — gate com a lei antiga escrita como
  oráculo sobre `>30 000` combinações (`fora_de_um_painel_a_coluna_e_a_da_seccao_ao_bit`).
- **A lei da secção partiu-se nas duas perguntas que ela juntava** (`row::limites_da_seccao` → `(prefere, tecto)`,
  e a coluna da secção é `prefere.min(tecto)` ao bit). O painel escolhe
  `min( max(recuo + prefere), min(tecto das linhas SEM recuo) )`: o mais largo que os NOMES pedem, sem apertar o
  CONTROLO de nenhuma linha de largura inteira abaixo do que ele declara.
- ⛔⛔ **A 1.ª redacção era o `min` das colunas das secções e foi REPROVADA pela régua das elisões:** no degrau
  estreito o Inspector armado passava de `79` para **`211`** nomes cortados, com a coluna do nome a `48 px` —
  *a METADE de uma linha sem nome largo não é uma necessidade, é o que ela ACEITA*, e o `min` punha-a a mandar no
  painel. Separadas, a metade passa a PISO e a cedência ao controlo a TECTO.
- ⚠️⚠️ **O tecto de um CARTÃO não manda no painel** — duas ordens do dono colidem ali (tudo alinhado · nenhum campo
  abaixo de `72 px`, 24/05): o recuo encurta o controlo do cartão, e deixá-lo mandar punha o Inspector estreito a
  `197` cortes. Um cartão alinha-se sempre que o campo dele caiba; quando não cabe, só ELE recua o recuo dele — e só
  no fim estreito do dock (a `300 px` o tecto do cartão está `63 px` acima da coluna).
- **Guardam-se os PEDIDOS e não a coluna** — um pedido não depende da largura, logo arrastar o dock acerta no MESMO
  quadro. A memória **só cresce** (fechar uma secção não mexe a coluna das outras).
- **A BASE** (o rectângulo das linhas de largura inteira) sai da PRIMEIRA linha do quadro mais o recuo que ela tinha
  no anterior — ⚠️ não da faixa do painel (`PaintCtx::slot` é a posição de NASCIMENTO de um flutuante, e o Grid Snap
  flutua). ⛔ **Nem o envelope** (uma célula de par na borda arrastava-o `67 px` e punha `392` de `392` linhas como
  assimétricas) **nem a moda** (no degrau estreito há mais linhas DENTRO de cartões que fora): é a geometria **mais
  larga que se repete**. Se a primeira linha mudar de ESPÉCIE (outra largura), ela é lida como recuada por igual
  dentro da largura lembrada.
- ⚠️ **Um painel converge no 3.º quadro**: o 1.º aprende a base, o 2.º os pedidos (medido no Inspector: `1` pedido
  depois do 1.º quadro, `51` depois do 2.º). No app são `33 ms` ao abrir um painel.

### §9-vicies-sexies.2 — O que mudou fora da lei

- **O arnês** (`ph2d-ui-testkit::medindo_a_pintura_do_registo_com_bandas`) pinta **dois** quadros de aquecimento
  dentro de uma porta nova, [`aquecimento`](../../../crates/ph2d-editor-core/src/aquecimento.rs) (módulo FOLHA: no `panel::` ele fechava um ciclo no DAG da fundação) — e
  os censos que acumulam através da pintura inteira perguntam-lhe e não os contam: o balão (sem isso acusava `26`
  rótulos que no quadro visto CABEM) e os avisos do Inspector (liam cada facto do painel escrito `3×`). No app a
  porta nunca arma. ⛔ A 1.ª cura era um «descartar» só do balão; *uma porta, N leitores* — duas maneiras de ignorar
  o mesmo quadro divergem no dia em que uma não for chamada.
- **A altura de abertura do Inspector DESCEU** (`918 → 822`): com a coluna do nome igual em todas as secções,
  escolhas que viravam PALETA por não caberem ao lado de um nome largo passam a caber na fileira.
- **Três gates recalculavam a coluna FORA da pintura** (as duas réguas dos selectores de cor e a da secção *Inspect*
  do Grid Snap) e passaram a ler a coluna do PAINEL — da pintura dele, ou da porta com o desejo certo. *Um oráculo
  que recalcula a lei fora do contexto dela mede outra lei.*
- **Duas secções passaram a PEDIR a coluna em vez de a escrever:** o cartão de instância do Inspector (a fracção
  `0,28` com tecto de `72 px`, `79,7` contra `111`) e a secção *Inspect* do Grid Snap (a lista medida mais o vão à
  mão, `112,5` contra `110`).
- **A régua do alinhamento virou LEI do painel inteiro** (`onde_comeca_o_valor`): `UMA_COLUNA` (a lista de quem JÁ
  estava numa coluna, que só crescia) deu lugar a `COLUNAS_DECLARADAS_POR_PAINEL` — as EXCEPÇÕES, numa igualdade
  com censo de obsolescência: a Timeline (`2`, as abas são um segmentado na faixa do título) e a galeria de widgets
  (`3`, um catálogo). Um painel novo nasce obrigado à coluna única. Piso de `11` painéis numa coluna.

### §9-vicies-sexies.3 — O PREÇO, medido e aceite pelo dono

| degrau | antes | depois |
|---|---|---|
| Inspector armado, `300 px` | valor em `111`/`136`/`141,8` | **`111`, 312 linhas** |
| Grid Snap | `110`/`112,5` | **`122`, 9 linhas** |
| Painter | `117`/`123` | **`123`** |
| cortes a `1366`/`1920`/`1280` | — | **`+24`** no Inspector, **`+2`** no Painter (todos com balão) |
| cortes no degrau estreito, Inspector | `79` / `75` letras | **`83` / `79`** |

Os `26` nomes vivem em `O_PRECO_DA_COLUNA_UNICA` (uma lista, UM mecanismo, com obsolescência); o `hand_right` SAIU
da dívida armada — com a coluna única o chip do artista cabe e é o nome que corta, a troca que aquela nota já
descrevia, feita ao contrário.

### §9-vicies-sexies.4 — O portão

`nextest-impacted` **17 690/17 690** · clippy `-D warnings` zero nas 4 crates tocadas · `cargo fmt` ·
`censos-da-arvore-combinada.sh` verde · a régua do alinhamento e as das elisões a `--workspace`.

**Mutações: 13 de 13 a sangrar** (o arnês com controlo de filtro vazio e de não-compila): o tecto de um cartão a
mandar no painel · sem o tecto do controlo · esquecer o recuo · a base pela MODA · sem a folga simétrica · a
memória que não fica · sem o teste de simetria · a cedência sempre · o `min` das secções (a 1.ª lei) · um só quadro
de aquecimento · o balão do aquecimento a ficar · a sonda do Grid Snap `+2,5 px` · o eixo da instância à mão.
⚠️⚠️ **Três SOBREVIVERAM à 1.ª ronda e a causa era UMA:** as fixturas não pediam empréstimo — e a METADE põe
toda linha SIMÉTRICA no mesmo `x` por construção (é o centro menos o vão), logo a base pela moda, a folga simétrica
e o teste de simetria eram invisíveis a elas. *Uma fixtura no ponto neutro de uma lei não testa essa lei* — as três
passaram a nomes acima da metade (e o par a `200 px`, onde o tecto dele deixa de dar a resposta por acaso).

⛔ **Três gates vizinhos recalculavam a coluna FORA da pintura** e reprovaram sobre produto certo — curados a ler a
do painel (ver §9-vicies-sexies.2). E a 1.ª versão do oráculo dos selectores de cor contava TODO controlo que
acaba na borda: a metade direita de um PAR também acaba lá, e no vetor a moda caía nela (`1792` contra os `1760`
dos campos sozinhos).

⚠️ **Flake do fan-out, já na lista do §5.0:** `an_abandoned_march_returns_nothing_and_returns_fast`
(`ph2d-field-render`) reprovou numa corrida de `17 690` e passou na seguinte — zero linhas de diff naquela crate.


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
| `toggle`/`command` da física | OUTRA COLUNA — ver abaixo |

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

## §11 — O que esta linha recomenda a quem a integrar

1. **Correr o `diag_onde_cai_a_pista_do_pente` da `line/sculpt3d` DEPOIS da fusão** e reescrever com
   a saída dele as **duas** tabelas vivas do §5 (o `CLAUDE.md` §5 e o doc-comment do
   `scenes_pente.rs`). ⛔ A do handoff `…_sculpt3d_2026-09-17.md` §94 **fica como está**.
2. **Promover à lista de flakes do §5.0:** `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`
   (`ph2d-tool-painter`) — único ✗ de `17 690`, gate de RAZÃO de custo, 3/3 verde sozinho a `load 23`, zero
   linhas de diff na crate (§9-vicies-septies).
3. A linha do §5 está no §9, pronta a colar.
