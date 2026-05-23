#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <memory>
#include <span>
#include <string>
#include <vector>

#include "maplibre_native_c.h"

namespace {

struct DartResourceRewriteRule {
  std::uint32_t kind;
  const char* url;
  const char* replacement_url;
};

struct DartResourceRewriteRules {
  const DartResourceRewriteRule* rules;
  std::size_t count;
};

struct DartResourceProviderRule {
  std::uint32_t kind;
  const char* url;
  mln_resource_response response;
};

struct DartResourceProviderRules {
  const DartResourceProviderRule* rules;
  std::size_t count;
};

struct DartQueuedResourceProviderRoute {
  std::uint32_t kind;
  const char* url;
};

using DartQueuedResourceRequestListener = void (*)(void* request);

struct DartQueuedResourceProvider {
  const DartQueuedResourceProviderRoute* routes;
  std::size_t route_count;
  DartQueuedResourceRequestListener listener;
};

struct DartQueuedResourceRequestView {
  void* owner;
  mln_resource_request_handle* handle;
  const char* url;
  std::uint32_t kind;
  std::uint32_t loading_method;
  std::uint32_t priority;
  std::uint32_t usage;
  std::uint32_t storage_policy;
  bool has_range;
  std::uint64_t range_start;
  std::uint64_t range_end;
  bool has_prior_modified;
  std::int64_t prior_modified_unix_ms;
  bool has_prior_expires;
  std::int64_t prior_expires_unix_ms;
  const char* prior_etag;
  const std::uint8_t* prior_data;
  std::size_t prior_data_size;
};

struct DartQueuedResourceRequest {
  DartQueuedResourceRequestView view{};
  std::string url;
  std::string prior_etag;
  std::vector<std::uint8_t> prior_data;
};

auto matches_rule(std::uint32_t rule_kind, std::uint32_t request_kind) -> bool {
  return rule_kind == MLN_RESOURCE_KIND_UNKNOWN || rule_kind == request_kind;
}

auto string_equals(const char* left, const char* right) -> bool {
  if (left == nullptr || right == nullptr) {
    return false;
  }
  return std::strcmp(left, right) == 0;
}

auto request_matches_route(
  std::span<const DartQueuedResourceProviderRoute> routes,
  const mln_resource_request& request
) -> bool {
  return std::ranges::any_of(routes, [&request](const auto& route) {
    return matches_rule(route.kind, request.kind) &&
           string_equals(route.url, request.url);
  });
}

auto copy_prior_data(const mln_resource_request& request)
  -> std::vector<std::uint8_t> {
  if (request.prior_data == nullptr || request.prior_data_size == 0) {
    return {};
  }
  auto data = std::vector<std::uint8_t>{};
  data.resize(request.prior_data_size);
  std::ranges::copy(
    std::span{request.prior_data, request.prior_data_size}, data.begin()
  );
  return data;
}

auto copy_request(
  const mln_resource_request& request, mln_resource_request_handle* handle
) -> DartQueuedResourceRequestView* {
  auto copy = std::make_unique<DartQueuedResourceRequest>();
  copy->url = request.url == nullptr ? std::string{} : std::string{request.url};
  copy->prior_etag = request.prior_etag == nullptr
                       ? std::string{}
                       : std::string{request.prior_etag};
  copy->prior_data = copy_prior_data(request);
  copy->view = DartQueuedResourceRequestView{
    .owner = copy.get(),
    .handle = handle,
    .url = copy->url.c_str(),
    .kind = request.kind,
    .loading_method = request.loading_method,
    .priority = request.priority,
    .usage = request.usage,
    .storage_policy = request.storage_policy,
    .has_range = request.has_range,
    .range_start = request.range_start,
    .range_end = request.range_end,
    .has_prior_modified = request.has_prior_modified,
    .prior_modified_unix_ms = request.prior_modified_unix_ms,
    .has_prior_expires = request.has_prior_expires,
    .prior_expires_unix_ms = request.prior_expires_unix_ms,
    .prior_etag = copy->prior_etag.empty() ? nullptr : copy->prior_etag.c_str(),
    .prior_data = copy->prior_data.empty() ? nullptr : copy->prior_data.data(),
    .prior_data_size = copy->prior_data.size(),
  };
  return &copy.release()->view;
}

void destroy_queued_request(DartQueuedResourceRequestView* request) noexcept {
  if (request == nullptr) {
    return;
  }
  auto* owner = static_cast<DartQueuedResourceRequest*>(request->owner);
  static_cast<void>(std::unique_ptr<DartQueuedResourceRequest>{owner});
}

}  // namespace

extern "C" MLN_API auto mln_dart_resource_transform_rewrite_callback(
  void* user_data, std::uint32_t kind, const char* url,
  mln_resource_transform_response* out_response
) noexcept -> mln_status {
  if (user_data == nullptr || url == nullptr || out_response == nullptr) {
    return MLN_STATUS_OK;
  }

  const auto& table = *static_cast<const DartResourceRewriteRules*>(user_data);
  for (const auto& rule : std::span{table.rules, table.count}) {
    if (matches_rule(rule.kind, kind) && string_equals(rule.url, url)) {
      out_response->url = rule.replacement_url;
      break;
    }
  }
  return MLN_STATUS_OK;
}

extern "C" MLN_API auto mln_dart_resource_provider_rules_callback(
  void* user_data, const mln_resource_request* request,
  mln_resource_request_handle* handle
) noexcept -> std::uint32_t {
  if (
    user_data == nullptr || request == nullptr || request->url == nullptr ||
    handle == nullptr
  ) {
    return MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
  }

  const auto& table = *static_cast<const DartResourceProviderRules*>(user_data);
  for (const auto& rule : std::span{table.rules, table.count}) {
    if (
      matches_rule(rule.kind, request->kind) &&
      string_equals(rule.url, request->url)
    ) {
      static_cast<void>(mln_resource_request_complete(handle, &rule.response));
      mln_resource_request_release(handle);
      return MLN_RESOURCE_PROVIDER_DECISION_HANDLE;
    }
  }
  return MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
}

extern "C" MLN_API auto mln_dart_queued_resource_provider_callback(
  void* user_data, const mln_resource_request* request,
  mln_resource_request_handle* handle
) noexcept -> std::uint32_t {
  if (
    user_data == nullptr || request == nullptr || request->url == nullptr ||
    handle == nullptr
  ) {
    return MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
  }

  const auto& provider =
    *static_cast<const DartQueuedResourceProvider*>(user_data);
  if (provider.listener == nullptr) {
    return MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
  }
  if (!request_matches_route(
        std::span{provider.routes, provider.route_count}, *request
      )) {
    return MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH;
  }

  try {
    auto* queued_request = copy_request(*request, handle);
    provider.listener(queued_request);
    return MLN_RESOURCE_PROVIDER_DECISION_HANDLE;
  } catch (...) {
    auto response = mln_resource_response{
      .size = sizeof(mln_resource_response),
      .status = MLN_RESOURCE_RESPONSE_STATUS_ERROR,
      .error_reason = MLN_RESOURCE_ERROR_REASON_OTHER,
      .bytes = nullptr,
      .byte_count = 0,
      .error_message = "resource provider request queue failed",
      .must_revalidate = false,
      .has_modified = false,
      .modified_unix_ms = 0,
      .has_expires = false,
      .expires_unix_ms = 0,
      .etag = nullptr,
      .has_retry_after = false,
      .retry_after_unix_ms = 0,
    };
    static_cast<void>(mln_resource_request_complete(handle, &response));
    mln_resource_request_release(handle);
    return MLN_RESOURCE_PROVIDER_DECISION_HANDLE;
  }
}

extern "C" MLN_API void mln_dart_resource_provider_request_destroy(
  void* request
) noexcept {
  destroy_queued_request(static_cast<DartQueuedResourceRequestView*>(request));
}
