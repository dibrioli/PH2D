# 06 — O PLANO do que falta no pincel de tecido

> ⚠️ **Leia isto ANTES de pegar qualquer item.** Cada linha traz o número que a define, o que já foi
> tentado e medido, e a pergunta exacta que a destrava — porque **onze** explicações já foram
> construídas, medidas e refutadas nesta linha, e reconstruir uma delas é trabalho pago duas vezes.
>
> A vitória e as réguas: [`05_a_vitoria_medida.md`](05_a_vitoria_medida.md).
> A espec clean-room (atestada): [`SPEC_cloth_brush.md`](../cleanroom/SPEC_cloth_brush.md).
> ⛔ **Quem implementa NUNCA abre o fonte do alvo** — as perguntas vão por subagente-E, pelo INBOX.

## §1 — O estado, em números

`31` dos `56` traços do oráculo estão dentro da barra de paridade (`0,13`) e `7` saem ao bit.

⭐⭐⭐ **E a fila deixou de ser ordenada pelo TAMANHO do erro** (06/09, 3.ª sessão). O tamanho não diz
se falta lei: Gauss–Seidel não comuta, e parte do resíduo de alguns traços vive num regime em que a
**ordem** de resolução decide tanto quanto a lei (a família do §5.2-ter, provada no aperto). A sonda
[`sonda_do_chao_de_ruido`](../../../crates/ph2d-cloth/tests/oraculo_do_pincel.rs) corre cada traço
nas **duas** ordens e devolve o resíduo **em unidades do que uma ordem errada custa** — e essa razão
**inverte a fila que este plano tinha**:

| traço | erro | ordem errada | razão | leitura |
|---|---|---|---|---|
| `esfera_empurrar_radial_dinamica` | `0,3231` | `0,0952` | **`3,39`** | **Push** — lei em falta, e a ordem quase não o move |
| `plano_inflar_radial_local`(`_origem`) | `0,2533` | `0,1025` | **`2,47`** | **Inflate** — idem |
| `esfera_expandir_radial_dinamica` | `0,5574` | `0,2728` | `2,04` | |
| `plano_empurrar_radial_local`(`_origem`) | `0,2141` | `0,1153` | `1,86` | **Push**, o resíduo do plano |
| `esfera_inflar_radial_dinamica` | `0,3780` | `0,2414` | `1,57` | |
| `plano_gancho_radial_local_1passo` | `0,4161` | `0,2894` | `1,44` | Snake Hook |
| `plano_apertar_ponto_radial_local` | `1,3800` | `1,0454` | `1,32` | aperto — o §4 |
| `plano_gancho_radial_local_2passos`(`_origem`) | `0,3881` | `0,3854` | `1,01` | Snake Hook |
| `plano_apertar_ponto_plano_local` | `0,5457` | `0,7137` | `0,76` | aperto |
| `plano_expandir_radial_local` | `0,1918` | `0,6322` | **`0,30`** | **Expand — o MENOS estrutural do corpus** |

⇒ **Push e Inflate são os resíduos determinísticos** e vão à frente. O gancho de poucos passos, o
aperto e o Expand vivem onde a ordem manda, e apertá-los seria ajustar a NOSSA ordenação.

## §2 — A FILA, pela razão da §1

### ✅ FECHADO em 06/09 — o censo da ESFERA (§4.6) inteiro

As **seis** grandezas que degeneram num plano visto de frente estão todas respondidas deste lado:

| # | grandeza | estado |
|---|---|---|
| 1 | o deslocamento `δ` projectado no plano do ecrã | ✅ implementada (Q12, 2.ª sessão) |
| 2 | a normal da área (disco de meio raio, peso `3p²−2p³`) | ✅ implementada — Push `0,944 → 0,215` |
| 3 | **o centro da área** (a média das posições **puxadas para o cursor**) | ✅ **implementada agora** — os quatro traços de falloff de plano melhoram, dois entram na paridade |
| 4 | a normal do vértice sobre a malha deformada | ✅ a FORMA já estava certa (soma face a face, malha actual, normalizada no fim); o **peso** é a metade que a própria §4.6 declara aberta, e o corpus continua sem o decidir (`PH2D_PESO_NORMAL=uniforme`: `0,253 → 0,253`) |
| 5 | os dois baldes pelo sinal contra a vista | ✅ implementada com a #2 |
| 6 | a distância como **corda 3D**, nunca geodésica | ✅ já era — toda distância desta crate passa pelo mesmo `dist` euclidiano |

⛔ **Fechar o censo não «cura a esfera», e é isso que ele serve para dizer:** o que sobra nos traços
de esfera é o resíduo do **modo**, não da curvatura — o arrasto, que é o CONTROLO da §4.6, bate
(`0,092`), e os outros sete carregam exactamente o resíduo que o mesmo modo tem no plano.

### ✅ FEITO em 06/09 — o CENTRO DA ÁREA (§4.4)

O plano de queda deixou de passar pelo cursor. Ele passa pelo **centro da área**, que **não é o
centroide do disco**: cada vértice entra na média já puxado para o cursor por `1 − a`, com `a` a mesma
*smoothstep* da normal — ⇒ zero no cursor, cresce para a borda.

    plano_empurrar_plano_local        0,215 → 0,106   (entra na paridade)
    plano_agarrar_plano_local         0,180 → 0,124   (entra na paridade)
    plano_arrastar_plano_local        0,233 → 0,134
    plano_apertar_ponto_plano_local   0,613 → 0,546

⚠️ **A recusa de 06/09 não foi refutada — ela respondia a outra pergunta.** «O plano pelo centro da
área afasta-o» (`empurrar 0,944 → 1,250`) foi medido com um **centroide**, e o alvo não usa um
centroide; o plano pelo cursor é a aproximação de **primeira ordem** desta lei.
⚠️ E o disco é centrado na **localização do cursor do modo**, que no Agarrar fica no pen-down durante
todo o traço — com o disco a seguir o cursor que anda, aquele traço sobe a `1,094`.

### ⭐ 1.º — o PUSH e o INFLATE, que são os dois resíduos DETERMINÍSTICOS

**O que se sabe.** Os dois têm o traço de um passo **ao bit** e a ordem de resolução quase não os
move (razões `3,39` · `2,47` · `1,86` · `1,57`) ⇒ *falta lei, e ela não se esconde atrás da
não-comutatividade.* O perfil radial do `plano_inflar_radial_local_origem` diz que **não é
magnitude uniforme**: o pico está no mesmo sítio (`x = 0,516`) e a banda é idêntica, mas a nossa
deformação **sobra atrás** do traço (`0,0307` contra `0,0215`) e **falta à frente** (`0,0076` contra
`0,0481`, `84 %` abaixo). *A frente do oráculo alcança muito mais longe que a nossa.*
⏳ A pergunta está no INBOX como **Q14** — e ela pode ser a mesma resposta do gancho, porque as duas
são sobre a **resposta da rede**, não sobre o gesto.

### ⭐ 2.º — o SNAKE HOOK de poucos passos, que é a mesma pergunta

**O que se sabe.** ⭐⭐ **O erro é função do TAMANHO de `δ` por passo**, no mesmo caminho e com o mesmo
pincel: `δ = 0,600 → 0,416` · `0,300 → 0,388` · `0,026 → 0,062`. E o perfil do traço de um passo
prova duas coisas: as duas caudas **decaem à mesma taxa** (`0,78` por célula, ao longo de 15 células)
⇒ a rede transmite igual; e o oráculo tem um **degrau de `4,24×` numa célula** a `0,45R`–`0,59R` do
centro, que a curva *smooth* não produz (nós caímos `2,23×`).
⏳ INBOX **Q14.2**.

### 3.º — o APERTO e o EXPAND, que vivem onde a ORDEM manda

Razões `1,32` a `0,30`. ⛔ **Não procure uma lei em falta aqui antes de o 1.º e o 2.º fecharem** — o
`plano_expandir_radial_local` erra `0,192` e uma ordem errada custa `0,632`, ou seja *o resíduo dele é
menos de um terço do que a nossa própria escolha de ordenação vale*. O aperto de força alta é o §4,
que é decisão do dono.

## §3 — ⛔ RECUSAS MEDIDAS — não as reconstrua

| o que foi tentado | o número que o matou |
|---|---|
| o anel-1 pela **triangulação** | acerta o Arrastar *Local* e derruba o *Global* de `0,6457` para `0,2699` |
| **`PH2D_VARREDURAS=10`** como lei do *Local* | bit-idêntico à construção dupla nos modos de força e **diverge** nos de âncora |
| o filtro de raio na criação de restrições da área *Dynamic* | `0,181 → 0,182`, e as outras nove inalteradas |
| a **direcção do aperto medida no repouso** | melhora o plano (`1,380 → 1,012`), piora a esfera (`0,542 → 0,939`) |
| a **trava** que impede o vértice de ultrapassar o alvo | **não é inerte**: parte os traços de um passo que hoje saem ao bit |
| o plano de queda pelo **CENTROIDE** do disco | `empurrar 0,944 → 1,250`, `arrastar 0,233 → 0,716`. ⚠️⚠️ **Esta recusa respondeu a UMA pergunta, e não é a da §4.4:** o alvo não usa um centroide — usa a média das posições **puxadas para o cursor**, que shipa desde 06/09 e melhora os quatro traços de plano. *O plano pelo cursor era a aproximação de 1.ª ordem dela, e é por isso que passava quase.* |
| a projecção do `δ` no plano **tangente do pen-down** | `agarrar 0,265 → 0,605`, `gancho 0,351 → 0,663` |
| **faces invertidas** como régua de classificação | não discrimina: o arrasto tem `41`–`57` e bate a `0,071` |
| a **compressão** do par mais apertado como régua | explica a família do aperto e nada mais |
| a **soma crua** das normais da área (sem o peso `3p²−2p³`) | ⚠️ **melhora** cinco traços (`0,214 → 0,194`) e mesmo assim **não** shipa: a lei da espec foi lida no fonte e atestada, e o que a medição diz é que o peso não é a causa do resíduo do Push |
| o **peso** da normal por vértice, RE-MEDIDO contra a lei de hoje | `uniforme` contra `área`: `inflar 0,253 → 0,253`, `esfera 0,378 → 0,379`, `empurrar 0,214 → 0,213` — a recusa de 06/09 continua de pé depois do Q12 e do centro da área |
| a ordem inversa como **CHÃO DE RUÍDO** (ruler minha, construída e apagada) | ⛔ inverter a ordem não é ruído, é uma lei ERRADA: erramos `0,0713` contra o oráculo no `plano_arrastar_radial_local` e a ordem inversa move-nos `0,2932` — *estamos 4× mais perto do alvo do que ela está de nós*, e um chão de ruído nunca é maior que a distância ao alvo |
| partir o corpus em «ruído de ordem» contra «lei em falta» com barra em `0,35` | ⛔ **sem VALE**: as razões dos 25 abertos são um contínuo de `0,30` a `3,39` e o maior vazio (`0,92`) está no topo, entre os dois últimos ⇒ qualquer barra a meio seria escolhida (CLAUDE.md §0.0). O gate foi apagado; a sonda ficou sem veredito por traço |
| a **banda `φ`** das restrições como causa do espalhamento (`PH2D_ESC_PHI` de `0,4` a `1,3`) | `1,0` é um óptimo AGUDO e o corpus di-lo pelo arrasto: `plano_arrastar_radial_local` mede `0,482 · 0,359 · 0,192 · 0,071 · 0,447` na varredura, e o Push/Inflate mal se mexem (`0,214`/`0,253` contra `0,207`/`0,274` a `0,8`) |
| **mais varreduras** de relaxação (`PH2D_VARREDURAS` `5 → 10 → 20`) | destrói o arrasto: `0,071 → 0,709 → 0,910`. ⚠️ O Inflate MELHORA (`0,253 → 0,191`) e não compra nada — *um knob que cura um traço e parte outro não é a lei que falta* |
| a **retenção** de velocidade (`PH2D_ESC_RET` de `0,6` a `1,3`) | `1,0` está no óptimo e satura acima dele: `arrastar 0,138 · 0,082 · 0,071 · 0,070` |
| tirar a **banda `φ`** da relaxação (leitura literal da §5.4, `PH2D_PHI_SEM_BANDA`) | destrói o arrasto: `plano_arrastar_radial_local` `0,011 → 0,443`, e o Push/Inflate pioram (`0,237 → 0,255`, `0,245 → 0,266`). ⇒ a banda **está** no `φ` da relaxação |
| tirar a banda só do termo de **ACELERAÇÃO** da integração (a outra leitura da §5.4) | **byte-idêntico nos 65 traços** — e a razão é estrutural: dentro do pincel a banda vale `1`, e fora dele a força vale `0`. *Onde `a ≠ 0` a banda é `1`* ⇒ a frase da espec e o nosso código dizem a mesma coisa, e o knob seria morto |
| **mais varreduras**, RE-MEDIDO com a ordem de visita certa | `5` é agora um óptimo AGUDO e quem o diz é o arrasto: `0,011 · 0,254 · 0,555 · 0,705` para `5 · 6 · 8 · 10`. ⚠️ E o Push/Inflate MELHORAM com `8` (`0,164`/`0,162`) — *dois erros a compensarem-se, não a lei que falta* |
| o **peso** da normal por vértice, medido pela TERCEIRA vez | com a ordem de visita certa: `inflar 0,245 → 0,245`, `empurrar 0,237 → 0,237`. A metade que a §4.6 declara aberta continua sem o corpus a decidir |
| **re-apanhar o cursor na superfície deformada** (o vértice mais próximo no plano do ecrã, posição actual) | destrói tudo: `arrastar 0,012 → 0,814`, `empurrar 0,252 → 0,825`, `inflar 0,248 → 0,630`. ⚠️ **A recusa é da APROXIMAÇÃO, não da ideia** — o vértice mais próximo faz o cursor saltar de vértice em vértice e herdar a cova inteira. O `caminho` das fixtures traz `z = 0` nos doze passos, e a §4.3 diz que o cursor é re-apanhado; a pergunta continua aberta |
| **re-apanhar o cursor na superfície deformada**, 2.ª tentativa: interpolação SUAVE dos vizinhos no plano do ecrã (⛔ não o vértice mais próximo, já refutado) | pior ainda: `empurrar 0,252 → 0,937`, `inflar 0,248 → 0,608`, e o **arrasto fica intacto** (`0,012`) nas duas tentativas, porque no plano ele não afunda. ⇒ *o `caminho` das fixtures É o cursor que o alvo usa; ele fica no plano de partida* |
| a queda medida na posição de **REPOUSO** em vez da actual (`PH2D_D_REPOUSO`) | destrói: `arrastar 0,012 → 0,654`, `inflar 0,248 → 0,862`. A §4.1 diz **actual** para todos menos o Agarrar, e a medição concorda |
| a regressão do §9 nº 20 da espec como causa do aperto | foi **fechada em 2024**, dois anos antes da versão que gravou as fixtures |

## §4 — ⛔⛔ A DECISÃO que é do DONO (e não há terceira saída)

Ao apertar com força alta, a inversão nasce no **primeiro** passo, antes de a relaxação correr, logo
nenhuma afinação do solver a evita. As duas saídas, nas frases que a espec §5.2-ter fixa:

- **(a) reproduzir** — o retalho debaixo do cursor vira do avesso, as faces atravessam-se e a
  superfície fica com um nó que nada desfaz depois; é o que o alvo faz hoje, e apertar com força
  baixa continua limpo. **É o que shipa agora.**
- **(b) limitar** — o aperto nunca ultrapassa o ponto para onde puxa, o nó não aparece em força
  nenhuma, e a nossa saída deixa de casar com a do alvo exactamente nos traços fortes.

⚠️ **O alvo sabe que (a) é defeito dele** — são duas entradas ABERTAS do tracker dele.

## §5 — Os instrumentos que já existem (⛔ não construa outro sem olhar)

```
cargo test --release -p ph2d-cloth --test oraculo_do_pincel <sonda> -- --ignored --nocapture
```

| sonda | o que devolve |
|---|---|
| `sonda_da_paridade_com_o_oraculo` | o corpus inteiro: movidos, máximos, erro e a razão, **nos dois lados** |
| `sonda_passo_a_passo` (`PH2D_TRACO=<nome>`) | passo a passo: o anel imediato, o **erro do passo**, onde está o **pico**, o vector do vértice do pen-down |
| `sonda_dos_artefatos_do_oraculo` | espinho · rasgo · estica · **faces invertidas** · **compressão**, nos dois lados |
| `sonda_do_perfil` · `sonda_da_cadeia_com_parede` | o perfil radial e a cadeia 1D |

Experiências por env, para bissecar: `PH2D_VARREDURAS` · `PH2D_ORDEM` (`inversa`, `celula:<n>`) ·
`PH2D_PARES=0` · `PH2D_TRI` · `PH2D_ESC_PHI` · `PH2D_ESC_RET`. No produto: `PH2D_CLOTH_LAW=vbd`.

## §6 — O PROTOCOLO, em quatro passos

1. **Meça** com a sonda por passo até localizar o defeito num vértice, num passo, ou numa grandeza.
2. **Refute o que puder sozinho** — metade das perguntas morre aqui, e cada refutação vai para o
   INBOX com o número.
3. **Escreva a pergunta no INBOX** (`docs/3D/cleanroom/INBOX_blender-cloth.md`) com a tabela dentro,
   e despache um **subagente-E**. ⛔ Contrato de retorno: uma frase, cinco linhas funcionais, zero
   identificador do alvo.
4. **Despache o R-pré** antes de LER a emenda, implemente, meça outra vez, e escreva o gate com a
   prova de mutação.
