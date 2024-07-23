#version 400 core

layout (location = 0) in vec3 aPos;

uniform mat4 lightSpaceMatrix;

out vec4 FragPosLightSpace;

void main()
{
    vec4 worldPosition = vec4(aPos, 1.0);
    FragPosLightSpace = lightSpaceMatrix * worldPosition;
    gl_Position = FragPosLightSpace;
}
