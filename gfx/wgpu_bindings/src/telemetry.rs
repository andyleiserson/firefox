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

    let limits = DefaultAdapterObjectItemLimitsObject {
        maxBindGroups: Some(max_bind_groups.into()),
        maxBindGroupsPlusVertexBuffers: None,
        maxImmediateSize: Some(max_immediate_size.into()),
        maxBindingsPerBindGroup: Some(max_bindings_per_bind_group.into()),
        maxBufferSize: Some(max_buffer_size.try_into().unwrap_or(i64::MAX)),
        maxColorAttachmentBytesPerSample: Some(max_color_attachment_bytes_per_sample.into()),
        maxColorAttachments: Some(max_color_attachments.into()),
        maxComputeInvocationsPerWorkgroup: Some(max_compute_invocations_per_workgroup.into()),
        maxComputeWorkgroupSizeX: Some(max_compute_workgroup_size_x.into()),
        maxComputeWorkgroupSizeY: Some(max_compute_workgroup_size_y.into()),
        maxComputeWorkgroupSizeZ: Some(max_compute_workgroup_size_z.into()),
        maxComputeWorkgroupStorageSize: Some(max_compute_workgroup_storage_size.into()),
        maxComputeWorkgroupsPerDimension: Some(max_compute_workgroups_per_dimension.into()),
        maxDynamicStorageBuffersPerPipelineLayout: Some(
            max_dynamic_storage_buffers_per_pipeline_layout.into(),
        ),
        maxDynamicUniformBuffersPerPipelineLayout: Some(
            max_dynamic_uniform_buffers_per_pipeline_layout.into(),
        ),
        maxInterStageShaderVariables: Some(max_inter_stage_shader_variables.into()),
        maxSampledTexturesPerShaderStage: Some(max_sampled_textures_per_shader_stage.into()),
        maxSamplersPerShaderStage: Some(max_samplers_per_shader_stage.into()),
        maxStorageBuffersInFragmentStage: None,
        maxStorageBuffersInVertexStage: None,
        maxStorageBuffersPerShaderStage: Some(max_storage_buffers_per_shader_stage.into()),
        maxStorageTexturesInFragmentStage: None,
        maxStorageTexturesInVertexStage: None,
        maxStorageTexturesPerShaderStage: Some(max_storage_textures_per_shader_stage.into()),
        maxStorageBufferBindingSize: Some(
            max_storage_buffer_binding_size
                .try_into()
                .unwrap_or(i64::MAX),
        ),
        maxTextureArrayLayers: Some(max_texture_array_layers.into()),
        maxTextureDimension1D: Some(max_texture_dimension_1d.into()),
        maxTextureDimension2D: Some(max_texture_dimension_2d.into()),
        maxTextureDimension3D: Some(max_texture_dimension_3d.into()),
        maxUniformBufferBindingSize: Some(
            max_uniform_buffer_binding_size
                .try_into()
                .unwrap_or(i64::MAX),
        ),
        maxUniformBuffersPerShaderStage: Some(max_uniform_buffers_per_shader_stage.into()),
        maxVertexAttributes: Some(max_vertex_attributes.into()),
        maxVertexBuffers: Some(max_vertex_buffers.into()),
        maxVertexBufferArrayStride: Some(max_vertex_buffer_array_stride.into()),
        minStorageBufferOffsetAlignment: Some(min_storage_buffer_offset_alignment.into()),
        minUniformBufferOffsetAlignment: Some(min_uniform_buffer_offset_alignment.into()),
    };

    let info = DefaultAdapterObjectItemInfoObject {
        subgroupMinSize: Some(subgroup_min_size.into()),
        subgroupMaxSize: Some(subgroup_max_size.into()),
        deviceName: (!device_name.is_empty()).then(|| device_name.clone()),
        vendorId: NonZeroI64::new(vendor_id.into()).map(|v| v.get()),
        deviceId: NonZeroI64::new(device_id.into()).map(|v| v.get()),
        driverName: (!driver.is_empty()).then(|| driver.clone()),
        driverInfo: (!driver_info.is_empty()).then(|| driver_info.clone()),
        backend: Some(backend.to_str().into()),
        limitBucket: limit_bucket.as_ref().map(|b| b.name.to_string()),
    };

    let mut feature_strs =
        Vec::with_capacity(usize::try_from(features.bits().count_ones() as u64).unwrap());
    for f in features.iter() {
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
