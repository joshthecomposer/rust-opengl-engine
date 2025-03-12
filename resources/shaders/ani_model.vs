#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec2 a_tex_coords;
layout (location = 3) in ivec4 bone_ids;
layout (location = 4) in vec4 bone_weights;

// out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords; // Pass texture coordinates to the fragment shader

const int MAX_BONE_INFLUENCE = 4;
const int MAX_BONES = 100;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;
uniform mat4 bone_transforms[MAX_BONES];

void main()
{
	vec4 totalPosition = vec4(0.0f);
	vec3 totalNormal = vec3(0.0f);
    for(int i = 0 ; i < MAX_BONE_INFLUENCE; i++)
    {
        if(bone_ids[i] == -1) 
            continue;
        if(bone_ids[i] >=MAX_BONES) 
        {
            totalPosition = vec4(a_pos,1.0f);
            break;
        }
        vec4 localPosition = bone_transforms[bone_ids[i]] * vec4(a_pos,1.0f);
        totalPosition += localPosition * bone_weights[i];
        totalNormal += mat3(bone_transforms[bone_ids[i]]) * a_normal * bone_weights[i];
    }
	
	Normal = totalNormal;
    mat4 viewModel = view * model;
    gl_Position =  projection * viewModel * totalPosition;
    TexCoords = a_tex_coords;
}
