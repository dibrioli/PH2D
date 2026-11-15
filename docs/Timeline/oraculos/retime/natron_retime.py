import os
D=os.getcwd()
r=app.createNode("fr.inria.openfx.ReadOIIO"); r.getParam("filename").setValue(D+"/nin/f.####.pgm")
rt=app.createNode("net.sf.openfx.Retime"); rt.connectInput(0,r)
sp=rt.getParam("speed")
MODE=os.environ.get("MODE","rampa")
if MODE=="rampa":
    sp.setValueAtTime(1.0,1); sp.setValueAtTime(3.0,50)
elif MODE=="volta":
    sp.setValueAtTime(1.0,1); sp.setValueAtTime(-1.0,50)
w=app.createNode("fr.inria.openfx.WriteOIIO"); w.connectInput(0,rt)
w.getParam("filename").setValue(D+"/nout/"+MODE+".####.png")
w.getParam("frameRange").setValue(2); w.getParam("firstFrame").setValue(1); w.getParam("lastFrame").setValue(60)
app.render(w,1,60)
quit()
