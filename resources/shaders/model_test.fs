#version 330 core
out vec4 FragColor;

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoords;
in vec4 FragPosLightSpace;

uniform sampler2D texture_diffuse1;
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

float ShadowCalculation(vec4 fragPosLightSpace) {
    // perform perspective divide
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;

    // transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;
	projCoords.xy = clamp(projCoords.xy, 0.0, 1.0);
    // get closest depth value from light's perspective (using [0,1] range fragPosLight as coords)
    float closestDepth = texture(shadow_map, projCoords.xy).r; 
    // get depth of current fragment from light's perspective
    float currentDepth = projCoords.z;
    // calculate bias (based on depth map resolution and slope)
    vec3 normal = normalize(Normal);
    vec3 lightDir = normalize(dir_light.view_pos - FragPos);
	// vec3 lightDir = normalize(-dir_light.direction);
	float bias = max(0.05 * (1.0 - dot(normal, lightDir)), 0.0005);
    // float bias = 1.0;
    // check whether current frag pos is in shadow
    // float shadow = currentDepth - bias > closestDepth  ? 1.0 : 0.0;
    // PCF
	// shadow = currentDepth - bias > closestDepth ? 1.0 : 0.0;
	float shadow = 0.0;
	vec2 texelSize = 1.0 / textureSize(shadow_map, 0);

	for(int x = -2; x <= 2; ++x)  // Expanded kernel range
	{
		for(int y = -2; y <= 2; ++y)
		{
			float pcfDepth = texture(shadow_map, projCoords.xy + vec2(x, y) * texelSize).r;
			shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
		}    
	}
	shadow /= 25.0;  // Normalize for 5x5 kernel
    
    // keep the shadow at 0.0 when outside the far_plane region of the light's frustum.
    // if(projCoords.z > 1.0)
        ///shadow = 0.0;
        
    return shadow;
}

vec3 calculate_directional_light() {

	vec3 diffuseColor = texture(texture_diffuse1, TexCoords).rgb;
	if (diffuseColor == vec3(0.0)) {
		diffuseColor = vec3(1.0); // Default white color
	}
	// Ambient
	vec3 ambient = dir_light.ambient * diffuseColor;

	// Diffuse
	vec3 norm = normalize(Normal);
	vec3 lightDir = normalize(dir_light.direction);
	float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = dir_light.diffuse * diff * diffuseColor;

	// Specular
	vec3 viewDir = normalize(ViewPosition - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 64.0);
    // vec3 specular = dir_light.specular * spec * texture(material.specular, TexCoords).rgb; 
	float shadow = ShadowCalculation(FragPosLightSpace);

	// return (ambient + (1.0 - shadow) * (diffuse)) * texture(texture_diffuse1, TexCoords).rgb;
	 return (ambient + diffuse);
}

void main() {    
	vec3 result = vec3(0.0f, 0.0f, 0.0f); 

	result += calculate_directional_light();
    // vec3 n = normalize(Normal);
    // FragColor = vec4(n * 0.5 + 0.5, 1.0); 
   FragColor = vec4(result, 1.0);
}
