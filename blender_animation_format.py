import bpy
import os
import mathutils

def convert_z_up_to_y_up(matrix):
    """Convert Blender’s Z-up coordinate system to OpenGL’s Y-up system."""
    conversion_matrix = mathutils.Matrix((
        (1,  0,  0,  0),
        (0,  0,  1,  0),
        (0, -1,  0,  0),
        (0,  0,  0,  1)
    ))
    return conversion_matrix @ matrix

def export_animation_data(filepath):
    with open(filepath, "w") as f:
        f.write("# WiseModel 0.0.1\n")
        
        armatures = [obj for obj in bpy.context.selected_objects if obj.type == 'ARMATURE']
        if not armatures:
            print("No armature selected for export.")
            return
        
        armature = armatures[0]  # Assuming one armature per model
        f.write(f"BONECOUNT: {len(armature.pose.bones)}\n")
        
        fps = bpy.context.scene.render.fps
        f.write(f"FPS: {fps}\n\n")
        for bone in armature.pose.bones:
            parent_index = -1 if bone.parent is None else list(armature.pose.bones).index(bone.parent)
            f.write(f"BONE_NAME: {bone.name}\nPARENT_INDEX: {parent_index}\nMATRIX:\n")
            
            matrix = convert_z_up_to_y_up(bone.bone.matrix_local)
            for row in matrix:
                f.write(f"{round(row[0], 6)} {round(row[1], 6)} {round(row[2], 6)} {round(row[3], 6)}\n")
            f.write("\n")

        if armature.animation_data and armature.animation_data.action:
            action = armature.animation_data.action
            f.write(f"# ========== {action.name} ==========\n")

            frame_start = int(action.frame_range[0])
            frame_end = int(action.frame_range[1])

            for frame in range(frame_start, frame_end + 1):
                bpy.context.scene.frame_set(frame)
                f.write(f"KEYFRAME: {frame}\n")

                for bone in armature.pose.bones:
                    position = bone.head  # Local position
                    rotation = bone.rotation_quaternion 
                    scale = bone.scale

                    f.write(f"{round(position.x, 6)} {round(position.y, 6)} {round(position.z, 6)}\n")
                    f.write(f"{round(rotation.w, 6)} {round(rotation.x, 6)} {round(rotation.y, 6)} {round(rotation.z, 6)}\n")
                    f.write(f"{round(scale.x, 6)} {round(scale.y, 6)} {round(scale.z, 6)}\n\n")

output_path = os.path.expanduser("E:/Software_Dev/rust/rust-opengl-engine/resources/meme.txt")
export_animation_data(output_path)