# SPEC — reescrita dos comentários da família da escultura que citam o alvo por nome interno

| campo | valor |
|---|---|
| Alvo | Blender (escultura: filtros de malha, dureza e curva do pincel, plano do pincel, simetria, verbos de carimbo) — o mesmo alvo do ledger `LEDGER_blender-cloth.md` |
| Licença do alvo | GPL-2.0-or-later |
| Data | 2026-09-13 |
| Origem | INC-4 do ledger (incidente registado pela janela I `9f820704-0d7e-4d96-847e-9cd720cbf178` no INBOX, 2026-09-13) |
| Auditoria | **auditado contra §4.2 por R** (subagente-R do INC-4, contexto novo, viu os dois lados), 2026-09-13 |
| Sweep | `bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_blender-cloth.txt <este ficheiro>` verde + busca pelos identificadores internos da população: zero ocorrências (registado no ledger) |

> **Este documento descreve o que cada comentário deve dizer; não contém expressão do alvo.**
> Ele não cita nenhum nome interno, nenhum trecho de código e nenhum número de linha do fonte do
> alvo. Os únicos nomes do alvo que aparecem são nomes de **API pública** verificados correndo o
> oráculo (§4.1.13): `hardness`, `normal_radius_factor`, `tip_scale_x`,
> `sharpen_intensify_detail_strength`, `surface_smooth_shape_preservation`,
> `surface_smooth_current_vertex`, e os **rótulos que o artista vê** na interface (*Smooth*,
> *Scale*, *Inflate*, *Sphere*, *Random*, *Relax*, *Surface Smooth*, *Enhance Details*, *Sharpen*,
> *Layer*, *Clay Strips*, *Clay Thumb*, *Multiplane Scrape*, *Crease*, *Blob*, *Auto Smooth*,
> *Front Faces Only*, *Advanced*, *face set*).

---

## §0 — Como aplicar (regras que valem para TODAS as linhas abaixo)

1. **A unidade de trabalho é o BLOCO de comentário**, não a linha. Cada entrada dá as linhas do
   sítio e o intervalo do bloco onde ele vive; reescreva o bloco inteiro de uma vez, dizendo **o
   facto** da coluna «facto a manter».
2. **Sai todo token entre crases, nas linhas indicadas, que designe uma função, variável, campo,
   constante, classe ou ficheiro do programa de referência.** ⚠️ Um token entre crases pode
   **atravessar duas linhas** — ele conta na mesma.
   - Quando a frase precisa de nomear a coisa, use a descrição de domínio da coluna «facto a
     manter» (ex.: *«a etapa da referência que remapeia a distância normalizada pela dureza»*).
   - ⛔ Não traduza o nome interno palavra a palavra: descreva **o que a etapa faz**.
3. **Sai todo número de linha do fonte da referência** (a forma `:NNN` ou `:NNN-MMM` sem nome de
   ficheiro nosso ao lado). ⚠️ **Não** confunda com citações de linha de ficheiros **nossos**
   (`stroke.rs:397`) nem do SculptGL (`Masking.js:66-69`, `Crease.js:67` — licença MIT, ficam).
4. **Sai todo fragmento de código da referência** — condição, guarda de retorno antecipado,
   chamada com argumentos, expressão com a notação de variáveis dela, cadeia de chamadas com
   setas. O que ele dizia vira **uma frase de comportamento** (ex.: *«com dureza zero a etapa
   não corre»*).
5. **Fica** tudo o que é nosso ou é facto: números medidos, nomes de gate, nomes de sonda, nomes
   dos nossos tipos e funções, fórmulas em notação matemática nossa (`(p̂ − p)/2 · |f|`), a
   divergência declarada, a decisão do dono e a fixture que mede.
6. **Palavras de fidelidade sem prova saem ou ganham o gate:** «verbatim», «ao pé da letra»,
   «transcrição literal», «a estrutura lida de cima a baixo» passam a *«a mesma lei — medida por
   `<gate>`»*. Se não houver gate que a meça, a frase fica só com o comportamento.
7. **Fica** «a referência» e «o Blender» como nome do programa, e os nomes públicos listados no
   cabeçalho. Um identificador em maiúsculas que seja o **identificador público** de um tipo de
   filtro (o que a API em Python aceita) pode ficar; uma bandeira ou constante interna em
   maiúsculas sai.
8. Duas linhas deste plano estão marcadas **⛔ APAGAR**: são **citações de comentário do alvo**
   (§4.2: «a expressão mais protegida do arquivo»). Não se reescrevem — apagam-se, e fica só o
   comportamento.

As entradas estão agrupadas por ficheiro. Em cada ficheiro, a tabela principal cobre os sítios da
população do incidente (a régua da janela: nome `snake_case` de 3+ palavras entre crases, em
comentário); a linha «mesmo ficheiro, fora da régua» lista outras linhas do MESMO ficheiro com a
mesma dívida, para que a edição seja uma só. O **Anexo B** tem os ficheiros que só aparecem fora da
régua.

---

## §1 — `crates/ph2d-panel-sculpt3d/src/ids/sculpt3d_brush.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 83 (79-85) | O controlo de dureza do pincel remapeia a **distância normalizada** que qualquer curva de queda consome; não é o expoente da curva própria do canal de máscara. | o nome interno da etapa de dureza (entre parênteses) |

Mesmo ficheiro, fora da régua: **91** (1 nome interno do factor de auto-alisamento — use o rótulo *Auto Smooth*).

## §2 — `crates/ph2d-panel-sculpt3d/src/paint/brush.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 137 (123-142) | No `Mask` a dureza ainda chega, porque o nosso `shaped_distance` corre **antes** da curva do canal — a mesma ordem da referência, que aplica a dureza antes de avaliar a curva de queda do pincel. | 2 nomes internos na linha 137 (a etapa de dureza e a avaliação da curva) |

## §3 — `crates/ph2d-panel-sculpt3d/src/rows.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 89 (85-103) | A dureza remapeia a distância que a curva do dab lê; sob campo elástico não há curva do dab e a dureza é inerte (medido pela sonda e gate já citados no bloco). | 1 nome interno |
| 174 (172-176) | O valor de fábrica `0` é o **neutro** da etapa de dureza da referência: com dureza zero a etapa não corre. Por isso escondê-la no Basic não tira capacidade. | 1 nome interno **e** o fragmento da guarda de retorno antecipado |

Mesmo ficheiro, fora da régua: **388** (1 chamada interna com prefixo de biblioteca do declarador de propriedades — diga «a faixa declarada da propriedade na referência»).

## §4 — `crates/ph2d-panel-sculpt3d/src/state_modes.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 48 (41-50) | Na interface da referência o painel de queda **não** está dentro da secção *Advanced* das definições do pincel; no cabeçalho da ferramenta ele é um popover sempre visível. | o nome interno da função de interface **e** o nome de classe interno do painel de queda, ambos na linha 48 |

## §5 — `crates/ph2d-panel-sculpt3d/tests/it/seam.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 1187 (1183-1192) | Na referência o factor de frente-de-face é propriedade da lei e existe sempre; uma **opção do pincel** (*Front Faces Only*) decide se ele corre. Por isso a varredura é por modo. | 1 nome interno na 1187; na 1188 o fragmento de condição e a bandeira interna em maiúsculas |

---

## §6 — `crates/ph2d-sculpt3d/src/brush.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 161 (158-164) | **FICA** — `tip_scale_x` é API pública; o facto (o nome público escala o eixo que corre ao longo do caminho) é nosso. | nada |
| 305 (298-319) | A nossa dureza é **a mesma lei** da etapa de dureza da referência, no mesmo ponto do pipeline; um dab reproduz a curva analítica a três decimais nas doze curvas. | 1 nome interno; troque «verbatim» por *«a mesma lei, medida pelo gate de dureza de `brush_tests.rs`»* |
| 560 (560) | `0.0` é o neutro da etapa de dureza — ver o campo. | 1 nome interno |

Mesmo ficheiro, fora da régua: **46**, **126**, **191** (1 constante interna em maiúsculas cada); **185-186** (fragmento de expressão da referência + número de linha: diga *«no modo dinâmico o ângulo autorado soma-se, escalado pela pressão, ao ângulo amostrado»*); **199** (número de linha).

## §7 — `crates/ph2d-sculpt3d/src/brush_scale.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 111 (97-120) | Na referência o passe de auto-alisamento chama o verbo *Smooth* com o **pincel inteiro do artista** e só a força substituída pelo factor; por isso dureza, curva, padrão e simetria valem também no alisamento. | 1 nome interno |

## §8 — `crates/ph2d-sculpt3d/src/brush_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 193 (191-195) | O oráculo é **a fórmula da dureza escrita à mão**, em unidades de raio (a referência multiplica e divide pelo raio dos dois lados; aqui a distância já chega normalizada). | 1 nome interno; troque «transcrição literal» por «a fórmula escrita à mão» |

Mesmo ficheiro, fora da régua: **445** (1 nome interno do factor de auto-alisamento).

## §9 — `crates/ph2d-sculpt3d/src/brush_verb.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 230 (216-244) | Num vértice de borda com **exactamente dois** vizinhos de borda a referência troca a normal pela **bissetriz** das arestas de borda; numa malha manifold todo vértice da curva de borda tem dois (medido: 12 de 12 no `open_tube3`). | 1 nome interno **e** o número de linha |
| 239 (216-244) | A referência tem um terceiro filtro que impede um vértice de atravessar a fronteira de um *face set*; nós não temos *face sets* (decisão do Enio, doc 21 §5.2), então o verbo relaxa através dela. | 1 nome interno de parâmetro **e** o número de linha |

Mesmo ficheiro, fora da régua: **156** (guarda de retorno antecipado + nome interno: diga *«sem deslocamento do dab a referência não faz nada»*); **184-185** (condição + número de linha: diga *«o culling de lado só corre com o ângulo não-negativo»*); ⛔ **APAGAR a citação entre aspas das linhas 190-192 e o número de linha** — fica só *«a ponta é deformada ao longo do traço para não deixar degraus em traços curvos, e por isso não é um disco»*; **341** (1 nome interno).

## §10 — `crates/ph2d-sculpt3d/src/brush_verb_defaults.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 192 (178-206) | Com dureza `h` toda distância normalizada abaixo de `h` vira zero, onde a curva vale um; a `0,90` a curva satura em **90,5 %** do raio, e o peso colapsa no cosseno da câmera. | 1 nome interno |
| 230 (227-244) | A dureza é uma etapa da referência que o SculptGL não tem, e os defaults por ferramenta do Blender vivem num `.blend` binário — o número não pode vir de tabela. | 1 nome interno |
| 237 (227-244) | `0` é o **neutro**: com dureza zero a etapa não corre, não é «pouca dureza». | 1 nome interno **e** o fragmento da guarda de retorno antecipado |

Mesmo ficheiro, fora da régua: **97** (2 nomes internos); **99** (expressão da referência sobre o acumulador: diga *«com o Accumulate desligado a referência lê a pose congelada do pen-down»*); **120**, **121** (1 nome interno cada).

## §11 — `crates/ph2d-sculpt3d/src/brush_verb_filter.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 34 (32-36) | A ordem do `FilterKind` é a ordem do **menu de tipos do filtro de malha** da referência, porque é a ordem que o artista já viu. | 1 nome interno (a lista de tipos) |
| 39 (39-44) | O tipo *Smooth* do filtro de malha anda a fracção `f` na direcção da média do anel; com `f` negativo afia. | 1 nome interno |
| 46 (46-60) | O tipo *Scale* é uma homotetia em torno da **origem do objeto**; as posições de partida da referência estão em coordenadas de objeto. | 1 nome interno na 46; 1 nome interno de campo na 59 |
| 62 (62) | O tipo *Inflate* desloca pela normal congelada vezes `f`. | 1 nome interno e 1 nome de campo; troque «ao pé da letra» por «a mesma lei» |
| 64, 67 (64-75) | O tipo *Sphere* puxa cada vértice para a esfera unitária na origem do objeto; a força é o **valor absoluto** do factor, logo arrastar para qualquer lado esferiza; o vértice anda **metade** do caminho, `(p̂ − p)/2 · |f|`. | 2 nomes internos (64, 67); na 68 reescreva a expressão na nossa notação, sem índice de array |
| 77 (77-91) | O tipo *Random* desloca **ao longo da normal** congelada por `f · (hash − ½)`: sorteia a magnitude, mantém a direcção. **Divergência declarada:** a função de hash da referência não está definida no clone lido, então usamos o hash desta crate, com as quatro propriedades gateadas (determinismo, faixa `[−0,5, 0,5)`, estabilidade ao longo do arrasto, descorrelação). | 1 nome interno e 1 nome de campo na 77; na 84-85 o nome interno da etapa de sorteio, o nome interno da função de hash (duas vezes) e o comando de busca que o nomeia |
| 94 (94) | O tipo *Relax* é a mesma média com a componente normal removida. | 1 nome interno |
| 96 (96-97) | O tipo *Surface Smooth* é o HC (Vollmer et al.). | 1 nome interno |
| 99, 110, 113, 115, 116 (99-129) | O tipo *Enhance Details* é o mesmo núcleo do *Smooth* com o sinal trocado e **sem tecto** (medido: 1,2e-7 a 2,4e-7 contra a lei escrita à mão, controlo `0,000e0`). **Na referência o factor do *Smooth* é restringido a `[−1, 1]` e o do *Enhance Details* não é.** Medido nas forças 1,5 / 2,0 / 3,0: o *Smooth* fica em **0,072617** e a referência alcança **0,108926 / 0,145235 / 0,217852**. | os nomes internos das linhas 99, 110, 111, 113, 115, 116; **toda a cadeia de etapas com setas (112-116)**, que sai sem substituto — o facto é só «não passa pela restrição de faixa»; a chamada de restrição com argumentos (110-111); o intervalo de linhas (112) |
| 132 (130-136) | O **sinal** é o da referência: o deslocamento é a direcção de detalhe vezes menos a força, logo arrastar para a direita realça. | 1 nome interno de variável na 132 |
| 136 (130-136) | Cada um tem entrada própria no menu de tipos do filtro de malha. | 1 nome interno (a lista de tipos) |
| 138 (138-152) | O tipo *Sharpen* é a única lei desta família com **pré-passe**: o deslocamento de um vértice é pesado pela curvatura dos vizinhos. | 1 nome interno |
| 238 (238-241) | Os rótulos são os **rótulos públicos** do menu de tipos do filtro de malha. | 1 nome interno (a lista de tipos) |
| 269, 270 (268-273) | *Inflate*, *Scale* e *Random* **não** têm restrição de faixa na referência; o deslocamento é a força em unidades de objeto, e quem calibra é a nossa escala de arrasto. | 3 nomes internos de tipo e 1 nome interno da restrição de faixa |
| 302, 304 (300-304) | O *Enhance Details* não tem tecto, e a única coisa que o separa do *Smooth* negado é a **ausência da restrição de faixa**. | 2 nomes internos; na 303 o nome interno da restrição e o intervalo de linhas |
| 380 (377-382) | O núcleo do verbo *Sharpen* é a **mesma expressão** do tipo *Enhance Details* do filtro. | 1 nome interno |

Mesmo ficheiro, fora da régua: **209**, **211** (nomes internos do array de factores de afiação e da cache do filtro: diga *«o factor de afiação, construído uma vez por gesto»*); **258** (2 nomes internos: diga *«escala pela força e depois restringe a faixa»*); **261**, **266** (nome interno da restrição de faixa; na 266 também a chamada com argumentos: diga *«faixa `[−1, 1]` na referência»*); **280** (chamada com argumentos + 2 números de linha: diga *«faixa `[0, 1]` na referência»*); **288** (chamada com argumentos + número de linha: diga *«faixa `[0, 0,5]` na referência»*).

## §12 — `crates/ph2d-sculpt3d/src/coat.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 10 (9-17) | O `1.05` é a **cabeça** que a referência dá à recorrência que enche a demão do *Layer*. | 1 nome interno |

## §13 — `crates/ph2d-sculpt3d/src/falloff.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 62 (48-70) | A leitura estática sugeria que um pincel novo nasce com uma curva editável e nenhuma das nove; **medido no Blender 5.2 a correr** (o script de oráculo em `docs/3D/ferramentas/`), o pincel de fábrica reporta o preset público *Smooth* e deposita a curva **analítica** (razão 0,835 contra 0,8348). *Um pincel nasce do ficheiro de startup.* | na 61 o nome interno do campo de preset e 2 constantes internas em maiúsculas (a curva personalizada e o seu valor); na 62 o nome interno da inicialização de pincel, a constante interna do preset e o nome interno em itálico da estrutura de curva (diga «a curva editável»). O nome da propriedade pública na 65 **fica** (API pública, verificada) |

Mesmo ficheiro, fora da régua: **73**, **86**, **89**, **99**, **104**, **129**, **131**, **136**, **142** (1 constante interna em maiúsculas cada, uma por curva) — troque cada uma pelo **rótulo público** que o menu de curva da referência mostra para essa curva.

## §14 — `crates/ph2d-sculpt3d/src/ref_profiles.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 23 (20-26) | **FICA** — `normal_radius_factor` e `hardness` são API pública. | nada |

## §15 — `crates/ph2d-sculpt3d/src/stroke.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 149 (141-152) | A referência avança a inclinação do polegar só na **passada principal, não espelhada**, de simetria; a nossa fronteira de chamada de `dab` é essa passada. | 1 nome interno |
| 439 (438-440) | O corte entre `plane` e `surface` são dois papers: lá o plano da pegada por **média de posições e normais ponderada pela queda** (o estimador da referência), aqui a projecção MLS de Alexa et al. 2003. | 1 nome interno |

Mesmo ficheiro, fora da régua: **155** (campo interno da cache: use o rótulo público do ângulo do *Multiplane Scrape*); **181**, **278**, **364** (1 nome interno cada).

## §16 — `crates/ph2d-sculpt3d/src/stroke_dab_core.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 20 (16-21) | O verbo *Smooth* da referência percorre as forças e escala os factores (em vez de as ler do pincel) — é a forma do nosso orçamento dinâmico. | 1 nome interno; «forma literal» vira «a mesma forma» |
| 254 (240-268) | A dureza entra **antes de qualquer curva** — a ordem da referência, que aplica a dureza antes de avaliar a curva de queda —, e por isso as duas curvas abaixo lêem a mesma distância; com `hardness = 0` ela devolve o argumento sem tocar num bit. | 1 nome interno na 254 e 1 nome interno da avaliação da curva na 255 |

Mesmo ficheiro, fora da régua: **399** (1 nome interno).

## §17 — `crates/ph2d-sculpt3d/src/stroke_filter.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 19 (5-33) | O filtro de malha da referência calcula as translações a partir das **posições congeladas no início do arrasto** e escreve posição + translação; a **devolução à pose do início** existe para que um arrasto seja UMA operação e não uma composição. Aqui ela é o `restore_frozen_pose`. | 1 nome interno e 1 nome de campo interno na 19 |
| 32 (5-33) | **Divergência declarada:** para *Smooth* e *Relax* a referência guarda os eventos do arrasto e **repete a cadeia inteira**, porque `smooth^n ≠ smooth(n·s)`; nós aplicamos um passo de magnitude `s` a partir do `pre`. | 1 nome interno |
| 52 (49-62) | O tipo *Inflate* desloca pela normal congelada vezes a força — a força **é** a distância, em unidades de objeto, sem raio no meio. | 1 nome interno e 1 nome de campo |
| 91 (90-105) | `restore_frozen_pose` é a devolução à pose do início do arrasto, e a metade da lei sem a qual o filtro deixa de ser *um passo a partir do `pre`*. | 1 nome interno |
| 266, 273 (263-280) | A expressão do `target_sharpen` é a **mesma expressão** do tipo *Enhance Details* da referência (a 1,2e-7 da forma do `target_smooth`, medido). O `f.abs()` corresponde a a referência usar **o valor absoluto da força, negado**, logo ela realça nos **dois** sentidos do arrasto. | 2 nomes internos (266, 273); o fragmento de expressão da 271; na 276-277 o nome interno de variável dentro da fórmula (escreva «direcção de detalhe») |
| 320 (318-334) | A esfera: `normalize(p)` é o ponto na esfera unitária, `−p` o espelho pela origem, o ponto médio dos dois é metade do caminho. | 1 nome interno; troque «verbatim» por *«a mesma lei, medida por `the_sphere_filter_pulls_halfway_to_the_unit_sphere`»* |

Mesmo ficheiro, fora da régua: **353** (nome interno da etapa de sorteio); **368** (nome interno da função de hash com prefixo de biblioteca).

## §18 — `crates/ph2d-sculpt3d/src/stroke_filter_laws_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 10, 11 (1-14) | Os oráculos das três leis são **escritos à mão**: *Scale* `t = base + base·f`; *Sphere* `midpoint(unit(p), −p)·|f|`; *Random* `normal · f · (hash − ½)`. | 2 nomes internos (10, 11); na 12 o nome interno da etapa de sorteio e o nome de campo interno |
| 83 (82-89) | A escala é sobre a origem do objeto, e o oráculo é a lei do tipo *Scale* escrita à mão. | 1 nome interno |
| 152 (151-152) | A esferização puxa para a esfera unitária, pelo oráculo da lei do tipo *Sphere* escrito à mão. | 1 nome interno |
| 399 (398-413) | O *Enhance Details* realça nos **dois** sentidos do arrasto: a referência usa o valor absoluto da força, negado, logo à entrada do tipo. | 1 nome interno e o fragmento de expressão; o identificador público do tipo na 398 pode ficar |

Mesmo ficheiro, fora da régua: **266** (chamada interna com prefixo de biblioteca); **268** (1 nome interno).

## §19 — `crates/ph2d-sculpt3d/src/stroke_filter_sharpen.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 105 (104-110) | **FICA** — `sharpen_intensify_detail_strength` é API pública. | nada |
| 202 (200-207) | Na referência o factor de afiação é construído **uma vez por gesto**, na inicialização do filtro, e toda iteração reusa o mesmo array; recomputá-lo por sub-passo faz a lei perseguir o próprio rasto e convergir para um estado alisado. | na 201 os nomes internos do array e da cache; na 202 o nome interno da inicialização |
| 297 (290-299) | O termo de intensificação **subtrai o laplaciano**: é a lei do tipo *Enhance Details* escalada pela curvatura, e é o único dos três termos que empurra o detalhe para fora. | 1 nome interno na 297; na 290-291 o nome interno de variável dentro da fórmula e o intervalo de linhas |

Mesmo ficheiro, fora da régua: **21**, **22** (nomes internos, incluindo o do recíproco seguro); **229**, **230**, **305**, **306**, **378** (1 nome interno cada); **232**, **372** (número de linha).

## §20 — `crates/ph2d-sculpt3d/src/stroke_filter_sharpen_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 14 (1-16) | Os gates defendem que o núcleo é o do tipo *Sharpen* (paridade contra a lei escrita à mão, numa malha regular); o factor de afiação é construído uma vez por gesto. | 1 nome interno na 14; na 7-8 os nomes internos do array e da cache |
| 76 (68-77) | O oráculo segue as **duas fases** da lei: um pré-passe que mede e normaliza a curvatura, e a soma sobre o anel que escreve. | 1 nome interno; troque «lido de cima a baixo» pela descrição das duas fases |
| 193 (189-205) | A soma do tipo *Sharpen* sobre o anel **não é normalizada pela contagem**: com `f[i]` baixo e vizinhos altos vale `valência × laplaciano`, e um passo de factor maior que um ultrapassa. | 1 nome interno |
| 248 (248-252) | O gate da wave: o núcleo é o do tipo *Sharpen*. | 1 nome interno |
| 350 (347-354) | A referência classifica o *Sharpen* entre os filtros que **não restauram a pose entre eventos**, então lá o número de iterações é o número de eventos entregues; o nosso resultado é facto do arrasto. | 1 nome interno |

Mesmo ficheiro, fora da régua: **83** (2 nomes internos); **288**, **289** (1 nome interno cada).

## §21 — `crates/ph2d-sculpt3d/src/stroke_filter_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 233 (224-237) | O `target_sharpen` escreve `live + (live − avg)·w`, a forma do tipo *Enhance Details*; o `target_smooth` escreve a mesma lei como `live·(1 − w) + avg·w`. | 1 nome interno |
| 387 (384-392) | O arrasto de volta devolve a pose exacta — o que a referência paga com a devolução à pose do início. | 1 nome interno |

Mesmo ficheiro, fora da régua: **669** (1 nome interno).

## §22 — `crates/ph2d-sculpt3d/src/stroke_hc.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 43 (41-45) | **FICA** — `surface_smooth_shape_preservation` é API pública. | nada |
| 76 (74-78) | **FICA** — `surface_smooth_current_vertex` é API pública. | nada |
| 191 (187-205) | A referência usa a **média crua dos vizinhos** nos dois lados; nós usamos o `ring_average` de propósito, para não desfazer a regra de borda. | 1 nome interno |

Mesmo ficheiro, fora da régua: **17** (nome interno do laço paralelo); **33** (seta para um campo interno: diga *«o deslocamento laplaciano guardado por vértice, alocado a zeros uma vez por traço»*).

## §23 — `crates/ph2d-sculpt3d/src/stroke_law_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 647 (644-653) | A referência corre o auto-alisamento no fim do **despacho por dab**, que é chamado **uma vez por cópia de simetria**. | 1 nome interno |

## §24 — `crates/ph2d-sculpt3d/src/stroke_plane.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 21 (16-23) | O estimador é a média ponderada pela queda das posições e das normais da pegada, como o da referência; difere de mínimos quadrados numa sela, e a divergência fica registada. | 1 nome interno |
| 159, 170 (145-173) | *Clay Thumb* e *Multiplane Scrape* usam **a mesma construção do plano do pincel** da referência, logo herdam a leitura congelada do pen-down com o Accumulate desligado. | 2 nomes internos |
| 314, 315 (312-318) | No modo dinâmico a referência amostra a **superfície viva**, enquanto a construção do plano do pincel lê a pose congelada do pen-down — duas perguntas diferentes. | na 314 o nome interno da amostragem e os 2 nomes internos dos arrays vivos; na 315 1 nome interno |

Mesmo ficheiro, fora da régua: **142** (linha de código comentada da referência — apague-a e deixe *«com o Accumulate desligado lê as posições e normais congeladas e sai»*); **291**, **326**, **412** (1 nome interno cada); **293** (guarda de retorno antecipado: *«sem amostra não faz nada»*); **366** (expressão + número de linha); **372**, **413**, **430** (número de linha); ⛔ **APAGAR a citação entre aspas das linhas 391-392 e o número de linha** — fica só *«no modo dinâmico o Ctrl zera o V para aparar superfícies planas sem trocar de pincel»*.

## §25 — `crates/ph2d-sculpt3d/src/stroke_surface.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 6 (1-20) | Irmão do `plane`: lá mora *que plano a pegada ajusta* (média ponderada de posições e normais, como a referência); aqui *que forma a superfície tem em torno dele* (MLS). | 1 nome interno |

## §26 — `crates/ph2d-sculpt3d/src/stroke_symmetry.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 105 (97-107) | A inclinação do polegar da referência só avança **dentro do verbo *Clay Thumb***; sem o nosso gate de verbo, um traço de *Draw* deixaria a inclinação carregada. | 1 nome interno na 105; 1 nome interno de campo na 104 |
| 157 (155-171) | O segundo passe (*Auto Smooth*) corre **depois** do verbo e **dentro** da passada de simetria, no fim do despacho por dab, chamado uma vez por cópia. | 1 nome interno na 157; na 155 o nome interno do factor (use o rótulo *Auto Smooth*) |

Mesmo ficheiro, fora da régua: **172**, **176** (1 nome interno cada).

## §27 — `crates/ph2d-sculpt3d/src/stroke_target.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 237 (234-251) | A projecção do *Clay Thumb* é a **mesma projecção bilateral** ao plano do *Flatten*; o plano passa pelo **centro do dab**, não pelo centro de área. | 1 nome interno na 237; na 242 a chamada com argumentos e os nomes internos; na 243 o nome interno da posição espelhada |
| 485 (471-499) | A referência aplica o peso nos **dois** lugares: dentro da recorrência da demão ele é a **taxa** com que o vértice enche; aqui é a **fracção** do caminho até a meta. A cadeia de factores da referência usa só a curva, sem a força; a força entra na recorrência. | 1 nome interno na 485; 1 nome de campo interno na 479; 1 nome interno da avaliação da curva na 496; o campo interno de força na 497 (diga «a força do traço») |

Mesmo ficheiro, fora da régua: **65**, **252**, **255**, **264** (1 nome interno cada); **292** (expressão com nome interno + número de linha: *«um vértice do lado de trás do plano tem factor zero»*); **464** (2 nomes internos).

## §28 — `crates/ph2d-sculpt3d/src/stroke_target_ring.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 67 (62-81) | A referência tem esta lei com outro nome — o tipo *Enhance Details* — e ela é a direcção de detalhe vezes **menos o valor absoluto** da força. | 1 nome interno na 67; na 68-69 o nome interno de variável e o número de linha |
| 104 (101-112) | O *Relax* remove a componente normal da média, e o que sobra desliza pela superfície. | 1 nome interno |

## §29 — `crates/ph2d-sculpt3d/src/verb_blob_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 64 (62-71) | No verbo *Crease/Blob* da referência o termo lateral é `(centro − posição) · força`, e a força do *Blob* é a do *Crease* com o **sinal trocado**. | 1 nome interno do verbo e 1 nome interno de variável na 64 |

Mesmo ficheiro, fora da régua: **245** (1 nome interno de variável).

## §30 — `crates/ph2d-sculpt3d/src/verb_layer_front_face_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 261 (256-271) | Com dureza `h` toda distância abaixo de `h` vai para zero, a curva satura, e todo vértice do disco interior tem o mesmo peso ⇒ a mesma altura absoluta. | 1 nome interno |

## §31 — `crates/ph2d-sculpt3d/src/verb_layer_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 303 (303) | O oráculo abaixo é **a recorrência da demão com cabeça 1,05, recortada ao tecto** (a lei, escrita à mão). | 2 nomes internos |

Mesmo ficheiro, fora da régua: **390** (chamada interna com prefixo de biblioteca); **419** (1 nome interno).

## §32 — `crates/ph2d-sculpt3d/src/verb_strip_law_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 187 (174-188) | O *Clay Strips* da referência aplica o factor de **frente-de-face** como terceiro termo da cadeia de factores. (A citação do SculptGL na 185 **fica** — MIT.) | 1 nome interno |

Mesmo ficheiro, fora da régua: **297** (expressão da referência sobre o acumulador: *«o plano congelado quando o Accumulate está desligado»*).

## §33 — `crates/ph2d-sculpt3d/src/verb_thumb_tests.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 215 (213-227) | A referência avança a inclinação só na passada principal, não espelhada, de simetria. | 1 nome interno e 1 nome de campo interno na 215 |

Mesmo ficheiro, fora da régua: **176** (1 nome interno).

## §34 — `crates/ph2d-sculpt3d/tests/it/measure_layer_comb.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 16 (2-27) | Com dureza `h` a distância abaixo de `h` vai a zero, a curva satura e o peso é constante no disco interior. | 1 nome interno |
| 108 (108-110) | O disco interior é onde a etapa de dureza satura. | 1 nome interno |

## §35 — `crates/ph2d-sculpt3d/tests/it/measure_layer_front_face.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 13 (1-27) | A etapa de dureza empurra o platô para fora: com dureza `h` toda distância abaixo de `h` vira zero, onde a curva vale um. | 1 nome interno |

Mesmo ficheiro, fora da régua: **31** (condição de código + bandeira interna: *«o verbo Layer só aplica a frente-de-face se a opção Front Faces Only estiver ligada»*); **32** (o nome da opção pública e o rótulo *Front Faces Only* **ficam**); **34** (atribuição de código da referência: *«a única escrita dessa opção fora de leitura é outra leitura»*); **76** (1 constante interna em maiúsculas).

## §36 — `crates/ph2d-sculpt3d/tests/it/measure_layer_law.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 37 (37) | A recorrência da demão do *Layer*, com cabeça 1,05 e recorte, escrita à mão. | 1 nome interno e a palavra «verbatim» (a matemática da função abaixo **fica**) |

Mesmo ficheiro, fora da régua: **187** (1 constante interna em maiúsculas).

## §37 — `crates/ph2d-sculpt3d/tests/it/measure_multiplane_scrape.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 241 (241-247) | A construção do plano do pincel da referência lê a pose congelada do pen-down com o Accumulate desligado, e este verbo herda a regra. | 1 nome interno |

## §38 — `crates/ph2d-sculpt3d/tests/it/measure_scale_filter.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 14 (1-28) | Na tabela: o tipo *Scale* da referência escala em torno da **origem do objeto**, `t = posição de partida × f`, peso **linear**. | 1 nome interno e 1 nome de campo interno |
| 38 (38-42) | A lei da referência para *Scale*: `t = posição de partida × (máscara livre × força)`, somada à pose congelada, escrita à mão. | 2 nomes internos na 38; 1 nome de campo interno na 39 |

## §39 — `crates/ph2d-sculpt3d/tests/it/measure_sharpen_filter.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 66 (66-72) | A lei da referência para *Enhance Details*: `t = direcção de detalhe × −força`, onde a direcção de detalhe é o deslocamento que um passo de *Smooth* daria sobre a pose congelada. | 2 nomes internos na 66; o nome interno de variável na 67 (duas vezes) |

Mesmo ficheiro, fora da régua: **3**, **14** (1 constante em maiúsculas cada — se for o identificador público do tipo, fica); **11** (1 constante + 2 nomes internos); **12**, **25** (1 nome interno cada); **24** (1 constante + 1 nome interno).

## §40 — `crates/ph2d-sculpt3d/tests/it/measure_sharpen_valence.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 4 (1-11) | A soma do tipo *Sharpen* é `Σ_vizinhos (p[n] − p[i])·f[n]` e não é normalizada pela contagem. | 1 nome interno (a fórmula **fica**) |

## §41 — `crates/ph2d-sculpt3d/tests/it/measure_valley.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 557 (556-561) | O terceiro termo da cadeia de factores do *Clay Strips* na referência é a frente-de-face. | 1 nome interno |

Mesmo ficheiro, fora da régua: **422** (expressão sobre o acumulador: *«congelado com o Accumulate desligado»*).

## §42 — `crates/ph2d-sculpt3d/tests/it/measure_where_the_curve_knobs_reach.rs`

| sítio (bloco) | facto a manter | a remover |
|---|---|---|
| 170 (165-177) | O `shaped_distance` corre **antes** da curva própria do canal de máscara (SculptGL, `Masking.js:66-69`, **fica**), exactamente como a referência aplica a dureza antes de avaliar a curva de queda. | 1 nome interno na 170 e 1 nome interno da avaliação da curva na 171 |

---

## Anexo A — sítios da população que são API PÚBLICA (ficam, sem edição)

`crates/ph2d-panel-sculpt3d/src/ids/sculpt3d_brush.rs:59` e `:66` · `crates/ph2d-sculpt3d/src/brush.rs:161` ·
`crates/ph2d-sculpt3d/src/ref_profiles.rs:23` · `crates/ph2d-sculpt3d/src/stroke_filter_sharpen.rs:105` ·
`crates/ph2d-sculpt3d/src/stroke_hc.rs:43` e `:76`.

## Anexo B — ficheiros que só têm a mesma dívida FORA da régua do incidente

Mesma cura das regras §0.2-§0.4. Contagem por linha; nenhum nome é reproduzido aqui.

| ficheiro | linha | o que sai |
|---|---|---|
| `crates/ph2d-app-sculpt3d/src/cursor.rs` | 30 | 1 constante interna em maiúsculas (use o rótulo público da forma de queda esférica) |
| `crates/ph2d-app-sculpt3d/src/scenes_filter.rs` | 30, 31 | 1 nome interno cada (o factor de afiação; a cache do filtro) |
| `crates/ph2d-app-sculpt3d/src/space_tests.rs` | 130, 140 | 1 constante interna em maiúsculas cada (o subtipo de propriedade de distância) |
| `crates/ph2d-panel-sculpt3d/src/ids/sculpt3d_shading.rs` | 117, 120 | 1 bandeira interna em maiúsculas cada (use os rótulos públicos *Accumulate* e *Front Faces Only*) |
| `crates/ph2d-sculpt3d/src/auto_smooth.rs` | 4 | 1 nome interno do factor (use *Auto Smooth*) |
| `crates/ph2d-sculpt3d/src/brush_pass.rs` | 86 | 1 constante interna em maiúsculas |
| `crates/ph2d-sculpt3d/src/brush_verb_predicados.rs` | 30 · 31 | 1 nome interno · 1 bandeira interna em maiúsculas |
| `crates/ph2d-sculpt3d/src/falloff_tests.rs` | 20, 63 | 1 constante interna em maiúsculas cada (use o rótulo público da curva) |
| `crates/ph2d-sculpt3d/src/ref_mode.rs` | 305 | 1 nome interno de variável |
| `crates/ph2d-sculpt3d/src/stroke_apply.rs` | 28 | 1 nome interno |
| `crates/ph2d-sculpt3d/src/stroke_ring.rs` | 91 · 92 · 138 | condição + número de linha · 2 números de linha + chamada interna · número de linha — diga *«com dois ou menos vizinhos, sem vizinho de borda sobrevivente, ou com bissetriz degenerada, a referência volta à normal do vértice»* |
| `crates/ph2d-sculpt3d/src/verb_field_tests.rs` | 369 | 2 constantes internas em maiúsculas |
| `crates/ph2d-sculpt3d/src/verb_mode_tests.rs` | 421 | 1 nome interno |
| `crates/ph2d-sculpt3d/src/verb_shape_tests.rs` | 129 | 1 bandeira interna em maiúsculas |
| `crates/ph2d-sculpt3d/src/verb_strip_tests.rs` | 326 | 1 expressão sobre o acumulador |
| `crates/ph2d-sculpt3d/tests/it/measure_draw_sharp.rs` | 8, 9 | 1 nome interno cada |
| `crates/ph2d-sculpt3d/tests/it/measure_layer_zoom_and_flank.rs` | 22 | 1 constante interna em maiúsculas |
| `crates/ph2d-sculpt3d/tests/it/measure_raycast_feedback.rs` | 14, 74 | 1 expressão de código sobre o acumulador cada — diga *«a superfície lida é a congelada do pen-down quando o Accumulate está desligado ou quando o chamador o força»* |

---

## Verificação que R correu sobre este documento

- `bash scripts/cleanroom-sweep.sh docs/3D/cleanroom/VASSOURA_blender-cloth.txt docs/3D/cleanroom/SPEC_reescrita_dos_comentarios_com_nomes_do_alvo.md` — resultado no ledger.
- Busca por palavra de cada um dos 37 identificadores internos da população, e de cada um dos
  identificadores internos achados fora da régua, neste documento — resultado no ledger.
- ⚠️ **A vassoura desta obra não cobre esta população** (zero dos 42 nomes do alvo da população
  casam uma entrada dela): o sweep verde aqui **não** prova ausência destes nomes, e é por isso que
  a segunda busca existe.
