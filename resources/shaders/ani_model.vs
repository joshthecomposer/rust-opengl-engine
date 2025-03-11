#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec2 a_tex_coords;
layout (location = 3) in vec4 bone_ids;
layout (location = 4) in vec4 bone_weights;

out vec3 FragPos;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main()
{
	FragPos = vec3(model * vec4(a_pos, 1.0));
    gl_Position = projection * view * vec4(FragPos, 1.0);
}
