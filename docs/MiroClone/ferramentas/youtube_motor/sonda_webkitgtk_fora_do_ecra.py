#!/usr/bin/env python3
"""W18 — mede o WebKitGTK 4.1 (instalado) a tocar o embed do YouTube FORA DO ECRÃ e a entregar quadros.

Pergunta: o tocador oficial (youtube-nocookie.com/embed/<id>) toca neste motor nesta máquina, e uma janela
GtkOffscreenWindow entrega quadros (píxeis) a que ritmo, com que CPU/RSS? Saída: JSON com cabeçalho + PNG
de alguns quadros. Uso: sonda_webkitgtk_fora_do_ecra.py <id_video> <segundos> <pasta_saida>
"""
import json, os, sys, time, hashlib, resource
import gi
gi.require_version("Gtk", "3.0"); gi.require_version("WebKit2", "4.1")
from gi.repository import Gtk, WebKit2, GLib

vid, segs, out = sys.argv[1], float(sys.argv[2]), sys.argv[3]
os.makedirs(out, exist_ok=True)
W, H = 560, 315
win = Gtk.OffscreenWindow(); win.set_default_size(W, H)
ctx = WebKit2.WebContext.get_default()
view = WebKit2.WebView.new_with_context(ctx)
s = view.get_settings()
s.set_media_playback_requires_user_gesture(False)
s.set_enable_mediasource(True)
s.set_hardware_acceleration_policy(WebKit2.HardwareAccelerationPolicy.NEVER if os.environ.get("SEM_GPU") else WebKit2.HardwareAccelerationPolicy.ALWAYS)
win.add(view); win.show_all()
html = f"""<!doctype html><html><body style='margin:0;background:#000'>
<iframe id=p width={W} height={H} src='https://www.youtube-nocookie.com/embed/{vid}?autoplay=1&mute=1&playsinline=1&enablejsapi=1'
 allow='autoplay; encrypted-media; picture-in-picture; fullscreen' frameborder=0></iframe></body></html>"""
view.load_html(html, "https://ph2d.local/")
t0 = time.monotonic(); quadros = []; hashes = []; fotos = []
cpu0 = resource.getrusage(resource.RUSAGE_SELF); cpuc0 = resource.getrusage(resource.RUSAGE_CHILDREN)
def tick():
    t = time.monotonic() - t0
    pb = win.get_pixbuf()
    if pb is not None:
        px = pb.get_pixels()
        h = hashlib.sha1(px).hexdigest()[:12]
        mudou = not hashes or hashes[-1] != h
        hashes.append(h)
        # brilho médio de uma faixa central: preto = vídeo não pinta
        quadros.append({"t": round(t, 3), "hash": h, "mudou": mudou})
        if (mudou and len(fotos) < 6 and t > 3) or (len(quadros) % 400 == 0):
            f = os.path.join(out, f"webkitgtk_{len(fotos)}.png"); pb.savev(f, "png", [], []); fotos.append(f)
    if t > segs:
        Gtk.main_quit(); return False
    return True
GLib.timeout_add(16, tick)
Gtk.main()
cpu1 = resource.getrusage(resource.RUSAGE_SELF)
dur = time.monotonic() - t0
mud = [q for q in quadros if q["mudou"] and q["t"] > 5]
json.dump({
  "o_que": "WebKitGTK 4.1 fora do ecrã (GtkOffscreenWindow.get_pixbuf a cada 16 ms) a tocar o embed do YouTube",
  "webkit": f"{WebKit2.get_major_version()}.{WebKit2.get_minor_version()}.{WebKit2.get_micro_version()}",
  "aceleracao": "nunca" if os.environ.get("SEM_GPU") else "sempre", "video": vid, "segundos": segs, "data": time.strftime("%F %T"),
  "loadavg": os.getloadavg(), "quadros_lidos": len(quadros), "quadros_distintos_depois_de_5s": len(mud),
  "fps_distintos_depois_de_5s": round(len(mud) / max(dur - 5, 1e-9), 2),
  "cpu_s_proprio": round(cpu1.ru_utime + cpu1.ru_stime - cpu0.ru_utime - cpu0.ru_stime, 2),
  "rss_max_mb_proprio": round(cpu1.ru_maxrss / 1024, 1), "fotos": fotos,
}, open(os.path.join(out, "webkitgtk" + ("_semgpu" if os.environ.get("SEM_GPU") else "") + ".json"), "w"), indent=1)
print(open(os.path.join(out, "webkitgtk" + ("_semgpu" if os.environ.get("SEM_GPU") else "") + ".json")).read())
