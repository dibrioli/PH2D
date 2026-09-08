# 07 — O FILTRO DE TECIDO (a segunda metade da W10)

> ⚠️ **Este doc descreve o mundo no dia em que foi escrito** (2026-09-07). O estado
> vivo é o `CLAUDE.md` §5; os números vivos estão no código, e as duas listas do
> [`oraculo_do_filtro.rs`](../../../crates/ph2d-cloth/tests/oraculo_do_filtro.rs)
> são a fonte do placar — ⛔ **não copie o placar para aqui.**

A W10 sempre foi *«Cloth (XPBD) + Cloth Filter (5 tipos)»*
([handoff de 19/08](../handoffs/HANDOFF_CONTINUACAO_line_sculpt3d_2026-08-19.md)).
O pincel fechou em 06–07/09; isto é a outra metade.

---

## §1 — A MEDIÇÃO QUE VEIO ANTES (e o que ela dispensou de construir)

*Antes de construir um item de lista aberta, MEÇA se a composição já o exprime*
(`CLAUDE.md` §5.0). A bancada é
[`mede_a_composicao_do_filtro.rs`](../../../crates/ph2d-cloth/tests/mede_a_composicao_do_filtro.rs)
e ela responde pelo OBSERVÁVEL — o que a malha faz —, nunca por leitura de código.

| o que se perguntou | o que a medição deu |
|---|---|
| a **área** do filtro é obra nova? | ⭐ **não.** `Area::Global` já devolve `w ≡ 1` por porta, põe toda a malha activa e faz a construção correr **uma vez** — as três cláusulas da espec §7 |
| o **Expand** e o **Pinch** são leis novas? | ⭐ **não.** O `Expandir` escreve `τ += 0,01·f` ao pé da letra da §7; o `ApertarPonto` com o cursor congelado é o *Pinch* |
| e o **Inflate**? | ⚠️ **é a mesma lei com outra fotografia.** A lei lê `Passo::normais`, que é do CHAMADOR: o traço dá as do início, o filtro refresca-as. Medido: **`0,000`** de divergência com um passo simulado (o controlo) e **`30,30 %`** com seis |
| e a **Gravity** e a **Scale**? | ⛔ **expressão nova, e há prova.** Rodando a peça **e** o gesto um quarto de volta, os oito modos do pincel rodam junto (desvio de equivariância `≤ 5,0e-11`) ⇒ **nenhum** exprime uma direcção de MUNDO, que é o que a orientação da §7 compra |
| dá para levar o `S` do filtro num `Pincel`? | ⛔ **não.** O `B` do traço é `10 · força² · flip` — **quadrático e sem sinal** (picos `0,00625 / 0,025 / 0,100` para `0,25 / 0,50 / 1,00`, erro `0,0000`; força `−1` dá o mesmo que `+1`) e o `S` da §7 é uma **recta com sinal** |
| e esconder «não há pincel» atrás de um raio enorme? | ⛔ **não.** O `Empurrar` traz o raio DENTRO da magnitude (`2R`): com raio `1e3` o pico dele salta de `~0,3` para **`598`**, `2000×`. *Um raio que finge ser infinito é lido como comprimento por quem tem comprimento na lei* |

⛔⛔ **E a régua da última linha errou primeiro.** A 1.ª redacção do gate da
gravidade perguntava se a direcção é **constante** sobre a peça, e acusou três
modos — *Push* (`1,0000`), *Grab* e *Snake Hook* (`0,999`). Todos certos e nenhum
é uma gravidade: a normal da **área** é **um** vector (não a de cada vértice), e
os de âncora levam toda a gente ao longo do `δ`. *Uma direcção única não é uma
direcção de MUNDO* — o discriminador é a **equivariância**.

---

## §2 — O QUE FOI CONSTRUÍDO

| peça | onde | o que ela é |
|---|---|---|
| `Accionamento` | [`verlet_gesto_pincel.rs`](../../../crates/ph2d-cloth/src/verlet_gesto_pincel.rs) | `Traco` (o `B` quadrático da §4.1) contra `Filtro { s }` (a recta com sinal da §7). ⚠️ No traço o `B` muda por arm (`10` nos de força, `0,1` no Expand); no filtro é **um `f` só** para os cinco |
| `Modo::Gravidade` | [`verlet_gesto.rs`](../../../crates/ph2d-cloth/src/verlet_gesto.rs) | a única direcção do ficheiro que não sai da malha nem do cursor |
| `Modo::Escala` | idem | o único tipo por ÂNCORA: `p⁰ + p⁰·f`, eixos desligados anulados no `Referencial` |
| `ClothFilterKind` · `ClothFilterOrientation` | [`cloth_filter_kind.rs`](../../../crates/ph2d-sculpt3d/src/cloth_filter_kind.rs) | o vocabulário do painel |
| `FilterLaw` | [`filter_law.rs`](../../../crates/ph2d-sculpt3d/src/filter_law.rs) | a **união** das nove leis de malha com os cinco tipos de tecido |
| `cloth_filter_{begin,step,end}` | [`stroke_cloth_filter.rs`](../../../crates/ph2d-sculpt3d/src/stroke_cloth_filter.rs) | o adaptador |
| o gesto + o painel | `shells/desktop/src/sculpt3d_filter.rs` · `ph2d-panel-sculpt3d` | duas fileiras de chips + a do referencial |
| a cena **`=37`** | [`sculpt3d_scenes_cloth_filter.rs`](../../../shells/desktop/src/sculpt3d_scenes_cloth_filter.rs) | o roteiro do dono, em nove passos |

### §2.1 — Por que uma UNIÃO e não um terceiro modo armado

⚠️ **O código já dizia onde ficava a fronteira:** o doc do `arm_filter` regista
que a exclusão filtro/transform vive em **duas portas** *«porque um `enum`
obrigaria a reescrever os cinco leitores do `transform_arm`»*. Com um terceiro, a
exclusão passa a ser **três pares** e o argumento inverte-se. E os dois filtros
são o **mesmo gesto** para quem usa o app: armar, arrastar na horizontal, uma lei
na peça inteira, **um** passo de undo. O que muda é a lei — que é o que um
selector escolhe.

⚠️ **Duas fileiras e não catorze chips numa:** *Inflate* e *Scale* existem nos
dois lados e são leis diferentes. *Um chip cujo rótulo não distingue a lei precisa
da fileira para o fazer.*

### §2.2 — A FASE 0 DO TRAÇO NÃO É DO FILTRO

⭐⭐⭐ Dois gates apanharam a mesma coisa: o primeiro movimento do rato não movia um
vértice (`0 de 625`). A espec §1 dá o MOTIVO dentro da própria cláusula — *«no
1.º passo o deslocamento do cursor é zero, e os modos que dele tiram direcção,
referencial ou alvo não têm resposta definida sem ele»* — e **nenhum dos cinco
tipos do filtro tira nada de um deslocamento de cursor**. A §7 confirma pelo outro
lado: as restrições nascem **ao carregar** e a lista de fases do passo dela
**começa na fase 2**.

⇒ o pen-down corre um passo de força **zero**, que é a carga da espec.
⛔ Não se mexe no `primeiro` à mão — isso seria escrever a lei a partir de fora dela.

### §2.3 — O GESTO É O OPOSTO DO FILTRO DE MALHA

O de malha **repõe a pose congelada** a cada passo (voltar com o dedo desfaz).
Este **acumula**: a §7 manda um passo de simulação por movimento do rato. Repor a
pose apagaria a memória do tecido (velocidade, plasticidade, desvio de repouso) e
o pano deixaria de cair — seria um filtro de malha a usar um solver caro para
nada. Há gate: um passo move `0,0100` e três movem `0,0596`.

---

## §3 — O CORPUS, E AS DUAS LEIS QUE ELE APANHOU

O oráculo passou a ter corpus de filtro (**17 corridas**, `fixtures/cloth/filtro/`,
espec §10.17), colhido por um subagente-E fora da árvore. ⛔ **O placar vive nas
duas listas do [`oraculo_do_filtro.rs`](../../../crates/ph2d-cloth/tests/oraculo_do_filtro.rs)** —
conte-o lá.

⭐⭐⭐ **E a bancada apanhou duas leituras erradas minhas, as duas da mesma
família — *uma frase da espec que nomeia UM número, e um código que tem DOIS*:**

| o que estava errado | erro antes | erro depois |
|---|---|---|
| a **Escala** aplicava a força de âncora **duas vezes** (o `0,01` da §7 é a **rigidez** de `ancorar(v, s)`; o `σ` do passo é a **activação**, e vale `1` como no Grab) | `0,988876` | **`0,007443`** |
| o **aperto** puxava para o **ponto do cursor** e o alvo puxa para um **VÉRTICE** | `0,696942` | **`0,010755`** |

⚠️ **O ponto de abertura nem sequer está no ficheiro:** os `c` gravados são as
posições do cursor **depois** de cada movimento. Ele extrapola-se de
`c₀ − (c₁ − c₀)` e encosta-se no vértice mais próximo.

⚠️⚠️ **E metade da segunda cura ficou fora do produto por um dia inteiro** — a
bancada encostava no vértice e o `filter_pinch_anchor` do shell continuava a
devolver o ponto da face. *Um gate que mede a lei é cego a quem lhe entrega os
argumentos.*

### §3.1 — O aberto que vale mais que o número

⛔ `plano_filtro_expandir` fica **ABERTO** com o diagnóstico **estreito**: a
QUANTIDADE está certa (o nosso máximo é `0,379332` contra `0,379347` do oráculo —
`4e-5`) e o que difere é o **PADRÃO**. Uma folha que cresce **encurva**, e para
que lado ela encurva sai de assimetrias minúsculas. ⭐ **O irmão NEGATIVO bate**
(`0,0197`), o que estreita a pergunta à **flambagem** e iliba o desvio de repouso.

Os outros dois abertos são **AUSÊNCIAS declaradas** — conjuntos de faces e
gravidade de cena não existem neste app. *Uma lei sem entrada não se mede.*

---

## §3-bis — ⛔⛔ O REPORT DE PERFORMANCE (07/09), e os DOIS defeitos que ele tinha dentro

> *«diferente dos filtros antigos e do pincel cloth que tem ótima performance,
> esses filtros novos têm péssima performance e estão impossíveis de usar»*

A sonda é
[`mede_o_filtro_de_tecido.rs`](../../../crates/ph2d-sculpt3d/tests/mede_o_filtro_de_tecido.rs),
e ela mede o **EXPOENTE** e não um relógio: um número de milissegundos desta
workstation não sobrevive à carga, mas *se dobrar os vértices multiplica o custo
por quatro, nenhuma máquina salva a ferramenta*.

### (1) O anel-1 era QUADRÁTICO — e escrito por mim ao lado da porta certa

A 1.ª redacção do adaptador escreveu um anel-1 próprio que **varria todas as
faces por cada vértice** (`O(V·F)`), onde o pincel — no ficheiro **irmão** — já
lia a tabela pronta de `mesh.adjacency()` em `O(1)`.

| | antes | depois |
|---|---|---|
| expoente do pen-down | **`V^1,77`** | **`V^1,00`** |
| pen-down a `6 836` vértices | `50,46 ms` | `4,12 ms` |
| pen-down na malha do smoke (`98 306`) | **`~5,6 s`** de paragem | **`106 ms`** |

⛔⛔ **E ele estava errado das DUAS maneiras que importam:** também fazia
`sort_unstable()`, e a espec §3.1 diz que a ordem do anel é a das **FACES** —
ela fixa a ordem da lista de restrições, e Gauss-Seidel **não comuta**. *Um
`sort` é uma ordem NOSSA a substituir a do alvo*, e a bancada do pincel já tinha
esse aviso escrito, em prosa, a três ficheiros de distância.

⇒ *uma segunda resposta a uma pergunta que a casa já tinha respondido — e o preço
dela foi um expoente **e** uma lei.*

### (2) Um evento de ponteiro NÃO é um passo de simulação

⛔⛔ O filtro corria **um passo de solver por evento do sistema operativo**. Um
rato de `1000 Hz` entrega ~16 por quadro; um de `125 Hz`, dois. ⇒ *quantos passos
a simulação avança era função da TAXA DE AMOSTRAGEM do rato* — **a lei que esta
casa já pagou seis vezes no relevo do Painter**, e que o braço **vizinho** do
mesmo `match` já escrevia: *«um evento de ponteiro NÃO é um dab»*.

⚠️ **É de CORRECÇÃO antes de ser de relógio:** dois artistas com ratos diferentes
obtinham panos diferentes do mesmo gesto. Que o report de performance caia junto
é consequência.

⇒ o evento **regista** e o **QUADRO** drena (`flush_cloth_filter`), exactamente
como o `flush_pending_grab` que já vivia ao lado — incluindo a chamada no pen-up,
*senão o gesto perde a ponta*. ⛔ As leis de **malha** continuam imediatas de
propósito: elas repõem a pose congelada e reaplicam, logo são **idempotentes** no
mesmo `x`.

⭐ Gate: `the_cloth_filter_advances_per_frame_and_not_per_pointer_event`, com as
três metades — o evento não mexe · o quadro mexe · **dez eventos e um quadro dão
o MESMO que um evento e um quadro, ao bit**.

### (3) E ele destapou um terceiro, que era de MIRA

O gate do vértice leu a âncora do aperto no **centro da caixa** — o raio nem
sequer acertava na peça. Causa: o `begin_filter` lia `self.last`, e o
`sculpt3d_pointer_down` escreve esse campo **depois** de o chamar, enquanto o
`pointer_move` só o actualiza **com um arrasto em curso**. ⇒ no pen-down ele
guardava *onde o gesto ANTERIOR acabou*. Hoje o ponto chega por **argumento**.

### O TECTO QUE SOBRA É A LEI, medido por ablação

Na malha do smoke o passo custa **`47,5 ms`**. Com `VARREDURAS = 1` em vez de `5`
ele custa **`12,8 ms`** ⇒ cada varredura vale `~8,7 ms` e **`~91 %` do passo é a
relaxação das restrições**; tudo o resto são `~4 ms`.

⇒ ⛔ **atacar as alocações compraria `8 %`**, e a relaxação **não se paraleliza**
(Gauss-Seidel numa ordem que é metade da lei). *O tecto é o do modelo, não o da
implementação* — quem quiser mais quadro por segundo reduz a **MALHA**, e o botão
de retopologia já existe.

---

## §4 — ⛔ O QUE FICA ABERTO, com o que acorda cada item

| item | o que falta | o gatilho |
|---|---|---|
| **`Force Axis`** (as bandeiras X/Y/Z que só a Escala lê) | ⚠️ **não é um knob morto: é um controlo que NÃO EXISTE.** A distinção importa — a cura é criá-lo, não ligar um braço | um pedido do dono; o motor já o honra (gate `escala_eixox` a `0,009151`) |
| **gravidade da CENA** | a escultura desta casa não tem esse ajuste. ⚠️ A espec tinha **três** afirmações erradas sobre ela (sinal, espaço e a dependência do arrasto), corrigidas na emenda de 07/09 | o dia em que a escultura ganhar um ajuste de gravidade |
| **conjuntos de faces** | o app não os tem | idem |
| a **flambagem** do Expand | §3.1 | quem quiser o `0,688534` |
| a **esfera** (`0,028226`) | está `2,6×` acima do sorteio do próprio oráculo (`0,0107`) | ⚠️ **a barra tem de continuar acima do sorteio dele** — abaixo mediria a lotaria do alvo |
| as **colisões** no filtro | o traço tem-nas; o filtro passa `&[]` | ⛔ o preço medido do traço (`2,6×` a `6,1×` o dab) vale igual aqui, e a peça inteira é o pior caso |

---

## §5 — O SMOKE

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=37 cargo run -p ph2d-host-desktop --release
```

O roteiro dos nove passos é impresso pela própria cena
([`sculpt3d_scenes_cloth_filter.rs`](../../../shells/desktop/src/sculpt3d_scenes_cloth_filter.rs)) —
⛔ **não o duplique aqui**, senão as duas cópias divergem e a que o artista lê é a
que envelhece.
