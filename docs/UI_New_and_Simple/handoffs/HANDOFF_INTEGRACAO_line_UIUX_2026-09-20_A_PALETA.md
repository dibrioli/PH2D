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

## §10 — O portão

*(preenchido no fecho — ver o commit de fecho desta linha)*
