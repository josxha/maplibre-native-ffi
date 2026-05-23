import 'dart:async';

/// Retains native callback resources until queued and active Dart upcalls drain.
abstract base class RetainedCallbackState {
  var _activeUpcalls = 0;
  var _retired = false;
  var _closeScheduled = false;
  var _closed = false;

  /// Runs [body] as one active Dart upcall.
  void runUpcall(void Function() body) {
    if (_closed) {
      return;
    }
    _activeUpcalls += 1;
    try {
      body();
    } finally {
      _activeUpcalls -= 1;
      _scheduleCloseIfReady();
    }
  }

  /// Retires this callback state after queued and active upcalls drain.
  void close() {
    if (_retired) {
      return;
    }
    _retired = true;
    _scheduleCloseIfReady();
  }

  void _scheduleCloseIfReady() {
    if (!_retired || _closed || _closeScheduled || _activeUpcalls != 0) {
      return;
    }
    _closeScheduled = true;
    Timer.run(() {
      _closeScheduled = false;
      if (!_retired || _closed || _activeUpcalls != 0) {
        _scheduleCloseIfReady();
        return;
      }
      _closed = true;
      closeResources();
    });
  }

  /// Releases native resources after the state has retired and upcalls drained.
  void closeResources();
}
