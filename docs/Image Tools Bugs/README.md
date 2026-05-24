# Image Tools — Bugs conhecidos & correções

Registro de bugs (e suas correções) das Image Tools do PH2D — Bg Removal,
Trim, etc. Cada tópico documenta **sintoma → causa raiz → correção →
como evitar regressão**, para que um bug que volta seja diagnosticado
rápido (a correção e o "porquê" ficam aqui, não só no histórico de git).

Índice:
1. [Outline rosa/magenta no Apply do Bg Removal sobre zona protegida](#1-outline-rosamagenta-no-apply-do-bg-removal-sobre-zona-protegida)
2. [Ferramenta de imagem continua ativa com "Image Tools" desligado](#2-ferramenta-de-imagem-continua-ativa-com-image-tools-desligado)
3. [Toasts "Tool → X" com glifo quebrado (seta vira tofu)](#3-toasts-tool--x-com-glifo-quebrado-seta-vira-tofu)

---

## 1. Outline rosa/magenta no Apply do Bg Removal sobre zona protegida


**Gate:** crates/ph2d-editor-core/tests/arch_color_space_typed.rs (heuristic match — refine if wrong)
**Tool:** Bg Removal (Chroma) · **Fase:** Apply (bake) · **Status:** ✅ corrigido (2026-05-20)

### Sintoma

Ao pintar a **máscara de proteção** ("keep" / pincel verde) sobre uma
região e dar **Apply**, a região mantida aparecia com uma **linha
rosa/magenta** contornando a borda — em especial onde a região mantida
encosta no **traço escuro (line-art)** do desenho. A área transparente ao
redor ficava limpa; o rosa traçava o contorno interno da silhueta mantida.

> Importante distinguir de um bug **parecido, porém diferente**, já
> corrigido antes (ver §1.5): o "purple/dark fringe" geral de borda, que
> era de **premultiply** no sampler. Esse continua corrigido. O rosa da
> zona protegida tem **outra causa** (despill) e voltou junto com o
> mascaramento.

### Causa raiz — despill (descontaminação de fundo)

O passo de **despill** (em `algorithm/compose.rs::write_output`) assume que
todo pixel de **alpha fracionário** é um composto "primeiro-plano sobre o
fundo detectado":

```text
C = a·fg + (1−a)·bg   ⇒   fg = (C − (1−a)·bg) / a
```

e reescreve o RGB para `fg`, removendo o halo da cor de fundo nas bordas
anti-aliased (o "despill" clássico de green-screen, generalizado para
qualquer cor de fundo detectada).

Quando uma região de **fundo verde é MANTIDA** pela máscara de proteção,
a fronteira entre esse verde mantido e o **traço escuro** do desenho tem
pixels de alpha fracionário. Ali o despill faz, com `bg ≈ verde`:

- canal **G**: `(G_escuro − (1−a)·G_verde_alto)/a` → **negativo → clamp 0**
- canais **R/B**: `(R_escuro − (1−a)·~0)/a` → **amplificados** (divisão por `a` pequeno)

Resultado: **G zerado + R/B altos = magenta**. Esse magenta é gravado no
texture do Apply → a linha rosa.

**Por que só apareceu com o mascaramento:** antes da máscara, esses pixels
de fronteira eram **removidos** (fundo → transparente), então o despill
neles era inofensivo / invisível. Ao **manter** a região (force-keep),
a fronteira verde↔traço passou a ser visível — e o despill a pinta de
magenta.

### Correção

`fix(bgremoval): skip despill on protected pixels` (commit `7b3b2b2`).

O despill agora **pula qualquer pixel pintado na máscara de proteção**
(`protect[i] > 0`). Princípio: uma região "keep" deve manter a **cor
verdadeira** — nunca ser descontaminada (a premissa "fg sobre bg" não vale
ali, pois o usuário quer justamente preservar o pixel como está).

```rust
// algorithm/compose.rs — laço de despill
for i in 0..n {
    if protect.is_some_and(|pm| pm[i] > 0) {
        continue; // pixel protegido: preserva cor real, sem despill
    }
    // ... despill normal nos demais pixels fracionários ...
}
```

### Como evitar regressão

- Teste determinístico: `compose::tests::despill_skips_protected_pixels_no_magenta_fringe`
  — um pixel escuro de alpha fracionário sobre fundo verde **muda** quando
  não-protegido (despill roda) e **fica intacto** quando protegido.
- Regra geral: **qualquer passo que assume "fg sobre o fundo detectado"
  (despill, edge-bleed, grow/shrink que reintroduz cor) deve respeitar a
  máscara de proteção.** Um pixel marcado como "keep" é verdade
  fotográfica do usuário; nenhum heurístico de fundo pode reescrevê-lo.

### §1.5 — Não confundir com o fringe de premultiply (já corrigido)

Bug **distinto**, corrigido em `8616383` (`edge-perfect Apply`): a borda
anti-aliased ganhava um "purple/dark fringe" porque o texture era gravado
em **straight-alpha** e o shader de sprite premultiplicava **depois** da
amostragem bilinear (≠ do preview Vello, que premultiplica **antes**).

Correção (ainda ativa): o Apply grava o texture **premultiplied** e marca
`Sprite.premultiplied` para o shader **pular** o premultiply pós-amostra;
o readback de re-segmentação desfaz o premultiply. Mais o **edge-bleed**,
que preenche só o colar totalmente transparente (`alpha == 0`) com a cor
da borda visível, nunca a faixa de AA (que é o traço).

Se o fringe **roxo/escuro** (não magenta) voltar, suspeite **deste**
caminho (bake premultiplied / flag no shader), não do despill.
Ponteiros: `shells/desktop/src/hero_intents/image_edit.rs` (bake
`into_premultiplied()`) e `ph2d-render` premul/flag por instância.

---

## 2. Ferramenta de imagem continua ativa com "Image Tools" desligado


**Gate:** crates/ph2d-editor-core/tests/arch_mode_has_reconcile.rs (heuristic match — refine if wrong)
**Tool:** Bg Removal / Padding (qualquer tool stateful de imagem) ·
**Fase:** ativação/desativação · **Status:** ✅ corrigido (2026-05-21,
commit `3ef9190`)

### Sintoma

Com o botão **Image Tools** visualmente **desligado** (vê-se o gear, não
os pills), o painel do Padding (ou o preview do Bg Removal com o fundo já
removido) **continuava aparecendo** e respondendo. Parecia que "clicar no
Config rodava o Bg Removal e abria o Padding" — mas o que se via era um
tool **órfão**, ativado antes e nunca desligado.

### Causa raiz — `mode_on` e o tool ativo do `ToolRegistry` desacoplados

O toggle Image Tools só flipa `hero.image_edit.mode_on`
([`chrome/image_tools_toggle.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/image_tools_toggle.rs)).
**Nada desativava o tool ativo** quando o modo era desligado. Como o
`padding_bridge`/`bgremoval_preview` decidem a visibilidade do painel por
`tools.active().id() == "padding"/"bgremoval"` — **sem olhar `mode_on`** —
o painel/preview persistiam enquanto o tool ficasse ativo.

Diagnóstico medido (probe env-gated `PH2D_UIDBG`): a captura do repro
mostrou o painel do Padding aberto **sem nenhum `ActivatePadding`** e com
`image_tools_mode_on=false` → prova do tool órfão. Um *guard* só na
ativação (tentativa anterior) NÃO resolve: o tool **já estava ativo**.

### Correção

`fix(image-tools): mode OFF deactivates active image tools` (`3ef9190`),
em [`render_loop/mod.rs`](../../shells/desktop/src/render_loop/mod.rs):

1. **Reconciliação por frame, ANTES dos bridges:** se `!mode_on` e o tool
   ativo é image tool, volta pro tool default e dropa o preview do Bg
   Removal. Invariante: **Image Tools OFF ⟹ nenhum tool de imagem ativo**,
   não importa como ficou ativo.
2. **Ativação gateada em `mode_on`:** image tools só ativam com o modo
   ligado (cobre o atalho Digit3 e qualquer caminho stale, sem flicker de
   1 frame nem toast espúrio).

#### 2.b — O segundo caminho: a Tool Palette (`32460b9`)

Depois do fix acima, ainda aparecia a toast "Tool · Padding" ao clicar no
canto superior direito com o modo off — **"só a mensagem"** (o reconcile
matava o tool, mas o toast já tinha saído). Causa: a **tool palette** da
zona top-right (`tool_palette_rects`) é uma **segunda via de ativação** —
clicar um ícone chama `tools.set_active` **direto**, fora das pills e do
guard. Pior: no caminho hero o paint da palette não roda, mas o hit-test
roda → **zona de clique invisível**.

Fix em duas partes:
1. **Filtrar os image tools da palette quando o modo está off** (`32460b9`),
   em paint E hit (`palette_visible_tool_indices`, mesmo mapeamento nos
   dois → sem drift de índice). Off ⟹ sem ícone.
2. **Hit-test SÓ onde a palette é pintada** (`952dc0c`): a palette só pinta
   no caminho legado sem-hero (demo); no editor (hero) não pinta, mas o
   hit rodava sempre. `Zone::TopRight` é a metade direita da faixa do
   toolbar — exatamente onde o topbar pinta o gear (Config) — então clicar
   no Config caía também num slot invisível da palette e trocava o tool
   ("Tool · Move"). Gate do hit em `hero_screen.is_none()` (mesma condição
   do paint) → no editor o top-right é só do topbar. **Lição:** hit-test e
   paint do mesmo widget têm que ser gateados pela MESMA condição; hit sem
   paint = zona de clique invisível.

**Definição canônica de "image tool" (data-driven):** `is_image_edit_tool`
em [`app_state.rs`](../../shells/desktop/src/app_state.rs) responde pela
**presença no cluster `"image_tools"` do manifest** — NÃO uma lista fixa
de ids. Cobre Bg Removal, Padding, Trim, Make Square, Real Size e
**qualquer tool futura** registrada nesse cluster, automaticamente.

### Como evitar regressão

- **Regra de ouro:** o estado de um modo de UI (`mode_on`) e o estado que
  ele controla (tool ativo, painel, preview) **NÃO podem viver
  desacoplados**. Quem liga um modo é responsável por **desligar tudo** que
  ele expõe quando o modo cai — e o melhor lugar é uma **reconciliação por
  frame** (estado derivado), não um guard pontual no caminho de clique.
- Um *guard* na ativação trata "não deixar ligar"; ele **não** trata "já
  está ligado". Bugs de "fica ativo indevidamente" pedem reconciliação de
  estado, não guard.
- **Enumere TODOS os caminhos de ativação.** Uma feature pode ser ligada
  por mais de uma via (pills da TopBar, **tool palette**, atalho de
  teclado, bus action). Gatear só uma deixa o bug vivo pelas outras (foi o
  que aconteceu: pills gateadas, palette não). Ao gatear um modo, faça um
  grep por TODOS os `set_active`/push de action do subsistema e cubra cada
  um — ou, melhor, centralize num predicado/reconciliação única.
- **Predicado de pertencimento é data-driven, não lista fixa.** "É um image
  tool?" = está no cluster `"image_tools"` do manifest. Nada de
  `id == "bgremoval" || id == "padding"` espalhado — um helper único
  (`is_image_edit_tool`) consultando o cluster cobre as tools futuras de
  graça.
- Ponteiros: `render_loop/mod.rs` (bloco "Image Tools OFF is
  AUTHORITATIVE"), `render_loop/padding_bridge.rs` +
  `render_loop/bgremoval_preview.rs` (visibilidade por tool ativo),
  `app_state.rs` (`is_image_edit_tool` / `palette_visible_tool_indices`),
  `input_dispatch.rs` + `render_loop/mod.rs` (filtro da palette).

---

## 3. Toasts "Tool → X" com glifo quebrado (seta vira tofu)


**Gate:** crates/ph2d-editor-core/tests/no_tofu_glyphs.rs (heuristic match — refine if wrong)
**Tool:** todas (troca de ferramenta) · **Status:** ✅ corrigido
(2026-05-21, commit `b62e0c5`)

### Sintoma

Ao trocar de ferramenta aparecia na tela uma toast "Tool **⎕** Padding" —
um **quadrado tofu** no lugar da seta.

### Causa raiz

As toasts usavam **U+2192 (→)**, que **não está** na cadeia de fallback da
fonte bundled (Inter). Caractere fora da fonte = tofu. Mesma classe dos
glifos U+2318/U+21B5 do topbar já conhecidos.

### Correção

Trocado por **U+00B7 (·)**, que está na fonte (o topbar já usa em "Save ·
Cmd+S"). Sites: `input_handlers.rs`, `input_dispatch.rs`,
`render_loop/mod.rs`.

### Como evitar regressão

**Nenhuma string de UI deve conter glifo fora do conjunto Inter bundled.**
Setas/símbolos (→, ⌘, ↵, ✕, ▸) viram tofu. Use ASCII ou os poucos
não-ASCII comprovadamente in-font (`·` U+00B7). Vide
[`docs/UI_Bugs/README.md`](../UI_Bugs/README.md) (seção Tipografia).
