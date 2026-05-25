part of 'runtime.dart';

final class _NativeResourceRewriteRules extends Struct {
  external Pointer<_NativeResourceRewriteRule> rules;

  @Size()
  external int count;
}

final class _NativeResourceRewriteRule extends Struct {
  @Uint32()
  external int kind;

  external Pointer<Char> url;

  external Pointer<Char> replacementUrl;
}

final class _ResourceTransformState {
  _ResourceTransformState(List<ResourceUrlRewriteRule> rules) {
    for (final rule in rules) {
      _checkNativeCString(rule.url);
      _checkNativeCString(rule.replacementUrl);
    }
    pointer = calloc<_NativeResourceRewriteRules>();
    pointer.ref.count = rules.length;
    pointer.ref.rules = rules.isEmpty
        ? nullptr.cast<_NativeResourceRewriteRule>()
        : calloc<_NativeResourceRewriteRule>(rules.length);
    for (var index = 0; index < rules.length; index += 1) {
      final rule = rules[index];
      pointer.ref.rules[index].kind =
          rule.kind?.rawValue ?? _resourceKindWildcard;
      pointer.ref.rules[index].url = _nativeOwnedCString(rule.url);
      pointer.ref.rules[index].replacementUrl = _nativeOwnedCString(
        rule.replacementUrl,
      );
    }
  }

  late final Pointer<_NativeResourceRewriteRules> pointer;

  void close() {
    final rules = pointer.ref.rules;
    for (var index = 0; index < pointer.ref.count; index += 1) {
      calloc.free(rules[index].url);
      calloc.free(rules[index].replacementUrl);
    }
    if (rules != nullptr) {
      calloc.free(rules);
    }
    calloc.free(pointer);
  }
}

final class _NativeResourceProviderRules extends Struct {
  external Pointer<_NativeResourceProviderRule> rules;

  @Size()
  external int count;
}

final class _NativeResourceProviderRule extends Struct {
  @Uint32()
  external int kind;

  external Pointer<Char> url;

  external raw.mln_resource_response response;
}

typedef _QueuedResourceRequestListenerFunction = Void Function(Pointer<Void>);

final class _NativeQueuedResourceProviderRoute extends Struct {
  @Uint32()
  external int kind;

  external Pointer<Char> url;
}

final class _NativeQueuedResourceProvider extends Struct {
  external Pointer<_NativeQueuedResourceProviderRoute> routes;

  @Size()
  external int routeCount;

  external Pointer<NativeFunction<_QueuedResourceRequestListenerFunction>>
  listener;
}

final class _NativeQueuedResourceRequest extends Struct {
  external Pointer<Void> owner;

  external Pointer<raw.mln_resource_request_handle> handle;

  external Pointer<Char> url;

  @Uint32()
  external int kind;

  @Uint32()
  external int loadingMethod;

  @Uint32()
  external int priority;

  @Uint32()
  external int usage;

  @Uint32()
  external int storagePolicy;

  @Bool()
  external bool hasRange;

  @Uint64()
  external int rangeStart;

  @Uint64()
  external int rangeEnd;

  @Bool()
  external bool hasPriorModified;

  @Int64()
  external int priorModifiedUnixMs;

  @Bool()
  external bool hasPriorExpires;

  @Int64()
  external int priorExpiresUnixMs;

  external Pointer<Char> priorEtag;

  external Pointer<Uint8> priorData;

  @Size()
  external int priorDataSize;
}

final class _ResourceProviderRulesState {
  _ResourceProviderRulesState(List<ResourceProviderRule> rules) {
    for (final rule in rules) {
      _checkNativeCString(rule.url);
      _checkResourceResponseNativeStrings(rule.response);
    }
    pointer = calloc<_NativeResourceProviderRules>();
    pointer.ref.count = rules.length;
    pointer.ref.rules = rules.isEmpty
        ? nullptr.cast<_NativeResourceProviderRule>()
        : calloc<_NativeResourceProviderRule>(rules.length);
    for (var index = 0; index < rules.length; index += 1) {
      final rule = rules[index];
      pointer.ref.rules[index].kind =
          rule.kind?.rawValue ?? _resourceKindWildcard;
      pointer.ref.rules[index].url = _nativeOwnedCString(rule.url);
      pointer.ref.rules[index].response = _resourceResponseToNative(
        rule.response,
        calloc,
      );
    }
  }

  late final Pointer<_NativeResourceProviderRules> pointer;

  void close() {
    final rules = pointer.ref.rules;
    for (var index = 0; index < pointer.ref.count; index += 1) {
      calloc.free(rules[index].url);
      _freeNativeResourceResponse(rules[index].response, calloc);
    }
    if (rules != nullptr) {
      calloc.free(rules);
    }
    calloc.free(pointer);
  }
}

final class _ResourceProviderCallbackState extends RetainedCallbackState {
  _ResourceProviderCallbackState(ResourceProvider provider) {
    for (final route in provider.routes) {
      _checkNativeCString(route.url);
    }
    callback = NativeCallable<_QueuedResourceRequestListenerFunction>.listener((
      Pointer<Void> request,
    ) {
      final ran = runUpcall(
        () => _invokeQueuedResourceProvider(provider.callback, request),
      );
      if (!ran) {
        _dropQueuedResourceProviderRequest(request);
      }
    });
    pointer = calloc<_NativeQueuedResourceProvider>();
    pointer.ref.routeCount = provider.routes.length;
    pointer.ref.routes = provider.routes.isEmpty
        ? nullptr.cast<_NativeQueuedResourceProviderRoute>()
        : calloc<_NativeQueuedResourceProviderRoute>(provider.routes.length);
    for (var index = 0; index < provider.routes.length; index += 1) {
      final route = provider.routes[index];
      pointer.ref.routes[index].kind =
          route.kind?.rawValue ?? _resourceKindWildcard;
      pointer.ref.routes[index].url = _nativeOwnedCString(route.url);
    }
    pointer.ref.listener = callback.nativeFunction;
  }

  late final Pointer<_NativeQueuedResourceProvider> pointer;
  late final NativeCallable<_QueuedResourceRequestListenerFunction> callback;

  @override
  void closeResources() {
    final routes = pointer.ref.routes;
    for (var index = 0; index < pointer.ref.routeCount; index += 1) {
      calloc.free(routes[index].url);
    }
    if (routes != nullptr) {
      calloc.free(routes);
    }
    calloc.free(pointer);
    callback.close();
  }
}

void _dropQueuedResourceProviderRequest(Pointer<Void> rawRequest) {
  try {
    final request = rawRequest.cast<_NativeQueuedResourceRequest>().ref;
    final handle = ResourceRequestHandle._(request.handle);
    if (!handle.isReleased) {
      try {
        handle.complete(
          const ResourceResponse(
            status: ResourceResponseStatus.error,
            errorReason: ResourceErrorReason.other,
            errorMessage: 'Dart resource provider callback was retired',
          ),
        );
      } catch (_) {
        handle.close();
      }
    }
  } finally {
    _c.dartResourceProviderRequestDestroy(rawRequest);
  }
}

void _invokeQueuedResourceProvider(
  ResourceProviderCallback callback,
  Pointer<Void> rawRequest,
) {
  try {
    final request = rawRequest.cast<_NativeQueuedResourceRequest>().ref;
    final handle = ResourceRequestHandle._(request.handle);
    try {
      callback(_copyResourceRequest(request), handle);
    } catch (_) {
      if (!handle.isReleased) {
        try {
          handle.complete(
            const ResourceResponse(
              status: ResourceResponseStatus.error,
              errorReason: ResourceErrorReason.other,
              errorMessage: 'Dart resource provider callback threw',
            ),
          );
        } catch (_) {
          handle.close();
        }
      }
    }
  } finally {
    _c.dartResourceProviderRequestDestroy(rawRequest);
  }
}

ResourceRequest _copyResourceRequest(_NativeQueuedResourceRequest request) {
  final priorData = request.priorData == nullptr || request.priorDataSize == 0
      ? null
      : Uint8List.fromList(
          request.priorData.asTypedList(request.priorDataSize),
        );
  return ResourceRequest(
    url: request.url.cast<Utf8>().toDartString(),
    kind: ResourceKind.fromRawValue(request.kind),
    loadingMethod: ResourceLoadingMethod.fromRawValue(request.loadingMethod),
    priority: ResourcePriority.fromRawValue(request.priority),
    usage: ResourceUsage.fromRawValue(request.usage),
    storagePolicy: ResourceStoragePolicy.fromRawValue(request.storagePolicy),
    range: request.hasRange
        ? (start: request.rangeStart, end: request.rangeEnd)
        : null,
    priorModifiedUnixMs: request.hasPriorModified
        ? request.priorModifiedUnixMs
        : null,
    priorExpiresUnixMs: request.hasPriorExpires
        ? request.priorExpiresUnixMs
        : null,
    priorEtag: request.priorEtag == nullptr
        ? null
        : request.priorEtag.cast<Utf8>().toDartString(),
    priorData: priorData,
  );
}

/// Runtime-owned offline operation handle.
