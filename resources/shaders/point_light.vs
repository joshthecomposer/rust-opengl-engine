
#version 330 core
layout (location = 0) in vec3 aPos; // the position variable has attribute position 0

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 LightColor;

out vec3 light_color;

void main()
{
	vec4 worldPosition = model * vec4(aPos , 1.0);

	vec4 viewPosition = view * worldPosition;

    gl_Position = projection * viewPosition; 
	light_color = LightColor;
}
