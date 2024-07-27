#version 400 core

out vec4 FragColor;

in vec2 TexCoords;
in vec4 FragPosLightSpace[10];
in vec3 Normal;
in vec3 FragPos;
in vec4 Color;

uniform sampler2D texture_diffuse1;
uniform sampler2D shadowMaps[10];

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
uniform LightSource lightSources[10];

struct Fog {
	vec3 color;
	float density;
};
uniform Fog fog;

uniform vec3 cameraPosition;
uniform int lightSourceNum;

float ShadowCalculation(vec4 fragPosLightSpace, int lightIndex) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;

    if (projCoords.z > 1.0) return 0.0;

    float closestDepth = texture(shadowMaps[lightIndex], projCoords.xy).r;
    float currentDepth = projCoords.z;
    float bias = max(0.005 * (1.0 - dot(Normal, normalize(lightSources[lightIndex].position - FragPos))), 0.005);
    float shadow = currentDepth - bias > closestDepth ? 1.0 : 0.0;

    float shadowSum = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMaps[lightIndex], 0);
    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadowMaps[lightIndex], projCoords.xy + vec2(x, y) * texelSize).r;
            shadowSum += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow = shadowSum / 9.0;

    return shadow;
}


void main() {
	vec3 ambient = ambientLight.color * ambientLight.intensity;

	vec3 norm = normalize(Normal);
	vec3 lighting = ambient;

	for (int i = 0; i < lightSourceNum; ++i) {
		vec3 lightDir = normalize(lightSources[i].position - FragPos);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = lightSources[i].color * diff * lightSources[i].intensity;
        float shadow = ShadowCalculation(FragPosLightSpace[i], i);
        lighting += (1.0 - shadow) * diffuse;
	}

	vec4 textureColor = texture(texture_diffuse1, TexCoords);
	vec3 color = Color.rgb * textureColor.rgb;
	color *= lighting;

	float distance = length(cameraPosition - FragPos);
	float fogFactor = 1.0 / (1.0 + fog.density * distance * distance);
	fogFactor = clamp(fogFactor, 0.0, 1.0);
	vec3 finalColor = mix(fog.color, color, fogFactor);

	FragColor = vec4(finalColor, Color.a * textureColor.a);
}
