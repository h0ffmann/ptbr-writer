package io.github.h0ffmann.ptbr_writer

import android.content.Intent
import android.os.Bundle
import android.provider.Settings
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // Edge to edge (Android 15 forces it): the window keeps clear of the status and navigation bars and
    // of the keyboard, so the title and the text field stay in view.
    ViewCompat.setOnApplyWindowInsetsListener(findViewById(android.R.id.content)) { v, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.ime())
      v.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }
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
