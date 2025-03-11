#version 330 core
out vec4 FragColor;

in vec2 TexCoords;
in vec3 Normal;

uniform sampler2D texture_diffuse1;

void main()
{    
	vec3 normal_color = normalize(Normal) * 0.5 + 0.5;
    FragColor = vec4(normal_color, 1.0);
}
