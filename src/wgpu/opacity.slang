import sample;

[format("rgba8")]
layout(binding = 0) Texture2D<float4> inputWhite;

[format("rgba8")]
layout(binding = 1) Texture2D<float4> inputBlack;

[format("rgba8")]
layout(binding = 2) WTexture2D<float4> outputTex;

// bbox is starting from bottom left of image: minx, miny, maxx, maxy
uniform int2 whiteBboxOffset;
uniform float4 blackBbox;
uniform float2 blackTranslation;
uniform float blackRotation;

[shader("compute")]
[numthreads(16, 16, 1)]
void imageMain(uint2 dispatchThreadID: SV_DispatchThreadID)
{
	uint2 outputDim = getDim(outputTex);
	uint2 whiteDim = getDim(inputWhite);
	uint2 blackDim = getDim(inputBlack);

	int2 whiteBase = whiteBboxOffset;
	// Bounds check.
	if (any(dispatchThreadID >= outputDim)) {
		return;
	}
	// Don't invert image.
	int2 whiteCoord = flipY(outputDim, dispatchThreadID) + whiteBase;
	// but we do have to flip because bottom left was (0,0) in plot space
	// and we want top left for texture reading.
	float4 whiteV = inputWhite[flipY(whiteDim, whiteCoord)];

	// Order was rotation around center of image, then translation.
	// 
	// Use inverses to find corresponding pixel => reverse order.
	// So, need to untranslate, move to origin, rotate, move from origin
 
	// Columns.
	float2 center = float2(blackDim) / 2.0;
	float2x2 rot_inv = float2x2(float2(cos(blackRotation), sin(blackRotation)), float2(-sin(blackRotation), cos(blackRotation)));
	float2 blackCoordf = mul(rot_inv, float2(whiteCoord) - blackTranslation - center) + center;
	if (!is_in_bounds(blackDim, blackCoordf)) {
		outputTex.Store(dispatchThreadID, float4(0.0));
	}
	float2 blackMin = float2(blackBbox[0], blackBbox[1]);
	float2 blackMax = float2(blackBbox[2], blackBbox[3]);
	if (any(blackCoordf < blackMin) || any(blackCoordf > blackMax)) {
		outputTex.Store(dispatchThreadID, float4(0.0));
	}
	float4 blackV = bilinearSample(inputBlack, flipY(blackDim, blackCoordf) / float2(blackDim));

	// So stuff doesn't get optimized away
	// TODO: remove
	outputTex.Store(dispatchThreadID, float4(whiteV.rgb, 1.0));
	outputTex.Store(dispatchThreadID, float4(blackV.rgb, 1.0));

	// if big diff, low opacity
	
	float3 diff = (whiteV - blackV).rgb;
	float m = max(max(diff.r, diff.g), diff.b);
	// Greater than threshoald => must've been (white - black), so
	// not a petal.
	float THRESHOLD = 0.55;
	if (m > THRESHOLD) {
		outputTex.Store(dispatchThreadID, float4(0.0));
		return;
	}
	float4 output = float4(whiteV.rgb, 1.0 - m);

	outputTex.Store(dispatchThreadID, output);

	return ;
}
