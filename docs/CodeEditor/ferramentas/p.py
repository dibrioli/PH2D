import sys,json
d=json.load(sys.stdin)
c=d["crate"]; v=d["versions"][0]
nm=c["name"]
print(f'{nm:24} max={c["max_version"]:16} upd={c["updated_at"][:10]} created_ver={v["created_at"][:10]} lic={v.get("license")} recent_dl={c.get("recent_downloads")} repo={c.get("repository")}')
