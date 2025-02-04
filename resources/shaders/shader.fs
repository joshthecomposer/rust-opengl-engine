#version 330 core

out vec4 FragColor;

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoords;
in vec4 FragPosLightSpace;

uniform vec3 point_light_color;
uniform vec3 normal_color;

uniform sampler2D shadow_map;

struct Material {
	sampler2D diffuse;
	sampler2D specular;

	float shininess;
};
uniform Material material;

struct PointLight {
 	vec3 position;

	vec3 ambient;
	vec3 diffuse;
	vec3 specular;

	float constant;
	float linear;
	float quadratic;
};
#define NR_POINT_LIGHTS 4
uniform PointLight point_lights[NR_POINT_LIGHTS];

struct DirLight {
 	vec3 direction;

	vec3 ambient;
	vec3 diffuse;
	vec3 specular;
};
uniform DirLight dir_light;

uniform vec3 ViewPosition;

vec3 calculate_point_light(PointLight light) {
	// Set some attenuation.
	float constant = light.constant;
	float linear = light.linear;
	float quadratic = light.quadratic;

	float distance    = length(light.position - FragPos);
	float attenuation = 1.0 / (constant + linear * distance + quadratic * (distance * distance));  

    // ambient
	vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoords));
  	
	
    // diffuse 
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(light.position - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
	vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoords));  
    
    // specular
    vec3 viewDir = normalize(ViewPosition - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
	vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoords));

	ambient  *= attenuation;
	diffuse  *= attenuation;
	specular *= attenuation;
        
    return (ambient + diffuse + specular);
}

vec3 calculate_directional_light() {
	// Ambient
	vec3 ambient = dir_light.ambient * texture(material.diffuse, TexCoords).rgb;

	// Diffuse
	vec3 norm = normalize(Normal);
	vec3 lightDir = normalize(dir_light.direction);
	float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = dir_light.diffuse * diff * texture(material.diffuse, TexCoords).rgb;

	// Specular
	vec3 viewDir = normalize(ViewPosition - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = dir_light.specular * spec * texture(material.specular, TexCoords).rgb; 

	return (ambient + diffuse + specular);
}

void main() {
	vec3 result = vec3(0.0f, 0.0f, 0.0f); 

	for(int i= 0; i < NR_POINT_LIGHTS; i++) {
		result += calculate_point_light(point_lights[i]);
	}

	// result += calculate_directional_light();

    FragColor = vec4(result, 1.0);
}

