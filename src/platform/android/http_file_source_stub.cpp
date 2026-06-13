#include <memory>
#include <utility>

#include <mbgl/storage/file_source.hpp>
#include <mbgl/storage/http_file_source.hpp>
#include <mbgl/storage/resource.hpp>
#include <mbgl/storage/resource_options.hpp>
#include <mbgl/storage/response.hpp>
#include <mbgl/util/async_request.hpp>
#include <mbgl/util/client_options.hpp>

namespace mbgl {

class HTTPFileSource::Impl {
 public:
  explicit Impl(ResourceOptions resource_options, ClientOptions client_options)
      : resource_options_(std::move(resource_options)),
        client_options_(std::move(client_options)) {}

  void setResourceOptions(ResourceOptions options) {
    resource_options_ = std::move(options);
  }

  ResourceOptions getResourceOptions() { return resource_options_.clone(); }

  void setClientOptions(ClientOptions options) {
    client_options_ = std::move(options);
  }

  ClientOptions getClientOptions() { return client_options_.clone(); }

 private:
  ResourceOptions resource_options_;
  ClientOptions client_options_;
};

class HTTPRequest : public AsyncRequest {
 public:
  explicit HTTPRequest(FileSource::Callback callback) {
    Response response;
    response.error = std::make_unique<Response::Error>(
      Response::Error::Reason::Other,
      "HTTP networking is not yet implemented for the Android headless C API "
      "build"
    );
    callback(response);
  }
};

HTTPFileSource::HTTPFileSource(
  const ResourceOptions& resource_options, const ClientOptions& client_options
)
    : impl(
        std::make_unique<Impl>(resource_options.clone(), client_options.clone())
      ) {}

HTTPFileSource::~HTTPFileSource() = default;

std::unique_ptr<AsyncRequest> HTTPFileSource::request(
  const Resource&, FileSource::Callback callback
) {
  return std::make_unique<HTTPRequest>(std::move(callback));
}

void HTTPFileSource::setResourceOptions(ResourceOptions options) {
  impl->setResourceOptions(std::move(options));
}

ResourceOptions HTTPFileSource::getResourceOptions() {
  return impl->getResourceOptions();
}

void HTTPFileSource::setClientOptions(ClientOptions options) {
  impl->setClientOptions(std::move(options));
}

ClientOptions HTTPFileSource::getClientOptions() {
  return impl->getClientOptions();
}

}  // namespace mbgl
