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
2. **O passe é um ACABAMENTO, nunca uma lei de contacto.** Medido: ele resolve arranjos com
   sobreposições **LOCAIS e INDEPENDENTES**; numa **CADEIA** ele não converge — 15 pares vão a 10 em
   8 varreduras, 6 em 32 e ainda **3 em 64**, que é o tecto do knob. ⇒ a cena `=121`/`=122` é feita
   de **pares independentes por NECESSIDADE**, não por conveniência de demo.
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

| item | de quem |
|---|---|
| **A realimentação no solver** — para o passe segurar uma simulação em **CADEIA** (hoje ele é acabamento de pares independentes, §4.2). É desenho novo, não afinação: `64` varreduras não bastam e o tecto do knob já lá está | **decisão do dono** |
| Os tectos de `motion.boids` e `motion.wave` seguem por medir (herdado, não desta linha) | próxima wave |
| **Pedido de promoção à lista de flakes do §5.0:** `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (`ph2d-tool-painter`) — gate de RAZÃO, nomeado durante esta jornada. ⚠️ **Nesta janela ele passou em todas as corridas** (17 085 verdes, três vezes), logo a confirmação com `loadavg` ao lado **fica por fazer**: *uma flake sem a carga medida ao lado não entra na lista* | integrador, **se** voltar a acusar |

---

## §7 — A PROVA DE FECHO

Sobre a árvore **já rebasada** no `main` de hoje:

| portão | resultado |
|---|---|
| formatação | limpa |
| lint com `-D warnings` (crates tocadas + shell) | **zero** |
| varredura impactada | **17 085** testes, **17 085 passaram**, 0 falharam |
| censos da árvore combinada (HR-15 + tecto de LOC) | **87 de 87**, com controlo do filtro |
| placar da conferência (DERIVADO) | `exit 0` · **P0 = P1 = P2 = 0** |
| rebase sobre o `main` | **sem um conflito** |

**Provas de mutação da jornada:** W6 `4 de 4` · §15 `5 de 5` · §16 `2 de 2` · §17 `3 de 3` — todas
com **controlo negativo** (a árvore intacta sobrevive) e **controlo sobre o próprio filtro**.

**Smokes que o dono APROVOU:** `PH2D_GPU_COOK_DEMO=121` (a separação) e o gizmo colado à forma
depois do §17.
