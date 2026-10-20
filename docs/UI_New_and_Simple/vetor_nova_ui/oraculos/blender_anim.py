import bpy
sc = bpy.context.scene
bpy.ops.mesh.primitive_plane_add()
ob = bpy.context.active_object
me = ob.data
def tryk(owner, path, label, v0, v1, idx=-1):
    try:
        exec(f"owner.{path} = v0") if not path.startswith('[') else None
        sc.frame_set(1); setattr_path(owner, path, v0); owner.keyframe_insert(path, frame=1)
        setattr_path(owner, path, v1); owner.keyframe_insert(path, frame=11)
        out = []
        for f in (1, 6, 11):
            sc.frame_set(f)
            dg = bpy.context.evaluated_depsgraph_get()
            out.append(f"f{f}={getval(owner, path)!r}")
        print(f"[OK] {label}: " + " ".join(out))
    except Exception as e:
        print(f"[RECUSA] {label}: {type(e).__name__}: {e}")
def setattr_path(owner, path, v):
    exec(f"owner.{path} = v", {"owner": owner, "v": v})
def getval(owner, path):
    v = eval(f"owner.{path}", {"owner": owner})
    try: return tuple(round(x,3) for x in v)
    except TypeError: return v
tryk(ob, "location", "float3 location", (0,0,0), (10,0,0))
tryk(ob, "hide_render", "bool hide_render", False, True)
tryk(ob, "display_type", "enum display_type", 'TEXTURED', 'WIRE')
tryk(ob, "name", "string name", "A", "B")
tryk(me, "vertices[0].co", "mesh vertices[0].co", (-1,-1,0), (-5,-5,0))
mat = bpy.data.materials.new("m"); mat.use_nodes = False
tryk(mat, "diffuse_color", "material diffuse_color RGBA", (1,0,0,1), (0,0,1,1))
txt = bpy.data.curves.new("t", 'FONT')
tryk(txt, "body", "text body", "abc", "xyz")
# Interpolação default de fcurves de enum/bool
for fc in (ob.animation_data.action.fcurves if hasattr(ob.animation_data.action,'fcurves') else []):
    print("fcurve", fc.data_path, [k.interpolation for k in fc.keyframe_points])
# avaliado: a malha avaliada segue a fcurve do vértice?
sc.frame_set(6)
dg = bpy.context.evaluated_depsgraph_get()
print("vert0 avaliado em f6:", tuple(round(c,3) for c in ob.evaluated_get(dg).data.vertices[0].co))
ts = sc.tool_settings
print("autokey props:", [p for p in dir(ts) if 'keyframe' in p or 'auto' in p.lower()])
print("prefs keying:", [p for p in dir(bpy.context.preferences.edit) if 'key' in p])
