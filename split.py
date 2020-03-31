#.\blender --background --python "C:\Users\Antoine\Desktop\Blender projects\Python\split.py"

import bpy
import sys
import json
import os

print("test")

def render(xMin,yMin, xMax, yMax, x, y, name):
    if os.path.exists("num.txt"):
        os.remove("num.txt")
    f = open("num.txt", 'w')
    f.write(str(y))
    f.write(str(x))
    f.close()
    bpy.ops.wm.open_mainfile(filepath=os.path.realpath(__file__)[0:-8] + name)
    bpy.context.scene.render.filepath = os.path.realpath(__file__)[0:-8] + str(y) + str(x) + ".png"
    bpy.context.scene.render.use_border = True
    bpy.context.scene.render.border_min_x = xMin
    bpy.context.scene.render.border_min_y = yMin
    bpy.context.scene.render.border_max_x = xMax
    bpy.context.scene.render.border_max_y = yMax
    bpy.ops.render.render(write_still=True)
    
argv = sys.argv
argv = argv[argv.index("--") + 1:]

f = open(argv[0], 'r')
renderData = json.load(f)
f.close()
print(os.path.realpath(__file__))
render(**renderData)