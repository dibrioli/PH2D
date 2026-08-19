# Bugs do módulo Flip — registro + soluções

> Log vivo dos bugs **não-triviais** do Flip (sintoma → causa-raiz → tentativas que falharam →
> solução → lições). O objetivo não é listar todo fix (o git já faz isso), mas registrar os bugs
> cuja **causa enganava** — aqueles em que a aparência levou a vários rounds na pista errada.
> Cada entrada termina em **lições generalizáveis**, para o próximo agente não repetir o erro de
> diagnóstico.
>
> Contexto técnico do traço: [`03_traco_rasterizacao.md`](03_traco_rasterizacao.md).
> Referência do Blender: [`02_referencia…`](02_referencia_algoritmos_blender_5.2.md).

> **O que está VIVO aqui:** os **24 bugs estão TODOS fechados**, então o que vale hoje não é nenhum
> deles — são **os INVARIANTES que eles compraram**, a **aparência que o Enio aprovou** (uma cerca de
> Chesterton que protege um DESENHO, não um mecanismo), o que o último fix **ABRE**, e as lições de
> ofício de gate. É isso, e só isso, que ficou abaixo.
>
> O post-mortem completo de cada bug foi movido **verbatim**, em 2026-08-18, para
> [`docs/archive/docs-2026-08-18/Flip/BUGS_flip.md`](../archive/docs-2026-08-18/Flip/BUGS_flip.md)
> — a seção `## #N` de lá tem sintoma · causa · tentativas que falharam · gates · lições.
> ⛔ Nada foi resumido: as duas metades remontam o original byte-a-byte (sha256).

---

## Índice dos 24 FECHADOS — o mecanismo de cada um, em uma linha

| # | O MECANISMO (é isto que se repete, não o sintoma) | Data |
|---|---|---|
| 1 | A **"mordida"** (8 rodadas): a borda macia de um segmento apagava o NÚCLEO de outro — a cura é a cobertura como **UNIÃO GLOBAL da polilinha**, não `over` por segmento. ⚠️ **O Blender tem o mesmo bug e convive com ele.** | 2026-07-12 |
| 2 | O oráculo GPU ficava **VERDE com o bug na tela** porque modelava a **implementação**, não a aparência. Virou regra do módulo. | 2026-07-12 |
| 3 | Linha fina **sumia**: o AA subestimava a cobertura em **10×** — dois bugs, um em cima do outro. ⚠️ *Um mecanismo portado pela metade pode não fazer nada — ou piorar.* | 2026-07-12 |
| 4 | Ponto duplicado **rasgava** o traço: `normalize(0)` = `NaN` no miter. | 2026-07-12 |
| 5 | O broadphase perdia vizinhos (**pad simétrico** — o vizinho mais GROSSO some) e o empate era **não-determinístico**. | 2026-07-12 |
| 6 | O fantasma da camada de cima ficava **atrás** da de baixo: a ordem do onion não é a ordem das camadas. | 2026-07-12 |
| 7 | Os ciclos (Loop/Ping-Pong) **não faziam nada** — o parâmetro existia e ninguém o consumia. | 2026-07-12 |
| 8 | *"Não existe botão fill"*: era **um gate que faltava ao projeto INTEIRO** (o botão pintado nunca foi clicado por teste nenhum). | 2026-07-12 |
| 9 | A barra que **escondia metade de si mesma** (W3). | 2026-07-12 |
| 10 | **A W4 estava morta no produto, e os testes diziam que não** — unit-verde ≠ funciona. | 2026-07-12 |
| 11 | *"Fill impreciso"*: o teto estava na **unidade errada**, e o zoom estragava o balde. | 2026-07-12 |
| 12 | O `grow` era um **chute**, e **nenhuma constante podia acertar** — o número não existia. | 2026-07-12 |
| 13 | A **âncora** do Grow: medir de onde a cor APARECE, não de onde a linha acaba. | 2026-07-12 |
| 14 | A referência do fill × a espessura da linha — o bug que **sobreviveu ao #12 e ao #13**, e a intuição do Enio era a resposta: **âncora no EIXO**. | 2026-07-12 |
| 15 | A cor parava no eixo e a metade EXTERNA da linha ficava sem cor por baixo — **o PIXEL foi o oráculo**; a constante saiu de uma varredura, não do olho. | 2026-07-13 |
| 16 | Os vértices do fill **não eram os da linha**: dessincronização que o zoom amplia. A cura foi **não vetorizar** (a forma fechada pinta a SI MESMA). | 2026-07-13 |
| 17 | **A cura do #16 nunca disparou: nada, no produto, é `closed`** — três sítios, e o terceiro não estava no diagnóstico. | 2026-07-13 |
| 18 | Dava para **VER e REALÇAR** uma aresta que não dava para **APONTAR**: quatro portas para *"quais são os segmentos deste traço?"*. Cura: **uma porta só, no modelo**. | 2026-07-13 |
| 19 | O fill de uma forma que **se CRUZA** saía fora da linha, e Gap/Trap eram irrelevantes — *área é um proxy FRACO de "é a mesma região"*. | 2026-07-13 |
| 20 | A dilatação do fill era **100× grande demais** (erro de UNIDADE) **e** uma **MÉDIA GLOBAL**. ⚠️ **OITO gates de pixel ficaram verdes com a cor 100 px fora da linha.** | 2026-07-18 |
| 21 | A **franja**: um remédio novo tornou o antigo **CONTAGEM DUPLA**, e ninguém aposentou o velho. (3º ajuste da mesma constante = o modelo está errado.) | 2026-07-18 |
| 22 | A dilatação **inteira** era contagem dupla — e a prova estava na **rota irmã**: a referência era o próprio produto (Draw:Filled). | 2026-07-18 |
| 23 | A cor escapava por uma quina que o artista **via FECHADA**: a parede é o **EIXO**, a arte é o **CORPO**. Solda derivada da largura, **sem knob**. | 2026-07-21 |
| 24 | O **proxy de um gate não separava** são de doente: espalhamento e correlação contra a **MEDIANA**. O oráculo foi re-medido — a barra **não** foi afrouxada. | 2026-07-21 |

---

# OS INVARIANTES QUE ESTES 24 BUGS COMPRARAM

> São eles que não podem ser quebrados de novo. O contexto de cada um está no
> [arquivo](../archive/docs-2026-08-18/Flip/BUGS_flip.md), na seção `## #N` indicada.

## #21 — o invariante, e a metade dele que a MEDIÇÃO derrubou

### O INVARIANTE (é isto que não se pode quebrar de novo)

> **A referência do fill é o EIXO da linha.** A cor termina NELE.

⚠️ **Esta seção dizia `largura = w + 2s`, e o termo `w` foi derrubado no #22** — por MEDIÇÃO
contra o Draw:Filled, quatro horas depois de eu escrever aqui que o invariante estava fechado.
A metade certa era *"a referência é o eixo"*; a metade errada era *"e a dilatação leva a cor
dali até a silhueta"*. **O erro está preservado acima de propósito**: ele é a 4ª instância
seguida da mesma doença, e apagá-lo esconderia justamente o padrão.

## #22 — a versão do invariante que sobreviveu à medição

### O INVARIANTE (a versão que sobreviveu à medição)

> **A cor do balde termina no EIXO da linha — exatamente onde o Draw:Filled a termina.**
> A largura do anel do fill é `2s`, e `s` é **só** o erro de vetorização do contorno, com
> sinal. Nenhum termo derivado da ESPESSURA da linha entra na conta. Onde o contorno já está
> sobre o eixo, a largura é **zero**.

## #22 — a APARÊNCIA que o Enio aprovou (cerca de Chesterton)

### A mudança de APARÊNCIA, aprovada explicitamente

Este fix não corrige só um defeito: ele **muda o desenho** com pincel macio. A cauda externa da
linha passa a misturar com o PAPEL em vez de com a cor — exatamente como no Draw:Filled, e é a
consequência direta de a cor parar no eixo.

Isso foi apresentado ao Enio como pergunta separada do defeito, junto com a alternativa
(*"se te desagradar, o remédio não é o `w` de volta; é decidir que o Draw:Filled também está
errado"*), e ele **aprovou** — a resposta veio no mesmo smoke que aprovou o fix.

⚠️ **Registrar isto importa mais que registrar o bug.** Um desenho aprovado é uma decisão do
dono, e sem esta linha o próximo a olhar a franja-que-não-existe-mais vai ler os 2956 pixels de
fundo sob a linha macia (a tabela acima) como defeito pendente, e "consertar" de volta. É a
mesma classe de cerca de Chesterton que o #21 documentou — só que agora ela protege uma
APARÊNCIA, não um mecanismo.

## #22 — o que este fix ABRE

### O que este fix ABRE

Com a lei reduzida a `2s`, a rota `filled_shape_target` deixou de ser um ramo especial: ela é o
**caso particular** em que `s = 0` por construção. O mesmo vale para a rota do arranjo (R1) —
o gate dela agora afirma que **não há nada a dilatar**. É a fatia **R3** do plano
(`10_regiao_por_curvas.md`) ficando barata: aposentar o ramo especial deixou de ser refactor e
virou remoção de código que a lei já subsome.

## #23 — o invariante da SOLDA

### O INVARIANTE

> **Se a tinta que o artista pintou cobre o vão, a parede é contínua ali — porque na TELA
> ela é.** A regra é `distância(ponta, vizinho) ≤ meia-largura(ponta) + meia-largura(vizinho)`:
> DERIVADA da arte, sem knob, sem constante.

## #23 — o achado do controle positivo (medido, nomeado, e depois PAGO)

### ⚠️ Achado do controle positivo, medido e NÃO perseguido

Escrevendo o gate da disjunção: o `reach` que fecha um vão de **1,0 doc** é **4,0** — 4× o
vão (varrido: 0,5 · 1,0 · 1,5 · 2,0 todos `Leaked`). O slider é rotulado pelo **alcance da
extensão**, então o artista que mede o próprio vão e digita esse número recebe um `Leaked`.
É ergonomia do **Gap Closure**, não da solda, e fica **nomeado** aqui em vez de contrabandeado
dentro desta wave ([[feedback_ergonomics_verdict_is_a_design_bug]]).

**✅ PAGO (2026-07-24, junto com o `trap_px` × `MAX_SIDE`):** o 4× **não era ergonomia — era
MECANISMO**. O vão da fixture tem as pontas COLINEARES frente a frente (o traço feito em dois
tempos, o vão canônico), e `ray_hit` trata colinear como PARALELO (`denom ≈ 0` ⇒ `None`): as
extensões se atravessavam sem "colidir", e o vão só fechava quando o raio alcançava a quina
DISTANTE da caixa (a 2,5 do vão — o "4,0" era o degrau seguinte do varrido, não o mínimo).
Cura: **pontas EMPARELHADAS** (`gap.rs` passe 3, a ponte do Harmony) — duas pontas que se
apontam a `dist ≤ reach` fecham pela reta entre elas, com guard de direção (hachura lado a
lado não vira tubo) e sem par degenerado (emenda ponta-na-ponta). **`reach` = o VÃO** no caso
que o artista mede. E o `trap_px` ganhou a régua que sobrevive ao clamp
(`Grid::px_from_requested`, porta única dos DOIS consumidores): o raio é promessa na escala
PEDIDA, e cru na grade clampada inflava a bola na razão do clamp (a "bola de 21,6 doc" do
doc 09) — no balde isso RECUSAVA com `BallTooFat` um clique com folga de sobra (gate
red-proven no corredor 2000×2). ⚠️ **Achado honesto do lado do Colorize:** o oráculo
comportamental da bola inflada NÃO separa lá — a atribuição unifica as câmaras pela moldura
de papel EXTERNA, então costurar uma passagem interna não muda rótulo de saída (medido: cru
e convertido idênticos no cenário de duas câmaras). A conversão entra pela MESMA porta; quem
carrega a prova é o gate do balde + o gate da porta com os números da lei do clamp escritos
à mão (`the_requested_px_door_follows_the_clamp_law`).

## #24 — as lições de ofício de gate

### As lições

1. **Estatística de POSIÇÃO robusta separou; a de FORMA, não.** O defeito do 5º smoke é a
   borda ATRAVESSAR o eixo (corda/zigue-zague); atravessar derruba a mediana e a fração, e
   quase não move a correlação.
2. **Um mínimo sobre ~300 amostras reporta a pior QUINA, não a cobertura da costura.** A
   perpendicular de um segmento não é a direção de offset numa dobra — o *undercut da mitra*,
   que o próprio `snap.rs` documenta com o mesmo 0,24 px. Trocar mínimo por mediana não afrouxa
   o gate: tira dele um ponto isolado que ele nunca quis medir.
3. ⚠️ **Escrever o número esperado no comentário ANTES de medir quase virou doc mentindo:** eu
   documentei *"segue = 0,98-0,99; mutado = 0,17-0,49"* por raciocínio. O medido foi 0,897 ×
   0,879 — e a conclusão INVERTEU (a correlação foi descartada). O comentário só vale depois da
   régua. [[feedback_stale_comment_and_dead_code_lie]]
4. **Não persegui a mitra.** Ela é offset de polilinha ruidosa, que pede trim/join — a saga
   inteira do `curve_join.rs` do Painter. Tentada, medida (limite 2,0 ⇒ mediana intacta mas
   espalhamento 1,93 e p90 pior) e **revertida**: um módulo aprovado em smoke não é lugar de
   contrabandear uma wave para fazer uma barra passar.

---

## Como adicionar um bug aqui

Uma seção `## #N — <título>` com sintoma · causa medida · tentativas que falharam · gates · lições.
⚠️ **Quando ele FECHAR, ele não fica aqui inteiro:** o post-mortem vai para o
[arquivo](../archive/docs-2026-08-18/Flip/BUGS_flip.md), sobra **uma linha no índice, com o
MECANISMO**, e sobe para cá **só** o que ele comprou de durável — um INVARIANTE, uma aparência
aprovada, ou uma recusa medida.
