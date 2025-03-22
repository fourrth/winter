#version 320 es
precision lowp float;

out vec4 outputF;
in vec4 vertexColor;

layout (location = 2) uniform float timeu;

void main()
{
	outputF = abs(sin(timeu)) * vertexColor;
}