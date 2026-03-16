@binding(2) @group(0) var outputTex_0 : texture_storage_2d<rgba8unorm, write>;

@binding(0) @group(0) var inputWhite_0 : texture_2d<f32>;

@binding(1) @group(0) var inputBlack_0 : texture_2d<f32>;

struct GlobalParams_std140_0
{
    @align(16) whiteBboxOffset_0 : vec2<i32>,
    @align(16) blackBbox_0 : vec4<f32>,
    @align(16) blackTranslation_0 : vec2<f32>,
    @align(8) blackRotation_0 : f32,
};

@binding(3) @group(0) var<uniform> globalParams_0 : GlobalParams_std140_0;
fn getDim_0( input_0 : texture_2d<f32>) -> vec2<u32>
{
    var width_0 : u32;
    var height_0 : u32;
    {var dim = textureDimensions((input_0));((width_0)) = dim.x;((height_0)) = dim.y;};
    return vec2<u32>(width_0, height_0);
}

fn flipY_0( bounds_0 : vec2<u32>,  point_0 : vec2<u32>) -> vec2<u32>
{
    return vec2<u32>(point_0.x, bounds_0.y - point_0.y - u32(1));
}

fn is_in_bounds_0( bounds_1 : vec2<u32>,  point_1 : vec2<f32>) -> bool
{
    return (all((point_1 < vec2<f32>(bounds_1))));
}

fn flipY_1( bounds_2 : vec2<u32>,  point_2 : vec2<f32>) -> vec2<f32>
{
    return vec2<f32>(point_2.x, f32(bounds_2.y) - point_2.y - 1.0f);
}

fn bilinearSample_0( input_1 : texture_2d<f32>,  uv_0 : vec2<f32>) -> vec4<f32>
{
    var width_1 : f32;
    var height_1 : f32;
    {var dim = textureDimensions((input_1));((width_1)) = f32(dim.x);((height_1)) = f32(dim.y);};
    var size_0 : vec2<f32> = vec2<f32>(width_1, height_1);
    var coord_0 : vec2<f32> = max(uv_0 * size_0 - vec2<f32>(0.5f), vec2<f32>(vec2<i32>(i32(0))));
    var _S1 : vec2<u32> = vec2<u32>(floor(coord_0));
    var w_0 : vec2<f32> = coord_0 - vec2<f32>(_S1);
    var wi_0 : vec2<f32> = vec2<f32>(1.0f) - w_0;
    var _S2 : vec3<i32> = vec3<i32>(vec3<u32>(_S1, u32(0)));
    var _S3 : vec2<u32> = vec2<u32>(size_0) - vec2<u32>(vec2<i32>(i32(1)));
    var _S4 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S1 + vec2<u32>(u32(0), u32(1)), _S3), u32(0)));
    var _S5 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S1 + vec2<u32>(u32(1), u32(0)), _S3), u32(0)));
    var _S6 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S1 + vec2<u32>(u32(1), u32(1)), _S3), u32(0)));
    var _S7 : vec4<f32> = vec4<f32>(w_0.x);
    var _S8 : vec4<f32> = vec4<f32>(wi_0.x);
    return ((textureLoad((input_1), ((_S6)).xy, ((_S6)).z)) * _S7 + (textureLoad((input_1), ((_S4)).xy, ((_S4)).z)) * _S8) * vec4<f32>(w_0.y) + ((textureLoad((input_1), ((_S5)).xy, ((_S5)).z)) * _S7 + (textureLoad((input_1), ((_S2)).xy, ((_S2)).z)) * _S8) * vec4<f32>(wi_0.y);
}

fn getDim_1() -> vec2<u32>
{
    var width_2 : u32;
    var height_2 : u32;
    {var dim = textureDimensions((outputTex_0));((width_2)) = dim.x;((height_2)) = dim.y;};
    return vec2<u32>(width_2, height_2);
}

@compute
@workgroup_size(16, 16, 1)
fn imageMain(@builtin(global_invocation_id) dispatchThreadID_0 : vec3<u32>)
{
    var dispatchThreadID_1 : vec2<u32> = dispatchThreadID_0.xy;
    var whiteDim_0 : vec2<u32> = getDim_0(inputWhite_0);
    var blackDim_0 : vec2<u32> = getDim_0(inputBlack_0);
    if((any((dispatchThreadID_1 >= (getDim_1())))))
    {
        return;
    }
    var _S9 : vec2<i32> = vec2<i32>(dispatchThreadID_1 + vec2<u32>(globalParams_0.whiteBboxOffset_0));
    var _S10 : vec3<i32> = vec3<i32>(vec3<u32>(flipY_0(whiteDim_0, vec2<u32>(_S9)), u32(0)));
    var _S11 : vec4<f32> = (textureLoad((inputWhite_0), ((_S10)).xy, ((_S10)).z));
    var blackCoordf_0 : vec2<f32> = (((vec2<f32>(_S9) - globalParams_0.blackTranslation_0) * (mat2x2<f32>(vec2<f32>(cos(globalParams_0.blackRotation_0), - sin(globalParams_0.blackRotation_0)), vec2<f32>(sin(globalParams_0.blackRotation_0), cos(globalParams_0.blackRotation_0))))));
    if(!is_in_bounds_0(blackDim_0, blackCoordf_0))
    {
        textureStore((outputTex_0), (dispatchThreadID_1), (vec4<f32>(0.0f)));
    }
    var blackMax_0 : vec2<f32> = vec2<f32>(globalParams_0.blackBbox_0[i32(2)], globalParams_0.blackBbox_0[i32(3)]);
    var _S12 : bool;
    if((any((blackCoordf_0 < vec2<f32>(globalParams_0.blackBbox_0[i32(0)], globalParams_0.blackBbox_0[i32(1)])))))
    {
        _S12 = true;
    }
    else
    {
        _S12 = (any((blackCoordf_0 > blackMax_0)));
    }
    if(_S12)
    {
        textureStore((outputTex_0), (dispatchThreadID_1), (vec4<f32>(0.0f)));
    }
    textureStore((outputTex_0), (dispatchThreadID_1), (vec4<f32>(_S11.xyz - bilinearSample_0(inputBlack_0, flipY_1(blackDim_0, blackCoordf_0) / vec2<f32>(blackDim_0)).xyz, 1.0f)));
    return;
}

