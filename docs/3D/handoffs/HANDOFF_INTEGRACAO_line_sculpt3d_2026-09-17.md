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

## §61 — ⛔⛔⛔ AS DUAS OPÇÕES DA §59 SAÍRAM, as duas por VEREDITO DO DONO

A §59 fica escrita **inteira e sem emenda** — ela é a memória do que foi
construído, medido e julgado. O que mudou foi o veredito, e ele veio em duas
frases, no mesmo dia em que os botões nasceram.

### §61.1 — O `Gap Law` (`ad920991b`)

> *«não gostei dessas opções. melhor a implementação anterior»* — e, perguntado
> o que tinha testado: *«testei apenaS Gap Law. Não vi onde ficam Drag reads»*.

A árvore voltou ao estado **anterior aos botões**, conferida ficheiro a ficheiro
(`git diff 95bef6f49 --stat` devolve **um** ficheiro). ⭐ **A lei alternativa
FICA viva sob `#[cfg(test)]`** (`projectar::folga_simetrica`) com a recusa
medida escrita ao lado — *apagá-la levava a medição junto, e a tabela da §59.1
é o que impede a próxima janela de a reconstruir do zero*.

⚠️ **Duas coisas NÃO foram revertidas, e as duas por medição:**

1. **A subida das fileiras do pincel no painel.** Ela nasceu por causa do
   `Gap Law` e vale por si: o `Ray Direction` e o `Search Both Ways` nasciam em
   `y = 967` e `1 001` num encaixe de `~880` — *já estavam abaixo da dobra antes
   de esta wave existir*. ⇒ *uma cura não deixa de ser certa por a queixa que a
   motivou ter mudado de assunto.*
2. **O gate da dobra** (`os_controlos_proprios_de_um_pincel_cabem_no_encaixe`),
   com a catraca dos cinco pincéis que passam a dobra hoje.

### §61.2 — O `Drag Reads`

> *«Full drag parece ser o único necessário..»*

⇒ o chip **sai do painel** e o produto crava [`ph2d_pose::Arrasto::Completo`] em
`PoseControlos::lei`. Saem com ele: a fileira do pintor, o braço do `event.rs`,
o array de ids, a linha do `populate`, a chave de i18n, o censo do painel, a
metade do gate de costura e o `Brush::offers_pose_drag_law`.

⛔⛔ **A LEI NÃO SAI, e a razão é o corpus:** `Controlos::default()` continua a
nascer em [`Arrasto::AoLongoDoOsso`], que é **o que as `69` fixturas do oráculo
alimentam** — *cravar a outra ali re-baseia o corpus inteiro em silêncio*. O
gate novo `o_produto_le_o_arrasto_inteiro_e_o_oraculo_fica_na_projeccao` afirma
as duas pontas **e** que elas discordam num arrasto transversal (senão a
divergência declarada não descreve nada).

⚠️ **É uma DIVERGÊNCIA DECLARADA da espec §5.4/§5.5**, que manda projectar o
arrasto no osso — e é de PRODUTO: puxar de lado deixou de ser deitado fora.
⚠️ Só as duas deformações do quociente de escala mudam de saída; a translação já
lia o deslocamento inteiro e as duas rotações resolvem uma cadeia.

⭐ **A premissa morta está à vista no diff:** o
`o_pincel_nasce_com_a_projeccao_no_osso` foi **substituído** e o nome está em
`MEMORIAS` do censo dos gates nomeados, com o motivo. *Uma premissa que morre em
doze horas é a melhor prova de que tinha de estar num gate.*

⏳ **O roteiro da `=41` foi reescrito**: o passo (8) deixa de ensinar que
`Scale`/`Squash` leem só a componente axial — *uma cena que ensina o contrário
do que acontece é pior que uma cena ausente*.

---

## §62 — ⭐⭐⭐ O TECTO DO `Weight smoothing` DA POSE: `100 → 300`, e o número é MEDIDO

Report do dono, com duas fotos:

> *«por que essas reentrâncias com pose? Por que não é mais regular a borda da
> deformação? Porque mesmo com Weight Smoothing no máximo não consigo uma
> transição mais suave? Poderia aumentar o máximo do slider em 3x?»*

**Zero linhas de lei.** O diff do motor é **um** número (`SUAVIZACOES_MAX`), a
row do painel a lê-lo, e três doc-comments.

### §62.1 — As reentrâncias são FACES VIRADAS DO AVESSO, e vivem TODAS na banda

A região da pose nasce **binária** (a varredura do §2.2 escreve `1` em quem
alcança e `0` no resto) e o único alisador é a difusão de Jacobi do §4 ⇒ a banda
entre os dois tem de **absorver a rotação toda**, e estreita de mais ela dobra
sobre si mesma.

Medido pelo caminho do produto (`sonda_de_onde_vivem_as_viradas`, esfera de
`97 922` vértices, arrasto `0,6`): das faces viradas, **`203/203`, `878/878`,
`1 336/1 336`, `801/801` e `10/10`** têm peso estritamente entre `0` e `1`.
**Zero** no miolo, **zero** fora da região.

### §62.2 — O knob conta ANÉIS DA MALHA, não raios do pincel

| `N` | banda, em **arestas** (`1 490` · `6 050` · `24 386` · `97 922` vértices) |
|---|---|
| `4` | `3,84` · `4,06` · `4,29` · `4,17` |
| `25` | `10,81` · `10,04` · `10,05` · `10,07` |
| `100` | — · `23,97` · `20,11` · `20,04` |
| `300` | — · — · `37,46` · `34,81` |

⇒ **banda ≈ `2,0 · √N` arestas**. Duas leituras, e as duas respondem ao dono: a
unidade é a **MALHA** (na densidade de fábrica o tecto antigo comprava `0,285`
do raio do pincel e o novo compra `0,495`), e a **raiz quadrada** quer dizer que
*triplicar o número dá `√3 ≈ 1,73×` de suavidade, nunca `3×`*.

⚠️ Os traços são as células em que a difusão **come o próprio miolo** (sem
condição de fronteira): a `1 490` vértices o núcleo cai de `1,0000` para `0,5498`
em `N = 300`. Na peça de fábrica ele fica em `1,0000` até `300`.

### §62.3 — Porque `300`, e não «três vezes o que era»

Faces viradas por (arrasto, `N`), pelo produto, na esfera de `97 922`:

| arrasto | `N=0` | `4` (fábrica) | `25` | `100` (tecto antigo) | `200` | **`300`** | `600` |
|---|---|---|---|---|---|---|---|
| `0,10` | 193 | 498 | 89 | **0** | 0 | **0** | 0 |
| `0,20` | 200 | 724 | 842 | **0** | 0 | **0** | 0 |
| `0,40` | 201 | 832 | 1 222 | 537 | **0** | **0** | 0 |
| `0,60` | 203 | 878 | 1 336 | 801 | 10 | **0** | 0 |
| `0,90` | 203 | 896 | 1 385 | 863 | 56 | **0** | 0 |
| `1,20` | 203 | 931 | 1 386 | 820 | 107 | **0** | 0 |

⭐ **`300` é a primeira coluna que lê `0` em TODA a linha**, até um arrasto de
`1,20` (= um raio e meio). O tecto antigo deixava `801` faces viradas no arrasto
que o dono fotografou. ⛔ Acima de `300` não há regime novo: a `600` já era zero.

**Custo:** `0,133 ms` por iteração por `98 k` vértices, **no pen-down e uma vez
por traço** — `12,83 ms` no tecto antigo e **`39,85 ms`** no novo (`--release`).

### §62.4 — Os dois gates e a prova

- `o_tecto_das_suavizacoes_e_onde_a_dobra_morre` — **três** metades: o controlo
  positivo (no tecto antigo a malha DOBRA — sem ele o gate afirmava o nada), o
  zero no tecto de hoje, e a metade da **licença** (a dois terços do tecto ainda
  dobra ⇒ o número que shipa é o menor da escada que chega a zero).
- `a_banda_conta_aneis_da_malha_e_nao_raios_do_pincel` — afirma o **defeito** de
  propósito, como o irmão das duas colunas do dyntopo: em arestas as duas
  densidades concordam (`1,003`) e em raios de pincel discordam por `2,007`.
  *No dia em que a banda se ancorar no raio, ele reprova e a premissa morre à
  vista no diff.*

⚠️ **A densidade da fixtura é load-bearing:** no tecto antigo as faces viradas
leem `0` a `1 490`, `0` a `6 050`, `0` a `24 386` e **`1 390`** a `97 922` — *um
gate escrito numa esfera de teste barata ficaria verde sobre o defeito do
report*.

⏳ **ABERTO e nomeado:** que o knob conte anéis é o defeito de fundo. A cura
(banda em raios de pincel, contagem derivada da densidade) **não é afordável com
esta lei** — `N ∝ (banda/aresta)²` e o custo é `O(V·N)` ⇒ `O(V²)` a banda
constante, e uma banda de **um** raio na peça de fábrica pede `~1 200` iterações.
*A cura de fundo é outra lei de peso, e é decisão do dono porque o corpus do
oráculo mede esta.*

### §62.5 — Os instrumentos ficam versionados

Três sondas `#[ignore]` em `pose_fronteira_tests.rs` produziram as três tabelas
acima: `sonda_da_banda`, `sonda_das_viradas`, `sonda_de_onde_vivem_as_viradas`.
*Uma régua que produziu um número publicado e não é versionada faz o número
envelhecer sem testemunha.*

---

## §63 — ⭐⭐⭐ OS DOIS GATES QUE FALTAVAM À ESPEC DO PLANO — e o G-5 apanhou o G-6

Ordem do dono: *«resolva o que está em aberto»*. A lista foi **auditada contra o
código antes de se pegar num item**, e encolheu antes de crescer: ela dizia
*«os gates G-5, G-6, G-13»* e o **G-6 já existia** desde a manhã.

### §63.1 — G-5, o gate DISCRIMINANTE

`o_centro_da_area_e_a_media_das_posicoes_puxadas_para_o_cursor`, em duas metades:

* **(a)** sobre as `11` células de `lei/*`, a lei do produto cai no plano que o
  **alvo** de facto usou — recuperado por ajuste da saída DELE. Pior `6,521e-8`.
* **(b)** sobre as `8` que discriminam, as **três** candidatas rejeitadas têm de
  **reprovar** por `≥ 1e-3`.

⭐⭐⭐ **O piso medido é `0,00269`, no `lei_area20` — EXACTAMENTE o número que a
§2.2 publica, e na célula que ela nomeia.** As três candidatas foram
reprogramadas aqui a partir do *enunciado* da espec (a lei da casa, a média
simples sobre `R_c`, a ponderada sobre `R_c`) e o piso saiu igual ao dela: *é
essa coincidência que prova que são as MESMAS três, e não três leis parecidas.*

⚠️ **O piso de população tem DOIS números** (`11` medidas, `8` discriminantes):
sem o segundo, degenerar o corpus deixaria o gate verde a julgar três células. E
a metade **negativa** afirma que as três não-discriminantes continuam a não
discriminar — *uma tabela que deixa de descrever o corpus é a licença do §5.0*.

⛔ **As três candidatas vivem na BANCADA e não no produto:** são leis que a
medição recusou, e pô-las atrás de uma porta do motor daria três caminhos vivos
para uma pergunta que já tem resposta.

### §63.2 — ⛔⛔⛔ E ele apanhou o G-6 a medir OUTRA LEI

O G-5 reprovou à primeira: `6,788e-2` fora do plano do oráculo — **a mesma ordem
de grandeza das candidatas que a espec REJEITA**. A causa não era a lei:

> a porta de bancada `plano_do_dab_para_teste` chamava o `fit_plane`, que é **o
> plano da CASA** — o dos quatro verbos portados da referência MIT. O
> `Verb::Plane` usa o `plano_da_pegada`, e as duas diferem **`17,1 %` do raio no
> centro** e **`31,2°` na normal** (números do cabeçalho daquele módulo).

⚠️⚠️ **O G-6 ficava VERDE por cima disso**, porque a barra dele é `0,75` raios —
folga que engole `17 %`. ⇒ *uma barra larga não é só uma afirmação fraca: é o
sítio onde uma régua errada sobrevive*, e a defesa é ter na mesma porta um gate
cuja barra não tenha folga nenhuma.

⛔⛔ **E o doc que eu escrevi de manhã dava uma explicação FALSA e plausível:** o
G-6 lia `0,2645` contra os `0,4947` que a espec publica, e a nota ao lado dizia
que a diferença era *«a régua ser mais larga»*. Com a porta corrigida ele lê
**`0,4947` ao dígito**, na célula que a espec nomeia. *O número não foi ajustado;
ele apareceu* — e é isso que prova que a régua passou a medir a lei certa.

### §63.3 — A porta foi reescrita DUAS vezes, e a segunda também estava errada

A 2.ª redacção chamava o `plano_da_pegada` **fora de um dab** — e aquele lê a
**pegada**, que é o dab que a monta. Fora dele a pegada é a do dab ANTERIOR (ou
está vazia): o `lei_area20`, cujo raio de área é `2 R`, lia `2,405e-2`.

⇒ a porta honesta é `plano_do_ultimo_dab_para_teste()`, que **LÊ o que o gesto
guardou**. ⭐ De graça ela fica sem `&mut` e sem tocar na memória do plano (espec
§6), que a 2.ª redacção tinha de repor à mão. ⚠️ Ela devolve `Option` e **não** um
plano emprestado de outra lei — que era o defeito nº 1.

### §63.4 — G-13, o gate contra o KNOB MORTO, com barra de MAGNITUDE

`cada_verbo_le_o_corte_que_o_nosso_painel_lhe_oferece`: para cada verbo que o
painel mostre o corte ou o deslocamento, varrer o knob tem de mover o barro
`≥ 1e-4` — **`73×` abaixo** da menor mudança que o alvo produz num knob vivo
entre fixturas **publicadas** (`7,3e-03`).

⭐ **Ele é o irmão de MAGNITUDE do censo dos knobs**, que pergunta *«muda ao
bit?»* — pergunta com ponto cego: um knob que mova `1e-9` está vivo ao bit e
morto para o artista. *É a cegueira que o `Density` pagou com uma foto do dono.*

### §63.5 — ⭐⭐⭐ E o G-13 achou um CONTROLO INALCANÇÁVEL

A população impressa saiu com **`6`** células e sem `Plane × plane_offset`. O
pincel de plano **lê** aquele knob desde que existe (`lift = raio × plane_offset`,
espec §2.4, com duas fixturas a exercitá-lo) e o painel **não lhe pintava a
fileira**: o `uses_plane()` não o continha, e o `show` daquela row é o **único
consumidor de produto** do predicado.

⛔⛔ **É a coluna «o painel esconde × o knob CHEGA» da tabela do próprio censo — o
INALCANÇÁVEL, cuja cura é OPOSTA à do morto.** ⚠️ *Um censo que procura knobs
mortos nunca o encontraria: ele não é pintado, logo não entra na varredura.* O
que o revelou foi o gate **imprimir** a população em vez de a contar em silêncio.

⇒ o `uses_plane()` passou a conter o `Verb::Plane` e a população foi a **`7`**.

⛔⛔⛔ **E o dono REPROVOU essa cura na mesma hora — ver a §64.** A ausência era
**certa**; o que lhe faltava era o motivo escrito. *Curei uma lacuna de
documentação criando um controlo que destrói a peça.*

⚠️⚠️ **E o gate que devia ter apanhado isto estava VERDE:** o
`the_families_that_the_ui_asks_about_agree_with_the_verb_list` compara a lista
consigo mesma ⇒ ele afirma que ela não **MUDOU**, nunca que ela está **CERTA**. O
doc do próprio predicado já dizia, por escrito, *«ficou verde sobre a omissão até
alguém a procurar»*.

⛔ **E o predicado NÃO é de roteamento:** ele responde *«este verbo lê o
`plane_offset`»*, e não *«que lei de plano ele corre»* — quem o usar para rotear
entrega o plano errado, que é exactamente o defeito da §63.2.

### §63.6 — O tecto de LOC, curado por CORTE

O `brush_verb_predicados.rs` foi a `725` de `700` ao registar porquê o pincel de
plano entrou na família. ⇒ `brush_verb_familias.rs` (`662` + `82`), com as **três
famílias de leitura que a UI pergunta** — máscara, plano e anel, exactamente as
que o gate acima enumera. ⛔ Nenhuma entrada nova no `FILE_OVERAGE_OK`.

### §63.7 — Prova

**7 de 7** mutações sangram, com controlo negativo verde: a lei invertida (o peso
multiplica em vez de puxar) · o centro a virar a média simples sobre `R_c` · o
`R_c` a virar o raio da normal · **o plano ancorado na ORIGEM** (o defeito que o
G-6 existe para apanhar — e que agora sangra sobre a lei certa) · a barra da
discriminação acima do piso publicado · a coluna `discrimina?` a mentir numa
célula · e uma candidata rejeitada a virar a do produto (que torna a metade (b)
vácua).

---

## §64 — ⛔⛔⛔ O DONO REPROVOU O CONTROLO QUE EU ACABARA DE LIGAR (§63.5)

> *«Plane Offset com resultado completamente errado. Primeira etapa: Plane Offset
> = 0 — correto. Segunda: Plane Offset = −0,5 — bizarro.»* (três fotos: a peça
> passa de bossas a lajes facetadas e depois a um barril irreconhecível)

### §64.1 — O que a medição diz, e ela ILIBA a lei

Sonda versionada `diag_o_deslocamento_do_plano` (a peça da cena `=47`: bossas de
amplitude `0,09` sobre raio `1`, pincel `0,35`, oito dabs, **valores de fábrica**
do verbo — tectos `1/0`, do perfil *aparar*):

| deslocamento | vértices movidos | corte máximo |
|---|---|---|
| `−0,50` | `739` | **`0,3552`** (`2,2×` o corte de `0`) |
| `−0,20` | `596` | `0,2732` |
| `−0,10` | `456` | `0,2320` |
| `−0,05` | `414` | `0,1942` |
| **`0`** | `380` | `0,1633` |
| `+0,05` | `354` | `0,1418` |
| `+0,10` | `333` | `0,1216` |
| `+0,20` | `216` | `0,0828` |
| `+0,50` | **`2`** | **`0,0007`** — inerte |

⭐ **A lei está certa e o knob é MONÓTONO** — nove posições, uma curva sem
degraus. ⇒ *o defeito não é a lei nem a fiação: é a FAIXA.*

### §64.2 — A assimetria é dos TECTOS, e isso separa duas leituras

A mesma varredura com os tectos **bilaterais** (`1/1`, o perfil *achatar*):

| deslocamento | movidos | corte máximo |
|---|---|---|
| `0` | `649` | `0,1863` |
| `−0,50` | `794` | `0,3559` |
| `+0,50` | **`478`** | **`0,1981`** — **vivo** |

⇒ *com um tecto de um lado só, afastar o plano do material deixa de haver
material para tirar* — a `+0,5` o plano já limpou o relevo inteiro. **A metade
morta do slider é dos valores de fábrica deste pincel, não do knob.**

### §64.3 — Porque a cura é RETIRAR, e não estreitar

A fileira herdaria a faixa `−1 … +1` dos quatro verbos da casa. Nela, com os
valores que o pincel ship: **metade do curso é inerte** e a outra **dobra o corte
por passagem** — que é a foto do dono, ao fim de algumas passagens.

⛔ **E eu não tenho de onde derivar uma faixa mais estreita.** O que a limita aqui
é a **altura do relevo em raios de pincel**, que é da PEÇA e não do produto; e a
faixa do alvo para este controlo **neste pincel** nunca foi medida — a §14.2 da
espec publica só o valor de fábrica (`0`). *Escolher um número sem isso é o
palpite que o `CLAUDE.md` §0.0 proíbe por escrito.*

### §64.4 — ⚠️⚠️ A LIÇÃO, e ela é sobre o meu próprio instrumento

O G-13 **imprimiu** a população e mostrou a ausência; eu li a ausência como um
**controlo inalcançável** e liguei-a. ⛔ *Mas um censo de knobs MORTOS não podia
proteger disto:* ele pergunta se o knob **move** o barro, e este move **de mais**.

⇒ **«o painel esconde × o knob CHEGA» tem DUAS leituras, e elas são opostas:** o
*inalcançável* (cura: ligar) e a *ausência DECIDIDA* (cura: escrever o motivo).
⚠️ **A sonda vê as duas iguais** — a mesma forma que o §5.0 já regista para o
morto contra o órfão, agora uma casa acima. *E o que me faltava para as separar
não era um gate: era a medição da FAIXA, que ninguém tinha corrido.*

### §64.5 — O que fica

* `uses_plane()` volta a `Flatten · Fill · Scrape · Clay`, **com a tabela medida
  no doc dele** — para ninguém a religar por leitura.
* A população do G-13 volta a `6`.
* Nasce `o_deslocamento_do_plano_nao_e_oferecido_ao_pincel_de_plano`, em **duas
  metades**: o painel não o pinta, **e a lei continua a lê-lo** (medido `1,225e-1`
  entre `0` e `−0,5`). ⛔ Sem a segunda, alguém leria a ausência como *«o verbo
  não tem deslocamento»* e apagaria a lei, levando as duas fixturas
  `lei_deslocado_*` e a §2.4 da espec com ela.
* A sonda `diag_o_deslocamento_do_plano` fica **versionada** — é ela que produziu
  as duas tabelas acima.

⏳ **DECISÃO DO DONO, com o preço na mesa:** se ele quiser o controlo, ele volta
no dia em que a faixa tiver de onde sair — e a pergunta que a destranca é *qual é
a faixa que o alvo oferece para este controlo NESTE pincel*, que é um acto do
**E** (correr o programa, não ler).

---

## §65 — ⭐⭐⭐ AS «CUNHAS FINAS» DO BOX TRIM: a nota era FALSA, e o defeito a sério era outro

> **Ordem do dono** (2026-09-17): *«As cunhas finas na costura do Box Trim»* — o
> item que os §46.4 e §47.6 deixaram abertos com a frase *«curá-las mexeria na
> malha da peça»*.

### §65.1 — ⛔⛔⛔ A primeira medição refutou a própria nota

O aberto dizia: *«os `~632` que sobram são cunhas finas onde a curva de
interseção passa rente a um vértice da peça»*. Medido pela porta do produto,
**a esfera de ENTRADA já tem `632` triângulos piores que `20`**, e `632` de
`632` têm `|y| > 0,99`: são o **leque do PÓLO** de uma esfera UV — a mesma
propriedade que o doc da `sculpt_sphere` descreve por escrito ao explicar porque
é que o módulo de escultura **não** abre com uma.

| | T | `>20` | p50 | p99 | MAX |
|---|---|---|---|---|---|
| PEÇA inteira | `49 612` | **`632`** | `2,75` | `25,65` | `25,72` |
| só a costura CRUA | `28 908` | `188` | `2,48` | `13,03` | `2 573 808,75` |
| só a costura LIMPA | `28 096` | **`8`** | `2,48` | `4,34` | `32,57` |
| SAÍDA inteira | `70 660` | `640` | `2,48` | `13,17` | `32,57` |

⇒ *a régua somava a peça inteira*, e a costura limpa é **melhor que a peça** em
todas as colunas menos o MAX. ⚠️⚠️ **E a sonda que já existia imprimia a
resposta:** o `diag_o_pico_na_borda` escreve `vértices ANTIGOS: 3/3` ao lado de
cada cunha — *quando uma página imprime o que desmente a hipótese, isso É o
achado, e eu li aquilo como confirmação da minha.*

### §65.2 — ⛔⛔ E a fixtura CENTRADA não continha o fenómeno (a oitava vez)

Com o círculo no meio da peça a borda do corte vive a `|z| = 0,8` e **nunca
encontra a silhueta**: ali a costura limpa mede `MAX 32,57`. Com o corte a
**sair pela beira** (`centro_x = 0,8`, que é a posição da foto do §47) ela mede
**`694,78`**. *A medição do §46 tinha sido feita toda na posição fácil.*

### §65.3 — ⭐⭐⭐ TRÊS réguas foram construídas e REFUTADAS antes de uma decidir

O aspecto **não é o que se vê**. As lascas que sobram são finas e **PLANAS**, e
uma face plana tem normal perfeita:

1. **desvio radial da normal da FACE** — as lascas leem `0,52°`, **melhor** que a
   mediana da costura sadia (`0,549°`);
2. **MÉDIA das normais das vizinhas** — mede a **QUINA** do corte, não a lasca
   (`p99 = 33°` sobre geometria correcta);
3. **MÍNIMO sobre as vizinhas** — separa a quina, e **o CONTROLO refutou-a**: a
   saída CRUA, com aspecto `2 573 809`, lê `1,15°`.

⭐ A que decide é a **normal do VÉRTICE contra a radial**, sobre a esfera onde a
resposta é exacta — é ela que o sombreamento usa, e a `ph2d_mesh::normals` soma
normais de face **unitárias** (*gather* sem peso de área) ⇒ **uma lasca vota com
peso cheio**. ⚠️ E ela precisou de **duas** correcções de população: `r > 0,995`
deixa entrar a **PAREDE** do corte, que junto da silhueta é quase tangente à
esfera. A população honesta é o vértice cujas faces estão **TODAS** a
`|r − 1| < 1e-4`.

### §65.4 — ⭐⭐⭐ Com a régua certa há UM defeito real, e ele fecha

| centro | peça | saída CRUA | limpa SEM a troca | **limpa** |
|---|---|---|---|---|
| `0,0` | `0,03°` (0) | `0,19°` (0) | `0,19°` (0) | `0,19°` (0) |
| `0,3` | `0,03°` (0) | **`28,40°` (2)** | **`18,37°` (1)** | **`0,89°` (0)** |
| `0,5` | `0,03°` (0) | `35,18°` (5) | `0,18°` (0) | `0,19°` (0) |
| `0,7` | `0,03°` (0) | `29,88°` (2) | `0,25°` (0) | `0,25°` (0) |
| `0,8` | `0,03°` (0) | `15,13°` (1) | `0,27°` (0) | `0,27°` (0) |
| `0,9` | `0,03°` (0) | `56,07°` (6) | `0,54°` (0) | `0,54°` (0) |

(entre parênteses, quantos vértices acima de `5°` — o vale entre `1` e `15` é de
duas ordens de grandeza, e é de lá que sai a barra.)

⭐ A cura é o 4.º passo da limpeza: **trocar a diagonal** do par onde a costura
deixou a lasca (`endireita_as_lascas`). De graça, as faces de **área ZERO** que o
motor deixa na costura — juntas em T que ele sela com uma face sem área — vão de
**`9` para `1`**: *a troca da diagonal de uma delas É a divisão em T.*

### §65.5 — ⛔⛔ A cerca do TAMANHO foi escrita por um gate VERMELHO do vizinho

A 1.ª redacção pedia só *«nenhuma das duas faces é inteiramente da PEÇA»*, e com
isso a troca **reescrevia a parede inteira de uma lâmina grossa** — o leque com
que o motor tapa a fronteira dela. O `a_face_que_o_corte_deixa_tem_a_densidade_da_peca`
reprovou **no CONTROLO**: a face grossa passou a medir `0,042` onde tinha de
medir `≥ 0,125`. ⭐⭐ *Uma cura que melhora o CONTROLO de outra cura apagou a
régua dela* — e o veredito certo é o que aquele gate já escreve: **a face grossa
de uma lâmina grossa cura-se ADENSANDO a lâmina (§45)**.

⇒ a fronteira é o **tamanho**, e ela separa por uma ordem de grandeza: na costura
a aresta longa de uma lasca mede `0,8`–`1,7` arestas da peça; na parede de uma
lâmina mínima mede **`~40`**. `ESCALA_DA_COSTURA = 3` cai num vazio de `23×`.

### §65.6 — ⭐⭐⭐ E uma mutação sobrevivente achou METADE DA ARQUITECTURA sem régua

Apagar a cerca *«nenhuma das duas faces pode ser inteiramente da PEÇA»* **não
partia um único teste**. O irmão que devia apanhá-la — o
`longe_do_corte_nenhum_vertice_se_move_um_bit` — mede **POSIÇÕES**, e *uma troca
de diagonal não move um vértice*: ela reescreve só a **LIGAÇÃO**.

⇒ `a_ligacao_da_peca_sobrevive_ao_corte`: toda face da saída feita só de vértices
antigos tem de **já existir na peça**. Medido: `41 890`–`42 626` faces dessas por
posição, **zero** inventadas. *A propriedade que decide a arquitectura desta
linha tinha metade sem régua desde que a limpeza existe.*

### §65.7 — As cercas, com quantas vezes cada uma DISPARA

Instrumentado sobre as seis posições do corte mais a lâmina mínima:

| cerca | disparos | régua |
|---|---|---|
| 1a — a face é toda da PEÇA | `9 494` | `a_ligacao_da_peca_sobrevive_ao_corte` |
| 1b — o TAMANHO da malha | só na lâmina mínima | `a_face_que_o_corte_deixa_tem_a_densidade_da_peca` |
| 2 — a aresta nova já existe | **`0`** | fixtura **sintética** |
| 3 — planura | `47` | ⚠️ **carril nomeado** |
| 4 — melhora estrita | `1` | ⚠️ **carril nomeado** |
| 5 — inversão | **`0`** | fixtura **sintética** |

⭐ **As duas de zero disparos ganharam fixtura sintética** (a mesma decisão que a
linha tomou com a almofada), e a da **inversão** só é construtível com uma face
de **área zero**: num par plano e coerentemente orientado o quadrilátero é
sempre convexo, logo a troca nunca inverte — *a inversão só é alcançável pelo
braço que trata a face degenerada como plana*.

⚠️⚠️ **DUAS mutações SOBREVIVEM de propósito, com a medição ao lado:**
* **cerca 3** (planura) — apagá-la muda o volume em `6e-9` relativo, *abaixo de
  toda barra deste repo*;
* **cerca 4** (melhora estrita) — apagá-la deixa a saída **byte-idêntica**
  (`−2,199e-6` · `−5,809e-6` · `−4,047e-6`, os mesmos dígitos), e não foi
  possível construir a fixtura que a torna observável: *o candidato é sempre uma
  lasca, e trocar a diagonal de uma lasca melhora por construção*.

As duas ficam pelos modos de falha que impedem (aparar a quina do corte; perder a
terminação do laço), **nomeadas no doc com o número**. ⛔ Quem as apagar tem de
trazer a fixtura que as torna observáveis.

### §65.8 — O que sobra, e porquê

Todo triângulo pior que `20` que fica é recusado por uma cerca **que existe por
um motivo**, e a sonda di-lo um a um: os `25,7` e os `30,0` são o **leque do
pólo da peça** (cerca 1a), e os `171`–`514` estão na **quina do corte**
(cerca 3). ⏳ Fica **uma** face de área zero, numa das seis posições.

### §65.9 — Números

* Gates novos: **5** (`a_costura_nao_estraga_o_sombreamento_da_casca` ·
  `a_ligacao_da_peca_sobrevive_ao_corte` · `a_limpeza_e_um_ponto_fixo` ·
  `o_controlo_num_par_sao_a_troca_acontece` + as duas sintéticas).
* Sondas versionadas: `diag_o_sombreamento_da_casca` · `diag_o_que_a_cerca_muda`.
* **Mutação: `8` corridas, `6` sangram, `2` sobrevivem NOMEADAS** (§65.7).
* Tecto de LOC: `costura.rs` `279 → 579` (tecto `700`).
* Portão: `nextest-impacted` **15 105/15 105** · censos da árvore COMBINADA
  **90/90** · clippy `-D warnings` zero · `cargo fmt --all --check` limpo.

---

## §66 — ⭐⭐⭐ «A TOPOLOGIA DAS BORDAS DO CORTE»: a borda serrilha, e a CENA é que a mostrava assim

> **Report do dono** (2026-09-17, depois do smoke da §65 aprovado): *«Smoke OK.
> O que não fica legal é a topologia das bordas do corte, pois com smooth não se
> consegue alisar»*.

### §66.1 — ⭐⭐⭐ O mecanismo, medido

A borda de um corte é feita de **duas espécies de ponto**: os que o motor põe
**na curva desenhada** e os **vértices da própria peça** que ele aproveita
quando estão perto. Medido na esfera de `50 k`, partindo a população:

| | vértices de borda | desvio da curva (p50, em arestas da malha) |
|---|---|---|
| os que estão na CURVA | `107` | **`0,0023`** |
| os que são da PEÇA | `322` | **`0,4954`** |

⇒ **a borda entra e sai do círculo desenhado de meia aresta em meia aresta**, com
picos de `1,2`. É isso que se vê.

⚠️ **E não é da nossa limpeza:** a saída CRUA do motor já traz `300` pontos da
peça na borda, e desligar o colapso de par misto move o `p50` de `0,4250` para
`0,4166` — *nada*. A escolha de qual vértice sobrevive num colapso misto também
é indiferente (`429` contra `434` pontos, mesmo `p50`).

### §66.2 — ⛔⛔ Nenhum pincel a pode curar, e a prova é o movimento IDEAL

* o **`Smooth` funciona**: ele amacia a quina (p90 `125,9° → 41,1°` numa
  passagem) e **não encrespa** a casca à volta (`p50 1,14°` antes e depois —
  ⚠️ a minha 1.ª leitura dizia `2° → 15°` e era um **conjunto de arestas
  poluído**, que misturava a quina do corte com a casca);
* mas ele **não põe a borda na curva**, porque *mover vértices não muda de que
  vértices o anel é feito*;
* e **nem o movimento ideal o faz**: pondo cada ponto da borda exactamente na
  curva, o desvio vai a `0` e o pior triângulo da casca salta de `25,7` para
  **`1 166`**.

⇒ *o que falta não são POSIÇÕES, são CÉLULAS* — a mesma frase que a linha do
quad remesh pagou duas vezes.

### §66.3 — ⭐⭐⭐ E a alavanca é a DENSIDADE, com o número

| peça | aresta | desvio da curva (mundo) p50 · p90 · MAX |
|---|---|---|
| `12 k` T | `0,0498` | `0,0277` · `0,0557` · `0,0665` |
| `30 k` T | `0,0314` | `0,0077` · `0,0277` · `0,0405` |
| **`50 k` T** (a `=46` até hoje) | `0,0242` | **`0,0103` · `0,0201` · `0,0298`** |
| `80 k` T | `0,0191` | `0,00007` · `0,0137` · `0,0209` |
| `120 k` T | `0,0156` | `0,00006` · `0,00007` · `0,0114` |
| **`196 k` T — a peça do MÓDULO** | `0,0124` | **`0,00000` · `0,00007` · `0,00007`** |

⇒ **na peça com que o artista de facto trabalha, a borda cai na curva.**

### §66.4 — ⛔⛔⛔ A CENA é que abria com uma peça `4×` mais grossa, e o cabeçalho dela nomeava UM recurso

O cabeçalho da `=46` defendia a esfera de `50 k` com o **relógio do corte**
(`58 ms` contra `380` no default) e **não tinha a coluna da BORDA** — que é a
que o dono julga. ⇒ *a cena estava a trocar a qualidade que ele avalia pela que
ela media*, e a `=46` deixou de escolher peça: ela abre com a do módulo, e o
corte custa os **`375,1 ms`** que o produto custa.

⚠️⚠️ **O precedente de 14/09 (*«meio travado»*) media OUTRO recurso:** ali era o
custo **por dab**, a 60 Hz, contra um *kill* de `8 ms`; aqui é uma espera **uma
vez por gesto**, numa operação destrutiva com desfazer. *Comparar os dois é o
que deixou a cena a ensinar uma borda que o artista não tem.*

### §66.5 — O que fica

* `scenes::mesh::peca_de_fabrica()` — **uma porta**, para que nenhuma cena
  reescreva a peça do módulo; o ramo da `=46` no roteador **saiu**.
* **`a_borda_do_corte_cai_na_curva_desenhada`** — a lei, com o CONTROLO a um
  quarto da densidade. ⚠️ **As duas metades correm sobre a MESMA primitiva**: a
  1.ª redacção comparava a `sculpt_sphere` com uma esfera UV e precisou de
  **duas** tolerâncias de *«isto está na casca»* (o raio da primeira varia
  `3,09 %`) — *com duas réguas o controlo leu `0,0000` e acusou a fixtura de não
  conter o fenómeno que ela continha*. A densidade da metade fina é **lida do
  produto**.
* **`a_cena_do_corte_nao_escolhe_a_propria_peca`** — substitui o
  `a_peca_da_cena_cabe_num_gesto_e_mostra_a_malha_do_corte`, cuja premissa
  morreu; a morte está registada em `MEMORIAS` (o censo dos gates nomeados
  apanhou a citação na mesma corrida, que é ele a funcionar).
* O roteiro da `=46` passa a mandar olhar para **a linha da borda** e diz a cura
  quando ela serrilha: *adensar com o `Density` por onde se vai cortar*.
* Sondas versionadas: `diag_o_zigue_zague_contra_a_densidade` ·
  `diag_a_borda_na_peca_de_fabrica` · `diag_o_relogio_do_corte` ·
  `diag_a_quina_contra_o_smooth` · `diag_o_smooth_encrespa_a_casca` ·
  `diag_o_premio_de_por_a_borda_na_curva` · `diag_de_quem_e_o_zigue_zague_da_borda`.

### §66.6 — Números

* **Mutação: `4` de `4` sangram.** ⚠️⚠️ **E o arnês mentiu primeiro:** a agulha da
  M4 casou **duas** vezes (o mesmo laço existe em duas funções da sonda), o
  `muta` abortou e o caso leu-se como **SOBREVIVEU** sobre produto correcto ⇒ o
  arnês passou a **abortar o caso** quando a mutação não entra, em vez de o
  reportar.
* Tecto de LOC curado por **CORTE**: a sonda tinha `893` linhas com cinco
  medições exploratórias que nenhuma tabela cita — saíram (`893 → 590`).
  ⛔ Nenhuma entrada no `FILE_OVERAGE_OK`.
* Portão: `nextest-impacted` **15 104 / 15 106**, com as **duas** reprovadas a
  serem membros **já nomeados** da família de flakes de fan-out do `CLAUDE.md`
  §5.0 — `the_cost_of_depth_is_linear_not_explosive` (`ph2d-timeline`) e
  `the_mask_stroke_cost_does_not_follow_the_canvas` (`ph2d-tool-painter`). As
  **três** assinaturas batem: o diff desta wave tem **zero linhas** nas duas
  crates; a corrida ANTERIOR da mesma árvore fechou **15 105 / 15 105** verde (o
  conjunto de reprovadas MUDOU); e sozinhas elas passam **7 de 7** ao longo da
  banda `load 18–37`, que **contém** a carga em que reprovaram.
  ⚠️⚠️ **A máquina nunca ficou calma:** uma espera com prazo chegou a ler abaixo
  de `6`, e à corrida seguinte já estava em `20` — há outra árvore a correr. ⇒ *o
  «sozinho» desta confirmação é «a esta carga», e é por isso que o número vai com
  o `loadavg` ao lado de cada corrida* (`CLAUDE.md` §5.0).
* ⏳ **ABERTO:** a borda só cai na curva onde a malha tem resolução para isso. A
  cura de fundo — **o corte partir as faces ao longo da curva em vez de
  aproveitar os vértices da peça** — vive dentro do motor de booleana
  (dependência Apache-2.0) e não é nossa; a nossa é adensar antes de cortar, e
  ela já tem ferramenta (`Density`).

---

## §67 — ⭐⭐⭐ A TRANSIÇÃO DA POSE DEIXA DE CONTAR ANÉIS DA MALHA E PASSA A SER UMA DISTÂNCIA NO BARRO

> **Ordem do dono** (2026-09-17), depois de perguntar o estado da arte do pincel
> e de eu propor três caminhos: *«1»* — o da faixa.

### §67.1 — O defeito, e porque nenhum tecto o curava

A região da pose nasce **binária** e o que a esbatia era a difusão de Jacobi do
§4: `N` passagens de «média dos vizinhos». A largura que isso compra conta-se em
**arestas da malha** (`≈ 2,0·√N`), logo:

* a mesma posição do slider dava transição **larga numa peça grossa e estreita
  numa fina** — medido pelo produto, a faixa em raios de pincel lê
  `0,417 · 0,211 · 0,108 · 0,054` sobre quatro densidades (`1 490` → `97 922`
  vértices), *partindo ao meio cada vez que a malha dobra*;
* **triplicar o número dava `√3 ≈ 1,73×`** de suavidade, nunca `3×`;
* numa peça grossa o topo do curso **diluía o núcleo** (`1,0000 → 0,5498`),
  porque a difusão não tem condição de fronteira;
* e o preço era `O(V·N)` **por segmento**.

⇒ o §62 subiu o tecto de `100` para `300` e curou o report; a nota que ficou
dizia que ancorar a faixa no raio *«não é afordável com esta lei»* — `N ∝
(banda/aresta)²` sobre `O(V·N)` dá `O(V²)`, e uma faixa de um raio pedia
`~1 200` passagens. **Está certo, e a saída era trocar a LEI.**

### §67.2 — ⭐⭐⭐ A lei nova: distância nas arestas, uma vez

[`ph2d_pose::pesos::por_distancia`] mede, para cada vértice, a distância à
fronteira do anel **andando pelas arestas** (Dijkstra, `O(V log V)`) e tira o
peso dela: `suave(0,5 − d/banda)`. A transição mede **exactamente `banda`**, e o
preço **não depende da largura pedida**.

| lei | faixa em RAIOS, nas quatro densidades | custo a `97 922` V |
|---|---|---|
| difusão `N = 4` (o de fábrica antigo) | `0,417 · 0,211 · 0,108 · 0,054` | `1,0 ms` |
| difusão `N = 300` (o tecto do §62) | — · — · `0,874` · `0,443` | **`47,6 ms`** |
| **distância `t = 1,0`** | **`0,507 · 0,501 · 0,501 · 0,502`** | **`3,4`–`4,7 ms`** |

⇒ **constante a `±0,6 %` sobre uma faixa de `8×` de aresta, e `14×` mais barata
que o tecto que substitui.** Em **arestas** as duas colunas trocam de lado (`4,4
· 8,8 · 17,6 · 35,3`), que é o que uma distância faz.

⚠️ **Ela esbate os campos CUMULATIVOS, não as diferenças**, e isso é o que
mantém a §3.3 de pé: o peso de um segmento é a diferença contra o estado depois
do anterior, e *uma diferença de dois campos binários não é binária* — esbatê-la
por «distância à fronteira» mediria a fronteira de um anel, não a do conjunto.

⚠️ **A fronteira fica a MEIA aresta** da que a atravessa; sem isso ela desloca-se
meia aresta para fora e a saída volta a depender da densidade — *o defeito
inteiro que a função existe para curar*. Há mutação a prová-lo.

### §67.3 — ⛔⛔⛔ E a primeira medição do valor de fábrica mediu OUTRO PROGRAMA

A varredura que escolheu `0,6` correu numa sonda com
[`ph2d_pose::Controlos::default()`], cuja lei de arrasto é a da espec
(**projectada no osso**). **O produto crava o arrasto INTEIRO** por veredito do
dono (§61), logo deforma mais e precisa de faixa mais larga: a `0,6` o gate
reprovou com **`606` faces viradas do avesso**.

Refeita pelo caminho do produto, a tabela dá o joelho em **`1,0`** — a primeira
coluna que lê `0` em toda a linha, até um arrasto de `1,20` (*um raio e meio*):

| arrasto | `0,6` | `0,7` | `0,8` | `0,9` | **`1,0`** |
|---|---|---|---|---|---|
| `0,40` | 398 | 160 | 0 | 0 | **0** |
| `0,60` | 606 | 380 | 119 | 0 | **0** |
| `1,20` | 715 | 552 | 310 | 19 | **0** |

*A régua é o PRODUTO* — quinta vez que esta linha o paga.

### §67.4 — O tecto, e de que recurso ele é

**`2,0`**, e o recurso é o **NÚCLEO**: a faixa é centrada na fronteira do anel,
logo uma larga de mais come o miolo. Medido, o peso do vértice sob o cursor é
`1,0000` até `2,0·R` e **`0,9394`** a `3,0·R` — acima daqui o pincel deixa de
mover inteiro o que está debaixo do dedo. ⭐ O gate tem as **duas** metades (no
tecto o núcleo sobrevive; meio acima dele não), senão um tecto a menos seria uma
faixa que o artista não alcança.

### §67.5 — ⛔ A divergência, declarada

A lei do alvo é a difusão, e é ela que os `69` traços do oráculo medem ⇒
[`ph2d_pose::Controlos::banda_do_peso`] nasce em **`None`** e a bancada pede-a
**campo a campo**. ⭐ *Um campo novo é erro de compilação ali*, logo ninguém lhe
pode dar um valor por omissão sem passar pela decisão. O produto ship a
distância porque ela ganha em todas as colunas medidas; **o oráculo continua
vivo e a medir a dele** — `57 de 69` a `≤ 1e-5`, sem uma fixtura mexida.

### §67.6 — O que mudou à volta

* O knob passa de `Weight smoothing` (contagem, `0..300`, `Pro`) a
  **`Transition`** (largura em raios, `0..2`, passo `0,05`, **`Basic`**).
  ⚠️ A subida de nível é medida: *um knob que separa a ferramenta boa da partida
  não é acabamento* — a `0,6` o mesmo gesto vira `606` faces e a `1,0` vira `0`.
* A chave de i18n e os dois ids seguem o nome (`pose_smoothings` →
  `pose_transition`): *uma chave que diz uma coisa e um rótulo que diz outra é a
  forma de o texto envelhecer sem ninguém ver*.
* **DOIS gates tiveram a premissa MORTA**, os dois registados em `MEMORIAS`:
  o `o_tecto_das_suavizacoes_e_onde_a_dobra_morre` e — ⭐ o mais bonito — o
  `a_banda_conta_aneis_da_malha_e_nao_raios_do_pincel`, que **afirmava o defeito
  de propósito** e trazia escrito *«no dia em que a banda passar a ancorar-se no
  raio do pincel ele reprova, e a premissa morre à vista no diff»*. O dia foi
  este, e o `a_faixa_mede_o_barro_e_nao_aneis_da_malha` afirma hoje as **duas
  mesmas colunas com os papéis trocados**.
* O roteiro da `=41` passa a mandar **descer** a `Transition` para ver os
  entalhes voltarem — *a metade negativa é o que torna a positiva uma
  afirmação*.

### §67.7 — Números

* Mutação **6 de 6 sangram** (a lei de volta à difusão · a largura abaixo do
  joelho · o tecto acima do núcleo · a fronteira fora da meia aresta · o sinal
  da distância · as duas leis ligadas ao mesmo tempo).
* Oráculo da pose **intacto**: `57 de 69`, os mesmos de antes.
* Portão: `nextest-impacted` **15 654 / 15 654** · censos da árvore COMBINADA
  **90 / 90** · clippy `-D warnings` zero · `fmt` limpo · vassouras `5 de 9` com
  os **mesmos 13 ficheiros pré-existentes** (⭐ a `blender-pose` fica **limpa**).
* ⏳ **ABERTO:** numa peça muito grossa a borda ainda dobra (`19` faces a `1 490`
  vértices, onde a faixa mede `2,8` arestas) — *uma transição não pode ser mais
  fina do que a malha*, e ali a cura é malha, a mesma frase da borda do corte.

---

## §68 — ⭐⭐⭐ «SURGEM ESTRIAS»: a frente da distância caminhava por ARESTAS, e uma malha só tem as direcções que tem

**Report do dono, 2026-09-17** (foto): *«está quase bom! mas surgem estrias»* — arcos paralelos na
casca à volta da região da pose, logo a seguir à wave do §67.

### §68.1 — A régua, e porque as duas que já existiam eram cegas

⛔⛔ **As duas réguas da faixa mediam a DOBRA, que é binária.** O
`a_transicao_de_fabrica_e_onde_a_dobra_morre` conta faces do avesso e já lia **`0`** no dia em que
ele fotografou; o `a_faixa_mede_o_barro_e_nao_aneis_da_malha` mede a LARGURA. *Nenhuma das duas vê
suavidade* — e uma estria não é uma face invertida, é uma quebra da **derivada**.

⇒ a régua é o **ângulo entre as normais de faces vizinhas**, sobre as arestas interiores cujas DUAS
faces tocam a faixa. Medida na esfera do report (`97 922` vértices, arrasto `0,6`):

| | casca por deformar | faixa, `transição 1,0` | núcleo |
|---|---|---|---|
| p50 | `0,703°` | **`1,460°`** | `0,724°` |
| p90 | `0,902°` | **`13,457°`** | — |
| p99 | `0,937°` | **`42,01°`** | `0,938°` |

⭐ **O núcleo é o controlo que fecha o diagnóstico:** ali a rotação é RÍGIDA e lê o facetado da
própria esfera. *O defeito está todo na faixa.*

### §68.2 — A causa, com o oráculo exacto

Numa esfera unitária a geodésica de um vértice à fronteira de uma calota **calcula-se à mão**
(`θ_v − θ_r`) ⇒ a lei tem oráculo de graça. Medindo `d_lei / d_exacto` na faixa:

| | p50 | p90 | p99 | max |
|---|---|---|---|---|
| **por ARESTAS (o que shipou)** | `1,069` | **`1,316`** | `1,368` | `1,371` |

⚠️⚠️ **Um caminho por arestas só toma as direcções que a malha tem.** Numa malha ESTRUTURADA — a
esfera do módulo é uma — isso põe `d` quase constante em cada *anel do grafo*: as curvas de nível
deixam de ser círculos e passam a ser os **losangos** da malha, o peso fica em **patamares**, e cada
degrau entre patamares é uma dobra da superfície. *As estrias da foto são as fronteiras desses
patamares.*

### §68.3 — A cura: a frente atravessa FACES

Marcha rápida à Kimmel–Sethian ([`ph2d_pose::pesos::atravessa`]): um vértice actualiza-se a partir
de uma **face** com os dois outros cantos já resolvidos, e o valor sai de uma quadrática que
interpola a frente **dentro** do triângulo em vez de a fazer dobrar num vértice. A aresta fica como
**tecto** (a marcha toma sempre o mínimo), que é o que mantém a monotonia de que ela depende.

⛔⛔⛔ **E a 1.ª tentativa não fez NADA — `0` de `22 652` actualizações — porque a malha é de QUADS.**
Ela procurava os triângulos na própria lista de vizinhos, e num quad **a diagonal não é aresta**:
dois vizinhos de um vértice nunca são vizinhos entre si. ⇒ a estrutura de face passou a viver onde
ela é construída: [`Vizinhanca::cantos`], o par `(antes, depois)` que cada face incidente dá a cada
vértice. *A adjacência é construída DAS faces e deitava fora exactamente a parte que diz de que face
cada par veio.* Num triângulo o canto é a aresta **oposta**; num quad é a **diagonal** — e nos dois
o segmento está dentro da face, logo atravessá-lo é um caminho a sério sobre a superfície.

### §68.4 — E a segunda metade: a fronteira fica DENTRO da aresta

Com a frente curada, a rugosidade do campo (`w(v) − média dos vizinhos`, em degraus de uma aresta)
ficou toda nas **4 primeiras arestas** — que é a SEMENTE, cravada a meia aresta. ⇒ duas passagens de
média sobre a pertença (`PASSAGENS_DA_INTERFACE`, com peso próprio, senão um campo binário oscila em
xadrez) dão um indicador contínuo cujo nível `0,5` é uma curva lisa, e é o cruzamento **dele** que
semeia.

⚠️ **O LADO continua a vir do campo binário**, nunca do indicador: quem pertence à região é o que o
crescimento do §3.3 disse, e alisá-lo apagaria uma região de um vértice só numa peça grossa.

### §68.5 — O placar

| | Dijkstra (o que shipou) | marcha por FACE | + interface sub-aresta |
|---|---|---|---|
| `d_lei/d_exacto` p50 | `1,069` | `0,997` | **`1,002`** |
| `d_lei/d_exacto` p90 | `1,316` | `1,007` | **`1,012`** |
| sombreamento da faixa p50 | `1,460°` | `1,184°` | **`1,162°`** |
| sombreamento da faixa p90 | `13,457°` | `4,077°` | **`3,551°`** |
| sombreamento da faixa p99 | `42,01°` | `16,73°` | **`11,81°`** |
| sombreamento da faixa max | `53,25°` | `58,01°` | **`35,21°`** |
| rugosidade `0`–`4` arestas p90 | — | `0,266` | **`0,141`** |
| rugosidade `20`+ arestas p90 | — | `0,035` | `0,035` |

⭐ **A última linha é o controlo da penúltima:** longe da fronteira as duas leis leem o mesmo, que é
o chão da discretização — *é isso que prova que a barra da interface mede a SEMENTE e não a malha.*

### §68.6 — O custo, e a nota que ficou FALSA

A marcha **PÁRA na meia-banda** (o resto está cortado em `0`/`1` por construção) ⇒ ela deixou de
varrer a malha inteira. ⭐ **E o corte é BYTE-NEUTRO, medido:** a impressão digital do campo lê
`ee632d84f7ae65c9` com ele e sem ele.

| a `97 922` vértices, `load 19` | pen-down |
|---|---|
| sem banda (o controlo) | `12,5 ms` |
| `transição 1,0` (fábrica) | `15,4` ⇒ **a banda custa `2,9`** |
| `transição 2,0` (tecto) | `21,2` ⇒ **`8,7`** |

⛔ **A nota do `TRANSICAO_MAX` dizia «plano na largura pedida» e deixou de ser verdade** — *a lei
antiga era plana porque varria a malha inteira em qualquer largura*. Hoje é mais barata onde o
artista vive (`2,9` contra os `3,4`–`4,7` registados) e mais cara no extremo, e continua `5,4×`
abaixo da difusão que as duas substituíram (`47,6 ms`).

### §68.7 — ⚠️ O gate vizinho ficou vermelho, e foi o CONTROLO dele

O `a_transicao_de_fabrica_e_onde_a_dobra_morre` exige que a `2/3` da largura de fábrica o gesto ainda
dobre (`>100` faces). Com a cura, a `2/3` viram **`33`**. ⇒ *a cura enfraqueceu a régua da outra
cura*, a mesma forma que a costura do Box Trim pagou no §65. Varrido o joelho pelo produto:

| transição | arrasto `0,60` | `0,90` | `1,20` |
|---|---|---|---|
| `0,300` | `1 506` | `1 557` | `1 579` |
| **`0,500`** | **`866`** | **`968`** | **`970`** |
| `0,667` | `33` | `43` | `43` |
| `0,800` | `0` | `3` | `5` |
| **`1,000`** | **`0`** | **`0`** | **`0`** |

⇒ **a largura de fábrica NÃO muda** (continua a ser *a primeira coluna que lê `0` em toda a linha*) e
o **controlo** passa para **metade** dela, onde a dobra vive com folga. ⛔ *Não é uma barra
afrouxada: é o controlo a mudar-se para onde o fenómeno ainda está.*

### §68.8 — ⚠️⚠️ Três mutações sobreviveram, e as três pelo ARRANJO

As cercas da travessia — causalidade `t > u`, `a·cosθ < h`, `h·cosθ < a` — **tapam-se umas às
outras** conforme a forma do canto, e na malha do produto **nenhuma delas morde** (os cantos de quad
são quase rectos, `cos θ ≈ 0`, onde as duas últimas valem `0 < h` e `0 < a`).

| recusa sozinha | canto | o que a mascarava |
|---|---|---|
| causalidade | **obtuso** (`154°`) | num canto agudo ou recto a cerca `a·cosθ < h` recusa primeiro |
| `a·cosθ < h` | **agudo** (`30°`) | num canto recto ela É a causalidade |
| `h·cosθ < a` | **agudo e torto** (`5°`, braços `0,3` e `0,2`) | com braços parecidos nunca morde |

⚠️ **E a minha 1.ª tentativa de as medir também não as media:** ela punha a frente tão obliqua que
quem recusava era a causalidade, uma cerca antes. *Três cercas, três casos* — e os três saíram de uma
busca numérica que MAXIMIZA a margem de cada um, não de um palpite.

⛔ **E o arnês da mutação mentiu antes disso:** `grep -cF` conta LINHAS e trata cada linha de uma
agulha multi-linha como um padrão à parte — ele leu `8` onde havia `1`, e o caso foi reportado como
defeito do arnês em vez de correr. A contagem passou a ser de **subcadeia**.

**Prova de mutação: `11` de `11` sangram.**

### §68.9 — O que fica ABERTO

- ⏳ O sombreamento da faixa ainda lê **p99 `11,8°`** e **max `35,2°`** à largura de fábrica. A
  rugosidade do campo a mais de 4 arestas está no chão da discretização (`p90 0,035`), logo *o que
  sobra é curvatura a sério* — a mesma rotação concentrada em metade da largura lê `p90 1,5°` a
  `transição 2,0`. **Não está medido de onde vem o `max`**, e a hipótese com endereço é o encontro de
  frentes (o corte da distância) dentro da faixa.
- ⏳ Numa peça **muito grossa** a borda continua a dobrar (o aberto do §67): *uma transição não pode
  ser mais fina do que a malha.*
- ⚠️ A cerca `h·cosθ < a` e a causalidade **não são exercitadas pela malha do produto** — elas têm
  gate de UNIDADE e o corpus está no neutro delas. *Uma malha com cantos agudos e tortos (a saída de
  uma retopologia apertada) é onde elas passam a decidir, e não há fixtura dessa.*

## §58 — 📦 PARA O AGENTE INTEGRADOR

### §58.1 — Os factos da linha

⚠️ **Esta tabela foi reescrita no fecho**: ela nasceu quando a linha tinha só o G-20 (um ficheiro,
todo em `#[cfg(test)]`) e desde então a linha ganhou duas waves de PRODUTO e dois vereditos do dono.

| grandeza | valor |
|---|---|
| base | `main` = `3090cac3f` (rebase por **fast-forward**) |
| ficheiros tocados contra o `main` | **46** (o handoff incluído; `45` tocados + `1` novo) |
| ficheiros de PRODUTO tocados | **sim** — §59 (revertida pela §61), §61, §62, **§65** (`ph2d-mesh-bool`), **§66** (`ph2d-app-sculpt3d`) e **§68** (`ph2d-pose`) |
| `PROJECT_SCHEMA` · `FIELD_DOC_VERSION` · `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` | **não se mexem** |
| os três registos de componentes (`ph2d-ecs` + os dois espelhos) | **não se mexem** |
| contratos congelados (§6) | **zero** |
| ADR | **zero** |
| pacote externo novo | **zero** |
| chaves de i18n | **líquido zero** (a `panel.sculpt3d.pose_arrasto` nasceu e morreu dentro da linha) |
| itens públicos NOVOS | `PoseControlos::SUAVIZACOES_MAX` · `ph2d_pose::Arrasto` (+`ALL`/`label`/`alavanca`) e o re-export `PoseArrasto` · `SculptStroke::plano_do_dab_para_teste` · **`ph2d_pose::Vizinhanca::cantos`** (§68) · `scenes::mesh::peca_de_fabrica` (§66). ⛔ A §65 não acrescenta nenhum, e a `pesos::atravessa` da §68 é `pub(crate)`. ⚠️ **A `Vizinhanca` ganhou DOIS campos privados** (`cantos_inicio`, `cantos`) — ela é `Clone + Debug` e nenhum consumidor a constrói campo a campo, logo é aditivo |

### §58.2 — O que NÃO pode colidir

- **Nenhuma lista partilhada ganha entrada**: o `populate.rs`, o `event.rs`, os `ids/` e a tabela de
  i18n voltam **byte a byte ao `main`** depois da §61.2 — o `git diff main --stat` não os nomeia.
- O único símbolo novo que atravessa crate é `PoseControlos::SUAVIZACOES_MAX`, **lido por um sítio**
  (a row do painel). ⛔ Ele vive na crate da LEI de propósito: um literal na row seria a segunda
  resposta a *«até onde vai este knob?»*.
- `ph2d-pose` continua **sem dependências**.

### §58.3 — ⚠️ O que só a árvore COMBINADA pode reprovar

- **`RADIUS_TRACK_MAX_PX`** vive na `ph2d-panel-sculpt3d` e o gate do §57 **lê-o**. Se outra linha
  mexer nesse número, o `o_que_aperta_o_raio_e_a_vista_nunca_o_widget` reprova — **e é para isso que
  ele existe**. A cura é ler o §57.3 antes de mexer, nunca afrouxar a asserção.
- **A catraca da DOBRA** (`ACIMA_DA_DOBRA`, §2 do gate de costura) nomeia **cinco** pincéis de outras
  waves com a altura medida em 17/09. Uma linha que acrescente controlos a um deles **sobe o número**
  e ela reprova — ⛔ a cura é **cortar/mover a fileira**, nunca subir a entrada.
- **Tecto de LOC:** nenhum ficheiro tocado se aproxima do tecto (`pose_controlos.rs` `152 → 312`,
  `rows_pose.rs` `101 → 117`, `rulers.rs` `149 → 279`).
- Os censos de texto (HR-15) foram corridos na árvore combinada: **90 / 90**.

### §58.4 — Portão de fecho (MEDIDO)

| etapa | resultado |
|---|---|
| `nextest-impacted` | **15 659 / 15 659** verdes (11 306 saltados) |
| `clippy --all-targets -D warnings` (as 7 crates da família) | **zero** avisos |
| `cargo fmt --all --check` | limpo |
| censos da **árvore COMBINADA** (HR-15 + tectos de LOC) | **90 / 90** verdes |
| prova de mutação — G-20 (§57) | **5 de 5** sangram |
| prova de mutação — a dobra do painel (§59) | **3 de 3** sangram |
| prova de mutação — as estrias (§68) | **11 de 11** sangram |
| prova de mutação — §61.2 + §62 | **7 de 7** sangram, com controlo negativo verde |
| prova de mutação — a troca de diagonais (§65) | **6 de 8** sangram; as **2** que sobrevivem estão NOMEADAS com a medição (§65.7) |
| vassouras da parede | 5 de 9 acusam, **13 ficheiros, TODOS pré-existentes** (§58.4-bis) |

⚠️⚠️ **E o arnês da mutação mentiu DUAS vezes nesta linha, as duas por uma letra ou um cano:**
o extractor procurava `^ *<nome> ... FAILED` e o cargo escreve `test <nome> ... FAILED` (§57.6); e o
controlo *«quantos testes correram?»* casava `running N tests` — **o cargo escreve `running 1 test`,
sem o `s`**, e leu `0` em toda mutação que isolasse um único teste, acusando **quatro** defeitos de
arnês sobre gates que sangravam. *Um arnês sem controlo sobre o próprio filtro reporta o produto.*

### §58.4-bis — ⛔ As VASSOURAS acusam, e as acusações são PRÉ-EXISTENTES

**5 das 9** vassouras vivas acusam, sobre o conjunto de caminhos
`ph2d-app-sculpt3d · ph2d-panel-sculpt3d · ph2d-sculpt3d · ph2d-mesh · ph2d-boundary · ph2d-pose · ph2d-cloth`:

| vassoura | ficheiros acusados |
|---|---|
| `blender-cloth` | `ph2d-sculpt3d/src/verb_scrape_tests.rs` |
| `blender-pincel-afiado` | `ph2d-sculpt3d/src/brush_verb_filter.rs` |
| `blender-pull` | `ph2d-panel-sculpt3d/src/ids/sculpt3d.rs` · `…/rows.rs` · `ph2d-sculpt3d/src/brush_default.rs` · `…/brush_magnitudes.rs` · `…/brush.rs` · `…/stroke_shape.rs` · `…/verb_strip_law_tests.rs` · `…/verb_strip_tests.rs` |
| `blender-trim-pincel` | `ph2d-sculpt3d/src/verb_scrape_tests.rs` |
| `blender-trim` | `ph2d-app-sculpt3d/src/host_contract_tests.rs` · `…/patch_valence.rs` · `…/undo_plano_tests.rs` · **`ph2d-mesh-bool/src/lib.rs`** |

⚠️⚠️ **O 13.º acusado é NOVO ao ALCANCE, não à árvore:** a §65 tocou a
`ph2d-mesh-bool`, logo ela entrou no conjunto de caminhos do sweep pela primeira
vez — e o `lib.rs` dela tem **zero adições desta linha** (`git diff main --stat`
sobre ele devolve vazio). É, muito provavelmente, a **isenção nomeada** do
`NoError` que o §44.8 já registou. ⭐ *É a mesma lei do §5.0 uma volta acima: o
sweep é propriedade do par (código, vassoura) — e também do CONJUNTO DE CAMINHOS.
Alargar o alcance faz aparecer dívida antiga como se fosse nova.*

⭐ **Nenhum dos 13 acusados está no diff desta linha.** ⇒ **zero adições**; a
reconciliação e a triagem são do **R**.

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
- **Nossas, nomeadas** — ⚠️ **AUDITADA contra o código em 17/09, e ela ENCOLHEU DUAS vezes:**
  - ✅ os gates **G-5, G-6 e G-13** do pincel de plano **FECHARAM** (§63). O G-6 já existia quando
    esta lista o nomeava; o G-5 e o G-13 nasceram aqui, e o G-5 apanhou o G-6 a medir **outra lei**.
  - ✅ a **composição por dab** do *Scene Project* estava **REFUTADA** no próprio ficheiro que a
    nomeia (a causa era o `GripLaw::from_live`, §34) — *recitar a nota mandava reconstruir trabalho
    já pago*.
  - ⏳ `dureza05` do *Scene Project*: `3` vértices de `301` na **borda móvel** da pegada, com os
    outros `298` a `≤ 5,0e-5`. Diagnosticado; é a família da banda de empate do último bit, e a
    decisão é se vira **divergência declarada** ou se a barra passa a ser por-vértice.
  - ✅ as **cunhas finas** da costura do Box Trim **FECHARAM (§65)**, e a nota que
    as descrevia era **FALSA**: os `~632` eram o leque do PÓLO da esfera de
    entrada, não da costura. O defeito a sério — `1`–`6` vértices da casca com o
    sombreamento torcido até `56°` — está curado nas seis posições do corte.
    ⏳ Sobra **uma** face de área zero, numa posição.
  - ⏳ o **`Visibility` que não propaga a descendentes** — house-wide, e o §5 já o classifica como
    decisão de produto com **ADR**: nada nesta família o pode fechar sozinho.
  - ⏳ Do pincel de plano ficam: a extensão do centro que segue a **pressão** (esta casa não a tem, e
    há gate a afirmá-lo) · a **ordem do deslocamento** contra a memória (o corpus só a exercita com a
    memória a zero) · a **pegada projectada** (decisão do dono, espec §15) · a **máscara** no arnês
    de bancada · o gémeo Motion do §49 · os passos de PAINEL da sonda do undo, que são roteiro morto
    com `active=vector`.
