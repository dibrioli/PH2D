# 09 — Sampling, Material & Blend

## 9.1 Texture Filter — per-node hierárquico (Godot pattern)

Filter mode de amostragem da textura. **Hierárquico:** Component `TextureFilter(FilterMode)` no entity sobrescreve herança do ancestral.

```rust
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FilterMode {
    Inherit,         // herda do ancestral (default em Components opcionais)
    Nearest,         // sem filtering; ideal pixel-art
    Linear,          // bilinear; ideal UI vetorial / smooth
    NearestMipmap,   // mip + nearest within mip
    LinearMipmap,    // mip + linear within mip (trilinear)
    NearestAniso,    // anisotropic + nearest
    LinearAniso,     // anisotropic + linear
}
```

**Default fallback:** quando NENHUM ancestral define filter, usa Project Setting global (Project Settings → "Default Texture Filter"). Pixel-art games configuram global=Nearest; HD games configuram global=Linear.

**Razão per-node:** pixel-art mundo + UI vetorial num jogo só. Mundo usa `Nearest`; UI usa `Linear`. Sem feature, força usuário a hackar global toggle ou aplicar Material override por sprite (caro). Godot acerta com per-node; Unity força via material.

**Implementação:** `RenderInstance` carrega `filter_mode: u8` (ou bitfield). Renderer agrupa instances por filter_mode antes do dispatch (mantém batching).

## 9.2 Texture Repeat — per-node

Idem filter — Component `TextureRepeat(RepeatMode)` no entity sobrescreve herança.

```rust
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RepeatMode {
    Inherit,
    Disabled,    // clamp em [0..1]; pixels fora viram transparent (ou clamp da borda)
    Enabled,     // repeat tile (wrap)
    Mirror,      // mirror-repeat (alterna)
}
```

**Caso de uso:** background scrolling com UV-shift (textura repete enquanto sprite "anda"). Tiling sem TileMap.

## 9.3 Anti-halo / Edge Filtering

Asset-level flag, NÃO Component runtime. Configurado no `SpriteAtlas` ou no asset cooker.

Função: pinta pixels totalmente-transparentes da borda do sprite com a cor do vizinho-opaco-mais-próximo. Evita halo escuro em mipmaps + bilinear filtering.

GameMaker chama "Edge Filtering"; Unity chama "Alpha is Transparency" (com efeito similar mas mais limitado). PH2D usa nome explícito.

Inspector mostra como `read-only label` na seção Sampling (informativo): "Anti-halo: enabled (atlas-level)". Usuário muda no atlas config, não no sprite.

## 9.4 Material — slot

Cada sprite tem um `Material` slot. Default = "sprite-default" (WGSL shader bundled em `ph2d-render`).

Component `Material(MaterialRef)` no entity override default. Permite shader custom per-sprite (dissolve, ghost, glow custom — quando NÃO usar FX chain).

**Nota:** **FX chain composável** (Outline, DropShadow, PaletteSwap, Dither, Glow, etc.) NÃO está aqui. Vai pro módulo Shader FX dedicado, futuro (vide [12_fora_de_escopo.md](12_fora_de_escopo.md)). Material slot continua existindo para casos legítimos de shader único custom.

## 9.5 Use Parent Material — batching brutal ⭐⭐

Component `UseParentMaterial` (zero-size marker) no filho. Quando presente, filho **ignora seu próprio Material** e usa o do pai.

**Performance:** 10k sprites filhos compartilhando 1 material instance = 1 draw call (batching pelo material handle). Sem isso, mesmo shader em material instances distintos = batches separados (alguns paths).

**Caso de uso típico:** tilemap renderizado como milhares de sprites filhos de um único entity; partícula em massa; foliage; UI list de N itens com mesma look.

**Limitação:** parâmetros de shader são do pai. Para variar uniform por filho, usar `InstanceShaderParams` Component (próxima seção).

## 9.6 Instance Shader Params ⭐⭐

Component `InstanceShaderParams(SmallVec<[(Box<str>, InstanceParamValue); 8]>)` no entity. Override de uniforms por-instância SEM clonar Material.

**Tipos canônicos** (corrigido pós-audit — `StringKey`/`Value` originalmente vapores):
- **Key:** `Box<str>` — string heap-allocated, owns the uniform name; sem dep externa (`SmolStr`/`CompactString`). **Cap ≤ 32 bytes UTF-8 enforced on setter** (Lens E E2 fix); rejeita oversize via `try_insert` retornar `Err(SpriteError::ShaderParamKeyTooLong)`.
- **Value:** `InstanceParamValue` enum.

**API canon (Lens E E2 fix):**

```rust
impl InstanceShaderParams {
    /// Cap obrigatório: ≤ 32 bytes UTF-8. Sem `pub` field direto.
    pub fn try_insert(&mut self, key: &str, value: InstanceParamValue) -> Result<(), SpriteError> {
        if key.len() > 32 { return Err(SpriteError::ShaderParamKeyTooLong); }
        // ... insert into SmallVec ...
        Ok(())
    }
}
```

Gate `instance_shader_params_key_length_byte_cap` em [11_arch_gates_e_caps.md §11.2](11_arch_gates_e_caps.md): test `try_insert` com 33 bytes UTF-8 retorna `Err`.

```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum InstanceParamValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color([f32; 4]),
    Int(i32),
    Texture(TextureRef),
}
```

**Pré-requisito:** shader do material declara `instance uniform <type> name;`. PH2D faz upload do uniform per-instance no instance buffer (mesmo path do per-vertex tint).

**Caso de uso:** 10k inimigos com hue distinto (mesmo material, hue variando per-instance); 100 partículas com intensity variável; 50 buttons com cor de tema variável.

**Inspetor UX:** key-value editor inline. Adicionar par "name + value" via dropdown (lista os uniforms declarados pelo shader atual). Sem entrada livre — só uniforms declarados.

```
Instance Shader Params:
  ┌─ hue_shift (f32) ─────── 0.5  [─────●─────]
  ├─ intensity (f32) ─────── 1.2  [────────●──]
  └─ tint_overlay (color) ── #FF8800  [█]      
  [+ Add param]
```

## 9.7 Blend Mode

Component `BlendMode(Mode)` no entity. Default Mix.

```rust
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    Mix,             // alpha-blend padrão (default)
    Add,             // soma RGB; magias, fogo, glow
    Sub,             // subtrai; sombras, "drain"
    Mul,             // multiplica; máscaras, lentes
    Screen,          // (1 - (1-src)*(1-dst)); flash, lens flare
    PremultAlpha,    // pré-multiplica RGB×alpha; composite chains
}
```

**Mix** = padrão (`src.rgb * src.a + dst.rgb * (1 - src.a)`).
**Add** = `src.rgb + dst.rgb` (alpha controla intensidade).
**Sub** = `dst.rgb - src.rgb` (alpha controla).
**Mul** = `src.rgb * dst.rgb` (mascara iluminação).
**Screen** = `1 - (1-src.rgb)(1-dst.rgb)` (luminescente).
**PremultAlpha** = espera RGB já × alpha; pula multiplique pré-blend.

6 modos cobrem ~99% dos casos. Modos exóticos (Difference, Exclusion, Hue/Sat/Color/Luminosity Photoshop-style) **fora de escopo Inspector v2** — vão pro módulo Shader FX se demanda surgir.

## 9.8 Inspector layout

Seção 10 "Material & Blend":

```
▼ Material & Blend
  Material:          [sprite-default ▾]   [+ Browse]
  ☐ Use Parent Material
  
  Instance Shader Params:                         (collapsed por default)
    [+ Add param]
  
  Blend Mode:        [Mix ▾]
```

Quando "Use Parent Material" está marcado, "Material" e "Instance Shader Params" ficam greyed.

## 9.9 Sampling section layout

Seção 9 "Sampling":

```
▼ Sampling
  Texture Filter:    [Inherit ▾ (Linear)]
  Texture Repeat:    [Inherit ▾ (Disabled)]
  
  Anti-halo / Edge Filtering:  enabled (atlas-level, read-only)
```

Quando "Inherit" selecionado, texto secundário em cinza mostra qual valor é herdado.

## 9.10 Default Material (sprite-default WGSL)

Shader bundled em `ph2d-render/shaders/sprite.wgsl`. Já existe; v4 expande para suportar:
- `Sprite.per_corner_tint[4]` (vertex color per-corner).
- `Sprite.tint_fill` (boolean uniform).
- `Sprite.opacity` (final multiplier).
- Texture Filter mode (não muda shader — muda sampler).
- Blend Mode (não muda fragment — muda pipeline state).

## 9.11 Interação com FX chain (futura)

Quando módulo Shader FX existir (futuro), FX chain será uma LISTA ordenada de FXPass no entity (Component separado, `FXChain(Vec<FXPass>)`). Cada FXPass tem seu próprio material/shader, mas USA o Material slot do entity como input.

Inspector da seção "Material & Blend" continua mostrando o Material **base** do sprite. FX chain virá numa **nova seção** ("Effects") que aparece SÓ se FXChain Component anexado. Isso preserva separação: este spec define só o material base + blend mode.

## 9.12 Caps gateados

| Cap | Valor | Razão |
|---|---|---|
| `BlendMode` variants | 6 (Mix/Add/Sub/Mul/Screen/PremultAlpha) | Cobre 99%; mais = FX chain |
| `FilterMode` variants | 7 (Inherit + 6 sampling modes) | Cobre GPU samplers |
| `RepeatMode` variants | 4 (Inherit + 3 wrap modes) | Cobre GPU wrap |
| `InstanceShaderParams` inline SmallVec | 8 pares | Shape comum (1-3 params); >8 vai heap |
| `InstanceShaderParams` key length | ≤ 32 chars | Uniform name length |

## 9.13 Anti-padrões evitados

1. **Material override forçado pra mudar 1 uniform** ❌ — Instance Shader Params resolve sem clone.
2. **Blend mode no Material** ❌ — Godot acerta colocando blend mode acessível direto (CanvasItemMaterial), e PH2D adopta como Component opcional. Material slot é só pra shader.
3. **Texture filter global only** ❌ — Per-node hierárquico (Godot) é estritamente melhor.
4. **MaterialPropertyBlock que quebra batching silenciosamente** ❌ — Unity URP ≤2023.x bug. PH2D garante Instance Shader Params + batching compatível desde dia 1.
5. **`Sprite.material: Option<MaterialRef>` no struct** ❌ — Component opcional preserva POD enxuto.
6. **Use Parent Material como propriedade no Sprite** ❌ — Component marker zero-size; ausência = comportamento default (usa próprio material).
