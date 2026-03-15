@binding(0) @group(0) var outputImage_0 : texture_storage_2d<rgba8unorm, write>;

struct _MatrixStorage_float4x2_ColMajorstd140_0
{
    @align(16) data_0 : array<vec4<f32>, i32(2)>,
};

struct GlobalParams_std140_0
{
    @align(16) quad_points_0 : _MatrixStorage_float4x2_ColMajorstd140_0,
};

@binding(1) @group(0) var<uniform> globalParams_0 : GlobalParams_std140_0;
@compute
@workgroup_size(16, 16, 1)
fn imageMain(@builtin(global_invocation_id) dispatchThreadID_0 : vec3<u32>)
{
    var dispatchThreadID_1 : vec2<u32> = dispatchThreadID_0.xy;
    var width_0 : f32;
    var height_0 : f32;
    {var dim = textureDimensions((outputImage_0));((width_0)) = f32(dim.x);((height_0)) = f32(dim.y);};
    var p_0 : vec2<f32> = (vec2<f32>(dispatchThreadID_1.xy) * vec2<f32>(2.0f) - vec2<f32>(width_0, height_0)) / vec2<f32>(height_0);
    var _S1 : i32;
    if((p_0.x) > 0.0f)
    {
        _S1 = i32(1);
    }
    else
    {
        _S1 = i32(0);
    }
    var _S2 : i32;
    if((p_0.y) > 0.0f)
    {
        _S2 = i32(2);
    }
    else
    {
        _S2 = i32(0);
    }
    textureStore((outputImage_0), (dispatchThreadID_1), (vec4<f32>(mat4x2<f32>(globalParams_0.quad_points_0.data_0[i32(0)][i32(0)], globalParams_0.quad_points_0.data_0[i32(1)][i32(0)], globalParams_0.quad_points_0.data_0[i32(0)][i32(1)], globalParams_0.quad_points_0.data_0[i32(1)][i32(1)], globalParams_0.quad_points_0.data_0[i32(0)][i32(2)], globalParams_0.quad_points_0.data_0[i32(1)][i32(2)], globalParams_0.quad_points_0.data_0[i32(0)][i32(3)], globalParams_0.quad_points_0.data_0[i32(1)][i32(3)])[_S1 + _S2], 0.0f, 1.0f)));
    return;
}

