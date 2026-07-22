# 22 — Texto em caminho (pesquisa + plano)

> **Origem:** Enio, 2026-07-22 — *"Creio que ainda não temos paths deformando textos. Faça estudo e
> pesquisa sobre a ferramenta e coloque nos planos para implementar."*
>
> **Estado:** PESQUISA FECHADA · PLANO PROPOSTO · **nada construído**. Aguarda ordem do Enio.
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
- Idem para um texto dentro de um envelope: a próxima tecla apaga a deformação daquele frame.

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

### W0 — O re-cook do texto PRESERVA o que não é geometria ⚠️ *pré-requisito, e é bug vivo*

Corrigir `*p = np` ([`vec_text_object.rs:142`](../../shells/desktop/src/vec_text_object.rs)) para
preservar `effects` (e o que mais não for geometria). Gate **red-first**: aplicar Zig Zag num texto,
editar uma letra, exigir que a pilha continue lá — nasce VERMELHO hoje.

Gate irmão: o mesmo para o texto **dentro de um envelope** (o `create` guarda a fonte autorada como
postcard; confirmar que a edição do texto não a invalida). Se o envelope-sobre-texto for
irreconciliável com a edição viva, a regra do ADR-0129 §325 passa a **valer para texto também, com
gate executável** — nunca com prosa.

**Esta wave fecha sozinha e vale sozinha.** Não depende de nada abaixo.

### W1 — O walker de arco vira porta única
`arc_walk.rs` extraído; `fx_zigzag` delega; gate de **byte-identidade** do zigzag (fixture: círculo
+ forma com quinas, o mesmo par que o `fx_zigzag_tests` já usa).

### W2 — O motor: arco → afim por glifo
Kernel puro em `ph2d-vec-scene` (`text_path.rs`): dado `(caminho, s, advance, altura do glifo,
efeito, alinhamento, flip)` devolve o afim. Cinco braços = as cinco variantes de §3.2. Sem UI, sem
shell. Gates: `Rainbow` num **círculo de raio R** põe cada glifo à distância R do centro com o eixo
radial (oráculo geométrico, não espelho da fórmula); `StairStep` mantém o eixo vertical em toda
posição; `Gravity` num círculo **reduz a Rainbow** (identidade pinada, como o gate Inflate/Layer do
Painter) e **difere** numa elipse — é esse par que dá sentido ao teste.

### W3 — O layout consome o motor
`vec_glyph.rs:80` passa a receber afim; `pen_x` vira `s`. `on_path: None` **byte-idêntico** (gate
com fingerprint, o padrão do repo). O caret (`caret_x_offset:193`) segue o mesmo caminho — senão o
cursor de digitação fica no texto reto enquanto as letras estão na curva.

### W4 — A UI
Seção **Text on Path** no painel (ids novos, strings por i18n, entrada em `VECTOR_SECTIONS`):
botão de vincular/desvincular, `startOffset` (slider+chip com `link_slider_number` +
`mark_chip_no_stepper`, DIRETRIZ §5.2), os 5 chips de efeito, os 4 de alinhamento, flip. Seam test
que **CLICA** cada um (DIRETIVA §2 — pintado ⟹ wirado ⟹ despachado).

### W5 — As alças no canvas
in / center / out, Node-only. É a wave que pode ser cortada sem matar a feature (o `startOffset`
numérico já entrega a capacidade) — e por isso é a última.

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

**O que este plano NÃO faz, de propósito:**
- **Não compensa colisão em curva apertada** (§3.3) — nomeado, com gatilho.
- **Não faz texto em ÁREA** (*Area Type*, texto fluindo dentro de uma forma) — é outra feature, com
  quebra de linha e hifenização; fora de escopo.
- **Não ressuscita o `Twist`** — foi construído, medido em 4 variantes e **cortado** por rasgar a
  geometria ([`fx_warp.rs:6-21`](../../crates/ph2d-vec-scene/src/fx_warp.rs)). Não re-derive.

---

## §8 — Superfície de colisão (para o handoff de integração)

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
