/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef GPU_SharedTextureDMABuf_H_
#define GPU_SharedTextureDMABuf_H_

#include "mozilla/gfx/FileHandleWrapper.h"
#include "mozilla/webgpu/ffi/wgpu.h"
#include "mozilla/webgpu/SharedTexture.h"
#include "mozilla/widget/DMABufSurface.h"
#include "nsTArrayForwardDeclare.h"

class DMABufSurface;

namespace mozilla {

namespace webgpu {

class SharedTextureDMABuf final : public SharedTexture {
 public:
  static UniquePtr<SharedTextureDMABuf> Create(
      WebGPUParent* aParent, const ffi::WGPUDeviceId aDeviceId,
      const uint32_t aWidth, const uint32_t aHeight,
      const struct ffi::WGPUTextureFormat aFormat,
      const ffi::WGPUTextureUsages aUsage);

  SharedTextureDMABuf(const uint32_t aWidth, const uint32_t aHeight,
                      const struct ffi::WGPUTextureFormat aFormat,
                      const ffi::WGPUTextureUsages aUsage,
                      RefPtr<DMABufSurface>&& aSurface,
                      const layers::SurfaceDescriptorDMABuf& aSurfaceDescriptor,
                      const DMABufInfo& aDMABufInfo);
  virtual ~SharedTextureDMABuf();

  Maybe<layers::SurfaceDescriptor> ToSurfaceDescriptor() override;

  void GetSnapshot(const ipc::Shmem& aDestShmem, size_t aDestStride) override;

  SharedTextureDMABuf* AsSharedTextureDMABuf() override { return this; }

  void onBeforeQueueSubmit(
      const ffi::WGPUGlobal* aContext, RawId aQueueId,
      nsTArray<ffi::WGPUVkSemaphoreHandle>& aSignalSemaphores) override;

  void CleanForRecycling() override;

  std::array<detail::FileHandleType, 4> CloneDmaBufFds();

  const DMABufInfo& GetDMABufInfo() const { return mDMABufInfo; }

 protected:
  RefPtr<DMABufSurface> mSurface;
  const layers::SurfaceDescriptorDMABuf mSurfaceDescriptor;
  const DMABufInfo mDMABufInfo;
  RefPtr<gfx::FileHandleWrapper> mSemaphoreFd;
};

}  // namespace webgpu
}  // namespace mozilla

#endif  // GPU_Texture_H_
