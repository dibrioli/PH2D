#!/usr/bin/env python3
# blender_sculpt_oracle.py
#
# ORACULO EXECUTAVEL do pincel de escultura do Blender.
#
# Este script e' NOSSO: ele dirige a API publica `bpy` do binario instalado.
# Nao contem nenhuma linha de fonte do Blender (GPL). Usar o binario e' permitido.
#
# USO (precisa de GUI real -- ver NOTA DE CONTEXTO abaixo):
#   blender --factory-startup -noaudio --window-geometry 0 0 640 480 \
#           --python blender_sculpt_oracle.py -- --out /caminho/saida.json [--cases casos.json]
#
# NOTA DE CONTEXTO (a barreira, medida):
#   Em `--background` o Blender NAO cria o RegionView3D da regiao WINDOW
#   (`region->regiondata == NULL`, porque `ED_region_init` so' roda no primeiro
#   draw). `CTX_wm_region_view3d()` devolve NULL, e o caminho do sculpt o
#   desreferencia -> SIGSEGV nao-deterministico. Nao ha' API Python para criar
#   esse ponteiro. Por isso o oraculo roda com GUI e faz o trabalho DENTRO de um
#   timer (`bpy.app.timers`), que so' dispara depois do primeiro draw -- e' nesse
#   instante que `region.data` passa a ser um RegionView3D valido.
#
# DETERMINISMO:
#   - view ORTOGRAFICA travada olhando de +Z para -Z (top), centrada na origem;
#   - raio do pincel em UNIDADES DE CENA (`use_locked_size='SCENE'`) nos DOIS
#     lugares que mandam: o brush E `unified_paint_settings` (o unified vence
#     quando `use_unified_size` esta' ligado -- foi o que fez o raio ser 0 nas
#     primeiras tentativas);
#   - `override_location=False`, senao o Blender DESCARTA o `location` fornecido
#     e refaz o raycast a partir do mouse.

import bpy
import sys
import json
import math
import traceback


# ----------------------------------------------------------------------------
# argumentos
# ----------------------------------------------------------------------------
def parse_argv():
    argv = sys.argv
    args = argv[argv.index("--") + 1:] if "--" in argv else []
    out = {"out": None, "cases": None, "human": False}
    i = 0
    while i < len(args):
        if args[i] == "--out":
            out["out"] = args[i + 1]; i += 2
        elif args[i] == "--cases":
            out["cases"] = args[i + 1]; i += 2
        elif args[i] == "--human":
            out["human"] = True; i += 1
        else:
            i += 1
    return out


def print_human(rep):
    """Saida literal, legivel e trivial de parsear (prefixo por linha)."""
    print("#ORACLE blender=%s ok=%s" % (rep["blender"], rep["ok"]), flush=True)
    for c in rep["cases"]:
        print("#CASE %s result=%s n_verts=%d"
              % (c["case"].get("label", "?"), ",".join(c["result"]), c["n_verts"]))
        for k in sorted(c["brush"]):
            print("#PARAM %s = %s" % (k, c["brush"][k]))
        v = c["view"]
        print("#VIEW perspective=%s distance=%s region=%dx%d"
              % (v["perspective"], v["distance"], v["region_w"], v["region_h"]))
        for row in v["view_matrix"]:
            print("#VIEWMAT " + " ".join("%.9g" % x for x in row))
        if c["falloff_curve"]:
            print("#FALLOFF " + " ".join("%.9g" % x for x in c["falloff_curve"]))
        for i, (a, b, dv) in enumerate(zip(c["verts_before"], c["verts_after"], c["disp"])):
            print("#V %d  init %.9g %.9g %.9g  final %.9g %.9g %.9g  d %.9g %.9g %.9g"
                  % (i, a[0], a[1], a[2], b[0], b[1], b[2], dv[0], dv[1], dv[2]))
        print("#ENDCASE")
    print("#ORACLE_END", flush=True)


# ----------------------------------------------------------------------------
# construcao da cena
# ----------------------------------------------------------------------------
def wipe_objects():
    if bpy.context.mode != 'OBJECT':
        try:
            bpy.ops.object.mode_set(mode='OBJECT')
        except Exception:
            pass
    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob, do_unlink=True)
    for me in list(bpy.data.meshes):
        if me.users == 0:
            bpy.data.meshes.remove(me)


def build_grid(subdiv, size):
    """Plano subdividido, deterministico. Vertices em z=0."""
    bpy.ops.mesh.primitive_grid_add(
        x_subdivisions=subdiv, y_subdivisions=subdiv, size=size,
        location=(0.0, 0.0, 0.0))
    ob = bpy.context.active_object
    ob.name = "OracleGrid"
    return ob


def lock_view_top(area, region, view_distance):
    """View ORTOGRAFICA de topo: olhar de +Z para -Z, centrada na origem.

    Com rotacao identidade o Blender olha ao longo de -Z, que e' exatamente a
    vista TOP. Isso torna o raycast do centro da regiao cair em (0,0,0) e a
    normal de vista ser +Z.
    """
    rv3d = region.data
    rv3d.view_perspective = 'ORTHO'
    rv3d.view_rotation = (1.0, 0.0, 0.0, 0.0)  # identidade = TOP
    rv3d.view_location = (0.0, 0.0, 0.0)
    rv3d.view_distance = view_distance
    rv3d.update()
    return rv3d


# ----------------------------------------------------------------------------
# configuracao do pincel -- TUDO explicito
# ----------------------------------------------------------------------------
def configure_brush(case):
    ts = bpy.context.scene.tool_settings
    sculpt = ts.sculpt
    ups = sculpt.unified_paint_settings
    br = sculpt.brush

    radius = case["unprojected_size"]

    # --- tamanho: travado em unidades de CENA nos dois lados ---
    # `BKE_brush_use_locked_size()` consulta unified_paint_settings quando
    # `use_unified_size` esta' ligado (o default), e so' cai no brush caso
    # contrario. Setar so' o brush deixa o raio vindo da projecao de tela.
    # ORDEM IMPORTA: escrever `size` (pixels) re-deriva `unprojected_size` pela
    # projecao, entao `size` vem PRIMEIRO e `unprojected_size` por ultimo.
    ups.use_unified_size = True
    ups.use_locked_size = 'SCENE'
    ups.size = case.get("size_px", 100)
    ups.unprojected_size = radius
    br.use_locked_size = 'SCENE'
    br.size = case.get("size_px", 100)
    br.unprojected_size = radius

    # --- forca ---
    # `strength_ups` / `strength_brush` permitem separar as duas fontes, para
    # descobrir qual delas o kernel de fato le (e se ele le as duas).
    ups.use_unified_strength = case.get("use_unified_strength", True)
    ups.strength = case.get("strength_ups", case["strength"])
    br.strength = case.get("strength_brush", case["strength"])

    # --- tipo e lei ---
    br.sculpt_brush_type = case.get("sculpt_brush_type", 'DRAW')
    br.direction = case.get("direction", 'ADD')
    br.hardness = case["hardness"]
    br.auto_smooth_factor = case["auto_smooth_factor"]
    br.curve_distance_falloff_preset = case.get("curve_preset", 'SMOOTH')
    br.falloff_shape = case.get("falloff_shape", 'SPHERE')
    br.use_frontface = case.get("use_frontface", False)
    br.use_accumulate = case.get("use_accumulate", False)
    br.use_original_normal = case.get("use_original_normal", False)
    br.use_original_plane = case.get("use_original_plane", False)
    br.normal_weight = case.get("normal_weight", 0.0)
    br.normal_radius_factor = case.get("normal_radius_factor", 0.5)
    br.area_radius_factor = case.get("area_radius_factor", 0.5)
    br.plane_offset = case.get("plane_offset", 0.0)
    br.plane_trim = case.get("plane_trim", 0.5)
    br.tip_roundness = case.get("tip_roundness", 1.0)
    br.tip_scale_x = case.get("tip_scale_x", 1.0)
    br.sculpt_plane = case.get("sculpt_plane", 'AREA')
    br.stroke_method = case.get("stroke_method", 'SPACE')
    br.spacing = case.get("spacing", 10)
    br.use_space_attenuation = case.get("use_space_attenuation", False)
    br.jitter = 0.0
    br.jitter_absolute = 0
    br.use_pressure_strength = False
    br.use_pressure_size = False

    # sem textura e sem automasking: o oraculo mede a LEI do pincel, nada mais
    for slot_name in ("texture_slot", "mask_texture_slot"):
        slot = getattr(br, slot_name, None)
        if slot is not None and hasattr(slot, "map_mode"):
            try:
                slot.map_mode = 'VIEW_PLANE'
            except Exception:
                pass
    br.texture = None
    br.mask_texture = None
    am = br.mesh_automasking_settings
    for p in am.bl_rna.properties:
        if p.identifier.startswith("use_") and not p.is_readonly:
            try:
                setattr(am, p.identifier, False)
            except Exception:
                pass

    return br, ups


def snapshot_brush(br, ups):
    """Todos os parametros que decidem o resultado, para o relatorio."""
    keys = [
        "sculpt_brush_type", "strength", "size", "unprojected_size",
        "use_locked_size", "hardness", "auto_smooth_factor",
        "curve_distance_falloff_preset", "falloff_shape", "direction",
        "use_frontface", "use_accumulate", "use_original_normal",
        "use_original_plane", "normal_weight", "normal_radius_factor",
        "area_radius_factor", "plane_offset", "plane_trim", "tip_roundness",
        "tip_scale_x", "sculpt_plane", "stroke_method", "spacing",
        "use_space_attenuation", "jitter", "use_pressure_strength",
        "use_pressure_size",
    ]
    snap = {}
    for k in keys:
        try:
            v = getattr(br, k)
            snap[k] = v if isinstance(v, (int, float, bool, str)) else repr(v)
        except Exception:
            snap[k] = "<n/a>"
    snap["_ups.use_unified_size"] = ups.use_unified_size
    snap["_ups.use_locked_size"] = ups.use_locked_size
    snap["_ups.unprojected_size"] = ups.unprojected_size
    snap["_ups.use_unified_strength"] = ups.use_unified_strength
    snap["_ups.strength"] = ups.strength
    return snap


def sample_falloff_curve(br, n=17):
    """A curva de falloff avaliada, util para o chamador comparar a LEI."""
    try:
        cm = br.curve_distance_falloff
        cm.update()
        c = cm.curves[0]
        return [cm.evaluate(c, i / float(n - 1)) for i in range(n)]
    except Exception:
        return None


# ----------------------------------------------------------------------------
# o dab
# ----------------------------------------------------------------------------
def apply_dab(ob, win, screen, area, region, location, n_steps=1):
    cx = region.width / 2.0
    cy = region.height / 2.0
    stroke = []
    for i in range(n_steps):
        stroke.append(dict(
            name="",
            location=tuple(location),
            mouse=(cx, cy),
            mouse_event=(cx, cy),
            pressure=1.0,
            size=100.0,
            x_tilt=0.0,
            y_tilt=0.0,
            is_start=(i == 0),
            time=float(i),
        ))
    with bpy.context.temp_override(window=win, screen=screen, area=area,
                                   region=region, object=ob, active_object=ob):
        res = bpy.ops.sculpt.brush_stroke(
            stroke=stroke, mode='NORMAL',
            override_location=False,      # usa o `location` que EU dou
            ignore_background_click=False)
    return res


def verts_of(ob):
    return [(v.co.x, v.co.y, v.co.z) for v in ob.data.vertices]


# ----------------------------------------------------------------------------
# um caso
# ----------------------------------------------------------------------------
def run_case(case, win, screen, area, region):
    wipe_objects()
    ob = build_grid(case.get("subdiv", 32), case.get("grid_size", 2.0))
    before = verts_of(ob)

    bpy.ops.object.mode_set(mode='SCULPT')
    br, ups = configure_brush(case)
    rv3d = lock_view_top(area, region, case.get("view_distance", 4.0))

    res = apply_dab(ob, win, screen, area, region,
                    case.get("location", (0.0, 0.0, 0.0)),
                    case.get("n_steps", 1))

    bpy.ops.object.mode_set(mode='OBJECT')
    after = verts_of(ob)

    disp = []
    for (a, b) in zip(before, after):
        d = (b[0] - a[0], b[1] - a[1], b[2] - a[2])
        disp.append(d)

    return {
        "case": case,
        "result": list(res),
        "brush": snapshot_brush(br, ups),
        "falloff_curve": sample_falloff_curve(br),
        "view": {
            "perspective": rv3d.view_perspective,
            "distance": rv3d.view_distance,
            "region_w": region.width, "region_h": region.height,
            "view_matrix": [list(r) for r in rv3d.view_matrix],
        },
        "n_verts": len(before),
        "verts_before": before,
        "verts_after": after,
        "disp": disp,
    }


DEFAULT_CASE = {
    "subdiv": 32,
    "grid_size": 2.0,
    "unprojected_size": 0.5,
    "strength": 1.0,
    "hardness": 0.0,
    "auto_smooth_factor": 0.0,
    "curve_preset": 'SMOOTH',
    "location": (0.0, 0.0, 0.0),
    "n_steps": 1,
}


def build_cases(path):
    if path:
        with open(path) as f:
            spec = json.load(f)
        cases = []
        for c in spec:
            m = dict(DEFAULT_CASE)
            m.update(c)
            cases.append(m)
        return cases
    # padrao: varredura de hardness e de auto_smooth_factor
    cases = []
    for h in (0.0, 0.25, 0.5, 0.75, 0.9, 1.0):
        c = dict(DEFAULT_CASE); c["hardness"] = h
        c["label"] = "hardness=%.2f" % h
        cases.append(c)
    for a in (0.0, 0.05, 0.1, 0.25, 0.5, 1.0):
        c = dict(DEFAULT_CASE); c["auto_smooth_factor"] = a
        c["label"] = "auto_smooth=%.2f" % a
        cases.append(c)
    return cases


# ----------------------------------------------------------------------------
# driver -- roda DENTRO de um timer (depois do primeiro draw)
# ----------------------------------------------------------------------------
def main():
    opts = parse_argv()
    report = {"ok": False, "blender": bpy.app.version_string, "cases": []}
    try:
        win = bpy.context.window_manager.windows[0]
        screen = win.screen
        area = next(a for a in screen.areas if a.type == 'VIEW_3D')
        region = next(r for r in area.regions if r.type == 'WINDOW')
        if region.data is None:
            raise RuntimeError(
                "region.data is None -- o timer disparou antes do primeiro draw")
        report["region_data"] = repr(region.data)

        for case in build_cases(opts["cases"]):
            report["cases"].append(run_case(case, win, screen, area, region))
        report["ok"] = True
    except Exception as e:
        report["error"] = "%s: %s" % (type(e).__name__, e)
        report["traceback"] = traceback.format_exc()

    if opts["human"]:
        print_human(report)
    payload = json.dumps(report)
    if opts["out"]:
        with open(opts["out"], "w") as f:
            f.write(payload)
        print("ORACLE_WROTE %s bytes=%d ok=%s" % (opts["out"], len(payload), report["ok"]),
              flush=True)
    else:
        print("@@ORACLE@@" + payload, flush=True)
    if "error" in report:
        print("ORACLE_ERROR " + report["error"], flush=True)
        print(report.get("traceback", ""), flush=True)


def _timer():
    try:
        main()
    finally:
        bpy.ops.wm.quit_blender()
    return None


bpy.app.timers.register(_timer, first_interval=1.5)
