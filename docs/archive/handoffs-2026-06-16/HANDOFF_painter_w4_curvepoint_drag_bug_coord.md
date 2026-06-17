═══════════════════════════════════════════════════════════════════
BUG → Coordenador · W4 §3 CurvePoint NÃO arrasta (1-line fix no teu dispatch)
Autor: Implementador Painter (sessão 2026-06-04) · §2 painel landado (fe9969e)
═══════════════════════════════════════════════════════════════════

## Sintoma (Enio, testando no app)
"A curva está lá, mas os pontos não aceitam ser arrastados; quando clicados
mudam apenas um pouco de lugar."

## Root cause (no TEU `interaction/dispatch/pointer.rs`)
O bloco de Move-drag é gated por `active_rect`:
```rust
// pointer.rs ~L296-297
if let Some(active) = store.active_id() {
    if let Some(rect) = store.active_rect() {   // ← GATE
        ... (todo o drag, incl. o braço CurvePoint ~L350-359) ...
    }
}
```
Mas o braço de **Down do CurvePoint** (~L737-745) seta `active` mas **NÃO**
`active_rect` (descarta o rect com `_`):
```rust
if let Some((id, _)) = hit                       // ← descarta o rect
    && matches!(store.get(id), Some(InteractiveState::CurvePoint { .. }))
{
    store.set_active(Some(id));                   // só active
    if let Some(parent) = apply_curve_point_drag(store, id, event.x, event.y) {
        events.push(WidgetEvent::ValueChanged(parent));
    }
    return events.into_bump_slice();
}
```
→ no Down o ponto pula pra posição do clique (o "muda um pouco"), mas no Move
`active_rect == None` → o bloco inteiro é pulado → **nunca arrasta**.

## Fix (1 linha — pega o rect do hit e seta active_rect)
```rust
if let Some((id, rect)) = hit                    // bind o rect
    && matches!(store.get(id), Some(InteractiveState::CurvePoint { .. }))
{
    store.set_active(Some(id));
    store.set_active_rect(Some(rect));            // ← ADICIONE: destrava o Move-gate
    if let Some(parent) = apply_curve_point_drag(store, id, event.x, event.y) {
        events.push(WidgetEvent::ValueChanged(parent));
    }
    return events.into_bump_slice();
}
```
O VALOR do `active_rect` é irrelevante pro mapeamento (teu `apply_curve_point_drag`
usa o `canvas` da variante, não esse rect) — só a PRESENÇA (`Some`) é que destrava
o gate do Move. Usar o grab-rect do hit é o natural.

## Por que passou (sugestão de gate)
`dispatch/curve.rs` testa `apply_curve_point_drag` direto (unit), mas NÃO o
caminho integrado pointer.rs Down→Move. Sugiro um teste de regressão: registrar
um CurvePoint, simular `dispatch_pointer` Down depois Move, e assertar que o Move
produz `take_curve_point_drag()` Some (com a current code FALHA; com o fix passa).

## Estado do meu lado (painel + tool — fe9969e, verde)
Tudo do §2 wirado e testado: paint do canvas + handles, drain `take_curve_point_
drag` no `event.rs`, parse → `set_curve_point` no tool (teste
`curve_edit_panel_event_routes_to_set_curve_point`). Confirmei que o caminho
painel→tool funciona — o único elo quebrado é o `active_rect` no Down do dispatch.
Assim que você dropar o fix, o arrasto 2D funciona end-to-end (sem mudança minha).

Isolamento: NÃO toquei teu `pointer.rs` (DIRETRIZ §3.C — dispatch foundational é
teu; "me chame"). Drope o 1-liner e o Enio re-testa.
═══════════════════════════════════════════════════════════════════
