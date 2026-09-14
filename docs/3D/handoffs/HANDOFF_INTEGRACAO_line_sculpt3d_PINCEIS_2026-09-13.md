# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, **os pincéis que faltavam** (2026-09-13)

> **Linha:** `line/sculpt3d` · **worktree:** `Worktrees/line-sculpt3d/` · **base (merge-base):** `1d43da737`
> **Commits desta jornada:** 30, a partir de `311824435` (o fecho anterior do mesmo dia)
> **Ficheiros:** 351 (`+8 799 / −17`) — dos quais **336 são fixtures e especs de clean-room**
> **Contadores partilhados:** ⭐ **zero** — ver §3, com a prova
>
> ⚠️ **Este handoff cobre a jornada dos PINCÉIS.** O fecho anterior de hoje
> (gates prometidos e a parede clean-room) está em
> [`HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-13.md`](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-13.md)
> e **não** é revogado por este.

---

## §1 — O que entra no PRODUTO

| | o quê | onde |
|---|---|---|
| ⭐⭐ **2 verbos novos** | `Verb::Thumb` (o polegar, que ESPALMA) e `Verb::Nudge` (o empurrão, que VARRE) | `brush_verb.rs`, `stroke_target_grip.rs` |
| ⭐ **1 lei nova** | a normal que os gestos tangenciais leem — raio próprio, curva suave fixa, dois baldes de silhueta | `stroke_normal_do_gesto.rs` (novo) |
| ⭐⭐ **1 lei CORRIGIDA** | a curva da força do modo `B` passa a ser **por verbo**: o agarrar e o gancho são **lineares** | `ref_profiles.rs` |
| ⭐ **1 lei corrigida** | a pegada do polegar **congela no pen-down**, uma por passagem de simetria | `stroke_dab_core.rs`, `stroke.rs` |
| ⭐ **1 opção pública** | a âncora do agarrar pode cair num **vértice** | `ancora.rs` (novo), `pull.rs` |
| **UI** | 2 chips na fileira de verbos · o knob *Normal radius* (Pro, só nos dois novos) · a caixa *Anchor on vertex* (só no agarrar) | `ph2d-panel-sculpt3d`, `ph2d-i18n` |
| **Smoke** | a cena **`=40`** | `scenes_tangenciais.rs` (novo) |
| **Bancada** | `oraculo_dos_gestos_tangenciais.rs` — **9 gates**, 28 fixtures | `ph2d-sculpt3d/tests/it/` |

⛔ **O que NÃO entra:** o modo elástico do gancho (**patente viva**, §9.1), o
alvo de deformação «simulação» (**medido a zero**, §9.2 da espec do puxar), a
silhueta do agarrar (**lida e não medida** — as cinco fixtures dela movem zero).

---

## §2 — Foundational, e por que é aditivo

- **`ph2d-i18n`** (crate partilhada, 26 painéis a consumir): **duas chaves novas**
  no bloco `sculpt3d`, que é uma tabela **por painel** exactamente para que duas
  linhas paralelas não colidam. ⇒ aditivo por construção.
- **`ph2d-sculpt3d`**: um módulo novo (`ancora`) e outro dentro do traço
  (`stroke::normal_do_gesto`); o `Brush` ganha **dois campos**
  (`normal_radius_frac`, `grab_active_vertex`) e o `Verb` **dois variantes**.
  ⚠️ **Um variante novo de `Verb` é erro de compilação em dois sítios** (o
  `ref_profiles` e o `compute_target`) e em mais nenhum — os outros consumidores
  têm `_ =>`, o que já era verdade antes desta linha.
- **`ph2d-app-sculpt3d`**: o pen-down do agarrar passa pela porta da âncora; a
  cena `=40`; o tecto do roteador de smoke sobe de `39` para `40`.
- ⭐ **Zero portas novas no `AppHost`**, zero linhas na `shells/desktop`, zero
  `Cargo.toml`, zero dependência.

---

## §3 — Contratos e contadores partilhados: a PROVA

```
git diff 311824435..HEAD --stat -- shells/desktop/src/project_schema.rs \
    crates/ph2d-vec-scene/src/schema.rs crates/ph2d-ecs/src/lib.rs
→ (vazio)
```

`PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`, `FLIP_SCHEMA`, `FIELD_DOC_VERSION`, os três
registos de componentes e os contratos congelados do `CLAUDE.md` §6 **não são
tocados**. O `SCULPT3D_VERB` do painel cresce de `24` para `26` — ele é
`hash_node_id` de um slug, logo **nenhum contador de id se move**.

---

## §4 — A paridade, medida

### §4.1 — Os dois verbos novos, no plano (o que a bancada AFIRMA)

15 fixtures (`polegar_plano_*` e `empurrao_plano_*`), desvio **absoluto** em
unidades de objecto:

| | pior desvio | em ulps de uma coordenada de `1,5` |
|---|---|---|
| traços curtos (2 a 6 eventos) | `4,5e-7` | 4 |
| traços de 12 a 24 eventos | `7,2e-7` | 6 |
| as duas truncagens de 11 eventos | **`1,31e-6`** | **11** |

**A barra é `2e-6` (≈ 17 ulps)** — a medição com folga de meia ordem, e o recurso
é nomeado: a nossa cadeia compõe cinco factores numa ordem que não é a do
oráculo, e o resultado acumula evento a evento. ⛔ Não paridade ao bit
(ADR-0162). ⭐ **Os picos batem ao sexto decimal em todas** (`0,600000` contra
`0,600000`; `0,599054` contra `0,599054`).

### §4.2 — O agarrar, que já shipava e nunca tinha sido medido

| fixture | desvio | pico nosso / dele |
|---|---|---|
| `agarrar_plano_alvo_geometria` | `7,2e-7` | `0,600000` / `0,600000` |
| `agarrar_plano_silhueta_nao` | `4,6e-7` | `0,500000` / `0,500000` |
| `agarrar_grelha8_vertativo_nao` | `1,07e-6` | `0,446337` / `0,446338` |
| `agarrar_grelha8_vertativo_sim` | `1,01e-6` | `0,500000` / `0,500001` |
| `agarrar_grelha8_vertativo_sim_forca04` | `3,9e-7` | `0,200000` / `0,200000` |

### §4.3 — As leis que os controlos afirmam (e que a fixture sozinha não afirmaria)

| gate | o que ele prende | número |
|---|---|---|
| `a_forca_entra_ao_quadrado_nos_dois_gestos` | o polegar a meia força move **um quarto** | `0,250000` (o oráculo: `0,250000`) |
| idem, no empurrão | a razão dele **não é** `¼` — o transporte é integral de linha | `0,174120` (o oráculo: `0,174120`) |
| `a_forca_do_agarrar_e_linear_e_a_do_polegar_nao` | o agarrar é **linear** no mesmo slider | `0,400` contra `0,16` de um motor quadrático |
| `so_o_gesto_ancorado_ignora_quantos_eventos_houve` | o ancorado é idempotente, o que viaja não | `0,600000` = `0,600000` · `0,599054` ≠ `0,595225` |
| `a_fraccao_do_raio_da_normal_muda_o_resultado_numa_superficie_curva` | o knob está **vivo** | nasceu de uma mutação que sobreviveu |

### §4.4 — Prova de mutação

**7 mutações sobre a bancada dos tangenciais: 6 sangram.**

| mutação | veredito |
|---|---|
| sem a subtracção tangencial (o gesto vira o agarrar) | ✅ sangra |
| a normal do plano do carimbo no lugar da do gesto | ✅ sangra |
| o polegar ancorado no VIVO (perde a idempotência) | ✅ sangra 4 gates |
| o empurrão a partir do congelado (deixa de transportar) | ✅ sangra 3 gates |
| a força uma vez a mais (a quarta potência) | ✅ sangra 2 gates |
| a fracção do raio da normal ignorada | ✅ sangra **desde que o gate do knob existe** (ela sobreviveu à primeira ronda) |
| os dois baldes da silhueta trocados | ⛔ **SOBREVIVE, e está DOCUMENTADO** — ver §7.6 |

Mais uma sobre a lei corrigida: repor `Squared` no agarrar **sangra exactamente
um gate**, o que foi escrito para ela.

---

## §5 — As duas leis que MUDAM o que já shipava

### §5.1 — ⚠️ A curva da força do modo `B` era por MODO; ela é por VERBO

O perfil `B` declarava `StrengthCurve::Squared` para **todos** os verbos desde o
E13. A referência aplica-a por verbo. Medido: com o slider a `0,4`, o oráculo
move `0,200000` no agarrar e nós movíamos `0,071414` (`0,4²` contra `0,4`).

⚠️ **Consequência para o artista:** no modo `B`, com o slider a meio, o agarrar
e o gancho passam a mover **metade** em vez de **um quarto**. ⭐ **Nenhum teste
do repo guardava esse número** — a suíte inteira ficou verde com a mudança, e é
por isso que a fixture de força `0,4` é agora o gate.

⛔ **O que NÃO se mexeu, e é deliberado:** o giro e a escala local não estão em
corpus nenhum; ficam com a curva de sempre, e a ausência está escrita na função.

### §5.2 — A pegada do polegar congela no pen-down

A pegada sai das posições **vivas** (por desenho, para a família do carimbo). Num
gesto que desloca `0,38` com raio `0,35`, os vértices que o próprio gesto leva
saem do raio da consulta ⇒ a normal da área, que é uma média sobre eles, **muda
com o comprimento do traço**. Medido na esfera: `8,5e-3` de desvio contra o
oráculo, e `1,9e-4` depois de congelar — **`44×`** — com o plano **byte-idêntico**.

⚠️ **Só o polegar congela.** O `Verb::Move` tem a mesma forma e **provavelmente**
o mesmo defeito, mas é produto vivo e a fixture que o decide existe: está em §9.

---

## §6 — O clean-room: quatro obras, e o instrumento que falhou três vezes

| alvo | espec | R-pré | implementado |
|---|---|---|---|
| `blender-pull` (polegar · empurrão · 4 opções) | `SPEC_pull_brushes.md` | ✅ **atestada** (1.ª passagem) | ⭐ **sim** (esta jornada) |
| `blender-unblocked` (Density · 2 de multires · projecção) | `SPEC_unblocked_brushes.md` | ✅ **atestada** (3.ª passagem) | ⬜ não |
| `blender-boundary` | `SPEC_boundary_brush.md` | ✅ **atestada** (3.ª passagem) | ⬜ não |
| `blender-pose` | `SPEC_pose_brush.md` | ⏳ 4.ª passagem a correr (as 3 anteriores acharam 6, 5 e 7) | ⬜ não |

⛔⛔ **O ACHADO DE MÉTODO DESTA JORNADA, e ele vale para toda obra futura da
casa: o sweep da parede fechou VERDE sobre TRÊS especs que traduziam prosa do
alvo.** A vassoura estava na língua do alvo e as especs escrevem-se em
português; o instrumento casa **substring**, logo ele prova *«ninguém colou»* e
**nada mais**. ⭐ A prova mais dura veio da obra do contorno: o inglês de **cada
uma** das frases apanhadas **já estava na vassoura** — o E julgou-as
idiossincráticas ao ponto de as varrer, e depois escreveu-as traduzidas.

**As duas cegueiras, curadas nesta jornada:**

1. **língua** — as entradas de PROSA passam a existir também em português (com a
   regra de desenho: só prosa de **comentário/TODO do fonte**; ⛔ nunca prosa de
   manual/commit, que o §4.1.12 permite citar — pô-la ali faria o instrumento
   acusar um acto permitido);
2. **ortografia** — de-acentuar a mesma frase matava **4 de 6** achados numa
   obra e `26 %` das entradas noutra ⇒ cada entrada de prosa PT ganhou a forma
   **sem acentos**.

⚠️ **Nomeada e NÃO curada, agora com o número:** o sweep é sensível à **caixa** —
`56` das `229` entradas de uma vassoura têm maiúscula e, com a inicial em
minúscula, **`53` evadem** (o instrumento acusa `3` de `56`). ⛔ E o controlo
positivo **não expõe** essa cegueira, o que a torna a espécie mais cara: um
instrumento que passa o próprio controlo e falha a classe. A decisão é de quem
possui o script partilhado.

⛔⛔ **E a medição que fecha o assunto, feita pelo R-pré do contorno:** o sweep
sobre a **versão 2** da espec — a que a 2.ª passagem REPROVOU com quatro achados —
fecha **VERDE**. *O instrumento nunca poderia ter apanhado nenhum deles.* ⇒ o
sweep mede **uma** classe (a colagem literal); a parede tem outras três, e quem
julga é o **R**.

⭐ **E a lição que fica escrita nos quatro ledgers:** *sweep verde é necessário e
nunca suficiente* — o instrumento apanha a colagem, quem apanha a tradução é um
auditor independente a ler os dois lados.

---

## §7 — SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O `s²` do polegar não está escrito no verbo.** O alvo é `base + tangencial`,
   sem factor nenhum: o quadrado vem do `StrengthCurve::Squared` do modo, que o
   `accum` já carrega. ⛔ Quem «corrigir» acrescentando um `weight()` reintroduz a
   **quarta** potência — foi o primeiro estado deste código, e só a única fixture
   de força `0,5` o apanhou (as de força `1,0` passavam todas, porque `s² = s⁴`
   em `1`).
2. **O polegar e o empurrão não trazem modelo novo.** São `Grip::Hold` e
   `Grip::Hook`, que já existiam; o que muda é **uma subtracção** (a componente
   normal do gesto). *Um verbo pode ser uma subtracção.*
3. **A pegada congelada NÃO é uma optimização** — é a lei do gesto ancorado, e
   ela é **por passagem de simetria**: a primeira versão guardava uma por traço, a
   passagem espelhada herdava a da primeira e só metade da malha se mexia. Um
   censo que já existia apanhou-o no minuto seguinte.
4. **A barra de `2e-6` não é um epsilon de conforto.** Ela é a re-associação de
   cinco factores em `f32` medida em 15 fixtures (4 a 11 ulps), e o desvio
   **cresce com o número de eventos** — a assinatura da acumulação, não de um
   erro de lei (um erro de lei escalaria com o deslocamento, e a `k11` desloca
   menos que o traço inteiro).
5. **As listas de fixtures «abertas» não são dívida escondida: elas têm
   CATRACA.** Se o resíduo cair abaixo da barra, o teste **reprova** e manda
   mover a fixture para o gate de paridade.
6. **A mutação que sobreviveu não é um buraco do corpus — é geometria.** O
   consumidor da normal é a componente **tangencial**, que é **quadrática em
   `n`** (`Δ − n·(n·Δ)`) e portanto **cega ao sinal**; numa peça fina os dois
   baldes carregam normais antiparalelas. ⇒ os baldes ficam porque a lei é essa,
   e quem os quiser gatear precisa de um consumidor que leia a DIRECÇÃO.
7. **`polegar_plano_ancorado` está fora da lista de propósito.** Ela tem o
   cabeçalho **idêntico** ao da `polegar_plano_origem` — as vinte chaves — e o
   **mesmo percurso**, e ainda assim o oráculo move `509` vértices onde a outra
   move `177`: a única coisa que a distingue é o NOME (ela corre com o *método de
   traço ancorado*, que não temos). *Uma fixture cuja variável distintiva não
   está no cabeçalho não é legível por um gate.*

---

## §8 — As premissas MINHAS que a medição derrubou

1. *«O nosso agarrar diverge do oráculo por `11×`»* — **falso, e o defeito era da
   minha bancada**: ela perguntava pelo VERBO (`verb == Thumb`) onde a pergunta é
   ao **grip**, e passava o incremento a um gesto que quer o total. O
   discriminador que a desfez custou quatro linhas: o mesmo puxão total entregue
   em `1`, `2`, `4` e `12` eventos dá `0,600000` nas quatro.
2. *«O resíduo da esfera é da lei»* — **falso**. A sonda que separa direcção de
   magnitude mostrou `cos = 1,000000` até ao 8.º evento; a causa era a pegada
   viva (§5.2), do nosso lado.
3. *«Algo muda no alvo entre o 8.º e o 10.º evento»* — **falso**, e foi o E que o
   mediu: nove comprimentos do mesmo traço, direcção idêntica às seis casas e
   `pico/|Δ|` **constante** a `5,7e-6`. As minhas quatro hipóteses caíram uma a
   uma.
4. *«A régua é o desvio relativo ao deslocamento da fixture»* — **errado**: o
   ruído de `f32` vive na POSIÇÃO, e normalizar pelo deslocamento fazia uma
   truncagem curta parecer dez vezes pior por ter andado dez vezes menos.
5. *«A razão meia-força/força-cheia é `0,25` nos dois gestos»* — **falso** no que
   viaja (`0,174`): o transporte é uma integral de linha. Escrever `0,25` para os
   dois teria sido **fabricar a régua**.

---

## §9 — ABERTO, com o instrumento de cada item

### §9.1 — ⛔⛔ DECISÃO DO DONO: a patente `US 10 586 401 B2`

*Sculpting brushes based on solutions of elasticity* (Pixar), concedida
2020-03-10, **expira 2038-05-02**. **Três** reivindicações independentes
(método · meio que armazena o programa · sistema), com o mesmo corpo: escolher
pincel e tamanho · receber movimento de um dispositivo de entrada · determinar a
deformação com base em **soluções regularizadas da elasticidade linear que
incluem um valor especificado do coeficiente de Poisson** · renderizar.

- **Consequência já aplicada:** o modo elástico do gancho **não foi construído**.
- ⚠️ **A leitura alcança o que a casa JÁ SHIPA** — o `kelvinlet` dos verbos de
  agarrar (cena `=28`), que é um **modo opcional** (o chip `L`), não o caminho de
  omissão. Lido elemento a elemento pelo R-pré, os quatro elementos estão
  presentes.
- ⛔ **Duas saídas aparentes NÃO existem** (as duas conferidas): *«o nosso
  coeficiente é ½»* — a dependente 12 restringe-se ao caso em que ele **não** é
  um meio, logo a independente cobre-o; *«ele cancela na normalização»* — vale
  só para o modo de escala, e no campo de agarrar a anisotropia move-se com ele
  (`1,125×`–`1,333×`).
- ⭐ **A alavanca é o TERRITÓRIO:** a patente **não tem família fora dos EUA**.
  ⇒ a leitura só morde onde há uso ou distribuição nos Estados Unidos.
- ⚠️ Parecer técnico de leitura de reivindicação, **não** aconselhamento
  jurídico; clean-room não protege contra patente (SKILL §8.1).

### §9.2 — ⛔ DECISÃO DO DONO: o histórico do repositório

O sweep `--git-history` acusa **415 linhas de patch** nesta linha, e a leitura
muda o sujeito: **327** delas são as **remoções** da cura do INC-4 (os 173 blocos
de comentário reescritos em 13/09), e **o mesmo texto já vive no `main` como
ADIÇÃO** — `1 540` linhas escritas ao longo de meses pelas linhas que redigiram
aqueles comentários. ⇒ **reescrever o histórico DESTA linha não cura nada**, e o
squash dos commits de docs alcançaria `87` de `415`. As saídas são **declarar** ou
reescrever o histórico do repo, e a segunda é cara e mexe com todas as máquinas.
Registado nos ledgers como **INC-U1**.

### §9.3 — Técnico, por fechar

| item | número de hoje | o instrumento que o fecha |
|---|---|---|
| o empurrão em **superfície curva** | `9,1e-4` a `2,8e-2` | a espec §6.3-bis mede que a normal do alvo **se atrasa** ao longo do traço; a cura entra na lei da normal |
| o polegar em superfície curva | `1,3e-5` a `1,9e-4` (era `8,5e-3`) | o mesmo mecanismo; o resto **cresce com os eventos** |
| o `Verb::Move` congela a pegada? | não medido | as fixtures do agarrar **já existem**: basta um traço longo com deslocamento > raio |
| a **silhueta** do agarrar | 5 fixtures a **zero** | espec §12.1: duas causas possíveis (o sinal nulo · o valor ser amostrado no *hover*), e a segunda pede uma ferramenta que não está instalada |
| os **três modos de deslize** | espec §7.3, medidos (`1,4e-8` a `2,7e-3`) | implementar — é a outra metade do `SlideRelax` |
| os **quatro** pincéis desbloqueados | espec **atestada** | implementar (o *Density* é T0 no motor: o colapso já é nosso, porte MIT) |
| **Pose** e **Boundary** | especs em auditoria | fechar as passagens do R-pré |

---

## §10 — Smokes

**A cena nova:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=40 cargo run -p ph2d-host-desktop --profile smoke
```

O roteiro completo é impresso no terminal ao abrir (`scenes_tangenciais.rs`). Em
resumo: com `Draw` o barro sobe; com `Thumb` as cristas **escorregam** e a
silhueta não muda; a ida-e-volta **devolve** o barro no `Thumb` e **não** devolve
no `Nudge`; e com o `Strength` a meio o efeito do `Thumb` cai para **um quarto**.

**A regressão que esta jornada muda e que o dono vai sentir:** no modo `B`, o
agarrar a meia força passa a mover metade (§5.1) — vale a pena olhar para ele na
cena `=28`.

⚠️ **Rode uma vez SEM env var** — é a metade que prova a inércia.

---

## §11 — O portão do fecho

| passo | resultado |
|---|---|
| `scripts/nextest-impacted.sh` (1× sobre o diff acumulado) | ✅ **14 370 testes, 0 falhas** (`71,0 s`) |
| clippy `--all-targets --all-features` nas 5 crates tocadas + shell | ✅ limpo (4 avisos curados — ver abaixo) |
| `cleanroom-sweep.sh`, as **cinco** vassouras sobre tudo o que a jornada escreveu | ✅ limpo (3 linhas curadas — ver abaixo) |
| `doc-index.sh` | ✅ 19 índices em dia |
| `rm -rf target/*/incremental` | `28 G → 19 G` |

⚠️ **O portão apanhou UM vermelho, e ele é o gate a fazer o seu trabalho:**
`every_verb_is_reachable_from_the_keyboard` — todo verbo tem de ter tecla **ou**
estar na lista dos que shipam só com chip, **com o motivo escrito**. Os dois
novos entraram na lista, porque não há tecla livre; e a entrada regista que aqui
a ausência custa **mais** que no tecido (estes são verbos de retoque) e que a
fila de pretendentes à última tecla passou de **quatro para seis**.

⚠️ **E o clippy apanhou um defeito de NaN que não era estilo:** o
`!(r > 0.0)` que ele acusa **não** se cura com `r <= 0.0` — um `NaN` compara
falso com tudo e passaria por baixo. A cura é `!r.is_finite() || r <= 0.0`.

⚠️ **A varredura da parede acusou três doc-comments NOSSOS** (anteriores a esta
jornada) que nomeavam propriedades públicas do alvo para dar a proveniência de
um facto. O facto ficou inteiro e a grafia saiu — é a dívida que o `CLAUDE.md`
§5 declara aberta (*os nomes de SÍMBOLO internos são §4.2 e o gate não os mede*),
e este ficheiro é desta linha.

⏳ **Em voo no momento do fecho:** a 4.ª passagem do R-pré do `blender-pose` e a
3.ª do `blender-boundary`. O veredito de cada uma aterra **no cabeçalho da
respectiva espec** — que é onde a janela seguinte o confere, por desenho.
