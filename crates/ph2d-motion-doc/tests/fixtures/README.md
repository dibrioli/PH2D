# Fixturas do `MotionDoc` gravadas ANTES do dono por nó (onda 3 da `line/ObjectsAndComponents`, 08/10)

Gravadas a partir de `demo_router::build_level(Some("3"|"2"))` com o código de `61fc08972` — **antes** de
a secção `[owner]` existir. Provam que um documento de antes do Motion-como-objecto abre igual (zero
perda) e sem donos; quem lhe dá um dono é a migração em tempo de execução (doc ObjectsAndComponents 01
§13). ⛔ Não regravar: o valor delas é terem nascido antes da mudança.

| ficheiro | cena |
|---|---|
| `motion_doc_sim_antes_do_dono.txt` | `PH2D_GPU_COOK_DEMO=3` — a simulação |
| `motion_doc_hibrido_antes_do_dono.txt` | `PH2D_GPU_COOK_DEMO=2` — a cadeia híbrida |
