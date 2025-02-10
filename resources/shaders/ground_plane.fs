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

float ShadowCalculation(float dot_light_normal) {
	vec3 pos = FragPosLightSpace.xyz * 0.5 + 0.5;

	// if (pos.x < 0.0 || pos.x > 1.0 || pos.y < 0.0 || pos.y > 1.0 || pos.z < 0.0 || pos.z > 1.0) {
	// 	return 1.0;
	// }
	if (pos.z > 1.0) {
		pos.z = 1.0;
	}

	float bias = max(0.05 * (1.0 - dot_light_normal), 0.005);

	float shadow = 0.0;
	vec2 texel_size = 1.0 / textureSize(shadow_map, 0);
	for (int x = -1; x <= 1; ++x) {
		for (int y = -1; y <=1; ++y) {
			float depth = texture(shadow_map, pos.xy + vec2(x, y) * texel_size).r;
			shadow += (depth + bias) < pos.z ? 0.0 : 1.0;
		}
	}

	return shadow / 9.0; 

}

vec3 calculate_directional_light() {
    vec3 lightColor = dir_light.diffuse;
	vec3 tex_color = ground_color;

	// Ambient
    vec3 ambient = vec3(dir_light.ambient) * tex_color;
	
	// Diffuse
    // vec3 lightDir = normalize(dir_light.view_pos - FragPos);
    vec3 lightDir = normalize(dir_light.direction);
    vec3 norm = normalize(Normal);
	float dot_light_normal = dot(lightDir, norm);
    float diff = max(dot_light_normal, 0.0);
    vec3 diffuse = diff * lightColor;

	float shadow = ShadowCalculation(dot_light_normal);

    return (shadow * (diffuse) + ambient) * tex_color;
}

void main() {
	vec3 result = calculate_directional_light();
    FragColor = vec4(result, 1.0);
}

