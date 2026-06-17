# 🔬 HANDOFF — BGRemoval preview overlay halo (parity vs Apply)

**Data:** 2026-05-26 noite
**Para:** próxima LLM (auditoria adversarial pós-Fix-C)
**Status:** Fix C aplicado (commit `7cb95d4`); user reportou **"diminuiu mas não curou"**. Causa raiz parcialmente endereçada. Falta nova lente.

---

## 1. Sintoma

Enquanto o BGRemoval tool está ativo, o on-canvas overlay (Vello compositor) mostra uma **"linha clara contornando a forma"** ao redor da silhueta do sujeito. No instante em que o Apply é disparado e a textura da sprite é substituída pelo resultado processado (sprite shader wgpu), o halo **desaparece** — contorno limpo.

Screenshot do user (commit `7cb95d4` ainda mostra resíduo): halo claro visível no chapéu/casaco do personagem; desaparece após Apply.

**Constraint operacional:** o user pediu literalmente "ferramenta atuando na imagem real em tempo real". Manter o overlay como mecanismo de live-preview, fazer ele virar **byte-idêntico ao Apply** visualmente.

---

## 2. Histórico das tentativas (timeline)

Todos os commits abaixo são locais (não-pushados). 14 commits sobre o overlay nas últimas horas:

| Commit | Intent | Resultado |
|---|---|---|
| `b3c15fa` | Aplicar transform completo (rotation+scale+anchor+camera) ao overlay via `draw_image_rgba_transformed` com Affine completo | Resolve drift geométrico; halo persiste. |
| `968e0f6` | `PREVIEW_MAX_DIM = u32::MAX` — preview pipeline roda em resolução source idêntica a Apply | Output bytes da compose pipeline agora idênticos; halo persiste. |
| `4084ee4` | Forçar `ImageQuality::Medium` (bilinear) em vez de Bicubic — eliminar overshoot do kernel bicúbico | Sem mudança visível; halo persiste. |
| `4eabab4` | Premultiplicar RGBA no Rust side via `premultiply_rgba8` (mesma fn do Apply path) + passar `AlphaPremultiplied` ao Vello — equalizar rounding de sub-pixel premul | Sem mudança visível; halo persiste. |
| **`7cb95d4`** ⬅ HEAD | **Gamma-correct premul**: novo `premultiply_rgba8_in_linear` que sRGB-decode → premul em linear → sRGB-encode. Endereça causa identificada pelos 3 agentes. | **Diminuiu mas não curou.** |

---

## 3. Diagnóstico convergente dos 3 agentes (sessão 2026-05-26)

Lançados 3 agentes `general-purpose` em paralelo com lentes distintas:
- **Agente A — color space / gamma pipeline**
- **Agente B — texture upload + framebuffer flow path**
- **Agente C — hands-on pixel-math arithmetic walkthrough**

**Todos convergiram na mesma causa raiz:** gamma-space mismatch entre os dois paths.

### Os dois paths em detalhe (file:line citados pelos agentes)

| Path | Texture format | Sample behavior | Bilinear space | Compose |
|---|---|---|---|---|
| **Sprite (Apply)** | `Rgba8UnormSrgb` ([individual.rs:326](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/individual.rs#L326)) | Hardware sRGB→linear no `textureSample` ([sprite.wgsl:82](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/shaders/sprite.wgsl#L82)) | **LINEAR** | `PREMULTIPLIED_ALPHA_BLENDING` em `Rgba16Float` game_rt ([pipeline.rs:101](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/pipeline.rs#L101)) |
| **Vello (overlay)** | `Rgba8Unorm` ([vello_pass.rs:302](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/vello_pass.rs#L302)) | Raw bytes / 255 sem decode | **sRGB-as-linear** (gamma-incorreto) | Porter-Duff "over" em sRGB-as-linear ([compositor.wgsl:85](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/shaders/compositor.wgsl#L85)) |

### Walkthrough de pixel concreto (Agente C)

Edge pixel com `alpha=128, RGB=(80,80,80)` straight + adjacente transparente bled `(80,80,80,0)`:

**Apply path:**
1. `premultiply_rgba8` byte-space: A → `(40,40,40,128)`, B → `(0,0,0,0)`.
2. Upload `Rgba8UnormSrgb`. Hardware decode no sample: A.r → `srgb_decode(40/255) ≈ 0.0213` linear.
3. Bilinear midpoint A↔B em linear: `(0.0107, 0.0107, 0.0107, 0.251)`.
4. Sprite shader → game_rt linear → compositor → encode sRGB → swap.

**Overlay path (pré-Fix-C):**
1. `premultiply_rgba8`: mesmos bytes. A=`(40,40,40,128)`.
2. Upload `Rgba8Unorm`. Sample raw: A.r → `40/255 = 0.157` (treated as linear!).
3. Bilinear midpoint A↔B: `(0.0784, 0.0784, 0.0784, 0.251)`.
4. Vello fine.wgsl unmul antes de store → bytes ≈ `(80,80,80,64)`.
5. Compositor lê como "sRGB designer-space": vello.rgb = 0.314 sRGB, blend em sRGB-space.

**Divergência:** ~7× brightness no mesmo edge pixel (`0.0213` linear vs `0.157` "fake linear"). Daí o halo claro.

### Fix C aplicado (commit `7cb95d4`)

Novo `ph2d_render::premultiply_rgba8_in_linear` ([premul.rs:142+](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/premul.rs#L142)):
```
sRGB-decode each channel → multiply by linear alpha → sRGB-encode
```

Bytes resultantes são sRGB-encoded mas semanticamente representam `rgb_linear * a_linear` (premul canônico em linear). Overlay agora chama essa fn em vez de `premultiply_rgba8`. Apply path **intacto** (continua com byte-space premul).

**User feedback:** "diminuiu mas não curou". → Fix C atenuou mas não eliminou.

---

## 4. Estado atual + hipóteses restantes

### Hipótese H1 — Vello unmul corrompe a banda pré-encoded

`fine.wgsl:1278` (Vello shaders) **divide RGB por alpha antes de store** (`AlphaPremultiplied → straight no store`). Mesmo passando bytes gamma-correct premul, Vello os trata como "sRGB premul" e desmultiplica em sRGB-space. O resultado straight no store NÃO é sRGB-encode de `rgb_linear / a_linear`.

Especificamente: para Vello, dado byte premul-em-linear-encoded-sRGB (R=62, A=128 do exemplo Agente C), Vello faz:
- raw R/A = 62/128 ≈ 0.484 → store byte 124.
- Compositor lê 124 como "designer sRGB" = 0.486. Blend over game_srgb.

Mas o sprite path teria dado: linear 0.0107 / 0.251 = 0.0426 linear → linear_to_srgb ≈ 0.241 sRGB = byte 61. NÃO 124.

**→ Vello internal unmul é o próximo culprit. Está revertendo a conversão linear que fizemos.**

### Hipótese H2 — Compositor blend equation diverge

[compositor.wgsl:1-19](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/shaders/compositor.wgsl#L1-L19) documenta a escolha deliberada de blend em "designer space" (sRGB-as-linear) para parity com Figma/browsers (UI tokens). Sprite path blend em linear (game_rt Rgba16Float). Mesmo com inputs gamma-correct, os dois blend operators dão visuais diferentes em transições parciais-alpha.

### Hipótese H3 — Arquitetural: usar sprite pipeline pro preview (Agente A Fix 2)

Em vez de Vello, criar uma `Individual` texture transient + render via sprite shader. Mesmo pipeline byte-por-byte. Mais invasivo (lifecycle: criar na ativação, destruir na desativação, sync com params_dirty). Risk médio-alto. Mas garante parity.

---

## 5. Sugestão de nova auditoria (próxima LLM)

### Lentes recomendadas (não-redundantes vs as 3 anteriores)

**Lente D — Vello fine.wgsl unmul reverse-engineering:**
- Ler `vello_shaders` crate em `~/.cargo/registry/...vello_shaders-0.8.0/shader/fine.wgsl:1253-1281`.
- Confirmar EXATAMENTE como Vello desmultiplica `AlphaPremultiplied` antes de armazenar no atlas.
- Computar a sequência completa: bytes_in → Vello internal premul/unmul → atlas store → compositor sample → blend.
- Verificar se há uma forma de **passar bytes que sobrevivem essa transformação como gamma-correct**.
- Output esperado: definitiva resposta — "Fix C completa não é possível com a API Vello atual" OU "passar bytes em formato X faz o round-trip preservar gamma".

**Lente E — Compositor blend space override:**
- Ler [compositor.wgsl](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/shaders/compositor.wgsl) inteiro.
- Avaliar se há uma forma de **gate por feature** (e.g., uniform flag) para fazer o blend do bgremoval preview em linear, mantendo UI tokens em sRGB.
- Custos: 1 uniform flag, 1 branch no shader. Cheap. Mas precisa ser threaded do shell até o shader binding.

**Lente F — Sprite pipeline para preview (Agente A Fix 2 implementado):**
- Mapear o que precisa ser tocado: `IndividualTextureStore` (criar transient slot), sprite extract (NÃO suprimir esse entity quando tool ativa), `drive_preview_cache` substituído por "upload preview RGBA pra individual texture override".
- Estimar LOC + risk. Provavelmente 3-5 arquivos, ~150 LOC.
- **Vantagem definitiva:** byte-por-byte parity. Mesmo shader, mesmo blend, mesmo gamma. Não há como divergir.

### Protocolo da auditoria

1. **Lançar 2-3 agentes em paralelo** com as lentes D, E, F.
2. Cada agente reporta sob 400 words com file:line citations.
3. Cross-reference e escolher fix.
4. Implementar. Smoke do Enio.

### NÃO recomendo (já esgotado)

- Mexer em `compose.rs::bleed_edges` (alpha=0 sempre tem RGB=0 após premul — Agente C confirmou).
- Mexer em filter mode (bilinear já forçado).
- Mexer em `PREVIEW_MAX_DIM` (já em u32::MAX).
- Cap-bust em ADR-0040/0041 (contratos congelados).

---

## 6. Arquivos relevantes (cheat-sheet)

### Tool side
- [`crates/ph2d-tool-bgremoval/src/tool.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-tool-bgremoval/src/tool.rs) — `run_canvas_preview`, `run_full_resolution`, `prepare_combined_protect_*`, `cached_auto_protect_source`.
- [`crates/ph2d-tool-bgremoval/src/algorithm/compose.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-tool-bgremoval/src/algorithm/compose.rs) — `write_output`, `bleed_edges`, `force_keep_protected`. **Não tocar.**
- [`crates/ph2d-tool-bgremoval/src/algorithm/silhouette.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-tool-bgremoval/src/algorithm/silhouette.rs) — Detect-subject + soft falloff. **Não tocar.**

### Render side
- [`crates/ph2d-render/src/premul.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/premul.rs) — `premultiply_rgba8` (byte-space) + `premultiply_rgba8_in_linear` (gamma-correct, novo).
- [`crates/ph2d-render/src/individual.rs:326`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/individual.rs#L326) — sprite texture format `Rgba8UnormSrgb`.
- [`crates/ph2d-render/src/vello_pass.rs:302`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/vello_pass.rs#L302) — Vello intermediate `Rgba8Unorm`.
- [`crates/ph2d-render/src/shaders/sprite.wgsl`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/shaders/sprite.wgsl) — sprite shader (sample + premul check).
- [`crates/ph2d-render/src/shaders/compositor.wgsl`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/shaders/compositor.wgsl) — blend final game_rt + vello_rt. **Lente E olha aqui.**
- [`crates/ph2d-render/src/pipeline.rs:101`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-render/src/pipeline.rs#L101) — `PREMULTIPLIED_ALPHA_BLENDING` para sprite.

### Vector / Vello wrapper
- [`crates/ph2d-vector/src/scene.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-vector/src/scene.rs) — `draw_image_rgba_premultiplied_transformed` (nova fn passing `AlphaPremultiplied`).
- `~/.cargo/registry/src/index.crates.io-*/vello_shaders-0.8.0/shader/fine.wgsl:1253-1281` — Vello fine kernel. **Lente D ler aqui.**

### Shell
- [`shells/desktop/src/render_loop/bgremoval_preview.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/shells/desktop/src/render_loop/bgremoval_preview.rs) — bridge, overlay paint, `sprite_image_to_screen_affine`, current `premultiply_rgba8_in_linear` call.
- [`shells/desktop/src/render_loop/mod.rs:223-231`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/shells/desktop/src/render_loop/mod.rs#L223) — `bgremoval_preview_entity` suppression gate (sprite extract).
- [`shells/desktop/src/hero_intents/image_edit/bgremoval.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/shells/desktop/src/hero_intents/image_edit/bgremoval.rs) — `drain_bgremoval` (Apply path; `into_premultiplied` byte-space).
- [`shells/desktop/src/hero_intents/texture_edit.rs:105`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/shells/desktop/src/hero_intents/texture_edit.rs#L105) — `commit_edited_texture` (acquire_individual).

---

## 7. Estado do branch + testes

- **HEAD:** `7cb95d4`.
- **Local commits ahead of origin/main:** ~50+ (não push).
- **Testes verdes:** `ph2d-tool-bgremoval` 131; `ph2d-render premul` 8; `ph2d-vector` 4; shell compile clean.
- **Working tree:** alguns arquivos modified por outras sessões paralelas (Painter, imageio). **Não tocar nessas pastas** sem confirmar com o Enio.

---

## 8. Confiança + nota final

Os 3 agentes da rodada anterior fizeram análise profunda e o diagnóstico do gamma-mismatch é sólido. O Fix C endereça parte da causa (gamma-correct premul) mas o Vello unmul/compositor blend ainda introduzem error residual.

**Mandato Enio padrão-ouro:** sem gambiarras, sem "v1 que dá pro gasto". Se a Lente D confirmar que Vello fundamentalmente impede parity exata, **Lente F (sprite pipeline pro preview)** é o caminho arquitetural correto. O custo de implementação é justificado pelo requisito visual de "imagem real em tempo real".

Próxima LLM: leia este handoff, leia os 3 reports dos agentes da rodada anterior (estão acima na conversa do user — pedir ao Enio se precisar), rode as 3 lentes novas em paralelo, escolha o fix com user input + commit + smoke.

Boa caçada.
