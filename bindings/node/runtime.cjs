"use strict";

const root = require("./index.cjs");

module.exports = {
  RuntimeHandle: root.RuntimeHandle,
  cVersion: root.cVersion,
  supportedRenderBackends: root.supportedRenderBackends,
  threadLastErrorMessage: root.threadLastErrorMessage,
  networkStatus: root.networkStatus,
  setNetworkStatus: root.setNetworkStatus,
};
