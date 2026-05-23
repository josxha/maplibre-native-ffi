import { NativeBuffer } from "@maplibre/native-ffi-node";
import { NativePointer } from "@maplibre/native-ffi-node/render";

if (!NativeBuffer || !NativePointer) {
  throw new Error("ESM named exports did not resolve");
}
