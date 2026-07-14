# 66 — FX de passe: **a premissa do plano é falsa** (documento de DECISÃO)

> **Status:** ⚠️ **NÃO IMPLEMENTADO — precisa de decisão do Enio.**
> Linha `line/motion-value`, 2026-07-14. Isto **não é** um pedido de permissão burocrático: a
> pesquisa mostrou que a fatia **não é o que o plano diz que é**, e as duas formas que ela pode
> tomar têm donos diferentes.

## 1. O que o plano manda fazer

> *"espacial → pass no compositor HDR (glow/bloom, blur dual-Kawase, vignette). Documento declara
> `layer_fx`."* — plano do módulo, §1.6 e §3
>
> *"Reuso obrigatório do compositor GPU do Painter (`ph2d-painter-effects`)."* — a fila, no handoff

**Três das quatro afirmações dessa frase estão erradas.** Levantamento com `file:line`, feito hoje:

| A frase diz | O código diz |
|---|---|
| *"o compositor GPU do Painter"* | O compositor **não está** em `ph2d-painter-effects` — está em **`ph2d-render`** (`layer_compositor/`). A `-effects` é só dados + kernels **CPU**. Ele **já é desacoplado** do Painter (fala `LayerOp` + `LayerPixelProvider`), então "reusar" nem é o obstáculo. |
| *"compositor **HDR**"* | O compositor é **8-BIT**. `inject_slice_from_texture` **rejeita** qualquer formato que não seja `Rgba8Unorm` (`api.rs:245`), e o storage de saída dele é `Rgba8Unorm` (`compositor/mod.rs:90`). O `game_rt` é **`Rgba16Float`** (`game_rt.rs:33`). |
| *"glow/bloom no compositor"* | **É aqui que a coisa quebra.** Passar o HDR por ele é um round-trip **16F → sRGB8 → 16F** — e isso **destrói exatamente aquilo de que o bloom vive**: os valores **acima de 1.0**. Um bloom que só enxerga até o branco não é um bloom, é um blur do branco. |
| *"blur dual-Kawase"* | Não existe. O blur do compositor é **gaussiano separável**. "Kawase" aparece **uma vez no repo**, num comentário de teste. Vignette **também não existe** em lugar nenhum. |

E o **`layer_fx` no `MotionDoc` não existe** (confirmado: o `ph2d-motion-doc` tem exatamente três
tipos públicos, e nenhum deles é isso).

O **Flip** paga esse mesmo pedágio de 8 bits e **se justifica** (`flip_pass.rs:12`): *"o artwork do
Flip é linha SDR — o round-trip 8-bit é imperceptível"*. **Essa justificativa não se transfere** pro
glow: bloom é *precisamente* o caso em que o HDR importa.

## 2. E tem uma segunda descoberta, que muda o DONO da fatia

**O Motion não é separável no GPU.** As instâncias que ele cozinha entram no **mesmo** passe de
sprites da cena, concatenadas e **ordenadas junto** com os sprites do ECS
(`present.rs:124` → `renderer.render_with_extra(...)` → `collect_sorted_instances`). Depois disso
**não há tag de origem**: o `game_rt` tem a cena inteira, e ninguém sabe mais o que era Motion.

Então um "pass de FX sobre o que o Motion renderiza" **não existe hoje**. O que existe é:

- um **`game_rt` HDR** (`Rgba16Float`, linear-light, e o tonemap **está bypassado**);
- um **slot de pós-processo VAZIO** entre o passe do Flip e o tonemap (`present.rs:161..164`).

## 3. As duas formas que a fatia pode tomar (e elas têm donos diferentes)

### Opção A — **Pós-processo do FRAME** (o slot que já está lá, vazio)

Um passe HDR-nativo em WGSL no `game_rt`: bright-pass → cadeia de down/up (Kawase de verdade) →
soma · vignette · levels · hue. **Nada de compositor de 8 bits.**

- **Custo:** 1 crate/módulo novo em `ph2d-render` (~400 linhas + WGSL) + 1 passe no `present.rs`.
- **Ganho:** é o *post stack* do app. Barato, honesto, e ocupa um slot que foi construído pra isso.
- **⚠️ Blast radius:** afeta **TUDO** — sprites, Painter, Flip, Vector. **Não é uma feature do módulo
  Motion**; é uma feature do **app**. Mexer no `present.rs` (que as 6 linhas compartilham) pra mudar
  a aparência do frame inteiro **não é uma decisão que eu deva tomar sozinho.**

### Opção B — **RT próprio do Motion** (o que o plano *queria* dizer)

O Motion desenha num `Rgba16Float` **só dele**, os FX rodam ali, e o resultado é composto de volta
no `game_rt` (premult-over).

- **Custo:** tudo da Opção A **+** um render target novo **+** um segundo passe de sprites (hoje o
  `render_with_extra` **concatena** ECS+Motion; seria preciso um `render_instances_only`, e isso é
  `ph2d-render` foundational).
- **Ganho:** **o glow é do módulo.** As faíscas brilham, o fundo não — que é o que motion graphics
  quer. Blast radius **zero** fora do Motion: nenhuma outra linha muda de aparência.
- **Preço:** ~2× o trabalho de sprite por frame quando a tool está ativa.

### O que eu faria: **B**

Porque *"os FX de passe são do Motion"* é a coisa que o plano estava tentando dizer, e porque é a
única das duas que **não muda o produto inteiro**. A Opção A é uma feature legítima — mas é a feature
de **outro plano**, com outro dono, e provavelmente merece um ADR próprio (*"o PH2D tem um post
stack"*).

## 4. O que NÃO está bloqueado (e já está pronto)

Os **FX de STREAM** — os que operam por-instância em vez de por-pixel — o próprio plano já os separa
dos de passe, e eles **já existem**: `fx.drop_shadow` (duplica e desloca), `fx.rgb_split` (3×+tint),
`motion.mirror`. Nada deles precisa de compositor.

## 5. Onde isto pousa

- Nada foi construído. Nada foi tocado no `ph2d-render` nem no `present.rs`.
- A fila do handoff deve dizer, daqui pra frente, **"FX de passe = Opção A ou B (doc 66)"** — e não
  mais *"reuso obrigatório do compositor do Painter"*, que é uma instrução que **produz um bloom
  errado**.
