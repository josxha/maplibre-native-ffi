#include <cstdint>
#include <cstring>
#include <stdexcept>
#include <string>
#include <vector>

#include <mbgl/util/image.hpp>
#include <mbgl/util/premultiply.hpp>

#include <android/imagedecoder.h>

namespace mbgl {
namespace {

void checkDecoderResult(int result, const char* operation) {
  if (result != ANDROID_IMAGE_DECODER_SUCCESS) {
    throw std::runtime_error(
      std::string(operation) + " failed with code " + std::to_string(result)
    );
  }
}

}  // namespace

PremultipliedImage decodeImage(const std::string& string) {
  AImageDecoder* decoder = nullptr;
  checkDecoderResult(
    AImageDecoder_createFromBuffer(string.data(), string.size(), &decoder),
    "AImageDecoder_createFromBuffer"
  );

  const AImageDecoderHeaderInfo* info = AImageDecoder_getHeaderInfo(decoder);
  const int32_t width = AImageDecoderHeaderInfo_getWidth(info);
  const int32_t height = AImageDecoderHeaderInfo_getHeight(info);
  const auto format = static_cast<AndroidBitmapFormat>(
    AImageDecoderHeaderInfo_getAndroidBitmapFormat(info)
  );

  if (width <= 0 || height <= 0) {
    AImageDecoder_delete(decoder);
    throw std::runtime_error("AImageDecoder returned empty image dimensions");
  }

  if (format != ANDROID_BITMAP_FORMAT_RGBA_8888) {
    AImageDecoder_delete(decoder);
    throw std::runtime_error(
      "AImageDecoder returned an unsupported bitmap format"
    );
  }

  const size_t stride = AImageDecoder_getMinimumStride(decoder);
  const size_t buffer_size = stride * static_cast<size_t>(height);
  std::vector<uint8_t> pixels(buffer_size);

  const auto alpha_flags = static_cast<uint32_t>(
    AImageDecoderHeaderInfo_getAlphaFlags(info) &
    ANDROID_BITMAP_FLAGS_ALPHA_MASK
  );
  const bool is_premultiplied =
    alpha_flags != ANDROID_BITMAP_FLAGS_ALPHA_UNPREMUL;

  checkDecoderResult(
    AImageDecoder_decodeImage(decoder, pixels.data(), stride, buffer_size),
    "AImageDecoder_decodeImage"
  );
  AImageDecoder_delete(decoder);

  const Size size{static_cast<uint32_t>(width), static_cast<uint32_t>(height)};
  const size_t tight_stride = size.width * 4;

  auto copy_rows = [&](auto& image) {
    if (stride == tight_stride) {
      std::memcpy(image.data.get(), pixels.data(), buffer_size);
      return;
    }

    for (uint32_t y = 0; y < size.height; ++y) {
      std::memcpy(
        image.data.get() + y * tight_stride, pixels.data() + y * stride,
        tight_stride
      );
    }
  };

  if (is_premultiplied) {
    PremultipliedImage image(size);
    copy_rows(image);
    return image;
  }

  UnassociatedImage image(size);
  copy_rows(image);
  return util::premultiply(std::move(image));
}

}  // namespace mbgl
