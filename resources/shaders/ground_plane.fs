#version 330 core

out vec4 FragColor;

in vec3 FragPos;
in vec3 Normal;
in vec4 FragPosLightSpace;

uniform vec3 point_light_color;

uniform sampler2D shadow_map;

struct DirLight {
 	vec3 direction;
	vec3 view_pos;

	vec3 ambient;
	vec3 diffuse;
	vec3 specular;
};
uniform DirLight dir_light;

uniform vec3 ViewPosition;
uniform vec3 ground_color;

float ShadowCalculation(vec4 fragPosLightSpace) {
	vec3 pos = FragPosLightSpace.xyz * 0.5 + 0.5;
	float depth = texture(shadow_map, pos.xy).r;
	return depth < pos.z ? 1.0 : 0.0;
}

vec3 calculate_directional_light() {
	// Ambient
	vec3 ambient = dir_light.ambient * ground_color;

	// Diffuse
	vec3 norm = normalize(Normal);
	vec3 lightDir = normalize(dir_light.direction);
	float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = dir_light.diffuse * diff * ground_color;

	// Specular
	vec3 viewDir = normalize(ViewPosition - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 64.0);
    vec3 specular = dir_light.specular * spec * ground_color; 

	float shadow = ShadowCalculation(FragPosLightSpace);

	return (ambient + (1.0 - shadow) * (diffuse + specular)) * ground_color;
}

void main() {
	vec3 result = vec3(0.0f, 0.0f, 0.0f); 

	result += calculate_directional_light();

    // float depth = texture(shadow_map, FragPosLightSpace.xy).r;
    // FragColor = vec4(vec3(depth), 1.0);
    FragColor = vec4(result, 1.0);
	// FragColor = vec4(vec3(FragPosLightSpace.z), 1.0);
}

