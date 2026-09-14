# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, **os pincéis que faltavam** (2026-09-13)

> **Linha:** `line/sculpt3d` · **worktree:** `Worktrees/line-sculpt3d/` · **base (merge-base):** `1d43da737`
> **Commits desta jornada:** 30, a partir de `311824435` (o fecho anterior do mesmo dia)
> **Ficheiros:** 351 (`+8 799 / −17`) — dos quais **336 são fixtures e especs de clean-room**
> **Contadores partilhados:** ⭐ **zero** — ver §3, com a prova
>
> ⭐ **E ele cobre também a jornada de 2026-09-14 — o OSSO** (o indicador do
> pincel de pose, por ordem do dono): §6-ter, e a tabela do portão no §11.
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
| `blender-pose` | `SPEC_pose_brush.md` | ✅ **atestada (5.ª passagem)** — 6 · 5 · 7 · 5 achados e depois verde; os **2** que sobravam (redacção, não bloqueantes, ZERO §4.2) estão **curados na emenda 6**, que o cabeçalho regista **a seguir** ao veredito | ⬜ não |

⭐ **A emenda 6 foi CONFERIDA CONTRA O DIFF por esta janela, não aceite pela
palavra de quem a escreveu** — que é o que a lei do escopo mínimo existe para
tornar barato: `2` ficheiros, `5` hunks, `24` linhas na espec, **zero fixtura
tocada**, e as duas curas são à letra as que o atestado prescreveu. ⭐⭐ **E a
cadeia de consistência fechou sozinha:** o censo subiu de `10` para `11` linhas
**e a contagem citada na lista de verificação subiu com ele** — *num documento
onde um número vive em dois sítios, é a segunda cópia que diz se a emenda foi
feita a sério.*

⚠️⚠️ **O padrão que as CINCO passagens do Pose mediram, e ele é do MÉTODO:** em
**três** delas seguidas a emenda curou **o endereço nomeado** e a mesma redacção
sobreviveu noutro sítio — incluindo em **títulos** que nenhuma emenda tocou. ⇒ a
instrução que fica para toda emenda futura é *cure pela FORMA da frase (um
`grep` pela promessa), nunca pelo endereço*.
⭐⭐ **E a 5.ª passagem é a PROVA de que a instrução funciona, com número:** mandada
varrer pela forma em vez dos cinco endereços, a emenda **achou uma afirmação sem
fixtura que o próprio auditor não tinha visto** (o censo passou de `6` para `10`
linhas, e o R re-mediu as quatro novas nos cabeçalhos das 69: **honestas, zero
fabricadas**). *Uma varredura por forma encontra o que a lista de achados não
continha; uma por endereço nunca pode.*
⚠️ **E a espécie da promessa reapareceu pela QUARTA vez seguida, agora numa
ABERTURA em vez de um título** — a §8 rotula de «observável» exactamente a
proposição que a cura do §7.2 acabou de declarar **não observável**, três secções
antes. ⭐ O R mediu porquê ela nunca poderia ter sido apanhada pelo corpus: as `5`
fixturas de simetria correm todas numa malha **exactamente espelhada** (`0` de
`4 930` vértices sem parceiro) ⇒ os dois lados da proposição dão a mesma saída
**por construção**. *Uma fixtura simétrica não pode refutar uma lei sobre simetria.*

⛔⛔ **E a lei que esta ronda deixa para toda emenda a uma espec JÁ ATESTADA:**
*toda linha nova é população nova a auditar.* A 5.ª passagem recusou herdar o
«§4.2 limpo» da 4.ª e varreu as **111** linhas da emenda anterior de raiz — foi
isso que tornou o atestado honesto. ⇒ uma emenda pós-atestado é de **escopo
mínimo declarado** (aqui: duas linhas mais a linha do censo), e o cabeçalho
regista **o que mudou depois do atestado** — *um atestado que cobre uma versão
que já não existe é pior que nenhum.*

⛔ **ACHADO DE PROTOCOLO para o R-PÓS, e ele não é desta espec:** o §7.2 põe a
barra do fechamento em *«zero hits sobre a árvore inteira»*, e o **§6 obriga o
ledger a ter a tabela de cobertura** — que nomeia ficheiros do alvo. O ledger
vive na árvore rastreada ⇒ **as duas exigências não são satisfazíveis ao mesmo
tempo**. Quem fechar tem de escolher e **declarar** (cobertura sem nomes, ou o
ledger fora do censo com o motivo escrito). *Uma barra que ninguém pode cumprir é
uma barra que se afrouxa em silêncio no dia do fechamento.*

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

## §6-bis — ⭐⭐⭐ O TERCEIRO PINCEL: a POSE, implementada e MEDIDA

> **Ordem do dono, 13/09:** *«vamos implementar, contudo vamos tentar superar o
> blender»*. ⚠️ **«Superar» só é verificável depois de «reproduzir»** — sem
> paridade medida, uma diferença é indistinguível de um defeito. Com ela, cada
> divergência passa a ser deliberada, com número e gate.

**A lei vive na crate-folha [`ph2d-pose`](../../../crates/ph2d-pose/)** — zero
dependências, como a `ph2d-cloth`: ela não sabe o que é uma `Mesh`, um pincel ou
uma câmera. O artista aponta a um ponto e arrasta; o pincel acha sozinho, **pela
forma e pela ligação da malha e sem esqueleto nenhum**, um pivô e uma cadeia de
segmentos, e roda a região à volta dele.

### §6-bis.1 — A paridade, à primeira corrida

| | |
|---|---|
| dentro de `1e-5` | **`57` de `69`** fixturas |
| corpo do corpus | `5,96e-8` a `6,74e-7` |
| gates | `8`, com barra **por CLASSE** (§12.3 da espec) e censo de obsolescência |
| mutação | **`14` de `14` sangram**, com controlo negativo |

⭐⭐ **E isto BATE o modelo de referência da própria espec**, que fecha em
`54/69`. Ganhámos **três** das quinze divergências dele: as **duas** da
descontinuidade do crescimento (ele `2,1e-2` e `2,2e-2`; nós `6,0e-7` e
`1,7e-7` — caímos do lado certo do `<` estrito nas duas) e a **degenerada §11.1**,
que ele declarava *«bug do nosso MODELO»* e que agora dá `0,000e0` com
`movidos = 0` dos dois lados, igual ao veredito do alvo.

⭐⭐⭐ **A cura da degenerada é o achado da jornada, e ela não é um epsilon.**
Quando o pivô cai em cima do cursor, a diferença entre a cabeça e a origem do
primeiro segmento vale **UM ULP** das próprias coordenadas: ela é feita
inteiramente do arredondamento com que as duas foram escritas, e a direcção
tirada dela é **ruído normalizado a comprimento `1`**. Varrido o corpus inteiro,
essa fixtura mede **`0,9` ulps** e a mais pequena de **todas** as outras
**`176 683`** — *cinco ordens de grandeza de vazio*. A barra sai desse vale, o
piso é **relativo** à magnitude das posições (um ulp perto de `2,2` é `2,4e-7`;
perto de `0,002` é `2,4e-10`), e os **quatro** consumidores de uma direcção
inicial passam pela mesma porta.

Os `12` que sobram são exactamente as classes que a espec declara — o polo da
escala atravessado, a família da escala (relativa, medida `≤ 1,7e-5`), o cursor
sobre o plano de espelho (veredito: quase nulo), a **auto-suavização não
modelada de propósito**, e a única que a espec deixa **por explicar**.

### §6-bis.2 — ⭐⭐ Onde SUPERAMOS o alvo, e cada linha com a fonte

| o que o alvo faz | o que custa ao artista | o nosso lado |
|---|---|---|
| reconstrói a cadeia **a cada movimento do rato**, sem traço nenhum, só para desenhar o indicador | o editor engasga em malha densa; com peças desligadas cada zoom paga `O(V²)` — **quatro** relatos públicos | construída **uma vez por traço**, e ⭐ **GATEADO** (`a_cadeia_da_pose_constroi_se_uma_vez_por_traco`) |
| a suavização **pode** não ser reprodutível acima de uma partição | o mesmo gesto poderia dar resultados diferentes | Jacobi limpo ⇒ determinístico em **toda** a malha. ⚠️ A reserva viaja: esse regime é **risco do mecanismo** e **nunca foi observado** |
| pivô sobre o cursor ⇒ **não faz nada, em silêncio** | o artista arrasta e nada acontece | detectável (`Pose::inerte`) |
| a auto-suavização só age dentro do **raio inicial** | o efeito «desaparece» longe do cursor | se a oferecermos, segue os **pesos** |
| vértice solto recebe a média de um conjunto **vazio** | peso indefinido | mantém o peso (§11.4, divergência declarada) |

⚠️⚠️ **E uma vantagem escrita num cabeçalho é uma PROMESSA; uma com contador é
uma PROPRIEDADE.** Sem o gate do contador, quem movesse a construção para dentro
do laço de eventos não partiria teste nenhum — a saída seria a mesma, só mais
lenta, e a regressão viajaria até ao dia em que o dono esculpisse uma malha
grande.

⛔ **O que NÃO fizemos, e é honesto dizê-lo:** a dependência da **taxa de
eventos** (§5.1-bis) e a leitura de **pixels** na torção (§5.2, *o mesmo gesto dá
torções diferentes conforme o zoom*) são defeitos reais de previsibilidade **e**
são o comportamento que o artista conhece e que as fixturas medem. Trocá-los é um
**MODO com gate próprio**, nunca uma correcção silenciosa.

### §6-bis.3 — Os três sobreviventes da mutação, e o que eles ensinam

A primeira volta matou `11` de `14`. ⭐ **Os três sobreviventes tinham a MESMA
causa, e não era fraqueza dos gates: são propriedades que os `69` traços
publicados NÃO discriminam.**

| sobreviveu | porque o corpus é cego | o gate que o mata |
|---|---|---|
| semente por ordem de chegada | com um eixo de espelho há `2` sementes e o desempate quase nunca decide | `a_semente_sai_ordenada` |
| **o espremer RESOLVE a cadeia** | ⭐ **todas** as fixturas de espremer são ancoradas, e com âncora e um segmento resolver é um **no-op** ⇒ o **item 13** da lista de verificação da espec **não tem fixtura que o imponha** | `o_espremer_nao_resolve_a_cadeia` |
| sem a guarda de `1e-5` | nenhum traço leva o quociente abaixo do limiar | `a_guarda_do_espremer_impede_o_infinito` |

⚠️⚠️ **E o terceiro sobreviveu DUAS vezes antes de morrer, por dois defeitos do
ARNÊS e não da lei:** procurei o regime no **extremo errado** (a guarda defende
`|δ| → ∞`, não o polo — eles são os dois lados opostos do mesmo quociente), e o
cursor estava no **CENTRO** da grelha, onde a franja é um anel simétrico, o pivô
cai sobre o cursor e o traço inteiro é **INERTE**. *Um gate sem controlo positivo
do próprio sujeito mede o nada e fica verde* — o arnês afirma agora que a cadeia
nasceu viva.

### §6-bis.4 — A fiação, e a porta que ela obrigou a existir

⭐⭐ **Havia DUAS respostas à mesma pergunta.** O desvio no `stroke_symmetry`
nomeava verbos à mão enquanto os censos do `stroke_apply` perguntavam ao `Grip` —
e as duas concordavam **por acaso**, enquanto o único que desviava era a
simulação. A pose quebrou o acaso: ela desvia e o grip dela é o `Hold`, que dois
verbos do laço também usam. ⇒ porta única **`Verb::resolve_a_propria_regiao()`**,
lida pelo desvio **e** pelos censos, com o despacho a falhar **alto** em debug se
alguém a declarar sem lhe dar destino.

⛔ **E tirar um verbo de um censo sem escrever o gate que o substitui é como uma
ausência vira permanente** — esta linha caçou **oito** gates *citados e nunca
escritos* em 13/09. Os dois substitutos existem e sangram.

### §6-bis.5 — ⭐⭐⭐ A cena `=41` abre na ORELHA, e a escolha é MEDIDA

Numa esfera lisa a franja que dá o pivô é um **anel simétrico** à volta do cursor
⇒ a média dela cai em cima dele, o primeiro segmento nasce com comprimento nulo e
**o pincel não move nada**. Uma cena de esfera mostraria a ferramenta a parecer
partida — e a `=36` já pagou esse preço, com o dono a responder *«do modo como o
objecto é não é possível testar»*. A orelha é um **apêndice**: a franja dela é
quase toda do lado do corpo e o pivô cai na **BASE**, que é o que faz o gesto
parecer uma articulação. O gate `a_pose_move_a_orelha_desta_cena` afirma que ela
de facto se mexe.

### §6-bis.6 — ⛔⛔ O ACHADO DE INSTRUMENTO: o sweep é do PAR, não do código

Ao ligar o verbo, as **cinco** vassouras sobre as cinco crates da família deram
**`18`** achados — `pose 0 · cloth 0 · pull 15 · boundary 2 · unblocked 1`.
⚠️ **Nenhum era novo:** toda essa prosa já tinha fechado sweeps **verdes**, e
nada dela estava no diff da jornada. O que mudou foram **as vassouras** — elas
são **por alvo**, os agentes E estendem-nas a cada emenda, e as três obras novas
desta jornada trouxeram vassouras de `298`, `229` e `227` entradas contra as
`149` e `137` que já existiam.

⇒ *«o sweep fechou verde» lê-se como propriedade do CÓDIGO e não é: é propriedade
do **PAR (código, vassoura)**, e a vassoura é um **alvo móvel** que outra pessoa
edita.* **Corolário:** emendar uma vassoura obriga a corrida nova sobre a **árvore
inteira**, e quem fecha uma linha corre **todas** as vassouras vivas — não só a
da obra dela.

Curados dois blocos; ⭐ **os `13` que sobram são UM único token** em 16 sítios,
que é frase comum do domínio, rótulo de interface e chave de i18n — e o
**atestado da espec que acompanha aquela vassoura já escreve a lei que o isenta**.
*Uma vassoura pode contradizer o R-pré que viaja com ela.* A cura é do dono dela;
até lá é **isenção NOMEADA** no ledger, nunca silêncio.

---

## §6-ter — ⭐⭐⭐ O OSSO: o indicador do pincel de pose (2026-09-14)

> **Ordem do dono:** *«no blender temos um gizmo do pincel que mostra como se
> fosse um bone de modo ao usuário perceber a área de atualização do pincel»*.

### §6-ter.1 — Porquê ele, e porquê só neste verbo

O anel do cursor descreve **mal** este pincel, e não por descuido: os outros 26
verbos têm atenuação radial, então o círculo **é** a pegada. A pose não tem —
espec §13: *«nenhuma parte da malha é excluída pelo raio»* —, e um vértice a dez
raios do cursor pode mover-se por inteiro porque a região cresce pela **ligação**
da malha. ⇒ *o anel mostra um círculo onde a ferramenta pensa num membro*, e a
pergunta que o verbo levanta — **onde é que ele vai achar a dobradiça?** — só
tinha resposta depois de arrastar e desfazer.

O que se desenha é a **cadeia**: uma silhueta octaédrica de armadura por
segmento, larga junto da dobradiça e afilada para a mão, com anéis nas juntas e
um anel cheio na dobradiça mais funda.

⭐ **A figura diz DE QUE LADO está a articulação sem uma seta**, e é isso que
uma linha entre dois pontos não pode dizer (ela é simétrica). Gate:
`a_silhueta_e_mais_larga_junto_da_dobradica`.

### §6-ter.2 — ⭐⭐ O indicador NÃO é uma segunda versão da lei

A cabeça de cada osso é a **imagem da cabeça inicial pelo mapa do próprio
segmento** (§6), que é exactamente a função por que passa todo vértice daquele
segmento — [`ph2d_pose::Cadeia::ossos`].

⛔ **O atalho plausível — `origem + rot·(cabeça₀ − origem₀)` — concorda com a lei
em quatro das cinco deformações**, e é por isso que ele entraria sem ninguém ver:
no espremer/esticar a rotação é a identidade e a escala vive numa base **local ao
segmento**, e o atalho desenharia um osso do tamanho original enquanto a peça
estica. Gate com o controlo negativo lá dentro:
`o_osso_e_a_lei_aplicada_a_cabeca_e_nao_o_atalho`.

⚠️⚠️ **E o octante é o da ÂNCORA, não o `0`.** As reflexões do §6 cancelam-se em
pares (`bit != (âncora[eixo] < 0)`), logo o mapa não reflectido é o do octante em
que a âncora vive — com simetria ligada e o cursor em coordenada negativa, o
octante `0` é o mapa do **outro lado** e o osso apareceria espelhado longe da
mão. Gate: `o_osso_fica_do_lado_da_ancora_com_simetria`.

⭐ **Uma porta só serve o sobrevoo e o traço** ([`SculptStroke::pose_ossos`]):
durante o gesto ela devolve os ossos **vivos** (de graça — a cadeia já existe e
já está resolvida), fora dele a cadeia que o pen-down construiria. *Duas funções
— uma «em repouso» e outra «a mexer» — seriam duas respostas à mesma pergunta, e
a que o artista vê é a que envelhece.* Gates:
`o_indicador_da_o_mesmo_osso_que_o_traco_constroi` (a anti-mentira) e
`o_osso_vivo_segue_o_arrasto` (que existe porque o primeiro **não o pode dar**:
com arrasto nulo a cadeia viva e uma reconstruída dão a mesma figura, logo a
mutação que trocasse uma pela outra sobreviveria).

### §6-ter.3 — ⭐⭐⭐ O custo, que é onde o alvo perde

O alvo reconstrói a cadeia inteira **a cada movimento do rato**, sem traço
nenhum, só para desenhar este indicador — quatro relatos públicos. Aqui são três
coisas distintas, cada uma com número:

| o que custa | como é pago | gate |
|---|---|---|
| a **adjacência** (`O(faces)`; com peças desligadas, `O(V²)`) | uma vez por gesto, guardada | `o_indicador_nao_reconstroi_quando_nada_muda` (contador `adjacencias`) |
| a **cadeia** | só quando a **chave** muda, e a chave traz **exactamente** o que a §2–§3 lê | `a_suavizacao_do_peso_nao_move_o_osso_e_a_chave_sabe_disso` |
| o **ritmo** | o alvo paga por EVENTO de ponteiro (~16/quadro a 1 kHz); aqui uma vez por **quadro**, e acima do orçamento nem isso | o orçamento em quadros |

**MEDIDO no perfil do smoke (`release`), na malha da própria cena `=41`
(`24 386` vértices):**

| segmentos | 1.ª construção | quadro repetido | quadros de silêncio |
|---|---|---|---|
| **1** (omissão) | `2,88 ms` | `0,37 µs` | `2` |
| `3` | `5,96 ms` | `0,23 µs` | `4` |
| `20` (tecto do painel) | `22,69 ms` | `0,74 µs` | `14` |

⇒ por omissão o osso segue o cursor a **~30 Hz**; no tecto do painel uma
construção sozinha passa o quadro, e o orçamento converte *«o editor arrasta»* em
*«o indicador demora»* — a troca certa para uma figura que descreve um gesto que
**ainda não aconteceu**.

⚠️ **O orçamento conta QUADROS, não relógio de parede** (`ORCAMENTO_POR_QUADRO =
1,67 ms`, um décimo de um quadro de 60 fps; o silêncio é `ceil(custo /
orçamento)` chamadas). O relógio entra **só** a medir o que a construção custou —
o número que se auto-calibra ao perfil. *Com um temporizador de parede o gate
«dentro de N quadros ele reconstrói» seria mais um membro da família de flakes
sob fan-out.*

### §6-ter.4 — ⛔ Um defeito meu que só a sonda apanhou: `456 µs` para não fazer nada

A primeira redacção corria o `mais_proximo_global` (`O(V)`) **e** alocava um
`vec![false; V]` **antes** de comparar a chave ⇒ um quadro de sobrevoo **parado**
custava `456 µs` na malha da cena (`2,7 %` de um quadro), mais uma alocação da
malha inteira por quadro.

⭐ A cura é reconstruir a chave com o **eleito anterior** (`O(1)`: se ele continua
onde estava e o resto bate, o eleito de hoje é o mesmo) e comparar a **struct
inteira** — `456 µs → 7,4 µs` no perfil de teste, `0,37 µs` no do smoke.
*Uma cache que faz o trabalho caro antes de perguntar se precisa dele não é uma
cache.* ⚠️ E a comparação é de struct inteira de propósito: uma lista de `&&`
escrita à mão é onde um campo novo da chave é esquecido.

### §6-ter.5 — ⭐ O §11.1 deixa de ser uma promessa

A tabela do `ph2d-pose` prometia *«o caso é detectável e deve ser avisado»*. Ele
é avisado agora: quando o pivô cai em cima do cursor, o indicador desenha **só um
anel vermelho sobre o cursor e nenhum osso** — porque *uma figura que desenhasse
um osso de comprimento zero diria que há uma dobradiça ali, que é o contrário do
facto*. O alvo cala-se neste caso.

### §6-ter.6 — Os TRÊS vermelhos que o portão apanhou, e os dois que já lá estavam

1. **`the_brush_radius_is_screen_pixels_converted_against_the_camera`** e
   **`the_pick_compares_in_world_and_the_brush_crosses_the_scale`** — os dois
   ancoravam no **corpo** do `armed_brush`, e a conversão saiu para
   `armed_brush_on` quando o indicador passou a precisar dela para uma peça que
   **não é a activa** (ao sobrevoar, o cursor pode estar sobre outra). *Um gate
   ancorado num corpo de função expira quando o corpo muda de casa, e a lei não
   se moveu — ela ganhou um segundo leitor.* Re-apontados, mais uma asserção
   nova: o delegado **não pode** ter uma segunda cópia da conta.
2. ⭐⭐ **`a_stroke_belongs_to_the_piece_it_started_on`** — este é **substantivo**:
   ele conta os consumidores de `self.pick(x, y)` e exige que só o `aim` mova a
   peça activa. O indicador é `&mut self` (a cache vive no traço) ⇒ seria o
   **primeiro** consumidor capaz de trocar de peça a meio de uma pincelada, que é
   o **pânico** que aquele gate existe para impedir. ⇒ a escolha da peça saiu
   para `pose_gizmo_alvo(&self, …)`: *a prova volta a ser do compilador, não de
   uma linha de texto*. O censo passa a `3`, com o terceiro **nomeado**.
3. ⛔ **Dois `clippy::assertions_on_constants` PRÉ-EXISTENTES** (as cenas `=40` e
   `=41`, da jornada anterior) — e o clippy tinha razão: os dois lados são
   constantes. Viraram `const { assert!(…) }`, que é **mais forte** (erro de
   compilação). ⚠️⚠️ **E o `cargo check` é CEGO a isto:** com o limiar mutado
   para `9999`, `cargo check --all-targets` fecha **verde** e só o `cargo test`
   devolve o `E0080` — *um `const` de dentro de uma função só é avaliado quando
   ela é CONSTRUÍDA*. Membro novo da família do `--bins` que não alcança `tests/`.
4. ⛔ **O `ph2d-pose` inteiro estava por formatar** (18 ficheiros, da jornada
   anterior) — latente para o `ship.sh`, que corre `fmt --check`. Corrido, e a
   varredura re-corrida **depois** dele: *o `rustfmt` já partiu réguas textuais
   deste repo três vezes.*

### §6-ter.7 — Prova de mutação: `9` de `9` sangram

| mutação | quem sangra |
|---|---|
| a chave curto-circuitada | `o_indicador_nao_reconstroi_quando_nada_muda` |
| `segmentos` fora da chave | `a_suavizacao_do_peso_…` (o controlo positivo) |
| o `begin` não esquece | `um_traco_novo_faz_o_indicador_esquecer_a_adjacencia` |
| sem a guarda de verbo | `nenhum_outro_verbo_paga_o_indicador` |
| a sessão viva ignorada | `o_osso_vivo_segue_o_arrasto` |
| o osso pelo **atalho** | `o_osso_e_a_lei_aplicada_a_cabeca_e_nao_o_atalho` |
| octante `0` em vez do da âncora | `o_osso_fica_do_lado_da_ancora_com_simetria` |
| a silhueta simétrica | `a_silhueta_e_mais_larga_junto_da_dobradica` |
| a cintura sem piso | `a_cintura_tem_piso_em_pixels` |

### §6-ter.8 — ⛔⛔ O UNDO DA POSE NÃO EXISTIA, e a causa é estrutural

> **Report do dono (2026-09-14):** *«undo/redo não funciona para esse pincel»*.

O `close_stroke` da cena grava `StrokeUndo::Stroke { verts: touched(),
positions: base_positions() }` e **devolve cedo** quando a janela está vazia.
Quem a enche é o `capture`, que vive no laço **por-vértice** do `dab_core` — e
este verbo [`Verb::resolve_a_propria_regiao`], logo **nunca passava por lá**.

⇒ o traço movia a malha (`171` vértices na fixtura do gate) e **não deixava rasto
nenhum**. ⚠️ *Uma janela vazia e um gesto que não fez nada são o mesmo byte para
quem grava* — nenhum gate acusava, porque nenhum perguntava.

⭐ **O tecido desvia igual e chamava o `capture` à mão**; a pose não. A cura é a
mesma porta, e a escrita passa a ser em **três passos**:

1. quem vai mudar, contra a malha **viva**;
2. `capture` de cada um — ⚠️ **antes** da escrita, porque ele lê
   `mesh.positions()` para congelar o `pre`;
3. a escrita.

⚠️ **O `capture` é idempotente (carimbo por época), e é isso que faz o `pre`
estar certo mesmo para um vértice que só entra na janela ao 3.º evento:** a lei
escreve sempre a partir do `p0` congelado, logo quem ainda não se moveu está
exactamente onde nasceu.

⚠️⚠️ **E o report tem a mesma FORMA do do tecido (05/09) com um mecanismo
OPOSTO**, o que vale mais que a cura: lá os dados estavam certos na crate e o
defeito era da **shell** (o foco preso num chip do painel comia o `Ctrl+Z`); aqui
a janela saía vazia **da crate**. *Dois relatos idênticos, duas causas em
camadas diferentes — e quem tratasse o segundo pela memória do primeiro
procuraria no sítio errado.*

**Gates, e são dois porque provam metades diferentes:**

| gate | onde | o que só ele afirma |
|---|---|---|
| `a_pose_enche_a_janela_do_undo` | `ph2d-sculpt3d` (unidade, corre sempre) | a janela enche, o `pre` é o do **pen-down** e **nenhum** vértice movido fica de fora |
| `a_pose_stroke_undoes_and_redoes` | `ph2d-app-sculpt3d` (produto, `#[ignore]` + GPU) | a corrente inteira: `take_hold` → `pending_grab`/`flush` → `close_stroke` → `undo` → `redo`, e que o desfazer **avisa a tela** |

⚠️ O gate de produto toca **fora** do centro do enquadramento de propósito: numa
esfera lisa o pivô cai em cima do cursor e o traço é **inerte** (§11.1) — *um
gate sobre um gesto que não move nada fica verde a medir o nada*.

**Prova de mutação: 2 de 2 sangram, nos dois gates** — apagar o `capture`, e
(o subtil) chamá-lo **depois** da escrita, que passa o `pre` a ser a pose deste
evento.

---

### §6-ter.9 — O tecto do *Pivot offset* sobe a `3` (ordem do dono)

O alvo oferece `0..2` (espec §1.1) e o dono pediu `3` ⇒ **divergência
declarada**, com a medição no doc da própria row
([`rows_pose.rs`](../../../crates/ph2d-panel-sculpt3d/src/rows_pose.rs)).

**MEDIDO antes de escrever o número** (sonda
`sonda_o_desvio_da_origem_alem_do_tecto`, três malhas do corpus, raio `0,25`,
arrasto `0,2`):

| desvio | comprimento do 1.º segmento | deslocamento máximo | construção |
|---|---|---|---|
| `0` | `0,223`–`0,251` | `0,156`–`0,180` | `0,17`–`0,55 ms` |
| `2` (tecto do alvo) | `0,723`–`0,751` | `0,189`–`0,202` | `0,98`–`5,75 ms` |
| **`3`** | `0,973`–`1,001` | `0,192`–`0,201` | `1,35`–`5,76 ms` |
| `4` | `1,223`–`1,251` | `0,194`–`0,200` | `1,74`–`5,75 ms` |

⭐ **Três leituras:** a alavanca é **exactamente linear** (`raio × (1 + desvio)`);
o efeito **satura** depois de `2` (com o pivô longe a rotação tende para uma
**translação** — o limite geométrico, não um artefacto); e o relógio **também
satura** nas duas malhas maiores. ⇒ subir de `2` para `3` **não abre regime novo
nenhum**: o tecto é de PRODUTO, não de recurso, e o que há do outro lado está
medido e é mais do mesmo.

---

### §6-ter.10 — ⏳ ABERTO, com o instrumento de cada item

- ⏳⏳ **NA FILA, por ordem do dono (14/09): quem subdivide no Dynamic Topology**
  — [plano 22](../22_plano_quem_subdivide_no_dyntopo.md). A causa está **medida**
  (o refino tem um chamador só, o braço do carimbo ⇒ `19` verbos refinam, `8`
  nunca refinam), a pergunta é de **oráculo** (metade livre no SculptGL, que é
  MIT; metade por janela **E** no Blender), e as três armadilhas da cura estão
  nomeadas. ⛔ **O `Mask` pode ser curado antes do estudo:** um gesto que não
  escreve posição não tem porque mudar a topologia.

- **A região não é pintada, só a cadeia.** O osso mostra a EXTENSÃO do membro;
  *quais vértices* e *com que peso* é outra superfície (o canal por-vértice do
  device já existe — é o do padrão do pincel) e ela **colide** com o preview do
  alpha. Preço não medido; a pergunta é de produto.
- **Não há interruptor.** O anel do cursor também não tem, e a coerência é
  deliberada; o alvo tem um. Decisão de produto.
- **O indicador fica desactualizado** se a malha mudar sem passar por um gesto
  **e** o vértice sob o cursor ficar exactamente onde estava (um desfazer que não
  toque a região apontada). É o indicador, nunca a deformação — o pen-down
  constrói sempre de raiz.
- ⚠️ **Uma flake de carga NOVA, por promover:**
  `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`
  (`ph2d-tool-painter`) reprovou no fan-out de `14 398` e passou **3 de 3**
  sozinha a `load 27`–`40`. ⚠️ A **irmã de ficheiro** dela
  (`the_mask_stroke_cost_does_not_follow_the_canvas`) já está na lista do
  `CLAUDE.md` §5.0 — *é exactamente assim que a lista envelhece*.

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

**A cena do PINCEL DE POSE (`=41`), com o OSSO:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=41 cargo run -p ph2d-host-desktop --profile smoke
```

⭐ **O passo (2) do roteiro é o novo e é o que muda o gesto:** escolher `Pose` e
**passar o rato sem carregar** — aparece o osso, com a bolinha cheia na
dobradiça, e passear ao longo da orelha mostra a dobradiça a MUDAR de sítio. O
passo (4) põe `Segments` em `3` e são **três ossos em fila**.

⚠️ **Rode uma vez SEM env var** — é a metade que prova a inércia.

---

## §11 — O portão do fecho

⚠️ **A tabela abaixo é a do fecho de 13/09. O fecho de 14/09 (o OSSO, §6-ter)
re-correu o portão inteiro:**

| passo | resultado (2026-09-14) |
|---|---|
| `scripts/nextest-impacted.sh` (1× sobre o diff acumulado) | ✅ **14 398 testes, 0 falhas** (`75,5 s`, `load 40`) |
| clippy `--workspace --all-targets` com `-D warnings` | ✅ limpo (2 avisos PRÉ-EXISTENTES curados — §6-ter.6) |
| `cargo fmt --all -- --check` | ✅ limpo (18 ficheiros por formatar da jornada anterior, corridos; a varredura re-corrida **depois**) |
| prova de mutação | ✅ **9 de 9 sangram** (§6-ter.7) |
| 3 vermelhos de gate, 1 deles substantivo | ✅ curados na CAUSA, não no gate (§6-ter.6) |

⚠️ **Quatro reprovas foram da família de flakes de carga**, todas verdes `3 de 3`
sozinhas (`load 27`–`92`) e todas com o conjunto a MUDAR entre corridas da mesma
árvore: `measure_brush_kernel` · `the_cost_of_sampling_a_path_is_flat_in_its_anchors` ·
`a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` · e a **nova**
`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`, que fica para
promover (§6-ter.8).

| passo | resultado (2026-09-13) |
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
