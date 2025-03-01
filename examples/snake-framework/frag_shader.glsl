#version 310 es

in mediump vec4 vertexColor;
out mediump vec4 fragColor;
void main()
{
	fragColor = vertexColor;
}