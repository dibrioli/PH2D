# 19 — Auditoria de precisão, ferramenta a ferramenta

> **Ordem:** Enio, 2026-08-20 — *"reduzi o tamanho e apliquei rasterize e caiu para 8. BGRemoval caiu
> para 8. Painter caiu para 8. **auditoria completa com cada tool**"*.
>
> Plano-mãe: [`18_precisao_de_16_bits_nas_sprites.md`](18_precisao_de_16_bits_nas_sprites.md).

---

## §1 — A pergunta única

Para cada ferramenta, uma pergunta e só uma:

> **O valor de um pixel de saída é CALCULADO a partir de mais de um pixel de entrada, ou de
> aritmética sobre a cor?**

- **Não** ⇒ a ferramenta **move, copia ou seleciona**. A precisão pode atravessá-la **exacta**, e
  perdê-la é gratuito.
- **Sim** ⇒ preservar 16 bits exige que a *aritmética* seja de 16 bits — código novo, não plumbing.
  Converter o resultado de volta para cima seria **pior**: o rótulo diria 16 sobre valores que
  passaram por 8.

⚠️ **A resposta é do ALGORITMO, não do nome da ferramenta.** Foi ler os algoritmos que mostrou que
*Upscale* é duas coisas diferentes e que o *BG-Removal* quase não toca em cor.

---

## §2 — A tabela

| Ferramenta | O que faz ao VALOR do pixel | Preserva? | Estado |
|---|---|---|---|
| **Real Size** | **nada** — só repõe a escala do `Transform` a ±1; nunca commita textura | — | ✅ **não pode perder** |
| **Trim Transparency** | escolhe uma janela e copia | sim | ✅ W4-bis |
| **Make Square** | copia + moldura transparente | sim | ✅ W4-bis |
| **Padding** | recorta/copia + moldura transparente | sim | ✅ W4-bis |
| **Upscale · Nearest** | **replica** pixels (`floor(x/f)`, sem filtro) | sim | ⚠️ **achado desta auditoria** |
| **Upscale · Lanczos3 / xBR** | filtra vizinhanças | não | ✅ correto converter |
| **BG-Removal** *(sem despill)* | **copia R, G, B verbatim; calcula só o ALFA** | RGB sim | ⚠️ **achado desta auditoria** |
| **BG-Removal** *(despill ligado)* | reescreve RGB nas bordas macias (`fg = (C − (1−a)·bg)/a`) | não | ✅ correto converter |
| **Rasterize** | assa escala+rotação nos pixels — reamostra | não | ✅ correto converter |
| **Equalize Sizes** | redimensiona | não | ✅ correto converter |
| **Color Equalization** | exposição / temperatura / tint / curvas | não | ✅ correto converter |
| **Painter** | o documento dele é de 8 bits | não | ✅ correto converter |

---

## §3 — Os dois achados, com o mecanismo

### §3.1 — `Upscale · Nearest` replica, não filtra

```rust
// crates/ph2d-tool-upscale/src/algorithm.rs
/// Nearest-neighbour pixel replication. Each destination pixel
/// `(x_d, y_d)` reads the source pixel at `(floor(x_d / factor), floor(y_d / factor))`
/// (no filtering).
```

⚠️ **A mesma ferramenta tem três modos e só um deles é geométrico.** É por isso que a tabela é por
**modo** e não por ferramenta — e por isso a resposta não podia vir do nome.

### §3.2 — O `BG-Removal` é uma operação de ALFA

O compositor copia a cor e escreve só o quarto canal:

```rust
// crates/ph2d-tool-bgremoval/src/algorithm/compose.rs
scratch.output_rgba[base]     = rgba[base];      // R copiado
scratch.output_rgba[base + 1] = rgba[base + 1];  // G copiado
scratch.output_rgba[base + 2] = rgba[base + 2];  // B copiado
scratch.output_rgba[base + 3] = alpha;           // A calculado
```

⚠️ **Duas ressalvas, e as duas importam:**

1. **O despill reescreve RGB.** Quando `params.chroma.despill` está ligado e há cor de fundo
   detectada, as bordas macias passam por `fg = (C − (1−a)·bg) / a`. Esse caminho **não** preserva,
   e a preservação tem de ser condicional ao despill estar **desligado**.
2. **O alfa é calculado em 8 bits** (`0..255`). Preservar dá **RGB exacto e alfa com 256 níveis**.
   Isso **não é uma mentira sobre o formato** (o armazenamento é mesmo de 16 bits), mas é o limite
   da ferramenta e não do formato — e é onde a banda **não** mora, por isso o ganho é real.

⚠️ **E a armadilha do ALFA ASSOCIADO:** o caminho de 8 bits premultiplica em espaço **sRGB**
(`into_premultiplied()` sobre bytes sRGB). Premultiplicar valores **lineares** de 16 bits pelo mesmo
alfa dá um resultado **diferente** — e mais correto. ⛔ Em vez de escolher entre dois erros, o
resultado de 16 bits fica em alfa **RETO** (`premultiplied = false`) e o shader premultiplica em
linear no desenho, que é onde isso pertence.

---

## §4 — O que esta auditoria NÃO propõe

⛔ **Nenhum resampler de 16 bits.** Rasterize, Equalize Sizes e os dois modos filtrados do Upscale
precisariam de um; é código novo com a sua própria conferência, e nenhum deles foi pedido.

⛔ **Nenhuma pilha de cor de 16 bits** para a Color Equalization. Ela já trabalha em `f32` por
dentro (`apply_exposure_linear(&mut [f32; 3], …)`) — o que falta é a **entrada e a saída** falarem
16 bits, e isso é a mesma wave do resampler.

⛔ **O Painter não muda.** O documento dele é de 8 bits por desenho; 16 bits ali é uma frente
própria, e o `docs/Sprite_projeto/18` §6 já a nomeia como decisão de produto.

⛔ **Converter o resultado de volta para cima** em qualquer um dos casos acima. É a opção que
*parece* resolver e é a pior: o Inspector diria `RGBA16` sobre valores que atravessaram 8 bits.
*O rótulo tem de prometer o que o modelo entrega* — esta wave já pagou essa lei três vezes.
