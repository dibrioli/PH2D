# Foto do Blender: a curva de TEMPO de um strip (Strip Time) com a alça puxada além do possível.
import bpy, os, sys
OUT = os.environ['FOTO_OUT']
A, S = float(os.environ.get('ALCA_A', '0.5')), float(os.environ.get('ALCA_S', '3'))


def montar():
    sc = bpy.context.scene
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    ob = bpy.data.objects.new('Clip', None)
    sc.collection.objects.link(ob)
    ob.animation_data_create()
    for f in (1, 101):
        ob.location.x = f
        ob.keyframe_insert('location', index=0, frame=f)
    act = ob.animation_data.action
    tr = ob.animation_data.nla_tracks.new()
    st = tr.strips.new('Clip', 1, act)
    ob.animation_data.action = None
    st.use_animated_time = True
    for f, v in ((1, 1), (101, 101)):  # o tempo do clip: do quadro 1 ao 101
        st.strip_time = v
        st.keyframe_insert('strip_time', frame=f)
    fc = st.fcurves[0]
    k0, k1 = fc.keyframe_points
    for k in (k0, k1):
        k.interpolation = 'BEZIER'
        k.handle_left_type = k.handle_right_type = 'FREE'
        k.select_control_point = True
    k0.handle_right = (1 + A * 100, 1 + S * A * 100)
    k1.handle_left = (101 - 100 / 3, 101 - 100 / 3)
    fc.update()
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    sc.frame_end = 101
    return st


def passo():
    st = montar()
    win = bpy.context.window_manager.windows[0]
    scr = win.screen
    areas = sorted(scr.areas, key=lambda a: -a.width * a.height)
    big = areas[0]
    big.type = 'GRAPH_EDITOR'
    other = [a for a in scr.areas if a.type == 'DOPESHEET_EDITOR'][0]
    other.type = 'NLA_EDITOR'
    st.select = True
    for a in scr.areas:
        if a.type == 'GRAPH_EDITOR':
            a.spaces[0].show_region_ui = False
    st.use_animated_time = True

    def fim():
        for a in scr.areas:
            if a.type in ('GRAPH_EDITOR', 'NLA_EDITOR'):
                reg = [r for r in a.regions if r.type == 'WINDOW'][0]
                with bpy.context.temp_override(window=win, area=a, region=reg):
                    try:
                        if a.type == 'GRAPH_EDITOR':
                            bpy.ops.graph.view_all()
                        else:
                            bpy.ops.nla.view_all()
                    except Exception as e:
                        print('view_all', e)
        return None

    def foto():
        with bpy.context.temp_override(window=win):
            bpy.ops.screen.screenshot(filepath=OUT)
        print('FOTO', OUT)
        bpy.ops.wm.quit_blender()
        return None
    bpy.app.timers.register(fim, first_interval=1.0)
    bpy.app.timers.register(fim, first_interval=2.5)
    bpy.app.timers.register(foto, first_interval=4.0)
    return None


bpy.app.timers.register(passo, first_interval=1.5)
