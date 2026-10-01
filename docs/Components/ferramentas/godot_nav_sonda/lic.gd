extends SceneTree
func _initialize():
	var li := Engine.get_license_info()
	for c in Engine.get_copyright_info():
		var n: String = c["name"]
		if n.to_lower().contains("rvo") or n.to_lower().contains("clipper") or n.to_lower().contains("recast") or n.to_lower().contains("polypartition") or n.to_lower().contains("navigation"):
			for p in c["parts"]:
				print("%s | files=%s | license=%s" % [n, str(p["files"]).left(120), p["license"]])
	quit()
