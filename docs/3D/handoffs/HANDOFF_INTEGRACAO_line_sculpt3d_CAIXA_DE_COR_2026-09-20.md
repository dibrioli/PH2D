# HANDOFF — `line/sculpt3d` · A COR DO PINCEL É UMA CAIXA (2026-09-20)

> **Ordem do dono**, logo depois de aprovar o smoke da pintura:
> *«Smoke OK. Agora troque os sliders de cor pelo seletor de Cor (caixa de cor).»*
>
> ⚠️ **A linha tinha FECHADO e reabriu com esta ordem.** O `main` (`395da6a55`)
> **já contém** a linha inteira de 20/09 (os 102 commits do
> [handoff da LINHA](HANDOFF_INTEGRACAO_line_sculpt3d_A_LINHA_2026-09-20.md)) —
> ⛔ *aquele documento descreve uma integração que já aconteceu e não se
> re-lê como pendente*. O que está vivo são **sete** commits: cinco das
> [manchas pretas](HANDOFF_INTEGRACAO_line_sculpt3d_MANCHAS_PRETAS_2026-09-20.md)
> e **dois** desta wave.

---

## §1 — A superfície de colisão (medida)

| grandeza | valor |
|---|---|
| commits vivos contra o `main` | **7** (5 manchas + 2 caixa) |
| ficheiros tocados pelos 7 | **25** (`+2 026` / `−128`) |
| ficheiros tocados por ESTA wave | **12** |
| `PROJECT_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` · os três registos | **INTACTOS** |
| `Cargo.lock` | **zero** linhas (nenhum pacote novo) |
| contratos congelados (§6 do roteador) | **zero** encostados |
| ADR | **zero** |
| `shells/desktop` | **zero** ficheiros |
| o `main` andou desde o merge-base | **0** commits |

⚠️ **As crates tocadas por esta wave são três, e duas delas são da família:**
`ph2d-panel-sculpt3d` (10 ficheiros) · `ph2d-app-sculpt3d` (2) · `ph2d-i18n`
(1 — a tabela `sculpt3d.rs`, onde **três** chaves saíram e **uma** entrou).
*A única superfície partilhada é essa tabela, e ela é append-only por linha.*

---

## §2 — A medição que reescreveu a wave antes da primeira linha

O §5.0 do roteador manda medir se a composição já exprime o item. Ela exprimia
**tudo**:

| peça | onde já estava |
|---|---|
| o selector de cor | **UM** na casa, flutuante (OKLCH, roda, canais, hex, paletas, conta-gotas) |
| o gesto que o abre | o `Down` genérico do `pointer_down` |
| o espelho do valor vivo → `widget_color` | o `hero`, **antes** de os painéis pintarem |
| o widget da amostra | `ph2d_editor_core::widget::ColorSwatch` |
| o molde inteiro | `ph2d-panel-model3d/src/paint_rows_swatch.rs` (19/09) |

⭐ **O braço do `pointer_down` declara-se generalizado por escrito:**

> *«GENERALIZED (was a per-id `PAINTER_COLOR_THUMB` special-case): **any panel
> that paints a `ColorSwatch` and calls `store.register_picker_swatch(id)` gets
> this for free**.»*

⇒ a wave é **composição**, com **zero lei nova**. O que se escreveu foi o
domínio (a linha, a repartição de colunas, a ponte `f32`↔`u8`) e as réguas.

### ⛔⛔ A dívida declarada no id tinha a premissa ERRADA

O doc dos três ids de canal dizia, por escrito:

> *«TRÊS pistas e não um selector de cor, e a dívida é DECLARADA: esta casa tem
> editores ricos de cor, e hospedá-los aqui é uma janela flutuante sobre o
> painel — substrato que este painel não tem e que é wave própria.»*

Medido: **falso, e falso no dia em que foi escrito**. O selector já flutuava, e
um painel entra nele por **duas linhas**.

⚠️ **Não é a §0.0** (*quem move o número que tornava algo inalcançável reconfere
a nota*): **ninguém teve de mover número nenhum** — a capacidade já estava do
outro lado quando a nota nasceu. *Uma ausência afirmada sem olhar a API é um
palpite com cara de medição* — a **terceira** vez que este repo a paga.

---

## §3 — Porque a cor NÃO podia continuar a ser uma `Row`

Uma [`Row`](../../../crates/ph2d-panel-sculpt3d/src/rows_types.rs) é **UM número
sobre uma faixa**: `min` · `max` · `step` · `decimals` · `track_of` · `scale` ·
`offset` · `curva`, com `get`/`set` a devolverem `f32`. **Uma cor são três.**

| saída | preço |
|---|---|
| campo novo na `Row` | **seis** campos inertes por linha de cor |
| — e o `populate` | registaria um `Slider` + um `NumberInput` que pintor nenhum desenha ⇒ **ids órfãos**, cuja cura é APAGAR |
| sair da tabela (**escolhido**) | perde as três listas que a travessia dá de graça |

⚠️ **O preço da saída está pago:** registo, despacho e costura foram repostos à
mão, **cada um com o gate que o prende** (§6).

---

## §4 — Onde ela é pintada, e porquê

**No topo da cauda da secção do pincel** — e a lei é a que esta cauda **já
escreveu**, por ordem do dono de 15/09 (*«que os botões de deformation fiquem na
seção details junto com os outros parâmetros do pincel»*):

> *«É o TOPO da cauda e não o bloco de knobs, e a diferença é estrutural: aquele
> bloco é percorrido a partir da tabela de `Row`. Aqui ela fica imediatamente
> abaixo do último knob do pincel, que é o mesmo sítio aos olhos de quem lê.»*

⚠️ **A ordem entre as quatro primeiras da cauda é indiferente AO OLHO, e isso é
medido:** os conjuntos de verbos são **disjuntos** — quem deposita cor não
oferece pose, nem projectar, nem contorno —, logo **nenhum pincel vê duas ao
mesmo tempo** e nenhuma empurra a outra em configuração nenhuma.

### A repartição da linha

`property_label_col_w(x, w)` + `NUMBER_INPUT_MIN_W_PX`, passados a
`slider_with_chip_label_rect`/`_chip_rect` — **os MESMOS dois números** que o
[`paint_row`](../../../crates/ph2d-panel-sculpt3d/src/paint.rs) passa ao pintor
das pistas irmãs.

⛔ *Uma segunda receita de repartição seria a terceira coluna que o painel do
modelador pagou com um report de «widgets embolados»* (`220` → `108` vivo contra
`118` travado; `720` → `608` contra `368`).

### O que a troca custou ao encaixe — MEDIDO

```
a caixa cai em  y = 684,0      (dobra do encaixe: 880)
altura de uma fileira:  25,0 px
três pistas viraram UMA  ⇒  −50,0 px na secção do pincel
```

⭐ O §94 desta linha mediu que **a secção do pincel sozinha já ocupava `657` dos
`880`**: *cada fileira aqui é paga por todos os pincéis*, e esta wave devolveu
duas ao orçamento. Sonda versionada: `diag_onde_a_caixa_cai_no_encaixe`.

---

## §5 — A ponte `f32` ↔ `u8`, e o preço declarado

**Sem gama**, e isso é a convenção **medida** desta casa e não uma escolha: as
duas cópias que o Painter já tem (`encode_rgb`, `encode_rgb3`) fazem
`clamp(0,1) × 255 + 0,5`, e o canal por-vértice entra no shader como **albedo**
(`CLAY * vcolor`), no mesmo espaço em que a amostra é pintada. *Uma conversão
com gama faria a caixa mostrar uma cor e o barro sair noutra.*

⚠️ **É a TERCEIRA cópia desta aritmética no repo, e ela é declarada:** as outras
duas vivem em `ph2d-panel-painter-layers`, uma crate que esta não conhece nem
deve conhecer (o precedente é o `vetor.rs` da `ph2d-boundary`). **O que a torna
honesta é o gate de ida-e-volta ao lado dela.**

### ⚠️⚠️ A volta NÃO é a identidade, e a consequência está medida

| grandeza | valor |
|---|---|
| cor de fábrica (o `_color` do `Paint.js:13`) | `[1,0 · 0,766 · 0,336]` |
| o ciclo `f32 → u8 → f32` devolve | `[1,0 · 0,764706 · 0,337255]` |
| erro máximo da quantização | `1/510 ≈ 0,00196` = **meio byte** num ecrã de 8 bits |
| valores por canal ao arrastar (pista, passo `0,05`) | **21** |
| valores por canal (amostra) | **256** |

⇒ *a troca AUMENTA o que o artista alcança, e o que ela custa é invisível por
construção.*

⭐ **E a cor de FÁBRICA sobrevive ao bit, não por sorte:** com o selector fechado
a amostra **semeia** e nunca escreve; com ele aberto a escrita só acontece
quando o `u8` de facto mudou. A guarda compara **bytes**, que é onde a fábrica é
um ponto fixo.

---

## §6 — ⭐⭐⭐ A TERCEIRA ESPÉCIE do censo dos ids soltos

A amostra é **pintada**, **hit-indexada** e **NÃO registada no `WidgetStore`** —
o braço do `pointer_down` que a serve **devolve antes** de o foco ser calculado,
logo `is_focusable` nunca chega a ser perguntado. O que a torna viva é o
`register_picker_swatch`, feito pelo **pintor** a cada quadro.

**Os dois censos acusaram-na, e os dois estavam errados sobre ela:**

| censo | leu-a como | a cura que ele prescreve | porque está errada |
|---|---|---|---|
| `todo_id_solto_pintado_e_registado_a_mao` | **MORTA** | registar como widget | poria um `InteractiveState` que nada lê |
| `a_catraca_dos_ids_soltos_nao_cresce` | **NOVA** | uma linha na catraca dos soltos | misturaria duas espécies numa lista cuja razão de existir é serem todas da mesma |

⇒ **categoria própria** (`SOLTOS_QUE_SAO_AMOSTRA_DE_COR`), e a exclusão nos três
gates é **DERIVADA do código**, nunca da lista.

### ⛔ A metade que impede a categoria de virar LICENÇA

`toda_amostra_declarada_e_de_facto_registada_como_amostra` fecha-a nos **dois**
sentidos:

* tudo o que está na lista é **passado a `register_picker_swatch`** por um
  pintor — *sem isto, escrever uma linha ali calaria o censo sobre um controlo
  genuinamente morto*;
* tudo o que é passado a `register_picker_swatch` **está na lista** — *sem isto,
  uma amostra nova nasceria fora de todo censo*.

mais o **piso de população** (uma extracção partida devolve zero e uma diferença
sobre o vazio é trivialmente verde).

⚠️⚠️ **É a mesma família que o `CLAUDE.md` §5.0 já regista** — *o morto e o órfão
leem-se iguais numa sonda, e as curas são opostas* —, com um terceiro membro.

### O preço de o censo a poder ler

O literal repete-se na chamada:

```rust
store.register_picker_swatch(crate::ids::SCULPT3D_COLOR_SWATCH);
```

⚠️ **De propósito.** É a forma que o censo lê, exactamente como lê
`&crate::ids::NOME[..]` no registo das fileiras. *Um `id` aqui deixaria o censo
cego, e um censo cego sobre uma espécie nova é a licença que aquela secção
existe para não ter.*

---

## §7 — Os gates

### Unidade (`paint/brush_cor_tests.rs`, 4)

| gate | o que prende |
|---|---|
| `a_cor_que_o_artista_escolhe_e_a_que_a_amostra_mostra` | a ida-e-volta `u8 → f32 → u8` é **exacta nos 256 valores** (o `+0,5` desfaz a divisão) |
| `a_cor_de_fabrica_e_um_ponto_fixo_da_guarda` | com o CONTROLO de que ela **não** é representável, senão o gate é trivial |
| `a_quantizacao_custa_meio_byte_e_compra_doze_vezes_mais_valores` | prende os dois números que o §5 cita |
| `uma_cor_fora_da_faixa_satura_nos_extremos` | `1,5` · `−0,5` · `NaN` |

### Costura (`tests/it/seam_cor.rs`, 6 + 1 sonda)

⚠️ **Ficheiro próprio, e a razão é medida:** o sweep do `seam.rs` arma o
**Crease**, que não deposita cor — com ele na mão a amostra **nem é desenhada**.
*Uma fixtura que não contém o fenómeno não afirma nada sobre ele*, e é a **sétima
vez** que esta crate o escreve.

⚠️ **E ela não pode pedir um `WidgetEvent::Click`:** o braço que a serve devolve
antes do foco, logo ela **nunca produz `Click`**. O que prova que está viva é **o
selector abrir**.

| gate | o que prende |
|---|---|
| `o_dedo_abre_o_selector_e_a_cor_escolhida_chega_ao_pincel` | **a corrente inteira**: pintar · dono dos pixels · `Down` real · `pick_colour_in_the_open_picker` · repintar · `SetUi` |
| `com_o_selector_fechado_a_amostra_semeia_e_nao_escreve` | as duas metades (a `widget_color` é semeada · a fila fica vazia) |
| `trocar_de_pincel_fecha_o_selector` | com o **MESMO** host — *um gate que semeia o que vai medir prova o que ele próprio pôs* |
| `com_o_selector_aberto_a_amostra_nao_repoe_a_cor_do_pincel` | **da mutação M5** |
| `a_cor_ja_aplicada_nao_volta_a_ser_publicada` | **da mutação M6** |
| `um_pincel_que_puxa_a_cor_do_anel_nao_mostra_a_caixa` | **da mutação M8** |

---

## §8 — A prova de mutação: **9 de 9 sangram**

| # | mutação | matada por |
|---|---|---|
| M1 | `para_u8` sem o arredondamento | a ida-e-volta |
| M2 | `para_f32` divide por `256` | a ida-e-volta |
| M3 | sem `register_picker_swatch` | o `Down` não abre o selector |
| M4 | sem `hit_index` | ninguém é dono dos pixels |
| M5 | semeia **sempre** (a escolha é apagada) | ⭐ gate novo |
| M6 | sem a guarda do *«mudou?»* | ⭐ gate novo |
| M7 | o selector órfão não fecha | a troca de pincel |
| M8 | a caixa é pintada em **todo** verbo | ⭐ gate novo |
| M9 | a lista das amostras é vazia | o fecho nos dois sentidos |

### ⚠️⚠️ As TRÊS sobreviventes nomearam três leis reais sem régua

**M5** — *a amostra repõe a cor do pincel por cima da que o selector escreveu.*
Os três gates de então ficavam verdes porque o `SetUi` do quadro do clique é
publicado **antes** de a reposição correr: **o defeito só se vê no quadro
SEGUINTE, e nenhum deles pintava dois.** O que o artista veria: *a roda move-se e
a cor volta atrás, todo quadro* — exactamente o que o doc do pintor descreve e
que nada media.

**M6** — *publica um `SetUi` por quadro com o selector aberto* ⇒ um passo de
`Ctrl+Z` por quadro sobre uma cor parada. ⚠️ **O gate tem de encenar o CICLO do
produto:** publicar **não** muda o retrato — quem o actualiza é a shell, ao
drenar. Sem esse passo, o quadro seguinte compara a cor nova com a **velha** e
publica outra vez, *que é o comportamento certo*.

**M8** — *a caixa oferecida a um verbo que puxa a cor do ANEL.* Os três gates
armam sempre o `Paint`: *uma fixtura que arma um só verbo não pode ver um
controlo oferecido a TODOS.* É a espécie que o dono reporta como **«não vejo
efeito»**.

---

## §9 — ⚠️ O ARNÊS mentiu DUAS vezes, as duas com modo de falha CONSERVADOR

1. **`bc` não existe nesta máquina.** O somatório de *«quantos testes de facto
   correram»* passava por `paste -sd+ | bc`, ficava vazio, e o arnês abortou as
   **nove** com *«o filtro correu ZERO testes»*. ⇒ `awk` puro.
2. **O `cargo fmt` juntou uma chamada numa linha** depois de a prova ter
   corrido, e a agulha da M7 passou a casar **zero** vezes.

⭐ *Uma mutação que não entra lê-se exactamente como uma que sobreviveu* — e nas
duas vezes ele **abortou alto** em vez de o fazer. E `grep -cF` conta **LINHAS**,
não ocorrências (§68 desta linha): a contagem da agulha passou a ser feita em
Python.

---

## §10 — Sete coisas que uma leitura rápida do diff entende ao contrário

1. **O selector não foi construído.** Ele é UM, é da casa, e já flutuava; esta
   wave liga-se a ele. Um segundo selector aqui seria *a segunda resposta à
   mesma pergunta, e a que envelhece*.
2. **A dívida no id não «expirou»: a premissa dela estava errada no dia em que
   foi escrita.** Ninguém moveu número nenhum.
3. **A amostra não é uma `Row` que mudou de aspecto** — ela saiu da tabela, e o
   diff em `rows_brush.rs` é uma remoção de 3 entradas, não uma edição.
4. **O literal repetido na chamada do `register_picker_swatch` e nas duas do
   `tr` não é descuido:** é o que torna as duas populações legíveis aos censos
   que apanharam, cada um, um defeito real nesta mesma jornada.
5. **A exclusão das amostras no `soltos_pintados` não é uma isenção:** ela é
   derivada do código e fechada nos dois sentidos por um gate próprio.
6. **A quantização a 8 bits não é uma regressão:** ela aumenta o que o artista
   alcança (`21 → 256` por canal) e o erro é meio byte num ecrã de 8 bits.
7. **O gate da cena não «afrouxou» ao perder a pergunta do nível:** a premissa
   morreu porque a amostra não tem `level`, e no lugar dela ficou a asserção que
   a torna verdadeira (*o pintor não consulta o `ui_level`*), lida do ficheiro.

---

## §11 — As duas premissas que a medição derrubou

1. *«hospedar um selector rico aqui é wave própria»* — **refutada** pela API.
2. *«o roteiro nomeia as pistas de cor que o painel pinta»* (o gate irmão) —
   **a premissa do NÍVEL morreu**: a amostra não é uma `Row` e não tem `level`.
   ⚠️ E o censo derivado do roteiro estava **cego a metade de uma população**: ele
   lia `tr("…")` de **um** pintor, e a caixa é pintada por outro. *Um censo sobre
   UMA das populações lê-se, num relatório, como um censo sobre todas* — a frase
   que aquele ficheiro já escrevia, cobrada a quem a escreveu.

---

## §12 — O portão

| passo | resultado |
|---|---|
| `nextest-impacted.sh` | **18 420 / 18 420** |
| censos da árvore COMBINADA | **127 / 127**, controlo do filtro `12 de 12` ✓ |
| `clippy --all-targets -D warnings` (3 crates) | **zero** |
| `cargo fmt --all --check` | limpo |
| prova de mutação | **9 de 9** sangram |
| tectos de LOC | **nenhum** encostado (`brush_cor.rs` = 232) |

---

## §13 — O que fica ABERTO

| item | mecanismo | dono |
|---|---|---|
| a cor **não viaja no `.ph2dproj`** | herdado da wave da pintura (§99); o canal é da malha e o ficheiro não o leva | linha |
| a caixa não tem **conta-gotas dedicado** no painel | o selector da casa tem um; um segundo botão aqui seria a 2.ª resposta | — |
| o **rótulo** é `Color` e o painel não diz *de que* cor | com o `Blur`/`Smear` na mão a caixa não existe, logo não há ambiguidade hoje | — |
| o caminho de **GPU** dos dois do anel | herdado de §99, custo por dab não varrido | linha |

---

## §14 — O smoke (cena `=51`)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=51 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **O passo (1) mudou** e o censo derivado do roteiro prende-o: ele manda
carregar na **caixa de cor da fileira `Color`**, escolher, e carregar **fora**
do selector para o fechar. O nome `Color` é o que o painel de facto pinta — há
gate a ligar as duas pontas.

### O binário fica PRÉ-CONSTRUÍDO (§1.5.9 item 9)

⚠️ **O item 7 corre ANTES do 9**, e a ordem importa: `rm -rf target/*/incremental`
reclamou **`6,3 GB`** e o binário do perfil `smoke` **sobrevive-lhe** — a 2.ª
passagem lê, com **zero** `Compiling`:

```
▸ linha line_sculpt3d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.35s
```

⇒ o dono não espera build nenhum.
