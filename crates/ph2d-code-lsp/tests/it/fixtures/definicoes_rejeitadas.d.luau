-- Control of crates/ph2d-code-lsp/tests/it/definicoes.rs: the pre-2026 `declare class` syntax that
-- luau-lsp 1.70.1 rejects SILENTLY (a showMessage, no failure) — the gate must see this rejection.
declare class Entity
    name: string
end

declare ph2d: { find: (name: string) -> Entity? }
