import bpy
import math
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

# Print the evaluated hand positions at the first frame of each action.
# This is useful for distinguishing an exporter issue from IK targets that
# are already near the body center inside Blender.
DEBUG_HAND_POSITIONS = True

# ===================================================
# Output configuration
# ===================================================

diffuse_texture_path = Path(
    r"E:\Software_Dev\rust\rust-wgpu-engine\resources\blender\img\base_skin.png"
)

output_mesh_path = Path(
    r"E:\Software_Dev\rust\rust-wgpu-engine\resources\models\animated\ik_guy\mesh.txt"
)

diffuse_texture_filename = "testtext.png"


# ===================================================
# Coordinate conversion
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


CONVERT = conversion_matrix()
CONVERT_INVERSE = CONVERT.inverted()


def matrix_to_engine(matrix):
    """
    Change both the input and output basis of a transform matrix.

    This is the correct conversion for bone rest matrices, bone pose
    matrices, and parent-relative animation matrices.
    """
    return CONVERT @ matrix @ CONVERT_INVERSE


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


# ===================================================
# Object and rig helpers
# ===================================================

def require_armature(name):
    obj = bpy.data.objects.get(name)

    if obj is None:
        raise RuntimeError(
            f'Could not find armature object "{name}".'
        )

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
    RIG_EXPORT supplies:
        * exported bone names
        * exported bone order
        * exported parent hierarchy

    RIG_CONTROL supplies:
        * SRC_ rest matrices used by the weighted mesh
        * evaluated SRC_ animation poses, including IK
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


def validate_export_hierarchy(export_armature):
    """
    Report common hierarchy mistakes without assuming every custom bone name.
    """
    export_bones = export_armature.pose.bones

    expected_parents = {
        "foot_l": "calf_l",
        "foot_r": "calf_r",
        "hand_l": "forearm_l",
        "hand_r": "forearm_r",
    }

    for child_name, expected_parent_name in expected_parents.items():
        child = export_bones.get(child_name)

        if child is None:
            continue

        actual_parent_name = (
            child.parent.name
            if child.parent is not None
            else None
        )

        if actual_parent_name != expected_parent_name:
            raise RuntimeError(
                f'RIG_EXPORT hierarchy error: "{child_name}" must be '
                f'parented to "{expected_parent_name}" with Keep Offset, '
                f'but its current parent is {actual_parent_name!r}.'
            )


def control_to_export_matrix(control_armature, export_armature):
    """
    Convert a matrix from RIG_CONTROL armature space to RIG_EXPORT
    armature space, while remaining in Blender's coordinate basis.
    """
    return (
        export_armature.matrix_world.inverted_safe()
        @ control_armature.matrix_world
    )


# ===================================================
# Skeleton and animation export
# ===================================================

def evaluated_source_pose_engine(
    control_armature,
    export_armature,
    export_bones,
):
    """
    Capture the evaluated SRC_ bone matrices for the current frame.

    The returned matrices are absolute matrices in:
        RIG_EXPORT armature space
        engine coordinate basis

    IK and constraints have already been evaluated by Blender at this point.
    """
    control_to_export = control_to_export_matrix(
        control_armature,
        export_armature,
    )

    result = {}

    for export_bone in export_bones:
        source_name = source_bone_name(export_bone.name)
        source_pose_bone = control_armature.pose.bones[source_name]

        source_pose_export_blender = (
            control_to_export
            @ source_pose_bone.matrix
        )

        result[export_bone.name] = matrix_to_engine(
            source_pose_export_blender
        )

    return result


def print_hand_debug(
    action_name,
    frame,
    absolute_pose_engine,
):
    if not DEBUG_HAND_POSITIONS:
        return

    print(
        f'Action "{action_name}", frame {frame}: '
        "evaluated hand positions in export/engine space"
    )

    for hand_name in ("hand_l", "hand_r"):
        matrix = absolute_pose_engine.get(hand_name)

        if matrix is None:
            continue

        position = matrix.translation

        print(
            f"  {hand_name}: "
            f"({position.x:.5f}, "
            f"{position.y:.5f}, "
            f"{position.z:.5f})"
        )


def write_skeleton_and_anims(
    f,
    control_armature,
    export_armature,
):
    control_armature.animation_data_create()

    original_action = control_armature.animation_data.action
    original_frame = bpy.context.scene.frame_current

    validate_rig_mapping(
        control_armature,
        export_armature,
    )

    validate_export_hierarchy(
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

    # All exported mesh and skeleton data are normalized into
    # RIG_EXPORT armature-local space, so no additional model-space
    # correction is required at runtime.
    f.write("GLOBAL_TRANSFORM:\n")
    write_mat_like_old(
        f,
        mathutils.Matrix.Identity(4),
    )
    f.write("\n")

    f.write(
        "\nSKELETON_DATA "
        "####################################\n\n"
    )
    f.write(f"BONECOUNT: {len(export_bones)}\n")

    control_to_export = control_to_export_matrix(
        control_armature,
        export_armature,
    )

    # RIG_EXPORT determines the runtime hierarchy.
    # SRC_ bones determine the inverse bind matrices, because the mesh is
    # weighted to the SRC_ rest skeleton.
    for export_bone in export_bones:
        if export_bone.parent is None:
            parent_index = -1
        else:
            parent_index = bone_index_of[
                export_bone.parent.name
            ]

        f.write(f"BONE_NAME: {export_bone.name}\n")
        f.write(f"PARENT_INDEX: {parent_index}\n")
        f.write("OFFSET_MATRIX:\n")

        source_name = source_bone_name(export_bone.name)
        source_bone = control_armature.data.bones[source_name]

        source_rest_export_blender = (
            control_to_export
            @ source_bone.matrix_local
        )

        source_rest_export_engine = matrix_to_engine(
            source_rest_export_blender
        )

        inverse_bind = (
            source_rest_export_engine.inverted_safe()
        )

        write_mat_like_old(
            f,
            inverse_bind,
        )
        f.write("\n")

    f.write(
        "\nANIMATION_DATA "
        "####################################\n\n"
    )

    fps = (
        bpy.context.scene.render.fps
        / bpy.context.scene.render.fps_base
    )

    f.write(f"FPS: {fps:.5f}\n")

    try:
        for action in list(bpy.data.actions):
            control_armature.animation_data.action = action
            bpy.context.view_layer.update()

            if control_armature.animation_data.action is None:
                continue

            frame_start = math.floor(action.frame_range[0])
            frame_end = math.ceil(action.frame_range[1])
            duration = (frame_end - frame_start) / fps

            f.write(f"ANIMATION_NAME: {action.name}\n")
            f.write(f"DURATION: {duration:.5f}\n\n")

            previous_rotations = {}

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
                f.write(f"TIMESTAMP: {timestamp:.5f}\n")

                # Capture every evaluated absolute pose before deriving
                # parent-relative transforms. This bakes IK visually without
                # exporting IK controls.
                absolute_pose_engine = (
                    evaluated_source_pose_engine(
                        control_armature,
                        export_armature,
                        export_bones,
                    )
                )

                if frame == frame_start:
                    print_hand_debug(
                        action.name,
                        frame,
                        absolute_pose_engine,
                    )

                for export_bone in export_bones:
                    child_absolute = absolute_pose_engine[
                        export_bone.name
                    ]

                    if export_bone.parent is None:
                        local_matrix = child_absolute
                    else:
                        parent_absolute = absolute_pose_engine[
                            export_bone.parent.name
                        ]

                        local_matrix = (
                            parent_absolute.inverted_safe()
                            @ child_absolute
                        )

                    position, rotation, scale = (
                        local_matrix.decompose()
                    )

                    # q and -q represent the same orientation. Keep a
                    # consistent sign between frames so engines that linearly
                    # interpolate quaternion components do not take a long
                    # rotational path.
                    previous_rotation = previous_rotations.get(
                        export_bone.name
                    )

                    if (
                        previous_rotation is not None
                        and previous_rotation.dot(rotation) < 0.0
                    ):
                        rotation.negate()

                    previous_rotations[
                        export_bone.name
                    ] = rotation.copy()

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
            control_armature.animation_data.action = original_action

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
    Keep only SRC_ groups that map to exported bones, retain the strongest
    four influences, and renormalize after filtering.
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

        weights.append(
            (exported_name, weight)
        )

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
    export_armature,
    bone_index_of,
    diffuse_texture,
):
    """
    Export the undeformed bind mesh in RIG_EXPORT armature-local space and
    in the engine coordinate basis.

    The Armature modifier is deliberately not evaluated here. The engine
    must receive the original bind mesh, not a mesh already deformed by the
    current Blender pose.
    """
    mesh_data = mesh_obj.data.copy()

    try:
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

        # Mesh local -> Blender world -> RIG_EXPORT armature local
        mesh_to_export_blender = (
            export_armature.matrix_world.inverted_safe()
            @ mesh_obj.matrix_world
        )

        # A point only needs the destination-space basis conversion on the
        # left. Its source coordinates are the original Blender mesh-local
        # coordinates.
        mesh_to_export_engine = (
            CONVERT
            @ mesh_to_export_blender
        )

        normal_matrix = (
            mesh_to_export_engine
            .to_3x3()
            .inverted_safe()
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
                    mesh_to_export_engine
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

            # Fan triangulation for polygons with more than three corners.
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
                "WARNING: Vertices without valid SRC_ weights: "
                f"{preview}"
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
        with open(
            filepath,
            "w",
            encoding="utf-8",
            newline="\n",
        ) as f:
            f.write("# WiseModel 0.0.1\n")

            bone_index_of = write_skeleton_and_anims(
                f,
                control_armature,
                export_armature,
            )

            write_mesh(
                f,
                mesh_obj,
                export_armature,
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

        bpy.context.view_layer.update()


def move_texture_file(
    source_path,
    destination_directory,
    destination_filename,
):
    if not source_path.exists():
        raise RuntimeError(
            f"Texture file does not exist: {source_path}"
        )

    destination_path = (
        destination_directory
        / destination_filename
    )

    # Avoid SameFileError when source and destination resolve to the
    # same file.
    if (
        source_path.resolve()
        == destination_path.resolve()
    ):
        return

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
