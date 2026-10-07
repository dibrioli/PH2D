import sys, os, json, collections, statistics
root = sys.argv[1]
KEYS = """apply_loading_to_paint use_glazing use_AA max_brush_size brush_mode brush_rotation_mode force_rotation_mode texture_order texture_random_order_ratio
splat_spacing splat_spacing_jitter splat_opacity splat_opacity_multiplier splat_opacity_jitter splat_scattering_jitter splat_size_jitter splat_rotation_jitter splat_smudge smudge_density smudged_lines_length maximum_blending
pressure_size_influence pressure_opacity_influence pressure_max pressure_multiplier tilt_size_multiplier precise_tip_tilt pressure_smoothing_strength position_smoothing_type position_smoothing_strength
impasto_strength paper_structure_influence paint_structure_influence paint_structure_scratch_mode grain_scale grain_tiles grain_random_start grain_random_splat grain_follow_brush_size grain_follow_distance grain_canvas_dpi_influence grain_dpi grain_brush_size
dual_brush_enabled dual_brush_splat_count dual_brush_splat_size dual_brush_opacity use_structure_textures thick_impasto use_default_brush_color dont_limit_pickax paint_mix_threshold length_water_reduction glaze_rgb_blend_mode scatter_mode cursor_mode""".split()
BG = ["enabled", "blend_mode", "blend_weight", "paper_strength", "opacity_multiplier", "random_start", "random_offset"]
by = collections.defaultdict(list)
for dp, dn, fn in os.walk(root):
    for f in fn:
        r = json.load(open(os.path.join(dp, f))); by[r["BrushID"]].append(r)
def fmt(vs):
    vs = [v for v in vs if v is not None]
    if not vs: return "—"
    if all(isinstance(v, bool) for v in vs): return f"T{sum(vs)}/{len(vs)}"
    if all(isinstance(v, (int, float)) for v in vs):
        if len(set(vs)) == 1: return f"{vs[0]:g}"
        return f"{min(vs):g}…{statistics.median(vs):g}…{max(vs):g}"
    c = collections.Counter(map(str, vs)); return "/".join(f"{k}{n}" for k, n in c.most_common(3))
order = ["AQUARELLE_PAINT_WATERCOLOR","AQUARELLE_PAINT_OIL","AQUARELLE_PAINT_EXPRESS_OIL","AQUARELLE_PAINT_PENCIL","AQUARELLE_PAINT_PASTEL","AQUARELLE_PAINT_INK","AQUARELLE_PAINT_MARKER","AQUARELLE_PAINT_AIRBRUSH","AQUARELLE_WET","AQUARELLE_DRY","AQUARELLE_BLEND","AQUARELLE_SMUDGE","AQUARELLE_ERASER","AQUARELLE_CLONE"]
short = {k: k.replace("AQUARELLE_PAINT_","").replace("AQUARELLE_","")[:9] for k in order}
print("chave".ljust(34) + "".join(short[k].ljust(21) for k in order))
for key in KEYS:
    row = []
    for k in order:
        row.append(fmt([r["Parameters"].get(key) for r in by[k]]))
    print(key.ljust(34) + "".join(c[:20].ljust(21) for c in row))
for key in BG:
    row = [fmt([r["Parameters"]["background_texture_params"][0].get(key) for r in by[k]]) for k in order]
    print(("bg0." + key).ljust(34) + "".join(c[:20].ljust(21) for c in row))
for key in ["paint_type", "enabled"]:
    row = [fmt([(r["ParametersEx"] or {}).get("painting_params", {}).get(key) for r in by[k]]) for k in order]
    print(("X.painting." + key).ljust(34) + "".join(c[:20].ljust(21) for c in row))
for key in ["size", "opacity", "water", "pressure"]:
    row = [fmt([(r["ParametersEx"] or {}).get("volume_params", {}).get(key) for r in by[k]]) for k in order]
    print(("X.volume." + key).ljust(34) + "".join(c[:20].ljust(21) for c in row))
# texturas: quantas shapes/grains cada pincel usa
row = []
for k in order:
    ns = [sum(1 for t in (r["Textures"] or {}) if t.startswith("Shape")) for r in by[k]]
    ng = [sum(1 for t in (r["Textures"] or {}) if t.startswith("Texture")) for r in by[k]]
    row.append(f"S{fmt(ns)} G{fmt(ng)}")
print("texturas(shapes,grains)".ljust(34) + "".join(c[:20].ljust(21) for c in row))
