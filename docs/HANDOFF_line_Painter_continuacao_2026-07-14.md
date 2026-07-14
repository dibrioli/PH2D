# HANDOFF de CONTINUAÇÃO — `line/Painter` (2026-07-14)

> **Para o PRÓXIMO IMPLEMENTADOR da linha.** A linha foi **INTEGRADA na `main`** (2026-07-13,
> `--ff-only`, ADR-0107). Este documento substitui os três handoffs anteriores do sculpt, que viram
> **histórico**:
> [`HANDOFF_line_Painter_INTEGRACAO.md`](HANDOFF_line_Painter_INTEGRACAO.md) ·
> [`HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md`](HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md) (o detalhe técnico ainda VALE — leia se for mexer no kernel) ·
> [`HANDOFF_line_Painter_sculpt_2026-07-13.md`](HANDOFF_line_Painter_sculpt_2026-07-13.md).
>
> Plano vivo: [`docs/Painter/18_plano_sculpt_relevo.md`](Painter/18_plano_sculpt_relevo.md).
> Registro da integração: [`REGISTRO_integracao_jornada_2026-07-13.md`](REGISTRO_integracao_jornada_2026-07-13.md).

---

## 0. 🔴 O ITEM Nº 1 NÃO É W4. É O SMOKE.

**A linha inteira está na `main` e NUNCA foi vista rodando.**

O Enio ordenou a integração antes do smoke (opção B). O registro da jornada é explícito:

> | `line/Painter` | 🔴 **ZERO** | **a linha inteira** — material per-pixel (Roughness/Metallic/Wax) + Sculpt (8 verbos) |

Isto **não é formalidade**. Nesta linha, nesta mesma jornada, duas coisas passaram por TODOS os gates e
estavam erradas:

* **O rig de luzes shipou MORTO sob o mouse.** Pintava, registrava hit-rect, o `event.rs` encaminharia o
  Click — e o `populate` nunca deu `InteractiveState`. Nenhum teste via.
* **O card do Sculpt shipou com um bug de DESIGN** pinado por um gate **verde, bem escrito e
  mutation-proven**. O Enio derrubou em uma frase no 1º smoke: pegar o **Sharpen** (pra afiar em *outro
  lugar*) convertia o **Smooth** que ele acabara de fazer no **oposto** dele.

> **Gates provam que o código faz o que você DISSE. Nenhum gate diz que o que você disse está errado.**
> O smoke do Enio é o único oráculo dessa classe.

### O roteiro (uma tela, ~3 minutos)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

| # | O quê | O que tem que acontecer |
|---|---|---|
| 1 | Pinte 2-3 traços grossos | Cristas com marcas de pincel, **acesas** (a luz lê a inclinação) |
| 2 | Card **Impasto** → lâmpadas `1 2 3 4` | Ligar a 2 dá um *fill*. **Uma lâmpada colorida tinge a tinta só onde ela INCLINA** — tinta chata fica como você misturou |
| 3 | Card **Material** → Roughness / Metallic / **Wax** + a cor do Wax | Fosco vs brilhante **na mesma camada** (é per-pixel). O Wax é um FILTRO: a luz atravessa a tinta e volta com a cor dele |
| 4 | Pinte fosco, cruze com brilhante, **Ctrl+Z no brilhante** | A tinta fosca volta **fosca** (foi o bug do `mats` fora do snapshot — só fala em tinta-sobre-tinta) |
| 5 | Rail → **SCULP** → arraste sobre as cristas | Smooth derruba · Sharpen levanta |
| 6 | **Flatten** ATRAVESSANDO o flanco de uma crista | **O flanco continua um flanco.** Uma espátula nivelada araria um vale ali — é o teste de 1 segundo do plano fit inclinado |
| 7 | **Scrape** (só tira) · **Fill** (só põe) · gire o **Offset** | Offset negativo = a faca crava; positivo = o Fill amontoa |
| 8 | **Chisel** ao longo de uma crista, gire o **Angle** | Poupa os flancos, corta o eixo ⇒ **sulco com vinco**. Angle 0 = vira Scrape. **O Offset do Chisel só vai pra baixo** (a pista inteira corta) e o **vinco tem direção DESDE O 1º DAB** — comece o traço e olhe o começo dele |
| 9 | **Layer**: passe **10× no mesmo lugar, num traço só** | A demão continua com **uma** espessura de Depth. Nenhum outro verbo faz isso |
| 10 | **Inflate** sobre uma mancha grossa, Depth alto | **A MANCHA ENGORDA** — a borda é empurrada pra fora, vincos enchem. Depois **Layer** no mesmo lugar: ele só *levanta*, a silhueta não muda. Essa diferença é o teste. Depois **Depth negativo**: a forma **encolhe** (come as bordas), não só abaixa |
| 10b | **Inflate** numa área de tinta CHATA | Sobe igual ao Layer — **e isso é correto**, é geometria (deslocar um plano ao longo da normal é transladá-lo). A diferença mora na FORMA, não no chão |
| 11 | Depois de um traço, **troque o verbo** | O traço que já está lá **NÃO pode mudar**. (Exceção: com um **shape aberto**, antes do Apply, ele re-renderiza — é preview, não tela.) |

**Se qualquer linha falhar: PARE, e conserte antes de abrir W4.** Bug em código já integrado é o mais caro
que existe — ele já está debaixo do trabalho de outras 5 linhas.

### 0.1 — RODADA 1 DO SMOKE (2026-07-14) — 2 achados, ambos CONSERTADOS (pendente re-smoke)

O Enio rodou e derrubou dois. Nenhum dos dois tinha gate vermelho; **um deles tinha gate VERDE pinando o
bug.**

**① *"Inflate parece fazer a mesma coisa de Layer"* — fazia, AO BIT.**
O alvo era `pre + Depth·n_z`, errado duas vezes:
* **A normal ia invertida.** O offset verdadeiro sobe pela **secante** (`Depth·S`, `S = 1/n_z`): íngreme
  move **MAIS**. É assim que uma parede anda de lado e a forma **engorda**. `·n_z` movia MENOS, o que
  *arredonda a crista* — ou seja, era um **Smooth pior**. (E o passo 10 deste roteiro **descrevia o bug como
  se fosse o correto**. Um roteiro de smoke escrito pelo autor herda as ideias erradas do autor.)
* **Consertar o sinal NÃO resolveria.** Medi `n_z` sobre o relevo do **depósito real**: `p50 = 1.000` — o
  miolo de um traço é chapado. Logo `Depth·n_z = Depth/n_z = Depth`. **Nenhuma fórmula por-texel infla:**
  `h + d·S` é UM passo de Euler da PDE de offset, e um passo não move matéria **de lado** — que é a palavra
  inteira. O operador é **não-local**: dilatação/erosão por uma **BOLA** (`sculpt_offset.rs`).

  Inflate **mudou de família**: `Height` (sem buffer) → **`Memo`** (o mesmo maquinário de tiles do blur,
  outro kernel). A *engine family* deixou de coincidir com a *knob family* — o painel pergunta
  `knob_family()`. Perf: `O(ρ²)` taps/texel, **15,9 → 8,7 → 5,7 ms/move** (kill 8); as duas quedas são
  layout contíguo e quebrar a cadeia serial do `max`.

**② Chisel — duas correções.**
* **Offset agora é negativo na pista inteira** (`−MAX..=0`). Um chisel **corta**, e acima do plano ajustado
  (que é o fit *através* da tinta) não sobra o que cortar: a metade de cima era "Scrape, porém mais fraco".
* **O V tem direção desde o 1º dab.** O *warm-up* de heading era gateado em `texture.rake || shape.rake` —
  uma **enumeração dos leitores de `Dab::dir`** escrita quando os 2 slots de textura eram os únicos. O
  Chisel é o **terceiro leitor**: o dab do pen-down saía com `dir = [0,0]`, `perp = [0,0]`, e o **V colapsava
  em Scrape**. `BrushSpec::needs_heading` é o canal. O pincel de Sculpt **nasce com Rake ligado** (pedido do
  Enio), mas **desligar o Rake não pode tirar o V** — senão são duas portas pra mesma pergunta.

**Placar de mutação: 9 mutações, 9 mortas** — incluindo as duas que **restauram o bug que o Enio viu**
(`Inflate := Layer` e `Inflate := p + depth·n_z`). Duas delas **sobreviveram na 1ª tentativa** e o motivo
está escrito em `sculpt_tests/inflate.rs`: o gate do memo (a) re-implementava o produto em vez de dirigi-lo,
e depois (b) rodava num traço **invariante em x** que cabia **inteiro dentro de uma fileira de tiles** — uma
fixture realista, provando nada.

### 0.2 — RODADA 2 DO SMOKE (mesmo dia) — os DOIS voltaram vermelhos

> *"inflate não engorda e chisel nem recebeu o checkbox que eu pedi, nem tem rake"*

**① O Inflate crescia NO VAZIO.** A bola dilatava o campo de altura certo — e **a luz pesa por COBERTURA**
(`impasto_light::paint_body(cover) = cover`), então **relevo sobre cobertura zero não acende**. A forma
crescia para texels **sem tinta**, e a tela não mudava um pixel. **Meus gates mediam o buffer `heights`**,
que engordava honestamente. Terceira vez nesta linha que o oráculo modelou a implementação em vez da
aparência — e a mais cara, porque o gate era novo e eu o escrevi *depois* do relato do Enio.

E a regra **já estava escrita no meu próprio plano** (doc 18 §5): *"empurrar tinta move a matéria, e matéria
carrega cor"* — arquivada como exceção de **W4**. Eu li o Inflate como *reshape* e não como *crescimento*.
**Uma exceção documentada só protege quem reconhece que está dentro dela.**

Fix: a bola responde a **DUAS** perguntas — *que altura* e **de onde veio a matéria** (o argmax, `memo_src`)
— e o que chega num texel traz a **cobertura, o material e a COR** de onde veio. (Cobertura sem pixel
pintaria *papel em relevo*: a luz **modula** o RGBA que existe, não o inventa — os dois têm de viajar.)
`SculptMode::moves_matter()` é a porta única, e **W4 entra ali**: o modelo advectivo chegou cedo, forçado
pelo único verbo que cresce. A sessão congela os 4 planos (`Arc` ⇒ refcount, não cópia) e há **UMA** porta
de restore. Perf 5,7 → **6,85 ms/move** (kill 8).

**② O checkbox de Rake não existia onde o Enio olhou.** O `PAINTER_SHAPE_RAKE` mora dentro de
`if kind != TextureKind::None` — **só é pintado depois de você carregar uma imagem de Shape** (com o falloff
redondo padrão não há silhueta pra girar). Meu "default rake = true" da rodada 1 foi um default **num
controle invisível e inerte**. Agora há um **Rake no card do SCULPT** (só no Chisel, ligado por padrão):
ligado, o V segue o traço; desligado, a faca fica no **ângulo fixo do dab**. O gate anti-botão-morto varre o
**Chisel**, que é o card superset.

**Placar: 13 mutações, 11 mortas — e as 2 que sobreviveram eram COMENTÁRIOS MEUS ERRADOS, não gates
frouxos** (o `moves_matter` de um 2º verbo é inerte porque só a bola escreve `memo_src`; o `>=` é inerte
porque no chapado o polo da bola é máximo *estrito* — não há empate a desempatar). Os dois comentários foram
corrigidos no código: uma afirmação de mutação que você não derrubou é uma história, não um gate.

⚠️ **E eu estraguei um arquivo no meio disso:** desfiz uma mutação com `git checkout -- sculpt_offset.rs`, e
como o arquivo **já estava commitado**, o checkout não desfez a mutação — ele reverteu pro HEAD e **apagou o
argmax inteiro**. Reconstruído e re-verde. É exatamente a armadilha que já está na memória
([[feedback_mutation_undo_with_cp_never_git_checkout]]): **desfazer mutação é `cp` de um backup, por caminho
absoluto — NUNCA `git checkout`.**

### 0.3 — RODADA 3 (mesmo dia) — o Rake estava com os dois modos TROCADOS

> *"o tool identifica a direção do traço e mantém o ângulo … até o final. Isso é bom e deve permanecer
> assim — **mas esse é o modo onde Rake NÃO está checado**. Com Rake checado … o ângulo se ajusta em tempo
> real."*

Eu tinha construído o **oposto**. Meu Rake OFF usava o `dab_angle_deg` do pincel — um número que o artista
**não relaciona com o traço nenhum**. Não era o que ele pediu, e nunca foi.

**Os dois modos leem o traço. A pergunta é QUANDO:**
* **ON** (default) — cada dab usa o **próprio** heading ⇒ o ângulo se ajusta em **tempo real**, o sulco curva
  com a mão.
* **OFF** — a faca **entra no ângulo em que o traço começou** e o **segura até o fim** (`locked_dir`).

Antes de escrever, **medi**: o motor de traço já entrega heading vivo (num quarto de círculo os dabs varrem
de +93° a +165°). Ou seja, o mecanismo estava lá — faltava o **modelo**.

Corolário que quase passou: o `needs_heading` vale para os **DOIS** modos. Gateá-lo em `rake` faria a faca
não-raked travar em `[0,0]` (o dab do pen-down, antes de qualquer viagem) e cortar um scrape chato o traço
inteiro — exatamente o bug que a flag existe pra matar, vestindo o outro estado do checkbox.

**Gates:** o da curva (os dois sulcos TÊM de divergir num quarto de círculo) + o irmão de presença — **num
traço RETO o Rake não muda nada**, e é ele que dá sentido ao primeiro (seguir o heading e travar o primeiro
heading são a **mesma instrução** quando ele não vira). O reto não é byte-idêntico: o EMA do heading
re-normaliza a cada passo, então o vetor unitário oscila no último bit — **4,8e-7 loads**, um milésimo do
`RELIEF_EPS` que o próprio depósito usa pra decidir que um texel não mudou. Exigir `to_bits()` aqui seria
exigir que um EMA fosse idempotente.

**3 mutações, 2 mortas.** A que sobreviveu é uma **propriedade**, não um gate frouxo: com Rake ON a trava
**nunca é populada** (o `stamp_dabs_sculpt` guarda em `!rake`), então `locked_dir.unwrap_or(dir)` já É `dir`.
Duas coisas independentes seguram o caminho vivo. A que de fato o quebra é `if rake { return [1.0, 0.0] }` —
e derruba o `the_chisel_carves_a_crease`. Escrito no doc do gate.

### 0.4 — RODADA 4 (mesmo dia) — o teto e o TAMANHO do pincel

> *"em 2 pinceladas bate no teto e achata tudo … esse limite é mesmo necessário? … a altura do relevo não
> está vinculada ao tamanho do pincel, mas é fixa … pincéis grandes ficam parecendo apenas tinta."*

Dois problemas, e o 2º é a raiz do 1º.

**① A altura do depósito era ABSOLUTA, sem o raio.** Um pincel de 6 px e um de 60 px picavam no MESMO pico
(em cargas). Um monte dessa altura sobre 60 px tem `n_z ≈ 1` — a luz o desenha **chato**. Fix: `derive_height`
escala por `radius / 10` (a referência = pincel default), então a **razão de aspecto** (altura÷largura) fica
constante e o falloff lê em toda escala. Um dab na referência é exatamente o Depth (arte com o pincel default
intacta). Isso EXPLICA o problema do teto: com a altura escalada, um pincel grande estourava o joelho antigo
(2 cargas) num único dab.

**② O teto não era um limite — era um clamp que ACHATAVA (de novo, do outro lado).** A compressão que eu
tinha posto (joelho 2, assíntota 8) ainda topava em ~8 cargas e ainda ficava *progressivamente mais difícil
de subir*. O Enio: **isso não é desejável — sobe na proporção real do peso.** Fix: joelho **2→24**, assíntota
**8→128**. A faixa que o artista alcança agora é **LINEAR** (cada pincelada soma o peso inteiro; 6 pinceladas
= 6 cargas, byte por byte); a compressão sobrou só como **guarda far-field** (existe para a luz nunca receber
uma inclinação infinita, nunca porque o artista a alcança).

**Gates novos:** `the_relief_height_scales_with_the_brush_size` (pico ∝ raio) · `a_big_brush_still_shows_
relief_it_is_not_just_flat_paint` (a APARÊNCIA — a luz move o interior de um domo grande tanto quanto de um
pequeno) · `stacking_is_linear_and_weight_proportional` (6 pinceladas = 6 cargas, sem compressão).

**2 armadilhas no oráculo de aparência, ambas minhas** (a lição de sempre): (a) medi min-max da imagem
iluminada e peguei a borda **papel→tinta** (branco→vermelho), não o relevo — todo disco parecia domo; (b)
usei `hardness 1.0`, que faz uma **mesa** (topo chato + penhasco), e amostrei a 0.95×raio, **perdendo o
penhasco** no dab grande → 0 de sombreamento num relevo de 3,6 cargas. Só sondando o PERFIL vi que era mesa,
não domo. O oráculo certo: dab **impasto on vs off** (a cobertura cancela, sobra a luz), `hardness 0` (domo
real), varrendo o dab todo.

**Gate churn:** 11 gates existentes quebraram (a altura mudou). Consertados por princípio, não afrouxados:
`impasto_canvas` e os unit tests foram **fixados na referência** (escala 1) pra testarem o perfil, não a
escala; os gates de luz a raio 40 tiveram a profundidade compensada (×0.25) pra restaurar o relevo calibrado
mantendo a pegada; os 2 gates de teto foram reescritos pra faixa linear. **2 mutações, 2 mortas** (altura
ignora o raio; joelho volta pra 2).

---

## 1. Estado da linha (medido em 2026-07-14, não lembrado)

| | |
|---|---|
| **Branch** | `line/Painter` **resetada em `4d203d48`** = a `main` integrada. Nada meu ficou de fora. |
| **Gates na árvore COMBINADA** | ✅ `cargo test --workspace` → **6984 passed / 0 failed** |
| **Meus gates do sculpt, pós-merge** | ✅ 32 (lib) + 9 (seam) = **41, todos verdes** |
| **O integrador mexeu no meu código?** | **Não** — `ph2d-tool-painter` / `-painter-brush` / `-panel-painter-layers` idênticos ao que fechei |

### O que MUDOU debaixo de mim na integração (leia antes de tocar em save/deps)

* **`PROJECT_SCHEMA` foi CONTADO, não escolhido.** 4 linhas bumparam, 6 quebras ⇒ **13**. Os meus dois
  viraram **v11** (`mats` novo) e **v12** (`mats` mudou de FORMA, 4 B → 7 B, pela cor do Wax). O pin do
  esquema agora é uma **TRIPLA**: `(PROJECT_SCHEMA, FLIP_SCHEMA_VERSION, VEC_SCENE_SCHEMA_VERSION) ==
  (13, 5, 8)`. **Se você bumpar, CONTE** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
* **`deny.toml`:** o ignore órfão do `RUSTSEC-2023-0089` foi removido (eu reportei, o integrador
  consertou). E ele achou **`spin` 0.10.0/0.9.8 YANKED** que **nenhuma das 6 linhas reportou** — ninguém
  rodava `cargo deny`. **Rode.**
* **`ph2d-ui-testkit`:** o `MockPanelHost::click_at` (meu) e uma dep da linha `motion-value` colidiram no
  `Cargo.toml` — **merge limpo, TOML quebrado** (o comentário colou na linha do `ph2d-text`, sem newline;
  deps duplicadas). Consertado. É o caso-escola do
  [[feedback_clean_text_merge_can_be_semantically_broken]].

---

## 2. W4 — a família ADVECTIVA (Grab · Pinch · Nudge · Rotate · Thumb)

### O achado que faz W4 valer a pena: **não é construir um motor. É passar 3 planos a um que já roda.**

**Levantado no código hoje, não no plano** (`crates/ph2d-tool-painter/src/tool/paint/warp/`):

* O Deform **já é um inverse-warp**: pra cada texel de destino, ele amostra a fonte em `dst − d` com
  `bilinear_clamped` (`warp/apply.rs`, `warp/reconstruct.rs`). O campo de deslocamento `d` (`warp/field.rs`)
  já tem Push / Twist / Pinch — *"only the displacement field D changes"*.
* **Ele carrega SÓ RGBA.** Grep confirmou: `warp/` não menciona `heights`, `covers` nem `mats` (as
  ocorrências de "covers" são a palavra em inglês, em comentário).

Então W4 é: **um sampler, quatro canais.** Os planos do relevo pegam carona no **MESMO `d`** que já foi
computado. `bilinear_clamped` hoje é `&[u8] → [u8;4]`; precisa de irmãos para `f32` (`heights`), `u8`
(`covers`) e `[u8;7]` (`mats`) — ou de uma versão genérica.

**Isso destrava CINCO pincéis de uma vez**, e unifica "Liquify" e "sculpt-warp" numa engine só.

### A decisão de superfície que W4 tem que tomar (e a recomendação)

Os warps de relevo são **sub-modos do Sculpt** ou um **toggle "afeta o relevo" no Deform**?

> **Recomendação: o toggle no Deform. Um motor, um lugar.**
> Um Grab que existe em dois cantos da UI é duas ferramentas que divergem. E repare que isto é o mesmo
> raciocínio que fez Clay/Clay Strips/Draw Sharp **não** virarem chips (W3): quando a ferramenta já está
> alcançável, um segundo caminho até ela é um bug de design com fantasia de feature.

### As 3 armadilhas que W4 vai encontrar (todas já pagas nesta linha)

1. **⚠️ A EXCEÇÃO do §5 do plano.** O sculpt escreve `h` e SÓ `h` — mas **empurrar tinta MOVE MATÉRIA, e
   matéria carrega COR**. Grab/Pinch/Nudge têm que advectar **`h` + `covers` + `mats` + RGBA JUNTOS**. É o
   que separa "sculpt numa imagem" de "mexer em tinta", e é decisão de **modelo**, não de código.
2. **⚠️ Plano novo ⇒ snapshot do undo NO MESMO COMMIT.** O `mats` já ficou de fora uma vez, e o buraco
   **se escondia na tela vazia**. Se W4 tocar um plano, o `ModelSnapshot` entra junto.
3. **⚠️ `DEPTH_UNIT_PX` na entrada de toda grandeza geométrica.** `x` é texel, `h` é carga de tinta. Um
   ângulo cru já inclinou um plano **16× demais** nesta linha ([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]).

---

## 3. Depois de W4

* **W5 — Conserve** (a *bow wave*, §6 do plano): pra onde vai a tinta raspada. O Blender **deleta** matéria;
  tinta não faz isso. O kernel **já computa** o volume deslocado e o joga fora explicitamente
  (`sculpt_displaced_volume`, **gateado**) ⇒ o Conserve é um **flag**, não uma reescrita.
  **Critério de desempate escrito no plano:** *se o Enio olhar o Scrape e disser "cadê a tinta que eu
  tirei", W5 sobe na fila.*
* **Filtros de camada inteira** (Smooth/Sharpen/Inflate/Relax aplicados à camada, sem traço).
* **A TINTA EMPURRADA (o Push)** — o Enio deixou no fim da fila. **Repare: o Conserve do Scrape e o Push
  são o MESMO problema pelos dois lados.**
* Herdados, dormentes: [`HANDOFF_per_layer_color_perf_artifacts.md`](HANDOFF_per_layer_color_perf_artifacts.md)
  (Bug #11 — listras retangulares + lentidão em camadas-como-brush).

---

## 4. Dois gates que passam por NÃO OLHAR (pré-existentes — herde a dívida sabendo dela)

Um gate que passa por não olhar é **pior** que gate nenhum: ele dá sensação de cobertura.

* **`architecture_panel_wiring_parity`** lê o conjunto registrado **só de `src/populate.rs`** e nunca abre
  os irmãos `populate_*.rs`. Os dois lados saem vazios ⇒ **verde independentemente de qualquer coisa**.
  Vale igual pro `populate_deform`. A cobertura REAL é `tests/seam_sculpt.rs`, que **clica de verdade**.
* **`node_id_collisions`** é lista **mantida à mão**. Não tem os ids do Sculpt — nem **nenhum**
  `PAINTER_DEFORM_*`, `PAINTER_IMPASTO_*`, `PAINTER_MASK_*` ou `PAINTER_SEL_*`.

---

## 5. O que esta linha aprendeu (leia antes de escrever a primeira linha de W4)

| Lição | O preço que ela cobrou |
|---|---|
| **Um widget não está pronto quando PINTA — está pronto quando um TESTE CLICA nele** | O rig de luzes shipou morto. E a de 2ª ordem: registrar como `Checkbox` emite `Toggled`, que o `event.rs` **não encaminha** ⇒ registrado e **ainda morto**. Daí nasceu o `MockPanelHost::click_at`. |
| **Um gate VERDE pode pinar um bug de DESIGN** | O card do sculpt reescrevia o traço passado. Gate verde, mutation-proven, e **errado**. Só o smoke enxerga. |
| **Affordance herdada por analogia** | Tinta é **substância** (tem propriedades que você segue afinando); sculpt é **operação** (já aconteceu; se desfaz, não se re-disca). Copiar o "Adjust Last Stroke" foi herdar sem re-derivar. |
| **Geometria sobre eixos de unidades diferentes** | `tan(36°)` cru inclinou o plano em 0,73 load/texel — **4× o teto do campo**. Acertei no Inflate porque fui procurar a normal da luz; errei no Chisel porque não fui. |
| **A mutação que não sangra tem TRÊS causas** | A terceira: **o gate certo não existe ainda**. Explicar *por que* ela é inofensiva ALI nomeia o caminho onde ela não é. |
| **O oráculo modela a APARÊNCIA** | O fit horizontal é **invisível ao longo do traço** (a média móvel dos planos reconstrói a encosta por acidente). Meu 1º gate de produto media na direção errada e ficou **verde sob a mutação**. |
| **`invalidate_composite()` no caminho quente** | 148 ms/move — 37× o kill — contra baseline 0,0. O aviso estava em letras garrafais no `impasto::sync_relief_flags` e eu andei direto nele. |
| **Crase em mensagem de commit é substituição de comando** | O fish **executou** `cargo deny check` e colou a saída no meio do texto. Eu tinha a memória e caí mesmo assim. **`git commit -F <arquivo>`, sempre — e RELEIA o log.** |
