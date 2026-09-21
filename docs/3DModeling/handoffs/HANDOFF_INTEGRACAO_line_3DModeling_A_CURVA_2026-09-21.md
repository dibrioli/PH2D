# HANDOFF — A CURVA: o assado e a malha passam a escrever o MESMO byte no ecrã

> **Linha:** `line/3DModeling` · **Data:** 2026-09-21 · ⛔ **NÃO integrado, NÃO enviado** (§0.7).
>
> **A ordem do dono, ao 4.º report do mesmo defeito** (foto de duas esferas, a da esquerda lavada e
> a da direita com contraste cheio): *«auditoria com 3 agentes com lentes diferentes. Quero
> idênticos e o estado da arte»*.

---

## §1 — O veredito em uma frase

**Havia uma QUARTA causa** por baixo das três que o [`16`](../../Render3d/16_o_que_se_ve_e_o_que_se_assa.md)
já registava, e ela é uma **função de transferência**: o bake escrevia valores **lineares** numa
ranhura cujos bytes são **códigos sRGB**. Os dois lados produziam o mesmo `v` e punham no ecrã
`v·255` contra `srgb(v)·255` — **até `73,2` de `255`** no meio-tom, e **zero** no branco puro.

⛔⛔ **E a exposição estava a pagar por ela desde que existe:** o `OLHAR_DA_FORMA` foi escolhido por
escada porque a lei saía *«visivelmente mais escura»* (`59` contra `105`) — medição **certa**,
atribuição **errada**: `srgb⁻¹(0,5) = 0,214` reproduz aquele `128 → ~55 ≈ 59` à unidade.

---

## §2 — O que mudou no PRODUTO (6 ficheiros)

| ficheiro | o quê |
|---|---|
| `ph2d-form-pbr/src/imagem.rs` | a porta `codigo::{para_luz, de_luz}` e as **duas pontas** ligadas |
| `ph2d-form-donation/src/baked_form/passe_da_forma.rs` | o gémeo em WGSL: `srgb_to_linear` à entrada, `linear_to_srgb` antes da quantização |
| `ph2d-form-donation/src/lei_da_luz.rs` | `OLHAR_DA_FORMA` **`2,10 → 1,50`**, re-derivado pela mesma escada |
| `ph2d-mesh-render/src/pipeline.rs` | a textura do albedo do visor **`Rgba8Unorm → Rgba8UnormSrgb`** |
| `ph2d-app-sculpt3d/src/scripts.rs` | o passo **(6-bis)** da cena `=11`, que ensina o 4.º report |
| `ph2d-app-sculpt3d/src/albedo.rs` | **§8** — a matéria do visor passa a ser da PEÇA e não da SELECÇÃO |

⚠️ **Zero contador partilhado, zero contrato encostado, zero ADR, zero pacote externo.** As duas
dependências novas (`ph2d-color`) são **`[dev-dependencies]`** — o produto da `ph2d-form-donation`
não a usa, e na `ph2d-app-sculpt3d` ela só serve a régua.

---

## §3 — A RÉGUA que faltava, e porque nenhuma existente servia

`os_dois_lados_leem_o_mesmo_byte_no_ecra` ([`bake_light_pbr_ecra.rs`](../../../crates/ph2d-app-sculpt3d/src/bake_light_pbr_ecra.rs))
lê os bytes **finais** dos dois lados **depois** do [`ph2d_render::Tonemap`], alimentando o passe do
**produto** duas vezes (`rebind_game_view`) — ⭐ **zero motor novo**: dar-lhe a ranhura `…Srgb` da
sprite faz o hardware descodificar à entrada e codificar à saída, que é letra por letra a cadeia
dela; dar-lhe o `Rgba16Float` da malha é a cadeia da malha.

⛔⛔⛔ **Porque o irmão não o podia ver:** ele quantiza o valor da MALHA com a regra da SPRITE
(`v·255`), logo os dois lados partilham a convenção sob suspeita e concordam **por construção**.
*Ele lia `1` código sobre a foto em que o dono via duas esferas diferentes.*

| | antes | depois |
|---|---|---|
| o irmão, em VALORES | `1` código | `1` código |
| **no ECRÃ** | até **`73`** | **`1`** |
| controlo (a malha sem luz) | — | reprova por **`130×`** / `111×` |

---

## §4 — SETE coisas que uma leitura rápida do diff entende ao contrário

1. ⛔ **Não é «a sprite estava errada, a malha certa»** — a sprite estava errada **nas duas
   direcções** (lia código como luz E escrevia luz como código), e as duas cancelavam-se em parte.
2. ⛔ **O passe da TINTA não muda e está CERTO** — ele é **RELATIVO** (multiplica códigos por uma
   razão e nunca sai do espaço de códigos, por isso tinta plana sai byte-idêntica). *Só quem sai do
   espaço de códigos tem de pagar a viagem de volta.*
3. ⛔ **A descida `2,10 → 1,50` não é uma afinação de gosto** — é a mesma escada (`diag_a_escada_do_olhar_com_ceu`,
   alvo `186,1`) re-tirada de raiz, e o candidato bate melhor que o antigo alguma vez bateu
   (**`+0,4` contra `+1,7`**). O `2,10` lê hoje `+32,6`, que é o tamanho do que a curva trazia.
4. ⛔ **A troca de formato da textura do albedo NÃO é cosmética** — ela vale `25` códigos em arte
   colorida e **`0`** numa tela branca. *O enquadramento em que o dono estava a olhar é o único em
   que ela é invisível.*
5. ⚠️ **O `lum` das sondas do céu não é um detalhe de teste** — pesos Rec.709 estão definidos sobre
   **luz**, e aplicados a códigos dão *luma*. A razão do gate passava por **`0,0071`**.
6. ⚠️ **O `fora_da_silhueta_o_byte_sai_intacto` não foi afrouxado — ele foi quem mandou** na segunda
   metade da lei. Com a codificação sozinha, um `3` saía `28`.
7. ⛔ **A sonda `diag_o_que_a_curva_valia` não afirma nada** e fica de propósito: é a tabela que
   explica a foto, código a código.

---

## §5 — CINCO premissas escritas no repo que a medição derrubou

| onde | o que dizia | porque morreu |
|---|---|---|
| `imagem.rs` §"albedo LINEAR" | *«decodificar aqui faria as duas leis discordarem»* | as duas leis **não fazem a mesma coisa** (relativa vs absoluta); a própria nota trazia a saída: *«se esta convenção estiver errada, ela está errada nas duas»* |
| `lei_da_luz.rs` | a exposição cura a escuridão medida | ela **multiplica tudo**; o que faltava era a curva |
| `pipeline.rs` | `Rgba8Unorm` *«porque a lei do bake é LINEAR»* | ⭐ e o **mesmo parágrafo previu o defeito** que a troca em falta produziria, à letra |
| `16 §4.2` | a mesma, em prosa | idem — a linha da tabela de recusas está agora **riscada com a data** |
| `ceu_da_forma_sondas.rs` | a barra era *«o vale medido, razão 2,6×»* | o vale mede-se em **luz**; em códigos ele encolhe para `1,5071` |

---

## §6 — Provas e portão

* **Mutação (CPU): 3 a sangrar + 1 NOMEADA com a medição.**
  * apagar a **codificação** ⇒ sangra (3 gates) · apagar a **descodificação** ⇒ sangra (3 gates)
  * apagar **as duas** (a reversão inteira) ⇒ sangra — ⭐ e só sangra **porque existe a PORTA**:
    o arnês lê o `codigo::*` e a mutação sai dele. *Sem a porta, reverter a lei inteira restaurava a
    identidade e os gates de no-op ficavam verdes.*
  * ⛔ **NOMEADA:** devolver o `lum` a pesar códigos deixa o gate VERDE — o que ela compra é
    **MARGEM** (`+0,5 %` → `+59 %`), não veredito. A régua que faltaria afirma a **SEPARAÇÃO**
    (produto ÷ controlo: `5,7×` em luz contra `2,27×` em códigos) e fica por escrever de propósito:
    a barra dela não tem hoje fonte independente.
* **Mutação (PLACA): 3 de 3 a sangrar**, e cada uma devolve **o defeito que a lei cura**, medido
  pela régua nova (`no ECRÃ a malha e a sprite diferem … códigos`):

  | mutação | o que a régua lê |
  |---|---|
  | apagar a **codificação** da lei | **`74`** — ⭐ o número da FOTO do dono |
  | apagar a **descodificação** da lei | `25` |
  | o espelho do visor volta a `Rgba8Unorm` | `25` |

* **Placa — o que CORREU e o que ele diz:**

  | alvo | veredito |
  |---|---|
  | `a_placa_e_a_regua_concordam_no_pixel` | **ok** — o gémeo em WGSL concorda no pixel |
  | `os_dois_lados_leem_o_mesmo_byte_no_ecra` | **`1` código** · controlo `130×` / `111×` |
  | `o_visor_e_os_bytes_da_sprite…` + `a_projeccao_da_fonte…` | **ok** (curados pela porta) |
  | `ph2d-form-donation` (placa) | **8 / 8** |
  | `ph2d-mesh-render` (placa) | **74 / 76** |
  | `ph2d-app-sculpt3d` (placa) | **90 corridos**, 3 vermelhas |

  ⛔ **As CINCO vermelhas estão atribuídas por MEDIÇÃO e não por suposição** — a árvore foi
  **ablacionada por inteiro** (`git checkout -- .` + o ficheiro novo movido para fora) e reposta por
  patch, e as mensagens são **idênticas ao bit** dos dois lados:
  * `probe_wire_continuity` ×2 (`62 %` · `57,4 % (sem empurrao: 75,1 %, para fora: 72,9 %)`) — do
    commit `b313e9d25`, de outra linha;
  * `global_retopo` ×2 (`detail=0.00: a aresta mais longa e' 21.3 %` · *«a peça saiu do campo
    SÓ-SUAVIDADE»*);
  * `the_quads_are_as_square_as_the_oracles` — **já registada como vermelha no `CLAUDE.md §5`**,
    com endereço.

  ⚠️ **A bateria de `ph2d-app-sculpt3d` foi MORTA pelo prazo aos `90`** (os que faltam são todos da
  família de retopologia, a lenta) — *não é um verde e não é um vermelho: é um não-veredito*, e está
  dito como tal.

* **`nextest-impacted`: 17 691 / 17 693**, as duas resolvidas —
  * `a_contagem_de_cada_familia_e_a_do_ficheiro` era **minha** (as memórias novas), re-contada;
  * `a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` é membro **já nomeado** da
    família de flakes de fan-out: **3/3 verde sozinha** com a carga impressa, **zero linhas** do
    diff naquela crate.
* **clippy `-D warnings`: zero** · **`cargo fmt`: limpo**.

---

## §7 — ABERTO, com dono em cada item

* ⏳ **O smoke do dono** — a 1.ª metade JÁ foi corrida e aprovada (*«Finalmente Correto»*); o que
  falta é a da **§8**: cena `=11`, escolher a imagem, **depois escolher o objecto 3D** e ver que a
  aparência NÃO muda (antes clareava, porque o visor voltava ao barro).
* ⏳ **Os ~10 gates de placa da retopologia que o prazo cortou** — nenhum toca no que esta wave
  mudou (o ficheiro deles não referencia albedo, lei do bake, exposição nem formato de textura), mas
  isso é uma leitura e não uma medição: quem tiver a placa livre corre-os com `PH2D_PRAZO` maior.
* ⏳ As **três mutações que precisam do cartão** (apagar a codificação/descodificação do WGSL;
  reverter o formato da textura) — a terceira já foi medida a sangrar (`25` códigos) no caminho.
* ⚠️ **ACHADO DE INSTRUMENTO, não curado:** o guarda de exclusão da placa (`ph2d-run.sh`)
  **recusa com código de saída `0`** — *um roteiro que confie no código de saída lê a recusa como
  aprovado*. É a família do `| head` que destrói o exit code, do outro lado.
* ⚠⚠ **AVISO AO INTEGRADOR — tecto de LOC apertado:** o `ph2d-mesh-render/src/pipeline.rs` fica
  em **`685` de `700`** (esta wave pôs lá `+29` linhas de prosa que registam a morte da premissa).
  ⛔ O tecto por-ficheiro é a única grandeza deste repo que **SOMA entre linhas sem ninguém a
  contar** (§5.0) — se outra linha da rodada lhe tocar, ele estoura na ÁRVORE COMBINADA e não aqui.
  A cura é **corte por responsabilidade**, nunca uma entrada no `FILE_OVERAGE_OK`.
* ⏳ O **material por PIXEL** (coluna B2 do [`15`](../../Render3d/15_as_metas.md)) e o contador de
  revisão da textura individual continuam abertos, como antes.

---

## §8 — E o smoke devolveu *«Finalmente Correto, contudo…»*: a matéria era da SELECÇÃO

> *«Se seleciono a imagem, o objeto 3d fica com a aparência exata do Bake. Mas se seleciono o
> objeto 3d, ele muda a aparência (fica mais brilhante).»*

⭐ **A primeira metade da frase é o veredito da §1**: com a imagem escolhida, os dois lados leem
**`1` código** de diferença no ecrã, nos dois materiais. A segunda metade é uma wave nova.

### §8.1 — O mecanismo, medido pela porta do produto

A matéria do visor era **indexada pela selecção** (`albedo::decide`): largar a sprite devolvia
`Decisao::Esquece`, que chama `clear_albedo_source()`, e o shader cai no **`CLAY`** de fábrica.
⇒ *pegar na peça para a esculpir desfazia a pré-visualização que o dono acabara de aprovar.*

Medido com a sonda nova `diag_o_que_a_seleccao_faz_a_materia` (o mesmo caminho do produto):

| o que está escolhido | média do ecrã | pior contra a sprite |
|---|---|---|
| a **imagem** | `195,99` | **`1`** |
| o **objecto 3D** (⇒ `CLAY`) | `169,66` | **`42`** |

⚠️⚠️ **E o «mais brilhante» do report é a leitura CERTA de um barro mais ESCURO.** O `CLAY` é mais
escuro (`169,66` contra `195,99`) e na composição do OpenPBR **só o lobo difuso escala com o
`base_color`** — o especular não. Uma base mais escura deixa o mesmo realce **relativamente** mais
forte, que é exactamente o que o olho chama de *brilhante*. ⇒ *procurar a causa num knob de brilho
não daria nada; a grandeza que mudou foi o ALBEDO.*

### §8.2 — A cura

**A matéria é da PEÇA, não de quem está escolhido.** Em `albedo::decide`:

* **sair da lei que a lê** (`Lighting != Pbr`) é hoje o **único** caminho para `Decisao::Esquece`;
* **largar a selecção** devolve `Decisao::Nada` — a matéria que já lá está fica;
* em `sincroniza`, uma leitura que **falha** deixa a matéria como estava (antes reescrevia).

### §8.3 — ⛔⛔ A mutação que SOBREVIVEU, e o gate que ela obrigou

Das quatro mutações, **M2 sobreviveu**: repor o `clear_albedo_source()` no braço `Err` do
`sincroniza` — **a porta das traseiras**. Ela é inalcançável a um gate escrito de dentro do
`sincroniza`, porque **o efeito dela É o dispositivo**: não há valor de retorno que a denuncie.

⇒ censo `o_barro_tem_um_caminho_de_volta_e_e_o_esquece`, em duas metades: **uma só** linha de
produto chama `clear_albedo_source`, **e** ela é a do braço `Decisao::Esquece`. Com o censo no
sítio, **M2 sangra** (*«há 2 caminhos de volta ao barro — o segundo devolve o `CLAY` pela porta das
traseiras»*).

⚠️ **E um gate teve a PREMISSA MORTA, com a morte à vista no diff:**
`largar_a_seleccao_devolve_o_barro_do_shader` **afirmava o defeito** e foi reescrito como
`largar_a_seleccao_mantem_a_materia_da_peca`, com os dois controlos ao lado. Mais
`sair_da_lei_que_le_a_materia_devolve_o_barro_uma_vez_so` (o `Esquece` dispara **uma** vez, e o
quadro seguinte é `Nada` — senão o visor limpava a matéria em todo quadro fora do PBR).

### §8.4 — Portão desta metade

* `albedo_tests` **8 de 8** · a régua do ecrã lê **`1` código** nos dois materiais, com o controlo
  sem luz a `130`/`111` (*a distância que a lei de facto percorre*).
* **Mutação 4 de 4 a sangrar** (M2 depois do censo).
* Portão acumulado: `nextest-impacted` **17 695 / 17 695** · clippy `-D warnings` **zero** ·
  `cargo fmt --check` limpo.

### §8.5 — ⏳ DECISÃO DO DONO que esta wave deixa NOMEADA

O **bake** continua a recusar sem uma sprite escolhida (*«selecione um SPRITE antes»*) ⇒ com a peça
na mão o visor mostra a matéria certa e o `Shift+B` recusa. Torná-lo pegajoso — assar na **última**
sprite escolhida — é **decisão de produto e não de lei**: transforma uma recusa numa AÇÃO que
escreve numa imagem que o artista não está a olhar. ⛔ Não foi feito.
