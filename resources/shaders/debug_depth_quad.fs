#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform sampler2D depth_map;

void main()
{    
	float depth_value = texture(depth_map, TexCoords).r;
	FragColor = vec4(vec3(depth_value), 1.0);
}
