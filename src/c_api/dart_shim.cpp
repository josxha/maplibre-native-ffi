#include <cstddef>
#include <cstdint>
#include <cstring>
#include <span>

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

auto matches_rule(std::uint32_t rule_kind, std::uint32_t request_kind) -> bool {
  return rule_kind == MLN_RESOURCE_KIND_UNKNOWN || rule_kind == request_kind;
}

auto string_equals(const char* left, const char* right) -> bool {
  if (left == nullptr || right == nullptr) {
    return false;
  }
  return std::strcmp(left, right) == 0;
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
