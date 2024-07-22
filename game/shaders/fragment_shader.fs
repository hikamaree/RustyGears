#version 400 core

in vec2 TexCoords;
in vec3 FragPos;
in vec3 Normal;
in vec4 FragPosLightSpace[4];  // Max 4 svetla

out vec4 FragColor;

uniform sampler2D texture_diffuse1;
uniform sampler2D shadowMaps[4];

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
uniform LightSource lightSources[4];

float ShadowCalculation(vec4 fragPosLightSpace, sampler2D shadowMap)
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
    vec3 result = ambient;

    for (int i = 0; i < 4; ++i) {
        vec3 lightDir = normalize(lightSources[i].position - FragPos);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = lightSources[i].color * diff * lightSources[i].intensity;

        float shadow = ShadowCalculation(FragPosLightSpace[i], shadowMaps[i]);
        result += (1.0 - shadow) * diffuse;
    }

    vec3 textureColor = texture(texture_diffuse1, TexCoords).rgb;
    FragColor = vec4(result * textureColor, 1.0);
}
