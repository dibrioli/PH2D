# 01 — O ASSADO É IDÊNTICO AO QUE SE VÊ: cinco reports, cinco causas, e a receita de as re-medir

> **O dono, 2026-09-20/21 — cinco vezes, cada uma DEPOIS de uma correcção enviada:**
> 1. *«o bake não é idêntico ao que se vê em 3d»* (foto)
> 2. *«Para mim nada mudou. A malha 3d parece ter mais luz indireta que a imagem do Bake.»* (foto)
> 3. *«nada ainda»* (foto) — e depois *«siga»*
> 4. (duas esferas: a assada **lavada**, a malha com contraste cheio)
>    **«auditoria com 3 agentes com lentes diferentes. Quero idênticos e o estado da arte»**
> 5. *«**Finalmente Correto**, contudo algo acontece: Se seleciono a imagem, o objeto 3d fica com a
>    aparência exata do Bake. Mas se seleciono o objeto 3d, ele muda a aparência (fica mais
>    brilhante).»*

⚠️⚠️ **Cinco reports com a mesma frase não são cinco vezes o mesmo defeito.** Eram **cinco causas
empilhadas**, cada uma escondida pela anterior — e cada correcção enviada só tornava a seguinte
visível. *Nenhuma das cinco se teria achado a recomeçar; as cinco acharam-se a medir.*

⚠️ **Esta página é a PORTA do assunto.** O mecanismo das causas **1 a 4** está, com as tabelas
completas, em [`../Render3d/16_o_que_se_ve_e_o_que_se_assa.md`](../Render3d/16_o_que_se_ve_e_o_que_se_assa.md)
— ⛔ **não o repita aqui**, que é como duas páginas sobre o mesmo número passam a discordar. O que
só existe **nesta** página é a **causa 5** (§2), as **duas leis que ficam** (§3) e a **RECEITA** de
medir quando o report voltar (§4).

---

## §0 — O veredito de hoje, com os números

| régua | o que ela mede | leitura |
|---|---|---|
| `os_dois_lados_leem_o_mesmo_byte_no_ecra` | o **byte no ecrã** dos dois lados, depois do `Tonemap` | **`1` código** nos dois materiais |
| ⭐ o **CONTROLO** da mesma régua | a malha **sem luz** contra a sprite assada | reprova por **`130×`** / `111×` |
| `diag_o_que_a_seleccao_faz_a_materia` | a média do ecrã por matéria no visor | sprite `195,99` (pior **`1`**) · `CLAY` `169,66` (pior **`42`**) |

⭐ O controlo é metade do valor: sem ele, *«os dois lados leem o mesmo byte»* seria compatível com
uma lei que não faz nada. Ele diz **a distância que a lei de facto percorre**.

---

## §1 — As CINCO respostas que tinham de coincidir

Há cinco respostas independentes a *«de que cor sai este pixel?»*, e as cinco tinham de dar o mesmo:

| # | a pergunta | o que estava errado | quanto valia | onde está documentada |
|---|---|---|---|---|
| 1 | **com que LUZ?** (o modo da vista) | o app abria num **matcap** (a luz do OLHO) | `1 615×` o resíduo do `Pbr` | [`16` §2](../Render3d/16_o_que_se_ve_e_o_que_se_assa.md) |
| 2 | **com que LEI?** (o motor da acendida) | o bake corria a lei da **TINTA** do Painter | `28×` | [`16` §3](../Render3d/16_o_que_se_ve_e_o_que_se_assa.md) |
| 3 | **de que MATÉRIA?** (o albedo) | o visor pintava um **barro cravado no shader** | **`61×`** tudo o resto somado | [`16` §4](../Render3d/16_o_que_se_ve_e_o_que_se_assa.md) |
| 4 | **em que ESPAÇO?** (a função de transferência) | o bake escrevia **radiância crua** numa ranhura de **códigos sRGB** | **`+73` códigos**, zero no branco | [`16` §4-ter](../Render3d/16_o_que_se_ve_e_o_que_se_assa.md) |
| 5 | **de QUEM é a matéria?** | ela era da **SELECÇÃO** e tinha de ser da **PEÇA** | **`42` códigos** (`195,99 → 169,66`) | **§2 desta página** |

⭐⭐ **A ordem não podia ser outra, e isso não é narrativa — é estrutura:** com o visor num matcap
(1) nenhuma medição de lei (2) afirma nada; com duas leis diferentes a matéria (3) é ruído ao lado
delas; a curva (4) só se torna a maior de todas depois de as outras três fecharem; e a (5) só é
**produzível como gesto** depois de a (3) existir — antes dela o visor não tinha matéria nenhuma
para perder.

⛔⛔⛔ **E a (4) estava invisível a TODA régua desta casa por uma razão estrutural:** as réguas
comparavam os dois lados **em valores**, e ali eles concordam. A divergência nasce *depois*, quando
cada lado vira **byte** — e nenhuma régua olhava para lá.

---

## §2 — Causa 5: a matéria era da SELECÇÃO, e passou a ser da PEÇA

### §2.1 — O mecanismo, medido pela porta do produto

A matéria do visor era indexada pela **selecção** (`albedo::decide`): largar a sprite devolvia
`Decisao::Esquece`, que chama `clear_albedo_source()`, e o shader cai no **`CLAY`** de fábrica.

⇒ *pegar na peça para a esculpir desfazia a pré-visualização que o dono acabara de aprovar.*

| o que está escolhido | a matéria no visor | média do ecrã | pior contra a sprite |
|---|---|---|---|
| a **imagem** | os pixels da sprite | `195,99` | **`1`** |
| o **objecto 3D** | ⛔ o `CLAY` do shader | `169,66` | **`42`** |

### §2.2 — ⚠️⚠️ *«Mais brilhante»* é a leitura CERTA de um barro mais ESCURO

O `CLAY` é **mais escuro** (`169,66` contra `195,99`), e na composição do **OpenPBR** só o lobo
**difuso** escala com o `base_color` — o **especular não**. Uma base mais escura deixa o mesmo
realce **relativamente** mais forte, que é exactamente o que o olho chama de *brilhante*.

⇒ ⛔ **procurar a causa num knob de brilho não daria nada: a grandeza que mudou foi o ALBEDO.**
*Um report de «mais brilhante» mede-se primeiro no albedo, e só depois numa lei de brilho.*

### §2.3 — A cura

**A matéria é da PEÇA, não de quem está escolhido.** Em [`albedo::decide`](../../crates/ph2d-app-sculpt3d/src/albedo.rs):

```rust
// ⛔ Fora da lei que lê a matéria, esquecer é honesto — e UMA VEZ SÓ.
if lighting != ph2d_mesh_render::Lighting::Pbr {
    return if memo.take().is_some() { Decisao::Esquece } else { Decisao::Nada };
}
// ⭐ Largar a selecção NÃO larga a matéria.
let Some(bits) = selected else { return Decisao::Nada; };
```

e em `sincroniza`, uma leitura que **falha** deixa a matéria onde estava (antes reescrevia):

```rust
if let Ok((px, size)) = materia_para(forms, bits, &mut || ler_fonte(sim, renderer)) {
    scene.renderer.set_albedo_source(&gpu.device, &gpu.queue, &px, size);
}
```

⚠️ A segunda metade é load-bearing e **não** é defensiva: escolher o objecto 3D é uma selecção que
**não tem pixels**, logo ela cai exactamente ali — limpar era devolver o `CLAY` pela porta das
traseiras, com o mesmo sintoma e sem passar pelo `Esquece`.

### §2.4 — ⛔⛔ A mutação que SOBREVIVEU, e o censo que ela obrigou

Das quatro mutações desta metade, **M2 sobreviveu**: repor o `clear_albedo_source()` no braço `Err`
do `sincroniza` — *a porta das traseiras*.

⚠️ **Ela é inalcançável a um gate escrito de dentro da `sincroniza`, porque o EFEITO DELA É O
DISPOSITIVO:** não existe valor de retorno que a denuncie.

⇒ o censo `o_barro_tem_um_caminho_de_volta_e_e_o_esquece`, em duas metades: **uma só** linha de
produto chama `clear_albedo_source`, **e** ela é a do braço `Decisao::Esquece`. Com o censo no
sítio, **M2 sangra** com o endereço.

⭐ **A lei geral, que vale para toda esta casa:** *uma mutação cuja consequência É o device gateia-se
por **censo de chamadores**, nunca por asserção sobre um valor.*

### §2.5 — Os gates desta metade

| gate | o que afirma |
|---|---|
| `largar_a_seleccao_mantem_a_materia_da_peca` | largar a sprite devolve `Nada` (dois controlos ao lado) |
| `sair_da_lei_que_le_a_materia_devolve_o_barro_uma_vez_so` | o `Esquece` dispara **uma** vez — senão o visor limpava a matéria em todo quadro fora do PBR |
| `o_barro_tem_um_caminho_de_volta_e_e_o_esquece` | o censo do §2.4 |

⚠️ **E um gate teve a PREMISSA MORTA, com a morte à vista no diff:**
`largar_a_seleccao_devolve_o_barro_do_shader` **afirmava o defeito** e foi reescrito. *Um gate que
descreve o comportamento antigo passa a defendê-lo.*

---

## §3 — As duas leis que ficam escritas

### ⭐⭐ LEI A — só quem SAI do espaço de códigos paga a viagem de volta

A convenção de uma ranhura `Individual` é **código sRGB** (`Rgba8UnormSrgb`, descodificado pelo
hardware na amostragem). Há **três** escritores, e só um estava errado:

| escritor | o que faz | está certo? |
|---|---|---|
| a imagem importada | PNG em sRGB, copiado | ✅ |
| a tinta do Painter | **multiplica códigos por uma razão** — RELATIVA, nunca sai do espaço | ✅ |
| o **bake da forma** | devolve **radiância** e escrevia-a crua — ABSOLUTA | ⛔ |

⇒ a porta [`ph2d_form_pbr::imagem::codigo`](../../crates/ph2d-form-pbr/src/imagem.rs)
(`para_luz` / `de_luz`), com as **duas pontas** ligadas, e o gémeo em WGSL no
[`passe_da_forma.rs`](../../crates/ph2d-form-donation/src/baked_form/passe_da_forma.rs).

⛔ **A ORDEM é load-bearing:** `floor(linear_to_srgb(c) * 255.0 + 0.5) / 255.0` — **codificar ANTES
de quantizar**. Ao contrário, a quantização acontece no espaço errado e o passe perde o escuro.

⚠️ **E uma EXPOSIÇÃO estava a pagar a conta da curva:** o `OLHAR_DA_FORMA` fora escolhido por uma
escada contra o alvo da tinta porque a lei nova saía *«visivelmente mais escura»*. As duas curas não
são substitutas — a exposição **multiplica tudo** (levanta o meio, **queima** o alto, não salva o
escuro) e a curva **levanta o escuro preservando o alto**. Com a curva no sítio a escada foi
re-tirada de raiz e o número caiu de **`2,10` para `1,50`**, batendo melhor do que o antigo alguma
vez bateu.

### ⭐⭐ LEI B — a matéria é da PEÇA, e o estado de pré-visualização nunca é da selecção

É a causa 5 generalizada: *um estado que descreve **o que se está a ver** não pode ser indexado por
**o que está escolhido***, porque o artista larga a selecção precisamente para trabalhar na peça.

---

## §4 — A RECEITA: o que correr quando o report voltar

⛔ **Passo zero: medir, nunca recomeçar.** A partição do §1 diz em qual das cinco perguntas procurar,
e cada régua abaixo responde a uma delas sozinha.

**1. O byte no ecrã** (a régua que decide — as outras concordam por construção):

```
env PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib \
  bake_light_pbr::ecra::os_dois_lados_leem_o_mesmo_byte_no_ecra -- --ignored --nocapture
```

**2. Quanto vale a CURVA hoje** (sonda: imprime uma tabela, não afirma nada):

```
env PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib \
  bake_light_pbr::ecra::diag_o_que_a_curva_valia -- --ignored --nocapture
```

**3. O que a SELECÇÃO faz à matéria:**

```
env PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib \
  bake_light_pbr::ecra::diag_o_que_a_seleccao_faz_a_materia -- --ignored --nocapture
```

**4. A decisão, pura** (sem device, corre sempre):

```
bash scripts/cargo-test-narrow.sh ph2d-app-sculpt3d albedo
```

**5. O smoke do dono** — a cena `=11`, com o passo **(6-bis)** (a curva) e o **(6-ter)** (a selecção):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && \
  env PH2D_SCULPT3D_SMOKE=11 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **Os três primeiros são `#[ignore]`** (precisam de adapter, ou são sondas) ⇒ **o CI nunca os
corre**. Quem toca nesta família corre-os à mão, e com `PH2D_GPU=1` (exclusão: uma linha de cada vez).

---

## §5 — Os ficheiros do produto

| ficheiro | o quê |
|---|---|
| [`ph2d-form-pbr/src/imagem.rs`](../../crates/ph2d-form-pbr/src/imagem.rs) | a porta `codigo::{para_luz, de_luz}` e as **duas pontas** ligadas |
| [`ph2d-form-donation/src/baked_form/passe_da_forma.rs`](../../crates/ph2d-form-donation/src/baked_form/passe_da_forma.rs) | o gémeo em WGSL: descodificar à entrada, codificar **antes** da quantização |
| [`ph2d-form-donation/src/lei_da_luz.rs`](../../crates/ph2d-form-donation/src/lei_da_luz.rs) | `OLHAR_DA_FORMA` **`2,10 → 1,50`**, re-derivado pela mesma escada |
| [`ph2d-mesh-render/src/pipeline.rs`](../../crates/ph2d-mesh-render/src/pipeline.rs) | a textura do albedo do visor **`Rgba8Unorm → Rgba8UnormSrgb`** |
| [`ph2d-app-sculpt3d/src/albedo.rs`](../../crates/ph2d-app-sculpt3d/src/albedo.rs) | a matéria do visor é da **PEÇA** (§2) |
| [`ph2d-app-sculpt3d/src/scripts.rs`](../../crates/ph2d-app-sculpt3d/src/scripts.rs) | os passos **(6-bis)** e **(6-ter)** da cena `=11` |
| [`ph2d-app-sculpt3d/src/bake_light_pbr_ecra.rs`](../../crates/ph2d-app-sculpt3d/src/bake_light_pbr_ecra.rs) | **a régua do ecrã** e as duas sondas (não é produto) |

⚠️ **Zero contador partilhado, zero contrato encostado, zero ADR, zero pacote externo.**

---

## §6 — O que fica ABERTO

* ⏳ **DECISÃO DO DONO — o bake pegajoso.** Ele continua a recusar sem uma sprite escolhida
  (*«selecione um SPRITE antes»*) ⇒ com a peça na mão o visor mostra a matéria certa e o `Shift+B`
  recusa. Assar na **última** sprite escolhida é **produto e não lei**: transforma uma recusa numa
  **acção que escreve numa imagem que o artista não está a olhar**. ⛔ Não foi feito.
* ⏳ **Os outros três abertos deste assunto são os do [`16` §6](../Render3d/16_o_que_se_ve_e_o_que_se_assa.md)**,
  e é lá que eles se leem com o mecanismo: o **contador de revisão** da textura individual · o
  **material por PIXEL** (a coluna B2 das metas) · o `Lighting::Flat`, que é VISTA e não entra no bake.

---

## ⛔ Recusas MEDIDAS desta metade

> As das causas 1–4 estão no fim do [`16`](../Render3d/16_o_que_se_ve_e_o_que_se_assa.md).

| o que foi tentado | porque NÃO ficou |
|---|---|
| procurar o *«mais brilhante»* num knob de brilho | a grandeza que mudou foi o **albedo** — o especular do OpenPBR não escala com o `base_color` (§2.2) |
| gatear a porta das traseiras por asserção dentro da `sincroniza` | o efeito dela **é** o device: não há valor de retorno que a denuncie ⇒ **censo de chamadores** (§2.4) |
| limpar a matéria quando a leitura falha | escolher o objecto 3D é uma selecção **sem pixels**: cai ali, e limpar reabre o report (§2.3) |
| re-ler a matéria por quadro | `4 MiB` de `readback` por quadro para uma resposta que só muda num gesto |
