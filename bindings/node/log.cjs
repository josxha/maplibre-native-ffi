"use strict";

const root = require("./index.cjs");

module.exports = {
  setLogCallback: root.setLogCallback,
  clearLogCallback: root.clearLogCallback,
  setAsyncLogSeverities: root.setAsyncLogSeverities,
  restoreDefaultAsyncLogSeverities: root.restoreDefaultAsyncLogSeverities,
};
