// WebGPU default limits from <https://gpuweb.github.io/gpuweb/#limits>
const WEBGPU_DEFAULT_LIMITS = {
  maxTextureDimension1D: 8192,
  maxTextureDimension2D: 8192,
  maxTextureDimension3D: 2048,
  maxTextureArrayLayers: 256,
  maxBindGroups: 4,
  maxBindGroupsPlusVertexBuffers: 12,
  maxBindingsPerBindGroup: 1000,
  maxDynamicUniformBuffersPerPipelineLayout: 8,
  maxDynamicStorageBuffersPerPipelineLayout: 4,
  maxSampledTexturesPerShaderStage: 16,
  maxSamplersPerShaderStage: 16,
  maxStorageBuffersPerShaderStage: 8,
  maxStorageBuffersInVertexStage: 8,
  maxStorageBuffersInFragmentStage: 8,
  maxStorageTexturesPerShaderStage: 4,
  maxStorageTexturesInVertexStage: 4,
  maxStorageTexturesInFragmentStage: 4,
  maxUniformBuffersPerShaderStage: 12,
  maxUniformBufferBindingSize: 65536,
  maxStorageBufferBindingSize: 134217728,
  maxVertexBuffers: 8,
  maxBufferSize: 268435456,
  maxVertexAttributes: 16,
  maxVertexBufferArrayStride: 2048,
  maxInterStageShaderVariables: 16,
  minUniformBufferOffsetAlignment: 256,
  minStorageBufferOffsetAlignment: 256,
  maxColorAttachments: 8,
  maxColorAttachmentBytesPerSample: 32,
  maxComputeWorkgroupStorageSize: 16384,
  maxComputeInvocationsPerWorkgroup: 256,
  maxComputeWorkgroupSizeX: 256,
  maxComputeWorkgroupSizeY: 256,
  maxComputeWorkgroupSizeZ: 64,
  maxComputeWorkgroupsPerDimension: 65535,
};

// "Unlimited" limit value used by noop backend.
const NOOP_UNLIMITED = 2147483647;

// Limit values that are reduced from the noop backend defaults by `restrict_limits`
// in `gfx/wgpu_bindings/src/server.rs`.
const MAX_TEXTURE_DIMENSION = 32767;
const MAX_BUFFER_SIZE = 1073741824;

const EXPECTED_ADAPTER_INFO_KEYS = [
  "vendor",
  "architecture",
  "device",
  "description",
  "subgroupMinSize",
  "subgroupMaxSize",
  "isFallbackAdapter",
];
