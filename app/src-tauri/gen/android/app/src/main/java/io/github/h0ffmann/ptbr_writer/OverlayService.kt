package io.github.h0ffmann.ptbr_writer

import android.accessibilityservice.AccessibilityService
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.PixelFormat
import android.graphics.Rect
import android.graphics.RectF
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.text.SpannableString
import android.text.style.StrikethroughSpan
import android.util.Log
import android.util.TypedValue
import android.view.Gravity
import android.view.View
import android.view.WindowManager
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import java.util.concurrent.Executors

// The overlay: in any app, the text field being typed in is checked with Claude after a pause, its
// words underlined where a rule asks for a change, and a badge at the field's corner opens the cards
// (Aceitar, Ignorar). Password fields are never read; a paused app is never read. The check, the
// session and Aceitar's new text are Rust's (Ptbr); this only reads the field and draws.
class OverlayService : AccessibilityService() {
    companion object {
        const val TAG = "ptbr"
        const val PAUSE_MS = 1500L
        const val MIN_WORDS = 3
        // Above this the extension sends only the cursor's paragraph; the overlay sends nothing.
        const val MAX_CHARS = 4000
        private const val PREFS = "overlay"
        private const val PAUSED = "paused"

        // Whether the user turned the service on in Android's accessibility settings.
        fun enabled(ctx: Context): Boolean {
            val me = ComponentName(ctx, OverlayService::class.java)
            val on = Settings.Secure.getString(ctx.contentResolver, Settings.Secure.ENABLED_ACCESSIBILITY_SERVICES) ?: return false
            return on.split(':').any { ComponentName.unflattenFromString(it) == me }
        }
    }

    private val main = Handler(Looper.getMainLooper())
    private val worker = Executors.newSingleThreadExecutor()
    private lateinit var wm: WindowManager
    private lateinit var lines: Underlines
    private var badge: TextView? = null
    private var panel: View? = null

    private var field: AccessibilityNodeInfo? = null
    // The field's text at its last change: a spell checker's spans fire changes with the same text.
    private var seen: String? = null
    private var checked: Checked? = null
    private var error: String? = null
    // A reply for an older text than the field's is dropped.
    private var generation = 0

    private val checkLater = Runnable { checkNow() }
    private val redrawLater = Runnable { redraw() }

    override fun onServiceConnected() {
        wm = getSystemService(WINDOW_SERVICE) as WindowManager
        lines = Underlines(this)
        wm.addView(lines, params(WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.MATCH_PARENT, touch = false))
        Log.i(TAG, "overlay: on")
    }

    override fun onDestroy() {
        hide()
        if (::lines.isInitialized) wm.removeView(lines)
        worker.shutdownNow()
        super.onDestroy()
    }

    override fun onInterrupt() {}

    override fun onAccessibilityEvent(e: AccessibilityEvent) {
        // The overlay's own windows and the app's window are not someone's text.
        if (e.packageName?.toString() == packageName) return
        when (e.eventType) {
            AccessibilityEvent.TYPE_VIEW_TEXT_CHANGED, AccessibilityEvent.TYPE_VIEW_FOCUSED -> onField(e)
            AccessibilityEvent.TYPE_VIEW_SCROLLED, AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED -> {
                if (checked != null) {
                    main.removeCallbacks(redrawLater)
                    main.postDelayed(redrawLater, 80)
                }
            }
            AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED -> {
                if (e.packageName?.toString() != field?.packageName?.toString()) forget()
            }
        }
    }

    private fun onField(e: AccessibilityEvent) {
        val n = e.source ?: return
        if (!n.isEditable || n.isPassword || paused(n.packageName?.toString())) {
            if (e.eventType == AccessibilityEvent.TYPE_VIEW_FOCUSED) forget()
            return
        }
        val text = textOf(n)
        val same = field?.let { it == n } ?: false
        field = n
        // Aceitar's own change, or the focus back on a checked field: the cards still hold.
        if (same && checked?.text == text) {
            seen = text
            redraw()
            return
        }
        // Not a change of the text: the wait (or the check under way) goes on.
        if (same && seen == text) return
        seen = text
        drop()
        main.removeCallbacks(checkLater)
        if (words(text) >= MIN_WORDS && text.length <= MAX_CHARS) main.postDelayed(checkLater, PAUSE_MS)
    }

    private fun checkNow() {
        val n = field ?: return
        n.refresh()
        val text = textOf(n)
        if (words(text) < MIN_WORDS) return
        val gen = ++generation
        val dir = applicationContext.dataDir.absolutePath
        worker.execute {
            val reply = parseReply(Ptbr.check(dir, text))
            main.post { if (gen == generation) shown(reply, text) }
        }
    }

    private fun shown(reply: Reply, sent: String) {
        val n = field ?: return
        n.refresh()
        if (textOf(n) != sent) return
        when (reply) {
            is Reply.Err -> {
                checked = null
                error = reply.message
                Log.i(TAG, "overlay: error ${reply.message}")
            }
            is Reply.Ok -> {
                checked = reply.checked
                error = null
                // scripts/app-android.sh waits for this line: the overlay ran its check end to end.
                Log.i(TAG, "overlay: ${reply.checked.cards.size} cards")
            }
        }
        redraw()
    }

    private fun act(reply: String, writes: Boolean) {
        val r = parseReply(reply)
        if (r !is Reply.Ok) return
        checked = r.checked
        if (writes) {
            val args = Bundle()
            args.putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, r.checked.text)
            field?.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
        }
        // The field moves after a new text: draw once it has laid out.
        main.postDelayed(redrawLater, 120)
        if (panel != null) openPanel()
    }

    // The underlines, the badge and the open panel, from `checked` (or `error`) and the field.
    private fun redraw() {
        val n = field ?: return hide()
        n.refresh()
        val c = checked
        if (c == null && error == null) return hide()
        if (c != null && (c.cards.isEmpty() || textOf(n) != c.text)) return hide()
        val bounds = Rect().also { n.getBoundsInScreen(it) }
        lines.clip = bounds
        lines.lines = c?.marks?.flatMap { m ->
            val color = if (c.cards.getOrNull(m.card)?.severity == "aviso") AVISO else ERRO
            rects(n, m.start, m.end).map { it to color }
        } ?: emptyList()
        showBadge(bounds, c?.cards?.size)
    }

    private fun showBadge(bounds: Rect, count: Int?) {
        val size = dp(30)
        val b = badge ?: TextView(this).also { t ->
            t.gravity = Gravity.CENTER
            t.setTextColor(Color.WHITE)
            t.typeface = Typeface.DEFAULT_BOLD
            t.setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
            t.setOnClickListener { if (panel == null) openPanel() else closePanel() }
            wm.addView(t, params(size, size, touch = true))
            badge = t
        }
        b.text = count?.toString() ?: "!"
        b.background = GradientDrawable().apply {
            shape = GradientDrawable.OVAL
            setColor(if (count == null) ERRO else PEN)
        }
        val p = b.layoutParams as WindowManager.LayoutParams
        val screen = resources.displayMetrics
        p.x = (bounds.right - size - dp(4)).coerceIn(0, screen.widthPixels - size)
        p.y = (bounds.bottom - size - dp(4)).coerceIn(0, screen.heightPixels - size)
        wm.updateViewLayout(b, p)
        Log.i(TAG, "overlay: badge at ${p.x + size / 2},${p.y + size / 2}")
    }

    private fun openPanel() {
        val c = checked
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(dp(14), dp(10), dp(14), dp(12)) }
        val head = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; gravity = Gravity.CENTER_VERTICAL }
        head.addView(text(if (c == null) "ptbr-writer" else "ptbr-writer · ${c.cards.size} correções", 15f, INK, bold = true),
            LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f))
        head.addView(button("×", primary = false) { closePanel() })
        list.addView(head)
        if (c == null) {
            list.addView(text(error ?: "", 14f, ERRO))
            list.addView(button("Abrir ptbr-writer", primary = true) {
                closePanel()
                startActivity(Intent(this, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
            })
        }
        c?.cards?.forEach { list.addView(cardView(it)) }
        list.addView(button("Pausar neste app", primary = false) { pauseHere() })
        val scroll = ScrollView(this).apply {
            addView(list)
            background = GradientDrawable().apply { setColor(PAPER); cornerRadius = dp(14).toFloat(); setStroke(dp(1), RULE) }
        }
        val screen = resources.displayMetrics
        val bounds = Rect().also { field?.getBoundsInScreen(it) }
        // Away from the field: above it when it is in the lower half (a chat's box), else below it, in
        // the room between the field and the screen's edge.
        val p = params(WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.WRAP_CONTENT, touch = true)
        val above = bounds.centerY() > screen.heightPixels / 2
        p.gravity = if (above) Gravity.TOP else Gravity.BOTTOM
        p.y = dp(32)
        val room = (if (above) bounds.top else screen.heightPixels - bounds.bottom) - p.y - dp(8)
        p.horizontalMargin = 0.03f
        closePanel()
        scroll.isVerticalScrollBarEnabled = true
        wm.addView(scroll, p)
        // At most 45% of the screen, and never over the field: the rest scrolls.
        scroll.post {
            val max = minOf((screen.heightPixels * 0.45).toInt(), room).coerceAtLeast(dp(120))
            if (scroll.height > max) {
                p.height = max
                wm.updateViewLayout(scroll, p)
            }
        }
        panel = scroll
    }

    private fun cardView(card: Card): View {
        val v = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dp(12), dp(8), dp(12), dp(10))
            background = GradientDrawable().apply { setColor(SHEET); cornerRadius = dp(10).toFloat(); setStroke(dp(1), if (card.severity == "aviso") AVISO else ERRO) }
        }
        v.addView(text("${card.rule}  ${card.title}", 12f, REVIEW, bold = true))
        v.addView(text(SpannableString(card.original).apply { setSpan(StrikethroughSpan(), 0, length, 0) }, 14f, REVIEW))
        if (!card.detectOnly) v.addView(text(card.replacement.ifEmpty { "(remover)" }, 14f, PEN))
        val actions = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL }
        if (!card.detectOnly) actions.addView(button("Aceitar", primary = true) { act(Ptbr.accept(card.id), writes = true) })
        actions.addView(button("Ignorar", primary = false) { act(Ptbr.ignore(card.id), writes = false) })
        v.addView(actions)
        return v.also {
            it.layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT).apply { topMargin = dp(8) }
        }
    }

    private fun closePanel() {
        panel?.let { wm.removeView(it) }
        panel = null
    }

    private fun pauseHere() {
        val pkg = field?.packageName?.toString() ?: return
        val prefs = getSharedPreferences(PREFS, MODE_PRIVATE)
        prefs.edit().putStringSet(PAUSED, (prefs.getStringSet(PAUSED, emptySet()) ?: emptySet()) + pkg).apply()
        forget()
    }

    private fun paused(pkg: String?): Boolean =
        pkg != null && getSharedPreferences(PREFS, MODE_PRIVATE).getStringSet(PAUSED, emptySet())?.contains(pkg) == true

    // The cards no longer match the field.
    private fun drop() {
        generation++
        checked = null
        error = null
        Ptbr.clear()
        hide()
    }

    // The field is gone (another app, a field that is not text).
    private fun forget() {
        main.removeCallbacks(checkLater)
        field = null
        seen = null
        drop()
    }

    private fun hide() {
        if (::lines.isInitialized) lines.lines = emptyList()
        badge?.let { wm.removeView(it) }
        badge = null
        closePanel()
    }

    // The screen rects of text[start, end), one per line: Android 8's character locations.
    private fun rects(n: AccessibilityNodeInfo, start: Int, end: Int): List<RectF> {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O || end <= start) return emptyList()
        val key = AccessibilityNodeInfo.EXTRA_DATA_TEXT_CHARACTER_LOCATION_KEY
        val args = Bundle().apply {
            putInt(AccessibilityNodeInfo.EXTRA_DATA_TEXT_CHARACTER_LOCATION_ARG_START_INDEX, start)
            putInt(AccessibilityNodeInfo.EXTRA_DATA_TEXT_CHARACTER_LOCATION_ARG_LENGTH, end - start)
        }
        if (!n.refreshWithExtraData(key, args)) return emptyList()
        val chars: List<RectF?> = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            n.extras.getParcelableArray(key, RectF::class.java)?.toList() ?: emptyList()
        } else {
            @Suppress("DEPRECATION")
            n.extras.getParcelableArray(key)?.map { it as? RectF } ?: emptyList()
        }
        val out = mutableListOf<RectF>()
        for (r in chars) {
            if (r == null || r.width() <= 0f) continue
            val last = out.lastOrNull()
            if (last != null && kotlin.math.abs(last.bottom - r.bottom) < dp(2) && r.left >= last.left) last.union(r) else out.add(RectF(r))
        }
        return out
    }

    private fun textOf(n: AccessibilityNodeInfo): String {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && n.isShowingHintText) return ""
        return n.text?.toString() ?: ""
    }

    private fun words(s: String) = s.split(Regex("\\s+")).count { it.isNotEmpty() }

    private fun params(w: Int, h: Int, touch: Boolean) = WindowManager.LayoutParams(
        w, h,
        WindowManager.LayoutParams.TYPE_ACCESSIBILITY_OVERLAY,
        WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN or
            WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS or (if (touch) 0 else WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE),
        PixelFormat.TRANSLUCENT,
    ).apply { gravity = Gravity.TOP or Gravity.START }

    private fun text(s: CharSequence, sp: Float, color: Int, bold: Boolean = false) = TextView(this).apply {
        text = s
        setTextColor(color)
        setTextSize(TypedValue.COMPLEX_UNIT_SP, sp)
        if (bold) typeface = Typeface.DEFAULT_BOLD
        setPadding(0, dp(2), 0, dp(2))
    }

    private fun button(label: String, primary: Boolean, onClick: () -> Unit) = Button(this).apply {
        text = label
        isAllCaps = false
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
        setTextColor(if (primary) Color.WHITE else PEN)
        minHeight = 0
        minimumHeight = 0
        minWidth = 0
        minimumWidth = 0
        setPadding(dp(12), dp(6), dp(12), dp(6))
        background = GradientDrawable().apply {
            cornerRadius = dp(8).toFloat()
            if (primary) setColor(PEN) else { setColor(Color.TRANSPARENT); setStroke(dp(1), RULE) }
        }
        setOnClickListener { onClick() }
        layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT).apply { marginEnd = dp(8); topMargin = dp(6) }
    }

    private fun dp(v: Int) = (v * resources.displayMetrics.density).toInt()
}

// The window's tokens (app/ui/static/style.css).
private val PAPER = Color.parseColor("#fbf8f1")
private val SHEET = Color.parseColor("#ffffff")
private val RULE = Color.parseColor("#e4ddcd")
private val INK = Color.parseColor("#1b1a17")
private val PEN = Color.parseColor("#1d3fd0")
private val REVIEW = Color.parseColor("#c8102e")
private val ERRO = Color.parseColor("#d93025")
private val AVISO = Color.parseColor("#e8a400")

// The underlines over every app: draws, never takes a touch.
private class Underlines(ctx: Context) : View(ctx) {
    var clip: Rect? = null
    var lines: List<Pair<RectF, Int>> = emptyList()
        set(v) {
            field = v
            invalidate()
        }
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE
        strokeCap = Paint.Cap.ROUND
        strokeWidth = 2.5f * ctx.resources.displayMetrics.density
    }
    private val at = IntArray(2)

    override fun onDraw(c: Canvas) {
        getLocationOnScreen(at)
        val box = clip
        for ((r, color) in lines) {
            if (box != null && (r.bottom < box.top || r.top > box.bottom)) continue
            paint.color = color
            val y = r.bottom - at[1]
            c.drawLine(r.left - at[0], y, r.right - at[0], y, paint)
        }
    }
}
