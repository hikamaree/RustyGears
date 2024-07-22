#version 330 core
out vec4 FragColor;

in vec2 TexCoords;
in vec4 FragPosLightSpace;
in vec3 Normal;
in vec3 FragPos;

uniform sampler2D texture_diffuse1;
uniform sampler2D shadowMap;

struct AmbientLight {
    vec3 color;
    float intensity;
};
uniform AmbientLight ambientLight;

struct LightSource {
    vec3 position;
    vec3 color;
    float intensity;
};
uniform LightSource lightSource;

float ShadowCalculation(vec4 fragPosLightSpace)
{
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;
    float closestDepth = texture(shadowMap, projCoords.xy).r;
    float currentDepth = projCoords.z;
    float shadow = currentDepth > closestDepth + 0.005 ? 1.0 : 0.0;
    return shadow;
}

void main()
{
    vec3 ambient = ambientLight.color * ambientLight.intensity;
    
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightSource.position - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = lightSource.color * diff * lightSource.intensity;

    float shadow = ShadowCalculation(FragPosLightSpace);
    vec3 lighting = (ambient + (1.0 - shadow) * diffuse) * texture(texture_diffuse1, TexCoords).rgb;
    FragColor = vec4(lighting, 1.0);
}
