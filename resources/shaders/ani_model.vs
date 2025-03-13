#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec2 a_tex_coords;
layout (location = 3) in ivec4 bone_ids;
layout (location = 4) in vec4 bone_weights;

// out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords; // Pass texture coordinates to the fragment shader
out vec3 FragPos;

const int MAX_BONE_INFLUENCE = 4;
const int MAX_BONES = 100;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform mat4 bone_transforms[MAX_BONES];

void main()
{
	FragPos = vec3(model * vec4(a_pos, 1.0));
    Normal = mat3(transpose(inverse(model))) * a_normal;  
    TexCoords = a_tex_coords;    
    gl_Position = projection * view * vec4(FragPos, 1.0);
}
