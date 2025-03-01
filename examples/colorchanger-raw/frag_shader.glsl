#version 310 es
out mediump vec4 outputF;

uniform mediump float timeu;

void main()
{{
	outputF = vec4(normalize(vec3(abs(cos(timeu)),{},{})),1.0);
}};