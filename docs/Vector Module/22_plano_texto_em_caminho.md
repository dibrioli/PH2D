# 22 — Texto em caminho (pesquisa + plano)

> **Origem:** Enio, 2026-07-22 — *"Creio que ainda não temos paths deformando textos. Faça estudo e
> pesquisa sobre a ferramenta e coloque nos planos para implementar."*
>
> **Estado:** ✅ **CONSTRUÍDO (W0–W5, 2026-07-22)** — motor + vínculo + UI + alça de canvas, tudo
> smokado. Linha `line/Vector`, aguardando ordem de integração. **Zero bump de schema, zero
> contrato congelado tocado.** O que mudou de forma face ao plano está marcado ⚠️ em cada wave.
>
> Irmãos: [`20_pesquisa_ferramentas_de_artista.md`](20_pesquisa_ferramentas_de_artista.md) §1.2 (que já
> tinha nomeado o item e o classificado como "Faixa A #3, Pequeno/Médio, alto retorno") ·
> [`21_pesquisa_envelope_warp.md`](21_pesquisa_envelope_warp.md) (a deformação NÃO-afim, que é
> outra coisa — §1 abaixo) · [ADR-0121](../architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md)
> (fonte autorada ≠ geometria cozida) · [ADR-0129](../architecture/decisions/0129-vector-envelope-warp-one-spine-cage-as-container-entity.md)
> (o deformador é uma entidade-container).

---

## §1 — A premissa do pedido está meio certa, e a metade errada importa

*"Paths deformando textos"* são **três** features distintas na indústria. Elas se parecem numa
screenshot e não têm nada em comum na implementação:

| # | O que é | Nome canônico | Glifo é… | Nosso estado (MEDIDO) |
|---|---|---|---|---|
| **A** | O texto **corre ao longo** de uma curva; a curva vira a baseline | *Type on a Path* (AI) · `<textPath>` (SVG) | **RÍGIDO** — só translada e gira | **NÃO EXISTE** |
| **B** | O texto é **espremido dentro** de uma forma arbitrária | *Envelope Distort → Make with Top Object* | **DEFORMADO** (não-afim) | **EXISTE** — e alcança o texto |
| **C** | O texto é deformado por **preset** (Arco, Bandeira, Onda…) | *Effect → Warp* (15 presets) | **DEFORMADO** | **EXISTE** — `EnvelopeWarp`, 7 presets |

**A distinção não é acadêmica — ela decide o custo.** O caso **A** posiciona cada glifo por um
**afim rígido**, e um afim **comuta** com a avaliação de Bézier
([`ph2d-vec-envelope/src/lib.rs:12-17`](../../crates/ph2d-vec-envelope/src/lib.rs)) ⇒ **não há refit,
não há subdivisão, não há aproximação**. Os casos **B**/**C** amostram um campo não-afim e pagam
`fit_to_bezpath` — a espinha inteira do ADR-0129. É por isso que o **A** é pequeno e o **B** custou
um ADR.

### O que já funciona (não reconstrua)

O envelope (**B** e **C**) está costurado ponta a ponta — motor
([`ph2d-vec-envelope`](../../crates/ph2d-vec-envelope/)), componente ECS
([`ph2d-ecs/src/vec_envelope.rs:169`](../../crates/ph2d-ecs/src/vec_envelope.rs)), recook por frame
([`envelope_live.rs:214`](../../shells/desktop/src/envelope_live.rs)), gesto de canvas
([`envelope_gesture.rs`](../../shells/desktop/src/envelope_gesture.rs)) e seção própria no painel
([`paint_envelope.rs:30`](../../crates/ph2d-panel-vector/src/paint_envelope.rs)).

E ele **alcança o texto sem gate nenhum** (verificado): `warp_path`
([`lib.rs:91-113`](../../crates/ph2d-vec-envelope/src/lib.rs)) deforma `verts` **e** itera
`subpaths`, e um `VecShape::Text` é exatamente um `VecPath` compound
([`vec_glyph.rs:98-127`](../../shells/desktop/src/vec_glyph.rs)). Grep por `Text` em
`envelope_live.rs`/`envelope_gesture.rs`: **zero ocorrências**. As recusas do `create`
([`:130-151`](../../shells/desktop/src/envelope_live.rs)) são todas sobre id/entidade/domínio —
nenhuma olha para `VecShape`.

⚠️ **Mas "sem gate" não é o mesmo que "verificado"** — §2.

---

## §2 — O achado que vale mais que a feature: um bug VIVO no texto

**Isto existe hoje, independe deste plano, e ninguém o veria acontecer.**

`recook_text_object` ([`vec_text_object.rs:142`](../../shells/desktop/src/vec_text_object.rs)) faz:

```rust
*p = np;   // substitui o VecPath INTEIRO
```

O `np` sai de `text_to_compound_path` com `..Default::default()`
([`vec_glyph.rs:126`](../../shells/desktop/src/vec_glyph.rs)) ⇒ **`effects` volta a vazio** e a
geometria deformada some. Consequência medida por leitura:

- Aplique **Zig Zag** num texto → funciona (a pilha roda sobre compound; `apply_per_contour` trata
  cada glifo e cada furo por si, [`effect.rs:506-525`](../../crates/ph2d-vec-scene/src/effect.rs)).
- Depois **edite uma letra** → o re-cook dispara e **a pilha inteira desaparece em silêncio**.

⚠️ **CORREÇÃO desta seção (2026-07-22, medida ao construir a W0):** a 1ª versão deste plano dizia
que *"idem para um texto dentro de um envelope"*. **Falso.** O `write_shape` do envelope
([`envelope_live.rs:457`](../../shells/desktop/src/envelope_live.rs)) escrevia os seis campos **à
mão** e por isso **preservava** `effects` — era o único dos três re-cooks que acertava. Estava
certo por **enumeração**, o que é outra coisa: acertava os campos de hoje e ficaria errado em
silêncio no sétimo. É essa distinção que decidiu a forma da cura (§6, W0).

O re-cook é **event-driven, não por frame** (call sites em `vec_text_object.rs:164`,
`vec_text.rs:97,124,132,155,261-326`, `label_live.rs:418`, `vec_text_reopen.rs:186`) — por isso
não aparece parado na tela; só dispara quando o artista **volta a digitar**, que é o momento em que
ele não está olhando para o efeito.

⚠️ **O ADR-0129 já nomeou esta classe e o texto ficou de fora da enumeração:**

> *"Uma **Live Shape** não pode hospedar gaiola pelo mesmo motivo do raio (o `recook_into` reescreve
> `verts`). Escape: Convert to Curves."* — ADR-0129, linha 325

O texto **é** uma Live Shape com re-cook que reescreve tudo, e nunca foi conferido contra essa
regra. É a mesma doença que o repo já pagou três vezes: **uma condição que ENUMERA seus leitores
apodrece** quando entra o leitor N+1.

**Isto entra na Wave 0 abaixo, antes de qualquer feature nova** — e a correção é a mesma coisa que
o text-on-path precisa de qualquer jeito (o re-cook tem de PRESERVAR o que não é geometria).

---

## §3 — Estado da arte (o que a indústria de fato faz)

### 3.1 A especificação aberta é o SVG `<textPath>`

O algoritmo canônico é público e é o que Illustrator, Inkscape, Figma e os browsers implementam:

- `startOffset` — a posição inicial ao longo do caminho, absoluta ou em **percentagem do
  comprimento total** ([MDN](https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/startOffset)).
- **Cada glifo é ancorado pelo seu MEIO**, não pela borda esquerda: a especificação posiciona em
  `mid = x + advance/2 + offset`, e rotaciona em torno desse ponto
  ([Using SVG, cap. 7](https://oreillymedia.github.io/Using_SVG/extras/ch07-textpaths.html)).
  Isto **não é detalhe**: ancorar pela borda esquerda faz a letra girar para fora da linha e é a
  origem de metade dos artefatos em curva apertada.
- `side` (`left`/`right`) escolhe o lado; `text-anchor` combinado com `startOffset: 50%` é o idioma
  de "centrar no arco".

⚠️ **Prior art fechada:** a Adobe tem patentes sobre *"Warping text along a curved path"*
(US 6,803,913) e *"Displaying text on path"* (US 9,001,126). A implementação aqui sai da
**especificação SVG**, que é aberta e anterior — e a referência a Illustrator/Inkscape é
**comportamental** (o que a ferramenta faz), nunca código, exatamente como o projeto já fez com o
Blender Texture Paint.

### 3.2 Illustrator — *Type on a Path Options* (o vocabulário que o artista já conhece)

[Adobe — Create type on a path](https://helpx.adobe.com/uk/illustrator/using/creating-type-path.html) ·
[opções explicadas](http://vectips.com/tips/type-on-a-path-options/)

**Cinco efeitos de orientação por glifo** — e cada um é literalmente um afim diferente:

| Efeito | O que o afim faz |
|---|---|
| **Rainbow** (default) | eixo do glifo ⟂ à curva; gira com a tangente |
| **Skew** | glifo fica **vertical**, cisalhado horizontalmente pela tangente |
| **3D Ribbon** | glifo vertical, **sem** cisalhar; largura escala por `cos(θ)` (parece uma fita virando) |
| **Stair Step** | glifo vertical e **sem giro nenhum** — só a baseline sobe/desce em degraus |
| **Gravity** | base do glifo na curva, topo apontando para o **centro geométrico** (num círculo perfeito coincide com Rainbow; a diferença aparece em elipse/retângulo) |

**Quatro alinhamentos à curva** (que linha do glifo encosta no caminho): **Ascender · Descender ·
Center · Baseline** (default).

Mais: **flip** (inverter para o outro lado), e as alças **in/out/center** que definem começo, fim e
o gesto de arrastar o texto ao longo da curva.

### 3.3 O problema que ninguém resolve bem — e a decisão honesta

Em curva côncava apertada as letras **colidem** do lado interno; em convexa elas **abrem**. Isto é
geometria, não bug: o comprimento de arco na baseline difere do comprimento à altura do olho.

A [pesquisa 20](20_pesquisa_ferramentas_de_artista.md) já registrou isto (`:64-66`): *"Ninguém
resolve isso perfeitamente; os bons detectam e avisam, ou empurram a linha de base."* A literatura
de patente propõe um fator de compensação proporcional a `tamanho_do_glifo × sin(Δθ)`.

**Decisão proposta (reversível, e é a do Illustrator):** *não* compensar automaticamente na v1.
Ancorar pelo MEIO do glifo (§3.1) já remove a pior parte, e o **tracking** é o controle que o
artista já tem e já entende. Compensação automática vira knob que muda o espaçamento por motivo
invisível — a classe de ergonomia que este projeto já classificou como bug de design. Fica
**nomeado** como item futuro com gatilho: *se o smoke mostrar colisão com tracking 0 num raio que o
artista de fato desenha, reabrir com medição.*

### 3.4 Os vizinhos, para calibrar ambição

- **Inkscape** tem *Text on Path* e, separadamente, o LPE **Bend** — que entorta **geometria
  qualquer** ao longo de uma curva ([guia](https://inkscape-manuals.readthedocs.io/en/latest/live-path-effects.html)).
- **Blender Grease Pencil** não tem texto, mas o modificador **Envelope** e o **Shrinkwrap** são a
  mesma família ([manual](https://docs.blender.org/manual/en/latest/grease_pencil/modifiers/index.html)).
- **After Effects** não tem texto-em-caminho vetorial de verdade (usa *Path Text* / máscara).

---

## §4 — O que já temos, medido (por que isto é pequeno)

| Peça que um text-on-path precisa | Onde já está | Nota |
|---|---|---|
| Texto vetorial vivo, editável, com variable fonts | `VecShape::Text` · [`vec_text.rs`](../../shells/desktop/src/vec_text.rs) | eixos `wght`/`wdth`/`slnt`, família, tracking, entrelinha, align |
| Glifo → `VecPath` (só translação + escala) | [`vec_glyph_build.rs:25-42`](../../shells/desktop/src/vec_glyph_build.rs) | handles absolutos (Rive), quadrática elevada a cúbica |
| **Comprimento de arco exato + inverso** | [`ph2d-vec-scene/src/arclen.rs`](../../crates/ph2d-vec-scene/src/arclen.rs) | Gauss-Legendre 16 nós + bisseção 40 it.; só `sqrt` (HR-5) |
| **Tangente** (com `None` em cúspide) | `arclen.rs:62` `tangent_at` | é a metade da normal |
| **Caminhar por arco sobre VÁRIOS segmentos** | [`fx_zigzag.rs:281`](../../crates/ph2d-vec-scene/src/fx_zigzag.rs) `walk_to(segs, starts, s)` | ⚠️ **é aqui que mora a única decisão de arquitetura** — §5.1 |
| Largura total da linha + offset de alinhamento | `vec_glyph.rs:162` `line_advance` · `:182` `align_offset` | sobrevivem intactos: viram `s` inicial |

⚠️ **O `arclen.rs` foi escrito PARA isto.** O cabeçalho dele diz, textualmente (`:4`):

> *"é pré-requisito de metade da fila de efeitos (Trim, Repeater, Pattern Along Path, **texto em
> caminho**)"*

### O ponto de enxerto é UMA linha

[`vec_glyph.rs:80`](../../shells/desktop/src/vec_glyph.rs) — o argumento `origin` de
`glyph_to_vec_path`:

```rust
let mut pen_x = align_offset(layout.align, width);          // :70
for ch in line.chars() {
    let advance = f64::from(font.advance(gid, axes)…) * scale;   // :76
    glyph_to_vec_path(&outline, scale,
        [origin[0] + pen_x, origin[1] + pen_y],             // :80  ← translação PURA
        fill.clone(), *stroke);
    pen_x += advance + track_px;                            // :85
}
```

`pen_x` deixa de ser uma coordenada e passa a ser **distância de arco `s`**; o `[x, y]` passa a ser
um **afim** derivado de `(point_at, tangent_at)` em `inv_arclen(s + advance/2)`. Nada mais no laço
muda.

---

## §5 — Desenho proposto

### 5.1 A ÚNICA decisão de arquitetura: o walker de arco é COMPARTILHADO

`fx_zigzag` já anda por comprimento de arco sobre uma lista de cúbicas
([`fx_zigzag.rs:281`](../../crates/ph2d-vec-scene/src/fx_zigzag.rs) `walk_to`). O text-on-path
precisa **exatamente disso**. Duas cópias respondendo *"onde fica o arco `s` neste caminho?"*
divergiriam em silêncio — a doença que este repo já nomeou uma dúzia de vezes.

**Proposta:** extrair para `ph2d-vec-scene/src/arc_walk.rs` (módulo **irmão** de `arclen.rs`, o
padrão de isolamento da regra B'), com o `fx_zigzag` passando a delegar. Byte-identidade do
`fx_zigzag` é gate obrigatório da extração (o efeito já shipou; o desenho aprovado não pode mover).

### 5.2 O modelo: o vínculo mora no TEXTO, não num container

O envelope escolheu **entidade-container** (ADR-0129 §2) porque deforma *arte qualquer* e precisa
ser pai dela. Text-on-path é diferente: quem consome o caminho é o **layout do texto**, e o layout
já tem uma porta única de re-cook (`recook_text_object`). Um container aqui seria uma segunda porta
para a mesma pergunta.

```rust
// em VecShape::Text (append-only)
on_path: Option<VecPathId>,      // o caminho-guia; None = texto reto (byte-idêntico a hoje)
start_offset: f64,               // fração 0..1 do comprimento total (idioma SVG)
path_effect: TextPathEffect,     // Rainbow | Skew | Ribbon3D | StairStep | Gravity
path_align:  TextPathAlign,      // Ascender | Descender | Center | Baseline
flip: bool,                      // troca o lado
```

**Consequências que caem de graça:**

- O caminho-guia **continua um `VecPath` normal** — editável no modo Node, com Live Corners, com
  pilha de efeitos própria. Mover o caminho re-flui o texto (o re-cook já é event-driven).
- **Undo, save e keyframe** vêm de graça: é campo de componente ECS, como todo o resto.
- `on_path: None` é o caminho de hoje, **byte-idêntico** — e isso é gate.

⚠️ **Custo de schema:** apender campos a `VecShape::Text` é postcard **posicional** ⇒ exige bump de
`VEC_SCENE_SCHEMA_VERSION` (hoje **13**, [`lib.rs:466`](../../crates/ph2d-vec-scene/src/lib.rs)) e
provavelmente de `PROJECT_SCHEMA`. **O número se CONTA, não se escolhe** — quem integrar soma os
bumps das linhas vivas; esta linha anota "+1" no handoff e não crava o valor.

### 5.3 O que o caminho-guia faz na tela

Illustrator torna o caminho **invisível** (sem fill/stroke) mas mantém-no selecionável e editável.
**Proposta: espelhar isso** — o caminho não perde o `Paint`, ele passa a **não ser pintado enquanto
hospeda texto**, e volta a pintar se o texto for desvinculado. Assim nada é destruído e o gesto é
reversível. As três alças (in / center / out) são gizmo de canvas **Node-only**, exatamente como as
alças da gaiola do envelope ([`envelope_gesture.rs:14-17`](../../shells/desktop/src/envelope_gesture.rs)).

---

## §6 — Waves

### W0 — O re-cook preserva o que não é geometria ✅ **CONSTRUÍDA (2026-07-22)**

**A cura não foi corrigir dois `*p = np`: foi tirar de três sítios a pergunta que nenhum deles
devia estar respondendo.** Quem sabe *quais campos um re-cozimento produz* é a crate dona do tipo.

- **Porta única nova:** `VecPath::replace_cooked`
  ([`ph2d-vec-scene/src/recook.rs`](../../crates/ph2d-vec-scene/src/recook.rs), módulo irmão de
  `compound` — os dois só acrescentam métodos inerentes). Substitui geometria + estilo; preserva
  **`id`** e **`effects`**.
- ⚠️ **O guarda é o COMPILADOR, não um comentário:** o corpo faz `let Self { .. }` **exaustivo**,
  sem `..`. Acrescentar um campo a `VecPath` **deixa de compilar**, e obriga quem o acrescenta a
  responder *"isto é produzido pelo re-cozimento, ou sobrevive a ele?"* no commit em que o campo
  nasce — a única hora em que a resposta é conhecida. Era exatamente o modo de falha do
  `write_shape`, que estava **certo e frágil**.
- **Três chamadores** passaram por ela: `recook_text_object` · `regen_into` · `write_shape`.

**E a W0 achou um SEGUNDO defeito, pré-existente e independente do texto:** `envelope_live::create`
assa `src.cooked()` na fonte do filho (está escrito lá) **e deixava a pilha armada no path** ⇒ o
`cooked()` do renderer aplicava o efeito **outra vez** sobre a forma já ondulada. O artista via um
Zig Zag com o dobro do que pediu, sem número nenhum na tela a explicá-lo. Cura: **quem assa,
desarma** — o mesmo trade da booleana com as quinas (ADR-0121).

**Gates (todos nasceram VERMELHOS contra o produto de 2026-07-22):**

| Gate | Onde | Sintoma que pega |
|---|---|---|
| `editing_a_text_object_keeps_its_effects_stack` | shell, caminho do **painel** | `left: []` |
| `typing_in_a_live_text_session_keeps_its_effects_stack` | shell, caminho da **sessão viva** | `left: []` |
| `the_envelope_does_not_apply_a_baked_effect_a_second_time` | shell, envelope | efeito em dobro |
| 4 unit gates de `replace_cooked` | `ph2d-vec-scene` | invariantes da lei |

⚠️ O gate do envelope tem oráculo de **APARÊNCIA** (*o que renderiza é o que o recook escreveu*),
nunca a regra do conserto — continua válido se a cura mudar de forma.

**Mutações:** `*p = np` de volta ⇒ 3 RED · `create` sem desarmar ⇒ 1 RED · esquecer de copiar
`fill` ⇒ 2 RED · deixar o `id` do cozido vencer ⇒ 2 RED.

**Fica ABERTO, e é decisão do Enio (não a tomei sozinho):** `create` podia guardar a fonte
**AUTORADA** em vez da cozida, e então a pilha sobreviveria ao Release inteira (não-destrutivo de
verdade, ADR-0121). O preço é que o efeito passaria a ser calculado **depois** da deformação —
as cristas seguiriam o contorno deformado em vez de serem esticadas pela gaiola. Hoje o
comportamento é o do Illustrator (o envelope deforma a APARÊNCIA); mudar isso é redesenho do
envelope, com aceitação própria.

**Esta wave fechou sozinha e vale sozinha.** Não depende de nada abaixo.

### W1 — O walker de arco vira porta única ✅ **CONSTRUÍDA (2026-07-22)**

`ph2d-vec-scene::arc_path::ArcPath` (irmão do `arclen`, que responde por **uma** cúbica):
`from_contour` · `total` · `anchor_arcs` · `frame_at(s) -> (ponto, tangente)`. O `fx_zigzag`
**delega**, e o walker privado dele morreu.

⚠️ **A porta NÃO tem opinião sobre onde amostrar** — isso é do efeito (o Zig Zag quer a grade de
cristas unida às âncoras; o texto quer uma posição por glifo). Puxar a política para dentro faria
a porta legislar sobre features que ainda não existem.

**O oráculo da extração é uma impressão digital pinada** (`0xf6ebed490c31322e`), medida **antes**
de mexer no efeito, sobre 4 regimes: fechado · fechado subdividido (a UNIÃO) · **aberto com
quinas** · Roughen com semente. O Zig Zag já shipou; um refactor que "só move código" prova-o ao
BIT, não por inspeção. **Bateu exatamente.**

⚠️ **A fixture das QUINAS é que carrega o gate, e isto foi medido:** sob a mutação `p <= s` →
`p < s` (off-by-one na busca binária), **os outros 18 gates ficam verdes** — porque o *ponto*
resolvido é idêntico (satura no fim do segmento anterior) e só a **tangente** difere, e só onde há
descontinuidade. Num círculo liso as duas tangentes concordam. Sem uma quina na fixture, a suíte
inteira seria verde sobre um walker trocado.

**Mutações:** off-by-one na busca binária ⇒ 1 RED (só a impressão digital) · `anchor_arcs`
devolver também o total ⇒ 4 RED.

⚠️ **Um gate MEU nasceu com a tolerância errada e a medição corrigiu-o:** o perímetro do círculo
deu `377,0440` contra `2πR = 376,9911` (**1,40e-4**), acima do `1e-4` que eu tinha escrito. Não é
o integrador — é a assinatura do círculo aproximado por 4 cúbicas. O **controle** que separa as
duas explicações é o gate da reta, onde o mesmo integrador acerta `100.0` a **1e-9**.

### W2 — O motor: arco → afim por glifo ✅ **CONSTRUÍDA (2026-07-22), com escopo ESTREITADO**

`ph2d-vec-scene::text_path::GlyphFrame` — `on_path(caminho, s, dy, flip)` + `apply([dx, dy])`.
Kernel puro: **não sabe o que é um glifo**. Quem conhece fonte, avanço e ascender é o shell; quem
conhece arco é esta crate.

⚠️ **SÓ a Rainbow foi construída, e o corte é deliberado.** O plano previa as cinco variantes de
§3.2 num só passo. Ao implementar, a fonte da Adobe caiu **três vezes** (timeout · certificado
expirado · 403), e eu só tenho as outras quatro **de memória**. **Skew e 3D Ribbon são duais** —
uma preserva as arestas verticais do glifo, a outra as horizontais — e implementar a trocada
produziria um efeito coerente, bonito e **com o rótulo errado**. É o defeito exato que o Painter
pagou ao armar `Falloff::Sharp` no lugar de `Pow4` porque o *identificador* casava com a palavra
da UI. A Rainbow entrou porque tem especificação **aberta e normativa**: ela é o `<textPath>` do
SVG. As outras entram quando a spec estiver **fixada por fonte citável**, não por memória.

**A âncora é o MEIO do glifo** (`mid = pen + avanço/2`), que é normativo no SVG — ancorar pela
borda esquerda faz a letra girar *para fora* da linha e é a origem de metade dos artefatos em
curva apertada.

**O `flip` custa um sinal, não dois:** virar percorre o arco da outra ponta **e** inverte a
tangente; a normal inverte **junto** (é a perpendicular dela), então o `dy` passa para o outro
lado sozinho.

**Gates (9), todos com oráculo geométrico:** rigidez (eixos ortonormais em todo ponto) · num
círculo o texto fica **no círculo** · o `dy` levanta à **esquerda da marcha** · flip troca de lado
· flip lê da outra ponta · numa **reta** o referencial é a identidade transladada (texto em
caminho sobre linha reta é indistinguível de texto normal) · `apply([0,0])` é a origem · **o glifo
mantém o TAMANHO** em qualquer ponto da curva (é a diferença entre isto e um envelope, e é por
isso que não há refit) · **cúspide devolve `None`**, não um ângulo inventado.

⚠️ **Um gate meu nasceu vermelho por uma frase errada MINHA, não por código errado:** eu escrevi
que `dy` positivo vai *"para fora da curva"*. O círculo do fixture é **anti-horário** e a esquerda
de quem o percorre aponta para o **centro**. O árbitro foi o gate da reta, onde `dy = 7` põe o
glifo em `y = +7` — **acima** da baseline, que é o que texto precisa. *"Fora"* é
winding-dependente e não serve de especificação; **"à esquerda da marcha"** serve, e é o que o doc
diz agora.

**Mutações:** normal para a direita ⇒ 3 RED · flip sem inverter a tangente ⇒ 2 RED · flip sem
espelhar o arco ⇒ 1 RED · cúspide a devolver referencial inventado ⇒ 1 RED.

### W3 — O layout consome o motor ✅ **CONSTRUÍDA (2026-07-22)**
O laço de layout deixou de calcular um PONTO por glifo e passou a resolver um **referencial**;
`glyph_to_vec_path` recebe `&GlyphFrame`. O texto reto ficou **byte-idêntico**, pinado pelo
fingerprint `0x511c_90e4_7b1f_01db` — **medido no commit ANTERIOR** (`abcf2c603`), para a alegação
*"medido antes"* ser verificável no histórico e não uma afirmação minha.

**Duas decisões de forma que valem mais que o código:**

- **`TextPlacement` é um ENUM**, não `(origem, Option<caminho>)`. Sobre um caminho a origem do
  bloco não é ignorada por convenção — ela **deixa de existir** (o `startOffset` a substitui). Um
  par tornaria exprimível *"tenho origem E caminho"*, que nada sabe honrar, e é desse estado que
  nasce o bug em que metade do código lê um e metade lê o outro.
- **`glyph_frame` é PORTA ÚNICA**: o laço de glifos e o caret perguntam a ela. É o que impede
  estruturalmente o defeito que esta wave nomeou desde o plano — *o cursor no texto reto com as
  letras na curva* —, e que enumerar dois sítios só promete. **Um caret é um glifo de avanço zero.**

**O que a medição negou (de novo):** a 2ª linha num círculo **anti-horário** empilha para **FORA**,
não para dentro — a subida da letra aponta para o centro. Terceira vez que "dentro/fora" falha e
*"à esquerda do sentido de marcha"* acerta. O gate passou a medir **paralelismo** (as duas bordas
da banda andam o mesmo tanto), que é o que uma divergência ao longo da palavra quebraria — e um
*"está mais para lá"* não quebraria.

**⚠️ Um defeito MEDIDO e deliberadamente NÃO corrigido.** No documento uma reta é a cúbica
degenerada `(P0,P0,P3,P3)`, cuja derivada é **nula nas duas pontas**: `ArcPath::frame_at(0)` de um
segmento reto não tem tangente, e o texto — que não inventa rumo — salta ali. A cura é óbvia
(amostrar a derivada um passo para dentro) e foi **escrita e medida**: ela faz sangrar o
fingerprint do Zig Zag, porque o Zig Zag amostra **exatamente nas âncoras**, que são os pontos
estacionários. Ou seja **muda o desenho de um efeito que o Enio já aprovou em smoke** — decisão de
produto, não carona de uma wave de texto. Revertida, e o defeito fica **gateado**
(`a_stationary_parameterisation_has_no_direction_and_the_text_skips_it`) para não ser
re-descoberto do zero nem curado sem que alguém veja o preço. **A W4 encosta nisto**: um texto
ligado a um segmento reto com `startOffset = 0` perde o cursor.

**Dois gates nasceram fracos, pela mesma doença — a fixture não continha o fenômeno:**
o gate do *drop* usava uma **reta**, e ali a saturação já cai na tangente nula, então o glifo era
descartado **por acidente** e o gate ficava verde com o guard inteiro apagado (virou um **arco**);
e *"o pen avança nos glifos saltados"* estava afirmado num comentário e preso por nada — a mutação
**sobreviveu**, e o caso que o prova é o gesto mais comum da feature (texto **centrado**, cuja
primeira metade cai em arco negativo: com o pen travado a palavra inteira sumiria).
**6 mutações, 6 sangram.**

**⚠️ E um arch-gate que EU MESMO ceguei na W0:**
`every_live_host_that_rewrites_verts_is_named_by_the_radius_handle_policy` estava **vermelho
latente desde `5bc175013`** — o `envelope_live` passou a reescrever pela porta `replace_cooked`, a
assinatura sintática sumiu do detector e o controle positivo caiu de 5 para 4 hosts, que é
exatamente a falha que aquele `assert` existe para gritar. **Não vi porque as verificações
intermédias correram filtradas por nome, e `tests/*.rs` não corre com filtro.** `.replace_cooked(`
entrou no detector; toda porta NOVA de reescrita tem de entrar lá no commit em que nasce.

**Cena de smoke 22** (`PH2D_BUILD_SMOKE=22`) — a wave é motor, então sem cena seria invisível, e
*invisível* e *quebrado* têm a mesma cara. Onda + dois círculos (um virado). **Medido antes de a
mensagem afirmar o que mostra**: 11 + 39 + 22 glifos, todos desenhados, ângulos de +37° a −54° na
onda. ⚠️ A minha 1ª mensagem dizia *"a palavra dá a volta"* e era **falso** (21% do círculo) — os
textos foram alongados e a afirmação corrigida. A sonda virou **gate permanente** sobre a MESMA
tabela que a cena desenha.

### W4 — O vínculo + a UI ✅ **CONSTRUÍDA (2026-07-22)**, com o modelo CORRIGIDO

#### W4a — o vínculo

⚠️ **O §5.2 acima estava ERRADO e foi corrigido pela construção.** Ele previa apender
`on_path`/`start_offset`/`flip` ao `VecTextParams`. Isso custaria um bump de `PROJECT_SCHEMA`
(o blob de um componente é postcard **posicional**) — e **um bump RECUSA todo projeto já
salvo**. O que entrou foi um **componente OPCIONAL** `ph2d_ecs::VecTextPath`, que cunha a
própria blob-key e **não move nada**: o raciocínio que fez o `PhysicsJoint` não bumpar e o
`GravityScale` nascer opcional em vez de campo do `RigidBody`. **Zero bump.**

Ganha-se de graça uma frase melhor: *"texto reto é o que NÃO tem o componente"* é garantido
pela ausência, enquanto *"texto reto é `on_path: None`"* cada leitor tem de lembrar.

**O espaço, e ele não foi escolhido — foi herdado.** Um texto vinculado cozinha em **MUNDO**
(o guia já traz a pose dele) e vive na **IDENTIDADE**; uma pose por cima aplicaria a
transformação duas vezes. O `connector_live` já tinha escrito a regra e ela vale palavra por
palavra: *"ele vive na identidade, e é isso que o torna (corretamente) não-arrastável pelo
gizmo — arrastar um conector não quer dizer nada"*. **Mover um texto em caminho não quer dizer
nada; o que se move é o caminho** (é o que o Illustrator faz — lá os dois são um objeto só), e
o `settle_origins` já o respeita sem saber que existe (pula toda entidade com `VecShape`). A
identidade é re-imposta a CADA re-cook, não uma vez ao vincular: o gizmo continua lá.

Três escolhas que são decisão: o guia é lido `cooked()` (as Live Corners dele contam) **e
assado pela pose de mundo dele** (senão mover o caminho deixaria de mover o texto — a metade
visível da feature) · o `start_offset` é **fração**, convertida numa porta só · e um guia
**apagado** devolve o texto ao layout reto em vez de o fazer sumir.

#### W4b — a UI

Seção **Text on Path** com **duas caras**: texto solto mostra só a porta de entrada (e só com a
seleção que o gesto exige); texto preso mostra Offset, lado e Detach. Nunca as duas — e a
**ausência tem gate próprio**, senão um `paint` que desenhasse tudo sempre passaria no gate de
cliques.

⚠️ **Os 5 chips de efeito e os 4 de alinhamento NÃO entraram, e isso é a W2 a ser honrada:** só
a orientação **Rainbow** existe (a spec das outras quatro não está fixada), e o alinhamento
precisa de `ascender`/`descender` que a `ph2d-vector-font` não expõe. Um segmentado de um item
e quatro chips que não têm de onde tirar o número seriam controles mortos.

⚠️ **Um erro meu, corrigido antes de shipar:** a 1ª versão pôs o **Detach como terceira opção
do segmentado do lado**. Errado de um jeito que só se vê ao clicar — os outros dois são
ESTADOS e ele é uma AÇÃO sem estado ligado.

**O arch-gate que faltava:** o seam prova que o clique chega ao **barramento**; isso é metade.
Um id pode chegar lá e **morrer**, que é o bug que o Redo da barra teve por um ano
(*"registrado ≠ despachado"*). Como nenhum teste de unidade alcança a `render_loop`, a prova é
sobre o **fonte**, com contador de controle positivo. E o gate da **quarta condição de UI** (a
que a linha de física descobriu não ser implicada pelas outras três): *a SEQUÊNCIA leva a
algum lugar* — selecionar, prender, mover o offset, virar, soltar, medindo o que o artista VÊ.

**Cena 23** (`PH2D_BUILD_SMOKE=23`): a mesa posta para o gesto. Irmã da 22 — aquela mostra o
motor, esta o caminho até ele, e **as duas falham por motivos completamente diferentes**.

### W5 — A alça no canvas ✅ **CONSTRUÍDA (2026-07-22)**, e com o modelo CORRIGIDO

⚠️ **Uma alça, não três.** O plano dizia *in / center / out*, copiando os colchetes do
Illustrator. Duas das três **não existem no nosso modelo**, e uma alça que não faz nada é pior
que uma que falta:

- **out** é a extensão do *container* de texto (onde ele para e transborda). **Não temos
  container:** o texto flui livre pela curva, sem recorte. A alça não teria o que mover.
- **center-flip** vira o texto quando arrastado *através* da curva — espacial, mas fiddly: ao
  ajustar o offset o artista cruzaria a linha por acidente. O toggle **This side / Other side**
  já faz isso explícito.
- **in** é onde o texto começa (o `start_offset`) — a única das três que é uma posição a
  arrastar. É esta.

**Não é uma segunda porta para o Offset.** O slider já existe; a alça é o **mesmo** número
editado de dois modos, ambos por `vec_text_ride::edit`. É o precedente das alças de gradiente e
do gizmo de Transform: para uma grandeza **espacial**, arrastar na tela é legitimamente diferente
de um slider, não um segundo modelo dela.

Geometria: **`ArcPath::closest_arc(p)`** — o inverso do `frame_at` (onde na curva cai este ponto
de mundo). Amostra grossa + refino local, sem derivada (HR-5), porque a curva não é convexa e
Newton cairia no lóbulo errado numa curva em S.

⚠️ **Modo SELECT, não Node — corrigido pós-smoke (Enio):** no Node a bolinha se confundia com as
âncoras dos outros paths. Mudou para o **Select** (onde não há âncoras, e o gizmo de sprite é
inócuo sobre um texto vinculado — ele vive na identidade) e virou uma **ficha grande e sólida**
(10 px vs 6 das âncoras, preenchida de Accent). A costura seguiu o precedente da alça do
**conector** (press antes do picking/gizmo, `over_canvas_or_gizmo`).

⚠️ A alça é **puro desenho no shell**, então mutá-la deixa a workspace verde ⇒ **arch-gate
próprio sobre o fonte** (`the_textpath_handle_is_drawn_and_dragged`), que ainda checa a **ordem**
do press — e essa checagem pegou o próprio scanner sendo enganado por um comentário
`on_press_node` acima da chamada real. **20 gates no total** (motor + `closest_arc` + Node-only +
arch), **4 mutações de gesto, 4 sangram**.

---

## §7 — Aceitação e kill-criterion (DIRETIVA §5, declarados ANTES do build)

**Conjunto de aceitação concreto:**
1. Um texto vinculado a um círculo lê como texto em círculo, sem letra rodada para fora.
2. Arrastar o caminho re-flui o texto; editar o texto re-flui na curva; **as duas coisas
   sobrevivem a Ctrl+Z**.
3. `on_path: None` é **byte-idêntico** ao produto de hoje (fingerprint).
4. Um save antigo (v13) abre; um save novo carrega o vínculo.
5. As 5 variantes de efeito são **distinguíveis na tela** (gate compara GEOMETRIA, nunca o
   identificador — a lição do par `Layer`/`Layers` da timeline).

**Kill-criterion:** o re-cook é **event-driven**, então o orçamento é o do **gesto**, não do frame.
Se digitar uma tecla num texto de 200 glifos sobre um caminho de 50 âncoras passar de **8 ms**
(mesmo kill do irmão sculpt) depois de uma tentativa de otimização, a feature recua para
**re-cook preguiçoso com cache do walker** — e se nem assim, para **`Convert to Curves` obrigatório
antes de vincular** (texto-em-caminho destrutivo), que é pior UX mas honesto. *Não* se aceita
baixar o teto de glifos sem medir: o teto é do hardware.

✅ **MEDIDO na W3 (2026-07-22): 0,72 ms** contra os 8 do kill — **onze vezes de folga**, sem cache
nenhum. O recuo previsto acima **não é preciso**, e isso está decidido por medição em vez de por
receio. O texto reto custa 0,37 ms ⇒ a inversão de comprimento de arco por glifo **dobra** o
layout, que é o preço honesto de pousar cada letra onde o arco manda.

Os dois gates vivem em `text_path_smoke::perf`: uma **RAZÃO** (cavalgar contra reto, bar 3,5×) que
roda sempre — porque o perfil de teste do CI compila em `opt-level=1` e ali um limite de
milissegundos mede o PERFIL, não a regressão — e o **kill absoluto** em `--release`, `#[ignore]`.
⚠️ **Uma razão contra o número de ÂNCORAS foi tentada primeiro e DESCARTADA por medição**:
quadruplicar as âncoras dá razão **0,90**, e nem trocar a busca binária do `frame_at` por uma
varredura linear a move — o custo do caminho desaparece ao lado do custo dos contornos. Um gate que
nenhuma mutação mata *parece* cobertura e não é. A razão que ficou sangra com a mutação realista
(*"melhorar a precisão"* da inversão de arco, 40 → 400 iterações).

**O que este plano NÃO faz, de propósito:**
- **Não compensa colisão em curva apertada** (§3.3) — nomeado, com gatilho.
- **Não faz texto em ÁREA** (*Area Type*, texto fluindo dentro de uma forma) — é outra feature, com
  quebra de linha e hifenização; fora de escopo.
- **Não ressuscita o `Twist`** — foi construído, medido em 4 variantes e **cortado** por rasgar a
  geometria ([`fx_warp.rs:6-21`](../../crates/ph2d-vec-scene/src/fx_warp.rs)). Não re-derive.

---

## §8 — Superfície de colisão (para o handoff de integração)

### Já construído (W0–W5) — o que a integração vê hoje

| Símbolo novo | Onde | Nota de colisão |
|---|---|---|
| `VecPath::replace_cooked` | `ph2d-vec-scene/src/recook.rs` (módulo NOVO) | Método inerente, arquivo próprio ⇒ isolado. **É a porta única do re-cozimento**, com 3 chamadores |
| `ArcPath` (`from_contour`/`total`/`anchor_arcs`/`frame_at`) | `ph2d-vec-scene/src/arc_path.rs` (NOVO) | Arquivo próprio; o `fx_zigzag` passou a delegar (fingerprint pinado prova byte-identidade) |
| `GlyphFrame` (`on_path`/`shifted_along`/`apply`) | `ph2d-vec-scene/src/text_path.rs` (NOVO) | Arquivo próprio |
| `TextPlacement` · `caret_frame` | `shells/desktop/src/vec_glyph.rs` | `pub(crate)`; **assinatura mudada** em `text_to_vec_paths`/`text_to_compound_path`/`glyph_to_vec_path` (6 chamadores, todos na shell) |
| `PH2D_BUILD_SMOKE=21`, `=22` e `=23` | `build_smoke.rs` + `text_fx_smoke.rs`/`text_path_smoke.rs`/`text_path_gesture_smoke.rs` (NOVOS) | Os níveis de smoke são uma **lista compartilhada**: 21–23 estavam livres, mas o valor **se CONTA na integração** se outra linha os tiver tomado |
| `ph2d_ecs::VecTextPath` | `ph2d-ecs/src/vec_text_path.rs` (NOVO) | Componente novo ⇒ blob-key própria ⇒ **zero bump**. ⚠️ O contador de componentes é **TRÊS**: ecs `33→34`, render e script `34→35` — os dois últimos só aparecem no gate da árvore combinada |
| `VECTOR_SECTION_TEXTPATH` + 6 ids | `ph2d-editor-core/src/ids/chrome/vector_textpath.rs` (NOVO) | Arquivo próprio (split por LOC). ⚠️ **`VECTOR_SECTIONS` é lista compartilhada — só ADICIONAR**, e a entrada foi ao FIM |
| `panel.vector.section.textpath` | `ph2d-i18n/src/lib.rs` | Uma linha numa tabela compartilhada |
| `paint_textpath.rs` · `state_textpath.rs` · `vec_text_ride.rs` | painel + shell (NOVOS) | Arquivos próprios ⇒ isolados |
| `event.rs::track_slider_event` | `ph2d-panel-vector` | **Extração** do `apply_event` (teto de 200 LOC/fn) — move 5 braços existentes |
| `VERTS_REWRITE` ganhou `.replace_cooked(` | `shells/desktop/tests/every_host_that_rewrites_verts_faces_the_radius_handle.rs` | Arch-gate de OUTRA wave que esta linha cegou e curou (§6, W3) |
| `ArcPath::closest_arc` (W5) | `ph2d-vec-scene/src/arc_path.rs` | Método novo, mesmo arquivo — aditivo |
| `draw_text_handle` (W5) | `ph2d-vec-render/src/text_handle.rs` (NOVO) | Arquivo próprio ⇒ isolado; re-exportado no `lib.rs` |
| `VecOverlayPlan.textpath_handle` (W5) | `shells/desktop/src/vec_overlay.rs` | Campo apendado ao struct de plano de overlay (Node-only, testado) |
| `App.vec_textpath_handle_drag` (W5) | `shells/desktop/src/app_state.rs` | Flag runtime-only (não é documento) — apendado |
| `vec_text_ride::handle` + `HANDLE_R_PX` (W5) | `shells/desktop/src/vec_text_ride.rs` | Submódulo no arquivo próprio da feature |

**NADA de schema bumpou, W4 e W5 tampouco** — era o custo que o §5.2 previa e que o componente
opcional dissolveu. `VEC_SCENE_SCHEMA_VERSION` e `PROJECT_SCHEMA` seguem **intactos**.

### O que ainda vem

- **Contratos congelados: NENHUM tocado.**
- `VEC_SCENE_SCHEMA_VERSION` **13 → +1** · `PROJECT_SCHEMA` **+1** — valores se CONTAM na
  integração, não se escolhem aqui.
- Ids novos são **hash de string** (`hash_node_id("vector.textpath.…")`) — a colisão é de *nome*,
  não de número; as strings vão listadas no handoff.
- `VECTOR_SECTIONS` ([`ids/chrome/vector.rs`](../../crates/ph2d-editor-core/src/ids/chrome/vector.rs))
  é **lista compartilhada**: só ADICIONAR, nunca reordenar.
- Foundational tocado: `ph2d-vec-scene` (módulos novos + bump), `ph2d-ecs` (campos em `VecShape`),
  `ph2d-editor-core` (ids), `shells/desktop` — todos sob o protocolo testado do ADR-0107.
- **ADR:** o §5.2 (onde mora o vínculo) e o §5.3 (o caminho-guia não é pintado) são decisões de
  design com dono ⇒ merecem ADR próprio. **O número é PROVISÓRIO** — o próximo livre hoje é 0141,
  e um número escolhido em linha paralela **renumera na integração** (já aconteceu 3× no repo).
