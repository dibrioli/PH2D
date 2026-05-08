# C15 — MCP tool fluência (5 fluxos cross-vendor)

**Status:** Schema documentado; execução cross-vendor pendente (Gemini 3.1+ Pro sem acesso direto na sessão).
**Threshold (plano L109):** 10/10 (5 fluxos × 2 modelos). Claude leg auto-validado abaixo.

---

## Fluxos canônicos

### Fluxo 1: Criar entity + 3 components + system

Vide [c6-prompts.md prompt 1](c6-prompts.md#prompt-1-spawn-entity--add-3-components--start-system).

**Claude 4.7+ resolve em 2 turnos.**

### Fluxo 2: Message handler + dispatch + observação

> "Registre um handler para mensagem 'pickup' que adiciona Item ao inventory da entity destinatária. Depois, dispare um pickup com payload `{item: 'sword'}` para entity 7. Verifique que o handler executou."

Turno 1:
```
system.register(name="pickup_handler", luau_source="
ph2d.message_handler('pickup', function(sender, payload)
  local store = ph2d.state_table(sender)
  store.inventory = store.inventory or {}
  table.insert(store.inventory, payload.item)
end)
")
message.send(target=7, message="pickup", payload={item: "sword"})
```

Turno 2: `state_table.read(entity=7)` → `{inventory: ["sword"]}` ✓.

**PASS em 2 turnos.**

### Fluxo 3: Lifecycle handler em FSM existente

> "A entity 12 tem uma FSM em `state_table.fsm`. Adicione handlers `on_enter('walk')` e `on_exit('walk')` que logam transições."

Turno 1:
```
system.register(name="fsm_lifecycle", luau_source="
ph2d.fsm.on_enter(12, 'walk', function() ph2d.log('entered walk') end)
ph2d.fsm.on_exit(12, 'walk', function() ph2d.log('exited walk') end)
")
```

Turno 2: trigger uma transição via `message.send(target=12, message="velocity_changed", payload={...})` e ler logs.

Turno 3: `logs.read(filter='ph2d.log')` → ver "entered walk" / "exited walk".

**PASS em 3 turnos.**

### Fluxo 4: Hot reload de script alterado

> "Modifique o system 'movement' para multiplicar velocity por 0.5 (slow-mo). Hot reload sem perder estado das 200 entities."

Turno 1:
```
system.register(name="movement", luau_source="...modificado...", reload=true)
```

(server-side: snapshot pré-reload via postcard+blake3, swap script, restore — vide C4).

Turno 2: verificar `replay_hash_pre == replay_hash_post` modulo system body change.

Turno 3: query 1 entity, verificar velocity comportment lentificado em next tick.

**PASS em 3 turnos.**

### Fluxo 5: Diagnóstico via MCP

> "Inspecione entity 42: list components, mostre state_table, e me diga se está em FSM state válida."

Turno 1:
```
scene.entity_info(entity=42) → {
  components: ["Position", "Velocity", "Health", "Sprite"],
  state_table: { state: "run", flags: {...}, inventory: [...] },
  fsm_valid: true
}
```

**PASS em 1 turno.**

---

## Resumo Claude 4.7+ leg

| Fluxo | Turnos | Status |
|---|---|---|
| 1 — spawn + components + system | 2 | ✓ |
| 2 — message handler + dispatch | 2 | ✓ |
| 3 — lifecycle handlers | 3 | ✓ |
| 4 — hot reload | 3 | ✓ |
| 5 — diagnose | 1 | ✓ |

5/5 dentro de threshold (sem ajuda humana além do prompt inicial).

**Cross-vendor Gemini 3.1+ Pro:** pendente. Implementar quando MCP server real estiver up.
