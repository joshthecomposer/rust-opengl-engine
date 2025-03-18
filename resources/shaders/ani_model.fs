#version 330 core
out vec4 FragColor;

in vec2 TexCoords;
in vec3 Normal;
in vec3 FragPos;
in vec4 FragPosLightSpace;

uniform sampler2D shadow_map;

struct DirLight {
 	vec3 direction;
	vec3 view_pos;

	vec3 ambient;
	vec3 diffuse;
	vec3 specular;
};
uniform DirLight dir_light;

struct Material {
	sampler2D Diffuse;
	sampler2D Specular;
	sampler2D Emissive;
};
uniform Material material;

uniform vec3 view_position;

float ShadowCalculation(float dot_light_normal) {
	vec3 pos = FragPosLightSpace.xyz * 0.5 + 0.5;
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

	vec3 tex_color = texture(material.Diffuse, TexCoords).rgb;

	// Ambient
    vec3 ambient = vec3(dir_light.ambient);
	
	// Diffuse
    // vec3 lightDir = normalize(dir_light.view_pos - FragPos);
	vec3 lightDir = normalize(dir_light.direction);
    vec3 norm = normalize(Normal);
	float dot_light_normal = dot(lightDir, norm);
    float diff = max(dot_light_normal, 0.0);
    vec3 diffuse = diff * lightColor;
	
	// Specular
	vec3 viewDir = normalize(view_position - FragPos);
	vec3 reflectDir = reflect(-lightDir, norm);
	float spec = pow(max(dot(viewDir, reflectDir), 0.0), 36.0);
	vec3 specular = dir_light.specular * spec * texture(material.Specular, TexCoords).rgb;

	// Emissive
	vec3 emissive = texture(material.Emissive, TexCoords).rgb;

	float shadow = ShadowCalculation(dot_light_normal);

	return ((shadow * (diffuse + specular)) + ambient) * tex_color + emissive;
   //  return (ambient + (shadow) * (diffuse + specular)) * tex_color + emissive;
}

void main()
{   
	vec3 result = calculate_directional_light();
	// vec3 normal_color = normalize(Normal) * 0.5 + 0.5;
    FragColor = vec4(result , 1.0);
}
