# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-09-20

> **Leitor: o agente INTEGRADOR.** Denso de propósito (§0.8 — a resposta ao Enio é outra coisa).
> O que este documento existe para lhe poupar: a superfície de colisão **medida**, o que já está
> curado e não precisa de ser redescoberto no portão, e as sete leituras que um `git diff` rápido
> entende ao contrário.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/UIUX` |
| HEAD | `7f60e93f7` |
| merge-base com `main` | **`76bd6de02`** — ⭐ a linha **JÁ ESTÁ REBASEADA** sobre o `main` de hoje |
| commits | **84** |
| ficheiros | **777** · `+30 965` / `−5 045` |
| crates novas | **nenhuma** |
| ADR novo | **nenhum** ⇒ fora de toda disputa de número |
| contrato congelado (§6) | **intocado** nos dois ficheiros |

⚠️ **O rebase foi feito hoje e conflitou em `project-memory/` apenas** — os 19 commits que o `main`
ganhou desde o fork são todos da `line/cascadeur` e todos de memória. As quatro resoluções são
**duas adições independentes ao mesmo fim de ficheiro**, juntas nas duas (nenhuma linha perdida;
conferido por censo de ponteiros mortos do `MEMORY.md`, que fechou a **zero**).

---

## §2 — O que a linha fez, em quatro arcos

### A. A FRONTEIRA DOS MOTORES (HR-15) — de `1 519` rótulos a **ZERO**

O handoff de 2026-09-16 (`…_FECHO_…`) fechou o HR-15 sobre os **painéis** e nomeou, por escrito, a
fronteira seguinte: *«os MOTORES escrevem o rótulo numa tabela de dados, e ali a chave tem de ser
derivada do id»*. Esta linha pagou-a.

| motor | rótulos | a chave deriva de |
|---|---:|---|
| catálogo de componentes (`ph2d-component-desc`) | 291 | o **id** do campo |
| motor do pincel (`ph2d-painter-brush`) | 104 | o id do parâmetro |
| nomes de nó (`ph2d-node-registry-init`) | 133 | o **TIPO** do nó |
| parâmetros de nó | **828** | o par `(tipo, param)` |
| opções de selector | **562** | o **sítio que as declara** |
| canais | 24 | o par `(coluna, modo)` |

⭐ **74 % da fronteira era DERIVAÇÃO** e não autoria — medido antes da primeira linha
(`23be7d20b`), e é o que tornou a wave possível numa jornada.

Mais: **nove crates de UI que nunca tiveram censo** (e **19 chaves órfãs VIVAS** nelas), o catálogo
do vector (`163` — a nota do §5 dizia `7`), o painel da escultura (`124` rótulos crus com o censo
dele VERDE), os dez matcaps, os oito nomes de tema da barra do topo, a legenda e as queixas do
L-System (que o artista lia **em português**), os nomes de secção do painel de nós (`247` sítios).

**i18n:** `4 344` → **`5 386`** chaves (`+1 042`), **19 ficheiros de tabela novos**.

⚠️ **O `censo-texto-pintado.py` mentia nos DOIS sentidos** (`4d9d1150c`), e a população do gate de
runtime era **44 de 325 crates** (`acfccc52f`) — alargada, ela acusou `189` rótulos que ninguém
via. ⇒ *a dívida não era o que a régua dizia; a régua é que era estreita.*

### B. O IDIOMA DE TESTE — trinta censos leem o FONTE, um lê o ECRÃ

`6d1200613`. Os 30 censos do HR-15 respondem *«esta palavra vem da tabela?»*. Nenhum responde
***«ela COUBE?»***. O idioma de teste (`ph2d_i18n::pseudo::deforma`) é uma **função pura do
inglês**, logo uma pintura em inglês responde pelas duas línguas — ⛔ **sem mexer no `PH2D_LANG`
do processo**, que tornaria a suíte mais um membro da família de flakes de fan-out.

### C. A VARREDURA DAS ELISÕES — o que o ecrã de facto mostra

`crates/ph2d-panel-registry-init/tests/it/nenhum_rotulo_do_app_pinta_nada.rs`.

1. **A ESCADA** (`5474924a6`): a varredura media **UMA** largura de coluna — a de FÁBRICA. A
   largura de uma coluna docada vem do **TOKEN**, e a janela só decide **ONDE** ela fica; o dono
   trabalha no **mínimo** (`PANEL_MIN_W_PX = 220`) em cinco dos seis espaços de trabalho. ⇒ quatro
   degraus, e **`129` cortes invisíveis** apareceram de uma vez.
2. **A PASSAGEM ARMADA** (`3cc76b7b4`): o Inspector tem 28 secções que só existem com um objecto
   escolhido, e nenhuma régua de largura as via.
3. **CINCO painéis eram medidos VAZIOS** com o piso GLOBAL verde (`b0e992acd`) — *um piso sobre a
   SOMA não pergunta por ninguém*; e **duas ausências eram PALPITES** (`c77a88a11`): o `model3d` e
   o `sculpt3d` foram declarados inatingíveis com frases sobre a cena que os produz, e os dois lêem
   um `thread_local` que é **DADOS**. `+402` rótulos, **zero** cortes.
4. **`Medido::onde`** (`765259ff7`): a régua dizia QUE texto não coube e não dizia ONDE — nove
   buscas falhadas passaram a duas corridas. ⛔ `Location::caller()` **não** atravessa um fecho.
5. **TRÊS controlos pintavam NADA** na largura do dono (`05b395317`), curados.

### D. O PADRÃO DA CAIXA ÚNICA, e a ordem de hoje

- O mixer e as **cinco** secções do Audio Editor entraram na caixa única do app, atrás de **uma
  porta por crate** (`62baa41c6`, `3fc3fe2f0`) — a mesma forma estava escrita cinco vezes.
- ⭐⭐⭐ **A ordem do dono de 2026-09-19 — *«encurtar · balão ao passar o rato»* — está cumprida nas
  DUAS metades** (`f17b3ef1b`), e o §4 abaixo diz o que um diff rápido entende ao contrário.

---

## §3 — Foundational / partilhado tocado

| ficheiro / área | o quê | aditivo? |
|---|---|---|
| `ph2d-editor-core` (60 f) | `text_elide::balao` (**novo**) · `text_elide::encurtar` (**novo**) · `fit_do_nome` · `paint_hover_overlays` · `paint_hover_tooltip` passa a devolver `bool` | ⚠️ **uma assinatura mudou** — ver abaixo |
| `ph2d-i18n` (39 f) | 19 tabelas novas, `+1 042` chaves | aditivo |
| `ph2d-tokens` (9 f) | censo de texto próprio (`cada_palavra_desta_crate_vem_da_tabela`) + as recusas viram chaves | aditivo |
| `ph2d-ecs` (7 f) | rótulos de `audio_2d`/`blend`/`signal_actions`/`sprite_anim`/`vec_bindings` pela tabela | aditivo |
| `shells/desktop` (21 f) | 6 gates novos + os rótulos das pontes do Inspector | aditivo |
| `scripts/censos-da-arvore-combinada.sh` | **+2 termos no `$FILTRO`** — ver §5-bis | aditivo |
| `scripts/censo-texto-pintado.py` | a régua de porta deixou de mentir nos dois sentidos | — |
| 38 `Cargo.toml` | a crate-régua `ph2d-label-census` entra como **dev-dep** | aditivo |

⚠️⚠️ **A ÚNICA assinatura pública que mudou:**
`screens::hero::topbar::paint_hover_tooltip` passou de `-> ()` a **`-> bool`** (devolve se pintou).
Ela tem **um** chamador no repo, e ele mudou para a porta nova `paint_hover_overlays`. Se outra
linha lhe tiver acrescentado um chamador, o merge parte a **compilar** — que é o modo de falha bom.

⛔ **`ph2d-editor-core/tests/it/hr12_widgets_a11y.rs` ganhou uma lista nova e VERIFICADA**
(`PORTAS_DE_CRATE_VERIFICADAS`) — não é uma tolerância: há um gate irmão que exige que o ficheiro
da porta exista, defina a função e contenha ele próprio um marcador canónico. Ver §4.6.

---

## §4 — Sete coisas que uma leitura rápida do diff entende ao contrário

1. ⛔ **`CORTES_NO_DEGRAU_ESTREITO` não desceu com a lei de encurtar, e isso é a lei.** Um rótulo
   que sai `"Acceleration"` de `"Acceleration (0 = instant)"` **continua** a esconder a explicação;
   o que muda é ele deixar de comer o NOME. Quem conta essa diferença é a catraca IRMÃ,
   `LETRAS_PERDIDAS_NO_DEGRAU_ESTREITO`. *Duas grandezas estavam a ser lidas como uma.*
2. ⛔ **O balão é registado pela LEI DO CORTE, não pelos pintores.** Há ~40 sítios que pintam um
   rótulo e **um** que decide que ele não cabe. Um pintor novo herda o balão **por usar a porta** —
   quem acrescentar um pintor não tem nada a fazer, e o censo
   (`toda_palavra_cortada_tem_balao`) reprova se ele contornar a porta.
3. ⚠️ **O balão custa uma comparação por corte, não uma `String` por quadro.** Ele pergunta
   PRIMEIRO se o ponteiro está dentro da área; o caminho do produto aloca **no máximo uma** `String`
   por quadro. O cabeçalho do `elisao` recusa por escrito a versão cara, e esta não é ela.
4. ⚠️ **O ponteiro NÃO é um campo do `WidgetStore`, e a ausência é a decisão.** A 1.ª redacção
   pô-lo lá e custou uma linha de um ficheiro já no tecto de LOC. *Um facto que só um módulo
   consome vive nesse módulo* — e o `hot_id` não responde à pergunta (ele nomeia um widget
   REGISTADO, e a maior parte do texto deste app não é um widget).
5. ⛔⛔ **QUATRO renomes à mão foram construídos e REVERTIDOS, com o preço medido:** encurtar
   `"Always show anchors"` → `"Always show"` (e três irmãos, todos onde a SECÇÃO já diz a palavra
   que sai) fechou **`2`** rótulos de `80` e deixou **`54`** citações do nome antigo na prosa deste
   repo — **três delas no `CLAUDE.md` §5**, que uma linha não pode editar. *Renomear um rótulo custa
   as citações dele.* ⇒ os `~63` que ficam são decisão de VOCABULÁRIO do dono, não dívida técnica.
6. ⚠️ **A precedência das duas bolhas mudou-se para `topbar::paint_hover_overlays`** e o gate da
   ORDEM lê **esse** ficheiro, não o `paint.rs`. Quem procurar a lei no quadro não a encontra.
7. ⭐ **O `hr12_widgets_a11y` estava VERMELHO desde `3fc3fe2f0`** (o commit anterior a hoje) e
   nenhum portão daquela wave o viu: ele vive em `ph2d-editor-core/tests/it/` e a wave correu a
   crate do audio editor. **Quinta ocorrência** desta cegueira registada no repo. A causa é boa —
   as cinco secções passaram a chamar a porta da própria crate, e o gate só conhecia DUAS formas de
   ter a11y.

---

## §5 — O que só o `ship.sh` pega — **corrido ao fechar, tudo verde**

| portão | resultado |
|---|---|
| `cargo fmt --all --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ **zero** |
| `cargo machete` | ✅ *«didn't find any unused dependencies»* |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✅ zero |
| `scripts/check-standalone-optional.sh` | ✅ 10 crates |
| `scripts/check-workflow-packages.sh` | ✅ 32 nomes / 379 membros |
| `Cargo.lock` | **nenhum pacote externo novo** (só arestas internas: a dev-dep da crate-régua) |

⏳ **NÃO corrido:** `typos`, `cargo deny`, `cargo audit` e a matriz 3-OS — são do `ship.sh` do
integrador. ⚠️ A linha **não** acrescentou dependência externa nenhuma, logo `deny`/`audit` não têm
sujeito novo.

---

## §5-bis — A ÁRVORE COMBINADA (§1.5.9 item 5-bis) — **corrida, DOIS vermelhos, os DOIS curados**

```
bash scripts/censos-da-arvore-combinada.sh
→ 129 testes, 129 passed · controlo do filtro: 12 de 12 censos correram ✓
```

**(a) `ph2d-panel-model3d::every_key_of_this_panel_exists_on_both_sides`** — a fixtura do modelador
usava `panel.model3d.shading.solid`, uma chave que **NUNCA EXISTIU** (o produto emite `matcap` ou
`render`). ⚠️ Era pior que um rótulo curto: a fixtura media a largura de uma palavra que o painel
**não consegue pintar**, logo a promessa dela — *os rótulos da fixtura são os mais largos do
catálogo* — era vácua exactamente ali. ⛔ **Nenhum portão desta linha o via:** a chave é USADA numa
crate e DECLARADA noutra, e quem cruza as duas é o censo do painel, que vive numa terceira. ⇒ hoje
é **DERIVADA** da mesma família dos chips (`rotulo_da_barra()`).

**(b) O CONTROLO DE COBERTURA do próprio script** acusou **dois censos que ele nunca correu**:
`cada_palavra_desta_crate_vem_da_tabela` (`ph2d-tokens`) e `cada_palavra_deste_no_vem_da_tabela`
(`ph2d-node-source-lsystem`). ⚠️ **Os dois são PRÉ-EXISTENTES no `main`** e ficavam de fora porque
o nome do módulo está **em PORTUGUÊS** e o filtro casa prefixos ingleses — *a família de um censo é
o NOME DO MÓDULO*, a mesma lei do `feedback_a_nextest_filter_matches_the_module_not_the_function`
uma volta acima. ⇒ dois termos no `$FILTRO`; a corrida passou de `123` para **`129`** testes.

⛔ Isto **não** substitui o portão da árvore combinada (`--ff-only` + `foundational-integrate.sh`).

---

## §6 — Ordem, dependências e o que smokar

- **Sem ordem interna:** os 84 commits são lineares e não há dependência entre arcos. O rebase já
  está feito; o integrador precisa só do `--ff-only`.
- ⭐ **O dono JÁ SMOKOU e aprovou** três fatias desta linha, a última hoje: a caixa única do Audio
  Editor, o alinhamento do `Mute` do Master, e o **balão + encurtar**.
- ⚠️ **O que mudou DEPOIS do smoke dele são `2` commits e nenhum toca em produto:** `c1eba0458`
  (dois ficheiros de memória que o índice apontava e que nunca tinham sido commitados) e
  `7f60e93f7` (a fixtura do model3d + o filtro do script). **Zero bytes de produto.**
- **Re-smoke depois da fusão** (o `main` é outro binário):
  ```
  cd /home/enio/Documentos/Projetos/PH2D && env PH2D_PARTICLES_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
  ```
  Estreitar o painel da direita até os nomes cortarem · `Lifetime` tem de ler **`Lifetime`** e não
  `Lifetime…` · passar o rato por cima dele mostra **`Lifetime (s, 0 = forever)`** · passar o rato
  por cima de um nome comprido ainda cortado mostra-o inteiro.
- ⏳ **NÃO smokado:** os arcos A e B (HR-15 dos motores e o idioma de teste) **não têm smoke
  próprio** — eles são medidos por `1 042` chaves em 30 censos, e o que o dono veria é a ausência de
  identificadores crus na tela. ⚠️ Um `PH2D_LANG` noutro idioma é o smoke honesto deles e **não foi
  corrido**.

---

## §7 — O que fica ABERTO, e de quem é

| item | dono |
|---|---|
| os **~63** nomes compostos do Inspector que só um RENOME encurta (`Air Acceleration`, `Non-Spatialized Radius`) — o balão lê-os hoje | **dono** (vocabulário) |
| o mixer mostrar unidades reais (dB/Hz) em vez da fracção — pede o motor de áudio a publicar a unidade | **dono** |
| as três decisões antigas: a pose 2D/3D (`722` sítios), partir o `DrawMode` nos dois eixos, os 9 toggles de módulo → Layout | **dono** |
| converter à porta da casa: `ph2d-panel-painter-layers` (3 f) e `ph2d-panel-asset-browser` (1 f). ⛔ Os 4 sliders da tira do mixer FICAM (coluna de 22–46 px) | linha |
| os ~46 cortes do degrau estreito FORA do Inspector, em sete painéis, com causas diferentes | linha |
| *«travou por um minuto»* (09/09) — **sem reprodução**; o gatilho saiu na w49 | — |
| **promoção pedida ao `CLAUDE.md` §5.0:** `ph2d-tool-painter …the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` como membro da família de flakes de fan-out | **integrador** |

---

## §8 — Prova de mutação desta última fatia: **14 de 14 sangram**

O âmbito perdido em cada um dos três pintores · o âmbito a LIMPAR em vez de repor (⚠️ **sobreviveu
ao censo e sangra na unidade** — a varredura não aninha um corte dentro de um âmbito filho, e isso
está escrito) · o balão a guardar tudo · a lei de encurtar inerte · o parêntese-que-é-o-nome a
deixar de ser recusado · o degrau 2 a registar-se como «coube» · o nome encurtado a deixar de ter
de caber inteiro · o balão a ler tudo em vez do ponteiro · o rótulo de propriedade a voltar ao `fit`
cru · o pintor do balão a devolver cedo · o balão a guardar o texto JÁ cortado · o quadro a deixar
de esvaziar.

---

## §9 — A linha do `CLAUDE.md` §5, PRONTA A COLAR (é o integrador que a escreve)

⚠️ **Esta linha NÃO foi escrita nesta worktree, de propósito.** A tabela de anti-colisão da
DIRETRIZ diz que o `CLAUDE.md` §5 se edita *«só na integração, no primário, uma linha de trabalho
de cada vez»* — N linhas a editarem um ficheiro de 900 KB nas suas próprias árvores é conflito
garantido. ⇒ o entregável da linha é o texto; o escritor é quem funde.

**(a) Na secção `UI/UX — redesenho plano`, SUBSTITUIR o primeiro parágrafo de `**Aberto:**`
(o que começa em *«três decisões do dono»*) por:**

> **Aberto:** ⭐⭐⭐ **A FRONTEIRA DOS MOTORES FECHOU e o HR-15 está a ZERO sobre a população LARGA**
> (20/09, [handoff](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20.md)):
> os `1 519` rótulos que os MOTORES escreviam numa tabela de dados falam por `ph2d-i18n`
> (`4 344` → **`5 386`** chaves, 19 tabelas novas), com a chave **derivada do id** — ⭐ `74 %` da
> fronteira era DERIVAÇÃO e não autoria, medido antes da 1.ª linha. ⚠️ **A população do gate era
> `44` de `325` crates**, e alargada ela acusou `189` rótulos que ninguém via: *a dívida não era o
> que a régua dizia; a régua é que era estreita.* ⭐⭐ **E há uma régua que lê o ECRÃ e não o
> fonte** — os 30 censos respondem *«vem da tabela?»* e a varredura das elisões responde
> ***«COUBE?»***, sobre uma **ESCADA** de quatro larguras (⛔ ela media UMA, a de FÁBRICA, e o dono
> trabalha no mínimo: `129` cortes invisíveis e **três controlos que pintavam NADA** apareceram de
> uma vez). ⭐⭐⭐ **E a ordem do dono de 19/09 — *«encurtar · balão ao passar o rato»* — está
> cumprida nas duas metades:** toda palavra cortada por este app é legível ao passar o rato
> (`128` de `128`, com o gate a prová-lo pela MESMA varredura que as achou), e *um nome perde a
> EXPLICAÇÃO antes de perder LETRAS* tirou as reticências a `18` rótulos **sem uma chave nova**.
> ⛔ **Renomear um rótulo à mão foi MEDIDO e REVERTIDO:** quatro renomes fecharam `2` de `80` e
> deixaram `54` citações do nome antigo na prosa, três delas **neste §5** ⇒ os `~63` nomes
> compostos que ficam são decisão de VOCABULÁRIO do dono, não dívida. ⏳ **Ficam as três decisões
> dele:** a pose 2D/3D (`722` sítios de produto, 5 crates), partir o `DrawMode` nos dois eixos
> (`17` variantes vivas, eram 14) e os 9 toggles de módulo → Layout · o **«travou por um minuto»**
> de 09/09 segue **sem reprodução** · ⏳ as superfícies de UI que as outras linhas trouxeram foram
> escritas contra a lei de espaçamento ANTIGA.

**(b) Na lista `**Ler:**` da mesma secção, ACRESCENTAR à cabeça:**

> ⭐⭐⭐ **[handoff de 20/09](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-20.md)** — a fronteira dos motores, a escada das elisões e o balão; o §4 tem **sete** coisas que uma leitura rápida do diff entende ao contrário (entre elas que a catraca dos cortes **não desce** com a lei de encurtar, e porquê) e o §5-bis os **dois** vermelhos que só a árvore combinada viu ·

**(c) ⚠️ E o parágrafo do `dock_columns` desta mesma secção já foi CORRIGIDO por esta linha**
(commit `52f591b17`… na verdade `52d4c8f64`): ele dizia que o `close` tinha zero chamadores de
produto, e hoje os dois alternadores do menu *View* existem. O texto no `main` já reflecte isso —
**não o reescreva**.
