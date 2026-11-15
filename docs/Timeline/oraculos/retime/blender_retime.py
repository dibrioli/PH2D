# Oráculo: Blender F-Curve de TEMPO (NLA Strip Time) — saída medida, nunca o fonte.
# Entrada NOSSA: um trecho de 0 a 1 (escala 100), alça = (influência a, velocidade s relativa à média).
import bpy, json, sys
out = sys.argv[sys.argv.index('--') + 1]
SPAN = 100.0


def fcurves_of(act, ob):
    if hasattr(act, 'fcurves') and len(act.fcurves):
        return list(act.fcurves)
    slot = ob.animation_data.action_slot
    return list(act.layers[0].strips[0].channelbag(slot).fcurves)


def curve(gestures, ht='FREE'):
    ob = bpy.data.objects.new('o', None)
    bpy.context.scene.collection.objects.link(ob)
    ob.animation_data_create()
    ob.location.x = 0
    ob.keyframe_insert('location', index=0, frame=0)
    ob.location.x = SPAN
    ob.keyframe_insert('location', index=0, frame=SPAN)
    act = ob.animation_data.action
    fc = fcurves_of(act, ob)[0]
    k0, k1 = fc.keyframe_points
    for k in (k0, k1):
        k.interpolation = 'BEZIER'
        k.handle_left_type = ht
        k.handle_right_type = ht
    (a0, s0), (a1, s1) = gestures
    if ht == 'FREE':
        k0.handle_right = (a0 * SPAN, s0 * a0 * SPAN)
        k1.handle_left = (SPAN - a1 * SPAN, SPAN - s1 * a1 * SPAN)
    fc.update()
    xs = [i * SPAN / 200 for i in range(201)]
    ys = [fc.evaluate(x) for x in xs]
    dx = xs[1] - xs[0]
    r = dict(pedido=dict(saida=[a0, s0], entrada=[a1, s1]), handle_type=ht,
             alca_lida_saida=[k0.handle_right[0] / SPAN, k0.handle_right[1] / SPAN],
             alca_lida_entrada=[k1.handle_left[0] / SPAN, k1.handle_left[1] / SPAN],
             mapa=[round(y / SPAN, 5) for y in ys],
             monotono=all(b >= a - 1e-9 for a, b in zip(ys, ys[1:])),
             max=max(ys) / SPAN, min=min(ys) / SPAN,
             vel_ponta_saida=(ys[1] - ys[0]) / dx, vel_ponta_entrada=(ys[-1] - ys[-2]) / dx)
    bpy.data.objects.remove(ob)
    bpy.data.actions.remove(act)
    return r


G = [((1 / 3, 1), (1 / 3, 1)), ((0.33, 3), (1 / 3, 1)), ((0.5, 3), (1 / 3, 1)), ((0.9, 2), (1 / 3, 1)),
     ((0.8, 1), (0.8, 1)), ((1.5, 1), (1 / 3, 1)), ((0.5, 0), (0.5, 0)), ((0.5, -1), (1 / 3, 1))]
res = [curve(g) for g in G] + [curve(G[0], 'AUTO_CLAMPED')]

# NLA: o tempo animado do strip pode andar para trás?
ob = bpy.data.objects.new('n', None)
bpy.context.scene.collection.objects.link(ob)
ob.animation_data_create()
for f in (1, 101):
    ob.location.x = f
    ob.keyframe_insert('location', index=0, frame=f)
act = ob.animation_data.action
tr = ob.animation_data.nla_tracks.new()
st = tr.strips.new('s', 1, act)
ob.animation_data.action = None
st.use_animated_time = True
for f, v in ((1, 1), (50, 80), (101, 40)):
    st.strip_time = v
    st.keyframe_insert('strip_time', frame=f)
nla = []
for f in (1, 25, 50, 75, 101):
    bpy.context.scene.frame_set(f)
    nla.append([f, round(st.strip_time, 3), round(ob.location.x, 3)])
json.dump(dict(app='Blender ' + bpy.app.version_string, curvas=res, nla_strip_time_volta=nla),
          open(out, 'w'), indent=1)
print('ok', out)
