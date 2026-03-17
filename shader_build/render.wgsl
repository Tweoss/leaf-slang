@binding(1) @group(0) var output_0 : texture_storage_2d<rgba8unorm, write>;

struct Petal_std430_0
{
    @align(16) pos_0 : vec3<f32>,
    @align(16) xaxis_0 : vec3<f32>,
    @align(16) yaxis_0 : vec3<f32>,
    @align(16) texture_offset_0 : vec2<u32>,
    @align(8) texture_size_0 : vec2<u32>,
};

@binding(2) @group(0) var<storage, read> petals_0 : array<Petal_std430_0>;

@binding(0) @group(0) var input_0 : texture_2d<f32>;

struct GlobalParams_std140_0
{
    @align(16) petalCount_0 : u32,
    @align(8) cellSize_0 : vec2<u32>,
};

@binding(3) @group(0) var<uniform> globalParams_0 : GlobalParams_std140_0;
struct Camera_0
{
     focalLength_0 : f32,
};

fn Camera_x24init_0( focalLength_1 : f32) -> Camera_0
{
    var _S1 : Camera_0;
    _S1.focalLength_0 = focalLength_1;
    return _S1;
}

struct Ray_0
{
     origin_0 : vec3<f32>,
     direction_0 : vec3<f32>,
     tRange_0 : vec2<f32>,
};

fn Ray_x24init_0( origin_1 : vec3<f32>,  direction_1 : vec3<f32>,  tRange_1 : vec2<f32>) -> Ray_0
{
    var _S2 : Ray_0;
    _S2.origin_0 = origin_1;
    _S2.direction_0 = direction_1;
    _S2.tRange_0 = tRange_1;
    return _S2;
}

fn Camera_generateRay_0( this_0 : Camera_0,  uv_0 : vec2<f32>,  canvasSize_0 : vec2<f32>) -> Ray_0
{
    return Ray_x24init_0(vec3<f32>(0.0f), vec3<f32>((uv_0 - vec2<f32>(0.5f)) * canvasSize_0 / vec2<f32>(this_0.focalLength_0), -1.0f), vec2<f32>(0.0f, 3.4028234663852886e+38f));
}

struct RayHitResult_0
{
     normal_0 : vec3<f32>,
     t_0 : f32,
     uv_1 : vec2<f32>,
};

fn RayHitResult_x24init_0( normal_1 : vec3<f32>,  t_1 : f32,  uv_2 : vec2<f32>) -> RayHitResult_0
{
    var _S3 : RayHitResult_0;
    _S3.normal_0 = normal_1;
    _S3.t_0 = t_1;
    _S3.uv_1 = uv_2;
    return _S3;
}

struct _slang_Optional_RayHitResult_0
{
     value_0 : RayHitResult_0,
     hasValue_0 : bool,
};

fn Petal_hit_0( this_1 : ptr<function, Petal_std430_0>,  ray_0 : Ray_0) -> _slang_Optional_RayHitResult_0
{
    var normal_2 : vec3<f32> = normalize(cross((*this_1).xaxis_0, (*this_1).yaxis_0));
    var t_2 : f32 = (dot((*this_1).pos_0, normal_2) - dot(ray_0.origin_0, normal_2)) / dot(ray_0.direction_0, normal_2);
    var _S4 : vec3<f32> = ray_0.origin_0 + vec3<f32>(t_2) * ray_0.direction_0 - (*this_1).pos_0;
    var _S5 : f32 = length((*this_1).xaxis_0);
    var _S6 : f32 = length((*this_1).yaxis_0);
    var uv_3 : vec2<f32> = vec2<f32>(dot(_S4, (*this_1).xaxis_0) / _S5, dot(_S4, (*this_1).yaxis_0) / _S6) / vec2<f32>(_S5, _S6);
    var _S7 : bool;
    if((all((uv_3 >= vec2<f32>(0.0f)))))
    {
        _S7 = (all((uv_3 < vec2<f32>(1.0f))));
    }
    else
    {
        _S7 = false;
    }
    if(_S7)
    {
        _S7 = (ray_0.tRange_0[i32(0)]) <= t_2;
    }
    else
    {
        _S7 = false;
    }
    if(_S7)
    {
        _S7 = (ray_0.tRange_0[i32(1)]) >= t_2;
    }
    else
    {
        _S7 = false;
    }
    if(_S7)
    {
        var _S8 : _slang_Optional_RayHitResult_0 = _slang_Optional_RayHitResult_0( RayHitResult_x24init_0(normal_2, t_2, uv_3), true );
        return _S8;
    }
    var _S9 : RayHitResult_0 = RayHitResult_0( vec3<f32>(0.0f), 0.0f, vec2<f32>(0.0f) );
    var _S10 : _slang_Optional_RayHitResult_0 = _slang_Optional_RayHitResult_0( _S9, false );
    return _S10;
}

struct Petal_0
{
     pos_0 : vec3<f32>,
     xaxis_0 : vec3<f32>,
     yaxis_0 : vec3<f32>,
     texture_offset_0 : vec2<u32>,
     texture_size_0 : vec2<u32>,
};

struct PetalHitResult_0
{
     petal_0 : Petal_0,
     r_0 : RayHitResult_0,
};

fn PetalHitResult_x24init_0( petal_1 : ptr<function, Petal_std430_0>,  r_1 : RayHitResult_0) -> PetalHitResult_0
{
    var _S11 : PetalHitResult_0;
    _S11.petal_0.pos_0 = (*petal_1).pos_0;
    _S11.petal_0.xaxis_0 = (*petal_1).xaxis_0;
    _S11.petal_0.yaxis_0 = (*petal_1).yaxis_0;
    _S11.petal_0.texture_offset_0 = (*petal_1).texture_offset_0;
    _S11.petal_0.texture_size_0 = (*petal_1).texture_size_0;
    _S11.r_0 = r_1;
    return _S11;
}

struct _slang_Optional_PetalHitResult_0
{
     value_1 : PetalHitResult_0,
     hasValue_1 : bool,
};

fn hitPetals_0( ray_1 : Ray_0) -> _slang_Optional_PetalHitResult_0
{
    var _S12 : vec3<f32> = vec3<f32>(0.0f);
    var _S13 : vec2<u32> = vec2<u32>(u32(0));
    var _S14 : Petal_0 = Petal_0( _S12, _S12, _S12, _S13, _S13 );
    var _S15 : RayHitResult_0 = RayHitResult_0( _S12, 0.0f, vec2<f32>(0.0f) );
    var _S16 : PetalHitResult_0 = PetalHitResult_0( _S14, _S15 );
    var outv_0 : _slang_Optional_PetalHitResult_0;
    outv_0.value_1 = _S16;
    outv_0.hasValue_1 = false;
    var i_0 : i32 = i32(0);
    for(;;)
    {
        if(u32(i_0) < (globalParams_0.petalCount_0))
        {
        }
        else
        {
            break;
        }
        var _S17 : Petal_std430_0 = petals_0[i_0];
        var _S18 : _slang_Optional_RayHitResult_0 = Petal_hit_0(&(_S17), ray_1);
        var _S19 : bool;
        if(_S18.hasValue_0)
        {
            if(!outv_0.hasValue_1)
            {
                _S19 = true;
            }
            else
            {
                _S19 = (outv_0.value_1.r_0.t_0) > (_S18.value_0.t_0);
            }
        }
        else
        {
            _S19 = false;
        }
        if(_S19)
        {
            var _S20 : PetalHitResult_0 = PetalHitResult_x24init_0(&(_S17), _S18.value_0);
            outv_0.value_1 = _S20;
            outv_0.hasValue_1 = true;
        }
        i_0 = i_0 + i32(1);
    }
    return outv_0;
}

fn bilinearSample_0( input_1 : texture_2d<f32>,  uv_4 : vec2<f32>) -> vec4<f32>
{
    var width_0 : f32;
    var height_0 : f32;
    {var dim = textureDimensions((input_1));((width_0)) = f32(dim.x);((height_0)) = f32(dim.y);};
    var size_0 : vec2<f32> = vec2<f32>(width_0, height_0);
    var coord_0 : vec2<f32> = max(uv_4 * size_0 - vec2<f32>(0.5f), vec2<f32>(vec2<i32>(i32(0))));
    var _S21 : vec2<u32> = vec2<u32>(floor(coord_0));
    var w_0 : vec2<f32> = coord_0 - vec2<f32>(_S21);
    var wi_0 : vec2<f32> = vec2<f32>(1.0f) - w_0;
    var _S22 : vec3<i32> = vec3<i32>(vec3<u32>(_S21, u32(0)));
    var _S23 : vec2<u32> = vec2<u32>(size_0) - vec2<u32>(vec2<i32>(i32(1)));
    var _S24 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S21 + vec2<u32>(u32(0), u32(1)), _S23), u32(0)));
    var _S25 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S21 + vec2<u32>(u32(1), u32(0)), _S23), u32(0)));
    var _S26 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S21 + vec2<u32>(u32(1), u32(1)), _S23), u32(0)));
    var _S27 : vec4<f32> = vec4<f32>(w_0.x);
    var _S28 : vec4<f32> = vec4<f32>(wi_0.x);
    return ((textureLoad((input_1), ((_S26)).xy, ((_S26)).z)) * _S27 + (textureLoad((input_1), ((_S24)).xy, ((_S24)).z)) * _S28) * vec4<f32>(w_0.y) + ((textureLoad((input_1), ((_S25)).xy, ((_S25)).z)) * _S27 + (textureLoad((input_1), ((_S22)).xy, ((_S22)).z)) * _S28) * vec4<f32>(wi_0.y);
}

fn bilinearSampleRange_0( input_2 : texture_2d<f32>,  offset_0 : vec2<u32>,  size_1 : vec2<u32>,  uv_5 : vec2<f32>) -> vec4<f32>
{
    var width_1 : f32;
    var height_1 : f32;
    {var dim = textureDimensions((input_2));((width_1)) = f32(dim.x);((height_1)) = f32(dim.y);};
    return bilinearSample_0(input_2, (vec2<f32>(offset_0) + uv_5 * vec2<f32>(size_1)) / vec2<f32>(width_1, height_1));
}

fn Petal_sample_tex_0( this_2 : Petal_0,  uv_6 : vec2<f32>,  input_3 : texture_2d<f32>) -> vec4<f32>
{
    return bilinearSampleRange_0(input_3, this_2.texture_offset_0, this_2.texture_size_0, uv_6);
}

fn getDim_0() -> vec2<u32>
{
    var width_2 : u32;
    var height_2 : u32;
    {var dim = textureDimensions((output_0));((width_2)) = dim.x;((height_2)) = dim.y;};
    return vec2<u32>(width_2, height_2);
}

@compute
@workgroup_size(16, 16, 1)
fn imagemain(@builtin(global_invocation_id) dispatchThreadID_0 : vec3<u32>)
{
    var dispatchThreadID_1 : vec2<u32> = dispatchThreadID_0.xy;
    var _S29 : vec2<u32> = getDim_0();
    if((any((dispatchThreadID_1 >= _S29))))
    {
        return;
    }
    textureStore((output_0), (dispatchThreadID_1), (vec4<f32>(1.0f, 1.0f, 1.0f, 1.0f)));
    var _S30 : vec2<f32> = vec2<f32>(_S29);
    var ray_2 : Ray_0 = Camera_generateRay_0(Camera_x24init_0(f32(_S29.x)), vec2<f32>(dispatchThreadID_1) / _S30, _S30);
    for(;;)
    {
        var hit_0 : _slang_Optional_PetalHitResult_0 = hitPetals_0(ray_2);
        if(hit_0.hasValue_1)
        {
            var outv_1 : vec4<f32> = Petal_sample_tex_0(hit_0.value_1.petal_0, hit_0.value_1.r_0.uv_1, input_0);
            if((outv_1.w) == 0.0f)
            {
                ray_2.tRange_0[i32(0)] = hit_0.value_1.r_0.t_0 + 0.00009999999747379f;
                continue;
            }
            textureStore((output_0), (dispatchThreadID_1), (outv_1));
            return;
        }
        else
        {
            break;
        }
    }
    textureStore((output_0), (dispatchThreadID_1), (vec4<f32>(vec3<f32>(1.0f), 1.0f)));
    return;
}

