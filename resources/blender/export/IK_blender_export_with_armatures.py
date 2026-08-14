import bpy
import mathutils
from bpy_extras.io_utils import axis_conversion
from pathlib import Path
import os
import shutil

# ===================================================
# Rig configuration
# ===================================================

CONTROL_RIG_NAME = "RIG_CONTROL"
EXPORT_RIG_NAME = "RIG_EXPORT"
SRC_PREFIX = "SRC_"

# ===================================================
# Output configuration
# ===================================================

diffuse_texture_path = Path(
    r"C:\Users\jdwis\OneDrive\Desktop\blue_noise.png"
)

output_mesh_path = Path(
    r"E:\Software_Dev\rust\rust-wgpu-engine\resources\models\animated\ik_guy\mesh.txt"
)

diffuse_texture_filename = "testtext.png"


# ===================================================
# Helpers
# ===================================================

def conversion_matrix():
    """
    Blender:
        forward = -Y
        up      =  Z

    Engine:
        forward = -Z
        up      =  Y
    """
    return axis_conversion(
        from_forward="-Y",
        from_up="Z",
        to_forward="-Z",
        to_up="Y",
    ).to_4x4()


def write_mat_like_old(f, matrix: mathutils.Matrix):
    """
    Preserve the legacy on-disk matrix layout expected by the engine.
    """
    transposed = matrix.transposed()

    for row in transposed:
        f.write(
            f"{row[0]:.5f} "
            f"{row[1]:.5f} "
            f"{row[2]:.5f} "
            f"{row[3]:.5f}\n"
        )


def require_armature(name):
    obj = bpy.data.objects.get(name)

    if obj is None:
        raise RuntimeError(f'Could not find armature object "{name}".')

    if obj.type != "ARMATURE":
        raise RuntimeError(
            f'Object "{name}" exists, but it is not an armature.'
        )

    return obj


def selected_mesh():
    meshes = [
        obj
        for obj in bpy.context.selected_objects
        if obj.type == "MESH"
    ]

    if not meshes:
        raise RuntimeError(
            "Select the mesh that should be exported."
        )

    if len(meshes) > 1:
        print(
            "WARNING: More than one mesh is selected. "
            f'Using "{meshes[0].name}".'
        )

    return meshes[0]


def source_bone_name(export_bone_name):
    return f"{SRC_PREFIX}{export_bone_name}"


def validate_rig_mapping(control_armature, export_armature):
    """
    RIG_EXPORT supplies the actual exported hierarchy and rest pose.

    RIG_CONTROL supplies animation through matching SRC_<name> bones.
    """
    missing_source_bones = []

    for export_bone in export_armature.pose.bones:
        source_name = source_bone_name(export_bone.name)

        if control_armature.pose.bones.get(source_name) is None:
            missing_source_bones.append(
                f"{export_bone.name} -> {source_name}"
            )

    if missing_source_bones:
        formatted = "\n".join(
            f"  {mapping}"
            for mapping in missing_source_bones
        )

        raise RuntimeError(
            "RIG_EXPORT bones are missing matching source bones in "
            f"RIG_CONTROL:\n{formatted}"
        )


# ===================================================
# Skeleton and animation export
# ===================================================

def write_skeleton_and_anims(
    f,
    control_armature,
    export_armature,
):
    convert = conversion_matrix()
    convert_inverse = convert.inverted()

    control_armature.animation_data_create()
    original_action = control_armature.animation_data.action
    original_frame = bpy.context.scene.frame_current

    # Keep the previous exporter behavior: temporarily convert both rigs'
    # armature data into engine axes, then restore them afterward.
    control_armature.data.transform(convert)
    export_armature.data.transform(convert)
    bpy.context.view_layer.update()

    try:
        validate_rig_mapping(
            control_armature,
            export_armature,
        )

        export_bones = list(export_armature.pose.bones)

        bone_index_of = {
            bone.name: index
            for index, bone in enumerate(export_bones)
        }

        roots = [
            bone.name
            for bone in export_bones
            if bone.parent is None
        ]

        print(f"Export skeleton roots: {roots}")

        if len(roots) != 1:
            print(
                "WARNING: The export skeleton does not have exactly one "
                "root bone. Verify the hierarchy in RIG_EXPORT."
            )

        f.write("GLOBAL_TRANSFORM:\n")

        global_transform = (
            export_armature.matrix_world.copy().inverted()
        )

        write_mat_like_old(f, global_transform)
        f.write("\n")

        f.write(
            "\nSKELETON_DATA "
            "####################################\n\n"
        )
        f.write(f"BONECOUNT: {len(export_bones)}\n")

        # RIG_EXPORT determines:
        #   * bone order
        #   * parent indices
        #   * inverse bind matrices
        for export_bone in export_bones:
            if export_bone.parent is None:
                parent_index = -1
            else:
                parent_index = bone_index_of[
                    export_bone.parent.name
                ]

            f.write(
                f"BONE_NAME: {export_bone.name}\n"
            )
            f.write(
                f"PARENT_INDEX: {parent_index}\n"
            )
            f.write("OFFSET_MATRIX:\n")

            inverse_bind = (
                export_bone.bone.matrix_local.inverted()
            )

            write_mat_like_old(f, inverse_bind)
            f.write("\n")

        f.write(
            "\nANIMATION_DATA "
            "####################################\n\n"
        )

        fps = bpy.context.scene.render.fps
        f.write(f"FPS: {fps}\n")

        # Converts a RIG_CONTROL armature-space pose matrix into
        # RIG_EXPORT armature space. This is identity when both objects
        # share the same object transform, but is safer than assuming so.
        control_to_export_space = (
            export_armature.matrix_world.inverted_safe()
            @ control_armature.matrix_world
        )

        for action in bpy.data.actions:
            control_armature.animation_data.action = action
            bpy.context.view_layer.update()

            if control_armature.animation_data.action is None:
                continue

            frame_start = int(action.frame_range[0])
            frame_end = int(action.frame_range[1])
            duration = (frame_end - frame_start) / fps

            f.write(
                f"ANIMATION_NAME: {action.name}\n"
            )
            f.write(
                f"DURATION: {duration:.5f}\n\n"
            )

            for frame in range(
                frame_start,
                frame_end + 1,
            ):
                bpy.context.scene.frame_set(frame)
                bpy.context.view_layer.update()

                timestamp = (
                    frame - frame_start
                ) / fps

                f.write(f"KEYFRAME: {frame}\n")
                f.write(
                    f"TIMESTAMP: {timestamp:.5f}\n"
                )

                # Critical rule:
                #
                # Iterate RIG_EXPORT bones and calculate each local
                # transform relative to the parent declared by RIG_EXPORT.
                #
                # Do NOT use source_pose_bone.parent. The SRC_ foot bones
                # may be parented to IK/control bones in RIG_CONTROL.
                for export_bone in export_bones:
                    source_pose_bone = (
                        control_armature.pose.bones[
                            source_bone_name(
                                export_bone.name
                            )
                        ]
                    )

                    source_matrix = (
                        control_to_export_space
                        @ source_pose_bone.matrix
                    )

                    if export_bone.parent is None:
                        local_matrix = source_matrix
                    else:
                        source_parent = (
                            control_armature.pose.bones[
                                source_bone_name(
                                    export_bone.parent.name
                                )
                            ]
                        )

                        source_parent_matrix = (
                            control_to_export_space
                            @ source_parent.matrix
                        )

                        local_matrix = (
                            source_parent_matrix.inverted_safe()
                            @ source_matrix
                        )

                    position = local_matrix.translation
                    rotation = local_matrix.to_quaternion()
                    scale = local_matrix.to_scale()

                    f.write(
                        f"{position.x:.5f} "
                        f"{position.y:.5f} "
                        f"{position.z:.5f}\n"
                    )
                    f.write(
                        f"{rotation.x:.5f} "
                        f"{rotation.y:.5f} "
                        f"{rotation.z:.5f} "
                        f"{rotation.w:.5f}\n"
                    )
                    f.write(
                        f"{scale.x:.5f} "
                        f"{scale.y:.5f} "
                        f"{scale.z:.5f}\n\n"
                    )

            f.write("\n")

    finally:
        bpy.context.scene.frame_set(original_frame)

        if control_armature.animation_data is not None:
            control_armature.animation_data.action = (
                original_action
            )

        # Restore the .blend exactly as it was.
        export_armature.data.transform(convert_inverse)
        control_armature.data.transform(convert_inverse)
        bpy.context.view_layer.update()

    return bone_index_of


# ===================================================
# Mesh export
# ===================================================

def collect_export_weights(
    mesh_obj,
    vertex,
    valid_bone_names,
):
    """
    Keep only SRC_ vertex groups that map to RIG_EXPORT,
    retain the strongest four, then renormalize.
    """
    weights = []

    for group_assignment in vertex.groups:
        if group_assignment.group >= len(
            mesh_obj.vertex_groups
        ):
            continue

        group_name = mesh_obj.vertex_groups[
            group_assignment.group
        ].name

        if not group_name.startswith(SRC_PREFIX):
            continue

        exported_name = group_name.removeprefix(
            SRC_PREFIX
        )

        if exported_name not in valid_bone_names:
            continue

        weight = float(group_assignment.weight)

        if weight <= 0.0:
            continue

        weights.append((exported_name, weight))

    weights.sort(
        key=lambda item: item[1],
        reverse=True,
    )
    weights = weights[:4]

    total_weight = sum(
        weight
        for _, weight in weights
    )

    if total_weight <= 0.0:
        return []

    return [
        (name, weight / total_weight)
        for name, weight in weights
    ]


def write_mesh(
    f,
    mesh_obj,
    bone_index_of,
    diffuse_texture,
):
    convert = conversion_matrix()

    # Export the undeformed bind mesh, not the dependency-graph result
    # after the Armature modifier. Exporting the already-skinned mesh and
    # then skinning it again in the engine causes double deformation.
    mesh_data = mesh_obj.data.copy()

    try:
        mesh_data.transform(convert)
        mesh_data.update()

        uv_layer = mesh_data.uv_layers.active

        color_attributes = getattr(
            mesh_data,
            "color_attributes",
            None,
        )

        color_layer = (
            color_attributes.active
            if (
                color_attributes
                and color_attributes.active
            )
            else None
        )

        normal_matrix = (
            mesh_obj.matrix_world
            .to_3x3()
            .inverted()
            .transposed()
        )

        unique_vertices = []
        vertex_map = {}
        indices = []

        valid_bone_names = set(
            bone_index_of.keys()
        )

        unweighted_vertex_indices = set()

        for polygon in mesh_data.polygons:
            face_indices = []

            for loop_index in polygon.loop_indices:
                loop = mesh_data.loops[loop_index]
                vertex = mesh_data.vertices[
                    loop.vertex_index
                ]

                position = (
                    mesh_obj.matrix_world
                    @ vertex.co
                )

                normal = (
                    normal_matrix
                    @ polygon.normal
                ).normalized()

                if uv_layer:
                    uv = uv_layer.data[
                        loop_index
                    ].uv
                    uv_tuple = (
                        float(uv.x),
                        float(1.0 - uv.y),
                    )
                else:
                    uv_tuple = (0.0, 0.0)

                if (
                    color_layer
                    and color_layer.domain == "CORNER"
                ):
                    color = color_layer.data[
                        loop_index
                    ].color

                    color_tuple = (
                        float(color[0]),
                        float(color[1]),
                        float(color[2]),
                        float(
                            color[3]
                            if len(color) > 3
                            else 1.0
                        ),
                    )
                else:
                    color_tuple = (
                        1.0,
                        1.0,
                        1.0,
                        1.0,
                    )

                weights = collect_export_weights(
                    mesh_obj,
                    vertex,
                    valid_bone_names,
                )

                if not weights:
                    unweighted_vertex_indices.add(
                        vertex.index
                    )

                key = (
                    round(position.x, 6),
                    round(position.y, 6),
                    round(position.z, 6),
                    round(normal.x, 6),
                    round(normal.y, 6),
                    round(normal.z, 6),
                    round(uv_tuple[0], 6),
                    round(uv_tuple[1], 6),
                    tuple(
                        (
                            name,
                            round(weight, 6),
                        )
                        for name, weight in weights
                    ),
                    tuple(color_tuple),
                )

                if key not in vertex_map:
                    vertex_map[key] = len(
                        unique_vertices
                    )

                    unique_vertices.append(
                        (
                            position,
                            normal,
                            uv_tuple,
                            color_tuple,
                            weights,
                        )
                    )

                face_indices.append(
                    vertex_map[key]
                )

            # Fan triangulation.
            for index in range(
                1,
                len(face_indices) - 1,
            ):
                indices.extend(
                    [
                        face_indices[0],
                        face_indices[index],
                        face_indices[index + 1],
                    ]
                )

        if unweighted_vertex_indices:
            preview = sorted(
                unweighted_vertex_indices
            )[:20]

            print(
                "WARNING: Vertices without valid SRC_ "
                f"weights: {preview}"
            )

        f.write(
            "\nMESH_DATA "
            "##############################\n"
        )
        f.write(
            "TEXTURE_PROFILE: DecalCrisp\n"
        )
        f.write(
            f"TEXTURE_DIFFUSE: "
            f"{diffuse_texture}\n"
        )
        f.write(
            f"MESH_NAME: {mesh_obj.name}\n"
        )
        f.write(
            f"VERTEX_COUNT: "
            f"{len(unique_vertices)}\n"
        )

        for (
            position,
            normal,
            uv,
            color,
            weights,
        ) in unique_vertices:
            f.write("VERT:\n")
            f.write(
                f"{position.x:.5f} "
                f"{position.y:.5f} "
                f"{position.z:.5f}\n"
            )
            f.write(
                f"{normal.x:.5f} "
                f"{normal.y:.5f} "
                f"{normal.z:.5f}\n"
            )
            f.write(
                f"{uv[0]:.5f} "
                f"{uv[1]:.5f}\n"
            )

            if weights:
                f.write(
                    " ".join(
                        f"{name} {weight:.5f}"
                        for name, weight in weights
                    )
                    + "\n"
                )
            else:
                f.write("None\n")

            f.write("\n")

        f.write(
            f"INDEX_COUNT: {len(indices)}\n"
        )

        for index in range(
            0,
            len(indices),
            3,
        ):
            if index + 2 < len(indices):
                f.write(
                    f"{indices[index]} "
                    f"{indices[index + 1]} "
                    f"{indices[index + 2]} "
                )

        f.write("\n")

    finally:
        bpy.data.meshes.remove(mesh_data)


# ===================================================
# Main export
# ===================================================

def export_game_data(
    filepath,
    diffuse_texture,
):
    original_frame = bpy.context.scene.frame_current

    control_armature = require_armature(
        CONTROL_RIG_NAME
    )
    export_armature = require_armature(
        EXPORT_RIG_NAME
    )
    mesh_obj = selected_mesh()

    try:
        with open(filepath, "w") as f:
            f.write("# WiseModel 0.0.1\n")

            bone_index_of = (
                write_skeleton_and_anims(
                    f,
                    control_armature,
                    export_armature,
                )
            )

            write_mesh(
                f,
                mesh_obj,
                bone_index_of,
                diffuse_texture,
            )

        print(
            f"Exported data to {filepath}"
        )

    finally:
        bpy.context.scene.frame_set(
            original_frame
        )


def move_texture_file(
    source_path,
    destination_directory,
    destination_filename,
):
    destination_path = (
        destination_directory
        / destination_filename
    )

    shutil.copy2(
        source_path,
        destination_path,
    )


# ===================================================
# Run
# ===================================================

output_parent_dir = output_mesh_path.parent
os.makedirs(
    output_parent_dir,
    exist_ok=True,
)

move_texture_file(
    diffuse_texture_path,
    output_parent_dir,
    diffuse_texture_filename,
)

export_game_data(
    output_mesh_path,
    diffuse_texture_filename,
)
