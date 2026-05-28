# ADR-0020 Amendment 1 — Metal Direct Overlay + `PlatformHost::register_metal_overlay()` extension

**Status:** Accepted (2026-05-29)
**Amenda:** [ADR-0020 — Surface lifecycle and device-lost recovery](0020-surface-lifecycle.md).
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Triggered by:** [ADR-0064 — Vector multi-platform input](0064-vector-multi-platform-input.md) §2.4.
**Tags:** amendment, vector, painter, contract, metal, ios, macos

---

## 1. Contexto

L4F2 Antigravity 2ª iter + L1F5 3ª iter: sub-9 ms ProMotion latency inalcançável via Bevy ECS + wgpu standard frame queue (12-16ms typical). Solução: **Metal Direct Overlay shell-level** bypass via `CAMetalLayer` injected sobre Vello canvas.

L1F5 catch: sem coordination explícita, wgpu surface + CAMetalLayer overlay competem pelo mesmo display sem GPU semaphore sync → flickering / tearing / race conditions. ADR-0020 (Surface lifecycle) precisa amendment para extension formal.

---

## 2. Amendment

### 2.1 `PlatformHost::register_metal_overlay()` extension (iOS + macOS)

```rust
// ph2d-host::PlatformHost
pub trait PlatformHost {
    // ... existing methods (vide ADR-0020) ...

    /// Register Metal overlay layer attached to main wgpu surface,
    /// sharing GPU semaphore (Metal Shared Events) for sync.
    ///
    /// Used by Vector Module Modo B (sub-9ms ProMotion path) + future Painter
    /// Pencil Pro paths se needed.
    #[cfg(target_os = "ios")]
    fn register_metal_overlay(
        &mut self,
        config: MetalOverlayConfig,
    ) -> Result<OverlayHandle, OverlayError>;

    #[cfg(target_os = "macos")]
    fn register_metal_overlay_macos(
        &mut self,
        config: MetalOverlayConfig,
    ) -> Result<OverlayHandle, OverlayError>;
}

pub struct MetalOverlayConfig {
    pub pixel_format: PixelFormat,         // BGRA8Unorm typically
    pub framebuffer_only: bool,            // true for predicted preview overlay
    pub sync_strategy: SyncStrategy,       // SharedEvent OR Independent
    pub anchor: OverlayAnchor,             // TopLeft / Centered / Fullscreen
}

pub enum SyncStrategy {
    /// Default: overlay e wgpu compartilham MTLSharedEvent;
    /// overlay submit aguarda wgpu compositor pass; sem flickering.
    SharedEvent,
    /// Advanced: overlay submit independente; mais rápido mas pode glitch
    /// ocasional em loadshift. Use apenas se SharedEvent insuficiente.
    Independent,
}

pub struct OverlayHandle {
    pub layer_id: u64,
    pub max_predicted_frames: u8,         // typical 1-4 ahead
}
```

### 2.2 Coordenação Metal Shared Events

`SyncStrategy::SharedEvent` (default):
1. wgpu compositor frame submit → signals `MTLSharedEvent` value N.
2. CAMetalLayer overlay aguarda `MTLSharedEvent` value N.
3. Overlay submit own draw + signals value N+1.
4. Next wgpu compositor aguarda value N+1.

Zero race conditions; zero flickering; visual snap-to-corrected position em reconcile threshold violado.

### 2.3 OverlayHandle lifecycle

```rust
impl Drop for OverlayHandle {
    fn drop(&mut self) {
        // Unregister layer from shell; release Metal resources.
    }
}
```

Auto-cleanup quando handle dropped (e.g., user troca de Pen tool para Select tool → overlay desativado).

### 2.4 Sub-resource semantics

Overlay é **sub-resource** sob wgpu surface (não independent surface). DeviceLost em wgpu surface → overlay também perdido (cleanup automático). Recovery via mesma sequence ADR-0020 §3.

---

## 3. Consequências

### 3.1 Positivas

- **Sub-9 ms ProMotion alcançável** via shell-level overlay bypass.
- **Zero flickering** via Metal Shared Events sync.
- **Auto-cleanup** via OverlayHandle Drop impl.
- **Cross-tool reusable** — Painter Pencil Pro future + outros tools podem aproveitar.

### 3.2 Negativas

- **Metal-only** — Vulkan / D3D12 não têm direct overlay equivalente. Vector Module Modo A standard cobre non-Apple devices.
- **iOS + macOS shell-level code** complexity. Encapsulated em `PlatformHost` extension.
- **`#[cfg(target_os)]` attributes** em trait methods (necessary; HR-1 strict apenas para subsistema crates — `ph2d-host` é shell-adjacent).

### 3.3 Neutras

- 2 extension methods (iOS + macOS variants); shared `MetalOverlayConfig` struct.

---

## 4. Implementação (Wave 1 + Wave 17)

- **T0.14** (W1 pre-task): shell iPad scaffold include Metal overlay bootstrapping.
- **T17.1** (W17): Vector Modo B Metal Direct Overlay full integration.
- Painter future use: Pencil Pro paths se demand (não W1 immediate).

Gate: `vector_metal_overlay_no_flicker` (visual test em iPad simulator + Mac M1).

---

## 5. Referências

- ADR-0020 surface lifecycle (parent).
- ADR-0064 Vector multi-platform input (triggering).
- Apple Pencil sub-9ms HN: <https://news.ycombinator.com/item?id=20091610>
- MTLSharedEvent Apple docs: <https://developer.apple.com/documentation/metal/mtlsharedevent>
- Antigravity L4F2 (2ª iter) + L1F5 (3ª iter) absorbed.
