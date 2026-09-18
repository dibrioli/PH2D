# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-09-18

**Para o INTEGRADOR.** Esta linha fecha com **37 commits** sobre o `main` (merge-base `3090cac3f`),
**171 ficheiros**, `+10 939 / −8 652`. ⛔ Ela **não** integra e **não** faz ship (§0.7) — entrega
isto e para.

> **Leia primeiro a §2.** Ela é a resposta medida à pergunta que a integração redescobre mil vezes:
> *«o que desta linha colide com as outras?»* — e a resposta aqui é **quase nada**.

---

## §1 — O que a linha entrega, em DOIS assuntos

### (a) Ciclo 9 — RIG & CORPOS MOLES (doc 103, o protocolo em vigor)

O envelope por osso (o P0 da folha 16), a força de uma restrição como coluna que já existia, a
medição do preço dos corpos moles (uma ESCADA, e ela bateu num tecto não medido), o tecto do campo
de `60` para **`512`** (o número é o do irmão, medido), a cena **`=120`** e o **tutorial em PDF**.

⭐⭐ **E o PAINEL LATERAL DE PARAMS SAIU do app** — com três tectos órfãos atrás dele.

### (b) Doc 115 — O COLISOR SAI DO GRAFO (ordem do dono)

> *«tirar o collide e deixar tudo pela Shape e pelos outros objectos (como vector, Sprite, Flip) que
> serão criados com seus próprios colliders (sem usar o grafo)»*

W0..W6 + os abertos + **dois reports do dono já fechados e com smoke APROVADO**. O eixo:

| wave | o que ficou |
|---|---|
| W1 | a cerca que faltava (e eu tinha a que existe ao contrário) |
| W2 | **fechada por RECUSA MEDIDA** — os dois caminhos de CPU são de classes diferentes |
| W3 | o colisor de um Sprite **deriva-se** ⇒ a aresta entre famílias desapareceu |
| W4 | a **membrana** declara a forma do objecto |
| W5 | **o app SEPARA SOZINHO** — [`ph2d_contact::passe`](../../../crates/ph2d-contact/src/passe.rs) no fim do cozimento, com UM interruptor no sink |
| W6 | o nó sai da **LISTA** e o **motor FICA** (`register_out_of_catalogue`, side-metadata append-only) |
| §15 | o passe honra o `falloff` + a cena **`=122`** |
| §16 | a TOMADA passa a ver o que o sink DESENHA (report do dono) |
| §17 | o retrato do gizmo sai do cozido **DESTE** quadro (report do dono) |
| §18 | **o tecto das varreduras passa a ser MEDIDO** — a cadeia deixa de ser inalcançável, e TRÊS acelerações candidatas ficam como **recusas medidas** (ordem do dono) |
| §18.7 | a cena **`=123` — A CADEIA**, o único sítio onde a escada de varreduras se VÊ (as irmãs são pares independentes) |
| §19 | ⭐⭐⭐ **o tecto era honesto e o MOTOR não era** (report do dono) — o passe fica **4,4×** mais barato a 500 peças e **8,7×** no quadro dele; toca em `ph2d-nodegraph` (`par_build_if`, append-only) |
| §20 | ⭐⭐⭐ **o laço pára quando nada mais se VÊ** (2.º report) — `500` objectos a `1024` varreduras: `157,9 → 6,8 ms` (**46×** no caminho dele). ⚠️ **Não é bit-idêntico de propósito**, e o preço são DUAS barras de gate de produto re-precificadas, em duas crates |
| §21 | ⭐⭐⭐ **o acabamento era pago por TIQUE e o desenho é UM** (3.º report, com foto) — a shell cozinha um quadro por tique em dívida e **só o último é desenhado**; a cena da foto vai de `245 → 31 ms` (**3 → 32 FPS**). Toca na PONTE (`motion_bridge`) e na bomba (`set_separa_o_desenho` + o readout `separacoes()`) |
| §22 | **os MIL já estão no lado do Motion** (4.º report) — `1000` objectos a `64` varreduras custam `6,8 ms` e `2000` custam `12,2`. ⛔ Uma hipótese minha CAIU: o paralelo não é o alocador, é o `fork/join` por varredura (`5×` o CPU da série para o mesmo trabalho). Porta nova no seam auditado (`par_build_com_bloco`) |

---

## §2 — SUPERFÍCIE DE COLISÃO, medida (`collision-surface.sh`, pós-rebase)

⭐⭐⭐ **Esta linha não move NENHUM contador partilhado.**

| grandeza | linha | `main` de HOJE (lido no ficheiro, não na coluna) |
|---|---|---|
| `PROJECT_SCHEMA` | **144** | **144** |
| tripla do gate | `(144, 13, 22)` | `(144, 13, 22)` |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` · `FIELD_DOC_VERSION` | `22 · 13 · 18 · 23` | iguais |
| registo `ph2d-ecs` / espelho `-render` / espelho `-script` | `91 / 92 / 92` | iguais |
| contrato congelado (§6) | **intocado** nos dois ficheiros | — |
| `Cargo.lock` | **nenhum pacote externo novo** | — |
| ADR | **esta linha não cria nenhum** ⇒ fora de toda disputa de número | — |
| tecto de LOC | nenhum ficheiro da linha passa | — |

⇒ **não há degrau para recontar, não há espelho para corrigir, não há número de ADR a disputar.**
O rebase sobre o `main` de hoje (4 commits à frente do merge-base) correu **sem um conflito**.

---

## §3 — ⛔ O que só a árvore COMBINADA pode reprovar

1. **Tecto de LOC por ACUMULAÇÃO.** Nenhum ficheiro desta linha passa sozinho, mas o tecto é a
   única grandeza que **soma entre linhas sem ninguém a contar**. Se a catraca acender, a cura é
   **corte por responsabilidade** — e esta linha já o fez duas vezes (`ph2d-eval-motion/src/lib.rs`
   `715 → 691`, com a porta a descer para o `sink_style.rs`, cujo cabeçalho já fazia a pergunta).
2. **O censo de texto (HR-15).** Corrido nesta árvore depois do rebase: **87 censos, 87 verdes**,
   com controlo do próprio filtro (`8 de 8 correram`). ⚠️ O CI **não** os corre.
3. **A ordem do quadro.** Esta linha acrescenta uma fase nova
   ([`fase_motion_gizmos`](../../../shells/desktop/src/render_loop/fase_motion_gizmos.rs)) e um gate
   que afirma `cook < resolve < desenho`. Outra linha que mexa na ordem das fases da shell reprova
   ali — **e isso é o gate a funcionar**, não uma colisão a resolver por merge.

---

## §4 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O `motion.collide` NÃO foi apagado.** Ele saiu da **LISTA** do artista (paleta e catálogo); o
   **kernel de dispositivo FICOU**, por decisão do dono e com o número ao lado: `4,19 M` objectos
   correm **`88×`** mais rápido ali do que no passe de CPU. *Um diff que vê `register_out_of_catalogue`
   e lê «removeram o nó» inverte a decisão.*
2. **Uma CADEIA converge — o que ela pede é `~n²` varreduras** (§18, e isto CORRIGE a leitura da
   §15.3, que dizia *«3 a 64, que é o topo do knob»*: o `64` era um número **herdado** do clamp de
   outro nó, e ele segurava uma cadeia de **QUATRO**). Hoje o tecto é MEDIDO — `1024` no slider
   (fecha `n = 16`, `18,5 %` de um quadro) e `4096` digitável (fecha `n = 32`). ⇒ as cenas
   `=121`/`=122` continuam feitas de pares independentes porque foi assim que nasceram, **não**
   porque a cadeia seja inalcançável.
3. **`separa_o_que_se_desenha` devolve `None` em quase toda cena**, e é isso que mantém tudo o que
   já existia **byte-idêntico** — sem clone e sem um `if` a lembrar: ele devolve `None` quando
   ninguém declara colisor, quando o interruptor está desarmado, ou quando nada se moveu.
4. **O `falloff` do passe não é um knob novo.** É a **coluna** que o `motion.falloff` já escrevia.
   ⚠️ E a maneira de desligar uma fileira é pôr o campo **LONGE**, não usar `invert`: o nó dá `1`
   dentro do raio e **`0` exacto** fora dele, enquanto `invert` dá uma **RAMPA** (medido
   `0,68 · 0,51 · 0,16 …`) — que separaria a fileira **pela metade**, o pior dos dois mundos.
5. **No §17 moveu-se o RESOLVE, e NUNCA o desenho.** O desenho já estava no sítio certo, e movê-lo
   reabriria uma lei que este repo pagou em 13/09: *no Vello quem pinta depois fica por cima*, e os
   gizmos estão no fim de propósito para não ficarem **ATRÁS** da arte que manipulam.
6. **São DOIS gizmos que mudaram de sítio, e o terceiro ficou por MEDIÇÃO.** Colisor e warp leem
   `tap_streams`; o gizmo de **field** lê params do nó (nenhum ficheiro `field_gizmo*` menciona
   tomadas) ⇒ não tem o defeito, e mover o que não tem o defeito só alarga o diff.
7. **A porta `o_que_o_sink_desenha` vive no `sink_style.rs` e isso é responsabilidade, não arrumação.**
   Aquele módulo já respondia *«em que ESTILO este sink desenha?»* e já guardava o
   `sink_collide_sweeps`; as duas leem-se dos **MESMOS** params do sink.

---

## §5 — ⛔ As premissas MINHAS que a medição derrubou

| eu escrevi | a medição |
|---|---|
| «a W6 migra as **4** cenas de banco» | o censo **derivado** leu **3**, e **nenhuma** migra (todas se alimentam de `motion.grid`) |
| «o `falloff` é param de CARTÃO ⇒ o passe não tem de onde o tirar» | **erro de categoria**: `Strength` é param de cartão, `falloff` é **coluna de stream** — e ela já chega ao sink |
| «o `invert` do `motion.falloff` desliga a fileira» | ele dá uma **RAMPA**; a cura é o campo **longe**, onde a resposta é `0` exacto |
| «o overlay é construído antes do cook ⇒ o desenho também é cedo» | **falso**: `desenho = 662 982 > cook = 482 136`. `fase_hero_frame.rs` tem **TRÊS** fases, e a l. 369 vive na `fase_hero_tools`, chamada **uma linha antes** da `fase_hero_scene` ⇒ *número de linha no mesmo ficheiro não é ordem de execução* |
| «o gate do banco prova que o nó ainda funciona **cozinhando** as duas cenas» | `129 600` peças duas vezes, em CPU e debug: **`1 553 s`**, e teria sido **morta** pelo tecto de `180 s`. Quem responde pelo kernel é o **REGISTO** ⇒ `1 553 s → 0,00 s` |

⛔⛔ **E o ARNÊS DE MUTAÇÃO mentiu com TRÊS formas, duas delas novas nesta casa:**

1. `NAO-COMPILA` nas mutações que **sangravam** — o cargo imprime `error: test failed` quando um
   teste falha, e o classificador procurava `error:` **antes** do veredito. ⇒ *o veredito primeiro.*
2. `running 1 test` é **SINGULAR** com um teste só, e a régua exigia o plural.
3. ⭐ **NOVA:** correr um filtro num pacote corre **vários alvos** (a lib, e cada `tests/*`), e cada
   um imprime o seu `running N` e o seu `test result:`. Ler o **PRIMEIRO** dá o alvo da lib, que casa
   **zero** — e o primeiro `test result: ok` é dele. ⇒ o veredito lia-se **`SOBREVIVEU (0 correram)`**
   sobre gates que de facto sangram, *com as duas metades a mentir no mesmo sentido*. A cura é a
   **SOMA** de todos os alvos, e o `FAILED` perguntado **antes** do `ok`.

⭐ A guarda de «a mutação entrou» é o **FICHEIRO TER MUDADO** (`cmp`), nunca um `grep` — um
formatador que parte a linha faz o padrão casar zero, e um `grep` largo casa **outra** linha.

---

## §6 — ⏳ O que fica ABERTO (e de quem é)

> ⚠️ **A §19 mudou DOIS destes itens e acrescentou um** (report do dono de 18/09, doc 115 §19):
> - o *«acima de `n ≈ 32` o custo é do artista»* foi **re-medido e está desactualizado na direcção
>   boa**: `1024` varreduras custam hoje `3,79 ms` a 48 peças e `36,2 ms` a 500 (eram `6,88` e
>   `157,9`). ⛔ **O que NÃO mudou é a classe:** a `1000` peças ainda são `68 ms`, e daqui para
>   baixo é algoritmo (`82 %` de uma varredura já é a LEI), não escrituração.
> - *«a leitura de quanto o passe custa não está no cartão»* **pagou-se pela segunda vez** — foi o
>   relógio que informou o dono, não o app. Continua por construir.
> - ⏳ **NOVO:** o `Vec<u32>` dos vizinhos ainda é alocado por peça e por varredura (dentro dos
>   `18 %` de escrituração que sobram); um scratch por thread fecha-o. Não foi feito porque a
>   medição não o justificou sozinho.
>
> ⚠️⚠️ **E a §20 acrescentou o item que é o TECTO REAL do alvo do dono** (*«centenas a milhares de
> objectos em runtime»*): ⛔⛔ **a rota da PLACA não corre o passe — `ph2d-gpu-cook` não tem uma
> única referência a `ph2d-contact`.** Hoje isso é invisível porque toda cena com forma cai no
> caminho da CPU (medido: a fronteira do planeador é o `motion.duplicator`, com **zero** etapas de
> GPU, com e sem colisão) — *mas o caminho rápido e a colisão são hoje **mutuamente exclusivos***.
> Fechar isto é um kernel, com espec própria.
>
> ⭐⭐⭐ **E a §23 MEDIU o desenho, e derrubou a razão que a §22 dava:** preparar mil formas custa
> **`0,04 ms`** (o batch memoiza a tesselação por `geometry_id` ⇒ mil cópias são UMA tesselação e
> mil poses). ⇒ o que sobra dos `18 ms` é a **RASTERIZAÇÃO**, e ali a grandeza **não é a contagem de
> formas, é a ÁREA que elas cobrem**: os discos do report têm `~200` unidades de diâmetro e mil
> deles pintam `~31 M` de pixels por quadro. ⚠️ É isso que explica *«retirar o contorno azul não
> melhorou em nada»* — o anel é um traço FINO, o disco é uma ÁREA.
> ⛔⛔ **E a partição de LOD é CEGA a isso: a cerca dela é uma CONTAGEM**
> (`LOD_COUNT = 16 000` cópias), logo mil discos gigantes passam por baixo dela a pintar muito mais
> do que dezasseis mil formas pequenas. *Um tecto que não nomeia o recurso que o governa é um
> palpite à espera de um smoke.* **É o lever maior que sobra, é do render/Vector, e agora tem
> endereço e grandeza.**
>
> ⏳ **NOVO (§22.2):** o `fork/join` **por varredura** segura o paralelo em `1,5×` onde devia render
> dezenas — medido pelo CPU contra a parede (`5×` o CPU da série, `4`–`5` núcleos). A cura é uma
> região paralela que atravesse as varreduras, e é wave própria desta crate.

| item | de quem |
|---|---|
| ⛔ ~~A realimentação no solver~~ — **FECHADO em 18/09 por ordem do dono** (*«quero todas as possibilidades possíveis, não quero limitações no sistema»*) e o resultado inverteu a pergunta: a recusa era sobre um **número HERDADO** (`64`, o clamp de outro nó), a cadeia **converge sempre**, e as TRÊS acelerações candidatas foram construídas e **medidas e refutadas** — sobre-relaxação `~1,95×`, vermelho-preto `~3,3×`, realimentação **ZERO**. Doc 115 §18 | ✅ fechado |
| ⏳ **Acima de `n ≈ 32` a conta é do artista** (`4096` varreduras = `1,6` quadros) — removê-la por inteiro exige método **NÃO-local** (multigrid · resolução directa do grafo de contacto · propagação de choque), que é espec própria. E **o custo não é VISÍVEL** no cartão | próxima wave |
| Os tectos de `motion.boids` e `motion.wave` seguem por medir (herdado, não desta linha) | próxima wave |
| ⭐ **Pedido de promoção à lista de flakes do §5.0, agora COM a assinatura completa:** `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` (`ph2d-tool-painter`) — único ✗ de `17 086` a `load 18,86`, **zero linhas** do diff desta linha naquela crate, e **3 de 3 verde sozinho a `load 21,8`–`25,9`**, que é carga MAIS ALTA do que aquela em que reprovou ⇒ *o discriminador é o FAN-OUT, não o relógio* | integrador escreve |

---

## §7 — A PROVA DE FECHO

Sobre a árvore **já rebasada** no `main` de hoje:

| portão | resultado |
|---|---|
| formatação | limpa |
| lint com `-D warnings` (crates tocadas + shell) | **zero** |
| varredura impactada | **17 090** testes, **17 090 passaram**, 0 falharam |
| censos da árvore combinada (HR-15 + tecto de LOC) | **87 de 87**, com controlo do filtro |
| placar da conferência (DERIVADO) | `exit 0` · **P0 = P1 = P2 = 0** |
| rebase sobre o `main` | **sem um conflito** |

**Provas de mutação da jornada:** W6 `4 de 4` · §15 `5 de 5` · §16 `2 de 2` · §17 `3 de 3` · §18 `4 de 4` ·
**§19 `7 de 8`** — todas com **controlo negativo** (a árvore intacta sobrevive) e **controlo sobre o próprio
filtro**. ⚠️ A 8.ª está documentada **no código** como não-sangrante de propósito: ela corrigiu um comentário meu
(a ordem crescente dentro de uma célula da grelha **não** é load-bearing — quem cumpre a promessa é o `sort`).

⚠️⚠️ **A §19 chegou DEPOIS do fecho, por report do dono, e mexe numa crate FOUNDATIONAL** —
[`ph2d-nodegraph/src/attr.rs`] ganha `par_build_if`, **append-only** (o `par_build` de sempre passa a delegar
nele, com a mesma garantia de bits e o mesmo gate). ⇒ *a superfície de colisão do §2 muda por uma linha
naquele ficheiro*, e o tecto de LOC do `ph2d-eval-motion/src/lib.rs` foi curado por **CORTE** (o laço das
tomadas desceu para `taps.rs`, que já é o dono do assunto): `710 → 665`.

**Portão da §22:** varredura impactada **17 328** testes, **17 328 a passar**. ⚠️ **Para quem funde:**
ela acrescenta `par_build_com_bloco` ao `ph2d-nodegraph` (append-only, o `par_build_if` passa a
delegar nele) e **não muda produto** fora disso — o resto da wave é MEDIÇÃO, e o valor dela é o mapa
do §22.3 com o tamanho de cada lever que sobra.

**Portão da §21:** formatação limpa · lint `-D warnings` a **zero** · varredura impactada **17 328**
testes com **17 328 a passar** · mutação **4 de 4**. ⚠️ **Para quem funde:** a §21 acrescenta uma
bandeira de estado à bomba (`separa_o_desenho`, que **nasce `true`** — todo chamador que cozinha um
tique só continua igual) e uma linha no laço de `ticks_owed` do `motion_bridge`. ⛔ **E a fiação tem
gate de TEXTO** (`o_quadro_marca_so_o_ultimo_tique_como_desenhado`), porque o `dispatch` pede um
`HeroScreen`, um `ToolRegistry` e um `GpuContext` e não é alcançável de um teste — *um motor com a
lei certa e a shell a não a ligar lê-se como um motor sem a lei*.

**Portão da §20:** formatação limpa · lint `-D warnings` a **zero** nas quatro crates · varredura
impactada **17 326** testes com **17 326 a passar** (zero vermelhos) · mutação **4 de 5**, com a 5.ª
a ser o **controlo negativo** (tirar um atalho não pode mudar resposta nenhuma).

⚠️⚠️ **Para quem funde: a §20 muda a BARRA de dois gates de PRODUTO, e uma delas noutra crate**
(`ph2d-node-sim-step`). Elas eram `1e-6`/`1e-5` — escritas quando o laço varria sempre o tecto
inteiro — e hoje o produto pára no repouso visível, deixando uma cauda da ordem do limiar
(`1,4e-5` e `1,1e-4`, esta última `0,011 %` da largura da caixa). ⭐ Os dois ganharam o **controlo
do viés** no caminho que nunca pára cedo (`separate_all_pairs`), que é o que impede a barra nova de
esconder um defeito real.

**Portão da §19, sobre a árvore já rebasada:** formatação limpa · lint `-D warnings` a **zero** nas três
crates · varredura impactada **17 324** testes com **17 323** a passar — o único vermelho é
`the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`, o membro da família de flakes de fan-out que o
§6 abaixo pede para promover (**3 de 3 verde sozinho a `load 13,56`**).

**Smokes que o dono APROVOU:** `PH2D_GPU_COOK_DEMO=121` (a separação) e o gizmo colado à forma
depois do §17. ⏳ **Por smokar:** a `=123` (a cadeia e a escada de varreduras) — ela nasceu com o
§18 e o dono ainda não a viu.
