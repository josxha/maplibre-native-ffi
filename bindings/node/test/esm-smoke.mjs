import { NativeBuffer } from "@maplibre/native-ffi-node";
import { NativePointer } from "@maplibre/native-ffi-node/render";
import { ResourceRequestHandle } from "@maplibre/native-ffi-node/resource";
import { RuntimeHandle } from "@maplibre/native-ffi-node/runtime";

if (
  !NativeBuffer ||
  !NativePointer ||
  !ResourceRequestHandle ||
  !RuntimeHandle
) {
  throw new Error("ESM named exports did not resolve");
}
