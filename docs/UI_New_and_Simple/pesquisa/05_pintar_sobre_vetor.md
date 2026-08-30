# «Pintar sobre vetor» — a medição da feature que o Enio nomeou (2026-08-30)

> **Enio, 2026-08-30:** *"uma forma vetorial deveria ter um modo Pintar também, com o módulo
> painter atuando sobre o vector. Contudo esse feature ainda não existe."*
>
> ⚠️ Ele tem razão sobre a ausência. Mas a medição mudou **o tamanho e a forma** do buraco, e
> **refutou a minha própria hipótese** de que era o mesmo trabalho do *Flip sai do Flip*.

---

## §1 — A hipótese que eu levantei, e por que ela está ERRADA

Levantei que «pintar sobre vetor» fosse o mesmo buraco do
[*W-Saída*](../../Flip/01_plano_waves.md) do Flip, porque as duas entidades têm a mesma forma:

| objecto | componentes | tem pixels? |
|---|---|:--:|
| Flip | `Transform + Name + FlipObjectRef + RootOrder` | ⛔ não |
| **vetor** | `Transform + Name + VecPathRef` | ⛔ não |

⇒ **O substrato é o mesmo** (um objecto que o Painter não alcança porque não é feito de pixels).

⛔ **Mas a CURA não é.** O W-Saída.T1 é um **assado de sentido único** — *"um quadro composto vira
PIXELS"*, para exportar, empacotar em folha, e dar uma camada ao Painter. É uma **saída**.

O que o Enio pediu é o contrário: a tinta tem de **VIVER na forma** e sobreviver a editar a
geometria. Assar o vetor em pixels **mataria o vetor** — deixaria de ser editável, que é a razão
de existir do módulo.

⭐ **O precedente certo é o Blender *Texture Paint*, não uma exportação:** pinta-se numa **textura
mapeada** sobre o objecto, e a geometria continua geometria. É, aliás, literalmente um dos 13
modos do Blender, e um dos que só a malha tem.

⇒ **Mesma família, cura diferente.** *Um substrato partilhado não implica um trabalho partilhado.*

---

## §2 — ⭐⭐ E metade JÁ EXISTE — mas não a metade que se imagina

### 2.1 — A EXIBIÇÃO já ship-a hoje

`Paint::Pattern(Box<PatternFill>)` (`ph2d-vec-scene`) é a 5.ª variante da tinta de preenchimento, e
`PatternSource::Image(AssetId)` deixa a arte ser **uma imagem do projecto**.

E há o modo que interessa (`ph2d-vec-pattern`):

```rust
pub enum PatternMode {
    Tile,    // repete (Extend::Repeat) — o caminho comum
    Mirror,  // espelha (Extend::Reflect)
    Clamp,   // ⭐ "estica a borda (Extend::Pad): UMA CÓPIA SÓ, e o resto é a orla dela"
}
```

⇒ **`Pattern { source: Image(id), mode: Clamp, size: <a caixa da forma> }` é, hoje, uma imagem
única mapeada dentro de uma forma vetorial.** A metade que mostra o resultado está pronta e
testada.

### 2.2 — E a ponte do Painter **não exige** um `Sprite`

`ph2d_ecs::PaintedDoc(pub u32)` é a única ponte, e o doc-comment dela é explícito:

> *"O que ela NÃO faz: não põe pixels no ECS. O `PainterTool` continua dono."*

O Painter é dono de `layers + images + heights + covers` e chaveia por `Entity::to_bits()`;
o `PaintedDoc` é só a **identidade estável** que sobrevive ao save/load. ⇒ **nada no contrato
obriga o alvo a ser uma sprite.** Qualquer entidade pode carregar um documento do Painter.

---

## §3 — O que falta, nomeado

| # | falta | porquê é trabalho |
|---|---|---|
| **1** | O Painter **escrever** naquela imagem | hoje ele produz um documento próprio; ninguém liga a saída dele a um `AssetId` que um `PatternFill` leia. É a ida-e-volta `documento ⟺ asset` |
| **2** | ⭐⭐ O mapa ser **da FORMA**, não do mundo | `PatternFill` posiciona-se por `origin` / `angle` / `size` em **world-space**. Editar a forma **não leva a tinta com ela** — a arte fica onde estava. É o análogo do UV do Blender, e é o item **caro** |
| **3** | A resolução e o enquadramento | *"um traço vive num campo infinito; o que se exporta é um rectângulo"* (W-Saída.T1). Aqui a forma **dá** o rectângulo (a caixa dela), mas ela muda de tamanho — e a decisão *«re-amostrar ou esticar?»* é de produto |
| **4** | O modo em si | o selector no cabeçalho da área, e o Painter a aceitar um alvo que não é sprite |

⭐ **O item 2 é o que decide o preço.** Sem ele, isto é «uma imagem colada dentro de uma forma» —
que já temos. Com ele, é *Texture Paint*: a tinta é uma propriedade da forma e acompanha-a.

⚠️ E ele tem um irmão já resolvido no repo, que é o sítio por onde começar a ler: o
`FieldProfileSource { path, level }` do módulo 3D Modeling mantém um vínculo **vivo** desenho→peça
— *editar a curva remodela a peça*. É a mesma pergunta («o que acontece à coisa derivada quando a
fonte muda?») com outra resposta.

---

## §4 — O que isto muda na tabela de Modos

⛔ **O modo `Pintar` do vetor NÃO entra na lista de hoje** — um modo declarado que não pinta é um
**controlo morto**, e o `CLAUDE.md` §5.0 tem duas espécies dele catalogadas com o custo medido.

⇒ **A tabela de modos declara o que o tipo consegue FAZER hoje;** o `Pintar` do vetor entra no
mapa da estrada, com este documento como endereço.

⚠️ E o Blender concorda por construção: *"Which modes are available depends on the object's type"*
— nele um modo indisponível simplesmente **não aparece**. Não há modo cinzento.

---

## §5 — Fontes medidas

- `crates/ph2d-vec-scene/src/lib.rs` — `Paint::Pattern`, `PatternFill`, `PatternSource`
- `crates/ph2d-vec-pattern/src/lib.rs:157` — `PatternMode { Tile, Mirror, Clamp }`
- `crates/ph2d-ecs/src/painted_doc.rs` — `PaintedDoc`, e o que ela declara não fazer
- `shells/desktop/src/project_painter.rs:41` — `stamp_painted_docs`, a ponte viva
- `docs/Flip/01_plano_waves.md:274` — W-Saída, o assado de sentido único
- `referencias/blender-manual/manual/editors/3dview/modes.rst` — Texture Paint como modo só-malha
