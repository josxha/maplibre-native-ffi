#include <stdexcept>

#include <mbgl/util/image.hpp>

namespace mbgl {

PremultipliedImage decodeImage(const std::string&) {
  throw std::runtime_error(
    "decodeImage is not yet implemented for the Android headless C API build"
  );
}

}  // namespace mbgl
