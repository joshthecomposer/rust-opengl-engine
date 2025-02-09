#version 330 core
out vec4 FragColor;

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoords;

uniform sampler2D texture_diffuse1;

struct DirLight {
 	vec3 direction;
	vec3 view_pos;

	vec3 ambient;
	vec3 diffuse;
	vec3 specular;
};
uniform DirLight dir_light;

vec3 calculate_directional_light() {
    vec3 lightDir = normalize(dir_light.direction);
    vec3 lightColor = dir_light.diffuse;

	// Ambient
    vec3 ambient = vec3(dir_light.ambient);
	
	// Diffuse
    vec3 norm = normalize(Normal);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    vec3 tex_color = texture(texture_diffuse1, TexCoords).rgb;

	return tex_color * (ambient + diffuse);
}

void main() {    
	vec3 result = vec3(0.0f, 0.0f, 0.0f); 

	result += calculate_directional_light();
    FragColor = vec4(result, 1.0);
}
