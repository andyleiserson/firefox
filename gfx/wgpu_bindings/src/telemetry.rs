/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Strings should have a maximum of 111 characters and be comprised of printable ASCII characters.

use std::num::NonZeroI64;

use firefox_on_glean::metrics::webgpu::{
    DefaultAdapterObjectItemInfoObject, DefaultAdapterObjectItemLimitsObject,
};
use nsstring::nsCString;
use serde_json::json;

extern "C" {
    fn wgpu_parent_report_adapter(is_fallback: bool, json: &nsCString);
}

pub fn get_wgpu_hooks() -> wgh::Telemetry {
    wgh::Telemetry {
        #[cfg(target_os = "windows")]
        d3d12_expose_adapter,
    }
}

#[cfg(target_os = "windows")]
fn d3d12_expose_adapter(
    desc: &windows::Win32::Graphics::Dxgi::DXGI_ADAPTER_DESC2,
    driver_version: Result<[u16; 4], windows::core::HRESULT>,
    result: wgh::D3D12ExposeAdapterResult,
) {
    // The key must be under 111 bytes:
    // - backend identifier 'D' = 1 byte
    // - 4 * u32's formatted as hex = 4 * 8 = 32 bytes
    // - 5 * ':' separators = 5 bytes
    // - 4 * u16's formatted as dec + 3 * '.' separators = 4 * 5 + 3 = 23 bytes OR
    //   'DVE:' + a u32 formatted as hex = 4 + 8 = 12 bytes
    // total = 61 or 50 bytes
    let key = match driver_version {
        Ok(driver_version) => &format!(
            "D:{:X}:{:X}:{:X}:{:X}:{}.{}.{}.{}",
            desc.VendorId,
            desc.DeviceId,
            desc.SubSysId,
            desc.Revision,
            driver_version[0],
            driver_version[1],
            driver_version[2],
            driver_version[3],
        ),
        Err(hresult) => &format!(
            "D:{:X}:{:X}:{:X}:{:X}:DVE:{:X}",
            desc.VendorId, desc.DeviceId, desc.SubSysId, desc.Revision, hresult.0
        ),
    };
    let category = match result {
        wgh::D3D12ExposeAdapterResult::CreateDeviceError(err) => match err {
            wgh::dx12::CreateDeviceError::GetProcAddress => "NONE:GetProcAddress",
            wgh::dx12::CreateDeviceError::D3D12CreateDevice(hresult) => {
                &format!("NONE:D3D12CreateDevice:{:X}", hresult.0)
            }
            wgh::dx12::CreateDeviceError::RetDeviceIsNull => "NONE:RetDeviceIsNull",
        },
        wgh::D3D12ExposeAdapterResult::UnknownFeatureLevel(fl) => {
            &format!("NONE:UNKNOWN_FL:{}", fl)
        }
        wgh::D3D12ExposeAdapterResult::ResourceBindingTier2Requirement => "NONE:REQ_RBT2",
        wgh::D3D12ExposeAdapterResult::ShaderModel6Requirement => "NONE:REQ_SM6",
        wgh::D3D12ExposeAdapterResult::Success(feature_level, shader_model) => {
            let feature_level = match feature_level {
                wgh::dx12::FeatureLevel::_11_0 => "11_0",
                wgh::dx12::FeatureLevel::_11_1 => "11_1",
                wgh::dx12::FeatureLevel::_12_0 => "12_0",
                wgh::dx12::FeatureLevel::_12_1 => "12_1",
                wgh::dx12::FeatureLevel::_12_2 => "12_2",
            };
            let shader_model = match shader_model {
                wgh::dx12::ShaderModel::_5_1 => "5.1",
                wgh::dx12::ShaderModel::_6_0 => "6.0",
                wgh::dx12::ShaderModel::_6_1 => "6.1",
                wgh::dx12::ShaderModel::_6_2 => "6.2",
                wgh::dx12::ShaderModel::_6_3 => "6.3",
                wgh::dx12::ShaderModel::_6_4 => "6.4",
                wgh::dx12::ShaderModel::_6_5 => "6.5",
                wgh::dx12::ShaderModel::_6_6 => "6.6",
                wgh::dx12::ShaderModel::_6_7 => "6.7",
                wgh::dx12::ShaderModel::_6_8 => "6.8",
                wgh::dx12::ShaderModel::_6_9 => "6.9",
            };
            &format!("SOME:FL{feature_level}:SM{shader_model}")
        }
    };
    firefox_on_glean::metrics::webgpu::expose_adapter
        .get(key, category)
        .add(1);
}

/// Helper to only report a limit if it differs from the adapter's limit bucket.
///
/// If `raw` is `None` (no limit bucket was applied), report `actual`.
/// If `raw` is `Some` and `actual` differs from `raw`, report `raw` (that is the useful information
/// for telemetry purposes -- `actual` is the bucket limit which is a known value).
/// Otherwise, do not report the limit.
fn maybe_report<T: Copy + PartialEq + TryInto<i64>>(actual: T, raw: Option<T>) -> Option<i64> {
    match raw {
        // A limit bucket was applied. Report the raw limit if it differs from the bucket's value
        // (which we already know, so isn't useful to report through telemetry).
        Some(raw) => (actual != raw).then_some(raw),
        // No limit bucket was applied. Report the device's actual limit.
        None => Some(actual),
    }
    .map(|report_val| report_val.try_into().unwrap_or(i64::MAX))
}

pub fn report_adapter(
    features: wgt::FeaturesWebGPU,
    info: &wgt::AdapterInfo,
    limits: &wgt::Limits,
) {
    // READ ME: This explicit destructuring is your reminder to update
    // the telemetry schema in `gfx/metrics.yaml`.
    let &wgt::Limits {
        max_bind_groups,
        // https://bugzilla.mozilla.org/show_bug.cgi?id=1967020
        //max_bind_groups_plus_vertex_buffers,
        max_immediate_size,
        max_bindings_per_bind_group,
        max_buffer_size,
        max_color_attachment_bytes_per_sample,
        max_color_attachments,
        max_compute_invocations_per_workgroup,
        max_compute_workgroup_size_x,
        max_compute_workgroup_size_y,
        max_compute_workgroup_size_z,
        max_compute_workgroup_storage_size,
        max_compute_workgroups_per_dimension,
        max_dynamic_storage_buffers_per_pipeline_layout,
        max_dynamic_uniform_buffers_per_pipeline_layout,
        max_inter_stage_shader_variables,
        // READ ME: This explicit destructuring is your reminder to update
        // the telemetry schema in `gfx/metrics.yaml`.
        max_sampled_textures_per_shader_stage,
        max_samplers_per_shader_stage,
        // `max_storage_{buffers,textures}_in_{vertex,fragment}_stage`:
        // https://bugzilla.mozilla.org/show_bug.cgi?id=2006720
        //max_storage_buffers_in_fragment_stage,
        //max_storage_buffers_in_vertex_stage,
        max_storage_buffers_per_shader_stage,
        //max_storage_textures_in_fragment_stage,
        //max_storage_textures_in_vertex_stage,
        max_storage_textures_per_shader_stage,
        max_storage_buffer_binding_size,
        max_texture_array_layers,
        max_texture_dimension_1d,
        max_texture_dimension_2d,
        max_texture_dimension_3d,
        max_uniform_buffer_binding_size,
        max_uniform_buffers_per_shader_stage,
        max_vertex_attributes,
        max_vertex_buffers,
        max_vertex_buffer_array_stride,
        min_storage_buffer_offset_alignment,
        min_uniform_buffer_offset_alignment,

        // READ ME: This explicit destructuring is your reminder to update
        // the telemetry schema in `gfx/metrics.yaml`.

        // Limits below this point relate to wgpu extensions that Firefox does not use.
        max_non_sampler_bindings: _,
        max_binding_array_elements_per_shader_stage: _,
        max_binding_array_sampler_elements_per_shader_stage: _,
        max_binding_array_acceleration_structure_elements_per_shader_stage: _,

        max_task_mesh_workgroup_total_count: _,
        max_task_mesh_workgroups_per_dimension: _,
        max_task_invocations_per_workgroup: _,
        max_task_invocations_per_dimension: _,
        max_mesh_invocations_per_workgroup: _,
        max_mesh_invocations_per_dimension: _,

        max_task_payload_size: _,
        max_mesh_output_vertices: _,
        max_mesh_output_primitives: _,
        max_mesh_output_layers: _,
        max_mesh_multiview_view_count: _,

        max_acceleration_structures_per_shader_stage: _,
        max_blas_primitive_count: _,
        max_blas_geometry_count: _,
        max_tlas_instance_count: _,

        max_multiview_view_count: _,
    } = limits;

    // READ ME: This explicit destructuring is your reminder to update
    // the telemetry schema in `gfx/metrics.yaml`.
    let &wgt::AdapterInfo {
        subgroup_min_size,
        subgroup_max_size,
        name: ref device_name,
        vendor: vendor_id,
        device: device_id,
        device_pci_bus_id: _,
        device_type,
        ref driver,
        ref driver_info,
        backend,
        transient_saves_memory: _,
        ref limit_bucket,
    } = info;

    let is_fallback = device_type == wgt::DeviceType::Cpu;

    let raw_limits = limit_bucket.as_ref().map(|b| &b.raw_limits);

    let limits = DefaultAdapterObjectItemLimitsObject {
        maxBindGroups: maybe_report(max_bind_groups, raw_limits.map(|r| r.max_bind_groups)),
        maxBindGroupsPlusVertexBuffers: None,
        maxImmediateSize: maybe_report(
            max_immediate_size,
            raw_limits.map(|r| r.max_immediate_size),
        ),
        maxBindingsPerBindGroup: maybe_report(
            max_bindings_per_bind_group,
            raw_limits.map(|r| r.max_bindings_per_bind_group),
        ),
        maxBufferSize: maybe_report(max_buffer_size, raw_limits.map(|r| r.max_buffer_size)),
        maxColorAttachmentBytesPerSample: maybe_report(
            max_color_attachment_bytes_per_sample,
            raw_limits.map(|r| r.max_color_attachment_bytes_per_sample),
        ),
        maxColorAttachments: maybe_report(
            max_color_attachments,
            raw_limits.map(|r| r.max_color_attachments),
        ),
        maxComputeInvocationsPerWorkgroup: maybe_report(
            max_compute_invocations_per_workgroup,
            raw_limits.map(|r| r.max_compute_invocations_per_workgroup),
        ),
        maxComputeWorkgroupSizeX: maybe_report(
            max_compute_workgroup_size_x,
            raw_limits.map(|r| r.max_compute_workgroup_size_x),
        ),
        maxComputeWorkgroupSizeY: maybe_report(
            max_compute_workgroup_size_y,
            raw_limits.map(|r| r.max_compute_workgroup_size_y),
        ),
        maxComputeWorkgroupSizeZ: maybe_report(
            max_compute_workgroup_size_z,
            raw_limits.map(|r| r.max_compute_workgroup_size_z),
        ),
        maxComputeWorkgroupStorageSize: maybe_report(
            max_compute_workgroup_storage_size,
            raw_limits.map(|r| r.max_compute_workgroup_storage_size),
        ),
        maxComputeWorkgroupsPerDimension: maybe_report(
            max_compute_workgroups_per_dimension,
            raw_limits.map(|r| r.max_compute_workgroups_per_dimension),
        ),
        maxDynamicStorageBuffersPerPipelineLayout: maybe_report(
            max_dynamic_storage_buffers_per_pipeline_layout,
            raw_limits.map(|r| r.max_dynamic_storage_buffers_per_pipeline_layout),
        ),
        maxDynamicUniformBuffersPerPipelineLayout: maybe_report(
            max_dynamic_uniform_buffers_per_pipeline_layout,
            raw_limits.map(|r| r.max_dynamic_uniform_buffers_per_pipeline_layout),
        ),
        maxInterStageShaderVariables: maybe_report(
            max_inter_stage_shader_variables,
            raw_limits.map(|r| r.max_inter_stage_shader_variables),
        ),
        maxSampledTexturesPerShaderStage: maybe_report(
            max_sampled_textures_per_shader_stage,
            raw_limits.map(|r| r.max_sampled_textures_per_shader_stage),
        ),
        maxSamplersPerShaderStage: maybe_report(
            max_samplers_per_shader_stage,
            raw_limits.map(|r| r.max_samplers_per_shader_stage),
        ),
        maxStorageBuffersInFragmentStage: None,
        maxStorageBuffersInVertexStage: None,
        maxStorageBuffersPerShaderStage: maybe_report(
            max_storage_buffers_per_shader_stage,
            raw_limits.map(|r| r.max_storage_buffers_per_shader_stage),
        ),
        maxStorageTexturesInFragmentStage: None,
        maxStorageTexturesInVertexStage: None,
        maxStorageTexturesPerShaderStage: maybe_report(
            max_storage_textures_per_shader_stage,
            raw_limits.map(|r| r.max_storage_textures_per_shader_stage),
        ),
        maxStorageBufferBindingSize: maybe_report(
            max_storage_buffer_binding_size,
            raw_limits.map(|r| r.max_storage_buffer_binding_size),
        ),
        maxTextureArrayLayers: maybe_report(
            max_texture_array_layers,
            raw_limits.map(|r| r.max_texture_array_layers),
        ),
        maxTextureDimension1D: maybe_report(
            max_texture_dimension_1d,
            raw_limits.map(|r| r.max_texture_dimension_1d),
        ),
        maxTextureDimension2D: maybe_report(
            max_texture_dimension_2d,
            raw_limits.map(|r| r.max_texture_dimension_2d),
        ),
        maxTextureDimension3D: maybe_report(
            max_texture_dimension_3d,
            raw_limits.map(|r| r.max_texture_dimension_3d),
        ),
        maxUniformBufferBindingSize: maybe_report(
            max_uniform_buffer_binding_size,
            raw_limits.map(|r| r.max_uniform_buffer_binding_size),
        ),
        maxUniformBuffersPerShaderStage: maybe_report(
            max_uniform_buffers_per_shader_stage,
            raw_limits.map(|r| r.max_uniform_buffers_per_shader_stage),
        ),
        maxVertexAttributes: maybe_report(
            max_vertex_attributes,
            raw_limits.map(|r| r.max_vertex_attributes),
        ),
        maxVertexBuffers: maybe_report(
            max_vertex_buffers,
            raw_limits.map(|r| r.max_vertex_buffers),
        ),
        maxVertexBufferArrayStride: maybe_report(
            max_vertex_buffer_array_stride,
            raw_limits.map(|r| r.max_vertex_buffer_array_stride),
        ),
        minStorageBufferOffsetAlignment: maybe_report(
            min_storage_buffer_offset_alignment,
            raw_limits.map(|r| r.min_storage_buffer_offset_alignment),
        ),
        minUniformBufferOffsetAlignment: maybe_report(
            min_uniform_buffer_offset_alignment,
            raw_limits.map(|r| r.min_uniform_buffer_offset_alignment),
        ),
    };

    let info = DefaultAdapterObjectItemInfoObject {
        subgroupMinSize: maybe_report(
            subgroup_min_size,
            limit_bucket.as_ref().map(|b| b.raw_subgroup_min_size),
        ),
        subgroupMaxSize: maybe_report(
            subgroup_max_size,
            limit_bucket.as_ref().map(|b| b.raw_subgroup_max_size),
        ),
        deviceName: (!device_name.is_empty()).then(|| device_name.clone()),
        vendorId: NonZeroI64::new(vendor_id.into()).map(|v| v.get()),
        deviceId: NonZeroI64::new(device_id.into()).map(|v| v.get()),
        driverName: (!driver.is_empty()).then(|| driver.clone()),
        driverInfo: (!driver_info.is_empty()).then(|| driver_info.clone()),
        backend: Some(backend.to_str().into()),
        limitBucket: limit_bucket.as_ref().map(|b| b.name.to_string()),
    };

    let report_features = if let Some(bucket) = limit_bucket {
        // A limit bucket was applied. Report any device features that are not included in the bucket.
        bucket.raw_features.features_webgpu.difference(features)
    } else {
        // No limit bucket was applied. Report the device's features.
        features
    };
    let mut feature_strs =
        Vec::with_capacity(usize::try_from(report_features.bits().count_ones() as u64).unwrap());
    for f in report_features.iter() {
        match wgt::Features::as_str(&f.into()) {
            Some(name) => feature_strs.push(name),
            None => {
                log::warn!("Unknown WebGPU feature bit: {f:?}");
            }
        }
    }

    let json = json!({
        "limits": limits,
        "info": info,
        "features": feature_strs,
    })
    .to_string();

    let json_ns = nsCString::from(&json);
    unsafe {
        wgpu_parent_report_adapter(is_fallback, &json_ns);
    }
}
