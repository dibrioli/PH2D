# ADR-0083 — 4K full-res watercolor field residency (lift the storage-buffer cap)

**Status:** Accepted (2026-06-09) — pedido pelo Enio (#3 da fila): "4K full-res GPU-residency do campo de 28 canais" (agora 32-ch após ADR-0081).
**Decisor(es):** Enio (dono/decisor) + Claude.
**Estende:** [ADR-0080](0080-watercolor-km-multipigment-field.md)/[ADR-0081](0081-watercolor-real-pigment-palette.md) (campo 32-ch), [ADR-0049](0049-fluid-brushes.md)/[ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md) (residência GPU).
**Tags:** painter, watercolor, gpu, limits, residency, foundational

---

## 1. Contexto

O campo molhado é **32 canais** (`PIG_CH`, ADR-0080/0081) = **128 B/célula**. Num grid **full-res**
4K (`scale = 1`, 3840×2160 = 8.3M células) o buffer de pigmento é **~1.06 GB**. O device do
`ph2d-gpu` pedia `wgpu::Limits::default()`, cujo `max_storage_buffer_binding_size` é **128 MiB** — então
o buffer estourava o limite e a alocação falhava (o benchmark `perf_resident` pulava ≥2048²; o smoke
handoff do K–M registrou isso como limitação conhecida). **A produção usa grid LOW-RES (canvas/4)**,
onde 4K → ~1024×540 → 70 MB (cabe folgado), então isso nunca bateu no uso real; o item da fila é
habilitar o **full-res** (scale=1) para quem quer detalhe fino.

## 2. Decisão — pedir o teto do ADAPTER (superset seguro), não um default conservador

No `GpuContext::new` (`crates/ph2d-gpu/src/context.rs`), partir de `Limits::default()` (o tier
desktop que o Vello exige) mas **subir `max_storage_buffer_binding_size` + `max_buffer_size` para o
máximo ANUNCIADO pelo adapter**:
```rust
let al = adapter.limits();
let mut required_limits = wgpu::Limits::default();
required_limits.max_storage_buffer_binding_size =
    required_limits.max_storage_buffer_binding_size.max(al.max_storage_buffer_binding_size);
required_limits.max_buffer_size = required_limits.max_buffer_size.max(al.max_buffer_size);
```
- **Seguro (superset):** o adapter sempre anuncia ≥ o default, então `required_limits ≤ adapter
  limits` ⇒ `request_device` não pode falhar por isso, e nada que funcionava quebra (só passa a
  PERMITIR buffers maiores — é um teto, não uma alocação). Os outros limites ficam no default
  (Vello-tier).
- **Habilita o full-res 4K** onde o hardware tem VRAM (Apple Silicon unified memory; dGPUs modernos
  com ≥ alguns GB): o Metal/Vulkan anunciam `max_storage_buffer_binding_size` na casa de GB.
- **Graceful onde NÃO cabe:** hardware com teto menor anuncia menos → o campo continua low-res
  (canvas/4, o default de produção) e o `perf_resident` pula os tamanhos que excedem o teto
  (lê `device.limits().max_storage_buffer_binding_size`, agora o valor pedido) — sem cap silencioso
  (loga o skip).

### 2.1 Perf já é O(frente molhada), não O(grid)
Habilitar a ALOCAÇÃO é o bloqueador; o custo POR FRAME já é region-scoped (ADR-0078 S1: diffuse/
advect/composite só rodam sobre o envelope molhado), então 4K full-res não custa O(grid) por frame.
A VRAM total (6 buffers de pigmento × 1.06 GB no pior caso 4K full-res) é uma realidade de hardware
— cabe em memória unificada Apple Silicon; um device pequeno usa o grid low-res.

## 3. Impacto

- **`ph2d-gpu` (foundational):** só sobe 2 tetos de limite para o máximo do adapter. Sem mudança de
  API; afeta todo o GPU mas é puramente aditivo (permite buffers maiores). ADR porque é foundational.
- **`perf_resident` bench:** já consulta `device.limits().max_storage_buffer_binding_size` (agora o
  teto elevado) → roda os tamanhos que couberem, pula o resto logando.

## 4. Consequências

A residência do campo 32-ch funciona em full-res 4K em hardware capaz (o caso do Enio, Apple
Silicon). Conclui a fila de 3 features (pigmentos / franja ramificada / 4K residency). Fidelidade
máxima 4K com tiling esparso (alocar só tiles molhados, não o grid inteiro) fica como follow-up de
escala se um device de VRAM menor precisar de full-res.
