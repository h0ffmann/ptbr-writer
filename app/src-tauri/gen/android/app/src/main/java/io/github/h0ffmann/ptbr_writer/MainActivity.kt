package io.github.h0ffmann.ptbr_writer

import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }

  // The window (app/ui, wasm) reads `window.ptbrAndroid`: whether the overlay is on, and a way to
  // Android's accessibility settings, where only the user can turn it on.
  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(Bridge(), "ptbrAndroid")
  }

  inner class Bridge {
    @JavascriptInterface
    fun overlayOn(): Boolean = OverlayService.enabled(this@MainActivity)

    @JavascriptInterface
    fun openSettings() {
      startActivity(Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    }
  }
}
