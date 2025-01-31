#version 330 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec2 a_tex_coords;
layout (location = 2) in vec3 a_normal;
  
out vec3 Normal;
out vec3 FragPos;
out vec2 TexCoords;

uniform mat4 Model;
uniform mat4 View;
uniform mat4 Projection;

void main()
{
	FragPos = vec3(Model * vec4(a_pos, 1.0));
    Normal = mat3(transpose(inverse(Model))) * a_normal;  
	TexCoords = a_tex_coords;
    
    gl_Position = Projection * View * vec4(FragPos, 1.0);
}
