"use strict";

const root = require("./index.cjs");

exports.RuntimeHandle = root.RuntimeHandle;
exports.cVersion = root.cVersion;
exports.supportedRenderBackends = root.supportedRenderBackends;
exports.threadLastErrorMessage = root.threadLastErrorMessage;
exports.networkStatus = root.networkStatus;
exports.setNetworkStatus = root.setNetworkStatus;
