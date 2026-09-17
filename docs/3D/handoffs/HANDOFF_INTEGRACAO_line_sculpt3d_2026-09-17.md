# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, 2026-09-17

> **Uma wave, e ela é só GATE: zero linhas de produto.**
> A linha reabriu depois de ter sido integrada em 17/09, rebaseou no `main` novo (`3090cac3f`,
> fast-forward — zero commits locais) e fechou o **G-20**, o último item da espec do pincel de plano
> que estava por escrever.
>
> ⚠️ **Para quem funde:** este diff **não muda o que o app faz**. Ele acrescenta dois testes e a
> tabela medida que os justifica. `PROJECT_SCHEMA`, `FIELD_DOC_VERSION`, `VEC_SCENE_SCHEMA`,
> `FLIP_SCHEMA` e os três registos de componentes **não se mexem** — e a prova é que nenhum ficheiro
> que os declara aparece no diff.

---

## §57 — ⭐⭐⭐ O G-20: o tecto do raio PASSA a pista do alvo e NÃO chega ao digitável dele

### §57.1 — Porque ele não existia

A espec [`SPEC_pincel_de_plano.md`](../cleanroom/SPEC_pincel_de_plano.md) encomenda o gate numa linha
da tabela §14.5, com o nome `o_tecto_do_raio_nao_e_menor_que_o_do_alvo` e o critério em duas metades:

> pista **`≥ 500`** px e digitável **`≥ 5 000`** px de raio — facto de interface do alvo (`1 000` e
> `10 000` px de **diâmetro**, §14.3). ⛔ Não é um número nosso: se o dono o recusar, o gate não nasce

e a **errata Q2** dela já nomeia o problema:

> o G-20 não nomeia o tamanho da vista, e a metade «digitável ≥ `5 000` px» é **inalcançável** sob um
> tecto derivado da vista (`2 203` px a 1920×1080) — se o dono o quiser, nasce com a população nomeada

⇒ o item ficou marcado **«do dono»** no [§52 do handoff de 16/09](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-16.md).
Em 17/09 o dono delegou (*«escolha e siga»*), e a wave é a que a errata prescreve: **nasce com a
população nomeada**.

### §57.2 — ⛔⛔ O gate NÃO tem o nome que a espec lhe deu, e essa é a decisão

Medida, a frase da espec é **verdadeira numa metade e falsa na outra**. Escrever um gate chamado
*«o tecto não é menor que o do alvo»* seria **um nome que mente em toda corrida verde** — a família
que esta crate já pagou com os oito gates citados em comentário que nunca existiram
([handoff de 13/09](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-13.md)).

⇒ são **dois** gates, cada um com o nome do que afirma, em
[`crates/ph2d-app-sculpt3d/src/rulers.rs`](../../../crates/ph2d-app-sculpt3d/src/rulers.rs):

| gate | o que afirma |
|---|---|
| `o_tecto_do_raio_passa_a_pista_do_alvo_e_nao_chega_ao_digitavel` | em toda vista nomeada, o tecto **≥ `500`** (a pista do alvo) · **< `5 000`** (o digitável dele — *a divergência, EXIGIDA a existir*) · e **ainda é a diagonal** |
| `o_que_aperta_o_raio_e_a_vista_nunca_o_widget` | a pista do painel oferece o digitável do alvo **em cheio** e fica estritamente acima do tecto em toda vista nomeada ⇒ quem clampa é o **recurso medido**, nunca a régua do widget; **e a TROCA é nomeada** |

### §57.3 — A tabela MEDIDA (a população nomeada que a errata Q2 pede)

O nosso tecto é [`RADIUS_MAX_FRAC_OF_DIAGONAL`] `= 1.0` sobre a **diagonal da vista**, e o recurso
que ele nomeia é o **ECRÃ**: acima dele o anel do pincel já contém a vista inteira a partir de
qualquer ponto, e não há mais barro ao alcance.

| vista | tecto MEDIDO | contra a pista do alvo (`500`) | contra o digitável (`5 000`) |
|---|---|---|---|
| `1024×768` | `1 280,0` | **`2,56×`** | `0,26×` |
| `1280×720` | `1 468,6` | `2,94×` | `0,29×` |
| `1920×1080` | `2 202,9` | `4,41×` | `0,44×` |
| `2560×1080` | `2 778,5` | `5,56×` | `0,56×` |
| `2560×1440` | `2 937,2` | `5,87×` | `0,59×` |
| `3840×2160` | `4 405,8` | **`8,81×`** | **`0,88×`** |

⭐ **Batemos a pista do alvo por `2,6×` a `8,8×`** — que é a metade que o report de 16/09
(*«o radius máximo permitido é pouco»*) encomendou, e ela fecha com folga em toda vista.

⛔ **E não alcançamos o digitável dele em vista nenhuma que este app tenha**, 4K incluído. Isso é uma
**divergência DECLARADA** e o gate **EXIGE que ela exista** — sem a segunda asserção o tecto vira
**licença**: quem subisse a fracção apagaria a divergência em silêncio, e a errata Q2 ficaria a
descrever um produto que já não existe.

⚠️ **E a TROCA é nomeada, não deixada implícita:** numa vista cuja diagonal passe `5 000` (16:9,
~`4358×2451` — **acima** de 4K) quem passa a apertar é a **pista**, e ali o tecto do alvo é que é o
mais apertado dos dois. *Sem essa metade alguém leria «a vista ganha sempre» como lei, e escreveria
a próxima cura contra ela.*

### §57.4 — ⚠️ A vista é o CANVAS, não a janela — e o erro é todo para o lado conservador

A `radius_ceiling_px` recebe o **viewport da escultura**, que é o sub-rectângulo em que ela desenha,
logo é sempre **menor** que os pares da tabela. ⇒ *a divergência real é **maior** que a que esta
tabela mede, nunca menor*. A tabela nomeia janelas porque são o que o dono reconhece; a conclusão
não muda de sinal por isso.

### §57.5 — ⛔⛔ A ORDEM das três asserções é load-bearing, e a 1.ª redacção estava errada

A 1.ª versão punha *«o tecto ainda é a diagonal»* **à frente** das duas comparações. Ela é a mais
apertada das três, logo seria **a única a disparar em toda mutação da lei** — e as outras duas
ficavam **impossíveis de matar**.

⚠️ *Uma asserção que nunca chega a ser a primeira a falhar é comentário com sintaxe de código* — a
lei que o estudo do dyntopo já tinha escrito nesta linha (o braço `Density => true` redundante com o
fallback, [handoff §26](HANDOFF_INTEGRACAO_line_sculpt3d_CONTORNO_2026-09-14.md)).

⇒ as duas **comparações** vêm primeiro (são o que a espec encomendou) e a da **forma da lei** fica
no fim, como tripwire para a mutação que satisfaz as duas e muda a lei mesmo assim (M3).

### §57.6 — Prova de mutação: **5 de 5 sangram**, e TRÊS isolam-se numa asserção só

| # | mutação | sangra em | `test result` |
|---|---|---|---|
| **M1** | `RADIUS_MAX_FRAC_OF_DIAGONAL` `1.0 → 0.1` | `…passa_a_pista…` (metade **≥ `500`**) | `0 passed; 2 failed` |
| **M2** | `RADIUS_MAX_FRAC_OF_DIAGONAL` `1.0 → 3.0` | `…passa_a_pista…` (metade **< `5 000`** — *a divergência*) | `0 passed; 2 failed` |
| **M3** | `hypot(w,h) → max(w,h)` | `…passa_a_pista…` (metade **«ainda é a diagonal»**) | **`1 passed; 1 failed`** |
| **M4** | `RADIUS_TRACK_MAX_PX` `5000 → 200` | `…aperta_o_raio…` (a pista oferece o digitável) | **`1 passed; 1 failed`** |
| **M5** | `RADIUS_TRACK_MAX_PX` `5000 → 10000` | `…aperta_o_raio…` (a **TROCA** nomeada) | **`1 passed; 1 failed`** |

⭐ **M3/M4/M5 derrubam UM teste e deixam o outro verde** — é isso que prova que cada metade é
carregada **sozinha**, e não que o par se cobre por acaso. M1 e M2 derrubam os dois de propósito
(baixar ou subir a fracção move também o lado da troca).

⭐ **M3 é a mais instrutiva:** com `max` no lugar de `hypot` as **duas comparações continuam a
fechar** em todas as seis vistas (`1024` ≥ 500 e < 5000, etc.) — só a asserção da FORMA acusa. *É a
mutação que a 1.ª ordem das asserções tornaria indistinguível das outras duas.*

### §57.7 — ⛔⛔ E o ARNÊS mentiu, com a forma exacta que este repo já registou três vezes

A 1.ª corrida devolveu **`0 de 5`**, com `correram 2` e `falharam: nenhum` em todas — sobre gates
que de facto sangravam.

**Causa:** o extractor procurava a linha de falha por `^ *<nome> ... FAILED` e o `cargo` escreve
`test <nome> ... FAILED`. O prefixo literal `test ` nunca casava ⇒ **toda** mutação se lia como
sobrevivente.

⚠️⚠️ *O cabeçalho do próprio script já avisava contra a espécie vizinha* (um filtro que casa zero
testes), e ele caiu na outra metade da mesma família. ⇒ o arnês passou a devolver **três sinais
independentes** — o **código de saída** do cargo · a linha `test result:` **verbatim** · os nomes
extraídos — e a verificação **cruza-os**: um `rc != 0` de que ele não consiga extrair um nome é
acusado como **DEFEITO DO ARNÊS**, nunca como sobrevivência.

⛔ **E houve uma segunda mentira, minha e de fora do arnês:** a 1.ª invocação canalizou a corrida por
`| tail -30`, que **destrói o código de saída** — o script morreu por caminho inexistente e o
harness leu `exit 0`. É a armadilha que o `CLAUDE.md` §2 escreve por extenso, paga na mesma sessão
em que eu a citei.

### §57.8 — ⛔ O que esta wave deliberadamente NÃO fez

- ⛔ **Não editou a espec.** A linha do G-20 na tabela §14.5 e a errata **Q2** continuam a descrever
  o estado anterior. A espec é **atestada**, e emendá-la é **acto do E** — a janela I não a reescreve.
  ⇒ **acto pendente do E:** trocar a linha do G-20 pelos dois nomes reais e marcar a Q2 como fechada,
  com a tabela do §57.3 como fonte.
- ⛔ **Não mexeu no tecto nem na pista.** `RADIUS_MAX_FRAC_OF_DIAGONAL` fica em `1.0` e
  `RADIUS_TRACK_MAX_PX` em `5000.0`; o gate **descreve** o que já shipa e proíbe a deriva silenciosa
  dos dois lados.
- ⛔ **Não tocou nas outras duas decisões abertas do dono** (a folga simétrica do *Scene Project*
  §10.3 · a componente transversal do arrasto da pose): as duas são **trocas** com a lei alternativa
  já escrita e medida, cada uma certa num caso e discutível no outro ⇒ são veredito de produto, e
  esta janela escolheu o item que não exigia um.

---

## §59 — ⭐⭐⭐ AS DUAS DECISÕES VIRARAM BOTÕES (*«coloque cada modo com opção»*)

O dono decidiu em 17/09, e a decisão **não foi escolher uma das leis**:

> *«Decisões: coloque cada modo com opção. Com um botão para mudar o modo.»*

⇒ as duas perguntas que estavam na fila dele deixam de ser um veredito e passam
a ser **um selector no painel**, com a lei da referência no valor de fábrica.

### §59.1 — A FOLGA do *Scene Project* (`Gap Law`)

A folga é subtraída de um `d` **com sinal**, e as duas leituras são:

| situação | `Signed` (de fábrica) | `Symmetric` |
|---|---|---|
| vão `+0,5`, folga `0,1` | `+0,4` — pára a `0,1` do alvo | `+0,4` — igual |
| vão `+0,5`, folga `0,6` | **`−0,1`** — o barro **AFASTA-SE** | **`0,0`** — não anda |
| vão `−0,5` (atrás), folga `0,1` | **`−0,6`** — **ULTRAPASSA** o alvo | **`−0,4`** — pára a `0,1` |

⭐ **Sob o rótulo «distância mínima» a 2.ª linha é defensável e a 3.ª não é** —
um acerto para trás faz a folga **crescer** a excursão. *Reproduzir o alvo
reproduz um defeito; divergir quebra a memória muscular de quem vem dele* ⇒ a
resposta certa era o botão, e não o meu voto.

A lei vive em [`ph2d_sculpt3d::FolgaModo`](../../../crates/ph2d-sculpt3d/src/folga_modo.rs),
e a `distancia` **delega**: ⛔ escrever `d − folga` ali seria a segunda resposta
à mesma pergunta. ⭐ **Com a folga em `0` as duas são a identidade ao bit**, logo
o chip é honesto quando o vão está no neutro: ele diz o que vai acontecer quando
a folga subir.

### §59.2 — O ARRASTO da POSE (`Drag Reads`)

| modo | `δ` (a alavanca do quociente `L/(L−δ)`) |
|---|---|
| `Along Bone` (de fábrica) | `dot(d, n̂)` — a projecção, a lei da espec §5.4/§5.5 |
| `Full Drag` | `sign(dot(d, n̂)) · ‖d‖` — a mão toda, com o sentido da projecção |

⭐⭐ **A propriedade que torna isto seguro:** com a mão a puxar **ao longo do
osso** as duas coincidem (`d = α·n̂` ⇒ `sign(α)·|α| = α`) — *o modo novo não abre
regime novo onde o corpus vive; ele só deixa de deitar fora o que a mão fez de
lado.* E o braço de fábrica chama **o mesmo código de antes**, logo as `69`
fixturas ficam intactas **por construção**.

### §59.3 — ⛔⛔ E a minha nota sobre ele estava ERRADA: são DUAS de três, não três

A lista que herdei dizia *«o `Scale`/`Translate`/`Squash` lêem só a componente
axial»*. Medido no código:

| deformação | o que ela lê |
|---|---|
| `Scale` | a **projecção** (pelo quociente de escala) |
| `Squash` | a **projecção** (o mesmo quociente) |
| **`Translate`** | ⭐ **o deslocamento INTEIRO** — `seg.origem = origem_inicial + g` |
| `Rotate` · `Twist` | nem uma nem outra: resolvem uma cadeia contra um alvo |

⇒ o chip só é pintado nas **duas** que consultam a lei
([`Brush::offers_pose_drag_law`]) — *num dos outros três ele não teria o que
governar, e um selector inerte é pior que um ausente*. ⚠️ **Recitar a nota teria
posto o botão em cima de um gesto que ele não governa**, que é um controlo morto
com cara de controlo vivo.

### §59.4 — Prova de mutação: **9 de 9 sangram**, e uma delas mudou um gate

| # | mutação | sangra em |
|---|---|---|
| **M1** | a lei simétrica vira a do alvo (botão decorativo) | `a_tabela_das_duas_leis_bate_celula_a_celula` |
| **M2** | a simétrica perde o piso e passa a INVERTER | `a_simetrica_nunca_inverte_o_sentido…` |
| **M3** | o PINCEL nasce com a outra lei de folga | `o_valor_de_fabrica_e_a_lei_do_alvo` |
| **M4** | a folga deixa de passar pela PORTA (crava a do alvo) | `a_folga_so_e_minima_no_sentido_de_avanco` |
| **M5** | a lei completa vira a projecção (botão decorativo) | `num_arrasto_transversal_as_duas_leis…` |
| **M6** | a projecção vira a lei completa (trocam de lado) | `num_arrasto_transversal_as_duas_leis…` |
| **M7** | o default do ENUM da pose troca | `o_arrasto_de_fabrica_e_a_projeccao_no_osso` |
| **M8** | o chip é pintado em TODA deformação (selector inerte) | `every_pose_control_is_clickable_where_it_is_drawn` |
| **M9** | o PINCEL nasce com a outra lei de arrasto | `o_pincel_nasce_com_a_projeccao_no_osso` |

⛔⛔ **A M3 SOBREVIVEU na 1.ª ronda, e o que ela expôs é o gate a medir a coisa
errada:** ele afirmava `FolgaModo::default()` — o `#[default]` do **enum** — e o
que SHIPA é `Brush::default().folga_modo`. Trocar o campo do pincel deixava-o
**verde**, e é exactamente essa troca que faria as `14` fixturas do oráculo medir
outro pincel. ⇒ *o `#[default]` do enum é uma conveniência de escrita; o campo do
pincel é o produto*, e eram **duas perguntas a partilhar uma asserção**. Hoje são
duas, e a irmã na pose (M9) nasceu da mesma leitura.

---

## §60 — ⛔⛔⛔ UM TESTE QUE ABORTAVA E SE LIA COMO «0 FALHARAM» (pré-existente)

`seam::every_command_reaches_the_shell` (`ph2d-panel-sculpt3d`) morria com
**`SIGABRT` por estouro de pilha** no perfil `dev` e **PASSAVA** no `ci-test`.

⇒ o CI e o `ship.sh` (que correm `--cargo-profile ci-test`) **nunca o viam**, e
quem corria o caminho **documentado** da corrida dirigida — o
`scripts/cargo-test-narrow.sh`, que é `dev` — via o binário inteiro morrer com o
script a imprimir **«✗ … 0 falharam · 32 passaram»**. *Um teste que aborta e se
lê como zero falhas é pior que um vermelho.*

**PRÉ-EXISTENTE, e medido em vez de suposto:** com as quatro crates postas na
versão do `main` (`git checkout main -- crates/ph2d-{panel-sculpt3d,sculpt3d,pose,i18n}`)
o estouro reproduz **igual** ⇒ zero relação com os dois botões desta jornada.
⚠️ Antes disso eu tinha tirado os meus dois braços do despacho e ele continuou a
estourar — *a primeira experiência já dizia que não era meu, e a segunda disse de
quem era.*

**A causa, por sonda descartável:**

| tipo | tamanho |
|---|---|
| `Sculpt3dIntent` | **9 608 bytes** (a variante `SetUi` carrega um `Sculpt3dUi`) |
| `Sculpt3dUi` | 9 608 bytes |
| `Sculpt3dSnapshot` | 9 688 bytes |

O teste materializava um array literal de **24** intents ⇒ **~230 KB**, e uma
build sem optimização copia-o vezes suficientes (o literal · o `IntoIterator` ·
a desestruturação por iteração) para passar os `2 MB` da pilha de uma thread do
`libtest`.

⇒ a lista passa a guardar **construtores** (`fn() -> Sculpt3dIntent`, 16 bytes
cada) e **um** intent fica vivo de cada vez. ⛔ **A cura NÃO é encolher o
`Sculpt3dIntent`**: ele é grande porque carrega o retrato inteiro, que é o
desenho — *o defeito era o TESTE materializar vinte e quatro de uma vez*.

**Medido depois:** `105 de 105` verdes no perfil `dev`, onde o binário abortava.

⚠️⚠️ **Para o integrador, e vale para além desta crate:** um teste pode ser
**vermelho só no perfil `dev`** e o portão inteiro deste repo não o ver. Quem
correr `cargo-test-narrow.sh` e ler *«0 falharam»* com um `✗` ao lado está a ler
um binário que ABORTOU — ⛔ o `✗` é o sinal, e a contagem não.

---

## §58 — 📦 PARA O AGENTE INTEGRADOR

### §58.1 — Os factos da linha

| grandeza | valor |
|---|---|
| base | `main` = `3090cac3f` (rebase por **fast-forward**, zero commits locais antes desta wave) |
| ficheiros de PRODUTO tocados | **zero** |
| ficheiros tocados | `crates/ph2d-app-sculpt3d/src/rulers.rs` (`+130`, todo dentro de `#[cfg(test)] mod tests`) |
| `PROJECT_SCHEMA` · `FIELD_DOC_VERSION` · `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` | **não se mexem** |
| os três registos de componentes (`ph2d-ecs` + os dois espelhos) | **não se mexem** |
| contratos congelados (§6) | **zero** |
| ADR | **zero** |
| pacote externo novo | **zero** |
| portas públicas novas | **zero** (o gate lê `ph2d_panel_sculpt3d::rows::row_for`, que já era `pub`) |

### §58.2 — O que NÃO pode colidir

O diff vive **num só ficheiro**, dentro do `mod tests`, e as duas únicas leituras que atravessam
crate (`rows::row_for` e `ids::SCULPT3D_RADIUS`) são **pré-existentes e públicas**. Não há símbolo
novo, não há `mod` novo, não há entrada em lista partilhada.

### §58.3 — ⚠️ O que só a árvore COMBINADA pode reprovar

- **`RADIUS_TRACK_MAX_PX`** vive na `ph2d-panel-sculpt3d` e o gate novo **lê-o**. Se outra linha
  mexer nesse número, o `o_que_aperta_o_raio_e_a_vista_nunca_o_widget` reprova — **e é para isso que
  ele existe**. A cura é ler o §57.3 antes de mexer, nunca afrouxar a asserção.
- **Tecto de LOC:** o `rulers.rs` vai de `149` para `279` linhas, muito abaixo do tecto. Sem risco de
  acumulação.
- Os censos de texto (HR-15) não são acordados: **zero literais de UI novos** (o diff é teste).

### §58.4 — Portão de fecho (MEDIDO)

| etapa | resultado |
|---|---|
| `nextest-impacted` | **14 400 / 14 400** verdes (12 526 saltados) |
| `clippy --all-targets -D warnings` (`ph2d-app-sculpt3d` + `ph2d-panel-sculpt3d`) | **zero** avisos |
| censos da **árvore COMBINADA** (HR-15 + tectos de LOC) | **90 / 90** verdes |
| prova de mutação | **5 de 5** sangram, **3** isolam-se numa asserção (§57.6) |
| tecto de LOC do ficheiro tocado | `rulers.rs` `149 → 279`, folga larga |

### §58.4-bis — ⛔ As VASSOURAS acusam, e as acusações são PRÉ-EXISTENTES

**5 das 9** vassouras vivas acusam, sobre o conjunto de caminhos
`ph2d-app-sculpt3d · ph2d-panel-sculpt3d · ph2d-sculpt3d · ph2d-mesh · ph2d-boundary · ph2d-pose · ph2d-cloth`:

| vassoura | ficheiros acusados |
|---|---|
| `blender-cloth` | `ph2d-sculpt3d/src/verb_scrape_tests.rs` |
| `blender-pincel-afiado` | `ph2d-sculpt3d/src/brush_verb_filter.rs` |
| `blender-pull` | `ph2d-panel-sculpt3d/src/ids/sculpt3d.rs` · `…/rows.rs` · `ph2d-sculpt3d/src/brush_default.rs` · `…/brush_magnitudes.rs` · `…/brush.rs` · `…/stroke_shape.rs` · `…/verb_strip_law_tests.rs` · `…/verb_strip_tests.rs` |
| `blender-trim-pincel` | `ph2d-sculpt3d/src/verb_scrape_tests.rs` |
| `blender-trim` | `ph2d-app-sculpt3d/src/host_contract_tests.rs` · `…/patch_valence.rs` · `…/undo_plano_tests.rs` |

⭐ **O diff desta linha é UM ficheiro** — `crates/ph2d-app-sculpt3d/src/rulers.rs` — e **nenhum** dos
13 acusados é ele. ⇒ **zero adições desta linha**; são todas propriedade do `main`.

⚠️ **E o número não bate com o do fecho anterior** (§56.5 de 16/09 registou **três** acusações): ou o
conjunto de caminhos era mais estreito, ou as vassouras foram estendidas desde então — *o sweep é
propriedade do PAR (código, vassoura)*, e os agentes **E** estendem-nas a cada emenda. **A
reconciliação e a triagem são do R**, não da janela I, que não pode ler o ledger para decidir.

⛔⛔ **INCIDENTE DE PROCESSO desta janela, registado:** a 1.ª montagem do portão **redirigiu a saída
do sweep para um ficheiro de log** — e ela contém as expressões do alvo que casaram. O
`cleanroom-sweep.sh` decodifica a vassoura **só em memória** exactamente para que nada do alvo toque
o disco, e o redireccionamento desfez isso. ⇒ os dois logs foram **sobrescritos** assim que o defeito
foi visto, e o instrumento passou a consumir a saída **em pipe**, deixando sair só o veredito e os
NOMES de ficheiro (`scratchpad/vassouras.sh`). *Uma cura que abre um caminho novo herda as cegueiras
que o caminho antigo já tinha pago* — aqui foi o contrário: um caminho novo **desfez** uma cura que o
instrumento já tinha.

### §58.5 — Como reproduzir o veredito

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && bash scripts/ph2d-run.sh bash scripts/cargo-test-narrow.sh ph2d-app-sculpt3d -- tecto_do_raio aperta_o_raio
```

### §58.6 — ⏳ O que fica ABERTO, e de quem é

- **E:** a emenda da espec (§57.8) · regenerar as `14` `fab_*` com pincel nosso · os `25` cabeçalhos
  com `o_que_ela_fixa` vazio · o salto de `20` px sem fixtura (errata Q8) · as **6** fixturas do
  *Scene Project* que precisam da geometria do alvo.
- **Dono:** a **folga simétrica** do *Scene Project* (§10.3 — a lei alternativa está escrita e
  medida, ainda `#[cfg(test)]` e sem chamador de produto; *cada uma está certa num caso e discutível
  no outro*) · a **componente transversal** do arrasto da pose (espec §5.4/§5.5) · e, em menor grau,
  desenhar a fileira da CURVA **desactivada** onde hoje ela é pintada sempre com a razão à vista
  (desenho novo).
  ⚠️ **Auditado contra o código em 17/09 e ENCOLHEU:** a lista que eu herdei dizia *«o `Strength` e a
  curva do `Density`»* e o `Strength` foi **escondido em 15/09**, com o `Brush::curva_inerte` a dar a
  razão da outra metade — *o §5 acumula trabalho já pago, e recitar a nota manda reconstruí-lo*.
- **R:** o `tip_roundness` (medido: `14` no merge-base, `14` no HEAD, **zero** adições desta linha)
  · o `NoError` público do motor de booleana.
- **Nossas, nomeadas:** `dureza05` do *Scene Project* (`3` vértices de `301` na borda móvel da
  pegada) · a composição por dab · o `Visibility` que não propaga a descendentes · as cunhas finas
  da costura do Box Trim · os gates G-5, G-6, G-13 do pincel de plano.
