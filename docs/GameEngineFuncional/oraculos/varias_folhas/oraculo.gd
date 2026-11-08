extends SceneTree
func sheet(path, cols, rows):
    var img = Image.load_from_file(path)
    var tex = ImageTexture.create_from_image(img)
    var w = img.get_width()/cols; var h = img.get_height()/rows
    var out = []
    for r in rows:
        for c in cols:
            var a = AtlasTexture.new(); a.atlas = tex; a.region = Rect2(c*w, r*h, w, h); out.append(a)
    return out
func _init():
    var sf = SpriteFrames.new()
    sf.remove_animation("default")
    sf.add_animation("idle"); sf.set_animation_speed("idle", 8)
    for a in sheet("res://idle.png",4,1): sf.add_frame("idle", a)
    sf.add_animation("run"); sf.set_animation_speed("run", 12)
    for a in sheet("res://run.png",6,2): sf.add_frame("run", a)
    sf.add_frame("run", sheet("res://idle.png",4,1)[0], 2.0)
    print("ANIMS ", sf.get_animation_names())
    for n in sf.get_animation_names():
        for i in sf.get_frame_count(n):
            var t = sf.get_frame_texture(n,i)
            print("FRAME ", n, " ", i, " atlas=", t.atlas.get_size(), " region=", t.region, " dur=", sf.get_frame_duration(n,i))
    var s = AnimatedSprite2D.new(); s.sprite_frames = sf; root.add_child(s)
    for n in ["idle","run"]:
        s.play(n); s.frame = 0
        print("PLAY ", n, " size=", sf.get_frame_texture(n,0).get_size(), " centered=", s.centered, " offset=", s.offset)
    print("ATLAS_DISTINTOS ", sf.get_frame_texture("idle",0).atlas != sf.get_frame_texture("run",0).atlas)
    quit()
